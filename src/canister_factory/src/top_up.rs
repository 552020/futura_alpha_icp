//! Keeping personal canisters alive: the factory tops them up.
//!
//! After the hand-off the factory is not a controller of a personal canister,
//! so it cannot ask the network how many cycles the canister has. Two things
//! still work without any rights:
//!
//! - the canister tells the factory its balance (`cycles_liquid_balance`, a
//!   query the backend answers to its owner and to the factory that installed
//!   it), and
//! - anybody may deposit cycles into any canister (`deposit_cycles`).
//!
//! Once a day the factory asks every `Ready` canister for its liquid balance:
//! the cycles it can still spend before it freezes. Below the floor the
//! factory sends enough to reach the target, but never more per calendar
//! month than the owner's cap, and never so much that the factory itself
//! falls below `min_factory_cycles`.
//!
//! The canister's code belongs to its owner, who may install anything. So the
//! reported number is not trusted beyond the cap: a canister that lies gets
//! at most its monthly cap. The balance call is a bounded-wait call, so a
//! canister that never answers cannot keep the factory from being upgraded.
//!
//! A frozen canister answers nothing; the network rejects the call for it
//! ("out of cycles", a transient system reject). That counts as a balance of
//! zero and is topped up, if the canister answered at least once before.
//!
//! Timers do not survive an upgrade. `start_timer` is called from `init` and
//! `post_upgrade`; it ticks every hour and runs the check when the last run
//! (kept in the saved state) is a day old, so frequent upgrades do not
//! postpone the check forever.

use candid::{CandidType, Principal};
use ic_cdk::api::msg_caller;
use ic_cdk::call::{Call, CallFailed, RejectCode};
use ic_cdk::management_canister::{deposit_cycles, DepositCyclesArgs};
use ic_cdk_macros::{query, update};
use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::time::Duration;

use crate::personal::{with_personal, PersonalState, PersonalStatus};
use crate::{must_be_admin, STATE};

const T: u128 = 1_000_000_000_000;
/// Top up when a canister can spend less than this
pub const DEFAULT_FLOOR: u128 = T;
/// ... and bring it back to this
pub const DEFAULT_TARGET: u128 = 2 * T;
/// Most the factory sends one owner in a calendar month, unless an admin
/// set another cap for that owner
pub const DEFAULT_MONTHLY_CAP: u128 = 3 * T;
/// Smaller amounts are not worth a call
const MIN_DEPOSIT: u128 = 10_000_000_000; // 0.01T
/// Upper bound an admin can set for target and cap: a typing error should
/// not empty the factory
const MAX_SETTING: u128 = 1_000 * T;

const CHECK_EVERY_NS: u64 = 24 * 60 * 60 * 1_000_000_000;
const TICK: Duration = Duration::from_secs(60 * 60);
/// How long the factory waits for a canister to report its balance
const BALANCE_TIMEOUT_SECS: u32 = 60;

#[derive(CandidType, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct TopUpConfig {
    /// The daily timer runs only when this is set; the admin calls always work
    pub enabled: bool,
    pub floor: u128,
    pub target: u128,
    pub monthly_cap: u128,
}

impl Default for TopUpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            floor: DEFAULT_FLOOR,
            target: DEFAULT_TARGET,
            monthly_cap: DEFAULT_MONTHLY_CAP,
        }
    }
}

/// What the factory knows about topping up one personal canister. Kept in
/// its registry entry. Every field is optional so that later additions still
/// load from saved state.
#[derive(CandidType, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Debug)]
pub struct TopUpRecord {
    /// Liquid cycles the canister last reported (0 when it was frozen)
    pub last_balance: Option<u128>,
    pub last_checked_at: Option<u64>,
    pub last_top_up_cycles: Option<u128>,
    pub last_top_up_at: Option<u64>,
    /// Calendar month (UTC) of `month_total`, as year * 100 + month
    pub month: Option<u32>,
    /// Cycles sent in that month
    pub month_total: Option<u128>,
    /// This owner's monthly cap, if an admin set one
    pub monthly_cap: Option<u128>,
    /// Why the last check did not go through
    pub last_error: Option<String>,
}

/// Result of one check, for the caller
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct TopUpReport {
    pub owner: Principal,
    pub canister_id: Option<Principal>,
    /// Liquid cycles reported; none when the canister could not be asked
    pub balance: Option<u128>,
    /// Cycles sent by this check
    pub sent: u128,
    pub note: String,
}

