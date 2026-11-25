//! Inheritance ("legacy"): events on a capsule that open event-conditioned access.
//!
//! An owner shares a memory with `AccessCondition::EventTriggered { event }`.
//! That entry grants nothing until the event is in effect on the capsule:
//!
//! - **Death** is never recorded by one click. The owner names trustees, how
//!   many must confirm, and a waiting period. When enough trustees have
//!   confirmed, the waiting period starts; the owner can cancel at any time
//!   with their own key. Only after the waiting period is the death in effect.
//!   Nothing runs on a timer: "in effect" is computed from the stored
//!   timestamps whenever access is checked.
//! - **Other events** (wedding, graduation, a birthday, custom ones) are
//!   recorded and revoked by the owner.
//! - **Computed events** (capsule maturity, connection count) follow from the
//!   capsule itself and cannot be recorded by hand.
//!
//! People who were granted access read through `memories_list_shared` and
//! `memories_read_shared`; the owner-only endpoints are unchanged.

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

use crate::capsule::domain::{
    effective_perm_mask_with_events, AccessEvent, Capsule, Perm, PrincipalContext,
};
use crate::types::{Error, Memory, MemoryHeader, PersonRef};

const DAY_NS: u64 = 24 * 60 * 60 * 1_000_000_000;
const YEAR_NS: u64 = 365 * DAY_NS;
/// Waiting period between the last needed confirmation and the death taking effect
pub const DEFAULT_WAITING_PERIOD_NS: u64 = 30 * DAY_NS;
pub const MAX_WAITING_PERIOD_NS: u64 = 10 * YEAR_NS;
pub const MAX_TRUSTEES: usize = 10;
pub const MAX_RECORDED_EVENTS: usize = 100;
const MAX_CUSTOM_EVENT_LEN: usize = 64;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct DeathConfirmation {
    pub trustee: Principal,
    pub confirmed_at: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct EventRecord {
    pub event: AccessEvent,
    pub recorded_at: u64,
    pub recorded_by: Principal,
}

/// Stored on the capsule (`Capsule::legacy`). `None` there means: never configured.
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct LegacyState {
    pub trustees: Vec<Principal>,
    pub required_confirmations: u32,
    pub waiting_period_ns: u64,
    pub death_confirmations: Vec<DeathConfirmation>,
    /// When the number of confirmations first reached `required_confirmations`
    pub death_threshold_reached_at: Option<u64>,
    pub occurred_events: Vec<EventRecord>,
}

impl Default for LegacyState {
    fn default() -> Self {
        Self {
            trustees: vec![],
            required_confirmations: 1,
            waiting_period_ns: DEFAULT_WAITING_PERIOD_NS,
            death_confirmations: vec![],
            death_threshold_reached_at: None,
            occurred_events: vec![],
        }
    }
}

impl LegacyState {
    /// Replace trustees, threshold and waiting period (owner only; checked by the caller).
    pub fn set_settings(
        &mut self,
        mut trustees: Vec<Principal>,
        required_confirmations: u32,
        waiting_period_ns: u64,
        now: u64,
    ) -> Result<(), Error> {
        trustees.sort();
        trustees.dedup();
        if trustees.len() > MAX_TRUSTEES {
            return Err(Error::InvalidArgument(format!(
                "at most {MAX_TRUSTEES} trustees"
            )));
        }
        if trustees.contains(&Principal::anonymous()) {
            return Err(Error::InvalidArgument(
                "the anonymous principal cannot be a trustee".to_string(),
            ));
        }
        if required_confirmations == 0 {
            return Err(Error::InvalidArgument(
                "required_confirmations must be at least 1".to_string(),
            ));
        }
        if !trustees.is_empty() && required_confirmations as usize > trustees.len() {
            return Err(Error::InvalidArgument(
                "required_confirmations cannot exceed the number of trustees".to_string(),
            ));
        }
        if waiting_period_ns > MAX_WAITING_PERIOD_NS {
            return Err(Error::InvalidArgument(
                "waiting period too long".to_string(),
            ));
        }
        self.trustees = trustees;
        self.required_confirmations = required_confirmations;
        self.waiting_period_ns = waiting_period_ns;
        // Confirmations of removed trustees no longer count
        let trustees = &self.trustees;
        self.death_confirmations
            .retain(|c| trustees.contains(&c.trustee));
        self.refresh_threshold(now);
        Ok(())
    }

    fn refresh_threshold(&mut self, now: u64) {
        let enough = !self.trustees.is_empty()
            && self.death_confirmations.len() >= self.required_confirmations as usize;
        match (enough, self.death_threshold_reached_at) {
            (true, None) => self.death_threshold_reached_at = Some(now),
            (false, Some(_)) => self.death_threshold_reached_at = None,
            _ => {}
        }
    }

    pub fn is_trustee(&self, who: &Principal) -> bool {
        self.trustees.contains(who)
    }

    /// A trustee confirms the death. Idempotent per trustee.
    pub fn confirm_death(&mut self, trustee: Principal, now: u64) -> Result<(), Error> {
        if !self.is_trustee(&trustee) {
            return Err(Error::Unauthorized);
        }
        if !self
            .death_confirmations
            .iter()
            .any(|c| c.trustee == trustee)
        {
            self.death_confirmations.push(DeathConfirmation {
                trustee,
                confirmed_at: now,
            });
        }
        self.refresh_threshold(now);
        Ok(())
    }

    /// A trustee takes their confirmation back. Not possible once the death is in effect.
    pub fn withdraw_confirmation(&mut self, trustee: Principal, now: u64) -> Result<(), Error> {
        if !self.is_trustee(&trustee) {
            return Err(Error::Unauthorized);
        }
        if self.death_in_effect(now) {
            return Err(Error::Conflict(
                "the death is already in effect; only an owner can cancel it".to_string(),
            ));
        }
        self.death_confirmations.retain(|c| c.trustee != trustee);
        self.refresh_threshold(now);
        Ok(())
    }

    /// The owner is alive: drop every confirmation (owner only; checked by the caller).
    pub fn cancel_death(&mut self) {
        self.death_confirmations.clear();
        self.death_threshold_reached_at = None;
    }

    pub fn death_effective_at(&self) -> Option<u64> {
        self.death_threshold_reached_at
            .map(|t| t.saturating_add(self.waiting_period_ns))
    }

    pub fn death_in_effect(&self, now: u64) -> bool {
        matches!(self.death_effective_at(), Some(at) if at <= now)
    }

    /// The owner records that an event happened. Idempotent.
    pub fn record_event(
        &mut self,
        event: AccessEvent,
        by: Principal,
        now: u64,
    ) -> Result<(), Error> {
        validate_recordable(&event)?;
        if self.occurred_events.iter().any(|r| r.event == event) {
            return Ok(());
        }
        if self.occurred_events.len() >= MAX_RECORDED_EVENTS {
            return Err(Error::ResourceExhausted);
        }
        self.occurred_events.push(EventRecord {
            event,
            recorded_at: now,
            recorded_by: by,
        });
        Ok(())
    }

    pub fn revoke_event(&mut self, event: &AccessEvent) -> Result<(), Error> {
        validate_recordable(event)?;
        let before = self.occurred_events.len();
        self.occurred_events.retain(|r| &r.event != event);
        if self.occurred_events.len() == before {
            return Err(Error::NotFound);
        }
        Ok(())
    }
}

fn validate_recordable(event: &AccessEvent) -> Result<(), Error> {
    match event {
        AccessEvent::AfterDeath => Err(Error::InvalidArgument(
            "a death is confirmed by trustees (legacy_confirm_death), not recorded".to_string(),
        )),
        AccessEvent::CapsuleMaturity(_) | AccessEvent::ConnectionCount(_) => {
            Err(Error::InvalidArgument(
                "this event is computed from the capsule and cannot be recorded".to_string(),
            ))
        }
        AccessEvent::Custom(name) => {
            if name.trim().is_empty() || name.len() > MAX_CUSTOM_EVENT_LEN {
                Err(Error::InvalidArgument(format!(
                    "custom event names have 1 to {MAX_CUSTOM_EVENT_LEN} characters"
                )))
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
}

/// Which events are in effect for one capsule at one moment. Built once per
/// call and handed to the access check.
#[derive(Clone, Debug, Default)]
pub struct ActiveEvents {
    death: bool,
    occurred: Vec<AccessEvent>,
    capsule_age_ns: u64,
    connections: u32,
}

impl ActiveEvents {
    /// No event is in effect: event-conditioned entries grant nothing.
    pub fn none() -> Self {
        Self::default()
    }

    pub fn for_capsule(capsule: &Capsule, now: u64) -> Self {
        let legacy = capsule.legacy.as_ref();
        Self {
            death: legacy.map(|l| l.death_in_effect(now)).unwrap_or(false),
            occurred: legacy
                .map(|l| l.occurred_events.iter().map(|r| r.event.clone()).collect())
                .unwrap_or_default(),
            capsule_age_ns: now.saturating_sub(capsule.created_at),
            connections: capsule.connections.len() as u32,
        }
    }

    pub fn is_active(&self, event: &AccessEvent) -> bool {
        match event {
            AccessEvent::AfterDeath => self.death,
            AccessEvent::CapsuleMaturity(years) => {
                self.capsule_age_ns >= (*years as u64).saturating_mul(YEAR_NS)
            }
            AccessEvent::ConnectionCount(n) => self.connections >= *n,
            other => self.occurred.contains(other),
        }
    }
}

// ============================================================================
// API types
// ============================================================================

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct LegacyStatus {
    pub trustees: Vec<Principal>,
    pub required_confirmations: u32,
    pub waiting_period_secs: u64,
    pub death_confirmations: Vec<DeathConfirmation>,
    /// Set once enough trustees confirmed: when the death takes (or took) effect
    pub death_effective_at: Option<u64>,
    pub death_in_effect: bool,
    pub occurred_events: Vec<EventRecord>,
    pub now: u64,
}

fn status_of(legacy: &LegacyState, now: u64) -> LegacyStatus {
    LegacyStatus {
        trustees: legacy.trustees.clone(),
        required_confirmations: legacy.required_confirmations,
        waiting_period_secs: legacy.waiting_period_ns / 1_000_000_000,
        death_confirmations: legacy.death_confirmations.clone(),
        death_effective_at: legacy.death_effective_at(),
        death_in_effect: legacy.death_in_effect(now),
        occurred_events: legacy.occurred_events.clone(),
        now,
    }
}

// ============================================================================
// Pure operations on a capsule (unit-tested; the canister endpoints wrap them)
// ============================================================================

fn require_owner(capsule: &Capsule, caller: &Principal) -> Result<(), Error> {
    // Not found rather than unauthorized: do not confirm that a capsule exists
    if capsule.is_owner(&PersonRef::Principal(*caller)) {
        Ok(())
    } else {
        Err(Error::NotFound)
    }
}

fn require_trustee(capsule: &Capsule, caller: &Principal) -> Result<(), Error> {
    match &capsule.legacy {
        Some(l) if l.is_trustee(caller) => Ok(()),
        _ => Err(Error::NotFound),
    }
}

pub fn settings_set(
    capsule: &mut Capsule,
    caller: Principal,
    trustees: Vec<Principal>,
    required_confirmations: u32,
    waiting_period_secs: u64,
    now: u64,
) -> Result<LegacyStatus, Error> {
    require_owner(capsule, &caller)?;
    if trustees
        .iter()
        .any(|t| capsule.is_owner(&PersonRef::Principal(*t)))
    {
        return Err(Error::InvalidArgument(
            "an owner cannot be their own trustee".to_string(),
        ));
    }
    let mut legacy = capsule.legacy.clone().unwrap_or_default();
    legacy.set_settings(
        trustees,
        required_confirmations,
        waiting_period_secs.saturating_mul(1_000_000_000),
        now,
    )?;
    let status = status_of(&legacy, now);
    capsule.legacy = Some(legacy);
    capsule.updated_at = now;
    Ok(status)
}

pub fn status(capsule: &Capsule, caller: Principal, now: u64) -> Result<LegacyStatus, Error> {
    let allowed = capsule.has_write_access(&PersonRef::Principal(caller))
        || require_trustee(capsule, &caller).is_ok();
    if !allowed {
        return Err(Error::NotFound);
    }
    Ok(status_of(&capsule.legacy.clone().unwrap_or_default(), now))
}

pub fn confirm_death(
    capsule: &mut Capsule,
    caller: Principal,
    now: u64,
) -> Result<LegacyStatus, Error> {
    require_trustee(capsule, &caller)?;
    let legacy = capsule.legacy.as_mut().ok_or(Error::NotFound)?;
    legacy.confirm_death(caller, now)?;
    Ok(status_of(legacy, now))
}

pub fn withdraw_confirmation(
    capsule: &mut Capsule,
    caller: Principal,
    now: u64,
) -> Result<LegacyStatus, Error> {
    require_trustee(capsule, &caller)?;
    let legacy = capsule.legacy.as_mut().ok_or(Error::NotFound)?;
    legacy.withdraw_confirmation(caller, now)?;
    Ok(status_of(legacy, now))
}

pub fn cancel_death(
    capsule: &mut Capsule,
    caller: Principal,
    now: u64,
) -> Result<LegacyStatus, Error> {
    require_owner(capsule, &caller)?;
    let mut legacy = capsule.legacy.clone().unwrap_or_default();
    legacy.cancel_death();
    let status = status_of(&legacy, now);
    capsule.legacy = Some(legacy);
    capsule.updated_at = now;
    Ok(status)
}

pub fn event_record(
    capsule: &mut Capsule,
    caller: Principal,
    event: AccessEvent,
    now: u64,
) -> Result<LegacyStatus, Error> {
    require_owner(capsule, &caller)?;
    let mut legacy = capsule.legacy.clone().unwrap_or_default();
    legacy.record_event(event, caller, now)?;
    let status = status_of(&legacy, now);
    capsule.legacy = Some(legacy);
    capsule.updated_at = now;
    Ok(status)
}

pub fn event_revoke(
    capsule: &mut Capsule,
    caller: Principal,
    event: AccessEvent,
    now: u64,
) -> Result<LegacyStatus, Error> {
    require_owner(capsule, &caller)?;
    let mut legacy = capsule.legacy.clone().unwrap_or_default();
    legacy.revoke_event(&event)?;
    let status = status_of(&legacy, now);
    capsule.legacy = Some(legacy);
    capsule.updated_at = now;
    Ok(status)
}

fn ctx_for(caller: Principal, now: u64) -> PrincipalContext {
    PrincipalContext {
        principal: caller,
        groups: vec![],
        link: None,
        now_ns: now,
    }
}

fn can_view(memory: &Memory, ctx: &PrincipalContext, events: &ActiveEvents) -> bool {
    (effective_perm_mask_with_events(memory, ctx, events) & Perm::VIEW.bits()) != 0
}

/// What a grantee gets to see: the memory, but of the access list only the
/// entries about themselves (not who else was granted what).
fn for_grantee(mut memory: Memory, caller: &Principal) -> Memory {
    let me = PersonRef::Principal(*caller);
    memory
        .access_entries
        .retain(|e| e.is_public || e.person_ref.as_ref() == Some(&me));
    memory
}

/// Memory ids in the capsule the caller may view through access entries, sorted.
/// Not found when there are none: a stranger learns nothing about the capsule.
pub fn shared_memory_ids(
    capsule: &Capsule,
    caller: Principal,
    now: u64,
) -> Result<Vec<String>, Error> {
    let ctx = ctx_for(caller, now);
    let events = ActiveEvents::for_capsule(capsule, now);
    let mut ids: Vec<String> = capsule
        .memories
        .values()
        .filter(|m| can_view(m, &ctx, &events))
        .map(|m| m.id.clone())
        .collect();
    if ids.is_empty() {
        return Err(Error::NotFound);
    }
    ids.sort();
    Ok(ids)
}

pub fn list_shared(
    capsule: &Capsule,
    caller: Principal,
    cursor: Option<String>,
    limit: Option<u32>,
    now: u64,
) -> Result<crate::capsule_store::types::Page<MemoryHeader>, Error> {
    let ids = shared_memory_ids(capsule, caller, now)?;
    let limit = limit.unwrap_or(50).clamp(1, 100) as usize;
    let start = cursor.and_then(|c| c.parse::<usize>().ok()).unwrap_or(0);
    let items: Vec<MemoryHeader> = ids
        .iter()
        .skip(start)
        .take(limit)
        .filter_map(|id| capsule.memories.get(id))
        .map(|m| m.to_header())
        .collect();
    let end = start.saturating_add(items.len());
    Ok(crate::capsule_store::types::Page {
        items,
        next_cursor: if end < ids.len() {
            Some(end.to_string())
        } else {
            None
        },
    })
}

pub fn read_shared(
    capsule: &Capsule,
    caller: Principal,
    memory_id: &str,
    now: u64,
) -> Result<Memory, Error> {
    let memory = capsule.memories.get(memory_id).ok_or(Error::NotFound)?;
    let ctx = ctx_for(caller, now);
    let events = ActiveEvents::for_capsule(capsule, now);
    if !can_view(memory, &ctx, &events) {
        return Err(Error::NotFound);
    }
    Ok(for_grantee(memory.clone(), &caller))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capsule::domain::{
        AccessCondition, AccessEntry, GrantSource, ResourceRole,
    };
    use candid::{Decode, Encode};
    use std::collections::HashMap;

    const DAY: u64 = DAY_NS;

    fn p(n: u8) -> Principal {
        Principal::from_slice(&[n; 29])
    }
    fn owner() -> Principal {
        p(1)
    }

    fn capsule() -> Capsule {
        let mut owners = HashMap::new();
        owners.insert(
            PersonRef::Principal(owner()),
            crate::types::OwnerState {
                since: 0,
                last_activity_at: 0,
            },
        );
        Capsule {
            id: "c1".into(),
            subject: PersonRef::Principal(owner()),
            owners,
            controllers: HashMap::new(),
            connections: HashMap::new(),
            connection_groups: HashMap::new(),
            memories: HashMap::new(),
            galleries: HashMap::new(),
            folders: HashMap::new(),
            created_at: 0,
            updated_at: 0,
            bound_to_neon: false,
            inline_bytes_used: 0,
            has_advanced_settings: false,
            hosting_preferences: crate::types::HostingPreferences::default(),
            legacy: None,
        }
    }

    fn entry(who: Principal, role: ResourceRole, condition: AccessCondition) -> AccessEntry {
        AccessEntry {
            id: format!("e-{who}"),
            person_ref: Some(PersonRef::Principal(who)),
            is_public: false,
            grant_source: GrantSource::User,
            source_id: None,
            role,
            perm_mask: (Perm::VIEW | Perm::DOWNLOAD).bits(),
            invited_by_person_ref: None,
            created_at: 0,
            updated_at: 0,
            condition,
        }
    }

    fn after_death() -> AccessCondition {
        AccessCondition::EventTriggered {
            event: AccessEvent::AfterDeath,
        }
    }

    /// A capsule with one memory "m-heir" shared with `heir` after death, one
    /// "m-now" shared with `friend` immediately, and one private "m-private".
    fn capsule_with_memories(heir: Principal, friend: Principal) -> Capsule {
        let mut c = capsule();
        for (id, extra) in [
            ("m-heir", Some(entry(heir, ResourceRole::Guest, after_death()))),
            (
                "m-now",
                Some(entry(friend, ResourceRole::Guest, AccessCondition::Immediate)),
            ),
            ("m-private", None),
        ] {
            let mut m = crate::test_utils::create_test_memory();
            m.id = id.to_string();
            m.access_entries = vec![entry(
                owner(),
                ResourceRole::Owner,
                AccessCondition::Immediate,
            )];
            if let Some(e) = extra {
                m.access_entries.push(e);
            }
            c.memories.insert(id.to_string(), m);
        }
        c
    }

    // ---- settings -----------------------------------------------------------

    #[test]
    fn only_an_owner_sets_settings_and_strangers_see_not_found() {
        let mut c = capsule();
        assert_eq!(
            settings_set(&mut c, p(9), vec![p(2)], 1, 0, 10),
            Err(Error::NotFound)
        );
        assert!(c.legacy.is_none());
        let st = settings_set(&mut c, owner(), vec![p(2), p(3), p(2)], 2, 86_400, 10).unwrap();
        assert_eq!(st.trustees.len(), 2); // deduplicated
        assert_eq!(st.waiting_period_secs, 86_400);
        assert_eq!(status(&c, p(9), 10), Err(Error::NotFound));
        assert!(status(&c, p(2), 10).is_ok()); // a trustee may look
    }

    #[test]
    fn settings_are_validated() {
        let mut c = capsule();
        let bad = |c: &mut Capsule, t: Vec<Principal>, n: u32| {
            matches!(
                settings_set(c, owner(), t, n, 0, 1),
                Err(Error::InvalidArgument(_))
            )
        };
        assert!(bad(&mut c, vec![p(2)], 0)); // zero confirmations
        assert!(bad(&mut c, vec![p(2)], 2)); // more than trustees
        assert!(bad(&mut c, vec![owner()], 1)); // owner as own trustee
        assert!(bad(&mut c, vec![Principal::anonymous()], 1));
        assert!(bad(&mut c, (10..21).map(p).collect(), 1)); // too many
        assert!(c.legacy.is_none()); // nothing was stored by the failed calls
    }

    // ---- death --------------------------------------------------------------

    #[test]
    fn death_needs_enough_trustees_and_the_waiting_period() {
        let mut c = capsule();
        settings_set(&mut c, owner(), vec![p(2), p(3), p(4)], 2, 30 * 86_400, 0).unwrap();

        // not a trustee
        assert_eq!(confirm_death(&mut c, p(9), DAY), Err(Error::NotFound));
        // the owner cannot confirm their own death
        assert_eq!(confirm_death(&mut c, owner(), DAY), Err(Error::NotFound));

        let st = confirm_death(&mut c, p(2), DAY).unwrap();
        assert_eq!(st.death_effective_at, None); // one of two
        confirm_death(&mut c, p(2), 2 * DAY).unwrap(); // same trustee again: no effect
        assert_eq!(c.legacy.as_ref().unwrap().death_confirmations.len(), 1);

        let st = confirm_death(&mut c, p(3), 3 * DAY).unwrap();
        assert_eq!(st.death_effective_at, Some(33 * DAY));
        assert!(!st.death_in_effect);

        let l = c.legacy.as_ref().unwrap();
        assert!(!l.death_in_effect(33 * DAY - 1));
        assert!(l.death_in_effect(33 * DAY));
    }

    #[test]
    fn the_owner_can_cancel_and_a_trustee_can_withdraw_only_before_effect() {
        let mut c = capsule();
        settings_set(&mut c, owner(), vec![p(2), p(3)], 2, 10 * 86_400, 0).unwrap();
        confirm_death(&mut c, p(2), DAY).unwrap();
        confirm_death(&mut c, p(3), DAY).unwrap();

        // withdraw during the waiting period: the clock stops
        let st = withdraw_confirmation(&mut c, p(3), 2 * DAY).unwrap();
        assert_eq!(st.death_effective_at, None);
        // confirming again restarts the waiting period from that moment
        let st = confirm_death(&mut c, p(3), 5 * DAY).unwrap();
        assert_eq!(st.death_effective_at, Some(15 * DAY));

        // once in effect a trustee cannot undo it
        assert!(matches!(
            withdraw_confirmation(&mut c, p(2), 20 * DAY),
            Err(Error::Conflict(_))
        ));
        // a stranger cannot cancel; the owner can, even after it took effect
        assert_eq!(cancel_death(&mut c, p(9), 20 * DAY), Err(Error::NotFound));
        assert_eq!(cancel_death(&mut c, p(2), 20 * DAY), Err(Error::NotFound));
        let st = cancel_death(&mut c, owner(), 20 * DAY).unwrap();
        assert!(!st.death_in_effect);
        assert!(st.death_confirmations.is_empty());
    }

    #[test]
    fn removing_a_trustee_drops_their_confirmation() {
        let mut c = capsule();
        settings_set(&mut c, owner(), vec![p(2), p(3)], 1, 0, 0).unwrap();
        confirm_death(&mut c, p(2), 1).unwrap();
        assert!(c.legacy.as_ref().unwrap().death_in_effect(1)); // waiting period 0
        let st = settings_set(&mut c, owner(), vec![p(3)], 1, 0, 2).unwrap();
        assert!(!st.death_in_effect);
        assert!(st.death_confirmations.is_empty());
    }

    #[test]
    fn without_trustees_nobody_can_start_it() {
        let mut c = capsule();
        settings_set(&mut c, owner(), vec![], 1, 0, 0).unwrap();
        assert_eq!(confirm_death(&mut c, p(2), 1), Err(Error::NotFound));
        assert!(!c.legacy.as_ref().unwrap().death_in_effect(u64::MAX));
    }

    // ---- other events -------------------------------------------------------

    #[test]
    fn owner_records_and_revokes_events_but_not_death_or_computed_ones() {
        let mut c = capsule();
        assert_eq!(
            event_record(&mut c, p(9), AccessEvent::Wedding, 1),
            Err(Error::NotFound)
        );
        event_record(&mut c, owner(), AccessEvent::Wedding, 1).unwrap();
        event_record(&mut c, owner(), AccessEvent::Wedding, 2).unwrap(); // idempotent
        assert_eq!(c.legacy.as_ref().unwrap().occurred_events.len(), 1);
        for e in [
            AccessEvent::AfterDeath,
            AccessEvent::CapsuleMaturity(1),
            AccessEvent::ConnectionCount(1),
            AccessEvent::Custom("  ".into()),
        ] {
            assert!(matches!(
                event_record(&mut c, owner(), e, 3),
                Err(Error::InvalidArgument(_))
            ));
        }
        let ev = ActiveEvents::for_capsule(&c, 5);
        assert!(ev.is_active(&AccessEvent::Wedding));
        assert!(!ev.is_active(&AccessEvent::Graduation));
        assert!(!ev.is_active(&AccessEvent::AfterDeath));
        event_revoke(&mut c, owner(), AccessEvent::Wedding, 6).unwrap();
        assert!(!ActiveEvents::for_capsule(&c, 7).is_active(&AccessEvent::Wedding));
        assert_eq!(
            event_revoke(&mut c, owner(), AccessEvent::Wedding, 8),
            Err(Error::NotFound)
        );
    }

    #[test]
    fn computed_events_follow_the_capsule() {
        let c = capsule(); // created_at 0, no connections
        let ev = ActiveEvents::for_capsule(&c, 2 * YEAR_NS);
        assert!(ev.is_active(&AccessEvent::CapsuleMaturity(2)));
        assert!(!ev.is_active(&AccessEvent::CapsuleMaturity(3)));
        assert!(ev.is_active(&AccessEvent::ConnectionCount(0)));
        assert!(!ev.is_active(&AccessEvent::ConnectionCount(1)));
        assert!(!ActiveEvents::none().is_active(&AccessEvent::AfterDeath));
    }

    // ---- access through the events -----------------------------------------

    #[test]
    fn an_heir_sees_nothing_before_and_their_memory_after() {
        let (heir, friend) = (p(5), p(6));
        let mut c = capsule_with_memories(heir, friend);
        settings_set(&mut c, owner(), vec![p(2)], 1, 7 * 86_400, 0).unwrap();

        // before: the heir is a stranger, the friend sees their one memory
        assert_eq!(shared_memory_ids(&c, heir, DAY), Err(Error::NotFound));
        assert_eq!(read_shared(&c, heir, "m-heir", DAY), Err(Error::NotFound));
        assert_eq!(shared_memory_ids(&c, friend, DAY).unwrap(), vec!["m-now"]);

        // confirmed, but still in the waiting period
        confirm_death(&mut c, p(2), DAY).unwrap();
        assert_eq!(read_shared(&c, heir, "m-heir", 7 * DAY), Err(Error::NotFound));

        // in effect
        let t = 8 * DAY;
        assert_eq!(shared_memory_ids(&c, heir, t).unwrap(), vec!["m-heir"]);
        let m = read_shared(&c, heir, "m-heir", t).unwrap();
        // the heir sees their own entry only, not the owner's or anyone else's
        assert_eq!(m.access_entries.len(), 1);
        assert_eq!(
            m.access_entries[0].person_ref,
            Some(PersonRef::Principal(heir))
        );
        // and still not the other memories
        assert_eq!(read_shared(&c, heir, "m-private", t), Err(Error::NotFound));
        assert_eq!(read_shared(&c, heir, "m-now", t), Err(Error::NotFound));
        // a trustee confirmed the death but was granted nothing
        assert_eq!(shared_memory_ids(&c, p(2), t), Err(Error::NotFound));

        // the owner turns out to be alive: access closes again
        cancel_death(&mut c, owner(), 9 * DAY).unwrap();
        assert_eq!(read_shared(&c, heir, "m-heir", 10 * DAY), Err(Error::NotFound));
    }

    #[test]
    fn an_owner_role_that_waits_for_death_grants_nothing_early() {
        // Regression: the ownership short-circuit ignored the entry's condition,
        // so an heir entered as future Owner had full rights immediately.
        let heir = p(5);
        let mut c = capsule();
        let mut m = crate::test_utils::create_test_memory();
        m.id = "m".into();
        m.access_entries = vec![entry(heir, ResourceRole::Owner, after_death())];
        c.memories.insert("m".into(), m);
        settings_set(&mut c, owner(), vec![p(2)], 1, 0, 0).unwrap();

        let mask = |c: &Capsule, now: u64| {
            effective_perm_mask_with_events(
                c.memories.get("m").unwrap(),
                &ctx_for(heir, now),
                &ActiveEvents::for_capsule(c, now),
            )
        };
        assert_eq!(mask(&c, 1), 0);
        confirm_death(&mut c, p(2), 2).unwrap();
        assert_ne!(mask(&c, 3) & Perm::OWN.bits(), 0);
        // and without capsule context (the plain check) it stays closed
        assert_eq!(
            crate::capsule::domain::effective_perm_mask(
                c.memories.get("m").unwrap(),
                &ctx_for(heir, 3)
            ),
            0
        );
    }

    #[test]
    fn shared_list_pages_in_a_stable_order() {
        let friend = p(6);
        let mut c = capsule();
        for i in 0..5 {
            let mut m = crate::test_utils::create_test_memory();
            m.id = format!("m{i}");
            m.access_entries = vec![entry(friend, ResourceRole::Guest, AccessCondition::Immediate)];
            c.memories.insert(m.id.clone(), m);
        }
        let page = list_shared(&c, friend, None, Some(2), 1).unwrap();
        assert_eq!(page.items.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(), ["m0", "m1"]);
        assert_eq!(page.next_cursor.as_deref(), Some("2"));
        let last = list_shared(&c, friend, Some("4".into()), Some(2), 1).unwrap();
        assert_eq!(last.items.len(), 1);
        assert_eq!(last.next_cursor, None);
    }

    // ---- storage ------------------------------------------------------------

    #[test]
    fn records_stored_before_the_legacy_field_still_decode() {
        // Capsule gained `legacy: Option<LegacyState>`. Candid decodes a record
        // that lacks an optional field as None; this pins that for our type.
        #[derive(CandidType, Deserialize)]
        struct Old {
            id: String,
        }
        #[derive(CandidType, Deserialize)]
        struct New {
            id: String,
            legacy: Option<LegacyState>,
        }
        let bytes = Encode!(&(1u16, Old { id: "c".into() })).unwrap();
        let (ver, new) = Decode!(&bytes, (u16, New)).unwrap();
        assert_eq!(ver, 1);
        assert_eq!(new.id, "c");
        assert!(new.legacy.is_none());
    }
}
