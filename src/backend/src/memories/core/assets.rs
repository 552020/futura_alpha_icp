//! Asset management operations
//!
//! This module contains the core business logic for managing memory assets
//! including cleanup, removal, and listing.

use super::traits::*;
use crate::capsule_acl::CapsuleAcl;
use crate::types::{Error, Memory, MemoryId};

/// Clean up all assets from a memory while preserving the memory record
pub fn memories_cleanup_assets_all_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_id: MemoryId,
) -> std::result::Result<crate::memories::types::AssetCleanupResult, Error> {
    let caller = env.caller();
    let accessible_capsules = store.get_accessible_capsules(&caller);
    for capsule_id in accessible_capsules {
        if let Some(memory) = store.get_memory(&capsule_id, &memory_id) {
            let capsule_access = store
                .get_capsule_for_acl(&capsule_id)
                .ok_or(Error::NotFound)?;
            if !capsule_access.can_write(&caller) {
                return Err(Error::Unauthorized);
            }
            let assets_count = memory.inline_assets.len()
                + memory.blob_internal_assets.len()
                + memory.blob_external_assets.len();
            cleanup_memory_assets(&memory)?;
            let mut updated_memory = memory.clone();
            updated_memory.blob_internal_assets.clear();
            updated_memory.blob_external_assets.clear();
            updated_memory.inline_assets.clear();
            store.insert_memory(&capsule_id, updated_memory)?;
            return Ok(crate::memories::types::AssetCleanupResult {
                memory_id: memory_id.clone(),
                assets_cleaned: assets_count as u32,
                message: format!("Cleaned {} assets from memory", assets_count),
            });
        }
    }
    Err(Error::NotFound)
}

/// Bulk cleanup assets from multiple memories
pub fn memories_cleanup_assets_bulk_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_ids: Vec<MemoryId>,
) -> std::result::Result<crate::memories::types::BulkAssetCleanupResult, Error> {
    let mut cleaned_count = 0;
    let mut failed_count = 0;
    let mut total_assets_cleaned = 0;
    let mut errors = Vec::new();
    for memory_id in memory_ids {
        match memories_cleanup_assets_all_core(env, store, memory_id.clone()) {
            Ok(result) => {
                cleaned_count += 1;
                total_assets_cleaned += result.assets_cleaned;
            }
            Err(e) => {
                failed_count += 1;
                errors.push(format!("Memory {}: {}", memory_id, e));
            }
        }
    }
    Ok(crate::memories::types::BulkAssetCleanupResult {
        cleaned_count,
        failed_count,
        total_assets_cleaned,
        message: format!(
            "Cleaned {} memories, {} failed, {} total assets cleaned",
            cleaned_count, failed_count, total_assets_cleaned
        ),
    })
}

/// Remove a specific asset (inline, internal blob, or external) by its reference
pub fn asset_remove_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_id: MemoryId,
    asset_ref: String,
) -> std::result::Result<crate::memories::types::AssetRemovalResult, Error> {
    let caller = env.caller();
    let accessible_capsules = store.get_accessible_capsules(&caller);
    for capsule_id in accessible_capsules {
        if let Some(mut memory) = store.get_memory(&capsule_id, &memory_id) {
            let capsule_access = store
                .get_capsule_for_acl(&capsule_id)
                .ok_or(Error::NotFound)?;
            if !capsule_access.can_write(&caller) {
                return Err(Error::Unauthorized);
            }
            let mut asset_removed = false;
            if let Some(index) = memory
                .inline_assets
                .iter()
                .enumerate()
                .position(|(i, _)| format!("inline-{}", i) == asset_ref)
            {
                memory.inline_assets.remove(index);
                asset_removed = true;
            }
            if !asset_removed {
                if let Some(index) = memory
                    .blob_internal_assets
                    .iter()
                    .position(|asset| asset.blob_ref.locator == asset_ref)
                {
                    memory.blob_internal_assets.remove(index);
                    asset_removed = true;
                }
            }
            if !asset_removed {
                if let Some(index) = memory
                    .blob_external_assets
                    .iter()
                    .position(|asset| asset.storage_key == asset_ref)
                {
                    memory.blob_external_assets.remove(index);
                    asset_removed = true;
                }
            }
            if asset_removed {
                store.insert_memory(&capsule_id, memory)?;
                return Ok(crate::memories::types::AssetRemovalResult {
                    memory_id: memory_id.clone(),
                    asset_removed: true,
                    message: format!("Asset {} removed from memory", asset_ref),
                });
            } else {
                return Ok(crate::memories::types::AssetRemovalResult {
                    memory_id: memory_id.clone(),
                    asset_removed: false,
                    message: format!("Asset {} not found in memory", asset_ref),
                });
            }
        }
    }
    Err(Error::NotFound)
}

