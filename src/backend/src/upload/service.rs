use crate::capsule_store::{CapsuleStore, Store};
use crate::types::{CapsuleId, Error, Memory, MemoryId, MemoryMeta, PersonRef};
use crate::upload::types::*;
use crate::upload::{BlobStore, SessionStore};
use sha2::{Digest, Sha256};

/// Upload service using concrete Store enum (no trait objects)
/// Provides both inline and chunked upload workflows
#[cfg_attr(not(feature = "upload"), allow(dead_code))]
pub struct UploadService<'a> {
    store: &'a mut Store,
    sessions: SessionStore,
    blobs: BlobStore,
}

#[cfg_attr(not(feature = "upload"), allow(dead_code))]
impl<'a> UploadService<'a> {
    pub fn new(store: &'a mut Store) -> Self {
        Self {
            store,
            sessions: SessionStore::new(),
            blobs: BlobStore::new(),
        }
    }

    /// Inline-only endpoint - rejects large payloads at ingress
    pub fn create_inline(
        &mut self,
        capsule_id: &CapsuleId,
        bytes: Vec<u8>,
        meta: MemoryMeta,
    ) -> Result<MemoryId, Error> {
        if bytes.len() > INLINE_MAX {
            return Err(Error::ResourceExhausted);
        }

        // Check per-capsule inline budget
        let current_inline_size = self
            .store
            .get(capsule_id)
            .map(|capsule| {
                capsule
                    .memories
                    .values()
                    .filter_map(|m| m.data.data.as_ref().map(|data| data.len()))
                    .sum::<usize>()
            })
            .unwrap_or(0);

        if current_inline_size + bytes.len() > CAPSULE_INLINE_BUDGET {
            return Err(Error::ResourceExhausted);
        }

        // Verify caller has write access
        let caller = ic_cdk::api::msg_caller();
        let person_ref = PersonRef::Principal(caller);
        if let Some(capsule) = self.store.get(capsule_id) {
            if !capsule.has_write_access(&person_ref) {
                return Err(Error::Unauthorized);
            }
        } else {
            return Err(Error::NotFound);
        }

        let memory = Memory::inline(bytes, meta);
        let memory_id = memory.id.clone();

        // Atomic update to capsule using the existing pattern
        self.store.update(capsule_id, |capsule| {
            capsule.memories.insert(memory_id.clone(), memory);
            capsule.updated_at = ic_cdk::api::time();
        })?;

        Ok(memory_id)
    }

    /// Begin chunked upload for large files
    pub fn begin_upload(
        &mut self,
        capsule_id: CapsuleId,
        meta: MemoryMeta,
        expected_chunks: u32,
    ) -> Result<SessionId, Error> {
        // Verify caller has write access
        let caller = ic_cdk::api::msg_caller();
        let person_ref = PersonRef::Principal(caller);
        if let Some(capsule) = self.store.get(&capsule_id) {
            if !capsule.has_write_access(&person_ref) {
                return Err(Error::Unauthorized);
            }
        } else {
            return Err(Error::NotFound);
        }

        let session_id = SessionId::new();
        let provisional_memory_id = MemoryId::new();

        let session_meta = SessionMeta {
            capsule_id,
            provisional_memory_id,
            caller,
            chunk_count: expected_chunks,
            expected_len: None,
            expected_hash: None,
            status: SessionStatus::Pending,
            created_at: ic_cdk::api::time(),
            meta,
        };

        self.sessions.create(session_id.clone(), session_meta)?;
        Ok(session_id)
    }

    /// Upload chunk with authorization and bounds checking
    pub fn put_chunk(
        &mut self,
        session_id: &SessionId,
        chunk_idx: u32,
        bytes: Vec<u8>,
    ) -> Result<(), Error> {
        // Verify session exists and caller matches
        let session = self
            .sessions
            .get(session_id)?
            .ok_or(Error::NotFound)?;

        let caller = ic_cdk::api::msg_caller();
        if session.caller != caller {
            return Err(Error::Unauthorized);
        }

        // Verify chunk index is within expected range
        if chunk_idx >= session.chunk_count {
            return Err(Error::InvalidArgument("chunk_index".to_string()));
        }

        // Verify chunk size (except possibly last chunk)
        if bytes.len() > CHUNK_SIZE {
            return Err(Error::ResourceExhausted);
        }

        // Store chunk
        self.sessions.put_chunk(session_id, chunk_idx, bytes)?;
        Ok(())
    }

    /// Commit upload and attach to capsule (crash-safe with idempotency)
    pub fn commit(
        &mut self,
        session_id: SessionId,
        expected_sha256: [u8; 32],
        total_len: u64,
    ) -> Result<MemoryId, Error> {
        let mut session = self
            .sessions
            .get(&session_id)?
            .ok_or(Error::NotFound)?;

        // Verify caller matches
        let caller = ic_cdk::api::msg_caller();
        if session.caller != caller {
            return Err(Error::Unauthorized);
        }

        // Handle idempotent retry (crash recovery)
        if let SessionStatus::Committed { blob_id } = session.status {
            // Check if already attached to capsule
            if let Some(capsule) = self.store.get(&session.capsule_id) {
                if capsule
                    .memories
                    .contains_key(&session.provisional_memory_id)
                {
                    // Already committed and attached
                    self.sessions.cleanup(&session_id);
                    return Ok(session.provisional_memory_id);
                }
            }

            // Blob exists but not attached - retry attach
            let memory =
                Memory::from_blob(blob_id, total_len, expected_sha256, session.meta.clone());
            let memory_id = memory.id.clone();

            self.store.update(&session.capsule_id, |capsule| {
                capsule.memories.insert(memory_id.clone(), memory);
                capsule.updated_at = ic_cdk::api::time();
            })?;

            self.sessions.cleanup(&session_id);
            return Ok(memory_id);
        }

        // First-time commit

        // 1. Verify all chunks exist (integrity check)
        self.sessions
            .verify_chunks_complete(&session_id, session.chunk_count)?;

        // 2. Stream chunks to blob store with verification
        let blob_id = self.blobs.store_from_chunks(
            &self.sessions,
            &session_id,
            session.chunk_count,
            total_len,
            expected_sha256,
        )?;

        // 3. Mark session as committed (crash-safe checkpoint)
        session.status = SessionStatus::Committed { blob_id: blob_id.0 };
        self.sessions.update(&session_id, session.clone())?;

        // 4. Create memory with blob reference
        let memory = Memory::from_blob(blob_id.0, total_len, expected_sha256, session.meta.clone());
        let memory_id = memory.id.clone();

        // 5. Atomic attach to capsule
        self.store.update(&session.capsule_id, |capsule| {
            capsule.memories.insert(memory_id.clone(), memory);
            capsule.updated_at = ic_cdk::api::time();
        })?;

        // 6. Cleanup session and chunks
        self.sessions.cleanup(&session_id);

        Ok(memory_id)
    }

    /// Abort upload and cleanup with authorization
    pub fn abort(&mut self, session_id: SessionId) -> Result<(), Error> {
        // Verify caller matches (if session exists)
        if let Some(session) = self.sessions.get(&session_id)? {
            let caller = ic_cdk::api::msg_caller();
            if session.caller != caller {
                return Err(Error::Unauthorized);
            }
        }

        self.sessions.cleanup(&session_id);
        Ok(())
    }

    /// Utility function to compute SHA256 for client-side verification
    pub fn compute_sha256(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }
}

// Tests will be added after core functionality is working
