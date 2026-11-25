//! Personal canisters, the way Juno's console creates satellites:
//!
//! 1. An admin uploads the backend wasm (chunked upload API) and releases it.
//! 2. A user calls `create_personal_canister`. The factory creates a canister
//!    with itself and the user as controllers, installs the released wasm with
//!    `init(owner = user)`, then removes itself: the user is the only
//!    controller. After that the factory cannot touch the canister.
//! 3. The registry remembers which canister belongs to which user.
//!
//! Who may create one: creation costs the factory real cycles, so it is not
//! open to everybody. An admin gives a principal credits
//! (`personal_credit_set`); one credit pays for one canister and is used up
//! when the canister exists. Admins need no credit. For local test networks
//! an admin can open creation to every signed-in principal
//! (`personal_open_set`).
//!
//! Upgrades are pulled, not pushed: the owner fetches a release
//! (`release_wasm`) and installs it with their own identity.
//!
//! A failed step is resumed by calling `create_personal_canister` again: the
//! registry keeps the canister id and how far it got, so no second canister is
//! created and no cycles are spent twice.

use candid::{CandidType, Principal};
use ic_cdk::api::msg_caller;
use ic_cdk::call::Call;
use ic_cdk::management_canister::{
    install_code, update_settings, CanisterInstallMode, CanisterSettings, CreateCanisterArgs,
    CreateCanisterResult, InstallCodeArgs, UpdateSettingsArgs,
};
use ic_cdk_macros::{query, update};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{must_allowed, must_be_admin, must_have_cycles_left, sha256_hex, STATE};

/// Cycles given to a new personal canister (creation fee included)
pub const DEFAULT_PERSONAL_CYCLES: u128 = 2_000_000_000_000; // 2T
const MAX_VERSION_LEN: usize = 32;
/// A new canister freezes (stops answering, keeps its data) when its cycles
/// would last less than this. Set explicitly, to the network's default of 30
/// days: the threshold is a deposit (about 0.8T per GiB stored for 30 days),
/// and a longer one would freeze a canister with the 2T start after a few
/// hundred MB. Raise it together with the start cycles once top-ups exist.
const FREEZING_THRESHOLD_SECS: u64 = 30 * 24 * 60 * 60;
/// A create call that has not finished after this long is taken as dead (the
/// factory was upgraded or trapped in between) and may be resumed.
const STALE_LOCK_NS: u64 = 10 * 60 * 1_000_000_000;

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct Release {
    pub version: String,
    pub sha256_hex: String,
    pub wasm: Vec<u8>,
    pub released_at: u64,
    pub released_by: Principal,
}

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct ReleaseInfo {
    pub version: String,
    pub sha256_hex: String,
    pub size: u64,
    pub released_at: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PersonalStatus {
    /// Registered, nothing created yet
    Pending,
    /// Canister exists; factory and user are controllers
    Created,
    /// Code installed; factory is still a controller
    Installed,
    /// Handed off: the user is the only controller
    Ready,
}

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct PersonalCanister {
    pub owner: Principal,
    pub canister_id: Option<Principal>,
    pub status: PersonalStatus,
    pub version: Option<String>,
    pub module_sha256_hex: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
    /// A create call is running for this owner
    pub busy: bool,
    pub last_error: Option<String>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Default)]
pub struct PersonalState {
    pub release: Option<Release>,
    pub canisters: BTreeMap<Principal, PersonalCanister>,
    pub cycles_per_canister: Option<u128>,
    /// How many personal canisters each principal may still create.
    /// Optional so that state saved by an older factory still loads.
    pub credits: Option<BTreeMap<Principal, u32>>,
    /// Creation open to every signed-in principal (local test networks only)
    pub open: Option<bool>,
}

/// Init argument of the backend wasm (`personal::BackendInitArg` there)
#[derive(CandidType, Serialize, Deserialize)]
struct BackendInitArg {
    owner: Option<Principal>,
}

fn with_personal<R>(f: impl FnOnce(&mut PersonalState) -> R) -> R {
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        f(st.personal.get_or_insert_with(PersonalState::default))
    })
}

fn is_wasm_or_gzip(bytes: &[u8]) -> bool {
    bytes.starts_with(b"\0asm") || bytes.starts_with(&[0x1f, 0x8b])
}

// ---- releases ---------------------------------------------------------------

