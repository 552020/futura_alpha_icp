//! Core memory management functions with dependency injection
//!
//! This module implements the decoupling pattern:
//! - Pure business logic separated from ICP-specific APIs
//! - Trait-based dependency injection for testability
//! - Post-write assertions to catch silent failures

use crate::types::{
    AssetMetadata, BlobRef, CapsuleId, Error, Memory, MemoryAccess, MemoryAssetBlobExternal,
    MemoryAssetBlobInternal, MemoryAssetInline, MemoryId, MemoryMetadata, MemoryType,
    MemoryUpdateData, PersonRef, StorageEdgeBlobType,
};
use candid::Principal;
use std::collections::{BTreeMap, HashMap};

// ============================================================================
// TRAIT DEFINITIONS
// ============================================================================

/// Environment abstraction for ICP-specific APIs
pub trait Env {
    fn caller(&self) -> PersonRef;
    fn now(&self) -> u64;
}

/// Storage abstraction for capsule store operations
pub trait Store {
    fn insert_memory(
        &mut self,
        capsule: &CapsuleId,
        memory: Memory,
    ) -> std::result::Result<(), Error>;
    fn get_memory(&self, capsule: &CapsuleId, id: &MemoryId) -> Option<Memory>;
    fn delete_memory(
        &mut self,
        capsule: &CapsuleId,
        id: &MemoryId,
    ) -> std::result::Result<(), Error>;
    fn get_accessible_capsules(&self, caller: &PersonRef) -> Vec<CapsuleId>;
}

// Removed unused struct: CapsuleRefMut

// ============================================================================
// CORE FUNCTIONS (PURE BUSINESS LOGIC)
// ============================================================================

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
    idem: String,
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

    // Check if capsule exists and caller has access
    let accessible_capsules = store.get_accessible_capsules(&env.caller());
    if !accessible_capsules.contains(&capsule_id) {
        return Err(Error::Unauthorized);
    }

    // Capture timestamp once for consistency
    let now = env.now();
    let caller = env.caller();

    // Generate deterministic memory ID
    let memory_id = format!("mem:{}:{}", &capsule_id, idem);

    // Check for existing memory (idempotency)
    if let Some(_existing) = store.get_memory(&capsule_id, &memory_id) {
        return Ok(memory_id); // Return existing ID for idempotency
    }

    // Create memory based on asset type
    let memory = if let Some(bytes_data) = bytes {
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

    // Insert memory into store
    store.insert_memory(&capsule_id, memory)?;

    // POST-WRITE ASSERTION: Verify memory was actually created
    // This catches silent failures where the function returns Ok but no data is persisted
    if store.get_memory(&capsule_id, &memory_id).is_none() {
        return Err(Error::Internal(
            "Post-write readback failed: memory was not persisted".to_string(),
        ));
    }

    Ok(memory_id)
}

/// Core memory reading function - pure business logic
pub fn memories_read_core<E: Env, S: Store>(
    env: &E,
    store: &S,
    memory_id: MemoryId,
) -> std::result::Result<Memory, Error> {
    // Get all accessible capsules for the caller
    let accessible_capsules = store.get_accessible_capsules(&env.caller());

    // Search for the memory across all accessible capsules
    for capsule_id in accessible_capsules {
        if let Some(memory) = store.get_memory(&capsule_id, &memory_id) {
            return Ok(memory);
        }
    }

    Err(Error::NotFound)
}

