use super::{CapsuleId, CapsuleStore, Order, Page};
use crate::types::{Capsule, Error};

pub enum Store {
    /// StableBTreeMap backend (persistent production storage)
    Stable(crate::capsule_store::stable::StableStore),
}

impl Store {
    /// Create a new Stable store (for production)
    pub fn new_stable() -> Self {
        Store::Stable(crate::capsule_store::stable::StableStore::new())
    }

    /// Create a new Stable store with fresh memory (for testing)
    #[cfg(test)]
    pub fn new_stable_test() -> Self {
        Store::Stable(crate::capsule_store::stable::StableStore::new_test())
    }

    /// Get the current backend type (for debugging/testing)
    #[allow(dead_code)] // Used in tests
    pub fn backend_type(&self) -> &'static str {
        match self {
            Store::Stable(_) => "StableBTreeMap",
        }
    }
}

// Implement the CapsuleStore trait by delegating to the appropriate backend
impl CapsuleStore for Store {
    fn exists(&self, id: &CapsuleId) -> bool {
        match self {
            Store::Stable(store) => store.exists(id),
        }
    }

    fn get(&self, id: &CapsuleId) -> Option<Capsule> {
        match self {
            Store::Stable(store) => store.get(id),
        }
    }

    fn upsert(&mut self, id: CapsuleId, capsule: Capsule) -> Option<Capsule> {
        match self {
            Store::Stable(store) => store.upsert(id, capsule),
        }
    }

    fn put_if_absent(&mut self, id: CapsuleId, capsule: Capsule) -> std::result::Result<(), Error> {
        match self {
            Store::Stable(store) => store.put_if_absent(id, capsule),
        }
    }

    fn update<F>(&mut self, id: &CapsuleId, f: F) -> std::result::Result<(), Error>
    where
        F: FnOnce(&mut Capsule),
    {
        match self {
            Store::Stable(store) => store.update(id, f),
        }
    }

    fn update_with<R, F>(&mut self, id: &CapsuleId, f: F) -> std::result::Result<R, Error>
    where
        F: FnOnce(&mut Capsule) -> std::result::Result<R, Error>,
    {
        match self {
            Store::Stable(store) => store.update_with(id, f),
        }
    }

    fn remove(&mut self, id: &CapsuleId) -> Option<Capsule> {
        match self {
            Store::Stable(store) => store.remove(id),
        }
    }

    fn find_by_subject(&self, subject: &crate::types::PersonRef) -> Option<Capsule> {
        match self {
            Store::Stable(store) => store.find_by_subject(subject),
        }
    }

    fn list_by_owner(&self, owner: &crate::types::PersonRef) -> Vec<CapsuleId> {
        match self {
            Store::Stable(store) => store.list_by_owner(owner),
        }
    }

    fn get_many(&self, ids: &[CapsuleId]) -> Vec<Capsule> {
        match self {
            Store::Stable(store) => store.get_many(ids),
        }
    }

    fn paginate(&self, after: Option<CapsuleId>, limit: u32, order: Order) -> Page<Capsule> {
        match self {
            Store::Stable(store) => store.paginate(after, limit, order),
        }
    }

    fn count(&self) -> u64 {
        match self {
            Store::Stable(store) => store.count(),
        }
    }

    fn stats(&self) -> (u64, u64, u64) {
        match self {
            Store::Stable(store) => store.stats(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

    #[test]
    fn test_store_backend_type() {
        let stable_store = Store::new_stable_test();
        assert_eq!(stable_store.backend_type(), "StableBTreeMap");
    }

    #[test]
    fn test_store_delegation_stable() {
        let mut store = Store::new_stable_test();

        // Test basic operations delegate correctly
        let id = "test-123".to_string();
        assert!(!store.exists(&id));

        let capsule = create_test_capsule(id.clone());
        let result = store.upsert(id.clone(), capsule);
        assert!(result.is_none()); // Should be None for new item

        assert!(store.exists(&id));
        let retrieved = store.get(&id);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_subject_index_stable_backend() {
        let mut store = Store::new_stable_test();

        // Create a test capsule
        let capsule_id = "test_subject_capsule".to_string();
        let capsule = create_test_capsule(capsule_id.clone());

        // Insert the capsule
        let result = store.upsert(capsule_id.clone(), capsule.clone());
        assert!(result.is_none(), "Should be None for new insertion");

        // Get the subject from the capsule we just created
        let subject = &capsule.subject;

        // Try to find it by subject
        let found = store.find_by_subject(subject);
        assert!(found.is_some(), "Should find capsule by subject");

        let found_capsule = found.unwrap();
        assert_eq!(
            found_capsule.id, capsule_id,
            "Should return the correct capsule"
        );

        eprintln!(
            "✅ Subject index test passed - found capsule: {}",
            found_capsule.id
        );
    }

    #[test]
    #[ignore] // Ignore in normal test runs since it requires IC environment
    fn test_subject_index_stable_backend_production() {
        // This test can only run in IC environment, but let's document what we expect
        let mut store = Store::new_stable();

        // Create a test capsule
        let capsule_id = "test_stable_capsule".to_string();
        let capsule = create_test_capsule(capsule_id.clone());

        // Insert the capsule
        let result = store.upsert(capsule_id.clone(), capsule.clone());
        assert!(result.is_none(), "Should be None for new insertion");

        // Get the subject from the capsule we just created
        let subject = &capsule.subject;

        // Try to find it by subject - this is where the bug manifests
        let found = store.find_by_subject(subject);
        if found.is_none() {
            eprintln!("❌ BUG: Subject index failed to find capsule in stable backend");
            eprintln!("   Capsule ID: {}", capsule_id);
            eprintln!("   Subject: {:?}", subject);
            panic!("Subject index bug reproduced!");
        }

        let found_capsule = found.unwrap();
        assert_eq!(
            found_capsule.id, capsule_id,
            "Should return the correct capsule"
        );

        eprintln!("✅ Stable backend subject index test passed");
    }

    fn create_test_capsule(id: CapsuleId) -> Capsule {
        // Helper function to create a test capsule
        // This would normally be done with proper constructors
        use crate::types::OwnerState;
        use crate::types::{Capsule, PersonRef};
        use std::collections::HashMap;

        let subject = PersonRef::Principal(Principal::from_text("aaaaa-aa").unwrap());
        let mut owners = HashMap::new();
        owners.insert(
            subject.clone(),
            OwnerState {
                since: 1234567890,
                last_activity_at: 1234567890,
            },
        );

        Capsule {
            id,
            subject,
            owners,
            controllers: HashMap::new(),
            connections: HashMap::new(),
            connection_groups: HashMap::new(),
            memories: HashMap::new(),
            galleries: HashMap::new(),
            created_at: 1234567890,
            updated_at: 1234567890,
            bound_to_neon: false,
            inline_bytes_used: 0,
        }
    }
}
