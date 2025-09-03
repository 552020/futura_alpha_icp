use crate::types::{Capsule, CapsuleId, PersonRef};
use candid::Principal;
use ic_stable_structures::{memory_manager::VirtualMemory, DefaultMemoryImpl, StableBTreeMap};
use std::collections::HashMap;

// ============================================================================
// RECOMMENDED PATTERN: Boxed Iterator (Object-Safe, Clean API)
// ============================================================================

/// Memory type alias for stable storage
pub type Memory = VirtualMemory<DefaultMemoryImpl>;

/// Object-safe trait for capsule storage with boxed iterators
pub trait CapsuleStore {
    /// Get a capsule by ID
    fn get(&self, id: &CapsuleId) -> Option<Capsule>;

    /// Put a capsule (returns previous value if any)
    fn put(&mut self, id: CapsuleId, capsule: Capsule) -> Option<Capsule>;

    /// Remove a capsule by ID
    fn remove(&mut self, id: &CapsuleId) -> Option<Capsule>;

    /// Get an object-safe iterator over all capsules
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (CapsuleId, Capsule)> + 'a>;

    /// Find capsule by subject (convenience method)
    fn find_by_subject(&self, subj: &PersonRef) -> Option<Capsule> {
        self.iter().find_map(|(_, c)| if c.subject == *subj { Some(c) } else { None })
    }

    /// Check if subject owns any capsules
    fn subject_owns_capsules(&self, subj: &PersonRef) -> bool {
        self.iter().any(|(_, c)| c.subject == *subj)
    }
}

// ============================================================================
// STABLE BTREE MAP IMPLEMENTATION
// ============================================================================

pub struct StableCapsuleStore {
    capsules: StableBTreeMap<String, Capsule, Memory>,
}

impl StableCapsuleStore {
    pub fn new(memory: Memory) -> Self {
        Self {
            capsules: StableBTreeMap::init(memory),
        }
    }
}

impl CapsuleStore for StableCapsuleStore {
    fn get(&self, id: &CapsuleId) -> Option<Capsule> {
        self.capsules.get(id)
    }

    fn put(&mut self, id: CapsuleId, capsule: Capsule) -> Option<Capsule> {
        self.capsules.insert(id, capsule)
    }

    fn remove(&mut self, id: &CapsuleId) -> Option<Capsule> {
        self.capsules.remove(id)
    }

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (CapsuleId, Capsule)> + 'a> {
        // For stable maps, we need to collect into a vector first
        // This is necessary because StableBTreeMap iterators have complex lifetimes
        let items: Vec<_> = self.capsules.iter().collect();
        Box::new(items.into_iter())
    }
}

// ============================================================================
// HASHMAP IMPLEMENTATION (for testing)
// ============================================================================

pub struct HashMapCapsuleStore {
    capsules: HashMap<String, Capsule>,
}

impl HashMapCapsuleStore {
    pub fn new() -> Self {
        Self {
            capsules: HashMap::new(),
        }
    }
}

impl CapsuleStore for HashMapCapsuleStore {
    fn get(&self, id: &CapsuleId) -> Option<Capsule> {
        self.capsules.get(id).cloned()
    }

    fn put(&mut self, id: CapsuleId, capsule: Capsule) -> Option<Capsule> {
        self.capsules.insert(id, capsule)
    }

    fn remove(&mut self, id: &CapsuleId) -> Option<Capsule> {
        self.capsules.remove(id)
    }

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (CapsuleId, Capsule)> + 'a> {
        // Clone the data for the iterator (acceptable for testing)
        let items: Vec<_> = self
            .capsules
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        Box::new(items.into_iter())
    }
}

// ============================================================================
// RUNTIME POLYMORPHISM WITH DYN
// ============================================================================

/// Enum wrapper for runtime backend selection
pub enum CapsuleStoreBackend {
    Stable(StableCapsuleStore),
    HashMap(HashMapCapsuleStore),
}

impl CapsuleStore for CapsuleStoreBackend {
    fn get(&self, id: &CapsuleId) -> Option<Capsule> {
        match self {
            CapsuleStoreBackend::Stable(store) => store.get(id),
            CapsuleStoreBackend::HashMap(store) => store.get(id),
        }
    }

