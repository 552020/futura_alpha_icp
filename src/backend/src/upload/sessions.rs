use crate::memory::{MEM_SESSIONS, MEM_SESSIONS_CHUNKS, MEM_SESSIONS_COUNTER, MM};
use crate::types::{CapsuleId, Error};
use crate::upload::types::{SessionId, SessionMeta};
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::DefaultMemoryImpl;
use ic_stable_structures::{StableBTreeMap, StableCell};
use std::cell::RefCell;

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static STABLE_UPLOAD_SESSIONS: RefCell<StableBTreeMap<u64, SessionMeta, Memory>> = RefCell::new(
        StableBTreeMap::init(MM.with(|m| m.borrow().get(MEM_SESSIONS)))
    );

    static STABLE_CHUNK_DATA: RefCell<StableBTreeMap<(u64, u32), Vec<u8>, Memory>> = RefCell::new(
        StableBTreeMap::init(MM.with(|m| m.borrow().get(MEM_SESSIONS_CHUNKS)))
    );

    pub static STABLE_SESSION_COUNTER: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(MM.with(|m| m.borrow().get(MEM_SESSIONS_COUNTER)), 0)
            .expect("Failed to init session counter")
    );
}

/// Session store for managing upload sessions and chunks
// During the storage migration, some upload APIs are not wired yet in
// production paths. Silence "associated items are never used" locally,
// but keep the code compiled and testable. Enable strictly when the
// `upload` feature is on.
pub struct SessionStore;

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionStore {
    pub fn new() -> Self {
        SessionStore
    }

    /// Create a new upload session
    pub fn create(
        &self,
        session_id: SessionId,
        session_meta: SessionMeta,
    ) -> std::result::Result<(), Error> {
        STABLE_UPLOAD_SESSIONS.with(|sessions| {
            sessions.borrow_mut().insert(session_id.0, session_meta);
        });
        Ok(())
    }

    /// Get session metadata
    pub fn get(&self, session_id: &SessionId) -> std::result::Result<Option<SessionMeta>, Error> {
        let session = STABLE_UPLOAD_SESSIONS.with(|sessions| sessions.borrow().get(&session_id.0));
        Ok(session)
    }

    /// Update session metadata (for status changes)
    pub fn update(
        &self,
        session_id: &SessionId,
        session_meta: SessionMeta,
    ) -> std::result::Result<(), Error> {
        STABLE_UPLOAD_SESSIONS.with(|sessions| {
            sessions.borrow_mut().insert(session_id.0, session_meta);
        });
        Ok(())
    }

    /// Store a chunk for a session
    pub fn put_chunk(
        &self,
        session_id: &SessionId,
        chunk_idx: u32,
        bytes: Vec<u8>,
    ) -> std::result::Result<(), Error> {
        let chunk_key = (session_id.0, chunk_idx);
        STABLE_CHUNK_DATA.with(|chunks| {
            chunks.borrow_mut().insert(chunk_key, bytes);
        });
        Ok(())
    }

    // Removed unused method: get_chunk

    /// Verify all chunks exist for a session (integrity check)
    pub fn verify_chunks_complete(
        &self,
        session_id: &SessionId,
        chunk_count: u32,
    ) -> std::result::Result<(), Error> {
        for chunk_idx in 0..chunk_count {
            let chunk_key = (session_id.0, chunk_idx);
            let exists = STABLE_CHUNK_DATA.with(|chunks| chunks.borrow().contains_key(&chunk_key));

            if !exists {
                return Err(Error::NotFound);
            }
        }
        Ok(())
    }

    /// Clean up session and all associated chunks
    pub fn cleanup(&self, session_id: &SessionId) {
        // Remove session metadata
        STABLE_UPLOAD_SESSIONS.with(|sessions| {
            sessions.borrow_mut().remove(&session_id.0);
        });

        // Remove all chunks for this session
        let mut chunk_idx = 0u32;
        loop {
            let chunk_key = (session_id.0, chunk_idx);
            let removed = STABLE_CHUNK_DATA.with(|chunks| chunks.borrow_mut().remove(&chunk_key));

            if removed.is_none() {
                break; // No more chunks
            }
            chunk_idx += 1;
        }
    }

    /// Get chunks iterator for streaming (used by blob store)
    pub fn iter_chunks(&self, session_id: &SessionId, chunk_count: u32) -> ChunkIterator {
        ChunkIterator {
            session_id: session_id.0,
            chunk_count,
            current_idx: 0,
        }
    }

    /// Find pending session by capsule, caller, and idempotency key
    pub fn find_pending(
        &self,
        capsule_id: &CapsuleId,
        caller: &candid::Principal,
        idem: &str,
    ) -> Option<SessionId> {
        STABLE_UPLOAD_SESSIONS.with(|sessions| {
            sessions.borrow().iter().find_map(|(sid, session)| {
                if &session.capsule_id == capsule_id
                    && &session.caller == caller
                    && session.idem == idem
                    && matches!(session.status, crate::upload::types::SessionStatus::Pending)
                {
                    Some(SessionId(sid))
                } else {
                    None
                }
            })
        })
    }

    /// Count active sessions for a caller/capsule combination
    pub fn count_active_for(&self, capsule_id: &CapsuleId, caller: &candid::Principal) -> usize {
        STABLE_UPLOAD_SESSIONS.with(|sessions| {
            sessions
                .borrow()
                .iter()
                .filter(|(_, session)| {
                    &session.capsule_id == capsule_id
                        && &session.caller == caller
                        && matches!(session.status, crate::upload::types::SessionStatus::Pending)
                })
                .count()
        })
    }

    /// Clear all sessions (development/debugging only)
    pub fn clear_all_sessions(&self) {
        STABLE_UPLOAD_SESSIONS.with(|sessions| {
            let _ = sessions.borrow_mut().clear_new();
        });
        STABLE_CHUNK_DATA.with(|chunks| {
            let _ = chunks.borrow_mut().clear_new();
        });
    }

    /// Get total session count for monitoring
    pub fn total_session_count(&self) -> usize {
        STABLE_UPLOAD_SESSIONS.with(|sessions| sessions.borrow().len().try_into().unwrap_or(0))
    }

    /// Get session count by status for monitoring
    pub fn session_count_by_status(&self) -> (usize, usize) {
        STABLE_UPLOAD_SESSIONS.with(|sessions| {
            let mut pending_count = 0;
            let mut committed_count = 0;

            for (_, session) in sessions.borrow().iter() {
                match session.status {
                    crate::upload::types::SessionStatus::Pending => pending_count += 1,
                    crate::upload::types::SessionStatus::Committed { .. } => committed_count += 1,
                }
            }

            (pending_count, committed_count)
        })
    }

    /// Clean up expired sessions (older than specified milliseconds)
    pub fn cleanup_expired_sessions(&self, expiry_ms: u64) {
        let now = ic_cdk::api::time();
        let mut expired_sessions = Vec::new();

        // Find expired sessions
        STABLE_UPLOAD_SESSIONS.with(|sessions| {
            for (session_id, session) in sessions.borrow().iter() {
                if now - session.created_at > expiry_ms {
                    expired_sessions.push(session_id);
                }
            }
        });

        // Remove expired sessions
        for session_id in expired_sessions {
            self.cleanup(&crate::upload::types::SessionId(session_id));
        }
    }

    /// List all sessions for debugging
    pub fn list_all_sessions(&self) -> Vec<(u64, SessionMeta)> {
        STABLE_UPLOAD_SESSIONS.with(|sessions| {
            sessions
                .borrow()
                .iter()
                .map(|(id, session)| (id, session.clone()))
                .collect()
        })
    }
}