/// Admin: make a committed upload the release that new personal canisters get.
#[update]
fn release_set(upload_id: u64, version: String) -> Result<ReleaseInfo, String> {
    let caller = msg_caller();
    must_be_admin(caller)?;
    let version = version.trim().to_string();
    if version.is_empty() || version.len() > MAX_VERSION_LEN {
        return Err(format!("version must have 1 to {MAX_VERSION_LEN} characters"));
    }
    let wasm = STATE.with(|s| {
        let st = s.borrow();
        let up = st.uploads.get(&upload_id).ok_or("upload_id not found")?;
        if up.owner != caller {
            return Err("not owner of upload".to_string());
        }
        if up.committed_hash.is_none() {
            return Err("upload not committed".to_string());
        }
        Ok(up.chunks.concat())
    })?;
    if !is_wasm_or_gzip(&wasm) {
        return Err("not a wasm module (plain or gzipped)".into());
    }
    let release = Release {
        version,
        sha256_hex: sha256_hex(&wasm),
        wasm,
        released_at: ic_cdk::api::time(),
        released_by: caller,
    };
    let info = info_of(&release);
    STATE.with(|s| s.borrow_mut().uploads.remove(&upload_id));
    with_personal(|p| p.release = Some(release));
    Ok(info)
}

fn info_of(r: &Release) -> ReleaseInfo {
    ReleaseInfo {
        version: r.version.clone(),
        sha256_hex: r.sha256_hex.clone(),
        size: r.wasm.len() as u64,
        released_at: r.released_at,
    }
}

#[query]
fn release_info() -> Option<ReleaseInfo> {
    with_personal(|p| p.release.as_ref().map(info_of))
}

/// The released wasm, for owners who upgrade their canister themselves.
/// Check it against `release_info().sha256_hex` before installing.
#[query]
fn release_wasm() -> Result<Vec<u8>, String> {
    with_personal(|p| p.release.as_ref().map(|r| r.wasm.clone())).ok_or("no release".into())
}

/// Admin: cycles a new personal canister starts with.
#[update]
fn set_personal_canister_cycles(cycles: u128) -> Result<(), String> {
    must_be_admin(msg_caller())?;
    if cycles < 500_000_000_000 {
        return Err("at least 0.5T cycles: creation alone costs that".into());
    }
    with_personal(|p| p.cycles_per_canister = Some(cycles));
    Ok(())
}

// ---- who may create ---------------------------------------------------------

/// Admin: set how many personal canisters a principal may still create
/// (0 removes the entry).
#[update]
fn personal_credit_set(who: Principal, credits: u32) -> Result<(), String> {
    must_be_admin(msg_caller())?;
    if who == Principal::anonymous() {
        return Err("the anonymous principal cannot own a canister".into());
    }
    with_personal(|p| credit_set(p, who, credits));
    Ok(())
}

/// Admin: all principals that hold credits.
#[query]
fn personal_credits_list() -> Result<Vec<(Principal, u32)>, String> {
    must_be_admin(msg_caller())?;
    Ok(with_personal(|p| {
        p.credits
            .as_ref()
            .map(|c| c.iter().map(|(k, v)| (*k, *v)).collect())
            .unwrap_or_default()
    }))
}

/// How many personal canisters the caller may still create.
#[query]
fn my_personal_credits() -> u32 {
    let caller = msg_caller();
    with_personal(|p| credits_of(p, &caller))
}

/// Admin: open creation to every signed-in principal. For local test
/// networks; on mainnet this lets anybody spend the factory's cycles.
#[update]
fn personal_open_set(open: bool) -> Result<(), String> {
    must_be_admin(msg_caller())?;
    with_personal(|p| p.open = Some(open));
    Ok(())
}

#[query]
fn personal_open() -> bool {
    with_personal(|p| p.open.unwrap_or(false))
}

fn credit_set(p: &mut PersonalState, who: Principal, credits: u32) {
    let map = p.credits.get_or_insert_with(BTreeMap::new);
    if credits == 0 {
        map.remove(&who);
    } else {
        map.insert(who, credits);
    }
}

fn credits_of(p: &PersonalState, who: &Principal) -> u32 {
    p.credits
        .as_ref()
        .and_then(|c| c.get(who).copied())
        .unwrap_or(0)
}

/// May this principal start a new canister? Finishing one that already
/// exists needs no credit: it was paid for when it was created.
fn may_start(p: &PersonalState, who: &Principal, is_admin: bool) -> bool {
    let has_canister = p
        .canisters
        .get(who)
        .map(|e| e.canister_id.is_some())
        .unwrap_or(false);
    has_canister || is_admin || p.open.unwrap_or(false) || credits_of(p, who) > 0
}

