use crate::http::core_types::SecretStore;
use candid::{CandidType, Decode, Encode};
use ic_cdk::management_canister::raw_rand;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
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

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(&bytes, Self).unwrap()
    }
}

thread_local! {
    static SECRET_CELL: RefCell<OnceCell<StableCell<Secrets, Mem>>> = RefCell::new(OnceCell::new());
}

pub async fn init() {
    let mm = MemoryManager::init(DefaultMemoryImpl::default());
    let mem = mm.get(MemoryId::new(42));

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
    let mm = MemoryManager::init(DefaultMemoryImpl::default());
    let mem = mm.get(MemoryId::new(42));
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

#[allow(dead_code)]
pub async fn rotate_secret() {
    SECRET_CELL.with(|c| {
        let cell = c.borrow();
        let cell = cell.get().expect("secret cell not initialized");
        let mut s = cell.get().clone();
        s.previous = s.current;
        // We can't await here; do an outer async helper that calls set()
        // So: provide a separate async API to do set() after you fetch randomness
        ic_cdk::futures::spawn_017_compat(async move {
            let new_key = random_32().await;
            s.current = new_key;
            s.version = s.version.wrapping_add(1);
            // Re-open to set (StableCell doesn't need reopening; you need mut access)
            // Workaround: reopen inside post_upgrade/init or store a mutable handle in thread_local
            // Simpler: expose a `set_secrets(new: Secrets)` function also using SECRET_CELL.with()
        });
    });
}

#[allow(dead_code)]
async fn random_32() -> [u8; 32] {
    let r = raw_rand().await.expect("raw_rand");
    let mut out = [0u8; 32];
    out.copy_from_slice(&r[..32]);
    out
}

fn deterministic_key() -> [u8; 32] {
    // Generate deterministic key based on canister ID and time
    // This is used during initialization when system calls are not allowed
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
