//! Cycles top-up of personal canisters, end to end on PocketIC.
//!
//! Covers what unit tests cannot: the call from the factory to a canister it
//! does not control, the deposit, the monthly cap across a month change, the
//! hourly timer (also after a factory upgrade with saved state), and the
//! rescue of a frozen canister.
//!
//! Build the wasms first, then run from this directory:
//!
//!   cargo build --target wasm32-unknown-unknown --release -p backend -p canister_factory
//!   cargo test --bin top_up_pocket_ic -- --nocapture
//!
//! PocketIC starts its own server process; a running dfx replica is not touched.

#[path = "common.rs"]
mod common;

use candid::{decode_one, encode_args, encode_one, CandidType, Nat, Principal};
use common::{install_backend, wasm, T};
use flate2::{write::GzEncoder, Compression};
use ic_cdk::management_canister::CanisterSettings;
use pocket_ic::PocketIc;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::time::Duration;

const HOUR: Duration = Duration::from_secs(60 * 60);
const DAY: Duration = Duration::from_secs(24 * 60 * 60);

// ---- the parts of the two interfaces this test uses -------------------------

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
enum PersonalStatus {
    Pending,
    Created,
    Installed,
    Ready,
}

#[derive(CandidType, Deserialize, Clone, Debug, Default)]
struct TopUpRecord {
    last_balance: Option<u128>,
    last_checked_at: Option<u64>,
    last_top_up_cycles: Option<u128>,
    last_top_up_at: Option<u64>,
    month: Option<u32>,
    month_total: Option<u128>,
    monthly_cap: Option<u128>,
    last_error: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct PersonalCanister {
    owner: Principal,
    canister_id: Option<Principal>,
    status: PersonalStatus,
    top_up: Option<TopUpRecord>,
}

#[derive(CandidType, Deserialize, Clone, Copy, Debug, PartialEq)]
struct TopUpConfig {
    enabled: bool,
    floor: u128,
    target: u128,
    monthly_cap: u128,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct TopUpReport {
    owner: Principal,
    canister_id: Option<Principal>,
    balance: Option<u128>,
    sent: u128,
    note: String,
}

#[derive(CandidType, Deserialize)]
struct UploadCommit {
    expected_sha256_hex: String,
}

type Res<T> = Result<T, String>;

// ---- helpers ----------------------------------------------------------------

fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut enc = GzEncoder::new(Vec::new(), Compression::default());
    enc.write_all(bytes).unwrap();
    enc.finish().unwrap()
}

struct World {
    pic: PocketIc,
    factory: Principal,
    admin: Principal,
    owner: Principal,
    stranger: Principal,
    /// The owner's personal canister
    pc: Principal,
}

impl World {
    fn update<A: candid::utils::ArgumentEncoder, R: CandidType + for<'a> Deserialize<'a>>(
        &self,
        canister: Principal,
        sender: Principal,
        method: &str,
        args: A,
    ) -> Result<R, String> {
        let raw = self
            .pic
            .update_call(canister, sender, method, encode_args(args).unwrap())
            .map_err(|e| format!("{method} rejected: {}", e.reject_message))?;
        Ok(decode_one(&raw).unwrap_or_else(|e| panic!("{method}: reply not understood: {e}")))
    }

    fn query<R: CandidType + for<'a> Deserialize<'a>>(
        &self,
        canister: Principal,
        sender: Principal,
        method: &str,
    ) -> Result<R, String> {
        let raw = self
            .pic
            .query_call(canister, sender, method, encode_args(()).unwrap())
            .map_err(|e| format!("{method} rejected: {}", e.reject_message))?;
        Ok(decode_one(&raw).unwrap_or_else(|e| panic!("{method}: reply not understood: {e}")))
    }

    fn check(&self, sender: Principal) -> Result<Res<TopUpReport>, String> {
        self.update(self.factory, sender, "top_up_check", (self.owner,))
    }

    /// A check by the admin that must go through
    fn check_ok(&self) -> TopUpReport {
        self.check(self.admin).unwrap().unwrap()
    }

    fn set_config(&self, enabled: bool, floor: u128, target: u128, monthly_cap: u128) {
        let cfg = TopUpConfig {
            enabled,
            floor,
            target,
            monthly_cap,
        };
        let r: Res<()> = self
            .update(self.factory, self.admin, "top_up_config_set", (cfg,))
            .unwrap();
        r.unwrap();
    }

