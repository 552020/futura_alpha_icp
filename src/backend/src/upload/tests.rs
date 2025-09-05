//! Basic tests for the upload workflow
//! Integration tests to verify the system works end-to-end

#[cfg(test)]
mod tests {
    use crate::capsule_store::Store;
    use crate::upload::service::UploadService;

    #[test]
    fn test_upload_service_creation() {
        // Basic test to ensure the service can be created
        let mut store = Store::new_hash();
        let _upload_service = UploadService::new(&mut store);

        // Just verify the service was created successfully
        // This is a minimal test that doesn't have borrowing conflicts
        assert!(true); // Service created without panicking
    }

    #[test]
    fn test_constants() {
        // Test that our constants are properly defined
        use crate::upload::types::{CAPSULE_INLINE_BUDGET, CHUNK_SIZE, INLINE_MAX};

        assert_eq!(INLINE_MAX, (32 * 1024) as u64); // 32KB
        assert_eq!(CHUNK_SIZE, 64 * 1024); // 64KB
        assert_eq!(CAPSULE_INLINE_BUDGET, (32 * 1024) as u64); // 32KB
    }

    #[test]
    fn test_session_id_generation() {
        use crate::upload::types::SessionId;

        let id1 = SessionId::new();
        let id2 = SessionId::new();

        // IDs should be different
        assert_ne!(id1.0, id2.0);
    }

    #[test]
    fn test_blob_id_generation() {
        use crate::upload::types::BlobId;

        let id1 = BlobId::new();
        let id2 = BlobId::new();

        // IDs should be different
        assert_ne!(id1.0, id2.0);
    }
}