pub struct ChunkIterator {
    session_id: u64,
    chunk_count: u32,
    current_idx: u32,
}

impl Iterator for ChunkIterator {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_idx >= self.chunk_count {
            return None;
        }

        let chunk_key = (self.session_id, self.current_idx);
        let chunk = STABLE_CHUNK_DATA.with(|chunks| chunks.borrow().get(&chunk_key));

        self.current_idx += 1;
        chunk
    }
}

impl ExactSizeIterator for ChunkIterator {
    fn len(&self) -> usize {
        (self.chunk_count - self.current_idx) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AssetMetadata, AssetMetadataBase, AssetType, ImageAssetMetadata};
    use crate::upload::types::SessionStatus;
    use candid::Principal;

    fn create_test_session_meta_with_status(status: SessionStatus) -> SessionMeta {
        let caller = Principal::anonymous();
        SessionMeta {
            capsule_id: "test-capsule".to_string(),
            provisional_memory_id: "test-memory".to_string(),
            caller,
            chunk_count: 3,
            expected_len: Some(300),
            expected_hash: Some([0u8; 32]),
            status,
            created_at: 1234567890,
            asset_metadata: AssetMetadata::Image(ImageAssetMetadata {
                base: AssetMetadataBase {
                    name: "test.txt".to_string(),
                    description: Some("Test file".to_string()),
                    tags: vec!["test".to_string()],
                    asset_type: AssetType::Original,
                    bytes: 300,
                    mime_type: "text/plain".to_string(),
                    sha256: Some([0u8; 32]),
                    width: None,
                    height: None,
                    url: None,
                    storage_key: None,
                    bucket: None,
                    processing_status: None,
                    processing_error: None,
                    created_at: 1234567890,
                    updated_at: 1234567890,
                    deleted_at: None,
                    asset_location: None,
                },
                color_space: None,
                exif_data: None,
                compression_ratio: None,
                dpi: None,
                orientation: None,
            }),
            idem: "test-idem".to_string(),
        }
    }