    fn record(&self) -> TopUpRecord {
        let e: Option<PersonalCanister> = self
            .query(self.factory, self.owner, "my_personal_canister")
            .unwrap();
        e.expect("registry entry").top_up.unwrap_or_default()
    }

    fn liquid(&self) -> u128 {
        self.query(self.pc, self.owner, "cycles_liquid_balance").unwrap()
    }

    /// Release the backend wasm (gzipped, in chunks), as
    /// scripts/factory-release.sh does
    fn release_backend(&self) {
        let (factory, admin) = (self.factory, self.admin);
        let gz = gzip(&wasm("backend"));
        let upload: Res<u64> = self.update(factory, admin, "create_upload", ()).unwrap();
        let upload = upload.unwrap();
        for chunk in gz.chunks(900_000) {
            let r: Res<u64> = self
                .update(factory, admin, "put_chunk", (upload, chunk.to_vec()))
                .unwrap();
            r.unwrap();
        }
        let commit = UploadCommit {
            expected_sha256_hex: hex::encode(Sha256::digest(&gz)),
        };
        let r: Res<String> = self
            .update(factory, admin, "commit_upload", (upload, commit))
            .unwrap();
        r.unwrap();
        // The reply (release info) is not needed here
        self.pic
            .update_call(
                factory,
                admin,
                "release_set",
                encode_args((upload, "test".to_string())).unwrap(),
            )
            .expect("release_set");
    }

