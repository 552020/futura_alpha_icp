use crate::canister_factory::PersonalCanisterCreationStateData;
use crate::capsule_store::Store;
use candid::Principal;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
use ic_stable_structures::DefaultMemoryImpl;
use std::cell::RefCell;
use std::collections::HashMap;

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
pub const MEM_SESSIONS: MemoryId = MemoryId::new(3);
pub const MEM_SESSIONS_CHUNKS: MemoryId = MemoryId::new(4);
pub const MEM_SESSIONS_COUNTER: MemoryId = MemoryId::new(5);

// Blob storage
pub const MEM_BLOBS: MemoryId = MemoryId::new(6);
pub const MEM_BLOB_META: MemoryId = MemoryId::new(7);
pub const MEM_BLOB_COUNTER: MemoryId = MemoryId::new(8);

// Admin storage
pub const MEM_ADMINS: MemoryId = MemoryId::new(9);

thread_local! {
    /// Global memory manager for all stable structures
    /// This ensures no MemoryId collisions across modules
    pub static MM: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    // Nonce proof storage for II authentication
    static NONCE_PROOFS: std::cell::RefCell<HashMap<String, (Principal, u64)>> = std::cell::RefCell::new(HashMap::new());

    // Migration state storage (only available with migration feature)
    #[cfg(feature = "migration")]
    static MIGRATION_STATE: std::cell::RefCell<PersonalCanisterCreationStateData> = std::cell::RefCell::new(PersonalCanisterCreationStateData::default());
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

/// Get virtual memory for capsules from global memory manager

// Legacy memory access functions removed - using active capsule store system

// ============================================================================
// LEGACY ACCESS FUNCTIONS (backward compatibility during transition)
// ============================================================================

// Legacy HashMap capsule functions removed - using active CapsuleStore system

// ============================================================================
// BACKWARD COMPATIBILITY ALIASES
// ============================================================================

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
    F: FnOnce(&mut PersonalCanisterCreationStateData) -> R,
{
    MIGRATION_STATE.with(|state| f(&mut state.borrow_mut()))
}

#[cfg(feature = "migration")]
pub fn with_migration_state<F, R>(f: F) -> R
where
    F: FnOnce(&PersonalCanisterCreationStateData) -> R,
{
    MIGRATION_STATE.with(|state| f(&state.borrow()))
}
// ============================================================================
// CANISTER UPGRADE HOOKS - Stable Memory Persistence
// ============================================================================

// Note: pre_upgrade hook is defined in lib.rs to avoid conflicts
// The stable memory data will be automatically persisted by ic-stable-structures

// Note: post_upgrade hook is defined in lib.rs to avoid conflicts
// The stable memory data will be automatically restored by ic-stable-structures

// ============================================================================
// HELPER FUNCTIONS FOR STABLE MEMORY OPERATIONS
// ============================================================================

// Helper to clear all stable memory data (for emergency recovery)

pub fn clear_all_stable_memory() -> Result<(), String> {
    // Note: clear_new() modifies in place and returns ()
    // STABLE_MEMORY_ARTIFACTS removed - artifacts system deleted
    // STABLE_UPLOAD_SESSIONS removed - using new SessionStore
    // STABLE_CHUNK_DATA removed - using active chunk storage in upload/sessions.rs
    Ok(())
}
