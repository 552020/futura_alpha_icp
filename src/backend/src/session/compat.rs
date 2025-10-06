use super::service::SessionService;
use super::types::{ByteSink, SessionId, SessionSpec};
use crate::types::{CapsuleId, Error};
use candid::Principal;
use std::cell::RefCell;
use std::collections::BTreeMap;

/// Upload-specific session metadata (not in generic SessionMeta)
/// Contains ALL fields that upload service expects
#[derive(Clone)]
pub struct UploadSessionMeta {
    pub session_id: u64, // Include session_id to prevent parallel key collisions
    pub capsule_id: CapsuleId,
    pub caller: Principal,
    pub created_at: u64,
    #[allow(dead_code)]
    pub expected_chunks: u32,
    pub status: crate::session::types::SessionStatus,
    pub chunk_count: u32,
    pub provisional_memory_id: String,
    pub chunk_size: usize,
    pub idem: String,
    pub blob_id: Option<u64>, // Upload-specific: blob ID after commit
}

type IdemKey = (CapsuleId, Principal, String);
type SinkFactory = Box<dyn Fn(&UploadSessionMeta) -> Result<Box<dyn ByteSink>, Error>>;

/// Compatibility layer that implements old upload session API
/// by delegating to generic SessionService
pub struct SessionCompat {
    svc: RefCell<SessionService>,
    meta: RefCell<BTreeMap<u64, UploadSessionMeta>>,
    idem: RefCell<BTreeMap<IdemKey, SessionId>>,
    sink_factory: SinkFactory,
}

impl SessionCompat {
    pub fn new<F>(sink_factory: F) -> Self
    where
        F: Fn(&UploadSessionMeta) -> Result<Box<dyn ByteSink>, Error> + 'static,
    {
        Self {
            svc: RefCell::new(SessionService::new()),
            meta: RefCell::new(BTreeMap::new()),
            idem: RefCell::new(BTreeMap::new()),
            sink_factory: Box::new(sink_factory),
        }
    }

    /// Find pending session by capsule, caller, and idempotency key
    pub fn find_pending(
        &self,
        cap: &CapsuleId,
        caller: &Principal,
        idem: &str,
    ) -> Option<SessionId> {
        self.idem
            .borrow()
            .get(&(cap.clone(), *caller, idem.to_string()))
            .cloned()
    }

    /// Create session with upload-specific metadata (old API signature)
    pub fn create(&self, sid: SessionId, meta: UploadSessionMeta) -> Result<(), Error> {
        // Build a generic spec from upload meta
        let spec = SessionSpec {
            chunk_size: meta.chunk_size,
            bytes_expected: 0, // No longer available without asset metadata
            owner: meta.caller,
            idem: meta.idem.clone(),
        };

        let clock = crate::session::adapter::ICClock;
        self.svc.borrow_mut().begin_with_id(sid, spec, &clock)?;

        // Store compat meta + idem map
        self.meta.borrow_mut().insert(sid.0, meta.clone());
        self.idem.borrow_mut().insert(
            (meta.capsule_id.clone(), meta.caller, meta.idem.clone()),
            sid,
        );
        Ok(())
    }

    /// Get upload session metadata
    pub fn get(&self, sid: &SessionId) -> Result<Option<UploadSessionMeta>, Error> {
        Ok(self.meta.borrow().get(&sid.0).cloned())
    }

    /// Update upload session metadata
    pub fn update(&self, sid: SessionId, meta: UploadSessionMeta) -> Result<(), Error> {
        if !self.svc.borrow().exists(sid) {
            return Err(Error::NotFound);
        }
        self.meta.borrow_mut().insert(sid.0, meta);
        Ok(())
    }

    /// Count active sessions for capsule and caller
    pub fn count_active_for(&self, cap: &CapsuleId, caller: &Principal) -> usize {
        self.meta
            .borrow()
            .values()
            .filter(|m| &m.capsule_id == cap && &m.caller == caller)
            .count()
    }

    /// Verify chunks are complete
    pub fn verify_chunks_complete(&self, sid: &SessionId, chunk_count: u32) -> Result<(), Error> {
        let rc = self.svc.borrow().received_count(*sid)?;
        if rc == chunk_count {
            Ok(())
        } else {
            Err(Error::InvalidArgument("Incomplete chunks".to_string()))
        }
    }