/// Core memory update function - pure business logic
pub fn memories_update_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_id: MemoryId,
    updates: MemoryUpdateData,
) -> std::result::Result<(), Error> {
    // Capture timestamp once for consistency
    let now = env.now();

    // Find the memory across all accessible capsules
    let accessible_capsules = store.get_accessible_capsules(&env.caller());

    for capsule_id in accessible_capsules {
        if let Some(mut memory) = store.get_memory(&capsule_id, &memory_id) {
            // TODO: Add ownership check when we have proper owner tracking
            // For now, if the caller has access to the capsule, they can update memories

            // Apply updates
            if let Some(name) = updates.name {
                memory.metadata.title = Some(name);
            }

            if let Some(metadata) = updates.metadata {
                memory.metadata = metadata;
            }

            if let Some(access) = updates.access {
                memory.access = access;
            }

            // Update timestamp with captured value
            memory.metadata.updated_at = now;

            // Save the updated memory
            store.insert_memory(&capsule_id, memory)?;

            // POST-WRITE ASSERTION: Verify memory was actually updated
            if let Some(updated_memory) = store.get_memory(&capsule_id, &memory_id) {
                if updated_memory.metadata.updated_at != now {
                    return Err(Error::Internal(
                        "Post-update readback failed: memory was not updated".to_string(),
                    ));
                }
            } else {
                return Err(Error::Internal(
                    "Post-update readback failed: memory was not found".to_string(),
                ));
            }

            return Ok(());
        }
    }

    Err(Error::NotFound)
}

