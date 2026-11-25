use crate::http::core_types::SecretStore;
use candid::{CandidType, Decode, Encode};
use ic_cdk::management_canister::raw_rand;
use ic_stable_structures::{
    memory_manager::VirtualMemory,
    DefaultMemoryImpl, StableCell, Storable,
};
use once_cell::unsync::OnceCell; // unsync version
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

type Mem = VirtualMemory<DefaultMemoryImpl>;

#[derive(Clone, Copy, CandidType, Serialize, Deserialize)]
struct Secrets {
    current: [u8; 32],
    previous: [u8; 32], // zero = "none"
    version: u32,
}

impl Storable for Secrets {
    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 256,
            is_fixed_size: false,
        };

    fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(&bytes, Self).unwrap()
    }
}

thread_local! {
    static SECRET_CELL: RefCell<OnceCell<StableCell<Secrets, Mem>>> = const { RefCell::new(OnceCell::new()) };
}

pub async fn init() {
    // The one global memory manager: a second manager over the same stable
    // memory hands out buckets the first one does not know about.
    let mem = crate::memory::MM.with(|m| m.borrow().get(crate::memory::MEM_SECRETS));

    // Use deterministic initialization during init (system calls not allowed)
    let seeded = Secrets {
        current: deterministic_key(),
        previous: [0; 32],
        version: 1,
    };
    let cell = StableCell::init(mem, seeded).expect("init secret cell");

    SECRET_CELL.with(|c| {
        c.borrow_mut().set(cell).ok();
    });
}

pub fn post_upgrade() {
    // The one global memory manager: a second manager over the same stable
    // memory hands out buckets the first one does not know about.
    let mem = crate::memory::MM.with(|m| m.borrow().get(crate::memory::MEM_SECRETS));
    let cell = StableCell::init(
        mem,
        Secrets {
            current: [0; 32],
            previous: [0; 32],
            version: 0,
        },
    )
    .expect("open cell");
    SECRET_CELL.with(|c| {
        c.borrow_mut().set(cell).ok();
    });
}

// Adapter-facing API
pub fn current_key() -> [u8; 32] {
    SECRET_CELL.with(|c| {
        let cell = c.borrow();
        let cell = cell.get().expect("secret cell not initialized");
        cell.get().current
    })
}

/// From this version on the key is random
const RANDOM_KEY_VERSION: u32 = 2;

/// Is this key still a placeholder that outsiders can work out?
/// Version 1 is the key made at install from canister id and time, both
/// public; version 0 is the all-zero key of a canister older than this store.
fn is_guessable(version: u32) -> bool {
    version < RANDOM_KEY_VERSION
}

/// Replace a guessable key by a random one, right after `init` or
/// `post_upgrade` return. Randomness comes from the network (`raw_rand`),
/// which cannot be called during install or upgrade themselves, hence the
/// timer. Links signed with the old key stop working; they live minutes.
pub fn schedule_random_key() {
    if !is_guessable(key_version()) {
        return;
    }
    ic_cdk_timers::set_timer(std::time::Duration::ZERO, || {
        ic_cdk::futures::spawn(async {
            let random = match raw_rand().await {
                Ok(bytes) if bytes.len() >= 32 => bytes,
                other => {
                    ic_cdk::println!("signing key: no randomness ({other:?}), still guessable");
                    return;
                }
            };
            let mut current = [0u8; 32];
            current.copy_from_slice(&random[..32]);
            SECRET_CELL.with(|c| {
                let mut cell = c.borrow_mut();
                let cell = cell.get_mut().expect("secret cell not initialized");
                // Another timer may have been faster (init and a quick upgrade)
                if is_guessable(cell.get().version) {
                    cell.set(Secrets {
                        current,
                        previous: [0; 32], // never accept the guessable key again
                        version: RANDOM_KEY_VERSION,
                    })
                    .expect("store signing key");
                    ic_cdk::println!("signing key: replaced by a random one");
                }
            });
        });
    });
}

fn key_version() -> u32 {
    SECRET_CELL.with(|c| {
        let cell = c.borrow();
        cell.get().expect("secret cell not initialized").get().version
    })
}

fn deterministic_key() -> [u8; 32] {
    // A placeholder from canister id and time, for the moment between
    // install and the first timer: both are public, so `schedule_random_key`
    // replaces it at once.
    let canister_id = ic_cdk::api::canister_self();
    let time = ic_cdk::api::time();

    let mut key = [0u8; 32];
    let canister_bytes = canister_id.as_slice();
    let time_bytes = time.to_le_bytes();

    for i in 0..32 {
        key[i] = canister_bytes[i % canister_bytes.len()] ^ time_bytes[i % 8];
    }

    key
}

pub struct StableSecretStore;

impl SecretStore for StableSecretStore {
    fn get_key(&self) -> [u8; 32] {
        current_key()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_keys_from_before_the_random_one_are_replaced() {
        assert!(is_guessable(0), "the zero key of a canister older than the store");
        assert!(is_guessable(1), "the install-time placeholder");
        assert!(!is_guessable(RANDOM_KEY_VERSION));
        assert!(!is_guessable(RANDOM_KEY_VERSION + 1));
    }
}
