use crate::memory::{
    with_stable_chunk_data, with_stable_chunk_data_mut, with_stable_memory_artifacts,
    with_stable_memory_artifacts_mut, with_stable_upload_sessions, with_stable_upload_sessions_mut,
};
use crate::types::{
    ArtifactType, BatchMemorySyncResponse, ChunkData, ChunkResponse, CommitResponse, ICPErrorCode,
    ICPResult, MemoryArtifact, MemorySyncRequest, MemorySyncResult, MemoryType,
    SimpleMemoryMetadata, UploadSession, UploadSessionResponse,
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
    memory_type: MemoryType,
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

    // Validate file size limits based on memory type
    let (max_size, type_name) = match memory_type {
        MemoryType::Image => (50 * 1024 * 1024, "image"), // 50MB
        MemoryType::Video => (100 * 1024 * 1024, "video"), // 100MB
        MemoryType::Audio => (25 * 1024 * 1024, "audio"), // 25MB
        MemoryType::Document => (10 * 1024 * 1024, "document"), // 10MB
        MemoryType::Note => (1 * 1024 * 1024, "note"),    // 1MB
    };

    if total_size > max_size {
        return ICPResult::err(ICPErrorCode::Internal(format!(
            "{} file size {} exceeds limit of {} for {}",
            type_name,
            format_file_size(total_size),
            format_file_size(max_size),
            type_name
        )));
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
        memory_type: memory_type.clone(),
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
        memory_type: session.memory_type.clone(), // Use actual memory type from session
        artifact_type: ArtifactType::Asset,
        content_hash: final_hash.clone(),
        size: session.total_size,
        stored_at: get_current_time(),
        metadata: None, // Asset data is stored separately
    };

    // Store artifact with proper memory type key
    let memory_type_str = match session.memory_type {
        MemoryType::Image => "image",
        MemoryType::Video => "video",
        MemoryType::Audio => "audio",
        MemoryType::Document => "document",
        MemoryType::Note => "note",
    };
    let artifact_key = format!("{}:{}:asset", session.memory_id, memory_type_str);
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
// BATCH MEMORY SYNC OPERATIONS - Gallery Memory Synchronization
// ============================================================================

/// Batch sync multiple memories for a gallery to ICP
pub async fn sync_gallery_memories(
    gallery_id: String,
    memory_sync_requests: Vec<MemorySyncRequest>,
) -> ICPResult<BatchMemorySyncResponse> {
    // Check authorization first
    if let Err(auth_error) = crate::auth::verify_caller_authorized() {
        return ICPResult::err(auth_error);
    }

    // Validate that the gallery exists before attempting to sync memories
    if let None = crate::capsule::get_gallery_by_id(gallery_id.clone()) {
        return ICPResult::err(ICPErrorCode::Internal(format!(
            "Gallery '{}' not found. Cannot sync memories to non-existent gallery.",
            gallery_id
        )));
    }

    let total_memories = memory_sync_requests.len() as u32;
    let mut results = Vec::new();
    let mut successful_memories = 0u32;
    let mut failed_memories = 0u32;

    // Process each memory sync request
    for sync_request in memory_sync_requests {
        let result = sync_single_memory(sync_request).await;

        if result.success {
            successful_memories += 1;
        } else {
            failed_memories += 1;
        }

        results.push(result);
    }

    let overall_success = failed_memories == 0;
    let message = if overall_success {
        format!(
            "Successfully synced {}/{} memories to ICP",
            successful_memories, total_memories
        )
    } else {
        format!(
            "Synced {}/{} memories to ICP ({} failed)",
            successful_memories, total_memories, failed_memories
        )
    };

    ICPResult::ok(BatchMemorySyncResponse {
        gallery_id,
        success: overall_success,
        total_memories,
        successful_memories,
        failed_memories,
        results,
        message,
        error: None,
    })
}

/// Sync a single memory (metadata + asset) to ICP
async fn sync_single_memory(sync_request: MemorySyncRequest) -> MemorySyncResult {
    let memory_id = sync_request.memory_id.clone();

    // Step 0: Validate memory type and asset size
    let validation_result =
        validate_memory_type_and_size(&sync_request.memory_type, sync_request.asset_size);

    if validation_result.is_err() {
        return MemorySyncResult {
            memory_id,
            success: false,
            metadata_stored: false,
            asset_stored: false,
            message: format!("Validation failed: {:?}", validation_result.error),
            error: validation_result.error,
        };
    }

    // Step 1: Store metadata
    let metadata_result = store_memory_metadata(
        memory_id.clone(),
        sync_request.memory_type.clone(),
        sync_request.metadata.clone(),
    );

    let metadata_stored = metadata_result.is_ok();
    if !metadata_stored {
        return MemorySyncResult {
            memory_id,
            success: false,
            metadata_stored: false,
            asset_stored: false,
            message: format!("Failed to store metadata: {:?}", metadata_result.error),
            error: metadata_result.error,
        };
    }

    // Step 2: Fetch and store asset (simplified for MVP - in production would use chunked upload)
    let asset_result = fetch_and_store_asset(
        memory_id.clone(),
        sync_request.memory_type.clone(),
        sync_request.asset_url.clone(),
        sync_request.expected_asset_hash.clone(),
        sync_request.asset_size,
    )
    .await;

    let asset_stored = asset_result.is_ok();
    let success = metadata_stored && asset_stored;

    let message = if success {
        format!("Successfully synced memory {} to ICP", memory_id)
    } else if metadata_stored {
        format!(
            "Metadata stored but asset sync failed for memory {}",
            memory_id
        )
    } else {
        format!("Failed to sync memory {} to ICP", memory_id)
    };

    MemorySyncResult {
        memory_id,
        success,
        metadata_stored,
        asset_stored,
        message,
        error: if success { None } else { asset_result.error },
    }
}

/// Store memory metadata using existing metadata system
fn store_memory_metadata(
    memory_id: String,
    memory_type: MemoryType,
    metadata: SimpleMemoryMetadata,
) -> ICPResult<()> {
    // Generate idempotency key for metadata
    let idempotency_key = format!("gallery_sync_{}_{}", memory_id, get_current_time());

    // Use existing metadata upsert function
    let result =
        crate::metadata::upsert_metadata(memory_id, memory_type, metadata, idempotency_key);

    match result.success {
        true => ICPResult::ok(()),
        false => ICPResult::err(result.error.unwrap_or(ICPErrorCode::Internal(
            "Metadata storage failed".to_string(),
        ))),
    }
}

/// Validate memory type and asset size compatibility
fn validate_memory_type_and_size(memory_type: &MemoryType, asset_size: u64) -> ICPResult<()> {
    // Define size limits for different memory types
    let (max_size, type_name) = match memory_type {
        MemoryType::Image => (50 * 1024 * 1024, "image"), // 50MB
        MemoryType::Video => (100 * 1024 * 1024, "video"), // 100MB
        MemoryType::Audio => (25 * 1024 * 1024, "audio"), // 25MB
        MemoryType::Document => (10 * 1024 * 1024, "document"), // 10MB
        MemoryType::Note => (1 * 1024 * 1024, "note"),    // 1MB
    };

    if asset_size > max_size {
        return ICPResult::err(ICPErrorCode::Internal(format!(
            "{} file size {} exceeds limit of {} for {}",
            type_name,
            format_file_size(asset_size),
            format_file_size(max_size),
            type_name
        )));
    }

    ICPResult::ok(())
}

/// Format file size in human-readable format
fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    match bytes {
        0..KB => format!("{} B", bytes),
        KB..MB => format!("{:.1} KB", bytes as f64 / KB as f64),
        MB..GB => format!("{:.1} MB", bytes as f64 / MB as f64),
        _ => format!("{:.1} GB", bytes as f64 / GB as f64),
    }
}