// ---- the decision (pure) ----------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Decision {
    /// At or above the floor
    Enough,
    Send(u128),
    /// Needs cycles, but this month's cap is used up
    CapReached,
    /// Needs cycles, but the factory has none to spare
    FactoryLow,
}

/// Calendar month (UTC) of a network time, as year * 100 + month
pub fn month_of(time_ns: u64) -> u32 {
    // Days since 1970-01-01 to a civil date (Howard Hinnant's algorithm)
    let days = (time_ns / 1_000_000_000 / 86_400) as i64;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
    (year * 100 + month) as u32
}

/// Cycles already sent in `now_month`: a total from another month is over
pub fn sent_in_month(record: &TopUpRecord, now_month: u32) -> u128 {
    if record.month == Some(now_month) {
        record.month_total.unwrap_or(0)
    } else {
        0
    }
}

/// The settings that apply to one owner
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub floor: u128,
    pub target: u128,
    pub monthly_cap: u128,
}

impl Limits {
    /// The factory's settings, with the owner's own cap if they have one
    fn of(cfg: &TopUpConfig, record: &TopUpRecord) -> Self {
        Self {
            floor: cfg.floor,
            target: cfg.target,
            monthly_cap: record.monthly_cap.unwrap_or(cfg.monthly_cap),
        }
    }
}

/// How much to send a canister that can still spend `balance`.
/// `factory_spare` is what the factory holds above its own minimum.
pub fn decide(
    balance: u128,
    limits: Limits,
    sent_this_month: u128,
    factory_spare: u128,
) -> Decision {
    if balance >= limits.floor {
        return Decision::Enough;
    }
    let wanted = limits.target.saturating_sub(balance);
    if wanted < MIN_DEPOSIT {
        return Decision::Enough;
    }
    let cap_left = limits.monthly_cap.saturating_sub(sent_this_month);
    if cap_left < MIN_DEPOSIT {
        return Decision::CapReached;
    }
    if factory_spare < MIN_DEPOSIT {
        return Decision::FactoryLow;
    }
    Decision::Send(wanted.min(cap_left).min(factory_spare))
}

/// Book a top-up in the record, starting a new month if needed
pub fn book(record: &mut TopUpRecord, cycles: u128, now_month: u32) {
    record.month_total = Some(sent_in_month(record, now_month) + cycles);
    record.month = Some(now_month);
}

/// Take a booked top-up back (the deposit failed)
pub fn unbook(record: &mut TopUpRecord, cycles: u128, now_month: u32) {
    if record.month == Some(now_month) {
        record.month_total = Some(record.month_total.unwrap_or(0).saturating_sub(cycles));
    }
}

pub fn validate(cfg: &TopUpConfig) -> Result<(), String> {
    if cfg.floor == 0 {
        return Err("floor must be above 0".into());
    }
    if cfg.target <= cfg.floor {
        return Err("target must be above floor".into());
    }
    if cfg.target > MAX_SETTING || cfg.monthly_cap > MAX_SETTING {
        return Err("target and monthly_cap may be at most 1000T".into());
    }
    Ok(())
}

fn config_of(p: &PersonalState) -> TopUpConfig {
    p.top_up.unwrap_or_default()
}

// ---- locks (not saved: an upgrade clears them) ------------------------------

thread_local! {
    static IN_FLIGHT: RefCell<BTreeSet<Principal>> = const { RefCell::new(BTreeSet::new()) };
    static ALL_RUNNING: Cell<bool> = const { Cell::new(false) };
}

/// One check per owner at a time, so that two calls cannot both send.
/// Released on drop, also when a call traps after an await.
struct OwnerLock(Principal);

impl OwnerLock {
    fn take(owner: Principal) -> Option<Self> {
        IN_FLIGHT.with(|s| s.borrow_mut().insert(owner).then_some(OwnerLock(owner)))
    }
}

impl Drop for OwnerLock {
    fn drop(&mut self) {
        IN_FLIGHT.with(|s| {
            s.borrow_mut().remove(&self.0);
        });
    }
}

struct AllLock;

impl AllLock {
    fn take() -> Option<Self> {
        // replace returns the old value: free if it was false
        ALL_RUNNING.with(|r| (!r.replace(true)).then_some(AllLock))
    }
}

impl Drop for AllLock {
    fn drop(&mut self) {
        ALL_RUNNING.with(|r| r.set(false));
    }
}

// ---- checking one canister --------------------------------------------------

