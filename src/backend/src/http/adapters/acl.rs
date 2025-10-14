use crate::capsule::domain::{effective_perm_mask, Perm, PrincipalContext};
use crate::http::core_types::Acl;
use crate::memories::core::traits::Store;
use crate::memories::{CanisterEnv, StoreAdapter};
use crate::types::PersonRef;
use candid::Principal;

/// ACL adapter that wraps existing domain logic without importing domain code into HTTP layer
pub struct FuturaAclAdapter;

impl Acl for FuturaAclAdapter {
    fn can_view(&self, memory_id: &str, who: Principal) -> bool {
        // Create PrincipalContext for permission evaluation
        let ctx = PrincipalContext {
            principal: who,
            groups: vec![], // TODO: Get from user system if needed
            link: None,     // TODO: Extract from HTTP request if needed
            now_ns: ic_cdk::api::time(),
        };

        // Use the same pattern as existing memory operations
        let _env = CanisterEnv;
        let store = StoreAdapter;

        // Get all accessible capsules for the caller
        let accessible_capsules = store.get_accessible_capsules(&PersonRef::Principal(who));

        // Search for the memory across all accessible capsules
        for capsule_id in accessible_capsules {
            if let Some(memory) = store.get_memory(&capsule_id, &memory_id.to_string()) {
                // Use existing effective_perm_mask logic
                let perm_mask = effective_perm_mask(&memory, &ctx);
                return (perm_mask & Perm::VIEW.bits()) != 0;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capsule::domain::{
        AccessCondition, AccessEntry, GrantSource, ResourceRole, SharingStatus,
    };
    use crate::memories::types::{Memory, MemoryMetadata, MemoryType};
    use crate::types::PersonRef;
    use candid::Principal;
    use std::collections::HashMap;

    /// Helper function to create a test principal
    fn create_test_principal(id: &str) -> Principal {
        Principal::from_text(id).unwrap_or_else(|_| {
            // If parsing fails, create a Principal from bytes
            let bytes = id.as_bytes();
            let mut principal_bytes = [0u8; 29];
            let len = bytes.len().min(29);
            principal_bytes[..len].copy_from_slice(&bytes[..len]);
            Principal::from_slice(&principal_bytes)
        })
    }

    /// Helper function to create a test memory with owner access
    fn create_test_memory_with_owner(
        memory_id: &str,
        capsule_id: &str,
        owner_principal: Principal,
    ) -> Memory {
        let now = 1234567890; // Mock time for testing

        // Create owner access entry
        let owner_access = AccessEntry {
            id: format!("access_{}", memory_id),
            person_ref: Some(PersonRef::Principal(owner_principal)),
            is_public: false,
            grant_source: GrantSource::System,
            source_id: None,
            role: ResourceRole::Owner,
            perm_mask: (Perm::VIEW | Perm::DOWNLOAD | Perm::SHARE | Perm::MANAGE | Perm::OWN)
                .bits(),
            invited_by_person_ref: None,
            created_at: now,
            updated_at: now,
            condition: AccessCondition::Immediate,
        };

        Memory {
            id: memory_id.to_string(),
            capsule_id: capsule_id.to_string(),
            metadata: MemoryMetadata {
                memory_type: MemoryType::Image,
                title: Some("Test Memory".to_string()),
                description: Some("Test memory for ACL testing".to_string()),
                content_type: "image/jpeg".to_string(),
                created_at: now,
                updated_at: now,
                uploaded_at: now,
                date_of_memory: None,
                file_created_at: None,
                parent_folder_id: None,
                tags: vec!["test".to_string()],
                deleted_at: None,
                people_in_memory: None,
                location: None,
                memory_notes: None,
                created_by: Some(owner_principal.to_text()),
                database_storage_edges: vec![],
                shared_count: 0,
                sharing_status: SharingStatus::Private,
                total_size: 1024,
                asset_count: 1,
            },
            access_entries: vec![owner_access],
            inline_assets: vec![],
            blob_internal_assets: vec![],
            blob_external_assets: vec![],
        }
    }

    /// Helper function to create a test memory with public access
    fn create_test_memory_with_public_access(memory_id: &str, capsule_id: &str) -> Memory {
        let now = 1234567890; // Mock time for testing

        // Create public access entry
        let public_access = AccessEntry {
            id: format!("public_access_{}", memory_id),
            person_ref: None, // None for public access
            is_public: true,
            grant_source: GrantSource::System,
            source_id: None,
            role: ResourceRole::Guest,
            perm_mask: Perm::VIEW.bits(),
            invited_by_person_ref: None,
            created_at: now,
            updated_at: now,
            condition: AccessCondition::Immediate,
        };

        Memory {
            id: memory_id.to_string(),
            capsule_id: capsule_id.to_string(),
            metadata: MemoryMetadata {
                memory_type: MemoryType::Image,
                title: Some("Public Test Memory".to_string()),
                description: Some("Public test memory for ACL testing".to_string()),
                content_type: "image/jpeg".to_string(),
                created_at: now,
                updated_at: now,
                uploaded_at: now,
                date_of_memory: None,
                file_created_at: None,
                parent_folder_id: None,
                tags: vec!["public".to_string()],
                deleted_at: None,
                people_in_memory: None,
                location: None,
                memory_notes: None,
                created_by: None,
                database_storage_edges: vec![],
                shared_count: 0,
                sharing_status: SharingStatus::Public,
                total_size: 1024,
                asset_count: 1,
            },
            access_entries: vec![public_access],
            inline_assets: vec![],
            blob_internal_assets: vec![],
            blob_external_assets: vec![],
        }
    }

    /// Mock store for testing ACL logic
    struct MockStore {
        accessible_capsules: HashMap<PersonRef, Vec<String>>,
        memories: HashMap<(String, String), Memory>, // (capsule_id, memory_id) -> Memory
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                accessible_capsules: HashMap::new(),
                memories: HashMap::new(),
            }
        }

        fn with_accessible_capsules(mut self, person: PersonRef, capsules: Vec<String>) -> Self {
            self.accessible_capsules.insert(person, capsules);
            self
        }

        fn with_memory(mut self, capsule_id: String, memory: Memory) -> Self {
            self.memories
                .insert((capsule_id, memory.id.clone()), memory);
            self
        }

        fn get_accessible_capsules(&self, person: &PersonRef) -> Vec<String> {
            self.accessible_capsules
                .get(person)
                .cloned()
                .unwrap_or_default()
        }

        fn get_memory(&self, capsule_id: &str, memory_id: &str) -> Option<Memory> {
            self.memories
                .get(&(capsule_id.to_string(), memory_id.to_string()))
                .cloned()
        }
    }

    /// Test ACL adapter with mocked store
    struct TestAclAdapter {
        store: MockStore,
    }

    impl TestAclAdapter {
        fn new(store: MockStore) -> Self {
            Self { store }
        }

        fn can_view(&self, memory_id: &str, who: Principal) -> bool {
            // Create PrincipalContext for permission evaluation
            let ctx = PrincipalContext {
                principal: who,
                groups: vec![],
                link: None,
                now_ns: 1234567890, // Mock time for testing
            };

            // Get all accessible capsules for the caller
            let accessible_capsules = self
                .store
                .get_accessible_capsules(&PersonRef::Principal(who));

            // Search for the memory across all accessible capsules
            for capsule_id in accessible_capsules {
                if let Some(memory) = self.store.get_memory(&capsule_id, memory_id) {
                    // Use existing effective_perm_mask logic
                    let perm_mask = effective_perm_mask(&memory, &ctx);
                    return (perm_mask & Perm::VIEW.bits()) != 0;
                }
            }
            false
        }
    }

    #[test]
    fn test_owner_can_view_their_memory() {
        let owner = create_test_principal("owner-principal");
        let memory_id = "test-memory-123";
        let capsule_id = "test-capsule-456";

        let memory = create_test_memory_with_owner(memory_id, capsule_id, owner);

        let store = MockStore::new()
            .with_accessible_capsules(PersonRef::Principal(owner), vec![capsule_id.to_string()])
            .with_memory(capsule_id.to_string(), memory);

        let acl = TestAclAdapter::new(store);

        // Owner should be able to view their memory
        assert!(acl.can_view(memory_id, owner));
    }

    #[test]
    fn test_non_owner_cannot_view_private_memory() {
        let owner = create_test_principal("owner-principal");
        let other_user = create_test_principal("other-principal");
        let memory_id = "test-memory-123";
        let capsule_id = "test-capsule-456";

        let memory = create_test_memory_with_owner(memory_id, capsule_id, owner);

        let store = MockStore::new()
            .with_accessible_capsules(PersonRef::Principal(owner), vec![capsule_id.to_string()])
            .with_memory(capsule_id.to_string(), memory);

        let acl = TestAclAdapter::new(store);

        // Other user should not be able to view private memory
        assert!(!acl.can_view(memory_id, other_user));
    }

    #[test]
    fn test_anyone_can_view_public_memory() {
        let owner = create_test_principal("owner-principal");
        let other_user = create_test_principal("other-principal");
        let memory_id = "public-memory-123";
        let capsule_id = "test-capsule-456";

        let memory = create_test_memory_with_public_access(memory_id, capsule_id);

        let store = MockStore::new()
            .with_accessible_capsules(PersonRef::Principal(owner), vec![capsule_id.to_string()])
            .with_accessible_capsules(
                PersonRef::Principal(other_user),
                vec![capsule_id.to_string()],
            )
            .with_memory(capsule_id.to_string(), memory);

        let acl = TestAclAdapter::new(store);

        // Both owner and other user should be able to view public memory
        assert!(acl.can_view(memory_id, owner));
        assert!(acl.can_view(memory_id, other_user));
    }

    #[test]
    fn test_cannot_view_memory_not_in_accessible_capsules() {
        let owner = create_test_principal("owner-principal");
        let other_user = create_test_principal("other-principal");
        let memory_id = "test-memory-123";
        let capsule_id = "test-capsule-456";

        let memory = create_test_memory_with_owner(memory_id, capsule_id, owner);

        let store = MockStore::new()
            .with_accessible_capsules(PersonRef::Principal(owner), vec![capsule_id.to_string()])
            .with_accessible_capsules(PersonRef::Principal(other_user), vec![]) // No accessible capsules
            .with_memory(capsule_id.to_string(), memory);

        let acl = TestAclAdapter::new(store);

        // Owner should be able to view (has access to capsule)
        assert!(acl.can_view(memory_id, owner));

        // Other user should not be able to view (no access to capsule)
        assert!(!acl.can_view(memory_id, other_user));
    }

    #[test]
    fn test_cannot_view_nonexistent_memory() {
        let owner = create_test_principal("owner-principal");
        let memory_id = "nonexistent-memory-123";
        let capsule_id = "test-capsule-456";

        let store = MockStore::new()
            .with_accessible_capsules(PersonRef::Principal(owner), vec![capsule_id.to_string()]);
        // No memory added to store

        let acl = TestAclAdapter::new(store);

        // Should not be able to view nonexistent memory
        assert!(!acl.can_view(memory_id, owner));
    }

    #[test]
    fn test_memory_in_multiple_capsules() {
        let owner = create_test_principal("owner-principal");
        let memory_id = "shared-memory-123";
        let capsule1 = "capsule-1";
        let capsule2 = "capsule-2";

        let memory1 = create_test_memory_with_owner(memory_id, capsule1, owner);
        let memory2 = create_test_memory_with_owner(memory_id, capsule2, owner);

        let store = MockStore::new()
            .with_accessible_capsules(
                PersonRef::Principal(owner),
                vec![capsule1.to_string(), capsule2.to_string()],
            )
            .with_memory(capsule1.to_string(), memory1)
            .with_memory(capsule2.to_string(), memory2);

        let acl = TestAclAdapter::new(store);

        // Should be able to view memory (found in first accessible capsule)
        assert!(acl.can_view(memory_id, owner));
    }

    #[test]
    fn test_effective_perm_mask_owner_short_circuit() {
        let owner = create_test_principal("owner-principal");
        let memory_id = "test-memory-123";
        let capsule_id = "test-capsule-456";

        let memory = create_test_memory_with_owner(memory_id, capsule_id, owner);

        let ctx = PrincipalContext {
            principal: owner,
            groups: vec![],
            link: None,
            now_ns: 1234567890, // Mock time for testing
        };

        // Test that owner gets all permissions
        let perm_mask = effective_perm_mask(&memory, &ctx);
        assert!(perm_mask & Perm::VIEW.bits() != 0);
        assert!(perm_mask & Perm::DOWNLOAD.bits() != 0);
        assert!(perm_mask & Perm::SHARE.bits() != 0);
        assert!(perm_mask & Perm::MANAGE.bits() != 0);
        assert!(perm_mask & Perm::OWN.bits() != 0);
    }

    #[test]
    fn test_effective_perm_mask_public_access() {
        let user = create_test_principal("user-principal");
        let memory_id = "public-memory-123";
        let capsule_id = "test-capsule-456";

        let memory = create_test_memory_with_public_access(memory_id, capsule_id);

        let ctx = PrincipalContext {
            principal: user,
            groups: vec![],
            link: None,
            now_ns: 1234567890, // Mock time for testing
        };

        // Test that public access gives VIEW permission
        let perm_mask = effective_perm_mask(&memory, &ctx);
        assert!(perm_mask & Perm::VIEW.bits() != 0);
        assert!(perm_mask & Perm::DOWNLOAD.bits() == 0); // No download for public
        assert!(perm_mask & Perm::SHARE.bits() == 0); // No share for public
        assert!(perm_mask & Perm::MANAGE.bits() == 0); // No manage for public
        assert!(perm_mask & Perm::OWN.bits() == 0); // No own for public
    }

    #[test]
    fn test_effective_perm_mask_no_access() {
        let user = create_test_principal("user-principal");
        let owner = create_test_principal("owner-principal");
        let memory_id = "private-memory-123";
        let capsule_id = "test-capsule-456";

        let memory = create_test_memory_with_owner(memory_id, capsule_id, owner);

        let ctx = PrincipalContext {
            principal: user, // Different user
            groups: vec![],
            link: None,
            now_ns: 1234567890, // Mock time for testing
        };

        // Test that user with no access gets no permissions
        let perm_mask = effective_perm_mask(&memory, &ctx);
        assert!(perm_mask == 0);
    }
}
