use crate::session::types::{
    ByteSink, Clock, Session, SessionId, SessionMeta, SessionSpec, SessionStatus,
};
use crate::types::Error;
use std::collections::{BTreeMap, BTreeSet};

/// Generic session service (no upload semantics)
/// Manages session lifecycle and per-chunk book-keeping only
pub struct SessionService {
    next_id: u64,
    sessions: BTreeMap<u64, Session>,
}

impl Default for SessionService {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionService {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            sessions: BTreeMap::new(),
        }
    }

    /// Begin a new session
    pub fn begin(&mut self, spec: SessionSpec, clock: &dyn Clock) -> SessionId {
        let session_id = SessionId(self.next_id);
        self.next_id += 1;

        let session_meta = SessionMeta::new(spec.idem, clock.now_ms());

        let session = Session {
            owner: spec.owner.as_slice().to_vec(), // Convert Principal to bytes
            chunk_size: spec.chunk_size,
            bytes_expected: spec.bytes_expected,
            bytes_received: 0,
            received_idxs: BTreeSet::new(),
            session_meta,
        };

        self.sessions.insert(session_id.0, session);
        session_id
    }

    /// Put chunk data (write-through, no buffering)
    pub fn put_chunk(
        &mut self,
        sid: SessionId,
        idx: u32,
        data: &[u8],
        sink: &mut dyn ByteSink,
        clock: &dyn Clock,
    ) -> Result<(), Error> {
        let session = self.sessions.get_mut(&sid.0).ok_or(Error::NotFound)?;

        // Check if chunk already received
        if session.received_idxs.contains(&idx) {
            return Err(Error::InvalidArgument("Chunk already received".to_string()));
        }

        // Write directly to sink (no buffering!)
        let offset = (idx as u64) * (session.chunk_size as u64);
        sink.write_at(offset, data)?;

        // Update session state
        session.bytes_received += data.len() as u64;
        session.received_idxs.insert(idx);
        session.session_meta.last_seen = clock.now_ms();

        Ok(())
    }

    /// Finish session (just closes session, no business logic)
    pub fn finish(&mut self, sid: SessionId, clock: &dyn Clock) -> Result<(), Error> {
        let session = self.sessions.get_mut(&sid.0).ok_or(Error::NotFound)?;

        // Verify all chunks received
        let expected_chunks =
            (session.bytes_expected + session.chunk_size as u64 - 1) / session.chunk_size as u64;
        if session.received_idxs.len() != expected_chunks as usize {
            return Err(Error::InvalidArgument(
                "Not all chunks received".to_string(),
            ));
        }

        // Mark as committed
        session.session_meta.status = SessionStatus::Committed {
            completed_at: clock.now_ms(),
        };

        Ok(())
    }

    /// Abort session
    pub fn abort(&mut self, sid: SessionId) -> Result<(), Error> {
        self.sessions.remove(&sid.0).ok_or(Error::NotFound)?;
        Ok(())
    }

    /// Clean up expired sessions (TTL tick)
    pub fn tick_ttl(&mut self, now_ms: u64, ttl_ms: u64) -> usize {
        let expired: Vec<u64> = self
            .sessions
            .iter()
            .filter(|(_, session)| now_ms - session.session_meta.last_seen > ttl_ms)
            .map(|(id, _)| *id)
            .collect();

        for id in &expired {
            self.sessions.remove(id);
        }

        expired.len()
    }

    /// Get session info
    pub fn get_session(&self, sid: &SessionId) -> Option<&Session> {
        self.sessions.get(&sid.0)
    }

    /// Count active sessions for owner
    pub fn count_active_for(&self, owner: &[u8]) -> usize {
        self.sessions
            .values()
            .filter(|session| {
                session.owner == owner
                    && matches!(session.session_meta.status, SessionStatus::Pending)
            })
            .count()
    }

    /// List all sessions (for debugging)
    pub fn list_sessions(&self) -> Vec<(u64, SessionMeta)> {
        self.sessions
            .iter()
            .map(|(id, session)| (*id, session.session_meta.clone()))
            .collect()
    }

    /// Total session count
    pub fn total_sessions(&self) -> usize {
        self.sessions.len()
    }

    /// Session count by status
    pub fn session_count_by_status(&self) -> (usize, usize) {
        let (pending, committed) = self.sessions.values().fold((0, 0), |(p, c), session| {
            match session.session_meta.status {
                SessionStatus::Pending => (p + 1, c),
                SessionStatus::Committed { .. } => (p, c + 1),
            }
        });
        (pending, committed)
    }

    /// Begin session with specific ID (for compatibility)
    pub fn begin_with_id(
        &mut self,
        sid: SessionId,
        spec: SessionSpec,
        clock: &dyn Clock,
    ) -> Result<(), Error> {
        if self.sessions.contains_key(&sid.0) {
            return Err(Error::InvalidArgument("Session already exists".to_string()));
        }

        let session_meta = SessionMeta::new(spec.idem, clock.now_ms());
        let session = Session {
            owner: spec.owner.as_slice().to_vec(),
            chunk_size: spec.chunk_size,
            bytes_expected: spec.bytes_expected,
            bytes_received: 0,
            received_idxs: BTreeSet::new(),
            session_meta,
        };

        self.sessions.insert(sid.0, session);
        Ok(())
    }

    /// Check if session exists
    pub fn exists(&self, sid: SessionId) -> bool {
        self.sessions.contains_key(&sid.0)
    }

    /// Get received chunk count
    pub fn received_count(&self, sid: SessionId) -> Result<u32, Error> {
        let session = self.sessions.get(&sid.0).ok_or(Error::NotFound)?;
        Ok(session.received_idxs.len() as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

    /// Mock Clock for testing
    struct MockClock {
        time_ms: u64,
    }

    impl Clock for MockClock {
        fn now_ms(&self) -> u64 {
            self.time_ms
        }
    }

    /// Mock ByteSink for testing
    struct MockByteSink {
        pub data: std::cell::RefCell<Vec<u8>>,
        pub writes: std::cell::RefCell<Vec<(u64, usize)>>, // (offset, length)
    }

    impl MockByteSink {
        fn new() -> Self {
            Self {
                data: std::cell::RefCell::new(Vec::new()),
                writes: std::cell::RefCell::new(Vec::new()),
            }
        }

        fn get_data(&self) -> Vec<u8> {
            self.data.borrow().clone()
        }

        fn get_writes(&self) -> Vec<(u64, usize)> {
            self.writes.borrow().clone()
        }
    }

    impl ByteSink for MockByteSink {
        fn write_at(&mut self, offset: u64, data: &[u8]) -> Result<(), Error> {
            self.writes.borrow_mut().push((offset, data.len()));

            // Ensure data buffer is large enough
            let required_size = (offset as usize) + data.len();
            if self.data.borrow().len() < required_size {
                self.data.borrow_mut().resize(required_size, 0);
            }

            // Write data at offset
            self.data.borrow_mut()[(offset as usize)..(offset as usize + data.len())]
                .copy_from_slice(data);

            Ok(())
        }
    }

    fn create_test_spec(chunk_size: usize, bytes_expected: u64) -> SessionSpec {
        SessionSpec {
            chunk_size,
            bytes_expected,
            owner: Principal::anonymous(),
            idem: "test-idem".to_string(),
        }
    }

    #[test]
    fn test_begin_creates_session() {
        let mut service = SessionService::new();
        let clock = MockClock { time_ms: 1000 };
        let spec = create_test_spec(1024, 2048);

        let session_id = service.begin(spec.clone(), &clock);

        assert!(service.exists(session_id));
        assert_eq!(service.total_sessions(), 1);
    }

    #[test]
    fn test_begin_increments_session_id() {
        let mut service = SessionService::new();
        let clock = MockClock { time_ms: 1000 };
        let spec = create_test_spec(1024, 2048);

        let sid1 = service.begin(spec.clone(), &clock);
        let sid2 = service.begin(spec.clone(), &clock);

        assert_ne!(sid1.0, sid2.0);
        assert!(sid2.0 > sid1.0);
    }

    #[test]
    fn test_begin_with_id_prevents_duplicates() {
        let mut service = SessionService::new();
        let clock = MockClock { time_ms: 1000 };
        let spec = create_test_spec(1024, 2048);
        let sid = SessionId(42);

        // First insertion should succeed
        let result1 = service.begin_with_id(sid, spec.clone(), &clock);
        assert!(result1.is_ok());

        // Second insertion with same ID should fail
        let result2 = service.begin_with_id(sid, spec.clone(), &clock);
        assert!(result2.is_err());
    }

    #[test]
    fn test_exists_returns_correct_value() {
        let mut service = SessionService::new();
        let clock = MockClock { time_ms: 1000 };
        let spec = create_test_spec(1024, 2048);

        let sid = service.begin(spec, &clock);
        assert!(service.exists(sid));

        let non_existent = SessionId(99999);
        assert!(!service.exists(non_existent));
    }

    #[test]
    fn test_put_chunk_writes_to_sink() {
        let mut service = SessionService::new();
        let clock = MockClock { time_ms: 1000 };
        let spec = create_test_spec(1024, 2048);
        let sid = service.begin(spec, &clock);

        let mut sink = MockByteSink::new();
        let chunk_data = vec![1, 2, 3, 4, 5];

        let result = service.put_chunk(sid, 0, &chunk_data, &mut sink, &clock);
        assert!(result.is_ok());

        // Verify data was written to sink
        let writes = sink.get_writes();
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0], (0, 5)); // offset 0, length 5
    }

    #[test]
    fn test_put_chunk_updates_received_count() {
        let mut service = SessionService::new();
        let clock = MockClock { time_ms: 1000 };
        let spec = create_test_spec(1024, 4096);
        let sid = service.begin(spec, &clock);

        let mut sink = MockByteSink::new();

        // Put 3 chunks
        service
            .put_chunk(sid, 0, &vec![1; 1024], &mut sink, &clock)
            .unwrap();
        service
            .put_chunk(sid, 1, &vec![2; 1024], &mut sink, &clock)
            .unwrap();
        service
            .put_chunk(sid, 2, &vec![3; 1024], &mut sink, &clock)
            .unwrap();

        assert_eq!(service.received_count(sid).unwrap(), 3);
    }

    #[test]
    fn test_put_chunk_rejects_duplicate_chunk() {
        let mut service = SessionService::new();
        let clock = MockClock { time_ms: 1000 };
        let spec = create_test_spec(1024, 2048);
        let sid = service.begin(spec, &clock);

        let mut sink = MockByteSink::new();
        let chunk_data = vec![1, 2, 3];

        // First put should succeed
        let result1 = service.put_chunk(sid, 0, &chunk_data, &mut sink, &clock);
        assert!(result1.is_ok());

        // Second put with same index should fail
        let result2 = service.put_chunk(sid, 0, &chunk_data, &mut sink, &clock);
        assert!(result2.is_err());
    }

    #[test]
    fn test_put_chunk_calculates_correct_offset() {
        let mut service = SessionService::new();
        let clock = MockClock { time_ms: 1000 };
        let chunk_size = 1024;
        let spec = create_test_spec(chunk_size, 3072);
        let sid = service.begin(spec, &clock);

        let mut sink = MockByteSink::new();

        // Put chunk at index 2 (should write at offset 2048)
        service
            .put_chunk(sid, 2, &vec![1, 2, 3], &mut sink, &clock)
            .unwrap();

        let writes = sink.get_writes();
        assert_eq!(writes[0].0, 2048); // offset = index * chunk_size
    }

    #[test]
    fn test_finish_validates_completeness() {
        let mut service = SessionService::new();
        let clock = MockClock { time_ms: 1000 };
        let spec = create_test_spec(1024, 2048);
        let sid = service.begin(spec, &clock);

        let mut sink = MockByteSink::new();

        // Put all chunks
        service
            .put_chunk(sid, 0, &vec![1; 1024], &mut sink, &clock)
            .unwrap();
        service
            .put_chunk(sid, 1, &vec![2; 1024], &mut sink, &clock)
            .unwrap();

        // Finish should succeed
        let result = service.finish(sid, &clock);
        assert!(result.is_ok());
    }

    #[test]
    fn test_finish_fails_on_incomplete_chunks() {
        let mut service = SessionService::new();
        let clock = MockClock { time_ms: 1000 };
        let spec = create_test_spec(1024, 2048);
        let sid = service.begin(spec, &clock);

        let mut sink = MockByteSink::new();

        // Put only 1 chunk (need 2)
        service
            .put_chunk(sid, 0, &vec![1; 1024], &mut sink, &clock)
            .unwrap();

        // Finish should fail
        let result = service.finish(sid, &clock);
        assert!(result.is_err());
    }

    #[test]
    fn test_abort_removes_session() {
        let mut service = SessionService::new();
        let clock = MockClock { time_ms: 1000 };
        let spec = create_test_spec(1024, 2048);
        let sid = service.begin(spec, &clock);

        assert!(service.exists(sid));

        service.abort(sid).unwrap();

        assert!(!service.exists(sid));
        assert_eq!(service.total_sessions(), 0);
    }

    #[test]
    fn test_tick_ttl_removes_expired_sessions() {
        let mut service = SessionService::new();
        let clock1 = MockClock { time_ms: 1000 };
        let clock2 = MockClock { time_ms: 5000 };
        let spec = create_test_spec(1024, 2048);

        // Create session at time 1000
        let sid = service.begin(spec, &clock1);
        assert!(service.exists(sid));

        // Clean up sessions older than 3 seconds (3000ms)
        let removed = service.tick_ttl(clock2.now_ms(), 3000);

        assert_eq!(removed, 1);
        assert!(!service.exists(sid));
    }

    #[test]
    fn test_tick_ttl_preserves_recent_sessions() {
        let mut service = SessionService::new();
        let clock1 = MockClock { time_ms: 1000 };
        let clock2 = MockClock { time_ms: 3000 };
        let spec = create_test_spec(1024, 2048);

        // Create session at time 1000
        let sid = service.begin(spec, &clock1);

        // Clean up sessions older than 5 seconds (5000ms)
        let removed = service.tick_ttl(clock2.now_ms(), 5000);

        assert_eq!(removed, 0);
        assert!(service.exists(sid));
    }

    #[test]
    fn test_received_count_accuracy() {
        let mut service = SessionService::new();
        let clock = MockClock { time_ms: 1000 };
        let spec = create_test_spec(1024, 5120);
        let sid = service.begin(spec, &clock);

        let mut sink = MockByteSink::new();

        assert_eq!(service.received_count(sid).unwrap(), 0);

        service
            .put_chunk(sid, 0, &vec![1; 1024], &mut sink, &clock)
            .unwrap();
        assert_eq!(service.received_count(sid).unwrap(), 1);

        service
            .put_chunk(sid, 1, &vec![2; 1024], &mut sink, &clock)
            .unwrap();
        assert_eq!(service.received_count(sid).unwrap(), 2);

        service
            .put_chunk(sid, 3, &vec![4; 1024], &mut sink, &clock)
            .unwrap();
        assert_eq!(service.received_count(sid).unwrap(), 3);
    }

    #[test]
    fn test_session_count_by_status() {
        let mut service = SessionService::new();
        let clock = MockClock { time_ms: 1000 };
        let spec = create_test_spec(1024, 2048);

        // Create 3 sessions
        service.begin(spec.clone(), &clock);
        service.begin(spec.clone(), &clock);
        service.begin(spec.clone(), &clock);

        let (pending, committed) = service.session_count_by_status();
        assert_eq!(pending, 3);
        assert_eq!(committed, 0);
    }

    #[test]
    fn test_total_sessions_count() {
        let mut service = SessionService::new();
        let clock = MockClock { time_ms: 1000 };
        let spec = create_test_spec(1024, 2048);

        assert_eq!(service.total_sessions(), 0);

        service.begin(spec.clone(), &clock);
        assert_eq!(service.total_sessions(), 1);

        service.begin(spec.clone(), &clock);
        assert_eq!(service.total_sessions(), 2);
    }

    #[test]
    fn test_list_sessions() {
        let mut service = SessionService::new();
        let clock = MockClock { time_ms: 1000 };
        let spec = create_test_spec(1024, 2048);

        let sid1 = service.begin(spec.clone(), &clock);
        let sid2 = service.begin(spec.clone(), &clock);

        let sessions = service.list_sessions();
        assert_eq!(sessions.len(), 2);

        let session_ids: Vec<u64> = sessions.iter().map(|(id, _)| *id).collect();
        assert!(session_ids.contains(&sid1.0));
        assert!(session_ids.contains(&sid2.0));
    }
}