/// Core memory deletion function - pure business logic
pub fn memories_delete_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_id: MemoryId,
) -> std::result::Result<(), Error> {
    // Find the memory across all accessible capsules
    let accessible_capsules = store.get_accessible_capsules(&env.caller());

    for capsule_id in accessible_capsules {
        if let Some(_memory) = store.get_memory(&capsule_id, &memory_id) {
            // TODO: Add ownership check when we have proper owner tracking
            // For now, if the caller has access to the capsule, they can delete memories

            // Delete the memory
            store.delete_memory(&capsule_id, &memory_id)?;

            // POST-WRITE ASSERTION: Verify memory was actually deleted
            if store.get_memory(&capsule_id, &memory_id).is_some() {
                return Err(Error::Internal(
                    "Post-delete readback failed: memory was not removed".to_string(),
                ));
            }

            return Ok(());
        }
    }

    Err(Error::NotFound)
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Derive MemoryType from AssetMetadata variant
fn memory_type_from_asset(meta: &AssetMetadata) -> MemoryType {
    match meta {
        AssetMetadata::Note(_) => MemoryType::Note,
        AssetMetadata::Image(_) => MemoryType::Image,
        AssetMetadata::Document(_) => MemoryType::Document,
        AssetMetadata::Audio(_) => MemoryType::Audio,
        AssetMetadata::Video(_) => MemoryType::Video,
    }
}

/// Create an inline memory (small assets stored directly)
fn create_inline_memory(
    memory_id: &str,
    _capsule_id: &CapsuleId,
    bytes: Vec<u8>,
    asset_metadata: AssetMetadata,
    now: u64,
    caller: &PersonRef,
) -> Memory {
    let inline_assets = vec![MemoryAssetInline {
        bytes: bytes.clone(),
        metadata: asset_metadata.clone(),
    }];

    let base = asset_metadata.get_base();
    let created_by = match caller {
        PersonRef::Principal(p) => Some(p.to_text()),
        PersonRef::Opaque(s) => Some(s.clone()),
    };

    Memory {
        id: memory_id.to_string(),
        metadata: MemoryMetadata {
            memory_type: memory_type_from_asset(&asset_metadata),
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
            created_by,
            database_storage_edges: vec![],
        },
        access: MemoryAccess::Private {
            owner_secure_code: "test_code".to_string(), // TODO: Generate proper secure code
        },
        inline_assets,
        blob_internal_assets: vec![],
        blob_external_assets: vec![],
    }
}

/// Create a blob memory (large assets stored as blobs)
fn create_blob_memory(
    memory_id: &str,
    _capsule_id: &CapsuleId,
    blob_ref: BlobRef,
    asset_metadata: AssetMetadata,
    now: u64,
    caller: &PersonRef,
) -> Memory {
    let blob_internal_assets = vec![MemoryAssetBlobInternal {
        blob_ref,
        metadata: asset_metadata.clone(),
    }];

    let base = asset_metadata.get_base();
    let created_by = match caller {
        PersonRef::Principal(p) => Some(p.to_text()),
        PersonRef::Opaque(s) => Some(s.clone()),
    };

    Memory {
        id: memory_id.to_string(),
        metadata: MemoryMetadata {
            memory_type: memory_type_from_asset(&asset_metadata),
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
            created_by,
            database_storage_edges: vec![],
        },
        access: MemoryAccess::Private {
            owner_secure_code: "test_code".to_string(), // TODO: Generate proper secure code
        },
        inline_assets: vec![],
        blob_internal_assets,
        blob_external_assets: vec![],
    }
}

/// Create an external memory (assets stored outside ICP)
fn create_external_memory(
    memory_id: &str,
    _capsule_id: &CapsuleId,
    location: StorageEdgeBlobType,
    storage_key: Option<String>,
    url: Option<String>,
    _size: Option<u64>,
    _hash: Option<Vec<u8>>,
    asset_metadata: AssetMetadata,
    now: u64,
    caller: &PersonRef,
) -> Memory {
    let blob_external_assets = vec![MemoryAssetBlobExternal {
        location,
        storage_key: storage_key.unwrap_or_default(),
        url,
        metadata: asset_metadata.clone(),
    }];

    let base = asset_metadata.get_base();
    let created_by = match caller {
        PersonRef::Principal(p) => Some(p.to_text()),
        PersonRef::Opaque(s) => Some(s.clone()),
    };

    Memory {
        id: memory_id.to_string(),
        metadata: MemoryMetadata {
            memory_type: memory_type_from_asset(&asset_metadata),
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
            created_by,
            database_storage_edges: vec![],
        },
        access: MemoryAccess::Private {
            owner_secure_code: "test_code".to_string(), // TODO: Generate proper secure code
        },
        inline_assets: vec![],
        blob_internal_assets: vec![],
        blob_external_assets,
    }
}

// ============================================================================
// CANISTER ENVIRONMENT WRAPPER
// ============================================================================

// CanisterEnv moved to canister module to avoid ICP dependencies in core

// StoreAdapter moved to canister module to avoid ICP dependencies in core

// ============================================================================
// TEST IMPLEMENTATIONS
// ============================================================================

/// Test environment for unit testing
#[derive(Clone)]
pub struct TestEnv {
    pub caller: Principal,
    pub now: u64,
}

impl Env for TestEnv {
    fn caller(&self) -> PersonRef {
        PersonRef::Principal(self.caller)
    }

    fn now(&self) -> u64 {
        self.now
    }
}

/// In-memory store for unit testing
#[derive(Default)]
pub struct InMemoryStore {
    // capsule_id -> (memory_id -> Memory)
    by_capsule: BTreeMap<CapsuleId, BTreeMap<MemoryId, Memory>>,
    // caller -> accessible_capsules
    accessible_capsules: HashMap<PersonRef, Vec<CapsuleId>>,
}

impl InMemoryStore {
    // Removed unused methods: new, add_accessible_capsule
}

impl Store for InMemoryStore {
    // Removed unused method: get_capsule_mut

    fn insert_memory(
        &mut self,
        capsule: &CapsuleId,
        memory: Memory,
    ) -> std::result::Result<(), Error> {
        let cap_map = self
            .by_capsule
            .entry(capsule.clone())
            .or_insert_with(BTreeMap::new);
        // Allow updates - don't check for conflicts
        cap_map.insert(memory.id.clone(), memory);
        Ok(())
    }

    fn get_memory(&self, capsule: &CapsuleId, id: &MemoryId) -> Option<Memory> {
        self.by_capsule.get(capsule)?.get(id).cloned()
    }

    fn delete_memory(
        &mut self,
        capsule: &CapsuleId,
        id: &MemoryId,
    ) -> std::result::Result<(), Error> {
        let Some(cap_map) = self.by_capsule.get_mut(capsule) else {
            return Err(Error::NotFound);
        };
        match cap_map.remove(id) {
            Some(_) => Ok(()),
            None => Err(Error::NotFound),
        }
    }

    fn get_accessible_capsules(&self, caller: &PersonRef) -> Vec<CapsuleId> {
        self.accessible_capsules
            .get(caller)
            .cloned()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AssetMetadata, AssetMetadataBase, AssetType, ImageAssetMetadata};
    use candid::Principal;

    fn create_test_metadata(name: &str, mime: &str, bytes: u64) -> AssetMetadata {
        let base = AssetMetadataBase {
            name: name.to_string(),
            description: Some("Test asset".to_string()),
            tags: vec!["test".to_string()],
            asset_type: AssetType::Original,
            mime_type: mime.to_string(),
            bytes,
            created_at: 1000,
            updated_at: 1000,
            url: None,
            height: None,
            sha256: None,
            storage_key: None,
            processing_error: None,
            asset_location: None,
            width: None,
            processing_status: None,
            bucket: None,
            deleted_at: None,
        };

        AssetMetadata::Image(ImageAssetMetadata {
            base,
            dpi: None,
            color_space: Some("RGB".to_string()),
            exif_data: None,
            compression_ratio: None,
            orientation: None,
        })
    }

    #[test]
    fn test_create_inline_memory_success() {
        let env = TestEnv {
            caller: Principal::anonymous(),
            now: 111_222_333,
        };
        let mut store = InMemoryStore::new();
        let capsule_id = "cap_1".to_string();

        // Add capsule access
        store.add_accessible_capsule(env.caller(), capsule_id.clone());

        let id = memories_create_core(
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
            create_test_metadata("test.jpg", "image/jpeg", 4),
            "idem-1".to_string(),
        )
        .expect("create should succeed");

        assert!(!id.is_empty());
        assert_eq!(id, "mem:cap_1:idem-1");

        // Verify memory was actually created
        let memory = store
            .get_memory(&capsule_id, &id)
            .expect("memory should exist");
        assert_eq!(memory.id, id);
        assert!(!memory.inline_assets.is_empty());
        assert_eq!(memory.inline_assets[0].bytes, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_idempotency_same_id_returned() {
        let env = TestEnv {
            caller: Principal::anonymous(),
            now: 111_222_333,
        };
        let mut store = InMemoryStore::new();
        let capsule_id = "cap_1".to_string();
        let idem = "same".to_string();

        store.add_accessible_capsule(env.caller(), capsule_id.clone());

        // First creation
        let id1 = memories_create_core(
            &env,
            &mut store,
            capsule_id.clone(),
            Some(vec![1, 2, 3]),
            None,
            None,
            None,
            None,
            None,
            None,
            create_test_metadata("test.jpg", "image/jpeg", 3),
            idem.clone(),
        )
        .expect("first create should succeed");

        // Second creation with same idem
        let id2 = memories_create_core(
            &env,
            &mut store,
            capsule_id.clone(),
            Some(vec![1, 2, 3]),
            None,
            None,
            None,
            None,
            None,
            None,
            create_test_metadata("test.jpg", "image/jpeg", 3),
            idem.clone(),
        )
        .expect("second create should succeed");

        assert_eq!(id1, id2);
        assert_eq!(id1, "mem:cap_1:same");
    }

    #[test]
    fn test_unauthorized_capsule_access() {
        let env = TestEnv {
            caller: Principal::anonymous(),
            now: 111_222_333,
        };
        let mut store = InMemoryStore::new();
        let capsule_id = "cap_1".to_string();

        // Don't add capsule access - should fail

        let result = memories_create_core(
            &env,
            &mut store,
            capsule_id,
            Some(vec![1, 2, 3]),
            None,
            None,
            None,
            None,
            None,
            None,
            create_test_metadata("test.jpg", "image/jpeg", 3),
            "idem-1".to_string(),
        );

        assert!(matches!(result, Err(Error::Unauthorized)));
    }

    #[test]
    fn test_invalid_multiple_asset_types() {
        let env = TestEnv {
            caller: Principal::anonymous(),
            now: 111_222_333,
        };
        let mut store = InMemoryStore::new();
        let capsule_id = "cap_1".to_string();

        store.add_accessible_capsule(env.caller(), capsule_id.clone());

        let result = memories_create_core(
            &env,
            &mut store,
            capsule_id,
            Some(vec![1, 2, 3]), // bytes
            Some(BlobRef {
                len: 100,
                locator: "test".to_string(),
                hash: None,
            }), // blob_ref
            None,
            None,
            None,
            None,
            None,
            create_test_metadata("test.jpg", "image/jpeg", 4),
            "idem-1".to_string(),
        );

        assert!(matches!(result, Err(Error::InvalidArgument(_))));
    }

    #[test]
    fn test_memories_read_core_success() {
        let env = TestEnv {
            caller: Principal::anonymous(),
            now: 111_222_333,
        };
        let mut store = InMemoryStore::new();
        let capsule_id = "cap_1".to_string();

        store.add_accessible_capsule(env.caller(), capsule_id.clone());

        // Create a memory first
        let id = memories_create_core(
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
            create_test_metadata("test.jpg", "image/jpeg", 4),
            "idem-1".to_string(),
        )
        .expect("create should succeed");

        // Read it back
        let memory = memories_read_core(&env, &store, id.clone()).expect("read should succeed");

        assert_eq!(memory.id, id);
        assert!(!memory.inline_assets.is_empty());
        assert_eq!(memory.inline_assets[0].bytes, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_memories_read_core_not_found() {
        let env = TestEnv {
            caller: Principal::anonymous(),
            now: 111_222_333,
        };
        let store = InMemoryStore::new();

        let result = memories_read_core(&env, &store, "nonexistent".to_string());
        assert!(matches!(result, Err(Error::NotFound)));
    }

    #[test]
    fn test_memories_update_core_success() {
        let env = TestEnv {
            caller: Principal::anonymous(),
            now: 111_222_333,
        };
        let mut store = InMemoryStore::new();
        let capsule_id = "cap_1".to_string();

        store.add_accessible_capsule(env.caller(), capsule_id.clone());

        // Create a memory first
        let id = memories_create_core(
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
            create_test_metadata("test.jpg", "image/jpeg", 4),
            "idem-1".to_string(),
        )
        .expect("create should succeed");

        // Update it
        let updates = MemoryUpdateData {
            name: Some("Updated Name".to_string()),
            metadata: None,
            access: None,
        };

        memories_update_core(&env, &mut store, id.clone(), updates).expect("update should succeed");

        // Read it back and verify
        let memory = memories_read_core(&env, &store, id.clone()).expect("read should succeed");

        assert_eq!(memory.metadata.title, Some("Updated Name".to_string()));
        assert_eq!(memory.metadata.updated_at, env.now());
    }

    #[test]
    fn test_memories_delete_core_success() {
        let env = TestEnv {
            caller: Principal::anonymous(),
            now: 111_222_333,
        };
        let mut store = InMemoryStore::new();
        let capsule_id = "cap_1".to_string();

        store.add_accessible_capsule(env.caller(), capsule_id.clone());

        // Create a memory first
        let id = memories_create_core(
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
            create_test_metadata("test.jpg", "image/jpeg", 4),
            "idem-1".to_string(),
        )
        .expect("create should succeed");

        // Verify it exists
        assert!(memories_read_core(&env, &store, id.clone()).is_ok());

        // Delete it
        memories_delete_core(&env, &mut store, id.clone()).expect("delete should succeed");

        // Verify it's gone
        let result = memories_read_core(&env, &store, id.clone());
        assert!(matches!(result, Err(Error::NotFound)));
    }

    #[test]
    fn test_memories_delete_core_not_found() {
        let env = TestEnv {
            caller: Principal::anonymous(),
            now: 111_222_333,
        };
        let mut store = InMemoryStore::new();

        let result = memories_delete_core(&env, &mut store, "nonexistent".to_string());
        assert!(matches!(result, Err(Error::NotFound)));
    }

    #[test]
    fn test_property_create_read_roundtrip() {
        let env = TestEnv {
            caller: Principal::anonymous(),
            now: 111_222_333,
        };
        let mut store = InMemoryStore::new();
        let capsule_id = "cap_1".to_string();

        store.add_accessible_capsule(env.caller(), capsule_id.clone());

        // Create memory
        let id = memories_create_core(
            &env,
            &mut store,
            capsule_id.clone(),
            Some(vec![1, 2, 3, 4, 5]),
            None,
            None,
            None,
            None,
            None,
            None,
            create_test_metadata("roundtrip.jpg", "image/jpeg", 5),
            "roundtrip-1".to_string(),
        )
        .expect("create should succeed");

        // Read it back
        let memory = memories_read_core(&env, &store, id.clone()).expect("read should succeed");

        // Verify all properties are preserved
        assert_eq!(memory.id, id);
        assert_eq!(memory.inline_assets[0].bytes, vec![1, 2, 3, 4, 5]);
        assert_eq!(memory.metadata.title, Some("roundtrip.jpg".to_string()));
        assert_eq!(memory.metadata.content_type, "image/jpeg".to_string());
        assert_eq!(memory.metadata.created_at, env.now());
        assert_eq!(memory.metadata.updated_at, env.now());
    }

    #[test]
    fn test_property_create_update_read_roundtrip() {
        let env = TestEnv {
            caller: Principal::anonymous(),
            now: 111_222_333,
        };
        let mut store = InMemoryStore::new();
        let capsule_id = "cap_1".to_string();

        store.add_accessible_capsule(env.caller(), capsule_id.clone());

        // Create memory
        let id = memories_create_core(
            &env,
            &mut store,
            capsule_id.clone(),
            Some(vec![1, 2, 3]),
            None,
            None,
            None,
            None,
            None,
            None,
            create_test_metadata("original.jpg", "image/jpeg", 3),
            "update-roundtrip-1".to_string(),
        )
        .expect("create should succeed");

        // Update it
        let updates = MemoryUpdateData {
            name: Some("Updated Name".to_string()),
            metadata: None,
            access: None,
        };

        memories_update_core(&env, &mut store, id.clone(), updates).expect("update should succeed");

        // Read it back
        let memory = memories_read_core(&env, &store, id.clone()).expect("read should succeed");

        // Verify update was applied
        assert_eq!(memory.metadata.title, Some("Updated Name".to_string()));
        assert_eq!(memory.metadata.updated_at, env.now());
        // Original data should still be there
        assert_eq!(memory.inline_assets[0].bytes, vec![1, 2, 3]);
        assert_eq!(memory.metadata.content_type, "image/jpeg".to_string());
    }

    #[test]
    fn test_property_create_delete_roundtrip() {
        let env = TestEnv {
            caller: Principal::anonymous(),
            now: 111_222_333,
        };
        let mut store = InMemoryStore::new();
        let capsule_id = "cap_1".to_string();

        store.add_accessible_capsule(env.caller(), capsule_id.clone());

        // Create memory
        let id = memories_create_core(
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
            create_test_metadata("delete-test.jpg", "image/jpeg", 4),
            "delete-roundtrip-1".to_string(),
        )
        .expect("create should succeed");

        // Verify it exists
        assert!(memories_read_core(&env, &store, id.clone()).is_ok());

        // Delete it
        memories_delete_core(&env, &mut store, id.clone()).expect("delete should succeed");

        // Verify it's gone
        let result = memories_read_core(&env, &store, id.clone());
        assert!(matches!(result, Err(Error::NotFound)));

        // Verify we can create a new memory with the same ID (idempotency)
        let new_id = memories_create_core(
            &env,
            &mut store,
            capsule_id.clone(),
            Some(vec![5, 6, 7, 8]),
            None,
            None,
            None,
            None,
            None,
            None,
            create_test_metadata("new-memory.jpg", "image/jpeg", 4),
            "delete-roundtrip-1".to_string(), // Same idem
        )
        .expect("create should succeed");

        assert_eq!(new_id, id); // Should get the same ID
    }

    #[test]
    fn test_property_idempotency_multiple_creates() {
        let env = TestEnv {
            caller: Principal::anonymous(),
            now: 111_222_333,
        };
        let mut store = InMemoryStore::new();
        let capsule_id = "cap_1".to_string();
        let idem = "idempotency-test".to_string();

        store.add_accessible_capsule(env.caller(), capsule_id.clone());

        // Create memory multiple times with same idem
        let id1 = memories_create_core(
            &env,
            &mut store,
            capsule_id.clone(),
            Some(vec![1, 2, 3]),
            None,
            None,
            None,
            None,
            None,
            None,
            create_test_metadata("idem1.jpg", "image/jpeg", 3),
            idem.clone(),
        )
        .expect("first create should succeed");

        let id2 = memories_create_core(
            &env,
            &mut store,
            capsule_id.clone(),
            Some(vec![4, 5, 6]), // Different data
            None,
            None,
            None,
            None,
            None,
            None,
            create_test_metadata("idem2.jpg", "image/jpeg", 3), // Different metadata
            idem.clone(),
        )
        .expect("second create should succeed");

        // Should get the same ID (idempotency)
        assert_eq!(id1, id2);

        // Read it back - should have the original data (first create wins)
        let memory = memories_read_core(&env, &store, id1.clone()).expect("read should succeed");

        assert_eq!(memory.inline_assets[0].bytes, vec![1, 2, 3]); // Original data
        assert_eq!(memory.metadata.title, Some("idem1.jpg".to_string())); // Original metadata
    }

    // ============================================================================
    // VALIDATION TESTS
    // ============================================================================

    #[test]
    fn test_asset_consistency_inline_bytes_mismatch() {
        let env = TestEnv {
            caller: Principal::anonymous(),
            now: 111_222_333,
        };
        let mut store = InMemoryStore::new();
        let capsule_id = "cap_1".to_string();

        store.add_accessible_capsule(env.caller(), capsule_id.clone());

        // Create metadata with wrong byte count
        let mut metadata = create_test_metadata("test.jpg", "image/jpeg", 3);
        match &mut metadata {
            AssetMetadata::Image(img) => img.base.bytes = 999, // Wrong size
            _ => panic!("Expected Image metadata"),
        }

        let result = memories_create_core(
            &env,
            &mut store,
            capsule_id,
            Some(vec![1, 2, 3]), // 3 bytes
            None,
            None,
            None,
            None,
            None,
            None,
            metadata,
            "test-idem".to_string(),
        );

        assert!(matches!(result, Err(Error::InvalidArgument(_))));
        if let Err(Error::InvalidArgument(msg)) = result {
            assert!(msg.contains("inline bytes_len != metadata.base.bytes"));
        }
    }

    #[test]
    fn test_asset_consistency_blob_ref_mismatch() {
        let env = TestEnv {
            caller: Principal::anonymous(),
            now: 111_222_333,
        };
        let mut store = InMemoryStore::new();
        let capsule_id = "cap_1".to_string();

        store.add_accessible_capsule(env.caller(), capsule_id.clone());

        // Create metadata with wrong byte count
        let mut metadata = create_test_metadata("test.jpg", "image/jpeg", 3);
        match &mut metadata {
            AssetMetadata::Image(img) => img.base.bytes = 999, // Wrong size
            _ => panic!("Expected Image metadata"),
        }

        let result = memories_create_core(
            &env,
            &mut store,
            capsule_id,
            None,
            Some(BlobRef {
                len: 100, // Different from metadata.bytes
                locator: "test".to_string(),
                hash: None,
            }),
            None,
            None,
            None,
            None,
            None,
            metadata,
            "test-idem".to_string(),
        );

        assert!(matches!(result, Err(Error::InvalidArgument(_))));
        if let Err(Error::InvalidArgument(msg)) = result {
            assert!(msg.contains("blob_ref.len != metadata.base.bytes"));
        }
    }

    #[test]
    fn test_asset_consistency_external_missing_storage_key() {
        let env = TestEnv {
            caller: Principal::anonymous(),
            now: 111_222_333,
        };
        let mut store = InMemoryStore::new();
        let capsule_id = "cap_1".to_string();

        store.add_accessible_capsule(env.caller(), capsule_id.clone());

        let result = memories_create_core(
            &env,
            &mut store,
            capsule_id,
            None,
            None,
            Some(StorageEdgeBlobType::S3),
            None, // Missing storage key
            None,
            Some(100),
            None,
            create_test_metadata("test.jpg", "image/jpeg", 4),
            "test-idem".to_string(),
        );

        assert!(matches!(result, Err(Error::InvalidArgument(_))));
        if let Err(Error::InvalidArgument(msg)) = result {
            assert!(msg.contains("external_storage_key is required"));
        }
    }

    #[test]
    fn test_asset_consistency_external_size_mismatch() {
        let env = TestEnv {
            caller: Principal::anonymous(),
            now: 111_222_333,
        };
        let mut store = InMemoryStore::new();
        let capsule_id = "cap_1".to_string();

        store.add_accessible_capsule(env.caller(), capsule_id.clone());

        // Create metadata with wrong byte count
        let mut metadata = create_test_metadata("test.jpg", "image/jpeg", 3);
        match &mut metadata {
            AssetMetadata::Image(img) => img.base.bytes = 999, // Wrong size
            _ => panic!("Expected Image metadata"),
        }

        let result = memories_create_core(
            &env,
            &mut store,
            capsule_id,
            None,
            None,
            Some(StorageEdgeBlobType::S3),
            Some("test-key".to_string()),
            None,
            Some(100), // Different from metadata.bytes
            None,
            metadata,
            "test-idem".to_string(),
        );

        assert!(matches!(result, Err(Error::InvalidArgument(_))));
        if let Err(Error::InvalidArgument(msg)) = result {
            assert!(msg.contains("external_size != metadata.base.bytes"));
        }
    }

    #[test]
    fn test_memory_type_mapping_from_asset_metadata() {
        use crate::types::NoteAssetMetadata;

        let env = TestEnv {
            caller: Principal::anonymous(),
            now: 111_222_333,
        };
        let mut store = InMemoryStore::new();
        let capsule_id = "cap_1".to_string();

        store.add_accessible_capsule(env.caller(), capsule_id.clone());

        // Test Note metadata
        let note_metadata = AssetMetadata::Note(NoteAssetMetadata {
            base: AssetMetadataBase {
                name: "test.txt".to_string(),
                description: Some("Test note".to_string()),
                tags: vec!["note".to_string()],
                asset_type: AssetType::Original,
                mime_type: "text/plain".to_string(),
                bytes: 3,
                created_at: 1000,
                updated_at: 1000,
                url: None,
                height: None,
                sha256: None,
                storage_key: None,
                processing_error: None,
                asset_location: None,
                width: None,
                processing_status: None,
                bucket: None,
                deleted_at: None,
            },
            format: None,
            language: None,
            word_count: None,
        });

        let id = memories_create_core(
            &env,
            &mut store,
            capsule_id.clone(),
            Some(vec![1, 2, 3]),
            None,
            None,
            None,
            None,
            None,
            None,
            note_metadata,
            "note-test".to_string(),
        )
        .expect("create should succeed");

        let memory = memories_read_core(&env, &store, id).expect("read should succeed");
        assert_eq!(memory.metadata.memory_type, MemoryType::Note);
    }

    #[test]
    fn test_created_by_field_is_set() {
        let env = TestEnv {
            caller: Principal::anonymous(),
            now: 111_222_333,
        };
        let mut store = InMemoryStore::new();
        let capsule_id = "cap_1".to_string();

        store.add_accessible_capsule(env.caller(), capsule_id.clone());

        let id = memories_create_core(
            &env,
            &mut store,
            capsule_id,
            Some(vec![1, 2, 3]),
            None,
            None,
            None,
            None,
            None,
            None,
            create_test_metadata("test.jpg", "image/jpeg", 3),
            "creator-test".to_string(),
        )
        .expect("create should succeed");

        let memory = memories_read_core(&env, &store, id).expect("read should succeed");
        assert!(memory.metadata.created_by.is_some());
        // For Principal::anonymous(), the created_by should be the principal text
        assert_eq!(memory.metadata.created_by, Some("2vxsx-fae".to_string()));
    }
}
