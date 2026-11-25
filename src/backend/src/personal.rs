//! Canister mode: shared (many users, one capsule each) or personal (one owner).
//!
//! The same wasm runs both. A personal canister is created by the factory for
//! one user and installed with `init(opt record { owner = opt principal })`.
//! In personal mode only the owner can create capsules, so nobody else can
//! store data in the canister; everything a capsule owner may do (grant
//! access, name trustees) works as before, and people the owner granted
//! access read through the shared-read endpoints.
//!
//! The owner is kept in stable memory and survives upgrades. It can only be
//! set at install time: there is no endpoint that changes it.

use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::{memory_manager::VirtualMemory, DefaultMemoryImpl, StableCell};
use serde::Serialize;
use std::cell::RefCell;

use crate::memory::{MEM_PERSONAL_OWNER, MM};
use crate::types::Error;

type Mem = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    // Principal bytes; empty means shared mode
    static OWNER: RefCell<StableCell<Vec<u8>, Mem>> = RefCell::new(
        StableCell::init(MM.with(|m| m.borrow().get(MEM_PERSONAL_OWNER)), Vec::new())
            .expect("init personal owner cell"),
    );
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, Default)]
pub struct BackendInitArg {
    /// Set by the factory: makes this a personal canister of that principal
    pub owner: Option<Principal>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum CanisterMode {
    Shared,
    Personal { owner: Principal },
}

/// Called from `init` only.
pub fn init(arg: Option<BackendInitArg>) {
    if let Some(owner) = arg.and_then(|a| a.owner) {
        if owner == Principal::anonymous() {
            ic_cdk::trap("the anonymous principal cannot own a personal canister");
        }
        set_owner(Some(owner));
    }
}

fn set_owner(owner: Option<Principal>) {
    let bytes = owner.map(|p| p.as_slice().to_vec()).unwrap_or_default();
    OWNER.with(|c| {
        c.borrow_mut().set(bytes).expect("store personal owner");
    });
}

pub fn owner() -> Option<Principal> {
    OWNER.with(|c| {
        let cell = c.borrow();
        let bytes = cell.get();
        if bytes.is_empty() {
            None
        } else {
            Some(Principal::from_slice(bytes))
        }
    })
}

pub fn mode() -> CanisterMode {
    match owner() {
        Some(owner) => CanisterMode::Personal { owner },
        None => CanisterMode::Shared,
    }
}

/// May this caller create a capsule here?
pub fn check_can_create_capsule(caller: &Principal) -> Result<(), Error> {
    match owner() {
        Some(owner) if &owner != caller => Err(Error::Unauthorized),
        _ => Ok(()),
    }
}

/// Updates that people other than the owner may send to a personal canister:
/// trustees confirming or withdrawing, and the sign-in handshake.
const OPEN_UPDATES: [&str; 3] = [
    "legacy_confirm_death",
    "legacy_withdraw_confirmation",
    "register_with_nonce",
];

/// Should an update call from outside be accepted at all?
pub fn accepts_update(caller: &Principal, method: &str) -> bool {
    match owner() {
        None => true, // shared canister: every endpoint does its own checks
        Some(owner) => &owner == caller || OPEN_UPDATES.contains(&method),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_mode_lets_anyone_create_and_personal_mode_only_the_owner() {
        let (owner_p, other) = (
            Principal::from_slice(&[1; 29]),
            Principal::from_slice(&[2; 29]),
        );
        set_owner(None);
        assert_eq!(mode(), CanisterMode::Shared);
        assert!(check_can_create_capsule(&other).is_ok());

        set_owner(Some(owner_p));
        assert_eq!(mode(), CanisterMode::Personal { owner: owner_p });
        assert!(check_can_create_capsule(&owner_p).is_ok());
        assert_eq!(check_can_create_capsule(&other), Err(Error::Unauthorized));
        set_owner(None);
    }

    #[test]
    fn a_personal_canister_accepts_updates_from_its_owner_only() {
        let (owner_p, other) = (
            Principal::from_slice(&[1; 29]),
            Principal::from_slice(&[2; 29]),
        );
        set_owner(None);
        assert!(accepts_update(&other, "memories_create"));

        set_owner(Some(owner_p));
        assert!(accepts_update(&owner_p, "memories_create"));
        assert!(!accepts_update(&other, "memories_create"));
        assert!(!accepts_update(&other, "uploads_put_chunk"));
        assert!(!accepts_update(&Principal::anonymous(), "capsules_create"));
        // what a trustee needs
        assert!(accepts_update(&other, "legacy_confirm_death"));
        assert!(accepts_update(&other, "legacy_withdraw_confirmation"));
        set_owner(None);
    }
}