/// One credit is used up the moment the canister exists.
fn credit_use(p: &mut PersonalState, who: &Principal) {
    let left = credits_of(p, who).saturating_sub(1);
    if p.credits.is_some() {
        credit_set(p, *who, left);
    }
}

/// Is a create call for this entry running right now?
fn lock_held(e: &PersonalCanister, now: u64) -> bool {
    e.busy && now.saturating_sub(e.updated_at) < STALE_LOCK_NS
}

// ---- registry ---------------------------------------------------------------

#[query]
fn my_personal_canister() -> Option<PersonalCanister> {
    let caller = msg_caller();
    with_personal(|p| p.canisters.get(&caller).cloned())
}

/// Admin: every personal canister the factory created.
#[query]
fn personal_canisters_list() -> Result<Vec<PersonalCanister>, String> {
    must_be_admin(msg_caller())?;
    Ok(with_personal(|p| p.canisters.values().cloned().collect()))
}

// ---- creation ---------------------------------------------------------------

/// Create (or finish creating) the caller's personal canister.
/// Idempotent: once Ready it returns the same canister.
#[update]
async fn create_personal_canister() -> Result<PersonalCanister, String> {
    let caller = msg_caller();
    if caller == Principal::anonymous() {
        return Err("sign in first: the anonymous principal cannot own a canister".into());
    }
    must_allowed(caller)?;
    let is_admin = must_be_admin(caller).is_ok();

    // Take the per-owner lock before the first await
    let now = ic_cdk::api::time();
    let entry = with_personal(|p| {
        // Checked before anything is stored, so that strangers cannot fill
        // the registry with empty entries
        if !may_start(p, &caller, is_admin) {
            return Err(
                "no credit: ask an admin of this factory to allow a personal canister for your principal"
                    .to_string(),
            );
        }
        let e = p.canisters.entry(caller).or_insert_with(|| PersonalCanister {
            owner: caller,
            canister_id: None,
            status: PersonalStatus::Pending,
            version: None,
            module_sha256_hex: None,
            created_at: now,
            updated_at: now,
            busy: false,
            last_error: None,
        });
        if e.status == PersonalStatus::Ready {
            return Ok(Some(e.clone()));
        }
        if lock_held(e, now) {
            return Err("creation already in progress".to_string());
        }
        e.busy = true;
        e.updated_at = now;
        Ok(None)
    })?;
    if let Some(ready) = entry {
        return Ok(ready);
    }

    let result = run_creation(caller).await;

    with_personal(|p| {
        let e = p.canisters.get_mut(&caller).expect("entry exists");
        e.busy = false;
        e.updated_at = ic_cdk::api::time();
        e.last_error = result.as_ref().err().cloned();
        result.map(|_| e.clone())
    })
}

fn update_entry(owner: Principal, f: impl FnOnce(&mut PersonalCanister)) {
    with_personal(|p| {
        if let Some(e) = p.canisters.get_mut(&owner) {
            f(e);
        }
    });
}

async fn run_creation(owner: Principal) -> Result<(), String> {
    let (release, cycles, entry) = with_personal(|p| {
        (
            p.release.clone(),
            p.cycles_per_canister.unwrap_or(DEFAULT_PERSONAL_CYCLES),
            p.canisters.get(&owner).cloned().expect("entry exists"),
        )
    });
    let release = release.ok_or("no release: an admin must call release_set first")?;
    let me = ic_cdk::api::canister_self();

    // 1) create: factory + owner as controllers
    let canister_id = match entry.canister_id {
        Some(id) => id,
        None => {
            must_have_cycles_left()?;
            let args = CreateCanisterArgs {
                settings: Some(settings_with(vec![me, owner])),
            };
            let res = Call::unbounded_wait(Principal::management_canister(), "create_canister")
                .with_arg(&args)
                .with_cycles(cycles)
                .await
                .map_err(|e| format!("create_canister failed: {e:?}"))?;
            // The reply is `record { canister_id : principal }`
            let id = res
                .candid::<CreateCanisterResult>()
                .map_err(|e| format!("create_canister reply: {e:?}"))?
                .canister_id;
            update_entry(owner, |e| {
                e.canister_id = Some(id);
                e.status = PersonalStatus::Created;
            });
            with_personal(|p| credit_use(p, &owner));
            STATE.with(|s| s.borrow_mut().total_canisters_created += 1);
            id
        }
    };

    // 2) install the release with the owner baked in. Reinstall so that a
    //    retry after a half-finished install starts clean (no data yet).
    let status = with_personal(|p| p.canisters[&owner].status);
    if status == PersonalStatus::Created {
        let arg = candid::encode_one(Some(BackendInitArg { owner: Some(owner) }))
            .map_err(|e| format!("encode init arg: {e}"))?;
        install_code(&InstallCodeArgs {
            mode: CanisterInstallMode::Reinstall,
            canister_id,
            wasm_module: release.wasm.clone(),
            arg,
        })
        .await
        .map_err(|e| format!("install_code failed: {e:?}"))?;
        update_entry(owner, |e| {
            e.status = PersonalStatus::Installed;
            e.version = Some(release.version.clone());
            e.module_sha256_hex = Some(release.sha256_hex.clone());
        });
    }

    // 3) hand off: the owner is the only controller
    update_settings(&UpdateSettingsArgs {
        canister_id,
        settings: settings_with(vec![owner]),
    })
    .await
    .map_err(|e| format!("handoff (update_settings) failed: {e:?}"))?;
    update_entry(owner, |e| e.status = PersonalStatus::Ready);
    Ok(())
}