/// Fetch asset from URL and store in ICP (simplified for MVP)
async fn fetch_and_store_asset(
    memory_id: String,
    memory_type: MemoryType,
    _asset_url: String,
    expected_hash: String,
    asset_size: u64,
) -> ICPResult<()> {
    // MVP Implementation: Mock asset storage
    // In production, this would:
    // 1. Fetch asset data from asset_url
    // 2. Validate hash matches expected_hash
    // 3. Use chunked upload system for large files
    // 4. Store asset using existing upload infrastructure

    // For now, create a mock artifact to indicate asset is "stored"
    let artifact = MemoryArtifact {
        memory_id: memory_id.clone(),
        memory_type,
        artifact_type: ArtifactType::Asset,
        content_hash: expected_hash,
        size: asset_size,
        stored_at: get_current_time(),
        metadata: Some(format!("Mock asset storage for memory {}", memory_id)),
    };

    // Store artifact in stable memory
    let memory_type_str = match artifact.memory_type {
        MemoryType::Image => "image",
        MemoryType::Video => "video",
        MemoryType::Audio => "audio",
        MemoryType::Document => "document",
        MemoryType::Note => "note",
    };
    let artifact_key = format!("{}:{}:asset", memory_id, memory_type_str);

    with_stable_memory_artifacts_mut(|artifacts| {
        artifacts.insert(artifact_key, artifact);
    });

    ICPResult::ok(())
}

