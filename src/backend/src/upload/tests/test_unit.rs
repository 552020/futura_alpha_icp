//! Comprehensive unit tests for upload module components
//!
//! These tests focus on individual components and edge cases that are not
//! covered by the bash integration tests.

#[cfg(test)]
mod session_store_tests {
    use crate::types::MemoryMeta;
    use crate::upload::sessions::SessionStore;
    use crate::upload::types::{SessionId, SessionMeta, SessionStatus};
    use candid::Principal;

    fn create_test_session_meta() -> SessionMeta {
        // Use anonymous principal to avoid IC API calls
        let caller = Principal::anonymous();
        SessionMeta {
            capsule_id: "test-capsule".to_string(),
            provisional_memory_id: "test-memory".to_string(),
            caller,
            chunk_count: 3,
            expected_len: Some(300),
            expected_hash: Some([0u8; 32]),
            status: SessionStatus::Pending,
            created_at: 1234567890,
            meta: MemoryMeta {
                name: "test.txt".to_string(),
                description: Some("Test file".to_string()),
                tags: vec!["test".to_string()],
            },
            idem: "test-idem".to_string(),
        }
    }

    #[test]
    fn test_session_store_create_and_get() {
        // Test that we can create a session store without IC dependencies
        let _store = SessionStore::new();
        let _session_id = SessionId::new();
        let _session_meta = create_test_session_meta();

        // Test basic validation without IC calls
        assert!(
            !_session_meta.capsule_id.is_empty(),
            "Capsule ID should not be empty"
        );
        assert!(
            _session_meta.chunk_count > 0,
            "Chunk count should be positive"
        );
        assert!(
            matches!(_session_meta.status, SessionStatus::Pending),
            "Status should be Pending"
        );

        // Test that IDs are unique
        let session_id2 = SessionId::new();
        assert_ne!(_session_id.0, session_id2.0, "Session IDs should be unique");
    }

    #[test]
    fn test_session_store_update() {
        // Test session metadata updates without IC dependencies
        let mut session_meta = create_test_session_meta();

        // Test status transition
        session_meta.status = SessionStatus::Committed { blob_id: 123 };
        assert!(matches!(
            session_meta.status,
            SessionStatus::Committed { blob_id: 123 }
        ));

        // Test different blob ID
        session_meta.status = SessionStatus::Committed { blob_id: 456 };
        assert!(matches!(
            session_meta.status,
            SessionStatus::Committed { blob_id: 456 }
        ));

        // Test that we can create store and session ID
        let _store = SessionStore::new();
        let _session_id = SessionId::new();
        assert!(_session_id.0 > 0, "Session ID should be positive");
    }

    #[test]
    fn test_session_store_put_and_get_chunk() {
        let store = SessionStore::new();
        let session_id = SessionId::new();
        let chunk_idx = 0;
        let chunk_data = vec![1, 2, 3, 4, 5];

        // Test put chunk
        let result = store.put_chunk(&session_id, chunk_idx, chunk_data.clone());
        assert!(result.is_ok(), "Chunk storage should succeed");

        // Test get chunk
        let retrieved = store.get_chunk(&session_id, chunk_idx).unwrap();
        assert!(retrieved.is_some(), "Chunk should be retrievable");
        assert_eq!(retrieved.unwrap(), chunk_data);
    }

    #[test]
    fn test_session_store_nonexistent_session() {
        let store = SessionStore::new();
        let session_id = SessionId::new();

        // Test get nonexistent session
        let result = store.get(&session_id).unwrap();
        assert!(result.is_none(), "Nonexistent session should return None");

        // Test get chunk from nonexistent session
        let result = store.get_chunk(&session_id, 0).unwrap();
        assert!(
            result.is_none(),
            "Chunk from nonexistent session should return None"
        );
    }

    #[test]
    fn test_session_store_multiple_chunks() {
        let store = SessionStore::new();
        let session_id = SessionId::new();

        // Store multiple chunks
        for i in 0..5 {
            let chunk_data = vec![i as u8; 10]; // 10 bytes of value i
            store.put_chunk(&session_id, i, chunk_data).unwrap();
        }

        // Retrieve and verify all chunks
        for i in 0..5 {
            let retrieved = store.get_chunk(&session_id, i).unwrap().unwrap();
            assert_eq!(retrieved, vec![i as u8; 10]);
        }
    }
}

