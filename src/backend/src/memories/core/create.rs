//! Memory creation operations
//!
//! This module contains the core business logic for creating memories
//! with various asset types and storage backends.

use super::{model_helpers::*, traits::*};
use crate::capsule::domain::SharingStatus;
use crate::capsule_acl::CapsuleAcl;
use crate::memories::types::{InternalBlobAssetInput, MemoryMetadata};
use crate::types::{
    AssetMetadata, BlobRef, CapsuleId, Error, Memory, MemoryAssetBlobInternal, MemoryId,
    StorageEdgeBlobType,
};

/// Core memory creation function - pure business logic
///
/// This function contains all the business logic for memory creation
/// without any ICP-specific dependencies. It can be fully unit tested.
pub fn memories_create_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    capsule_id: CapsuleId,
    bytes: Option<Vec<u8>>,
    blob_ref: Option<BlobRef>,
    external_location: Option<StorageEdgeBlobType>,
    external_storage_key: Option<String>,
    external_url: Option<String>,
    external_size: Option<u64>,
    external_hash: Option<Vec<u8>>,
    asset_metadata: AssetMetadata,
    _idem: String,
) -> std::result::Result<MemoryId, Error> {
    // Validate that exactly one asset type is provided
    let asset_count =
        bytes.is_some() as u8 + blob_ref.is_some() as u8 + external_location.is_some() as u8;
    if asset_count != 1 {
        return Err(Error::InvalidArgument(
            "Exactly one asset type must be provided: bytes, blob_ref, or external_location"
                .to_string(),
        ));
    }

    // Enforce deep asset consistency
    let base = asset_metadata.get_base();
    match (&bytes, &blob_ref, &external_location) {
        (Some(b), None, None) => {
            // inline: base.bytes must equal bytes.len()
            if base.bytes != b.len() as u64 {
                return Err(Error::InvalidArgument(
                    "inline bytes_len != metadata.base.bytes".to_string(),
                ));
            }
        }
        (None, Some(br), None) => {
            // internal blob: base.bytes must equal blob_ref.len
            if base.bytes != br.len {
                return Err(Error::InvalidArgument(
                    "blob_ref.len != metadata.base.bytes".to_string(),
                ));
            }
        }
        (None, None, Some(_loc)) => {
            // external: storage_key must be present
            if external_storage_key.as_deref().unwrap_or("").is_empty() {
                return Err(Error::InvalidArgument(
                    "external_storage_key is required".to_string(),
                ));
            }
            // size consistency (prefer both present and equal)
            if let Some(sz) = external_size {
                if base.bytes != sz {
                    return Err(Error::InvalidArgument(
                        "external_size != metadata.base.bytes".to_string(),
                    ));
                }
            }
            // optional hash consistency
            if let (Some(h), Some(meta_hash)) = (&external_hash, &base.sha256) {
                if h != meta_hash {
                    return Err(Error::InvalidArgument(
                        "external_hash != metadata.base.sha256".to_string(),
                    ));
                }
            }
        }
        _ => {} // already handled by asset_count != 1 above
    }

    // Check if capsule exists and caller has write access using centralized ACL
    let caller = env.caller();
    let capsule_access = store
        .get_capsule_for_acl(&capsule_id)
        .ok_or(Error::NotFound)?;

    if !capsule_access.can_write(&caller) {
        ic_cdk::println!(
            "[ACL] op=create caller={} cap={} read={} write={} delete={} - UNAUTHORIZED",
            caller,
            capsule_id,
            capsule_access.can_read(&caller),
            capsule_access.can_write(&caller),
            capsule_access.can_delete(&caller)
        );
        return Err(Error::Unauthorized);
    }

    // Log successful ACL check
    ic_cdk::println!(
        "[ACL] op=create caller={} cap={} read={} write={} delete={} - AUTHORIZED",
        caller,
        capsule_id,
        capsule_access.can_read(&caller),
        capsule_access.can_write(&caller),
        capsule_access.can_delete(&caller)
    );

    // Capture timestamp once for consistency
    let now = env.now();

    // Generate deterministic UUID from idempotency key for proper idempotency
    let memory_id = generate_deterministic_uuid_from_idem(&_idem);

    // Check for existing memory (idempotency)
    if let Some(_existing) = store.get_memory(&capsule_id, &memory_id) {
        return Ok(memory_id); // Return existing ID for idempotency
    }

    // Create memory based on asset type
    let mut memory = if let Some(bytes_data) = bytes {
        create_inline_memory(
            &memory_id,
            &capsule_id,
            bytes_data,
            asset_metadata,
            now,
            &caller,
        )
    } else if let Some(blob) = blob_ref {
        create_blob_memory(&memory_id, &capsule_id, blob, asset_metadata, now, &caller)
    } else if let Some(location) = external_location {
        create_external_memory(
            &memory_id,
            &capsule_id,
            location,
            external_storage_key,
            external_url,
            external_size,
            external_hash,
            asset_metadata,
            now,
            &caller,
        )
    } else {
        return Err(Error::InvalidArgument(
            "No valid asset type provided".to_string(),
        ));
    };

    // NEW: Compute and store dashboard fields
    memory.update_dashboard_fields();

    // Insert memory into store
    store.insert_memory(&capsule_id, memory)?;

    // POST-WRITE ASSERTION: Verify memory was actually created
    // This catches silent failures where the function returns Ok but no data is persisted
    if store.get_memory(&capsule_id, &memory_id).is_none() {
        return Err(Error::Internal(
            "Post-write readback failed: memory was not persisted".to_string(),
        ));
    }

    // Debug: Log successful memory creation
    ic_cdk::println!(
        "[DEBUG] memories_create: successfully created memory {} in capsule {}",
        memory_id,
        capsule_id
    );

    Ok(memory_id)
}

