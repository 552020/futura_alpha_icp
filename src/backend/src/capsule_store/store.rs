//! Store Enum - Backend Selection and Delegation Logic
//!
//! This module provides the main Store enum that can switch between
//! HashMap (fast testing) and StableBTreeMap (production) backends.
//!
//! The enum delegates all operations to the appropriate backend implementation,
//! providing a clean runtime switch between storage engines.

use super::{AlreadyExists, CapsuleId, CapsuleStore, Order, Page, UpdateError};
use crate::types::Capsule;

/// Main storage enum that can switch between backends at runtime
///
/// This enum provides runtime polymorphism without trait objects,
/// allowing the same code to work with different storage backends.
pub enum Store {
    /// HashMap backend (fast for testing)
    Hash(crate::capsule_store::hash::HashStore),
    /// StableBTreeMap backend (persistent production storage)
    Stable(crate::capsule_store::stable::StableStore),
}

impl Store {
    /// Create a new HashMap store (for testing)
    pub fn new_hash() -> Self {
        Store::Hash(crate::capsule_store::hash::HashStore::new())
    }

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
    pub fn backend_type(&self) -> &'static str {
        match self {
            Store::Hash(_) => "HashMap",
            Store::Stable(_) => "StableBTreeMap",
        }
    }
}

// Implement the CapsuleStore trait by delegating to the appropriate backend
impl CapsuleStore for Store {
    fn exists(&self, id: &CapsuleId) -> bool {
        match self {
            Store::Hash(store) => store.exists(id),
            Store::Stable(store) => store.exists(id),
        }
    }

    fn get(&self, id: &CapsuleId) -> Option<Capsule> {
        match self {
            Store::Hash(store) => store.get(id),
            Store::Stable(store) => store.get(id),
        }
    }

    fn upsert(&mut self, id: CapsuleId, capsule: Capsule) -> Option<Capsule> {
        match self {
            Store::Hash(store) => store.upsert(id, capsule),
            Store::Stable(store) => store.upsert(id, capsule),
        }
    }

    fn put_if_absent(&mut self, id: CapsuleId, capsule: Capsule) -> Result<(), AlreadyExists> {
        match self {
            Store::Hash(store) => store.put_if_absent(id, capsule),
            Store::Stable(store) => store.put_if_absent(id, capsule),
        }
    }

    fn update<F>(&mut self, id: &CapsuleId, f: F) -> Result<(), UpdateError>
    where
        F: FnOnce(&mut Capsule),
    {
        match self {
            Store::Hash(store) => store.update(id, f),
            Store::Stable(store) => store.update(id, f),
        }
    }

    fn remove(&mut self, id: &CapsuleId) -> Option<Capsule> {
        match self {
            Store::Hash(store) => store.remove(id),
            Store::Stable(store) => store.remove(id),
        }
    }

    fn find_by_subject(&self, subject: &crate::types::PersonRef) -> Option<Capsule> {
        match self {
            Store::Hash(store) => store.find_by_subject(subject),
            Store::Stable(store) => store.find_by_subject(subject),
        }
    }

    fn list_by_owner(&self, owner: &crate::types::PersonRef) -> Vec<CapsuleId> {
        match self {
            Store::Hash(store) => store.list_by_owner(owner),
            Store::Stable(store) => store.list_by_owner(owner),
        }
    }

    fn get_many(&self, ids: &[CapsuleId]) -> Vec<Capsule> {
        match self {
            Store::Hash(store) => store.get_many(ids),
            Store::Stable(store) => store.get_many(ids),
        }
    }

    fn paginate(&self, after: Option<CapsuleId>, limit: u32, order: Order) -> Page<Capsule> {
        match self {
            Store::Hash(store) => store.paginate(after, limit, order),
            Store::Stable(store) => store.paginate(after, limit, order),
        }
    }

    fn count(&self) -> u64 {
        match self {
            Store::Hash(store) => store.count(),
            Store::Stable(store) => store.count(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

    #[test]
    fn test_store_backend_type() {
        let hash_store = Store::new_hash();
        assert_eq!(hash_store.backend_type(), "HashMap");

        // Note: Stable store creation would require IC environment
        // We'll test it in integration tests
    }

    #[test]
    fn test_store_delegation_hash() {
        let mut store = Store::new_hash();

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
    fn test_subject_index_hash_backend() {
        let mut store = Store::new_hash();

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
    fn test_subject_index_stable_backend() {
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
        }
    }
}
