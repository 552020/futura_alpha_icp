use crate::session::service::SessionService;
use crate::session::types::{ByteSink, Clock, SessionId, SessionMeta, SessionSpec};
use crate::types::{CapsuleId, Error};
use std::cell::RefCell;

// Thread-local storage for the session service
thread_local! {
    static SESSION_SERVICE: RefCell<SessionService> = RefCell::new(SessionService::new());
}

/// IC adapter for session management
/// This provides the IC-specific interface while delegating to the generic SessionService
pub struct SessionAdapter;

/// IC Clock implementation
pub struct ICClock;

impl Clock for ICClock {
    fn now_ms(&self) -> u64 {
        ic_cdk::api::time() / 1_000_000 // Convert nanoseconds to milliseconds
    }
}

impl SessionAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Begin a new session
    pub fn begin(&self, spec: SessionSpec) -> SessionId {
        SESSION_SERVICE.with(|service| {
            let mut service = service.borrow_mut();
            let clock = ICClock;
            service.begin(spec, &clock)
        })
    }

    /// Put chunk data (write-through, no buffering)
    pub fn put_chunk(
        &self,
        sid: SessionId,
        idx: u32,
        data: &[u8],
        sink: &mut dyn ByteSink,
    ) -> Result<(), Error> {
        SESSION_SERVICE.with(|service| {
            let mut service = service.borrow_mut();
            let clock = ICClock;
            service.put_chunk(sid, idx, data, sink, &clock)
        })
    }

    /// Finish session
    pub fn finish(&self, sid: SessionId) -> Result<(), Error> {
        SESSION_SERVICE.with(|service| {
            let mut service = service.borrow_mut();
            let clock = ICClock;
            service.finish(sid, &clock)
        })
    }

    /// Abort session
    pub fn abort(&self, sid: SessionId) -> Result<(), Error> {
        SESSION_SERVICE.with(|service| {
            let mut service = service.borrow_mut();
            service.abort(sid)
        })
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&self, expiry_ms: u64) {
        SESSION_SERVICE.with(|service| {
            let mut service = service.borrow_mut();
            let clock = ICClock;
            let now_ms = clock.now_ms();
            service.tick_ttl(now_ms, expiry_ms);
        });
    }

    /// Get session info
    pub fn get_session(&self, sid: &SessionId) -> Option<crate::session::types::Session> {
        SESSION_SERVICE.with(|service| service.borrow().get_session(sid).cloned())
    }

    /// Count active sessions for caller
    pub fn count_active_for(&self, caller: &candid::Principal) -> usize {
        SESSION_SERVICE.with(|service| {
            let caller_bytes = caller.as_slice().to_vec();
            service.borrow().count_active_for(&caller_bytes)
        })
    }

    /// List all sessions
    pub fn list_all_sessions(&self) -> Vec<(u64, SessionMeta)> {
        SESSION_SERVICE.with(|service| service.borrow().list_sessions())
    }

    /// Total session count
    pub fn total_session_count(&self) -> usize {
        SESSION_SERVICE.with(|service| service.borrow().total_sessions())
    }

    /// Session count by status
    pub fn session_count_by_status(&self) -> (usize, usize) {
        SESSION_SERVICE.with(|service| service.borrow().session_count_by_status())
    }

    /// Clear all sessions
    pub fn clear_all_sessions(&self) {
        SESSION_SERVICE.with(|service| {
            let mut service = service.borrow_mut();
            // Reset the service to clear all sessions
            *service = SessionService::new();
        });
    }
}

/// Chunk iterator for streaming chunks
pub struct ChunkIterator {
    session_id: SessionId,
    chunk_count: u32,
    current_chunk: u32,
}

impl ChunkIterator {
    pub fn new(session_id: SessionId, chunk_count: u32) -> Self {
        Self {
            session_id,
            chunk_count,
            current_chunk: 0,
        }
    }
}

impl Iterator for ChunkIterator {
    type Item = (u32, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_chunk >= self.chunk_count {
            return None;
        }

        // For now, return empty chunks - this would need to be implemented
        // based on how chunks are stored in the new architecture
        let chunk_idx = self.current_chunk;
        self.current_chunk += 1;

        // TODO: Implement actual chunk retrieval from storage
        Some((chunk_idx, Vec::new()))
    }
}