/// Remove a specific inline asset by its index
pub fn asset_remove_inline_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_id: MemoryId,
    asset_index: u32,
) -> std::result::Result<crate::memories::types::AssetRemovalResult, Error> {
    let caller = env.caller();
    let accessible_capsules = store.get_accessible_capsules(&caller);
    for capsule_id in accessible_capsules {
        if let Some(mut memory) = store.get_memory(&capsule_id, &memory_id) {
            let capsule_access = store
                .get_capsule_for_acl(&capsule_id)
                .ok_or(Error::NotFound)?;
            if !capsule_access.can_write(&caller) {
                return Err(Error::Unauthorized);
            }
            let index = asset_index as usize;
            if index < memory.inline_assets.len() {
                memory.inline_assets.remove(index);
                store.insert_memory(&capsule_id, memory)?;
                return Ok(crate::memories::types::AssetRemovalResult {
                    memory_id: memory_id.clone(),
                    asset_removed: true,
                    message: format!("Inline asset at index {} removed", asset_index),
                });
            } else {
                return Ok(crate::memories::types::AssetRemovalResult {
                    memory_id: memory_id.clone(),
                    asset_removed: false,
                    message: format!("Inline asset index {} out of range", asset_index),
                });
            }
        }
    }
    Err(Error::NotFound)
}

/// Remove a specific internal blob asset by its blob reference
pub fn asset_remove_internal_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_id: MemoryId,
    blob_ref: String,
) -> std::result::Result<crate::memories::types::AssetRemovalResult, Error> {
    let caller = env.caller();
    let accessible_capsules = store.get_accessible_capsules(&caller);
    for capsule_id in accessible_capsules {
        if let Some(mut memory) = store.get_memory(&capsule_id, &memory_id) {
            let capsule_access = store
                .get_capsule_for_acl(&capsule_id)
                .ok_or(Error::NotFound)?;
            if !capsule_access.can_write(&caller) {
                return Err(Error::Unauthorized);
            }
            if let Some(index) = memory
                .blob_internal_assets
                .iter()
                .position(|asset| asset.blob_ref.locator == blob_ref)
            {
                memory.blob_internal_assets.remove(index);
                store.insert_memory(&capsule_id, memory)?;
                return Ok(crate::memories::types::AssetRemovalResult {
                    memory_id: memory_id.clone(),
                    asset_removed: true,
                    message: format!("Internal blob asset {} removed", blob_ref),
                });
            } else {
                return Ok(crate::memories::types::AssetRemovalResult {
                    memory_id: memory_id.clone(),
                    asset_removed: false,
                    message: format!("Internal blob asset {} not found", blob_ref),
                });
            }
        }
    }
    Err(Error::NotFound)
}

/// Remove a specific external asset by its storage key
pub fn asset_remove_external_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_id: MemoryId,
    storage_key: String,
) -> std::result::Result<crate::memories::types::AssetRemovalResult, Error> {
    let caller = env.caller();
    let accessible_capsules = store.get_accessible_capsules(&caller);
    for capsule_id in accessible_capsules {
        if let Some(mut memory) = store.get_memory(&capsule_id, &memory_id) {
            let capsule_access = store
                .get_capsule_for_acl(&capsule_id)
                .ok_or(Error::NotFound)?;
            if !capsule_access.can_write(&caller) {
                return Err(Error::Unauthorized);
            }
            if let Some(index) = memory
                .blob_external_assets
                .iter()
                .position(|asset| asset.storage_key == storage_key)
            {
                memory.blob_external_assets.remove(index);
                store.insert_memory(&capsule_id, memory)?;
                return Ok(crate::memories::types::AssetRemovalResult {
                    memory_id: memory_id.clone(),
                    asset_removed: true,
                    message: format!("External asset {} removed", storage_key),
                });
            } else {
                return Ok(crate::memories::types::AssetRemovalResult {
                    memory_id: memory_id.clone(),
                    asset_removed: false,
                    message: format!("External asset {} not found", storage_key),
                });
            }
        }
    }
    Err(Error::NotFound)
}

/// List all assets in a memory
pub fn memories_list_assets_core<E: Env, S: Store>(
    env: &E,
    store: &S,
    memory_id: MemoryId,
) -> std::result::Result<crate::memories::types::MemoryAssetsList, Error> {
    let caller = env.caller();
    let accessible_capsules = store.get_accessible_capsules(&caller);
    for capsule_id in accessible_capsules {
        if let Some(memory) = store.get_memory(&capsule_id, &memory_id) {
            let capsule_access = store
                .get_capsule_for_acl(&capsule_id)
                .ok_or(Error::NotFound)?;
            if !capsule_access.can_read(&caller) {
                return Err(Error::Unauthorized);
            }
            let inline_assets: Vec<String> = memory
                .inline_assets
                .iter()
                .enumerate()
                .map(|(i, _)| format!("inline-{}", i))
                .collect();
            let internal_assets: Vec<String> = memory
                .blob_internal_assets
                .iter()
                .map(|asset| asset.blob_ref.locator.clone())
                .collect();
            let external_assets: Vec<String> = memory
                .blob_external_assets
                .iter()
                .map(|asset| asset.storage_key.clone())
                .collect();
            let total_count = inline_assets.len() + internal_assets.len() + external_assets.len();
            return Ok(crate::memories::types::MemoryAssetsList {
                memory_id: memory_id.clone(),
                inline_assets,
                internal_assets,
                external_assets,
                total_count: total_count as u32,
            });
        }
    }
    Err(Error::NotFound)
}