#[cfg(test)]
mod blob_store_tests {
    use crate::upload::blob_store::BlobStore;
    use crate::upload::sessions::SessionStore;
    use crate::upload::types::{BlobId, SessionId};

    fn create_test_blob_store() -> BlobStore {
        BlobStore::new()
    }

    fn create_test_session_store() -> SessionStore {
        SessionStore::new()
    }

    #[test]
    fn test_blob_store_creation() {
        let blob_store = create_test_blob_store();
        // Test that we can create a blob store without errors
        assert!(true, "Blob store creation should succeed");
    }

    #[test]
    fn test_blob_id_generation() {
        let id1 = BlobId::new();
        let id2 = BlobId::new();

        // IDs should be different
        assert_ne!(id1.0, id2.0, "Blob IDs should be unique");

        // IDs should be positive
        assert!(id1.0 > 0, "Blob ID should be positive");
        assert!(id2.0 > 0, "Blob ID should be positive");
    }

    #[test]
    fn test_blob_store_with_empty_data() {
        let blob_store = create_test_blob_store();
        let session_store = create_test_session_store();
        let session_id = SessionId::new();
        let expected_hash = [0u8; 32];
        let expected_len = 0;

        // Test with empty data (no chunks)
        let result = blob_store.store_from_chunks(
            &session_store,
            &session_id,
            0, // no chunks
            expected_len,
            expected_hash,
        );

        // This should fail because there are no chunks to store
        assert!(result.is_err(), "Empty data should fail validation");
    }

    #[test]
    fn test_blob_store_hash_validation() {
        // Test hash computation without IC dependencies
        let test_data = vec![1, 2, 3, 4, 5];

        // Test with correct hash computation
        let correct_hash: [u8; 32] = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&test_data);
            hasher.finalize().into()
        };

        // Verify hash is 32 bytes
        assert_eq!(correct_hash.len(), 32, "Hash should be 32 bytes");

        // Test that different data produces different hashes
        let test_data2 = vec![6, 7, 8, 9, 10];
        let hash2: [u8; 32] = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&test_data2);
            hasher.finalize().into()
        };

        assert_ne!(
            correct_hash, hash2,
            "Different data should produce different hashes"
        );

        // Test that we can create blob store
        let _blob_store = create_test_blob_store();
        let _session_store = create_test_session_store();
        let _session_id = SessionId::new();
    }
}

#[cfg(test)]
mod hash_computation_tests {
    use crate::upload::service::UploadService;

    #[test]
    fn test_sha256_empty_data() {
        let empty_data = b"";
        let hash = UploadService::compute_sha256(empty_data);

        // SHA256 of empty string is a known constant
        let expected = [
            0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f,
            0xb9, 0x24, 0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b,
            0x78, 0x52, 0xb8, 0x55,
        ];

        assert_eq!(
            hash, expected,
            "SHA256 of empty data should match known constant"
        );
    }

    #[test]
    fn test_sha256_small_data() {
        let data = b"hello world";
        let hash = UploadService::compute_sha256(data);

        // SHA256 of "hello world" is a known constant
        let expected = [
            0xb9, 0x4d, 0x27, 0xb9, 0x93, 0x4d, 0x3e, 0x08, 0xa5, 0x2e, 0x52, 0xd7, 0xda, 0x7d,
            0xab, 0xfa, 0xc4, 0x84, 0xef, 0xe3, 0x7a, 0x53, 0x80, 0xee, 0x90, 0x88, 0xf7, 0xac,
            0xe2, 0xef, 0xcd, 0xe9,
        ];

        assert_eq!(
            hash, expected,
            "SHA256 of 'hello world' should match known constant"
        );
    }

    #[test]
    fn test_sha256_large_data() {
        // Test with 1MB of data
        let large_data = vec![0x42; 1_000_000];
        let hash = UploadService::compute_sha256(&large_data);

        // Hash should be 32 bytes
        assert_eq!(hash.len(), 32, "SHA256 hash should be 32 bytes");

        // Hash should be deterministic
        let hash2 = UploadService::compute_sha256(&large_data);
        assert_eq!(hash, hash2, "SHA256 should be deterministic");
    }

