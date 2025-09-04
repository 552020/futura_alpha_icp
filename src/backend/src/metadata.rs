use crate::memory::{with_stable_memory_artifacts, with_stable_memory_artifacts_mut};
use crate::types::{
    ArtifactType, ICPErrorCode, ICPResult, MemoryArtifact, MemoryPresenceResult, MemoryType, MetadataResponse,
    SimpleMemoryMetadata,
};

#[cfg(test)]
use std::collections::HashMap;

#[cfg(not(test))]
use ic_cdk::api;

// ============================================================================
// MEMORY METADATA OPERATIONS - ICP Canister Endpoints
// ============================================================================

/// Store memory metadata on ICP with idempotency support
pub fn upsert_metadata(
    memory_id: String,
    memory_type: MemoryType,
    metadata: SimpleMemoryMetadata,
    idempotency_key: String,
) -> ICPResult<MetadataResponse> {
    // Check authorization first
    if let Err(auth_error) = crate::auth::verify_caller_authorized() {
        return ICPResult::err(auth_error);
    }

    // Validate memory type
    if !is_valid_memory_type(&memory_type) {
        return ICPResult::err(ICPErrorCode::Internal("Invalid memory type".to_string()));
    }

    // Create artifact key for stable storage
    let artifact_key = format!(
        "{}:{}:{}",
        memory_id,
        format!("{memory_type:?}").to_lowercase(),
        "metadata"
    );

    // Check if already exists with same idempotency key
    let existing = with_stable_memory_artifacts(|artifacts| artifacts.get(&artifact_key));

    if let Some(existing_artifact) = existing {
        // Check if this is the same operation (same idempotency key)
        if let Some(existing_metadata) = &existing_artifact.metadata {
            if existing_metadata.contains(&idempotency_key) {
                // Same operation, return success without re-writing
                return ICPResult::ok(MetadataResponse::ok(
                    memory_id,
                    "Metadata already exists with same idempotency key".to_string(),
                ));
            }
        }
    }

    // Serialize metadata to JSON
    let metadata_json = match serde_json::to_string(&metadata) {
        Ok(json) => json,
        Err(_) => {
            return ICPResult::err(ICPErrorCode::Internal(
                "Failed to serialize metadata".to_string(),
            ))
        }
    };

    // Create memory artifact
    let artifact = MemoryArtifact {
        memory_id: memory_id.clone(),
        memory_type,
        artifact_type: ArtifactType::Metadata,
        content_hash: compute_content_hash(&metadata_json),
        size: metadata_json.len() as u64,
        stored_at: get_current_time(),
        metadata: Some(format!("{idempotency_key}:{metadata_json}")),
    };

    // Store in stable memory
    with_stable_memory_artifacts_mut(|artifacts| {
        artifacts.insert(artifact_key, artifact);
    });

    ICPResult::ok(MetadataResponse::ok(
        memory_id,
        "Metadata stored successfully".to_string(),
    ))
}