fn record_update(owner: Principal, f: impl FnOnce(&mut TopUpRecord)) {
    with_personal(|p| {
        if let Some(e) = p.canisters.get_mut(&owner) {
            f(e.top_up.get_or_insert_with(TopUpRecord::default));
        }
    });
}

fn factory_spare() -> u128 {
    let min = STATE.with(|s| s.borrow().cfg.min_factory_cycles);
    ic_cdk::api::canister_cycle_balance().saturating_sub(min)
}

/// Note a failed check in the registry and hand the message back
fn failed(owner: Principal, msg: String) -> String {
    let now = ic_cdk::api::time();
    record_update(owner, |r| {
        r.last_checked_at = Some(now);
        r.last_error = Some(msg.clone());
    });
    msg
}

/// What a canister answered when asked for its cycles
enum Reading {
    /// The cycles it can still spend
    Liquid(u128),
    /// Nothing: the network rejected the call because the canister is frozen
    Frozen,
}

impl Reading {
    fn balance(&self) -> u128 {
        match self {
            Reading::Liquid(b) => *b,
            Reading::Frozen => 0,
        }
    }
}

/// Did the network reject the call because the canister is frozen?
fn is_frozen_reject(e: &CallFailed) -> bool {
    match e {
        CallFailed::CallRejected(r) => r.reject_code() == Ok(RejectCode::SysTransient),
        _ => false,
    }
}

/// Ask a canister for its liquid cycles. Bounded wait: its code is the
/// owner's, not ours. A frozen canister counts as such only if it
/// `reported_before`, so that one which never ran our code gets nothing.
async fn read_balance(canister_id: Principal, reported_before: bool) -> Result<Reading, String> {
    let asked = Call::bounded_wait(canister_id, "cycles_liquid_balance")
        .change_timeout(BALANCE_TIMEOUT_SECS)
        .await;
    match asked {
        Ok(reply) => reply
            .candid::<u128>()
            .map(Reading::Liquid)
            .map_err(|e| format!("balance reply not understood: {e:?}")),
        Err(e) if is_frozen_reject(&e) && reported_before => Ok(Reading::Frozen),
        Err(e) => Err(format!("balance call failed: {e:?}")),
    }
}

/// Store the balance and decide. A top-up is booked here, before the
/// deposit, so that a second check running meanwhile sees it.
fn decide_and_book(owner: Principal, balance: u128, month: u32) -> Result<Decision, String> {
    let now = ic_cdk::api::time();
    // Read before with_personal: both borrow the factory state
    let spare = factory_spare();
    with_personal(|p| {
        let cfg = config_of(p);
        let e = p.canisters.get_mut(&owner).ok_or("entry vanished")?;
        let r = e.top_up.get_or_insert_with(TopUpRecord::default);
        r.last_balance = Some(balance);
        r.last_checked_at = Some(now);
        let decision = decide(balance, Limits::of(&cfg, r), sent_in_month(r, month), spare);
        r.last_error = match decision {
            Decision::Enough | Decision::Send(_) => None,
            Decision::CapReached => Some("monthly cap reached".into()),
            Decision::FactoryLow => Some("factory low on cycles".into()),
        };
        if let Decision::Send(cycles) = decision {
            book(r, cycles, month);
        }
        Ok(decision)
    })
}

/// Send the booked cycles. No rights needed; works on a frozen canister too.
async fn deposit(
    owner: Principal,
    canister_id: Principal,
    cycles: u128,
    month: u32,
) -> Result<(), String> {
    match deposit_cycles(&DepositCyclesArgs { canister_id }, cycles).await {
        Ok(()) => {
            let now = ic_cdk::api::time();
            record_update(owner, |r| {
                r.last_top_up_cycles = Some(cycles);
                r.last_top_up_at = Some(now);
            });
            Ok(())
        }
        Err(e) => {
            record_update(owner, |r| unbook(r, cycles, month));
            Err(failed(owner, format!("deposit_cycles failed: {e:?}")))
        }
    }
}