/// Clean up all assets associated with a memory
pub fn cleanup_memory_assets(_memory: &Memory) -> std::result::Result<(), Error> {
    // Clean up inline assets (they're part of the memory struct, so no external cleanup needed)
    // Clean up internal blob assets (ICP blob store cleanup would go here)
    // Clean up external assets (S3, Vercel, etc. cleanup would go here)

    // For now, we just log what would be cleaned up
    // In a real implementation, this would:
    // 1. Delete ICP blobs for internal assets
    // 2. Delete S3 objects for external assets
    // 3. Update any external storage references

    Ok(())
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

        fn create_memory(
            &mut self,
            id: crate::types::MemoryId,
            capsule_id: crate::types::CapsuleId,
            _owner: crate::types::PersonRef,
        ) {
            let memory = crate::types::Memory {
                id: id.clone(),
                metadata: crate::types::MemoryMetadata {
                    memory_type: crate::types::MemoryType::Image,
                    title: Some("Test Memory".to_string()),
                    description: None,
                    content_type: "image/jpeg".to_string(),
                    created_at: 1_695_000_000_000,
                    updated_at: 1_695_000_000_000,
                    uploaded_at: 1_695_000_000_000,
                    date_of_memory: None,
                    file_created_at: None,
                    parent_folder_id: None,
                    tags: vec![],
                    deleted_at: None,
                    people_in_memory: None,
                    location: None,
                    memory_notes: None,
                    created_by: None,
                    database_storage_edges: vec![],
                },
                access: crate::types::MemoryAccess::Private {
                    owner_secure_code: "test-code".to_string(),
                },
                inline_assets: vec![],
                blob_internal_assets: vec![],
                blob_external_assets: vec![],
            };

            self.memories
                .entry(capsule_id.clone())
                .or_insert_with(HashMap::new)
                .insert(id.clone(), memory);

            // Add memory to capsule
            if let Some(capsule) = self.capsules.get_mut(&capsule_id) {
                capsule.memories.push(id);
            }
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
    fn test_memories_cleanup_assets_all_core_happy_path() {
        let mut store = MockStore::new();
        let owner = crate::types::PersonRef::Principal(
            Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap(),
        );
        let capsule_id = "test-capsule-1".to_string();
        let memory_id = "memory-1".to_string();
        let env = MockEnv {
            caller: owner.clone(),
            now: 1_695_000_000_000,
        };

        // Setup
        store.create_capsule(capsule_id.clone(), owner.clone());
        store.create_memory(memory_id.clone(), capsule_id.clone(), owner);

        // Test cleanup
        let result = memories_cleanup_assets_all_core(&env, &mut store, memory_id.clone());

        match result {
            Ok(cleanup_result) => {
                assert_eq!(cleanup_result.memory_id, memory_id);
                assert_eq!(cleanup_result.assets_cleaned, 0); // No assets in test memory
            }
            Err(e) => panic!("Asset cleanup should succeed: {:?}", e),
        }
    }

    #[test]
    fn test_asset_remove_inline_core_happy_path() {
        let mut store = MockStore::new();
        let owner = crate::types::PersonRef::Principal(
            Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap(),
        );
        let capsule_id = "test-capsule-1".to_string();
        let memory_id = "memory-1".to_string();
        let env = MockEnv {
            caller: owner.clone(),
            now: 1_695_000_000_000,
        };

        // Setup
        store.create_capsule(capsule_id.clone(), owner.clone());
        store.create_memory(memory_id.clone(), capsule_id.clone(), owner);

        // Test remove inline asset
        let result = asset_remove_inline_core(&env, &mut store, memory_id.clone(), 0);

        match result {
            Ok(removal_result) => {
                assert_eq!(removal_result.memory_id, memory_id);
                assert!(!removal_result.asset_removed); // No assets in test memory
            }
            Err(e) => panic!("Asset removal should succeed: {:?}", e),
        }
    }

    #[test]
    fn test_memories_list_assets_core_happy_path() {
        let mut store = MockStore::new();
        let owner = crate::types::PersonRef::Principal(
            Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap(),
        );
        let capsule_id = "test-capsule-1".to_string();
        let memory_id = "memory-1".to_string();
        let env = MockEnv {
            caller: owner.clone(),
            now: 1_695_000_000_000,
        };

        // Setup
        store.create_capsule(capsule_id.clone(), owner.clone());
        store.create_memory(memory_id.clone(), capsule_id.clone(), owner);

        // Test list assets
        let result = memories_list_assets_core(&env, &store, memory_id.clone());

        match result {
            Ok(assets_list) => {
                assert_eq!(assets_list.memory_id, memory_id);
                assert_eq!(assets_list.total_count, 0); // No assets in test memory
            }
            Err(e) => panic!("List assets should succeed: {:?}", e),
        }
    }
}