fn settings_with(controllers: Vec<Principal>) -> CanisterSettings {
    CanisterSettings {
        controllers: Some(controllers),
        compute_allocation: None,
        memory_allocation: None,
        freezing_threshold: Some(candid::Nat::from(FREEZING_THRESHOLD_SECS)),
        reserved_cycles_limit: None,
        log_visibility: None,
        wasm_memory_limit: None,
        wasm_memory_threshold: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_wasm_and_gzip() {
        assert!(is_wasm_or_gzip(b"\0asm\x01\0\0\0"));
        assert!(is_wasm_or_gzip(&[0x1f, 0x8b, 8, 0]));
        assert!(!is_wasm_or_gzip(b"hello"));
        assert!(!is_wasm_or_gzip(b""));
    }

    fn entry(owner: Principal, canister: bool) -> PersonalCanister {
        PersonalCanister {
            owner,
            canister_id: canister.then(|| Principal::from_slice(&[9; 10])),
            status: PersonalStatus::Pending,
            version: None,
            module_sha256_hex: None,
            created_at: 0,
            updated_at: 0,
            busy: false,
            last_error: None,
        }
    }

    #[test]
    fn creation_needs_a_credit_unless_open_or_admin() {
        let (a, b) = (Principal::from_slice(&[1; 29]), Principal::from_slice(&[2; 29]));
        let mut p = PersonalState::default();
        assert!(!may_start(&p, &a, false));
        assert!(may_start(&p, &a, true), "admins need no credit");

        credit_set(&mut p, a, 1);
        assert!(may_start(&p, &a, false));
        assert!(!may_start(&p, &b, false), "a credit is per principal");

        credit_use(&mut p, &a);
        assert_eq!(credits_of(&p, &a), 0);
        assert!(!may_start(&p, &a, false), "one credit, one canister");

        // finishing a canister that already exists needs no credit
        p.canisters.insert(a, entry(a, true));
        assert!(may_start(&p, &a, false));

        p.open = Some(true);
        assert!(may_start(&p, &b, false));
    }

    #[test]
    fn using_a_credit_never_goes_below_zero() {
        let a = Principal::from_slice(&[1; 29]);
        let mut p = PersonalState::default();
        credit_use(&mut p, &a);
        assert_eq!(credits_of(&p, &a), 0);
        credit_set(&mut p, a, 2);
        credit_use(&mut p, &a);
        assert_eq!(credits_of(&p, &a), 1);
    }

    #[test]
    fn a_stuck_lock_expires() {
        let mut e = entry(Principal::from_slice(&[1; 29]), false);
        assert!(!lock_held(&e, 5));
        e.busy = true;
        e.updated_at = 1_000;
        assert!(lock_held(&e, 1_000 + STALE_LOCK_NS - 1));
        assert!(!lock_held(&e, 1_000 + STALE_LOCK_NS));
    }

    #[test]
    fn init_arg_matches_the_backend_candid() {
        // (opt record { owner : opt principal })
        let owner = Principal::from_slice(&[7; 29]);
        let bytes = candid::encode_one(Some(BackendInitArg { owner: Some(owner) })).unwrap();
        let back: Option<BackendInitArg> = candid::decode_one(&bytes).unwrap();
        assert_eq!(back.unwrap().owner, Some(owner));
    }
}
