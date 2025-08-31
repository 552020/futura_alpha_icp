#[cfg(feature = "migration")]
use crate::canister_factory::MigrationStateData;
use crate::types::*;
use candid::Principal;
use std::collections::{HashMap, HashSet};

// Centralized storage for capsules and admin data
thread_local! {
    // Admin storage - auto-bootstrap first caller as admin
    static ADMINS: std::cell::RefCell<HashSet<Principal>> = std::cell::RefCell::new(HashSet::new());

    // Capsule storage (centralized data storage)
    static CAPSULES: std::cell::RefCell<HashMap<String, Capsule>> = std::cell::RefCell::new(HashMap::new());

    // Nonce proof storage for II authentication
    static NONCE_PROOFS: std::cell::RefCell<HashMap<String, (Principal, u64)>> = std::cell::RefCell::new(HashMap::new());

    // Migration state storage (only available with migration feature)
    #[cfg(feature = "migration")]
    static MIGRATION_STATE: std::cell::RefCell<MigrationStateData> = std::cell::RefCell::new(MigrationStateData::default());
}

// Access functions for centralized storage
pub fn with_capsules_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut HashMap<String, Capsule>) -> R,
{
    CAPSULES.with(|capsules| f(&mut capsules.borrow_mut()))
}

pub fn with_capsules<F, R>(f: F) -> R
where
    F: FnOnce(&HashMap<String, Capsule>) -> R,
{
    CAPSULES.with(|capsules| f(&capsules.borrow()))
}

pub fn with_admins_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut HashSet<Principal>) -> R,
{
    ADMINS.with(|admins| f(&mut admins.borrow_mut()))
}

pub fn with_admins<F, R>(f: F) -> R
where
    F: FnOnce(&HashSet<Principal>) -> R,
{
    ADMINS.with(|admins| f(&admins.borrow()))
}

// Nonce proof functions for II authentication
pub fn store_nonce_proof(nonce: String, principal: Principal, timestamp: u64) -> bool {
    NONCE_PROOFS.with(|proofs| {
        proofs.borrow_mut().insert(nonce, (principal, timestamp));
    });
    true
}

pub fn get_nonce_proof(nonce: String) -> Option<Principal> {
    NONCE_PROOFS.with(|proofs| proofs.borrow().get(&nonce).map(|(principal, _)| *principal))
}

// Migration state access functions (only available with migration feature)
#[cfg(feature = "migration")]
pub fn with_migration_state_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut MigrationStateData) -> R,
{
    MIGRATION_STATE.with(|state| f(&mut state.borrow_mut()))
}

#[cfg(feature = "migration")]
pub fn with_migration_state<F, R>(f: F) -> R
where
    F: FnOnce(&MigrationStateData) -> R,
{
    MIGRATION_STATE.with(|state| f(&state.borrow()))
}