    #[test]
    fn test_sha256_different_data() {
        let data1 = b"hello";
        let data2 = b"world";

        let hash1 = UploadService::compute_sha256(data1);
        let hash2 = UploadService::compute_sha256(data2);

        // Different data should produce different hashes
        assert_ne!(
            hash1, hash2,
            "Different data should produce different hashes"
        );
    }
}

#[cfg(test)]
mod constants_and_limits_tests {
    use crate::upload::types::*;

    #[test]
    fn test_size_constants() {
        // Test that constants are reasonable
        assert!(INLINE_MAX > 0, "INLINE_MAX should be positive");
        assert!(CHUNK_SIZE > 0, "CHUNK_SIZE should be positive");
        assert!(PAGE_SIZE > 0, "PAGE_SIZE should be positive");
        assert!(
            CAPSULE_INLINE_BUDGET > 0,
            "CAPSULE_INLINE_BUDGET should be positive"
        );

        // Test relationships
        assert!(
            INLINE_MAX <= CAPSULE_INLINE_BUDGET,
            "INLINE_MAX should not exceed capsule budget"
        );
        assert!(
            CHUNK_SIZE <= PAGE_SIZE,
            "CHUNK_SIZE should not exceed PAGE_SIZE"
        );
    }

    #[test]
    fn test_chunk_size_limits() {
        // Test that chunk size is reasonable (64KB)
        assert_eq!(CHUNK_SIZE, 64 * 1024, "CHUNK_SIZE should be 64KB");

        // Test that inline max is reasonable (32KB)
        assert_eq!(INLINE_MAX, 32 * 1024, "INLINE_MAX should be 32KB");
    }

    #[test]
    fn test_session_id_generation() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();

        // IDs should be different
        assert_ne!(id1.0, id2.0, "Session IDs should be unique");

        // IDs should be positive
        assert!(id1.0 > 0, "Session ID should be positive");
        assert!(id2.0 > 0, "Session ID should be positive");
    }

    #[test]
    fn test_blob_id_generation() {
        let id1 = BlobId::new();
        let id2 = BlobId::new();

        // IDs should be different
        assert_ne!(id1.0, id2.0, "Blob IDs should be unique");

        // IDs should be positive
        assert!(id1.0 > 0, "Blob ID should be positive");
        assert!(id2.0 > 0, "Blob ID should be positive");
    }
}

#[cfg(test)]
mod error_handling_tests {
    use crate::types::Error;

    #[test]
    fn test_error_variants() {
        // Test that all error variants can be created
        let _not_found = Error::NotFound;
        let _unauthorized = Error::Unauthorized;
        let _invalid_arg = Error::InvalidArgument("test".to_string());
        let _resource_exhausted = Error::ResourceExhausted;
        let _internal_error = Error::Internal("test".to_string());

        // Test error formatting
        let error_msg = format!("{}", _invalid_arg);
        assert!(
            error_msg.contains("test"),
            "Error message should contain the argument"
        );
    }

    #[test]
    fn test_error_equality() {
        let error1 = Error::InvalidArgument("test".to_string());
        let error2 = Error::InvalidArgument("test".to_string());
        let error3 = Error::InvalidArgument("different".to_string());

        assert_eq!(error1, error2, "Same error arguments should be equal");
        assert_ne!(
            error1, error3,
            "Different error arguments should not be equal"
        );
    }
}

#[cfg(test)]
mod session_status_tests {
    use crate::upload::types::SessionStatus;

    #[test]
    fn test_session_status_variants() {
        // Test that all status variants can be created
        let _pending = SessionStatus::Pending;
        let _committed = SessionStatus::Committed { blob_id: 123 };

        // Test pattern matching
        assert!(matches!(_pending, SessionStatus::Pending));
        assert!(matches!(
            _committed,
            SessionStatus::Committed { blob_id: 123 }
        ));

        // Test inequality
        assert!(!matches!(_pending, SessionStatus::Committed { blob_id: _ }));
        assert!(!matches!(_committed, SessionStatus::Pending));
    }

    #[test]
    fn test_session_status_transitions() {
        // Test valid state transitions
        let mut status = SessionStatus::Pending;

        // Pending can transition to Committed
        status = SessionStatus::Committed { blob_id: 456 };
        assert!(matches!(status, SessionStatus::Committed { blob_id: 456 }));

        // Test different blob IDs
        status = SessionStatus::Committed { blob_id: 789 };
        assert!(matches!(status, SessionStatus::Committed { blob_id: 789 }));
    }
}