    /// Put chunk with ByteSink (old API signature: sid, idx, bytes)
    pub fn put_chunk(&self, sid: &SessionId, idx: u32, data: &[u8]) -> Result<(), Error> {
        let meta = self
            .meta
            .borrow()
            .get(&sid.0)
            .cloned()
            .ok_or(Error::NotFound)?;
        let mut sink = (self.sink_factory)(&meta)?;
        let clock = crate::session::adapter::ICClock;
        self.svc
            .borrow_mut()
            .put_chunk(*sid, idx, data, &mut *sink, &clock)
    }

    /// Finish session (delegates to generic service)
    #[allow(dead_code)]
    pub fn finish(&self, sid: &SessionId) -> Result<(), Error> {
        let clock = crate::session::adapter::ICClock;
        self.svc.borrow_mut().finish(*sid, &clock)
    }

    /// Abort session (delegates to generic service)
    #[allow(dead_code)]
    pub fn abort(&self, sid: &SessionId) -> Result<(), Error> {
        self.svc.borrow_mut().abort(*sid)
    }

    /// Cleanup session (remove from compat maps)
    pub fn cleanup(&self, sid: &SessionId) {
        // Remove from meta and idempotency maps
        if let Some(meta) = self.meta.borrow_mut().remove(&sid.0) {
            let key = (meta.capsule_id.clone(), meta.caller, meta.idem.clone());
            self.idem.borrow_mut().remove(&key);
        }
    }

    /// Cleanup expired sessions for specific caller
    pub fn cleanup_expired_sessions_for_caller(
        &self,
        cap: &CapsuleId,
        caller: &Principal,
        expiry_ms: u64,
    ) {
        let now_ms = ic_cdk::api::time() / 1_000_000;
        let expired: Vec<u64> = self
            .meta
            .borrow()
            .iter()
            .filter(|(_, meta)| {
                meta.capsule_id == *cap
                    && meta.caller == *caller
                    && now_ms > meta.created_at + expiry_ms
            })
            .map(|(id, _)| *id)
            .collect();

        for id in expired {
            self.cleanup(&SessionId(id));
        }
    }

    /// Cleanup all expired sessions (for global cleanup)
    pub fn cleanup_expired_sessions(&self, expiry_ms: u64) {
        let now_ms = ic_cdk::api::time() / 1_000_000;
        let expired: Vec<u64> = self
            .meta
            .borrow()
            .iter()
            .filter(|(_, meta)| now_ms > meta.created_at + expiry_ms)
            .map(|(id, _)| *id)
            .collect();

        for id in expired {
            self.cleanup(&SessionId(id));
        }
    }

    /// List all sessions (for debugging)
    #[allow(dead_code)]
    pub fn list_all_sessions(&self) -> Vec<(u64, crate::session::types::SessionMeta)> {
        self.svc.borrow().list_sessions()
    }

    /// List all upload sessions with upload-specific metadata
    pub fn list_upload_sessions(&self) -> Vec<(u64, UploadSessionMeta)> {
        self.meta
            .borrow()
            .iter()
            .map(|(id, meta)| (*id, meta.clone()))
            .collect()
    }

    /// Total session count
    pub fn total_session_count(&self) -> usize {
        self.svc.borrow().total_sessions()
    }

    /// Session count by status
    #[allow(dead_code)]
    pub fn session_count_by_status(&self) -> (usize, usize) {
        self.svc.borrow().session_count_by_status()
    }

