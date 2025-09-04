use crate::memory_manager::{MEM_CHUNKS, MEM_SESSIONS, MEM_SESSION_COUNTER, MM};
use crate::types::Error;
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
        StableBTreeMap::init(MM.with(|m| m.borrow().get(MEM_CHUNKS)))
    );

    pub static STABLE_SESSION_COUNTER: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(MM.with(|m| m.borrow().get(MEM_SESSION_COUNTER)), 0)
            .expect("Failed to init session counter")
    );
}

/// Session store for managing upload sessions and chunks
// During the storage migration, some upload APIs are not wired yet in
// production paths. Silence "associated items are never used" locally,
// but keep the code compiled and testable. Enable strictly when the
// `upload` feature is on.
#[cfg_attr(not(feature = "upload"), allow(dead_code))]
pub struct SessionStore;

#[cfg_attr(not(feature = "upload"), allow(dead_code))]
impl SessionStore {
    pub fn new() -> Self {
        SessionStore
    }

    /// Create a new upload session
    pub fn create(&self, session_id: SessionId, session_meta: SessionMeta) -> Result<(), Error> {
        STABLE_UPLOAD_SESSIONS.with(|sessions| {
            sessions.borrow_mut().insert(session_id.0, session_meta);
        });
        Ok(())
    }

    /// Get session metadata
    pub fn get(&self, session_id: &SessionId) -> Result<Option<SessionMeta>, Error> {
        let session = STABLE_UPLOAD_SESSIONS.with(|sessions| sessions.borrow().get(&session_id.0));
        Ok(session)
    }

    /// Update session metadata (for status changes)
    pub fn update(&self, session_id: &SessionId, session_meta: SessionMeta) -> Result<(), Error> {
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
    ) -> Result<(), Error> {
        let chunk_key = (session_id.0, chunk_idx);
        STABLE_CHUNK_DATA.with(|chunks| {
            chunks.borrow_mut().insert(chunk_key, bytes);
        });
        Ok(())
    }

    /// Get a specific chunk
    pub fn get_chunk(
        &self,
        session_id: &SessionId,
        chunk_idx: u32,
    ) -> Result<Option<Vec<u8>>, Error> {
        let chunk_key = (session_id.0, chunk_idx);
        let chunk = STABLE_CHUNK_DATA.with(|chunks| chunks.borrow().get(&chunk_key));
        Ok(chunk)
    }

    /// Verify all chunks exist for a session (integrity check)
    pub fn verify_chunks_complete(
        &self,
        session_id: &SessionId,
        chunk_count: u32,
    ) -> Result<(), Error> {
        for chunk_idx in 0..chunk_count {
            let chunk_key = (session_id.0, chunk_idx);
            let exists = STABLE_CHUNK_DATA.with(|chunks| chunks.borrow().contains_key(&chunk_key));

            if !exists {
                return Err(Error::ChunkNotFound);
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
}

/// Iterator for streaming chunks in order
#[cfg_attr(not(feature = "upload"), allow(dead_code))]
pub struct ChunkIterator {
    session_id: u64,
    chunk_count: u32,
    current_idx: u32,
}

#[cfg_attr(not(feature = "upload"), allow(dead_code))]
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

#[cfg_attr(not(feature = "upload"), allow(dead_code))]
impl ExactSizeIterator for ChunkIterator {
    fn len(&self) -> usize {
        (self.chunk_count - self.current_idx) as usize
    }
}
