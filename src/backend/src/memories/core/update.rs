//! Memory update operations
//!
//! This module contains the core business logic for updating memories
//! and their metadata.

use super::traits::*;
use crate::capsule_acl::CapsuleAcl;
use crate::types::{Error, MemoryId, MemoryUpdateData};

/// Core memory update function - pure business logic
pub fn memories_update_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_id: MemoryId,
    update_data: MemoryUpdateData,
) -> std::result::Result<(), Error> {
    let caller = env.caller();
    let now = env.now();

    // Find the memory across all accessible capsules
    let accessible_capsules = store.get_accessible_capsules(&caller);

    for capsule_id in accessible_capsules {
        if let Some(mut memory) = store.get_memory(&capsule_id, &memory_id) {
            // Check permissions
            let capsule_access = store
                .get_capsule_for_acl(&capsule_id)
                .ok_or(Error::NotFound)?;

            if !capsule_access.can_write(&caller) {
                return Err(Error::Unauthorized);
            }

            // Apply updates
            if let Some(name) = update_data.name {
                memory.metadata.title = Some(name);
            }

            if let Some(metadata) = update_data.metadata {
                memory.metadata = metadata;
            }

            if let Some(access) = update_data.access {
                memory.access = access;
            }

            // Update timestamp
            memory.metadata.updated_at = now;

            // Store the updated memory
            store.insert_memory(&capsule_id, memory)?;

            return Ok(());
        }
    }

    Err(Error::NotFound)
}

/// Update memory metadata only
pub fn memories_update_metadata_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_id: MemoryId,
    metadata: crate::types::MemoryMetadata,
) -> std::result::Result<(), Error> {
    let caller = env.caller();
    let now = env.now();

    // Find the memory across all accessible capsules
    let accessible_capsules = store.get_accessible_capsules(&caller);

    for capsule_id in accessible_capsules {
        if let Some(mut memory) = store.get_memory(&capsule_id, &memory_id) {
            // Check permissions
            let capsule_access = store
                .get_capsule_for_acl(&capsule_id)
                .ok_or(Error::NotFound)?;

            if !capsule_access.can_write(&caller) {
                return Err(Error::Unauthorized);
            }

            // Update metadata
            memory.metadata = metadata;
            memory.metadata.updated_at = now;

            // Store the updated memory
            store.insert_memory(&capsule_id, memory)?;

            return Ok(());
        }
    }

    Err(Error::NotFound)
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
                    title: Some("Original Title".to_string()),
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
    fn test_memories_update_core_happy_path() {
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

        // Test update
        let update_data = MemoryUpdateData {
            name: Some("Updated Title".to_string()),
            metadata: None,
            access: None,
        };

        let result = memories_update_core(&env, &mut store, memory_id.clone(), update_data);

        match result {
            Ok(()) => {
                // Verify update
                let memory = store.get_memory(&capsule_id, &memory_id).unwrap();
                assert_eq!(memory.metadata.title, Some("Updated Title".to_string()));
                assert!(memory.metadata.updated_at > memory.metadata.created_at);
            }
            Err(e) => panic!("Memory update should succeed: {:?}", e),
        }
    }

    #[test]
    fn test_memories_update_core_not_found() {
        let mut store = MockStore::new();
        let owner = crate::types::PersonRef::Principal(
            Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap(),
        );
        let memory_id = "non-existent-memory".to_string();
        let env = MockEnv {
            caller: owner,
            now: 1_695_000_000_000,
        };

        // Test update non-existent memory
        let update_data = MemoryUpdateData {
            name: Some("Updated Title".to_string()),
            metadata: None,
            access: None,
        };

        let result = memories_update_core(&env, &mut store, memory_id, update_data);

        match result {
            Ok(_) => panic!("Memory update should fail for non-existent memory"),
            Err(crate::types::Error::NotFound) => {} // Expected
            Err(e) => panic!("Expected NotFound error, got: {:?}", e),
        }
    }

    #[test]
    fn test_memories_update_core_unauthorized() {
        let mut store = MockStore::new();
        let owner = crate::types::PersonRef::Principal(
            Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap(),
        );
        let stranger = crate::types::PersonRef::Principal(
            Principal::from_text("rrkah-fqaaa-aaaah-qcaiq-cai").unwrap(),
        );
        let capsule_id = "test-capsule-1".to_string();
        let memory_id = "memory-1".to_string();
        let env = MockEnv {
            caller: stranger,
            now: 1_695_000_000_000,
        };

        // Setup
        store.create_capsule(capsule_id.clone(), owner.clone());
        store.create_memory(memory_id.clone(), capsule_id.clone(), owner);

        // Test update with unauthorized user
        let update_data = MemoryUpdateData {
            name: Some("Updated Title".to_string()),
            metadata: None,
            access: None,
        };

        let result = memories_update_core(&env, &mut store, memory_id, update_data);

        match result {
            Ok(_) => panic!("Memory update should fail for unauthorized user"),
            Err(crate::types::Error::Unauthorized) => {} // Expected
            Err(e) => panic!("Expected Unauthorized error, got: {:?}", e),
        }
    }
}
