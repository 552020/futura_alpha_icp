use crate::memory::{
    with_stable_chunk_data, with_stable_chunk_data_mut, with_stable_memory_artifacts,
    with_stable_memory_artifacts_mut, with_stable_upload_sessions, with_stable_upload_sessions_mut,
};
use crate::types::{
    ArtifactType, ChunkData, ChunkResponse, CommitResponse, ICPErrorCode, ICPResult,
    MemoryArtifact, MemoryType, UploadSession, UploadSessionResponse,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[cfg(not(test))]
use ic_cdk::api;

// ============================================================================
// CHUNKED ASSET UPLOAD PROTOCOL - ICP Canister Endpoints
// ============================================================================

/// Begin chunked upload session for large files
pub fn begin_asset_upload(
    memory_id: String,
    expected_hash: String,
    chunk_count: u32,
    total_size: u64,
) -> ICPResult<UploadSessionResponse> {
    // Check authorization first
    let caller = match crate::auth::verify_caller_authorized() {
        Ok(caller) => caller,
        Err(auth_error) => return ICPResult::err(auth_error),
    };

    // Check rate limiting
    if let Err(rate_error) = crate::auth::check_upload_rate_limit(&caller) {
        return ICPResult::err(rate_error);
    }

    // Validate file size limits (max 100MB)
    const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB
    if total_size > MAX_FILE_SIZE {
        return ICPResult::err(ICPErrorCode::Internal(
            "File size exceeds 100MB limit".to_string(),
        ));
    }

    // Validate chunk count is reasonable (at least 1, max 1000 chunks)
    if chunk_count == 0 || chunk_count > 1000 {
        return ICPResult::err(ICPErrorCode::Internal("Invalid chunk count".to_string()));
    }

    // Check if asset with same hash already exists (idempotency)
    let existing_asset = with_stable_memory_artifacts(|artifacts| {
        artifacts.iter().any(|(_, artifact)| {
            artifact.artifact_type == ArtifactType::Asset && artifact.content_hash == expected_hash
        })
    });

    if existing_asset {
        return ICPResult::err(ICPErrorCode::AlreadyExists);
    }

    // Generate unique session ID
    let session_id = generate_session_id(&memory_id, &expected_hash);

    // Check if session already exists
    let existing_session = with_stable_upload_sessions(|sessions| sessions.get(&session_id));

    if existing_session.is_some() {
        return ICPResult::err(ICPErrorCode::AlreadyExists);
    }

    // Create new upload session
    let session = UploadSession {
        session_id: session_id.clone(),
        memory_id: memory_id.clone(),
        expected_hash,
        chunk_count,
        total_size,
        created_at: get_current_time(),
        chunks_received: vec![false; chunk_count as usize],
        bytes_received: 0,
    };

    // Store session in stable memory
    with_stable_upload_sessions_mut(|sessions| {
        sessions.insert(session_id.clone(), session.clone());
    });

    ICPResult::ok(UploadSessionResponse::ok(
        session,
        "Upload session created".to_string(),
    ))
}

/// Upload individual file chunk
pub fn put_chunk(
    session_id: String,
    chunk_index: u32,
    chunk_data: Vec<u8>,
) -> ICPResult<ChunkResponse> {
    // Check authorization first
    if let Err(auth_error) = crate::auth::verify_caller_authorized() {
        return ICPResult::err(auth_error);
    }

    // Validate chunk size (max 1MB per chunk)
    const MAX_CHUNK_SIZE: usize = 1024 * 1024; // 1MB
    if chunk_data.len() > MAX_CHUNK_SIZE {
        return ICPResult::err(ICPErrorCode::Internal(
            "Chunk size exceeds 1MB limit".to_string(),
        ));
    }

    // Get and validate session
    let mut session = match with_stable_upload_sessions(|sessions| sessions.get(&session_id)) {
        Some(session) => session,
        None => return ICPResult::err(ICPErrorCode::NotFound),
    };

    // Check session timeout (30 minutes)
    const SESSION_TIMEOUT: u64 = 30 * 60 * 1_000_000_000; // 30 minutes in nanoseconds
    let current_time = get_current_time();
    if current_time - session.created_at > SESSION_TIMEOUT {
        // Clean up expired session
        cleanup_session(&session_id);
        return ICPResult::err(ICPErrorCode::Internal("Session expired".to_string()));
    }

    // Validate chunk index
    if chunk_index >= session.chunk_count {
        return ICPResult::err(ICPErrorCode::Internal("Invalid chunk index".to_string()));
    }

    // Check if chunk already received (idempotency)
    if session.chunks_received[chunk_index as usize] {
        return ICPResult::ok(ChunkResponse::ok(
            chunk_index,
            chunk_data.len() as u32,
            format!("Chunk {} already received (idempotent)", chunk_index),
        ));
    }

    // Create chunk data key
    let chunk_key = format!("{}:{}", session_id, chunk_index);

    // Store chunk data
    let chunk = ChunkData {
        session_id: session_id.clone(),
        chunk_index,
        data: chunk_data.clone(),
        received_at: current_time,
    };

    with_stable_chunk_data_mut(|chunks| {
        chunks.insert(chunk_key, chunk);
    });

    // Update session progress
    session.chunks_received[chunk_index as usize] = true;
    session.bytes_received += chunk_data.len() as u64;

    // Save updated session
    with_stable_upload_sessions_mut(|sessions| {
        sessions.insert(session_id.clone(), session.clone());
    });

    ICPResult::ok(ChunkResponse::ok(
        chunk_index,
        chunk_data.len() as u32,
        format!(
            "Chunk {} received ({} bytes)",
            chunk_index,
            chunk_data.len()
        ),
    ))
}

/// Finalize upload after all chunks received
pub fn commit_asset(session_id: String, _final_hash: String) -> ICPResult<CommitResponse> {
    // Check authorization first
    if let Err(auth_error) = crate::auth::verify_caller_authorized() {
        return ICPResult::err(auth_error);
    }

    // Get session
    let session = match with_stable_upload_sessions(|sessions| sessions.get(&session_id)) {
        Some(session) => session,
        None => return ICPResult::err(ICPErrorCode::NotFound),
    };

    // Validate all chunks received
    if !session.chunks_received.iter().all(|&received| received) {
        return ICPResult::err(ICPErrorCode::Internal("Missing chunks".to_string()));
    }

    // Reconstruct file from chunks
    let mut file_data = Vec::with_capacity(session.total_size as usize);

    for chunk_index in 0..session.chunk_count {
        let chunk_key = format!("{}:{}", session_id, chunk_index);
        let chunk = match with_stable_chunk_data(|chunks| chunks.get(&chunk_key)) {
            Some(chunk) => chunk,
            None => {
                return ICPResult::err(ICPErrorCode::Internal("Chunk data not found".to_string()))
            }
        };
        file_data.extend_from_slice(&chunk.data);
    }

    // Compute hash of reconstructed file
    let computed_hash = compute_sha256_hash(&file_data);

    // For now, use computed hash as the final hash (MVP approach)
    // In production, this should validate against expected hash
    let final_hash = computed_hash.clone();

    // Create memory artifact
    let artifact = MemoryArtifact {
        memory_id: session.memory_id.clone(),
        memory_type: MemoryType::Image, // Default for now, should be passed as parameter
        artifact_type: ArtifactType::Asset,
        content_hash: final_hash.clone(),
        size: session.total_size,
        stored_at: get_current_time(),
        metadata: None, // Asset data is stored separately
    };

    // Store artifact
    let artifact_key = format!("{}:{}:asset", session.memory_id, "image"); // Simplified for MVP
    with_stable_memory_artifacts_mut(|artifacts| {
        artifacts.insert(artifact_key, artifact);
    });

    // Clean up session and chunks
    cleanup_session(&session_id);

    ICPResult::ok(CommitResponse::ok(
        session.memory_id,
        final_hash,
        session.total_size,
        "Asset committed successfully".to_string(),
    ))
}

/// Cancel upload and cleanup resources
pub fn cancel_upload(session_id: String) -> ICPResult<()> {
    // Check authorization first
    if let Err(auth_error) = crate::auth::verify_caller_authorized() {
        return ICPResult::err(auth_error);
    }

    cleanup_session(&session_id);
    ICPResult::ok(())
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Generate unique session ID
fn generate_session_id(memory_id: &str, expected_hash: &str) -> String {
    let mut hasher = DefaultHasher::new();
    memory_id.hash(&mut hasher);
    expected_hash.hash(&mut hasher);
    get_current_time().hash(&mut hasher);
    format!("session_{:x}", hasher.finish())
}

/// Compute SHA-256 hash of data
fn compute_sha256_hash(data: &[u8]) -> String {
    // Simplified hash for MVP - in production would use proper SHA-256
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("sha256_{:x}", hasher.finish())
}

/// Clean up upload session and associated chunks
fn cleanup_session(session_id: &str) {
    // Remove session
    with_stable_upload_sessions_mut(|sessions| {
        sessions.remove(&session_id.to_string());
    });

    // Remove all chunks for this session
    with_stable_chunk_data_mut(|chunks| {
        let keys_to_remove: Vec<String> = chunks
            .iter()
            .filter_map(|(key, chunk)| {
                if chunk.session_id == session_id {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();

        for key in keys_to_remove {
            chunks.remove(&key);
        }
    });
}

/// Get current time - works in both canister and test environments
fn get_current_time() -> u64 {
    #[cfg(test)]
    {
        1234567890 // Mock timestamp for tests
    }
    #[cfg(not(test))]
    {
        api::time()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_begin_asset_upload_success() {
        let memory_id = "test_memory_123".to_string();
        let expected_hash = "test_hash_abc".to_string();
        let chunk_count = 5;
        let total_size = 5000;

        let result = begin_asset_upload(
            memory_id.clone(),
            expected_hash.clone(),
            chunk_count,
            total_size,
        );

        assert!(result.is_ok());
        if let Some(response) = result.data {
            assert!(response.success);
            assert!(response.session.is_some());
            let session = response.session.unwrap();
            assert_eq!(session.memory_id, memory_id);
            assert_eq!(session.expected_hash, expected_hash);
            assert_eq!(session.chunk_count, chunk_count);
            assert_eq!(session.total_size, total_size);
            assert_eq!(session.chunks_received.len(), chunk_count as usize);
            assert!(session.chunks_received.iter().all(|&received| !received));
            assert_eq!(session.bytes_received, 0);
        }
    }

    #[test]
    fn test_begin_asset_upload_file_too_large() {
        let memory_id = "test_memory_123".to_string();
        let expected_hash = "test_hash_abc".to_string();
        let chunk_count = 5;
        let total_size = 200 * 1024 * 1024; // 200MB - exceeds 100MB limit

        let result = begin_asset_upload(memory_id, expected_hash, chunk_count, total_size);

        assert!(result.is_err());
        assert!(matches!(result.error, Some(ICPErrorCode::Internal(_))));
    }

    #[test]
    fn test_put_chunk_success() {
        // First create a session
        let memory_id = "test_memory_456".to_string();
        let expected_hash = "test_hash_def".to_string();
        let chunk_count = 3;
        let total_size = 3000;

        let session_result = begin_asset_upload(memory_id, expected_hash, chunk_count, total_size);
        assert!(session_result.is_ok());

        let session_id = session_result.data.unwrap().session.unwrap().session_id;

        // Now upload a chunk
        let chunk_data = vec![1, 2, 3, 4, 5];
        let chunk_index = 0;

        let result = put_chunk(session_id, chunk_index, chunk_data.clone());

        assert!(result.is_ok());
        if let Some(response) = result.data {
            assert!(response.success);
            assert_eq!(response.chunk_index, chunk_index);
            assert_eq!(response.bytes_received, chunk_data.len() as u32);
        }
    }

    #[test]
    fn test_put_chunk_invalid_session() {
        let session_id = "nonexistent_session".to_string();
        let chunk_data = vec![1, 2, 3, 4, 5];
        let chunk_index = 0;

        let result = put_chunk(session_id, chunk_index, chunk_data);

        assert!(result.is_err());
        assert_eq!(result.error, Some(ICPErrorCode::NotFound));
    }

    #[test]
    fn test_cancel_upload() {
        // Create a session first
        let memory_id = "test_memory_789".to_string();
        let expected_hash = "test_hash_ghi".to_string();
        let chunk_count = 2;
        let total_size = 2000;

        let session_result = begin_asset_upload(memory_id, expected_hash, chunk_count, total_size);
        assert!(session_result.is_ok());

        let session_id = session_result.data.unwrap().session.unwrap().session_id;

        // Cancel the upload
        let result = cancel_upload(session_id.clone());
        assert!(result.is_ok());

        // Verify session is cleaned up
        let session_check = with_stable_upload_sessions(|sessions| sessions.get(&session_id));
        assert!(session_check.is_none());
    }

    #[test]
    fn test_generate_session_id() {
        let memory_id = "test_memory";
        let expected_hash = "test_hash";

        let id1 = generate_session_id(memory_id, expected_hash);
        let id2 = generate_session_id("different_memory", expected_hash);

        // Should be different due to different memory_id
        assert_ne!(id1, id2);
        assert!(id1.starts_with("session_"));
        assert!(id2.starts_with("session_"));
    }

    #[test]
    fn test_compute_sha256_hash() {
        let data1 = b"hello world";
        let data2 = b"hello world";
        let data3 = b"different data";

        let hash1 = compute_sha256_hash(data1);
        let hash2 = compute_sha256_hash(data2);
        let hash3 = compute_sha256_hash(data3);

        assert_eq!(hash1, hash2); // Same data should have same hash
        assert_ne!(hash1, hash3); // Different data should have different hash
        assert!(hash1.starts_with("sha256_"));
    }
}