    /// Let time pass and the timers run
    fn pass(&self, d: Duration) {
        self.pic.advance_time(d);
        for _ in 0..10 {
            self.pic.tick();
        }
    }
}

/// Init argument of the factory
#[derive(CandidType)]
struct FactoryInitArg {
    min_factory_cycles: Option<u128>,
    allowlist: Option<Vec<Principal>>,
    admins: Option<Vec<Principal>>,
    max_upload_size: Option<u64>,
    upload_ttl_seconds: Option<u64>,
}

/// A factory with a released backend and one owner's personal canister
fn setup() -> World {
    setup_with(200 * T, None)
}

/// The same, for a factory that holds `factory_cycles` and must keep
/// `min_factory_cycles` (none: its default)
fn setup_with(factory_cycles: u128, min_factory_cycles: Option<u128>) -> World {
    let pic = PocketIc::new();
    let admin = Principal::from_slice(&[1; 29]);
    let owner = Principal::from_slice(&[2; 29]);
    let stranger = Principal::from_slice(&[3; 29]);

    // The factory: installed by the admin, who thereby becomes its admin
    let factory = pic.create_canister_with_settings(Some(admin), None);
    pic.add_cycles(factory, factory_cycles);
    let init = FactoryInitArg {
        min_factory_cycles,
        allowlist: None,
        admins: None,
        max_upload_size: None,
        upload_ttl_seconds: None,
    };
    pic.install_canister(
        factory,
        wasm("canister_factory"),
        encode_one(Some(init)).unwrap(),
        Some(admin),
    );

    let mut w = World {
        pic,
        factory,
        admin,
        owner,
        stranger,
        pc: Principal::anonymous(),
    };
    // Keep the timer quiet until a test asks for it
    w.set_config(false, T, 2 * T, 3 * T);
    w.release_backend();

    // One credit, one personal canister
    let r: Res<()> = w
        .update(factory, admin, "personal_credit_set", (owner, 1u32))
        .unwrap();
    r.unwrap();
    let created: Res<PersonalCanister> = w
        .update(factory, owner, "create_personal_canister", ())
        .unwrap();
    let created = created.expect("create_personal_canister");
    assert_eq!(created.status, PersonalStatus::Ready);
    w.pc = created.canister_id.expect("canister id");
    w
}

/// Equal apart from what a few calls cost the canister (well under 0.001T)
fn assert_near(got: u128, want: u128, what: &str) {
    assert!(got.abs_diff(want) < T / 1_000, "{what}: got {got}, want {want}");
}

// ---- the test ---------------------------------------------------------------

#[test]
fn top_up_end_to_end() {
    let w = setup();
    let (pic, factory, pc) = (&w.pic, w.factory, w.pc);

    // --- who may read the cycles -------------------------------------------
    assert_eq!(
        pic.get_controllers(pc),
        vec![w.owner],
        "after the hand-off the owner is the only controller"
    );
    let known: Option<Principal> = w.query(pc, w.stranger, "personal_factory").unwrap();
    assert_eq!(known, Some(factory), "the canister remembers who installed it");

    let total: u128 = w.query(pc, w.owner, "cycles_balance").unwrap();
    let liquid = w.liquid();
    assert!(liquid > T && liquid < total, "liquid {liquid}, total {total}");
    assert_eq!(total, pic.cycle_balance(pc));
    let r: Result<u128, String> = w.query(pc, w.stranger, "cycles_liquid_balance");
    assert!(r.unwrap_err().contains("owner and the factory"));
    let r: Result<u128, String> = w.query(pc, w.stranger, "cycles_balance");
    assert!(r.is_err());
    println!("ok: owner reads the cycles, a stranger does not (liquid {liquid}, total {total})");

    // --- who may ask the factory to check ----------------------------------
    let r = w.check(w.stranger).unwrap();
    assert!(r.unwrap_err().contains("Admin access required"));
    let r: Res<Vec<TopUpReport>> = w
        .update(factory, w.owner, "top_up_check_all", ())
        .unwrap();
    assert!(r.is_err(), "checking everybody is for admins");
    let r: Res<()> = w
        .update(factory, w.owner, "top_up_cap_set", (w.owner, Some(100 * T)))
        .unwrap();
    assert!(r.is_err(), "an owner cannot raise their own cap");

    // --- enough: nothing is sent -------------------------------------------
    let rep = w.check_ok();
    assert_eq!((rep.sent, rep.note.as_str()), (0, "enough"));
    // answering the factory's call cost the canister a little
    let reported = rep.balance.unwrap();
    assert!(reported <= liquid && liquid - reported < T / 100, "{reported} vs {liquid}");
    assert_eq!(w.record().last_balance, Some(reported));
    assert!(w.record().last_top_up_at.is_none());
    println!("ok: above the floor nothing is sent");

    // --- below the floor: filled to the target, within the cap ---------------
    w.set_config(false, 3 * T, 4 * T, 3 * T);
    let before = pic.cycle_balance(pc);
    let factory_before = pic.cycle_balance(factory);
    let rep = w.check(w.owner).unwrap().unwrap(); // the owner may ask for themselves
    let want = 4 * T - rep.balance.unwrap();
    assert_eq!(rep.sent, want);
    assert_eq!(rep.note, "topped up");
    assert_near(pic.cycle_balance(pc), before + want, "the cycles arrived");
    assert!(pic.cycle_balance(factory) <= factory_before - want);
    let rec = w.record();
    assert_eq!(rec.month_total, Some(want));
    assert_eq!(rec.last_top_up_cycles, Some(want));
    assert!(rec.last_top_up_at.is_some() && rec.month.is_some());
    println!("ok: below the floor the factory sent {want} cycles");

    let rep = w.check_ok();
    assert_eq!((rep.sent, rep.note.as_str()), (0, "enough"));

    // --- the monthly cap -----------------------------------------------------
    w.set_config(false, 10 * T, 12 * T, 3 * T);
    let rep = w.check_ok();
    assert_eq!(rep.sent, 3 * T - want, "only the rest of the 3T cap");
    assert_eq!(w.record().month_total, Some(3 * T));
    let before = pic.cycle_balance(pc);
    let rep = w.check_ok();
    assert_eq!(rep.sent, 0);
    assert!(rep.note.contains("monthly cap reached"), "{}", rep.note);
    assert_near(pic.cycle_balance(pc), before, "nothing arrived");
    println!("ok: the monthly cap stops further top-ups");

    // an admin raises this owner's cap
    let r: Res<()> = w
        .update(factory, w.admin, "top_up_cap_set", (w.owner, Some(5 * T)))
        .unwrap();
    r.unwrap();
    let rep = w.check_ok();
    assert_eq!(rep.sent, 2 * T, "5T cap, 3T already sent");
    let r: Res<()> = w
        .update(factory, w.admin, "top_up_cap_set", (w.owner, None::<u128>))
        .unwrap();
    r.unwrap();
    assert!(w.check_ok().note.contains("monthly cap reached"));
    println!("ok: a cap per owner overrides the default");

    // --- a new month starts at zero -----------------------------------------
    let month = w.record().month.unwrap();
    w.pass(32 * DAY);
    let rep = w.check_ok();
    assert_eq!(rep.sent, 3 * T, "the new month has its full cap");
    let rec = w.record();
    assert_ne!(rec.month.unwrap(), month);
    assert_eq!(rec.month_total, Some(3 * T));
    println!("ok: month {} -> {}: the total starts again", month, rec.month.unwrap());

    // --- the timer -----------------------------------------------------------
    let last_run: Option<u64> = w.query(factory, w.stranger, "top_up_last_run").unwrap();
    assert_eq!(last_run, None, "switched off so far, and it did not run");
    let floor = w.liquid() + 5 * T;
    w.set_config(true, floor, floor + T, 1_000 * T);
    let before = pic.cycle_balance(pc);
    w.pass(HOUR + Duration::from_secs(60));
    let first_run: Option<u64> = w.query(factory, w.stranger, "top_up_last_run").unwrap();
    assert!(first_run.is_some(), "the hourly tick ran the check");
    assert!(pic.cycle_balance(pc) > before + 4 * T, "and topped up");
    println!("ok: the timer checked and topped up by itself");

    // not again within a day, although the canister is below a higher floor
    let floor = w.liquid() + 5 * T;
    w.set_config(true, floor, floor + T, 1_000 * T);
    let before = w.record().month_total;
    w.pass(3 * HOUR);
    assert_eq!(w.record().month_total, before);
    let run: Option<u64> = w.query(factory, w.stranger, "top_up_last_run").unwrap();
    assert_eq!(run, first_run);
    println!("ok: within a day the timer does not check again");

    // --- upgrade: state survives, the timer starts again ----------------------
    let rec_before = w.record();
    pic.upgrade_canister(factory, wasm("canister_factory"), vec![], Some(w.admin))
        .expect("factory upgrade");
    let cfg: TopUpConfig = w.query(factory, w.stranger, "top_up_config").unwrap();
    assert_eq!(cfg.floor, floor, "settings survive the upgrade");
    let rec = w.record();
    assert_eq!(rec.month_total, rec_before.month_total);
    assert_eq!(rec.last_top_up_at, rec_before.last_top_up_at);
    let run: Option<u64> = w.query(factory, w.stranger, "top_up_last_run").unwrap();
    assert_eq!(run, first_run, "and so does the time of the last run");

    w.pass(2 * HOUR);
    assert_eq!(w.record().month_total, rec_before.month_total, "still within the day");
    w.pass(DAY);
    assert!(
        w.record().month_total > rec_before.month_total,
        "a day after the last run the restarted timer checked again"
    );
    println!("ok: after an upgrade the state is there and the timer runs again");

    // --- a canister that stops reporting gets nothing --------------------------
    w.set_config(false, 900 * T, 901 * T, 1_000 * T);
    // a stranger cannot name the factory (refused or answered with an error)
    let _ = pic.update_call(
        pc,
        w.stranger,
        "personal_factory_set",
        encode_one(Some(w.stranger)).unwrap(),
    );
    let known: Option<Principal> = w.query(pc, w.stranger, "personal_factory").unwrap();
    assert_eq!(known, Some(factory), "a stranger cannot name the factory");
    w.pic
        .update_call(
            pc,
            w.owner,
            "personal_factory_set",
            encode_one(None::<Principal>).unwrap(),
        )
        .expect("the owner unsets the factory");
    let before = pic.cycle_balance(pc);
    let err = w.check(w.admin).unwrap().unwrap_err();
    assert!(err.contains("balance call failed"), "{err}");
    assert_near(pic.cycle_balance(pc), before, "no report, no cycles");
    assert!(w.record().last_error.unwrap().contains("balance call failed"));
    w.pic
        .update_call(
            pc,
            w.owner,
            "personal_factory_set",
            encode_one(Some(factory)).unwrap(),
        )
        .expect("the owner names the factory again");
    println!("ok: a canister that refuses to report gets nothing; the owner can name the factory");

    // --- a frozen canister is rescued ------------------------------------------
    let liquid = w.liquid();
    w.set_config(false, liquid + T, liquid + 2 * T, 1_000 * T);
    // The owner freezes it: a threshold no balance can pay for
    let freeze = |seconds: u64| {
        pic.update_canister_settings(
            pc,
            Some(w.owner),
            CanisterSettings {
                freezing_threshold: Some(Nat::from(seconds)),
                ..Default::default()
            },
        )
        .expect("update_settings")
    };
    freeze(u64::MAX);
    let r: Result<u128, String> = w.query(pc, w.owner, "cycles_liquid_balance");
    println!("   frozen canister, asked by its owner: {:?}", r.as_ref().err());
    let before = pic.cycle_balance(pc);
    let rep = w.check_ok();
    assert_eq!(rep.note, "was frozen, topped up");
    assert_eq!(rep.balance, Some(0));
    assert_eq!(rep.sent, liquid + 2 * T, "a frozen canister counts as empty");
    assert_near(pic.cycle_balance(pc), before + rep.sent, "a deposit reaches a frozen canister");
    freeze(30 * 24 * 60 * 60);
    assert!(w.liquid() > liquid + 2 * T);
    println!("ok: a frozen canister is rescued");

    // --- check all ---------------------------------------------------------------
    let floor = w.liquid() + T;
    w.set_config(false, floor, floor + T, 1_000 * T);
    let all: Res<Vec<TopUpReport>> = w
        .update(factory, w.admin, "top_up_check_all", ())
        .unwrap();
    let all = all.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].owner, w.owner);
    assert!(all[0].sent > T);
    println!("ok: check_all reports every ready canister");
}

