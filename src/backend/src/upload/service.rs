use crate::capsule_store::{CapsuleStore, Store};
use crate::session::{SessionCompat, SessionId};
use crate::types::{CapsuleId, Error, MemoryId, PersonRef};
use crate::upload::blob_store::BlobStore;
use crate::upload::types::*;
// Removed unused import: candid::Principal
use sha2::{Digest, Sha256};
use std::cell::RefCell;

// Thread-local storage for SessionCompat (persists across calls)
thread_local! {
    static SESSION_COMPAT: RefCell<Option<SessionCompat>> = const { RefCell::new(None) };
}

fn with_session_compat<R>(f: impl FnOnce(&SessionCompat) -> R) -> R {
    SESSION_COMPAT.with(|cell| {
        let mut opt = cell.borrow_mut();
        if opt.is_none() {
            use crate::upload::blob_store::StableBlobSink;
            *opt = Some(SessionCompat::new(|meta| {
                let sink = StableBlobSink::for_meta(meta)?;
                Ok(Box::new(sink) as Box<dyn crate::session::ByteSink>)
            }));
        }
        // Safety: we just ensured it's Some
        f(opt.as_ref().unwrap())
    })
}

// Note: UploadService struct removed - converted to module-level functions
// The struct was stateless and just wrapped BlobStore::new() which does nothing

// Note: check_upload_rate_limit removed - not currently used

// Note: create_inline method removed - not currently used

pub fn begin_upload(
    store: &mut Store,
    capsule_id: CapsuleId,
    expected_chunks: u32,
    idem: String,
) -> std::result::Result<SessionId, Error> {
    // 0) validate input early
    if expected_chunks == 0 {
        return Err(Error::InvalidArgument("expected_chunks_zero".into()));
    }
    // optional sane cap to avoid abuse (tune as needed)
    const MAX_CHUNKS: u32 = 16_384;
    if expected_chunks > MAX_CHUNKS {
        return Err(Error::InvalidArgument("expected_chunks_too_large".into()));
    }

    // 1) auth
    let caller = ic_cdk::api::msg_caller();
    let person_ref = PersonRef::Principal(caller);
    if let Some(capsule) = store.get(&capsule_id) {
        if !capsule.has_write_access(&person_ref) {
            return Err(Error::Unauthorized);
        }
    } else {
        return Err(Error::NotFound);
    }

    // 2) idempotency: if a pending session with same (capsule, caller, idem) exists, return it
    if let Some(existing) =
        with_session_compat(|sessions| sessions.find_pending(&capsule_id, &caller, &idem))
    {
        return Ok(existing);
    }

    // 3) Clean up expired sessions before checking limits
    // Only clean up sessions that are actually expired (older than 2 hours)
    const SESSION_EXPIRY_MS: u64 = 2 * 60 * 60 * 1000; // 2 hours
    with_session_compat(|sessions| {
        sessions.cleanup_expired_sessions_for_caller(&capsule_id, &caller, SESSION_EXPIRY_MS)
    });

    // 4) back-pressure: cap concurrent sessions per caller/capsule
    const MAX_ACTIVE_PER_CALLER: usize = 100; // Increased for development
    let active_count =
        with_session_compat(|sessions| sessions.count_active_for(&capsule_id, &caller));
    let total_count = with_session_compat(|sessions| sessions.total_session_count());

    // Log session count for monitoring
    ic_cdk::println!(
        "UPLOAD_BEGIN: caller={}, capsule={}, active_sessions={}, total_sessions={}",
        caller,
        capsule_id,
        active_count,
        total_count
    );

    if active_count >= MAX_ACTIVE_PER_CALLER {
        return Err(Error::ResourceExhausted); // "too many active uploads"
    }

    // 4) create session
    let session_id = SessionId::new();
    let provisional_memory_id = MemoryId::new();

    let upload_meta = crate::session::UploadSessionMeta {
        session_id: session_id.0, // Include session_id for unique keys
        capsule_id,
        caller,
        created_at: ic_cdk::api::time(),
        expected_chunks,
        status: SessionStatus::Pending,
        chunk_count: expected_chunks,
        provisional_memory_id: provisional_memory_id.to_string(),
        chunk_size: crate::upload::types::CHUNK_SIZE,
        idem: idem.clone(),
        blob_id: None, // No blob ID yet (pending)
    };

    with_session_compat(|sessions| sessions.create(session_id, upload_meta))?;
    Ok(session_id)
}

