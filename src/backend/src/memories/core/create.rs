//! Memory creation operations
//!
//! This module contains the core business logic for creating memories
//! with various asset types and storage backends.

use super::{model_helpers::*, traits::*};
use crate::capsule_acl::CapsuleAcl;
use crate::types::{
    AssetMetadata, BlobRef, CapsuleId, Error, Memory, MemoryAccess, MemoryAssetBlobExternal,
    MemoryAssetBlobInternal, MemoryAssetInline, MemoryId, MemoryMetadata, StorageEdgeBlobType,
};

/// Core memory creation function - pure business logic
pub fn memories_create_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    capsule_id: CapsuleId,
    inline_bytes: Option<Vec<u8>>,
    blob_ref: Option<BlobRef>,
    storage_type: Option<StorageEdgeBlobType>,
    external_storage_key: Option<String>,
    external_url: Option<String>,
    _external_size: Option<u64>,
    _external_hash: Option<Vec<u8>>,
    asset_metadata: AssetMetadata,
    idempotency_key: String,
) -> std::result::Result<MemoryId, Error> {
    let caller = env.caller();
    let now = env.now();

    // Check permissions
    let capsule_access = store
        .get_capsule_for_acl(&capsule_id)
        .ok_or(Error::NotFound)?;

    if !capsule_access.can_write(&caller) {
        return Err(Error::Unauthorized);
    }

    // Generate memory ID from idempotency key
    let memory_id = format!("mem_{}", idempotency_key);

    // Check if memory already exists (idempotency)
    if let Some(existing_memory) = store.get_memory(&capsule_id, &memory_id) {
        return Ok(existing_memory.id);
    }

    // Determine memory type from asset metadata
    let memory_type = memory_type_from_asset(&asset_metadata);

    // Create memory metadata
    let base = match &asset_metadata {
        AssetMetadata::Image(meta) => &meta.base,
        AssetMetadata::Video(meta) => &meta.base,
        AssetMetadata::Audio(meta) => &meta.base,
        AssetMetadata::Document(meta) => &meta.base,
        AssetMetadata::Note(meta) => &meta.base,
    };

    let metadata = MemoryMetadata {
        memory_type,
        title: Some(base.name.clone()),
        description: base.description.clone(),
        content_type: base.mime_type.clone(),
        created_at: now,
        updated_at: now,
        uploaded_at: now,
        date_of_memory: None,
        file_created_at: None,
        parent_folder_id: None,
        tags: base.tags.clone(),
        deleted_at: None,
        people_in_memory: None,
        location: None,
        memory_notes: None,
        created_by: Some(caller.to_string()),
        database_storage_edges: vec![],
    };

    // Create access control
    let access = MemoryAccess::Private {
        owner_secure_code: format!("secure_{}", now),
    };

    // Build assets based on input
    let mut inline_assets = Vec::new();
    let mut blob_internal_assets = Vec::new();
    let mut blob_external_assets = Vec::new();

    // Handle inline assets
    if let Some(bytes) = inline_bytes {
        let inline_asset = MemoryAssetInline {
            bytes,
            metadata: asset_metadata.clone(),
        };
        inline_assets.push(inline_asset);
    }

    // Handle internal blob assets
    if let Some(blob_ref) = blob_ref {
        let internal_asset = MemoryAssetBlobInternal {
            blob_ref,
            metadata: asset_metadata.clone(),
        };
        blob_internal_assets.push(internal_asset);
    }

    // Handle external assets
    if let (Some(storage_type), Some(storage_key)) = (storage_type, external_storage_key) {
        if !is_valid_storage_type(&storage_type) {
            return Err(Error::InvalidArgument("Invalid storage type".to_string()));
        }

        let external_asset = MemoryAssetBlobExternal {
            location: storage_type,
            storage_key,
            url: external_url,
            metadata: asset_metadata,
        };
        blob_external_assets.push(external_asset);
    }

    // Create the memory
    let memory = Memory {
        id: memory_id.clone(),
        metadata,
        access,
        inline_assets,
        blob_internal_assets,
        blob_external_assets,
    };

    // Store the memory
    store.insert_memory(&capsule_id, memory)?;

    Ok(memory_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memories::core::traits::*;
    use candid::Principal;
    use std::collections::HashMap;

    // Mock implementations for testing
    struct MockEnv {
        caller: crate::types::PersonRef,
        now: u64,
    }

    impl Env for MockEnv {
        fn caller(&self) -> crate::types::PersonRef {
            self.caller.clone()
        }
        fn now(&self) -> u64 {
            self.now
        }
    }

    struct MockStore {
        capsules: HashMap<crate::types::CapsuleId, MockCapsule>,
        memories:
            HashMap<crate::types::CapsuleId, HashMap<crate::types::MemoryId, crate::types::Memory>>,
    }

    struct MockCapsule {
        id: crate::types::CapsuleId,
        owner: crate::types::PersonRef,
        memories: Vec<crate::types::MemoryId>,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                capsules: HashMap::new(),
                memories: HashMap::new(),
            }
        }

        fn create_capsule(&mut self, id: crate::types::CapsuleId, owner: crate::types::PersonRef) {
            self.capsules.insert(
                id.clone(),
                MockCapsule {
                    id: id.clone(),
                    owner,
                    memories: Vec::new(),
                },
            );
        }
    }

    impl Store for MockStore {
        fn insert_memory(
            &mut self,
            capsule: &crate::types::CapsuleId,
            memory: crate::types::Memory,
        ) -> std::result::Result<(), crate::types::Error> {
            self.memories
                .entry(capsule.clone())
                .or_insert_with(HashMap::new)
                .insert(memory.id.clone(), memory);
            Ok(())
        }

        fn get_memory(
            &self,
            capsule: &crate::types::CapsuleId,
            id: &crate::types::MemoryId,
        ) -> Option<crate::types::Memory> {
            self.memories.get(capsule)?.get(id).cloned()
        }

        fn delete_memory(
            &mut self,
            capsule: &crate::types::CapsuleId,
            id: &crate::types::MemoryId,
        ) -> std::result::Result<(), crate::types::Error> {
            if let Some(capsule_memories) = self.memories.get_mut(capsule) {
                capsule_memories.remove(id);
            }
            Ok(())
        }

        fn get_accessible_capsules(
            &self,
            caller: &crate::types::PersonRef,
        ) -> Vec<crate::types::CapsuleId> {
            self.capsules
                .iter()
                .filter(|(_, capsule)| &capsule.owner == caller)
                .map(|(id, _)| id.clone())
                .collect()
        }

        fn get_capsule_for_acl(
            &self,
            capsule_id: &crate::types::CapsuleId,
        ) -> Option<crate::capsule_acl::CapsuleAccess> {
            self.capsules.get(capsule_id).map(|capsule| {
                let mut owners = std::collections::HashMap::new();
                owners.insert(
                    capsule.owner.clone(),
                    crate::types::OwnerState {
                        since: 1_695_000_000_000,
                        last_activity_at: 1_695_000_000_000,
                    },
                );
                crate::capsule_acl::CapsuleAccess::new(
                    capsule.owner.clone(),
                    owners,
                    std::collections::HashMap::new(),
                )
            })
        }
    }

    #[test]
    fn test_memories_create_core_inline_asset() {
        let mut store = MockStore::new();
        let owner = crate::types::PersonRef::Principal(
            Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap(),
        );
        let capsule_id = "test-capsule-1".to_string();
        let env = MockEnv {
            caller: owner.clone(),
            now: 1_695_000_000_000,
        };

        // Setup
        store.create_capsule(capsule_id.clone(), owner.clone());

        // Test create with inline asset
        let inline_bytes = Some(vec![1, 2, 3, 4]);
        let asset_metadata = crate::types::AssetMetadata::Image(crate::types::ImageAssetMetadata {
            dpi: None,
            color_space: Some("RGB".to_string()),
            base: crate::types::AssetMetadataBase {
                url: None,
                height: Some(100),
                updated_at: 1_695_000_000_000,
                asset_type: crate::types::AssetType::Original,
                sha256: None,
                name: "test.jpg".to_string(),
                storage_key: None,
                tags: vec![],
                processing_error: None,
                mime_type: "image/jpeg".to_string(),
                description: None,
                created_at: 1_695_000_000_000,
                deleted_at: None,
                bytes: 4,
                asset_location: None,
                width: Some(100),
                processing_status: None,
                bucket: None,
            },
            exif_data: None,
            compression_ratio: None,
            orientation: None,
        });

        let result = memories_create_core(
            &env,
            &mut store,
            capsule_id.clone(),
            inline_bytes,
            None,
            None,
            None,
            None,
            None,
            None,
            asset_metadata,
            "test-create".to_string(),
        );

        match result {
            Ok(memory_id) => {
                assert_eq!(memory_id, "mem_test-create");

                // Verify memory was created
                let memory = store.get_memory(&capsule_id, &memory_id).unwrap();
                assert_eq!(memory.inline_assets.len(), 1);
                assert_eq!(memory.blob_internal_assets.len(), 0);
                assert_eq!(memory.blob_external_assets.len(), 0);
            }
            Err(e) => panic!("Memory creation should succeed: {:?}", e),
        }
    }

    #[test]
    fn test_memories_create_core_idempotency() {
        let mut store = MockStore::new();
        let owner = crate::types::PersonRef::Principal(
            Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap(),
        );
        let capsule_id = "test-capsule-1".to_string();
        let env = MockEnv {
            caller: owner.clone(),
            now: 1_695_000_000_000,
        };

        // Setup
        store.create_capsule(capsule_id.clone(), owner.clone());

        let asset_metadata = crate::types::AssetMetadata::Image(crate::types::ImageAssetMetadata {
            dpi: None,
            color_space: Some("RGB".to_string()),
            base: crate::types::AssetMetadataBase {
                url: None,
                height: Some(100),
                updated_at: 1_695_000_000_000,
                asset_type: crate::types::AssetType::Original,
                sha256: None,
                name: "test.jpg".to_string(),
                storage_key: None,
                tags: vec![],
                processing_error: None,
                mime_type: "image/jpeg".to_string(),
                description: None,
                created_at: 1_695_000_000_000,
                deleted_at: None,
                bytes: 4,
                asset_location: None,
                width: Some(100),
                processing_status: None,
                bucket: None,
            },
            exif_data: None,
            compression_ratio: None,
            orientation: None,
        });

        // First creation
        let result1 = memories_create_core(
            &env,
            &mut store,
            capsule_id.clone(),
            Some(vec![1, 2, 3, 4]),
            None,
            None,
            None,
            None,
            None,
            None,
            asset_metadata.clone(),
            "idempotent-test".to_string(),
        );

        // Second creation with same idempotency key
        let result2 = memories_create_core(
            &env,
            &mut store,
            capsule_id.clone(),
            Some(vec![5, 6, 7, 8]), // Different data
            None,
            None,
            None,
            None,
            None,
            None,
            asset_metadata,
            "idempotent-test".to_string(), // Same key
        );

        match (result1, result2) {
            (Ok(id1), Ok(id2)) => {
                assert_eq!(id1, id2); // Should return same ID
                assert_eq!(id1, "mem_idempotent-test");
            }
            _ => panic!("Both creations should succeed and return same ID"),
        }
    }
}