    #[test]
    fn test_clear_all_sessions() {
        let store = SessionStore::new();

        // Create some test sessions
        let session1 = SessionId::new();
        let session2 = SessionId::new();
        let meta1 = create_test_session_meta_with_status(SessionStatus::Pending);
        let meta2 = create_test_session_meta_with_status(SessionStatus::Committed { blob_id: 123 });

        // Add sessions
        store.create(session1.clone(), meta1).unwrap();
        store.create(session2.clone(), meta2).unwrap();

        // Verify sessions exist
        assert!(store.get(&session1).unwrap().is_some());
        assert!(store.get(&session2).unwrap().is_some());
        assert_eq!(store.total_session_count(), 2);

        // Clear all sessions
        store.clear_all_sessions();

        // Verify sessions are gone
        assert_eq!(store.total_session_count(), 0);
        assert!(store.get(&session1).unwrap().is_none());
        assert!(store.get(&session2).unwrap().is_none());
    }

    #[test]
    fn test_total_session_count() {
        let store = SessionStore::new();

        // Initially no sessions
        assert_eq!(store.total_session_count(), 0);

        // Add some sessions
        let session1 = SessionId::new();
        let session2 = SessionId::new();
        let meta1 = create_test_session_meta_with_status(SessionStatus::Pending);
        let meta2 = create_test_session_meta_with_status(SessionStatus::Committed { blob_id: 123 });

        store.create(session1, meta1).unwrap();
        assert_eq!(store.total_session_count(), 1);

        store.create(session2, meta2).unwrap();
        assert_eq!(store.total_session_count(), 2);
    }

    #[test]
    fn test_session_count_by_status() {
        let store = SessionStore::new();

        // Initially no sessions
        let (pending, committed) = store.session_count_by_status();
        assert_eq!(pending, 0);
        assert_eq!(committed, 0);

        // Add pending session
        let session1 = SessionId::new();
        let meta1 = create_test_session_meta_with_status(SessionStatus::Pending);
        store.create(session1, meta1).unwrap();

        let (pending, committed) = store.session_count_by_status();
        assert_eq!(pending, 1);
        assert_eq!(committed, 0);

        // Add committed session
        let session2 = SessionId::new();
        let meta2 = create_test_session_meta_with_status(SessionStatus::Committed { blob_id: 123 });
        store.create(session2, meta2).unwrap();

        let (pending, committed) = store.session_count_by_status();
        assert_eq!(pending, 1);
        assert_eq!(committed, 1);

        // Add another pending session
        let session3 = SessionId::new();
        let meta3 = create_test_session_meta_with_status(SessionStatus::Pending);
        store.create(session3, meta3).unwrap();

        let (pending, committed) = store.session_count_by_status();
        assert_eq!(pending, 2);
        assert_eq!(committed, 1);
    }
}