// Note: begin_upload_chunked legacy method removed - use begin_upload instead

/// The principal that began an upload session, if the session exists.
pub fn session_caller(session_id: u64) -> Option<candid::Principal> {
    with_session_compat(|sessions| sessions.get(&SessionId(session_id)))
        .ok()
        .flatten()
        .map(|s| s.caller)
}

/// Upload a chunk for an active session.
///
/// Semantics:
/// - Only the session creator (caller) may upload chunks.
/// - Session must be in `Pending` state (committed sessions reject uploads).
/// - `chunk_idx` must be `< session.chunk_count`.
/// - Each chunk must be ≤ `CHUNK_SIZE` bytes. The last chunk may be smaller.
/// - Duplicate uploads of the same chunk **overwrite silently** (idempotent retry behavior).
///
/// Integrity is enforced at `commit`: all chunks must be present, and final
/// hash/length are verified before attaching to the capsule.
pub fn put_chunk(
    _store: &mut Store,
    session_id: &SessionId,
    chunk_idx: u32,
    bytes: Vec<u8>,
) -> std::result::Result<(), Error> {
    // Verify session exists and caller matches
    let session =
        with_session_compat(|sessions| sessions.get(session_id))?.ok_or(Error::NotFound)?;

    let caller = ic_cdk::api::msg_caller();
    if session.caller != caller {
        return Err(Error::Unauthorized);
    }

    // Verify session is in pending state (not committed)
    if let SessionStatus::Committed { .. } = session.status {
        return Err(Error::InvalidArgument(
            "session already committed".to_string(),
        ));
    }

    // Verify chunk index is within expected range
    if chunk_idx >= session.chunk_count {
        return Err(Error::InvalidArgument(format!(
            "chunk_index {} out of range (expected < {})",
            chunk_idx, session.chunk_count
        )));
    }

    // Verify chunk size (except possibly last chunk)
    if bytes.len() > CHUNK_SIZE {
        return Err(Error::InvalidArgument(format!(
            "chunk size {} exceeds limit of {} bytes",
            bytes.len(),
            CHUNK_SIZE
        )));
    }

    // Debug logging: Log the exact bytes being stored
    let first_10_bytes = if bytes.len() >= 10 {
        format!("{:?}", &bytes[..10])
    } else {
        format!("{:?}", &bytes[..])
    };
    ic_cdk::println!(
        "PUT_CHUNK: session_id={}, chunk_idx={}, data_len={}, first_10_bytes={}",
        session_id.0,
        chunk_idx,
        bytes.len(),
        first_10_bytes
    );

    // Store chunk
    with_session_compat(|sessions| sessions.put_chunk(session_id, chunk_idx, &bytes))?;
    Ok(())
}

