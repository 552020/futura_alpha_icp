use ic_stable_structures::{DefaultMemoryImpl, memory_manager::{MemoryManager, MemoryId}};
use std::cell::RefCell;

thread_local! {
    /// Global memory manager for all stable structures
    /// This ensures no MemoryId collisions across modules
    pub static MM: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

// All MemoryId constants in one place to prevent collisions
// Keep these sequential and document usage

// Capsule storage (existing)
#[allow(dead_code)]
pub const MEM_CAPSULES: MemoryId = MemoryId::new(0);
#[allow(dead_code)]
pub const MEM_IDX_SUBJECT: MemoryId = MemoryId::new(1);

// Upload workflow (new)
pub const MEM_SESSIONS: MemoryId = MemoryId::new(2);
pub const MEM_CHUNKS: MemoryId = MemoryId::new(3);
pub const MEM_BLOBS: MemoryId = MemoryId::new(4);
pub const MEM_BLOB_META: MemoryId = MemoryId::new(5);
pub const MEM_SESSION_COUNTER: MemoryId = MemoryId::new(6);
pub const MEM_BLOB_COUNTER: MemoryId = MemoryId::new(7);

// Reserved for future use (8-15)
// Add new MemoryIds here to maintain sequential allocation
