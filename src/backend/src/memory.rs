use crate::capsule_store::Store;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
use ic_stable_structures::DefaultMemoryImpl;
use std::cell::RefCell;

// ============================================================================
// STABLE MEMORY INFRASTRUCTURE - MVP Implementation
// ============================================================================

// Memory manager for multiple stable memory regions
// Memory type alias removed - no longer needed

// Memory IDs for different data types - All MemoryId constants in one place to prevent collisions
// Keep these sequential and document usage

// Capsule storage
pub const MEM_CAPSULES: MemoryId = MemoryId::new(0);
pub const MEM_CAPSULES_IDX_SUBJECT: MemoryId = MemoryId::new(1);
pub const MEM_CAPSULES_IDX_OWNER: MemoryId = MemoryId::new(2);

// Upload workflow
#[allow(dead_code)]
pub const MEM_SESSIONS: MemoryId = MemoryId::new(3);
#[allow(dead_code)]
pub const MEM_SESSIONS_CHUNKS: MemoryId = MemoryId::new(4);
#[allow(dead_code)]
pub const MEM_SESSIONS_COUNTER: MemoryId = MemoryId::new(5);

// Blob storage
pub const MEM_BLOBS: MemoryId = MemoryId::new(6);
pub const MEM_BLOB_META: MemoryId = MemoryId::new(7);
pub const MEM_BLOB_COUNTER: MemoryId = MemoryId::new(8);

// Admin storage
pub const MEM_ADMINS: MemoryId = MemoryId::new(9);

// Personal-canister owner (personal.rs); empty in shared mode
pub const MEM_PERSONAL_OWNER: MemoryId = MemoryId::new(10);
/// Personal mode: the principal that installed this canister (its factory)
pub const MEM_PERSONAL_FACTORY: MemoryId = MemoryId::new(12);

// MemoryId 11 is retired: it held the state of the canister factory that
// used to live in the backend. Do not reuse it; deployed canisters may still
// have bytes there.

// HTTP token secrets (http/adapters/secret_store.rs)
pub const MEM_SECRETS: MemoryId = MemoryId::new(42);

thread_local! {
    /// Global memory manager for all stable structures
    /// This ensures no MemoryId collisions across modules
    pub static MM: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

// ============================================================================
// STABLE MEMORY ACCESS FUNCTIONS
// ============================================================================

// Stable memory artifact storage functions
// with_stable_memory_artifacts functions removed - artifacts system deleted

// Legacy chunk data functions removed - using active chunk storage in upload/sessions.rs

// ============================================================================
// NEW TRAIT-BASED ACCESS FUNCTIONS (recommended approach)
// ============================================================================

/// Access capsules using Store enum (runtime polymorphism without dyn)
pub fn with_capsule_store<F, R>(f: F) -> R
where
    F: FnOnce(&Store) -> R,
{
    // ✅ PRODUCTION: Use Stable storage for data persistence across upgrades
    let store = Store::new_stable();
    f(&store)
}

/// Mutably access capsules using Store enum
pub fn with_capsule_store_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut Store) -> R,
{
    // ✅ PRODUCTION: Use Stable storage for data persistence across upgrades
    let mut store = Store::new_stable();
    f(&mut store)
}

// ============================================================================
// GLOBAL MEMORY MANAGER ACCESS FUNCTIONS (for StableStore)
// ============================================================================

// Get virtual memory for capsules from global memory manager

// Legacy memory access functions removed - using active capsule store system

// ============================================================================
// LEGACY ACCESS FUNCTIONS (backward compatibility during transition)
// ============================================================================

// Legacy HashMap capsule functions removed - using active CapsuleStore system

// ============================================================================
// BACKWARD COMPATIBILITY ALIASES
// ============================================================================

// ============================================================================
// CANISTER UPGRADE HOOKS - Stable Memory Persistence
// ============================================================================

// Note: pre_upgrade hook is defined in lib.rs to avoid conflicts
// The stable memory data will be automatically persisted by ic-stable-structures

// Note: post_upgrade hook is defined in lib.rs to avoid conflicts
// The stable memory data will be automatically restored by ic-stable-structures