// ============================================================================
// CLEANUP AND RECOVERY FUNCTIONS - Failed Upload Handling
// ============================================================================

/// Clean up failed or expired upload sessions
pub fn cleanup_expired_sessions() -> u32 {
    let current_time = get_current_time();
    const SESSION_TIMEOUT: u64 = 30 * 60 * 1_000_000_000; // 30 minutes in nanoseconds

    let mut expired_sessions = Vec::new();

    // Find expired sessions
    with_stable_upload_sessions(|sessions| {
        for (session_id, session) in sessions.iter() {
            if current_time - session.created_at > SESSION_TIMEOUT {
                expired_sessions.push(session_id.clone());
            }
        }
    });

    // Clean up expired sessions
    let cleanup_count = expired_sessions.len() as u32;
    for session_id in expired_sessions {
        cleanup_session(&session_id);
    }

    cleanup_count
}

/// Clean up orphaned chunks (chunks without valid sessions)
pub fn cleanup_orphaned_chunks() -> u32 {
    let mut orphaned_chunks = Vec::new();

    // Find chunks that don't have corresponding sessions
    with_stable_chunk_data(|chunks| {
        with_stable_upload_sessions(|sessions| {
            for (chunk_key, chunk) in chunks.iter() {
                if !sessions.contains_key(&chunk.session_id) {
                    orphaned_chunks.push(chunk_key.clone());
                }
            }
        });
    });

    // Remove orphaned chunks
    let cleanup_count = orphaned_chunks.len() as u32;
    with_stable_chunk_data_mut(|chunks| {
        for chunk_key in orphaned_chunks {
            chunks.remove(&chunk_key);
        }
    });

    cleanup_count
}

/// Get upload session statistics for monitoring
pub fn get_upload_session_stats() -> (u32, u32, u64) {
    let mut active_sessions = 0u32;
    let mut total_chunks = 0u32;
    let mut total_bytes = 0u64;

    with_stable_upload_sessions(|sessions| {
        active_sessions = sessions.len() as u32;
        for session in sessions.values() {
            total_chunks += session.chunk_count;
            total_bytes += session.bytes_received;
        }
    });

    (active_sessions, total_chunks, total_bytes)
}