/// Create memory with internal blob assets (ICP blob storage)
///
/// This function creates a memory with one or more internal blob assets.
/// Each asset references a blob stored in the ICP blob store.
pub fn memories_create_with_internal_blobs_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    capsule_id: CapsuleId,
    memory_metadata: MemoryMetadata,
    internal_blob_assets: Vec<InternalBlobAssetInput>,
    _idem: String,
) -> std::result::Result<MemoryId, Error> {
    // Validate inputs
    if internal_blob_assets.is_empty() {
        return Err(Error::InvalidArgument(
            "At least one internal blob asset is required".to_string(),
        ));
    }

    // Get caller for ACL check
    let caller = env.caller();

    // Check ACL permissions
    let capsule_access = store
        .get_capsule_for_acl(&capsule_id)
        .ok_or(Error::NotFound)?;
    if !capsule_access.can_write(&caller) {
        return Err(Error::Unauthorized);
    }

    // ACL check passed - no logging in pure function

    // Capture timestamp once for consistency
    let now = env.now();

    // Generate deterministic UUID from idempotency key for proper idempotency
    let memory_id = generate_deterministic_uuid_from_idem(&_idem);

    // Check for existing memory (idempotency)
    if let Some(_existing) = store.get_memory(&capsule_id, &memory_id) {
        return Ok(memory_id); // Return existing ID for idempotency
    }

    // Create memory with multiple internal blob assets
    let mut blob_internal_assets = Vec::new();

    for asset_input in &internal_blob_assets {
        // Parse blob_id to get BlobRef
        let blob_ref = if asset_input.blob_id.starts_with("blob_") {
            // Extract the numeric ID from "blob_1234567890"
            let id_str = &asset_input.blob_id[5..]; // Remove "blob_" prefix
            let _blob_id = id_str.parse::<u64>().map_err(|_| {
                Error::InvalidArgument(format!("Invalid blob_id format: {}", asset_input.blob_id))
            })?;

            BlobRef {
                locator: asset_input.blob_id.clone(),
                hash: None, // Hash will be retrieved from blob store if needed
                len: 0,     // Length will be retrieved from blob store if needed
            }
        } else {
            return Err(Error::InvalidArgument(format!(
                "Invalid blob_id format: {}",
                asset_input.blob_id
            )));
        };

        // Create the internal blob asset
        let blob_asset = MemoryAssetBlobInternal {
            asset_id: generate_asset_id(&caller, now),
            blob_ref,
            metadata: asset_input.metadata.clone(),
        };

        blob_internal_assets.push(blob_asset);
    }

    // Store the count before moving the vector
    let _asset_count = blob_internal_assets.len();

    // Create the memory with multiple internal blob assets
    let mut memory = Memory {
        id: memory_id.clone(),
        capsule_id: capsule_id.clone(),
        metadata: memory_metadata,
        // access: crate::types::MemoryAccess::Private {
        //     owner_secure_code: "default_secure_code".to_string(),
        // },
        access_entries: vec![create_owner_access_entry(&caller, now)], // ✅ Create owner access entry
        // ❌ REMOVED: public_policy field - now unified in AccessEntry
        inline_assets: vec![],
        blob_internal_assets,
        blob_external_assets: vec![],
    };

    // NEW: Compute and store dashboard fields
    memory.update_dashboard_fields();

    // Insert memory into store
    store.insert_memory(&capsule_id, memory)?;

    // POST-WRITE ASSERTION: Verify memory was actually created
    if store.get_memory(&capsule_id, &memory_id).is_none() {
        return Err(Error::Internal(
            "Post-write readback failed: memory was not persisted".to_string(),
        ));
    }

    // Memory creation successful - no logging in pure function

    Ok(memory_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capsule_acl::CapsuleAccess;
    use crate::memories::types::{
        AssetMetadata, AssetMetadataBase, AssetType, ImageAssetMetadata, MemoryMetadata, MemoryType,
    };
    use crate::types::{ControllerState, OwnerState, PersonRef};
    use candid::Principal;
    use std::collections::HashMap;

    // Mock implementations for testing
    struct MockEnv {
        caller: PersonRef,
        now: u64,
    }

    impl Env for MockEnv {
        fn caller(&self) -> PersonRef {
            self.caller.clone()
        }

        fn now(&self) -> u64 {
            self.now
        }
    }

    struct MockStore {
        memories: HashMap<(CapsuleId, MemoryId), Memory>,
        capsules: HashMap<CapsuleId, CapsuleAccess>,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                memories: HashMap::new(),
                capsules: HashMap::new(),
            }
        }

        fn add_capsule(&mut self, capsule_id: CapsuleId, access: CapsuleAccess) {
            self.capsules.insert(capsule_id, access);
        }
    }

    impl Store for MockStore {
        fn insert_memory(
            &mut self,
            capsule: &CapsuleId,
            memory: Memory,
        ) -> std::result::Result<(), Error> {
            self.memories
                .insert((capsule.clone(), memory.id.clone()), memory);
            Ok(())
        }

        fn get_memory(&self, capsule: &CapsuleId, id: &MemoryId) -> Option<Memory> {
            self.memories.get(&(capsule.clone(), id.clone())).cloned()
        }

        fn delete_memory(
            &mut self,
            capsule: &CapsuleId,
            id: &MemoryId,
        ) -> std::result::Result<(), Error> {
            self.memories.remove(&(capsule.clone(), id.clone()));
            Ok(())
        }

        fn update_memory(
            &mut self,
            capsule: &CapsuleId,
            id: &MemoryId,
            memory: Memory,
        ) -> std::result::Result<(), Error> {
            self.memories.insert((capsule.clone(), id.clone()), memory);
            Ok(())
        }

        fn get_all_memories(&self, capsule: &CapsuleId) -> Vec<Memory> {
            self.memories
                .iter()
                .filter(|((c, _), _)| c == capsule)
                .map(|(_, memory)| memory.clone())
                .collect()
        }

        fn get_accessible_capsules(&self, _caller: &PersonRef) -> Vec<CapsuleId> {
            self.capsules.keys().cloned().collect()
        }

        fn get_capsule_for_acl(&self, capsule_id: &CapsuleId) -> Option<CapsuleAccess> {
            self.capsules.get(capsule_id).cloned()
        }
    }

    fn create_test_asset_metadata() -> AssetMetadata {
        let base = AssetMetadataBase {
            name: "test_image.jpg".to_string(),
            description: Some("Test image for unit testing".to_string()),
            tags: vec!["test".to_string(), "unit".to_string()],
            asset_type: AssetType::Original,
            bytes: 1024,
            mime_type: "image/jpeg".to_string(),
            sha256: Some([1u8; 32]),
            width: Some(1920),
            height: Some(1080),
            url: None,
            storage_key: None,
            bucket: None,
            asset_location: None,
            processing_status: None,
            processing_error: None,
            created_at: 1234567890,
            updated_at: 1234567890,
            deleted_at: None,
        };

        AssetMetadata::Image(ImageAssetMetadata {
            base,
            color_space: Some("sRGB".to_string()),
            exif_data: None,
            compression_ratio: Some(0.8),
            dpi: Some(72),
            orientation: Some(1),
        })
    }

    fn create_test_memory_metadata() -> MemoryMetadata {
        MemoryMetadata {
            memory_type: MemoryType::Image,
            title: Some("Test Memory".to_string()),
            description: Some("Test memory for unit testing".to_string()),
            content_type: "image/jpeg".to_string(),
            created_at: 1234567890,
            updated_at: 1234567890,
            uploaded_at: 1234567890,
            date_of_memory: None,
            file_created_at: None,
            parent_folder_id: None,
            tags: vec!["test".to_string()],
            deleted_at: None,
            people_in_memory: None,
            location: None,
            memory_notes: None,
            created_by: Some("test-user".to_string()),
            database_storage_edges: vec![],

            // NEW: Pre-computed dashboard fields (defaults)
            shared_count: 0,
            sharing_status: SharingStatus::Private,
            total_size: 1024,
            asset_count: 1,
            thumbnail_url: None,
            primary_asset_url: None,
            has_thumbnails: false,
            has_previews: false,
        }
    }

    #[test]
    fn test_memories_create_with_internal_blobs_single_asset() {
        // Setup
        let caller = PersonRef::Principal(Principal::from_slice(&[1, 2, 3, 4, 5]));
        let capsule_id = "capsule_123".to_string();
        let env = MockEnv {
            caller: caller.clone(),
            now: 1234567890,
        };
        let mut store = MockStore::new();

        // Add capsule with write access
        let mut owners = HashMap::new();
        owners.insert(
            caller.clone(),
            OwnerState {
                since: 1234567890,
                last_activity_at: 1234567890,
            },
        );
        let capsule_access = CapsuleAccess::new(caller.clone(), owners, HashMap::new());
        store.add_capsule(capsule_id.clone(), capsule_access);

        // Create test data
        let memory_metadata = create_test_memory_metadata();
        let asset_metadata = create_test_asset_metadata();
        let internal_blob_assets = vec![InternalBlobAssetInput {
            blob_id: "blob_1234567890".to_string(),
            metadata: asset_metadata,
        }];

        // Execute
        let result = memories_create_with_internal_blobs_core(
            &env,
            &mut store,
            capsule_id.clone(),
            memory_metadata,
            internal_blob_assets,
            "test-idem".to_string(),
        );

        // Verify
        assert!(result.is_ok());
        let memory_id = result.unwrap();
        // Should be a UUID format (not compound ID)
        assert!(memory_id.len() == 36); // UUID format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
        assert!(memory_id.contains('-'));

        // Verify memory was created
        let memory = store.get_memory(&capsule_id, &memory_id);
        assert!(memory.is_some());
        let memory = memory.unwrap();
        assert_eq!(memory.id, memory_id);
        assert_eq!(memory.blob_internal_assets.len(), 1);
        assert_eq!(
            memory.blob_internal_assets[0].blob_ref.locator,
            "blob_1234567890"
        );
        assert_eq!(memory.inline_assets.len(), 0);
        assert_eq!(memory.blob_external_assets.len(), 0);
    }

    #[test]
    fn test_memories_create_with_internal_blobs_multiple_assets() {
        // Setup
        let caller = PersonRef::Principal(Principal::from_slice(&[1, 2, 3, 4, 5]));
        let capsule_id = "capsule_456".to_string();
        let env = MockEnv {
            caller: caller.clone(),
            now: 1234567890,
        };
        let mut store = MockStore::new();

        // Add capsule with write access
        let mut owners = HashMap::new();
        owners.insert(
            caller.clone(),
            OwnerState {
                since: 1234567890,
                last_activity_at: 1234567890,
            },
        );
        let capsule_access = CapsuleAccess::new(caller.clone(), owners, HashMap::new());
        store.add_capsule(capsule_id.clone(), capsule_access);

        // Create test data with 4 assets (like the 2lane-4asset test)
        let memory_metadata = create_test_memory_metadata();
        let asset_metadata = create_test_asset_metadata();
        let internal_blob_assets = vec![
            InternalBlobAssetInput {
                blob_id: "blob_1111111111".to_string(),
                metadata: asset_metadata.clone(),
            },
            InternalBlobAssetInput {
                blob_id: "blob_2222222222".to_string(),
                metadata: asset_metadata.clone(),
            },
            InternalBlobAssetInput {
                blob_id: "blob_3333333333".to_string(),
                metadata: asset_metadata.clone(),
            },
            InternalBlobAssetInput {
                blob_id: "blob_4444444444".to_string(),
                metadata: asset_metadata,
            },
        ];

        // Execute
        let result = memories_create_with_internal_blobs_core(
            &env,
            &mut store,
            capsule_id.clone(),
            memory_metadata,
            internal_blob_assets,
            "test-4assets".to_string(),
        );

        // Verify
        assert!(result.is_ok());
        let memory_id = result.unwrap();
        // Should be a UUID format (not compound ID)
        assert!(memory_id.len() == 36); // UUID format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
        assert!(memory_id.contains('-'));

        // Verify memory was created with all 4 assets
        let memory = store.get_memory(&capsule_id, &memory_id);
        assert!(memory.is_some());
        let memory = memory.unwrap();
        assert_eq!(memory.id, memory_id);
        assert_eq!(memory.blob_internal_assets.len(), 4);

        // Verify all blob IDs are present
        let blob_ids: Vec<String> = memory
            .blob_internal_assets
            .iter()
            .map(|asset| asset.blob_ref.locator.clone())
            .collect();
        assert!(blob_ids.contains(&"blob_1111111111".to_string()));
        assert!(blob_ids.contains(&"blob_2222222222".to_string()));
        assert!(blob_ids.contains(&"blob_3333333333".to_string()));
        assert!(blob_ids.contains(&"blob_4444444444".to_string()));

        assert_eq!(memory.inline_assets.len(), 0);
        assert_eq!(memory.blob_external_assets.len(), 0);
    }

    #[test]
    fn test_memories_create_with_internal_blobs_empty_assets() {
        // Setup
        let caller = PersonRef::Principal(Principal::from_slice(&[1, 2, 3, 4, 5]));
        let capsule_id = "capsule_789".to_string();
        let env = MockEnv {
            caller: caller.clone(),
            now: 1234567890,
        };
        let mut store = MockStore::new();

        // Add capsule with write access
        let mut owners = HashMap::new();
        owners.insert(
            caller.clone(),
            OwnerState {
                since: 1234567890,
                last_activity_at: 1234567890,
            },
        );
        let capsule_access = CapsuleAccess::new(caller.clone(), owners, HashMap::new());
        store.add_capsule(capsule_id.clone(), capsule_access);

        // Create test data with empty assets
        let memory_metadata = create_test_memory_metadata();
        let internal_blob_assets = vec![];

        // Execute
        let result = memories_create_with_internal_blobs_core(
            &env,
            &mut store,
            capsule_id,
            memory_metadata,
            internal_blob_assets,
            "test-empty".to_string(),
        );

        // Verify
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidArgument(msg) => {
                assert!(msg.contains("At least one internal blob asset is required"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[test]
    fn test_memories_create_with_internal_blobs_unauthorized() {
        // Setup
        let caller = PersonRef::Principal(Principal::from_slice(&[1, 2, 3, 4, 5]));
        let other_caller = PersonRef::Principal(Principal::from_slice(&[6, 7, 8, 9, 10]));
        let capsule_id = "capsule_unauthorized".to_string();
        let env = MockEnv {
            caller: other_caller.clone(), // Different caller
            now: 1234567890,
        };
        let mut store = MockStore::new();

        // Add capsule with write access only for the original caller
        let mut owners = HashMap::new();
        owners.insert(
            caller.clone(),
            OwnerState {
                since: 1234567890,
                last_activity_at: 1234567890,
            },
        );
        let capsule_access = CapsuleAccess::new(caller.clone(), owners, HashMap::new());
        store.add_capsule(capsule_id.clone(), capsule_access);

        // Create test data
        let memory_metadata = create_test_memory_metadata();
        let asset_metadata = create_test_asset_metadata();
        let internal_blob_assets = vec![InternalBlobAssetInput {
            blob_id: "blob_1234567890".to_string(),
            metadata: asset_metadata,
        }];

        // Execute
        let result = memories_create_with_internal_blobs_core(
            &env,
            &mut store,
            capsule_id,
            memory_metadata,
            internal_blob_assets,
            "test-unauthorized".to_string(),
        );

        // Verify
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Unauthorized => {
                // Expected
            }
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[test]
    fn test_memories_create_with_internal_blobs_invalid_blob_id() {
        // Setup
        let caller = PersonRef::Principal(Principal::from_slice(&[1, 2, 3, 4, 5]));
        let capsule_id = "capsule_invalid".to_string();
        let env = MockEnv {
            caller: caller.clone(),
            now: 1234567890,
        };
        let mut store = MockStore::new();

        // Add capsule with write access
        let mut owners = HashMap::new();
        owners.insert(
            caller.clone(),
            OwnerState {
                since: 1234567890,
                last_activity_at: 1234567890,
            },
        );
        let capsule_access = CapsuleAccess::new(caller.clone(), owners, HashMap::new());
        store.add_capsule(capsule_id.clone(), capsule_access);

        // Create test data with invalid blob ID
        let memory_metadata = create_test_memory_metadata();
        let asset_metadata = create_test_asset_metadata();
        let internal_blob_assets = vec![InternalBlobAssetInput {
            blob_id: "invalid_blob_id".to_string(), // Invalid format
            metadata: asset_metadata,
        }];

        // Execute
        let result = memories_create_with_internal_blobs_core(
            &env,
            &mut store,
            capsule_id,
            memory_metadata,
            internal_blob_assets,
            "test-invalid".to_string(),
        );

        // Verify
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidArgument(msg) => {
                assert!(msg.contains("Invalid blob_id format"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[test]
    fn test_memories_create_with_internal_blobs_idempotency() {
        // Setup
        let caller = PersonRef::Principal(Principal::from_slice(&[1, 2, 3, 4, 5]));
        let capsule_id = "capsule_idempotent".to_string();
        let env = MockEnv {
            caller: caller.clone(),
            now: 1234567890,
        };
        let mut store = MockStore::new();

        // Add capsule with write access
        let mut owners = HashMap::new();
        owners.insert(
            caller.clone(),
            OwnerState {
                since: 1234567890,
                last_activity_at: 1234567890,
            },
        );
        let capsule_access = CapsuleAccess::new(caller.clone(), owners, HashMap::new());
        store.add_capsule(capsule_id.clone(), capsule_access);

        // Create test data
        let memory_metadata = create_test_memory_metadata();
        let asset_metadata = create_test_asset_metadata();
        let internal_blob_assets = vec![InternalBlobAssetInput {
            blob_id: "blob_1234567890".to_string(),
            metadata: asset_metadata,
        }];

        // Execute first time
        let result1 = memories_create_with_internal_blobs_core(
            &env,
            &mut store,
            capsule_id.clone(),
            memory_metadata.clone(),
            internal_blob_assets.clone(),
            "test-idem".to_string(),
        );

        // Execute second time with same idempotency key
        let result2 = memories_create_with_internal_blobs_core(
            &env,
            &mut store,
            capsule_id.clone(),
            memory_metadata,
            internal_blob_assets,
            "test-idem".to_string(), // Same idempotency key
        );

        // Verify both succeed and return the same memory ID
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());

        // Verify only one memory was created
        let memories = store.get_all_memories(&capsule_id);
        assert_eq!(memories.len(), 1);
    }
}
