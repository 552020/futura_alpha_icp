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
//!
//! A personal canister also remembers who installed it: the factory. The
//! factory is not a controller after the hand-off, so it cannot ask the
//! network for this canister's cycles. Instead the canister tells it
//! (`cycles_balance`, `cycles_liquid_balance`), and the factory tops it up
//! when it runs low. Depositing cycles needs no rights at all.

use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::{memory_manager::VirtualMemory, DefaultMemoryImpl, StableCell};
use serde::Serialize;
use std::cell::RefCell;

use crate::memory::{MEM_PERSONAL_FACTORY, MEM_PERSONAL_OWNER, MM};
use crate::types::Error;

type Mem = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    // Principal bytes; empty means shared mode
    static OWNER: RefCell<StableCell<Vec<u8>, Mem>> = RefCell::new(
        StableCell::init(MM.with(|m| m.borrow().get(MEM_PERSONAL_OWNER)), Vec::new())
            .expect("init personal owner cell"),
    );
    // Principal bytes of the installer; empty in shared mode and in personal
    // canisters installed before this cell existed
    static FACTORY: RefCell<StableCell<Vec<u8>, Mem>> = RefCell::new(
        StableCell::init(MM.with(|m| m.borrow().get(MEM_PERSONAL_FACTORY)), Vec::new())
            .expect("init personal factory cell"),
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

/// Called from `init` only. `installer` is the caller of `init`: whoever
/// sent `install_code`, which for a personal canister is its factory.
pub fn init(arg: Option<BackendInitArg>, installer: Principal) {
    if let Some(owner) = arg.and_then(|a| a.owner) {
        if owner == Principal::anonymous() {
            ic_cdk::trap("the anonymous principal cannot own a personal canister");
        }
        set_owner(Some(owner));
        set_factory(Some(installer));
    }
}

fn set_factory(factory: Option<Principal>) {
    let bytes = factory.map(|p| p.as_slice().to_vec()).unwrap_or_default();
    FACTORY.with(|c| {
        c.borrow_mut().set(bytes).expect("store personal factory");
    });
}

/// The factory that installed this personal canister
pub fn factory() -> Option<Principal> {
    FACTORY.with(|c| {
        let cell = c.borrow();
        let bytes = cell.get();
        if bytes.is_empty() {
            None
        } else {
            Some(Principal::from_slice(bytes))
        }
    })
}

/// Owner only: name the factory that may read this canister's cycles, or
/// none. For personal canisters installed before the factory was remembered,
/// and for an owner who wants no top-ups.
pub fn set_factory_by_owner(caller: &Principal, factory: Option<Principal>) -> Result<(), Error> {
    match owner() {
        Some(owner) if &owner == caller => {
            if factory == Some(Principal::anonymous()) {
                return Err(Error::InvalidArgument(
                    "the anonymous principal cannot be the factory".to_string(),
                ));
            }
            set_factory(factory);
            Ok(())
        }
        _ => Err(Error::Unauthorized),
    }
}

/// May this caller read the canister's cycles? In a personal canister the
/// owner and the factory; in the shared canister its controllers.
pub fn may_read_cycles(caller: &Principal, is_controller: bool) -> bool {
    match owner() {
        Some(owner) => &owner == caller || factory().as_ref() == Some(caller),
        None => is_controller,
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

    #[test]
    fn cycles_are_read_by_owner_and_factory_only() {
        let (owner_p, factory_p, other) = (
            Principal::from_slice(&[1; 29]),
            Principal::from_slice(&[3; 10]),
            Principal::from_slice(&[2; 29]),
        );
        // shared canister: controllers only
        set_owner(None);
        set_factory(None);
        assert!(may_read_cycles(&other, true));
        assert!(!may_read_cycles(&other, false));

        // personal canister installed before the factory was remembered
        set_owner(Some(owner_p));
        assert!(may_read_cycles(&owner_p, false));
        assert!(!may_read_cycles(&factory_p, false));

        set_factory(Some(factory_p));
        assert_eq!(factory(), Some(factory_p));
        assert!(may_read_cycles(&owner_p, false));
        assert!(may_read_cycles(&factory_p, false));
        assert!(!may_read_cycles(&other, false));
        assert!(!may_read_cycles(&other, true), "a controller that is not the owner");
        assert!(!may_read_cycles(&Principal::anonymous(), false));
        set_owner(None);
        set_factory(None);
    }

    #[test]
    fn only_the_owner_names_the_factory() {
        let (owner_p, factory_p, other) = (
            Principal::from_slice(&[1; 29]),
            Principal::from_slice(&[3; 10]),
            Principal::from_slice(&[2; 29]),
        );
        // shared canister: nobody
        set_owner(None);
        set_factory(None);
        assert_eq!(set_factory_by_owner(&other, Some(factory_p)), Err(Error::Unauthorized));

        set_owner(Some(owner_p));
        assert_eq!(set_factory_by_owner(&other, Some(other)), Err(Error::Unauthorized));
        assert_eq!(set_factory_by_owner(&factory_p, Some(factory_p)), Err(Error::Unauthorized));
        assert!(set_factory_by_owner(&owner_p, Some(Principal::anonymous())).is_err());
        assert_eq!(factory(), None);

        assert_eq!(set_factory_by_owner(&owner_p, Some(factory_p)), Ok(()));
        assert!(may_read_cycles(&factory_p, false));
        assert_eq!(set_factory_by_owner(&owner_p, None), Ok(()));
        assert!(!may_read_cycles(&factory_p, false));
        set_owner(None);
    }
}
