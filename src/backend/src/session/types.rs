use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use std::collections::BTreeSet;

/// Generic session ID (opaque to session layer)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SessionId(pub u64);

/// Generic session specification (no upload-specific fields)
#[derive(Debug, Clone)]
pub struct SessionSpec {
    pub chunk_size: usize,
    pub bytes_expected: u64,
    pub idem: String,
    pub owner: Principal, // opaque for session; not interpreted
}

/// Generic session metadata (minimal, no upload semantics)
#[derive(Debug, Clone)]
pub struct SessionMeta {
    #[allow(dead_code)] // Idempotency key - used for session deduplication
    pub idem: String,
    pub last_seen: u64, // timestamp in ms
    #[allow(dead_code)] // Session status tracking - used for session lifecycle management
    pub status: SessionStatus,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum SessionStatus {
    Pending,
    Committed { completed_at: u64 },
}

/// ByteSink trait for direct chunk writing (no buffering)
pub trait ByteSink {
    fn write_at(&mut self, offset: u64, data: &[u8]) -> Result<(), crate::types::Error>;
}

/// Clock trait for time operations
pub trait Clock {
    fn now_ms(&self) -> u64;
}

/// Generic session state (no upload-specific fields)
#[derive(Debug, Clone)]
pub struct Session {
    #[allow(dead_code)] // Session owner - used for access control and cleanup
    pub owner: Vec<u8>, // Principal as bytes (opaque)
    pub chunk_size: usize,
    #[allow(dead_code)] // Expected total bytes - used for upload validation
    pub bytes_expected: u64,
    pub bytes_received: u64,
    pub received_idxs: BTreeSet<u32>,
    pub session_meta: SessionMeta,
    // REMOVED: chunks: BTreeMap<u32, Vec<u8>> - no buffering!
}

impl SessionId {
    pub fn new() -> Self {
        use std::cell::Cell;
        thread_local! {
            static SESSION_COUNTER: Cell<u64> = Cell::new(1);
        }

        let id = SESSION_COUNTER.with(|counter| {
            let current = counter.get();
            counter.set(current + 1);
            current
        });
        SessionId(id)
    }
}

impl SessionMeta {
    pub fn new(idem: String, now_ms: u64) -> Self {
        Self {
            idem,
            last_seen: now_ms,
            status: SessionStatus::Pending,
        }
    }
}
