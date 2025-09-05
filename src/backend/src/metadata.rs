use crate::types::{
    Error, MemoryPresenceResult, MemoryType, MetadataResponse, Result, SimpleMemoryMetadata,
};

#[cfg(test)]
use std::collections::HashMap;

// ic_cdk::api import removed - no longer needed

// ============================================================================
// MEMORY METADATA OPERATIONS - ICP Canister Endpoints
// ============================================================================

/// Store memory metadata on ICP with idempotency support
pub fn upsert_metadata(
    memory_id: String,
    memory_type: MemoryType,
    _metadata: SimpleMemoryMetadata,
    _idempotency_key: String,
) -> Result<MetadataResponse> {
    // Check authorization first
    crate::auth::verify_caller_authorized()?;

    // Validate memory type
    if !is_valid_memory_type(&memory_type) {
        return Err(Error::Internal("Invalid memory type".to_string()));
    }

    // TODO: Implement metadata storage using capsule system instead of artifacts
    // For now, just return success - metadata should be stored in capsules
    Ok(MetadataResponse::ok(
        memory_id,
        "Metadata operation completed (artifacts system removed)".to_string(),
    ))
}

/// Check presence for multiple memories on ICP (consolidated from get_memory_presence_icp and get_memory_list_presence_icp)
pub fn memories_ping(memory_ids: Vec<String>) -> Result<Vec<MemoryPresenceResult>> {
    // TODO: Implement memory presence checking using capsule system instead of artifacts
    // For now, return false for all memories since artifacts system is removed
    let results: Vec<MemoryPresenceResult> = memory_ids
        .iter()
        .map(|memory_id| MemoryPresenceResult {
            memory_id: memory_id.clone(),
            metadata_present: false, // Artifacts system removed
            asset_present: false,    // Artifacts system removed
        })
        .collect();

    Ok(results)
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

// Helper functions removed - no longer needed without artifacts system

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

    // compute_content_hash test removed - function deleted with artifacts system

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
        if let Ok(response) = result {
            assert!(response.success);
            assert_eq!(response.memory_id, Some(memory_id));
        }
    }

    #[test]
    fn test_memories_ping_not_found() {
        let memory_id = "nonexistent_memory".to_string();
        let result = memories_ping(vec![memory_id]);

        assert!(result.is_ok());
        if let Ok(response) = result {
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
        if let Ok(response) = result {
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
        if let Ok(response) = result {
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
        if let Ok(response) = result {
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
    if let Ok(response) = presence_result {
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
    if let Ok(response) = idempotent_result {
        assert!(response.success);
        assert!(response.message.contains("same idempotency key"));
    }
}