    fn put(&mut self, id: CapsuleId, capsule: Capsule) -> Option<Capsule> {
        match self {
            CapsuleStoreBackend::Stable(store) => store.put(id, capsule),
            CapsuleStoreBackend::HashMap(store) => store.put(id, capsule),
        }
    }

    fn remove(&mut self, id: &CapsuleId) -> Option<Capsule> {
        match self {
            CapsuleStoreBackend::Stable(store) => store.remove(id),
            CapsuleStoreBackend::HashMap(store) => store.remove(id),
        }
    }

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (CapsuleId, Capsule)> + 'a> {
        match self {
            CapsuleStoreBackend::Stable(store) => store.iter(),
            CapsuleStoreBackend::HashMap(store) => store.iter(),
        }
    }
}

// ============================================================================
// UPDATED MEMORY MODULE INTEGRATION
// ============================================================================

/// Use this function instead of with_capsules for trait-based access
pub fn with_capsule_store<F, R>(f: F) -> R
where
    F: FnOnce(&dyn CapsuleStore) -> R,
{
    // Use HashMap for now during transition - switch to Stable later
    let store = HashMapCapsuleStore::new();
    f(&store)
}

/// Use this function for mutable access
pub fn with_capsule_store_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut dyn CapsuleStore) -> R,
{
    // Use HashMap for now during transition - switch to Stable later
    let mut store = HashMapCapsuleStore::new();
    f(&mut store)
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Capsule, PersonRef};

    #[test]
    fn test_hashmap_capsule_store_basic_operations() {
        let mut store = HashMapCapsuleStore::new();
        let capsule_id = "test-capsule-123".to_string();
        let subject = PersonRef::Principal(candid::Principal::from_text("aaaaa-aa").unwrap());
        let capsule = Capsule {
            id: capsule_id.clone(),
            subject: subject.clone(),
            owners: std::collections::HashMap::new(),
            controllers: std::collections::HashMap::new(),
            connections: std::collections::HashMap::new(),
            connection_groups: std::collections::HashMap::new(),
            memories: std::collections::HashMap::new(),
            galleries: std::collections::HashMap::new(),
            created_at: 1234567890,
            updated_at: 1234567890,
            bound_to_neon: false,
        };

        // Test put and get
        let result = store.put(capsule_id.clone(), capsule.clone());
        assert!(result.is_none()); // Should be None since it's a new capsule

        let retrieved = store.get(&capsule_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, capsule_id);

        // Test find_by_subject
        let found = store.find_by_subject(&subject);
        assert!(found.is_some());
        assert_eq!(found.unwrap().subject, subject);

        // Test iteration
        let count = store.iter().count();
        assert_eq!(count, 1);

        // Test remove
        let removed = store.remove(&capsule_id);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, capsule_id);

        // Verify it's gone
        let retrieved_after_remove = store.get(&capsule_id);
        assert!(retrieved_after_remove.is_none());
    }

    #[test]
    fn test_trait_object_safety() {
        // This test verifies that we can use dyn CapsuleStore
        let store = HashMapCapsuleStore::new();
        let store_ref: &dyn CapsuleStore = &store;

        // Test that trait object methods work
        let count = store_ref.iter().count();
        assert_eq!(count, 0);

        let non_existent = store_ref.get(&"non-existent".to_string());
        assert!(non_existent.is_none());
    }

    #[test]
    fn test_runtime_polymorphism() {
        // Test the enum wrapper for runtime polymorphism
        let hashmap_store = HashMapCapsuleStore::new();
        let mut backend = CapsuleStoreBackend::HashMap(hashmap_store);

        // Use it through the enum
        let initial_count = backend.iter().count();
        assert_eq!(initial_count, 0);

        // This demonstrates that we can switch backends at runtime
        match &backend {
            CapsuleStoreBackend::HashMap(_) => {
                // Currently using HashMap, could switch to Stable here
            }
            CapsuleStoreBackend::Stable(_) => {
                // Currently using Stable, could switch to HashMap here
            }
        }
    }
}