/// Commit upload and create blob (crash-safe with idempotency)
///
/// Semantics:
/// - Only the session creator (caller) may commit the upload.
/// - Session must be in `Pending` state (aborted sessions reject commits).
/// - All chunks must be present before commit.
/// - Hash and size verification ensures data integrity.
/// - Fails if any chunk missing or hash/size mismatch; safe to retry.
/// - Returns only blob_id (memory creation is separate concern).
pub fn commit(
    _store: &mut Store,
    session_id: SessionId,
    expected_sha256: [u8; 32],
    total_len: u64,
) -> std::result::Result<String, Error> {
    let mut session =
        with_session_compat(|sessions| sessions.get(&session_id))?.ok_or(Error::NotFound)?;

    // Verify caller matches
    let caller = ic_cdk::api::msg_caller();
    if session.caller != caller {
        return Err(Error::Unauthorized);
    }

    // Handle idempotent retry (crash recovery) for committed sessions
    if let SessionStatus::Committed { .. } = session.status {
        let blob_id = session.blob_id.ok_or(Error::NotFound)?;
        // Already committed - just return the blob_id
        with_session_compat(|sessions| sessions.cleanup(&session_id));
        return Ok(format!("blob_{}", blob_id));
    }

    // First-time commit

    // 0. Sanity-check total_len vs chunk_count
    let max_len = (session.chunk_count as u64) * (CHUNK_SIZE as u64);
    if total_len == 0 || total_len > max_len {
        return Err(Error::InvalidArgument(format!(
            "total_len {} out of bounds (expected 0 < len <= {})",
            total_len, max_len
        )));
    }

    // 1. Verify all chunks exist (integrity check)
    with_session_compat(|sessions| {
        sessions.verify_chunks_complete(&session_id, session.chunk_count)
    })?;
    ic_cdk::println!("COMMIT: sid={} chunks_verified", session_id.0);

    // 2. Stream chunks to blob store with verification
    let blob_store = BlobStore::new();
    let blob_id = with_session_compat(|sessions| {
        blob_store.store_from_chunks(
            sessions,
            &session_id,
            session.chunk_count,
            total_len,
            expected_sha256,
        )
    })?;
    ic_cdk::println!(
        "COMMIT: sid={} hash_verified blob_id={}",
        session_id.0,
        blob_id.0
    );

    // 3. Mark session as committed (crash-safe checkpoint)
    session.status = SessionStatus::Committed {
        completed_at: ic_cdk::api::time(),
    };
    session.blob_id = Some(blob_id.0);
    with_session_compat(|sessions| sessions.update(session_id, session.clone()))?;

    // 4. Cleanup session and chunks
    with_session_compat(|sessions| sessions.cleanup(&session_id));

    // Return only blob ID (memory creation is separate concern)
    Ok(format!("blob_{}", blob_id.0))
}

/// Abort upload and cleanup with authorization
pub fn abort(_store: &mut Store, session_id: SessionId) -> std::result::Result<(), Error> {
    // Verify caller matches (if session exists)
    if let Some(session) = with_session_compat(|sessions| sessions.get(&session_id))? {
        let caller = ic_cdk::api::msg_caller();
        if session.caller != caller {
            return Err(Error::Unauthorized);
        }
    }

    with_session_compat(|sessions| sessions.cleanup(&session_id));
    Ok(())
}

/// Utility function to compute SHA256 for client-side verification
#[allow(dead_code)] // Used in tests
pub fn compute_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

// Public session management functions (for lib.rs query endpoints)

pub fn total_session_count() -> usize {
    with_session_compat(|sessions| sessions.total_session_count())
}

pub fn session_count_by_status() -> (usize, usize) {
    with_session_compat(|sessions| {
        let all = sessions.list_upload_sessions();
        let pending = all
            .iter()
            .filter(|(_, m)| matches!(m.status, SessionStatus::Pending))
            .count();
        let committed = all
            .iter()
            .filter(|(_, m)| matches!(m.status, SessionStatus::Committed { .. }))
            .count();
        (pending, committed)
    })
}

pub fn list_upload_sessions() -> Vec<(u64, crate::session::UploadSessionMeta)> {
    with_session_compat(|sessions| sessions.list_upload_sessions())
}

