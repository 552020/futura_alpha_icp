use crate::types::*;
use candid::Principal;
use ic_cdk::api::msg_caller;
use std::collections::{HashMap, HashSet};

// Centralized storage for capsules and admin data
thread_local! {
    // Admin storage - auto-bootstrap first caller as admin
    static ADMINS: std::cell::RefCell<HashSet<Principal>> = std::cell::RefCell::new(HashSet::new());

    // Capsule storage (centralized data storage)
    static CAPSULES: std::cell::RefCell<HashMap<String, Capsule>> = std::cell::RefCell::new(HashMap::new());

    // Nonce proof storage for II authentication
    static NONCE_PROOFS: std::cell::RefCell<HashMap<String, (Principal, u64)>> = std::cell::RefCell::new(HashMap::new());
}

// Helper function to generate secure codes
pub fn generate_secure_code() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    ic_cdk::api::time().hash(&mut hasher);
    msg_caller().hash(&mut hasher);

    format!("{:x}", hasher.finish())
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