/// Check one owner's canister: ask, decide, deposit.
async fn check_owner(owner: Principal) -> Result<TopUpReport, String> {
    let (canister_id, reported_before) = with_personal(|p| {
        let e = p.canisters.get(&owner).ok_or("no personal canister")?;
        if e.status != PersonalStatus::Ready {
            return Err(format!("canister is not ready ({:?})", e.status));
        }
        let id = e.canister_id.ok_or("canister has no id")?;
        let reported = e.top_up.as_ref().is_some_and(|r| r.last_balance.is_some());
        Ok((id, reported))
    })?;
    let _lock = OwnerLock::take(owner).ok_or("a check for this owner is running")?;

    let reading = read_balance(canister_id, reported_before)
        .await
        .map_err(|e| failed(owner, e))?;
    let balance = reading.balance();
    let month = month_of(ic_cdk::api::time());

    let (sent, note) = match decide_and_book(owner, balance, month)? {
        Decision::Enough => (0, "enough"),
        Decision::CapReached => (0, "needs cycles, monthly cap reached"),
        Decision::FactoryLow => (0, "needs cycles, factory low on cycles"),
        Decision::Send(cycles) => {
            deposit(owner, canister_id, cycles, month).await?;
            match reading {
                Reading::Frozen => (cycles, "was frozen, topped up"),
                Reading::Liquid(_) => (cycles, "topped up"),
            }
        }
    };
    Ok(TopUpReport {
        owner,
        canister_id: Some(canister_id),
        balance: Some(balance),
        sent,
        note: note.to_string(),
    })
}

async fn check_all() -> Result<Vec<TopUpReport>, String> {
    let _lock = AllLock::take().ok_or("a check of all canisters is running")?;
    let owners: Vec<(Principal, Option<Principal>)> = with_personal(|p| {
        // Noted at the start: a run that fails half-way is not repeated
        // every hour
        p.top_up_last_run = Some(ic_cdk::api::time());
        p.canisters
            .values()
            .filter(|e| e.status == PersonalStatus::Ready)
            .map(|e| (e.owner, e.canister_id))
            .collect()
    });
    let mut reports = Vec::with_capacity(owners.len());
    for (owner, canister_id) in owners {
        reports.push(match check_owner(owner).await {
            Ok(r) => r,
            Err(e) => TopUpReport {
                owner,
                canister_id,
                balance: None,
                sent: 0,
                note: e,
            },
        });
    }
    Ok(reports)
}

// ---- timer ------------------------------------------------------------------

/// Is the daily check due?
pub fn due(enabled: bool, last_run: Option<u64>, now: u64) -> bool {
    enabled && last_run.is_none_or(|t| now.saturating_sub(t) >= CHECK_EVERY_NS)
}

/// Call from `init` and `post_upgrade`: timers are lost on upgrade.
pub fn start_timer() {
    ic_cdk_timers::set_timer_interval(TICK, || {
        let now = ic_cdk::api::time();
        let run = with_personal(|p| due(config_of(p).enabled, p.top_up_last_run, now));
        if run {
            ic_cdk::futures::spawn(async {
                if let Err(e) = check_all().await {
                    ic_cdk::println!("top-up timer: {e}");
                }
            });
        }
    });
}

// ---- endpoints --------------------------------------------------------------

/// Check one owner's canister now and top it up if needed.
/// For admins, and for the owner (the monthly cap applies either way).
#[update]
async fn top_up_check(owner: Principal) -> Result<TopUpReport, String> {
    let caller = msg_caller();
    if caller != owner {
        must_be_admin(caller)?;
    }
    check_owner(owner).await
}

/// Admin: check every ready canister now. The daily timer does the same.
#[update]
async fn top_up_check_all() -> Result<Vec<TopUpReport>, String> {
    must_be_admin(msg_caller())?;
    check_all().await
}

#[query]
fn top_up_config() -> TopUpConfig {
    with_personal(|p| config_of(p))
}

/// Admin: floor, target, default monthly cap, and whether the timer runs.
#[update]
fn top_up_config_set(cfg: TopUpConfig) -> Result<(), String> {
    must_be_admin(msg_caller())?;
    validate(&cfg)?;
    with_personal(|p| p.top_up = Some(cfg));
    Ok(())
}

/// Admin: one owner's monthly cap; `null` goes back to the default.
/// A cap of 0 stops top-ups for that owner.
#[update]
fn top_up_cap_set(owner: Principal, monthly_cap: Option<u128>) -> Result<(), String> {
    must_be_admin(msg_caller())?;
    if monthly_cap.is_some_and(|c| c > MAX_SETTING) {
        return Err("monthly_cap may be at most 1000T".into());
    }
    with_personal(|p| {
        let e = p.canisters.get_mut(&owner).ok_or("no personal canister")?;
        e.top_up.get_or_insert_with(TopUpRecord::default).monthly_cap = monthly_cap;
        Ok(())
    })
}