    /// Clear all sessions
    pub fn clear_all_sessions(&self) {
        *self.svc.borrow_mut() = SessionService::new();
        self.meta.borrow_mut().clear();
        self.idem.borrow_mut().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

    /// Mock ByteSink for testing
    struct MockByteSink {
        writes: std::cell::RefCell<Vec<(u64, Vec<u8>)>>,
    }

    impl MockByteSink {
        fn new() -> Self {
            Self {
                writes: std::cell::RefCell::new(Vec::new()),
            }
        }

        #[allow(dead_code)] // Test utility for debugging chunk writes - may be used in future tests
        fn get_writes(&self) -> Vec<(u64, Vec<u8>)> {
            self.writes.borrow().clone()
        }
    }

    impl ByteSink for MockByteSink {
        fn write_at(&mut self, offset: u64, data: &[u8]) -> Result<(), Error> {
            self.writes.borrow_mut().push((offset, data.to_vec()));
            Ok(())
        }
    }

    fn create_test_meta(caller: Principal, capsule_id: CapsuleId) -> UploadSessionMeta {
        UploadSessionMeta {
            session_id: 42, // Test session ID
            capsule_id,
            caller,
            created_at: 1000,
            expected_chunks: 2,
            status: crate::session::types::SessionStatus::Pending,
            chunk_count: 2,
            provisional_memory_id: "test-mem-123".to_string(),
            chunk_size: 1024,
            idem: "test-idem".to_string(),
            blob_id: None,
        }
    }

    #[test]
    fn test_create_stores_upload_meta() {
        let compat = SessionCompat::new(|_| Ok(Box::new(MockByteSink::new()) as Box<dyn ByteSink>));

        let caller = Principal::anonymous();
        let capsule_id = "test-capsule".to_string();
        let sid = SessionId(42);
        let meta = create_test_meta(caller, capsule_id.clone());

        let result = compat.create(sid, meta.clone());
        assert!(result.is_ok());

        // Verify meta is stored
        let retrieved = compat.get(&sid).unwrap();
        assert!(retrieved.is_some());
        let retrieved_meta = retrieved.unwrap();
        assert_eq!(retrieved_meta.capsule_id, capsule_id);
        assert_eq!(retrieved_meta.caller, caller);
    }

    #[test]
    fn test_find_pending_returns_existing() {
        let compat = SessionCompat::new(|_| Ok(Box::new(MockByteSink::new()) as Box<dyn ByteSink>));

        let caller = Principal::anonymous();
        let capsule_id = "test-capsule".to_string();
        let idem = "test-idem".to_string();
        let sid = SessionId(42);
        let meta = create_test_meta(caller, capsule_id.clone());

        compat.create(sid, meta).unwrap();

        // Should find the session by idempotency key
        let found = compat.find_pending(&capsule_id, &caller, &idem);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), sid);
    }

    #[test]
    fn test_find_pending_returns_none_for_nonexistent() {
        let compat = SessionCompat::new(|_| Ok(Box::new(MockByteSink::new()) as Box<dyn ByteSink>));

        let caller = Principal::anonymous();
        let capsule_id = "test-capsule".to_string();
        let idem = "nonexistent-idem".to_string();

        let found = compat.find_pending(&capsule_id, &caller, &idem);
        assert!(found.is_none());
    }

    #[test]
    fn test_put_chunk_calls_sink_factory() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let sink_called = Arc::new(AtomicBool::new(false));
        let sink_called_clone = sink_called.clone();

        let compat = SessionCompat::new(move |_meta| {
            sink_called_clone.store(true, Ordering::SeqCst);
            Ok(Box::new(MockByteSink::new()) as Box<dyn ByteSink>)
        });

        let caller = Principal::anonymous();
        let capsule_id = "test-capsule".to_string();
        let sid = SessionId(42);
        let meta = create_test_meta(caller, capsule_id);

        compat.create(sid, meta).unwrap();

        let chunk_data = vec![1, 2, 3, 4, 5];
        let result = compat.put_chunk(&sid, 0, &chunk_data);

        assert!(result.is_ok());
        assert!(
            sink_called.load(Ordering::SeqCst),
            "ByteSink factory should have been called"
        );
    }

    #[test]
    fn test_update_modifies_meta() {
        let compat = SessionCompat::new(|_| Ok(Box::new(MockByteSink::new()) as Box<dyn ByteSink>));

        let caller = Principal::anonymous();
        let capsule_id = "test-capsule".to_string();
        let sid = SessionId(42);
        let mut meta = create_test_meta(caller, capsule_id);

        compat.create(sid, meta.clone()).unwrap();

        // Update meta
        meta.blob_id = Some(999);
        meta.status = crate::session::types::SessionStatus::Committed { completed_at: 2000 };

        let result = compat.update(sid, meta.clone());
        assert!(result.is_ok());

        // Verify meta was updated
        let retrieved = compat.get(&sid).unwrap().unwrap();
        assert_eq!(retrieved.blob_id, Some(999));
    }

    #[test]
    fn test_cleanup_removes_meta_and_idem() {
        let compat = SessionCompat::new(|_| Ok(Box::new(MockByteSink::new()) as Box<dyn ByteSink>));

        let caller = Principal::anonymous();
        let capsule_id = "test-capsule".to_string();
        let idem = "test-idem".to_string();
        let sid = SessionId(42);
        let meta = create_test_meta(caller, capsule_id.clone());

        compat.create(sid, meta).unwrap();

        // Verify session exists
        assert!(compat.find_pending(&capsule_id, &caller, &idem).is_some());

        // Cleanup
        compat.cleanup(&sid);

        // Verify session is removed
        assert!(compat.get(&sid).unwrap().is_none());
    }

    #[test]
    fn test_count_active_for_returns_correct_count() {
        let compat = SessionCompat::new(|_| Ok(Box::new(MockByteSink::new()) as Box<dyn ByteSink>));

        let caller1 = Principal::anonymous();
        let caller2 = Principal::from_text("aaaaa-aa").unwrap();
        let capsule_id = "test-capsule".to_string();

        // Create 3 sessions for caller1, 2 for caller2
        for i in 0..3 {
            let mut meta = create_test_meta(caller1, capsule_id.clone());
            meta.idem = format!("idem-{}", i);
            compat.create(SessionId(i), meta).unwrap();
        }

        for i in 10..12 {
            let mut meta = create_test_meta(caller2, capsule_id.clone());
            meta.idem = format!("idem-{}", i);
            compat.create(SessionId(i), meta).unwrap();
        }

        assert_eq!(compat.count_active_for(&capsule_id, &caller1), 3);
        assert_eq!(compat.count_active_for(&capsule_id, &caller2), 2);
    }

    #[test]
    fn test_verify_chunks_complete_success() {
        let compat = SessionCompat::new(|_| Ok(Box::new(MockByteSink::new()) as Box<dyn ByteSink>));

        let caller = Principal::anonymous();
        let capsule_id = "test-capsule".to_string();
        let sid = SessionId(42);
        let meta = create_test_meta(caller, capsule_id);

        compat.create(sid, meta).unwrap();

        // Put all expected chunks
        compat.put_chunk(&sid, 0, &vec![1; 1024]).unwrap();
        compat.put_chunk(&sid, 1, &vec![2; 1024]).unwrap();

        // Should succeed
        let result = compat.verify_chunks_complete(&sid, 2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_chunks_complete_failure() {
        let compat = SessionCompat::new(|_| Ok(Box::new(MockByteSink::new()) as Box<dyn ByteSink>));

        let caller = Principal::anonymous();
        let capsule_id = "test-capsule".to_string();
        let sid = SessionId(42);
        let meta = create_test_meta(caller, capsule_id);

        compat.create(sid, meta).unwrap();

        // Put only 1 chunk (need 2)
        compat.put_chunk(&sid, 0, &vec![1; 1024]).unwrap();

        // Should fail
        let result = compat.verify_chunks_complete(&sid, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_upload_sessions_returns_all() {
        let compat = SessionCompat::new(|_| Ok(Box::new(MockByteSink::new()) as Box<dyn ByteSink>));

        let caller = Principal::anonymous();
        let capsule_id = "test-capsule".to_string();

        // Create 3 sessions
        for i in 0..3 {
            let mut meta = create_test_meta(caller, capsule_id.clone());
            meta.idem = format!("idem-{}", i);
            compat.create(SessionId(i), meta).unwrap();
        }

        let sessions = compat.list_upload_sessions();
        assert_eq!(sessions.len(), 3);

        let session_ids: Vec<u64> = sessions.iter().map(|(id, _)| *id).collect();
        assert!(session_ids.contains(&0));
        assert!(session_ids.contains(&1));
        assert!(session_ids.contains(&2));
    }

    #[test]
    fn test_total_session_count() {
        let compat = SessionCompat::new(|_| Ok(Box::new(MockByteSink::new()) as Box<dyn ByteSink>));

        assert_eq!(compat.total_session_count(), 0);

        let caller = Principal::anonymous();
        let capsule_id = "test-capsule".to_string();
        let meta = create_test_meta(caller, capsule_id);

        compat.create(SessionId(1), meta.clone()).unwrap();
        assert_eq!(compat.total_session_count(), 1);

        compat.create(SessionId(2), meta).unwrap();
        assert_eq!(compat.total_session_count(), 2);
    }

    #[test]
    fn test_clear_all_sessions() {
        let compat = SessionCompat::new(|_| Ok(Box::new(MockByteSink::new()) as Box<dyn ByteSink>));

        let caller = Principal::anonymous();
        let capsule_id = "test-capsule".to_string();

        // Create multiple sessions
        for i in 0..5 {
            let mut meta = create_test_meta(caller, capsule_id.clone());
            meta.idem = format!("idem-{}", i);
            compat.create(SessionId(i), meta).unwrap();
        }

        assert_eq!(compat.total_session_count(), 5);

        compat.clear_all_sessions();

        assert_eq!(compat.total_session_count(), 0);
        assert_eq!(compat.list_upload_sessions().len(), 0);
    }
}