/// Check presence for multiple memories on ICP (consolidated from get_memory_presence_icp and get_memory_list_presence_icp)
pub fn memories_ping(memory_ids: Vec<String>) -> ICPResult<Vec<MemoryPresenceResult>> {
    // Check presence for each memory
    let results: Vec<MemoryPresenceResult> = memory_ids
        .iter()
        .map(|memory_id| {
            let (metadata_present, asset_present) = with_stable_memory_artifacts(|artifacts| {
                let metadata_exists = artifacts
                    .iter()
                    .any(|(key, _)| key.contains(memory_id) && key.contains("metadata"));
                let asset_exists = artifacts
                    .iter()
                    .any(|(key, _)| key.contains(memory_id) && key.contains("asset"));

                (metadata_exists, asset_exists)
            });

            MemoryPresenceResult {
                memory_id: memory_id.clone(),
                metadata_present,
                asset_present,
            }
        })
        .collect();

    ICPResult::ok(results)
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Validate memory type
fn is_valid_memory_type(memory_type: &MemoryType) -> bool {
    matches!(
        memory_type,
        MemoryType::Image
            | MemoryType::Video
            | MemoryType::Audio
            | MemoryType::Document
            | MemoryType::Note
    )
}

/// Compute simple content hash for metadata
fn compute_content_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("hash_{:x}", hasher.finish())
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
    use std::collections::HashMap;

    #[test]
    fn test_is_valid_memory_type() {
        assert!(is_valid_memory_type(&MemoryType::Image));
        assert!(is_valid_memory_type(&MemoryType::Video));
        assert!(is_valid_memory_type(&MemoryType::Audio));
        assert!(is_valid_memory_type(&MemoryType::Document));
        assert!(is_valid_memory_type(&MemoryType::Note));
    }

    #[test]
    fn test_compute_content_hash() {
        let content1 = "test content";
        let content2 = "test content";
        let content3 = "different content";

        let hash1 = compute_content_hash(content1);
        let hash2 = compute_content_hash(content2);
        let hash3 = compute_content_hash(content3);

        assert_eq!(hash1, hash2); // Same content should have same hash
        assert_ne!(hash1, hash3); // Different content should have different hash
        assert!(hash1.starts_with("hash_")); // Should have correct prefix
    }

    #[test]
    fn test_upsert_metadata_basic() {
        let memory_id = "test_memory_123".to_string();
        let memory_type = MemoryType::Image;
        let mut custom_fields = HashMap::new();
        custom_fields.insert("author".to_string(), "test_user".to_string());

        let metadata = SimpleMemoryMetadata {
            title: Some("Test Memory".to_string()),
            description: Some("A test memory".to_string()),
            tags: vec!["test".to_string(), "memory".to_string()],
            created_at: 1234567890,
            updated_at: 1234567890,
            size: Some(1024),
            content_type: Some("image/jpeg".to_string()),
            custom_fields,
        };
        let idempotency_key = "test_key_123".to_string();

        let result = upsert_metadata(memory_id.clone(), memory_type, metadata, idempotency_key);

        assert!(result.is_ok());
        assert!(result.data.is_some());
        if let Some(response) = result.data {
            assert!(response.success);
            assert_eq!(response.memory_id, Some(memory_id));
        }
    }

    #[test]
    fn test_memories_ping_not_found() {
        let memory_id = "nonexistent_memory".to_string();
        let result = memories_ping(vec![memory_id]);

        assert!(result.is_ok());
        if let Some(response) = result.data {
            assert_eq!(response.len(), 1);
            assert!(!response[0].metadata_present);
            assert!(!response[0].asset_present);
        }
    }

    #[test]
    fn test_memories_ping_empty() {
        let memory_ids = vec![];
        let result = memories_ping(memory_ids);

        assert!(result.is_ok());
        if let Some(response) = result.data {
            assert_eq!(response.len(), 0);
        }
    }

    #[test]
    fn test_memories_ping_multiple() {
        let memory_ids = vec![
            "mem1".to_string(),
            "mem2".to_string(),
            "mem3".to_string(),
            "mem4".to_string(),
            "mem5".to_string(),
        ];

        let result = memories_ping(memory_ids);
        assert!(result.is_ok());
        if let Some(response) = result.data {
            assert_eq!(response.len(), 5);
            // All memories should not be present (nonexistent)
            for memory_presence in response {
                assert!(!memory_presence.metadata_present);
                assert!(!memory_presence.asset_present);
            }
        }
    }

    #[test]
    fn test_memories_ping_single() {
        let memory_ids = vec!["mem1".to_string()];

        let result = memories_ping(memory_ids);
        assert!(result.is_ok());
        if let Some(response) = result.data {
            assert_eq!(response.len(), 1);
            assert!(!response[0].metadata_present);
            assert!(!response[0].asset_present);
        }
    }
}
#[test]
fn test_integration_upsert_and_query() {
    // Test the full flow: upsert metadata, then query it
    let memory_id = "integration_test_memory".to_string();
    let memory_type = MemoryType::Image;
    let mut custom_fields = HashMap::new();
    custom_fields.insert("camera".to_string(), "Canon EOS".to_string());

    let metadata = SimpleMemoryMetadata {
        title: Some("Integration Test Memory".to_string()),
        description: Some("A test memory for integration testing".to_string()),
        tags: vec!["integration".to_string(), "test".to_string()],
        created_at: 1234567890,
        updated_at: 1234567890,
        size: Some(2048),
        content_type: Some("image/png".to_string()),
        custom_fields: custom_fields.clone(),
    };
    let idempotency_key = "integration_test_key".to_string();

    // 1. Upsert metadata
    let upsert_result = upsert_metadata(
        memory_id.clone(),
        memory_type,
        metadata,
        idempotency_key.clone(),
    );
    assert!(upsert_result.is_ok());

    // 2. Query memory presence
    let presence_result = memories_ping(vec![memory_id.clone()]);
    assert!(presence_result.is_ok());
    if let Some(response) = presence_result.data {
        assert_eq!(response.len(), 1);
        assert!(response[0].metadata_present);
        assert!(!response[0].asset_present); // Asset not stored yet
    }

    // 4. Test idempotency - same operation should succeed without error
    let idempotent_result = upsert_metadata(
        memory_id.clone(),
        MemoryType::Image,
        SimpleMemoryMetadata {
            title: Some("Integration Test Memory".to_string()),
            description: Some("A test memory for integration testing".to_string()),
            tags: vec!["integration".to_string(), "test".to_string()],
            created_at: 1234567890,
            updated_at: 1234567890,
            size: Some(2048),
            content_type: Some("image/png".to_string()),
            custom_fields,
        },
        idempotency_key,
    );
    assert!(idempotent_result.is_ok());
    if let Some(response) = idempotent_result.data {
        assert!(response.success);
        assert!(response.message.contains("same idempotency key"));
    }
}