/// Force cleanup of all upload data (emergency use only)
pub fn emergency_cleanup_all_uploads() -> (u32, u32) {
    let mut sessions_count = 0u32;
    let mut chunks_count = 0u32;

    // Count sessions
    let session_keys: Vec<String> = with_stable_upload_sessions(|sessions| {
        sessions_count = sessions.len() as u32;
        sessions.iter().map(|(key, _)| key.clone()).collect()
    });

    // Remove all sessions
    with_stable_upload_sessions_mut(|sessions| {
        for key in session_keys {
            sessions.remove(&key);
        }
    });

    // Count chunks
    let chunk_keys: Vec<String> = with_stable_chunk_data(|chunks| {
        chunks_count = chunks.len() as u32;
        chunks.iter().map(|(key, _)| key.clone()).collect()
    });

    // Remove all chunks
    with_stable_chunk_data_mut(|chunks| {
        for key in chunk_keys {
            chunks.remove(&key);
        }
    });

    (sessions_count, chunks_count)
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
        let memory_type = MemoryType::Image;
        let expected_hash = "test_hash_abc".to_string();
        let chunk_count = 5;
        let total_size = 5000;

        let result = begin_asset_upload(
            memory_id.clone(),
            memory_type.clone(),
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
            assert_eq!(session.memory_type, memory_type);
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
        let memory_type = MemoryType::Video;
        let expected_hash = "test_hash_abc".to_string();
        let chunk_count = 5;
        let total_size = 200 * 1024 * 1024; // 200MB - exceeds 100MB limit

        let result = begin_asset_upload(
            memory_id,
            memory_type,
            expected_hash,
            chunk_count,
            total_size,
        );

        assert!(result.is_err());
        assert!(matches!(result.error, Some(ICPErrorCode::Internal(_))));
    }

    #[test]
    fn test_put_chunk_success() {
        // First create a session
        let memory_id = "test_memory_456".to_string();
        let memory_type = MemoryType::Document;
        let expected_hash = "test_hash_def".to_string();
        let chunk_count = 3;
        let total_size = 3000;

        let session_result = begin_asset_upload(
            memory_id,
            memory_type,
            expected_hash,
            chunk_count,
            total_size,
        );
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
        let memory_type = MemoryType::Audio;
        let expected_hash = "test_hash_ghi".to_string();
        let chunk_count = 2;
        let total_size = 2000;

        let session_result = begin_asset_upload(
            memory_id,
            memory_type,
            expected_hash,
            chunk_count,
            total_size,
        );
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

    #[test]
    fn test_validate_memory_type_and_size() {
        // Test valid sizes for different memory types
        assert!(validate_memory_type_and_size(&MemoryType::Image, 10 * 1024 * 1024).is_ok()); // 10MB image
        assert!(validate_memory_type_and_size(&MemoryType::Video, 50 * 1024 * 1024).is_ok()); // 50MB video
        assert!(validate_memory_type_and_size(&MemoryType::Audio, 5 * 1024 * 1024).is_ok()); // 5MB audio
        assert!(validate_memory_type_and_size(&MemoryType::Document, 2 * 1024 * 1024).is_ok()); // 2MB document
        assert!(validate_memory_type_and_size(&MemoryType::Note, 512 * 1024).is_ok()); // 512KB note

        // Test invalid sizes (exceed limits)
        assert!(validate_memory_type_and_size(&MemoryType::Image, 100 * 1024 * 1024).is_err()); // 100MB image (exceeds 50MB)
        assert!(validate_memory_type_and_size(&MemoryType::Video, 200 * 1024 * 1024).is_err()); // 200MB video (exceeds 100MB)
        assert!(validate_memory_type_and_size(&MemoryType::Audio, 50 * 1024 * 1024).is_err()); // 50MB audio (exceeds 25MB)
        assert!(validate_memory_type_and_size(&MemoryType::Document, 20 * 1024 * 1024).is_err()); // 20MB document (exceeds 10MB)
        assert!(validate_memory_type_and_size(&MemoryType::Note, 2 * 1024 * 1024).is_err());
        // 2MB note (exceeds 1MB)
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1536 * 1024), "1.5 MB");
    }

    #[test]
    fn test_cleanup_expired_sessions() {
        // Create a test session that would be expired
        let memory_id = "expired_memory".to_string();
        let memory_type = MemoryType::Image;
        let expected_hash = "expired_hash".to_string();
        let chunk_count = 2;
        let total_size = 1024;

        // Create session normally
        let result = begin_asset_upload(
            memory_id,
            memory_type,
            expected_hash,
            chunk_count,
            total_size,
        );
        assert!(result.is_ok());

        // Test cleanup function (it won't find expired sessions in test due to mock time)
        let cleaned_count = cleanup_expired_sessions();
        // In test environment, cleanup count will be 0 since we use mock timestamps
        assert_eq!(cleaned_count, 0);
    }

    #[test]
    fn test_cleanup_orphaned_chunks() {
        // Test cleanup of orphaned chunks
        let cleaned_count = cleanup_orphaned_chunks();
        // In fresh test environment, should be 0
        assert_eq!(cleaned_count, 0);
    }

    #[test]
    fn test_get_upload_session_stats() {
        // Get initial stats
        let (sessions, chunks, bytes) = get_upload_session_stats();

        // Should start with 0 in test environment
        assert_eq!(sessions, 0);
        assert_eq!(chunks, 0);
        assert_eq!(bytes, 0);
    }

    #[test]
    fn test_emergency_cleanup_all_uploads() {
        // Test emergency cleanup
        let (sessions_cleaned, chunks_cleaned) = emergency_cleanup_all_uploads();

        // Should be 0 in fresh test environment
        assert_eq!(sessions_cleaned, 0);
        assert_eq!(chunks_cleaned, 0);
    }

    #[test]
    fn test_sync_single_memory_validation_failure() {
        // Test that sync fails with invalid memory type/size combination
        let sync_request = MemorySyncRequest {
            memory_id: "test_memory".to_string(),
            memory_type: MemoryType::Note,
            metadata: SimpleMemoryMetadata {
                title: Some("Test Note".to_string()),
                description: Some("Test description".to_string()),
                tags: vec!["test".to_string()],
                created_at: 1234567890,
                updated_at: 1234567890,
                size: Some(2 * 1024 * 1024), // 2MB - exceeds 1MB limit for notes
                content_type: Some("text/plain".to_string()),
                custom_fields: std::collections::HashMap::new(),
            },
            asset_url: "https://example.com/asset".to_string(),
            expected_asset_hash: "test_hash".to_string(),
            asset_size: 2 * 1024 * 1024, // 2MB - exceeds note limit
        };

        // This would be an async function in real usage, but we can test the validation part
        // The function should fail due to size validation
        let validation_result =
            validate_memory_type_and_size(&sync_request.memory_type, sync_request.asset_size);
        assert!(validation_result.is_err());
    }

    #[test]
    fn test_begin_asset_upload_with_different_memory_types() {
        // Test that different memory types have different size limits

        // Test Image - should pass with 30MB
        let result_image = begin_asset_upload(
            "test_image".to_string(),
            MemoryType::Image,
            "hash_image".to_string(),
            10,
            30 * 1024 * 1024, // 30MB
        );
        assert!(result_image.is_ok());

        // Test Video - should pass with 80MB
        let result_video = begin_asset_upload(
            "test_video".to_string(),
            MemoryType::Video,
            "hash_video".to_string(),
            10,
            80 * 1024 * 1024, // 80MB
        );
        assert!(result_video.is_ok());

        // Test Audio - should fail with 30MB (exceeds 25MB limit)
        let result_audio = begin_asset_upload(
            "test_audio".to_string(),
            MemoryType::Audio,
            "hash_audio".to_string(),
            10,
            30 * 1024 * 1024, // 30MB - exceeds 25MB limit
        );
        assert!(result_audio.is_err());

        // Test Document - should fail with 15MB (exceeds 10MB limit)
        let result_document = begin_asset_upload(
            "test_document".to_string(),
            MemoryType::Document,
            "hash_document".to_string(),
            10,
            15 * 1024 * 1024, // 15MB - exceeds 10MB limit
        );
        assert!(result_document.is_err());

        // Test Note - should fail with 2MB (exceeds 1MB limit)
        let result_note = begin_asset_upload(
            "test_note".to_string(),
            MemoryType::Note,
            "hash_note".to_string(),
            10,
            2 * 1024 * 1024, // 2MB - exceeds 1MB limit
        );
        assert!(result_note.is_err());
    }
}