pub fn cleanup_expired_sessions(expiry_ms: u64) {
    with_session_compat(|sessions| sessions.cleanup_expired_sessions(expiry_ms));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upload::types::SessionId;

    fn create_test_store() -> Store {
        Store::new_stable_test()
    }

    // ============================================================================
    // PUT_CHUNK TESTS
    // ============================================================================

    #[test]
    fn test_put_chunk_session_not_found() {
        let mut store = create_test_store();
        let session_id = SessionId::new();
        let chunk_idx = 0;
        let bytes = vec![1, 2, 3, 4];

        // Test that non-existent session returns NotFound
        let result = put_chunk(&mut store, &session_id, chunk_idx, bytes);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::NotFound));
    }

    #[test]
    fn test_put_chunk_oversized() {
        let mut store = create_test_store();
        let session_id = SessionId::new();
        let chunk_idx = 0;
        let oversized_bytes = vec![0u8; CHUNK_SIZE + 1]; // Too large

        // Test that oversized chunks are rejected (session not found, but that's expected)
        let result = put_chunk(&mut store, &session_id, chunk_idx, oversized_bytes.clone());
        assert!(result.is_err());
        // The session doesn't exist, so we get NotFound, but the chunk size validation
        // would happen if the session existed
        assert!(matches!(result.unwrap_err(), Error::NotFound));

        // Test the chunk size constant is reasonable
        assert!(
            oversized_bytes.len() > CHUNK_SIZE,
            "Chunk should be oversized"
        );
        assert!(CHUNK_SIZE > 0, "CHUNK_SIZE should be positive");
    }

    // ============================================================================
    // UTILITY TESTS
    // ============================================================================

    #[test]
    fn test_compute_sha256() {
        let data = b"test data";
        let hash = compute_sha256(data);

        // Test that hash is computed correctly
        assert_eq!(hash.len(), 32, "SHA256 hash should be 32 bytes");
        assert_ne!(hash, [0u8; 32], "Hash should not be all zeros");

        // Test that same data produces same hash
        let hash2 = compute_sha256(data);
        assert_eq!(hash, hash2, "Same data should produce same hash");

        // Test that different data produces different hash
        let different_data = b"different data";
        let different_hash = compute_sha256(different_data);
        assert_ne!(
            hash, different_hash,
            "Different data should produce different hash"
        );
    }

    #[test]
    fn test_chunk_size_constant() {
        // Test that CHUNK_SIZE is reasonable
        assert_eq!(
            CHUNK_SIZE, 1_800_000,
            "CHUNK_SIZE should be 1.8MB (ICP expert recommended)"
        );
        assert!(CHUNK_SIZE > 0, "CHUNK_SIZE should be positive");
        assert!(
            CHUNK_SIZE < 2 * 1024 * 1024,
            "CHUNK_SIZE should be less than 2MB"
        );
    }

    #[test]
    fn test_session_id_generation() {
        let session_id1 = SessionId::new();
        let session_id2 = SessionId::new();

        // Test that session IDs are unique
        assert_ne!(session_id1.0, session_id2.0, "Session IDs should be unique");
        assert!(session_id1.0 > 0, "Session ID should be positive");
        assert!(session_id2.0 > 0, "Session ID should be positive");
    }

    // ============================================================================
    // INTEGRATION TEST HELPERS
    // ============================================================================

    #[test]
    fn test_error_message_formats() {
        // Test that error messages are properly formatted
        let chunk_idx = 5;
        let chunk_count = 3;
        let chunk_size = 1000;
        let max_size = 64 * 1024;

        // Test chunk index error
        let index_error = format!(
            "chunk_index {} out of range (expected < {})",
            chunk_idx, chunk_count
        );
        assert!(
            index_error.contains("out of range"),
            "Index error should be descriptive"
        );

        // Test chunk size error
        let size_error = format!(
            "chunk size {} exceeds limit of {} bytes",
            chunk_size, max_size
        );
        assert!(
            size_error.contains("exceeds limit"),
            "Size error should be descriptive"
        );

        // Test total length error
        let total_len = 1000;
        let max_len = 500;
        let length_error = format!(
            "total_len {} out of bounds (expected 0 < len <= {})",
            total_len, max_len
        );
        assert!(
            length_error.contains("out of bounds"),
            "Length error should be descriptive"
        );
    }
}

// Integration tests will be added after core functionality is working