#[test]
fn the_factory_keeps_its_own_minimum() {
    // 20T, of which 17T must stay. Creating the canister costs 2T: about 1T
    // is left to spare.
    const MINIMUM: u128 = 17 * T;
    let w = setup_with(20 * T, Some(MINIMUM));
    let spare = w.pic.cycle_balance(w.factory) - MINIMUM;
    assert!(spare < 2 * T, "spare {spare}");

    // The canister wants 5T
    let liquid = w.liquid();
    w.set_config(false, liquid + 4 * T, liquid + 5 * T, 1_000 * T);
    let rep = w.check_ok();
    assert!(rep.sent > 0 && rep.sent <= spare, "sent {} of spare {spare}", rep.sent);
    // It ends at its minimum: a little above (it sent slightly less than
    // its spare) or a hair below (what the calls themselves cost)
    let left = w.pic.cycle_balance(w.factory);
    assert!(
        left > MINIMUM - T / 100 && left < MINIMUM + T / 10,
        "the factory stays at its minimum: {left}"
    );

    let rep = w.check_ok();
    assert_eq!(rep.sent, 0);
    assert!(rep.note.contains("factory low on cycles"), "{}", rep.note);
    println!("ok: the factory sends only what it holds above its minimum");
}

/// The factory holds the cycle pool. Only what an admin set up may spend it:
/// a stranger can neither upload a wasm nor have a canister created.
#[test]
fn a_stranger_cannot_spend_the_factorys_cycles() {
    let w = setup();
    let before = w.pic.cycle_balance(w.factory);

    let upload: Res<u64> = w
        .update(w.factory, w.stranger, "create_upload", ())
        .unwrap();
    assert!(upload.unwrap_err().contains("Admin access required"));

    let created: Res<PersonalCanister> = w
        .update(w.factory, w.stranger, "create_personal_canister", ())
        .unwrap();
    assert!(created.unwrap_err().contains("no credit"));

    // The endpoint that created a canister for any caller, with as many of
    // the factory's cycles as the caller named, is gone
    let gone = w.pic.update_call(
        w.factory,
        w.stranger,
        "create_and_install_with",
        encode_args(()).unwrap(),
    );
    assert!(gone.is_err());

    assert_near(w.pic.cycle_balance(w.factory), before, "the factory's cycles");
}

#[test]
fn a_shared_backend_shows_its_cycles_to_controllers_only() {
    let pic = PocketIc::new();
    let controller = Principal::from_slice(&[1; 29]);
    let stranger = Principal::from_slice(&[3; 29]);
    let backend = install_backend(&pic, controller);
    let raw = pic
        .query_call(backend, controller, "cycles_balance", encode_args(()).unwrap())
        .expect("a controller reads the balance");
    let balance: u128 = decode_one(&raw).unwrap();
    assert!(balance > 4 * T);
    assert!(pic
        .query_call(backend, stranger, "cycles_balance", encode_args(()).unwrap())
        .is_err());
    let raw = pic
        .query_call(backend, stranger, "personal_factory", encode_args(()).unwrap())
        .unwrap();
    let f: Option<Principal> = decode_one(&raw).unwrap();
    assert_eq!(f, None, "a shared canister has no factory");
}

fn main() {}