/// When the check of all canisters last started (network time, ns)
#[query]
fn top_up_last_run() -> Option<u64> {
    with_personal(|p| p.top_up_last_run)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY_NS: u64 = 86_400 * 1_000_000_000;
    const SPARE: u128 = 100 * T;

    /// Floor 1T, target 2T, monthly cap 3T
    const LIMITS: Limits = Limits {
        floor: T,
        target: 2 * T,
        monthly_cap: 3 * T,
    };

    /// With a factory that has plenty to spare
    fn d(balance: u128, sent: u128) -> Decision {
        decide(balance, LIMITS, sent, SPARE)
    }

    /// With a factory that has `spare` above its minimum, nothing sent yet
    fn with_spare(balance: u128, spare: u128) -> Decision {
        decide(balance, LIMITS, 0, spare)
    }

    #[test]
    fn nothing_is_sent_at_or_above_the_floor() {
        assert_eq!(d(T, 0), Decision::Enough);
        assert_eq!(d(5 * T, 0), Decision::Enough);
    }

    #[test]
    fn below_the_floor_it_is_filled_to_the_target() {
        assert_eq!(d(T - 1, 0), Decision::Send(T + 1));
        assert_eq!(d(T / 4, 0), Decision::Send(2 * T - T / 4));
        assert_eq!(d(0, 0), Decision::Send(2 * T), "a frozen canister counts as 0");
    }

    #[test]
    fn the_monthly_cap_limits_what_is_sent() {
        // 3T cap, 2T already sent: only 1T left although 2T are wanted
        assert_eq!(d(0, 2 * T), Decision::Send(T));
        assert_eq!(d(0, 3 * T), Decision::CapReached);
        assert_eq!(d(0, 4 * T), Decision::CapReached, "more sent than the cap allows");
        // a rest too small to be worth a call
        assert_eq!(d(0, 3 * T - MIN_DEPOSIT + 1), Decision::CapReached);
        assert_eq!(d(0, 3 * T - MIN_DEPOSIT), Decision::Send(MIN_DEPOSIT));
        // cap 0 switches an owner off
        let off = Limits {
            monthly_cap: 0,
            ..LIMITS
        };
        assert_eq!(decide(0, off, 0, SPARE), Decision::CapReached);
    }

    #[test]
    fn the_factory_keeps_its_own_minimum() {
        assert_eq!(with_spare(0, 0), Decision::FactoryLow);
        assert_eq!(with_spare(0, MIN_DEPOSIT - 1), Decision::FactoryLow);
        assert_eq!(with_spare(0, T / 2), Decision::Send(T / 2));
        // a full canister is not a factory problem
        assert_eq!(with_spare(T, 0), Decision::Enough);
    }

    #[test]
    fn the_cap_is_checked_before_the_factory() {
        assert_eq!(decide(0, LIMITS, 3 * T, 0), Decision::CapReached);
    }

    #[test]
    fn months_are_calendar_months_in_utc() {
        assert_eq!(month_of(0), 197001);
        // 2026-11-25 00:00:00 UTC = 1_795_564_800 s
        assert_eq!(month_of(1_795_564_800 * 1_000_000_000), 202611);
        // 2026-11-30 23:59:59 and one second later
        let end_nov = 1_796_083_199u64 * 1_000_000_000;
        assert_eq!(month_of(end_nov), 202611);
        assert_eq!(month_of(end_nov + 1_000_000_000), 202612);
        // year end
        let end_2026 = 1_798_761_599u64 * 1_000_000_000;
        assert_eq!(month_of(end_2026), 202612);
        assert_eq!(month_of(end_2026 + 1_000_000_000), 202701);
        // leap day 2028-02-29 12:00 UTC = 1_835_438_400 s
        assert_eq!(month_of(1_835_438_400 * 1_000_000_000), 202802);
        assert_eq!(month_of((1_835_438_400 + 86_400) * 1_000_000_000), 202803);
    }

    #[test]
    fn a_new_month_starts_at_zero() {
        let mut r = TopUpRecord::default();
        assert_eq!(sent_in_month(&r, 202611), 0);

        book(&mut r, 2 * T, 202611);
        book(&mut r, T, 202611);
        assert_eq!(sent_in_month(&r, 202611), 3 * T);
        assert_eq!(d(0, sent_in_month(&r, 202611)), Decision::CapReached);

        // December: November's total no longer counts
        assert_eq!(sent_in_month(&r, 202612), 0);
        assert_eq!(d(0, sent_in_month(&r, 202612)), Decision::Send(2 * T));
        book(&mut r, T / 2, 202612);
        assert_eq!(r.month, Some(202612));
        assert_eq!(r.month_total, Some(T / 2), "the old total is replaced, not added to");
    }

    #[test]
    fn a_failed_deposit_is_taken_back() {
        let mut r = TopUpRecord::default();
        book(&mut r, T, 202611);
        book(&mut r, T, 202611);
        unbook(&mut r, T, 202611);
        assert_eq!(sent_in_month(&r, 202611), T);
        // a deposit booked in November that fails after midnight of 1 Dec
        // must not reduce December
        book(&mut r, T, 202612);
        unbook(&mut r, T, 202611);
        assert_eq!(sent_in_month(&r, 202612), T);
        unbook(&mut r, 5 * T, 202612);
        assert_eq!(sent_in_month(&r, 202612), 0, "never below zero");
    }

    #[test]
    fn the_daily_check_is_due_after_a_day_and_survives_upgrades() {
        let now = 1_795_564_800u64 * 1_000_000_000;
        assert!(due(true, None, now), "never ran");
        assert!(!due(true, Some(now - DAY_NS + 1), now));
        assert!(due(true, Some(now - DAY_NS), now));
        assert!(!due(false, None, now), "switched off");
        assert!(!due(true, Some(now + 5), now), "clock behind the last run");
    }

    #[test]
    fn an_owners_own_cap_replaces_the_default() {
        let cfg = TopUpConfig::default();
        let mut r = TopUpRecord::default();
        assert_eq!(Limits::of(&cfg, &r).monthly_cap, DEFAULT_MONTHLY_CAP);
        r.monthly_cap = Some(7 * T);
        let l = Limits::of(&cfg, &r);
        assert_eq!((l.floor, l.target, l.monthly_cap), (cfg.floor, cfg.target, 7 * T));
        r.monthly_cap = Some(0);
        assert_eq!(Limits::of(&cfg, &r).monthly_cap, 0, "0 is a cap, not 'unset'");
    }

    #[test]
    fn settings_are_checked() {
        assert!(validate(&TopUpConfig::default()).is_ok());
        let bad = |f: fn(&mut TopUpConfig)| {
            let mut c = TopUpConfig::default();
            f(&mut c);
            validate(&c).is_err()
        };
        assert!(bad(|c| c.floor = 0));
        assert!(bad(|c| c.target = c.floor));
        assert!(bad(|c| c.target = 1_001 * T));
        assert!(bad(|c| c.monthly_cap = 1_001 * T));
        let mut off = TopUpConfig::default();
        off.monthly_cap = 0;
        assert!(validate(&off).is_ok(), "cap 0 means no top-ups");
    }

    #[test]
    fn state_saved_before_top_ups_still_loads() {
        // A registry entry and a state as the previous factory wrote them
        #[derive(CandidType, Serialize)]
        struct OldCanister {
            owner: Principal,
            canister_id: Option<Principal>,
            status: PersonalStatus,
            version: Option<String>,
            module_sha256_hex: Option<String>,
            created_at: u64,
            updated_at: u64,
            busy: bool,
            last_error: Option<String>,
        }
        #[derive(CandidType, Serialize)]
        struct OldState {
            release: Option<crate::personal::Release>,
            canisters: std::collections::BTreeMap<Principal, OldCanister>,
            cycles_per_canister: Option<u128>,
            credits: Option<std::collections::BTreeMap<Principal, u32>>,
            open: Option<bool>,
        }
        let owner = Principal::from_slice(&[1; 29]);
        let old = OldState {
            release: None,
            canisters: [(
                owner,
                OldCanister {
                    owner,
                    canister_id: Some(Principal::from_slice(&[9; 10])),
                    status: PersonalStatus::Ready,
                    version: Some("1".into()),
                    module_sha256_hex: None,
                    created_at: 1,
                    updated_at: 2,
                    busy: false,
                    last_error: None,
                },
            )]
            .into(),
            cycles_per_canister: None,
            credits: None,
            open: Some(true),
        };
        let bytes = candid::encode_one(&old).unwrap();
        let new: PersonalState = candid::decode_one(&bytes).unwrap();
        assert_eq!(new.top_up, None);
        assert_eq!(new.top_up_last_run, None);
        assert_eq!(new.canisters[&owner].top_up, None);
        assert_eq!(new.canisters[&owner].status, PersonalStatus::Ready);
        assert_eq!(config_of(&new), TopUpConfig::default());
    }
}
