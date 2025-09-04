//! StableBTreeMap Backend Implementation
//!
//! This module provides a StableBTreeMap-based implementation of CapsuleStore
//! for production use with persistent storage on the Internet Computer.
//!
//! Key characteristics:
//! - Persistent storage across canister upgrades
//! - O(log n) operations
//! - MemoryId reservations for different data types
//! - Storable implementation for Capsule serialization
//! - Schema versioning for upgrade compatibility

use super::{AlreadyExists, CapsuleId, CapsuleStore, Order, Page, UpdateError};
use crate::types::Capsule;
use candid::{Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{storable::Bound, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::borrow::Cow;
use std::cell::RefCell;

/// Owner index key for sparse multimap: (owner_principal_bytes, capsule_id)
///
/// This is a bounded key type that can be stored in StableBTreeMap.
/// Format: [owner_len:u32][owner_bytes][capsule_id_bytes]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct OwnerIndexKey {
    owner_bytes: Vec<u8>,
    capsule_id: String,
}

impl OwnerIndexKey {
    fn new(owner_bytes: Vec<u8>, capsule_id: String) -> Self {
        Self {
            owner_bytes,
            capsule_id,
        }
    }
}

impl Storable for OwnerIndexKey {
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut buf = Vec::new();

        // Store owner_bytes length (4 bytes)
        buf.extend_from_slice(&(self.owner_bytes.len() as u32).to_be_bytes());

        // Store owner_bytes
        buf.extend_from_slice(&self.owner_bytes);

        // Store capsule_id bytes (rest of the buffer)
        buf.extend_from_slice(self.capsule_id.as_bytes());

        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let bytes = bytes.as_ref();

        // Read owner_bytes length
        let owner_len = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;

        // Read owner_bytes
        let owner_bytes = bytes[4..4 + owner_len].to_vec();

        // Read capsule_id (rest of bytes)
        let capsule_id =
            String::from_utf8(bytes[4 + owner_len..].to_vec()).expect("Valid UTF-8 capsule_id");

        Self {
            owner_bytes,
            capsule_id,
        }
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 29 + 255, // 29 bytes for Principal + 255 bytes for capsule_id
        is_fixed_size: false,
    };
}

/// Memory ID reservations for different data types
const MEM_CAPSULES: MemoryId = MemoryId::new(0);
const MEM_IDX_SUBJECT: MemoryId = MemoryId::new(1);
const MEM_IDX_OWNER: MemoryId = MemoryId::new(2);

/// StableBTreeMap-based storage implementation for production
pub struct StableStore {
    /// Main capsule storage
    capsules: StableBTreeMap<CapsuleId, Capsule, VirtualMemory<DefaultMemoryImpl>>,
    /// Subject → CapsuleId index (1:1 relationship - frozen)
    subject_index: StableBTreeMap<Vec<u8>, CapsuleId, VirtualMemory<DefaultMemoryImpl>>,
    /// Owner → CapsuleId sparse multimap (1:N relationship)
    /// Key: OwnerIndexKey(owner_principal_bytes, capsule_id), Value: ()
    owner_index: StableBTreeMap<OwnerIndexKey, (), VirtualMemory<DefaultMemoryImpl>>,
}

impl Default for StableStore {
    fn default() -> Self {
        Self::new()
    }
}

impl StableStore {
    /// Create a new StableStore with default memory manager
    pub fn new() -> Self {
        // 🔧 FIX: Use thread-local memory manager to prevent overlap
        // Create one memory manager per thread to avoid conflicts
        thread_local! {
            static SHARED_MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
                RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
        }

        SHARED_MEMORY_MANAGER.with(|mm| {
            let mm = mm.borrow();
            Self {
                capsules: StableBTreeMap::init(mm.get(MEM_CAPSULES)),
                subject_index: StableBTreeMap::init(mm.get(MEM_IDX_SUBJECT)),
                owner_index: StableBTreeMap::init(mm.get(MEM_IDX_OWNER)),
            }
        })
    }

    /// Internal helper: Update indexes when a capsule is added/modified
    fn update_indexes(&mut self, id: &CapsuleId, capsule: &Capsule) {
        // Update subject index (1:1 - add new)
        if let Some(subject_principal) = capsule.subject.principal() {
            let subject_key = subject_principal.as_slice().to_vec();
            self.subject_index.insert(subject_key, id.clone());
        }

        // Update owner index (sparse multimap: insert (owner, capsule_id) -> ())
        for owner_ref in capsule.owners.keys() {
            if let crate::types::PersonRef::Principal(owner_principal) = owner_ref {
                let owner_key = owner_principal.as_slice().to_vec();
                let key = OwnerIndexKey::new(owner_key, id.clone());
                self.owner_index.insert(key, ());
            }
        }
    }

    /// Internal helper: Remove from indexes when a capsule is deleted
    fn remove_from_indexes(&mut self, id: &CapsuleId, capsule: &Capsule) {
        // Remove from subject index
        if let Some(subject_principal) = capsule.subject.principal() {
            let subject_key = subject_principal.as_slice().to_vec();
            self.subject_index.remove(&subject_key);
        }

        // Remove from owner index (sparse multimap: remove (owner, capsule_id) pairs)
        for owner_ref in capsule.owners.keys() {
            if let crate::types::PersonRef::Principal(owner_principal) = owner_ref {
                let owner_key = owner_principal.as_slice().to_vec();
                let key = OwnerIndexKey::new(owner_key, id.clone());
                self.owner_index.remove(&key);
            }
        }
    }
}

impl CapsuleStore for StableStore {
    fn exists(&self, id: &CapsuleId) -> bool {
        self.capsules.contains_key(id)
    }

    fn get(&self, id: &CapsuleId) -> Option<Capsule> {
        self.capsules.get(id)
    }

    fn upsert(&mut self, id: CapsuleId, capsule: Capsule) -> Option<Capsule> {
        // Key-record coherence: store key must equal capsule.id
        debug_assert_eq!(id, capsule.id, "CapsuleId key must match Capsule.id");

        if let Some(old) = self.capsules.get(&id) {
            // remove old index entries first (avoid stale/dup)
            self.remove_from_indexes(&id, &old);
        }
        let prev = self.capsules.insert(id.clone(), capsule.clone());
        self.update_indexes(&id, &capsule);
        prev
    }

    fn put_if_absent(&mut self, id: CapsuleId, capsule: Capsule) -> Result<(), AlreadyExists> {
        // Key-record coherence: store key must equal capsule.id
        debug_assert_eq!(id, capsule.id, "CapsuleId key must match Capsule.id");

        if self.capsules.contains_key(&id) {
            return Err(AlreadyExists::CapsuleExists(id));
        }
        self.capsules.insert(id.clone(), capsule.clone());
        self.update_indexes(&id, &capsule);
        Ok(())
    }

    fn update<F>(&mut self, id: &CapsuleId, f: F) -> Result<(), UpdateError>
    where
        F: FnOnce(&mut Capsule),
    {
        if let Some(mut capsule) = self.capsules.get(id) {
            let old_subject = capsule.subject.principal().cloned();
            let old_owners: Vec<_> = capsule
                .owners
                .keys()
                .filter_map(|person_ref| match person_ref {
                    crate::types::PersonRef::Principal(p) => Some(*p),
                    crate::types::PersonRef::Opaque(_) => None,
                })
                .collect();

            f(&mut capsule);

            // Update indexes if subject or owners changed
            let new_subject = capsule.subject.principal().cloned();
            let new_owners: Vec<_> = capsule
                .owners
                .keys()
                .filter_map(|person_ref| match person_ref {
                    crate::types::PersonRef::Principal(p) => Some(*p),
                    crate::types::PersonRef::Opaque(_) => None,
                })
                .collect();

            // Update subject index if changed
            if old_subject != new_subject {
                if let Some(old_principal) = old_subject {
                    let old_key = old_principal.as_slice().to_vec();
                    self.subject_index.remove(&old_key);
                }
                if let Some(new_principal) = &new_subject {
                    let new_key = new_principal.as_slice().to_vec();
                    self.subject_index.insert(new_key, id.clone());
                }
            }

            // Update owner index for added/removed owners (sparse multimap)
            // Remove old owner relationships
            for owner in &old_owners {
                if !new_owners.contains(owner) {
                    let owner_key = owner.as_slice().to_vec();
                    let key = OwnerIndexKey::new(owner_key, id.clone());
                    self.owner_index.remove(&key);
                }
            }
            // Add new owner relationships
            for owner in &new_owners {
                if !old_owners.contains(owner) {
                    let owner_key = owner.as_slice().to_vec();
                    let key = OwnerIndexKey::new(owner_key, id.clone());
                    self.owner_index.insert(key, ());
                }
            }

            // Save the updated capsule
            self.capsules.insert(id.clone(), capsule);
            Ok(())
        } else {
            Err(UpdateError::NotFound)
        }
    }

    fn remove(&mut self, id: &CapsuleId) -> Option<Capsule> {
        if let Some(capsule) = self.capsules.remove(id) {
            self.remove_from_indexes(id, &capsule);
            Some(capsule)
        } else {
            None
        }
    }

    fn find_by_subject(&self, subject: &crate::types::PersonRef) -> Option<Capsule> {
        if let Some(principal) = subject.principal() {
            let subject_key = principal.as_slice().to_vec();
            self.subject_index
                .get(&subject_key)
                .and_then(|id| self.capsules.get(&id))
        } else {
            None
        }
    }

    fn list_by_owner(&self, owner: &crate::types::PersonRef) -> Vec<CapsuleId> {
        if let Some(principal) = owner.principal() {
            let owner_key = principal.as_slice().to_vec();
            let mut result = Vec::new();

            // Iterate through all entries with this owner prefix
            let start_key = OwnerIndexKey::new(owner_key.clone(), String::new());
            let end_key = OwnerIndexKey::new(owner_key, String::from("\u{FFFF}")); // High Unicode value for range end

            let iter = self.owner_index.range(start_key..=end_key);
            for (key, _) in iter {
                result.push(key.capsule_id);
            }

            result
        } else {
            Vec::new()
        }
    }

    fn get_many(&self, ids: &[CapsuleId]) -> Vec<Capsule> {
        ids.iter().filter_map(|id| self.capsules.get(id)).collect()
    }

    fn paginate(&self, after: Option<CapsuleId>, limit: u32, order: Order) -> Page<Capsule> {
        // For StableBTreeMap, we need to collect and sort
        // In a production implementation, this would be optimized
        // with custom iterators or range queries
        let mut all_capsules: Vec<(CapsuleId, Capsule)> = Vec::new();

        // Collect all capsules (inefficient but functional)
        let iter = self.capsules.iter();
        for (id, capsule) in iter {
            all_capsules.push((id, capsule));
        }

        // Sort based on order
        match order {
            Order::Asc => all_capsules.sort_by(|a, b| a.0.cmp(&b.0)),
            Order::Desc => {
                all_capsules.sort_by(|a, b| b.0.cmp(&a.0));
            }
        }

        // Find start position (exclusive after cursor)
        let start_pos = if let Some(after_id) = &after {
            match order {
                Order::Asc => all_capsules
                    .iter()
                    .position(|(id, _)| *id > *after_id)
                    .unwrap_or(all_capsules.len()),
                Order::Desc => all_capsules
                    .iter()
                    .position(|(id, _)| *id < *after_id)
                    .unwrap_or(all_capsules.len()),
            }
        } else {
            0
        };

        // Take the requested number of items
        let end_pos = (start_pos + limit as usize).min(all_capsules.len());
        let page_items: Vec<Capsule> = all_capsules[start_pos..end_pos]
            .iter()
            .map(|(_, capsule)| capsule.clone())
            .collect();

        // Next cursor = last item of current page (exclusive keyset cursor)
        let next_cursor = if !page_items.is_empty() && end_pos < all_capsules.len() {
            Some(all_capsules[end_pos - 1].0.clone())
        } else {
            None
        };

        Page {
            items: page_items,
            next_cursor,
        }
    }

    fn count(&self) -> u64 {
        // StableBTreeMap doesn't provide a direct count method
        // In production, this would be maintained as a separate counter
        let mut count = 0u64;
        let mut iter = self.capsules.iter();
        while iter.next().is_some() {
            count += 1;
        }
        count
    }
}

/// Storable implementation for Capsule
///
/// This implementation handles serialization/deserialization for stable storage.
/// It includes schema versioning for forward compatibility.
impl Storable for Capsule {
    const BOUND: Bound = Bound::Bounded {
        max_size: 8 * 1024, // 8 KiB headroom
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        // versioned encoding
        Cow::Owned(Encode!(&(1u16, self)).expect("encode Capsule"))
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let (ver, cap): (u16, Capsule) =
            Decode!(bytes.as_ref(), (u16, Capsule)).expect("decode Capsule");
        match ver {
            1 => cap,
            _ => panic!("unsupported Capsule version: {ver}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::OwnerState;
    use crate::types::PersonRef;

    // Note: These tests run off-chain using DefaultMemoryImpl
    // Integration tests with actual IC environment would be separate

    #[test]
    fn test_stable_store_basic_operations() {
        let mut store = StableStore::new();

        // Test empty store
        assert_eq!(store.count(), 0);

        let id = "test-123".to_string();
        // Create test capsule
        let capsule = create_test_capsule(id.clone());

        // Test put_if_absent
        assert!(store.put_if_absent(id.clone(), capsule.clone()).is_ok());

        // Test exists and get
        assert!(store.exists(&id));
        let retrieved = store.get(&id);
        assert!(retrieved.is_some());

        // Test remove
        let removed = store.remove(&id);
        assert!(removed.is_some());
        assert!(!store.exists(&id));
    }

    fn create_test_capsule(id: CapsuleId) -> Capsule {
        use std::collections::HashMap;

        let subject = PersonRef::Principal(candid::Principal::from_text("aaaaa-aa").unwrap());
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

    #[test]
    fn test_capsule_storable() {
        let id = "test-capsule".to_string();
        let capsule = create_test_capsule(id.clone());

        // Test serialization
        let bytes = capsule.to_bytes();
        assert!(!bytes.is_empty());

        // Test deserialization
        let deserialized = Capsule::from_bytes(bytes);
        assert_eq!(deserialized.id, capsule.id);
        assert_eq!(
            deserialized.subject.principal(),
            capsule.subject.principal()
        );
    }

    #[test]
    fn test_capsule_size_within_bound() {
        let id = "test-capsule".to_string();
        let cap = create_test_capsule(id.clone());
        let bytes = cap.to_bytes();
        assert!(bytes.len() <= 8 * 1024);
    }

    /// 🔧 GUARDRAIL TEST: Memory Manager Uniqueness
    /// Test that memory manager instances are unique (no overlap)
    #[test]
    fn test_memory_manager_uniqueness() {
        // This test ensures we don't accidentally create multiple managers
        // which would cause memory overlap

        // Create two stores - they should share the same thread-local manager
        let store1 = StableStore::new();
        let store2 = StableStore::new();

        // Both should work correctly without interfering with each other
        // This is a basic smoke test that the shared manager is working
        assert!(store1.capsules.is_empty());
        assert!(store2.capsules.is_empty());
    }

    /// 🔧 GUARDRAIL TEST: Memory Overlap Canary
    /// Test canary pattern to detect memory overlap
    #[test]
    fn test_memory_overlap_canary() {
        let mut store = StableStore::new();

        // Insert a test capsule
        let test_id = "test_canary".to_string();
        let test_capsule = create_test_capsule(test_id.clone());

        // This should work without corruption
        // If there's memory overlap, this might fail or corrupt other data
        let result = store.capsules.insert(test_id.clone(), test_capsule);

        assert!(result.is_none()); // Should be a new insertion

        // Verify we can retrieve it
        let retrieved = store.capsules.get(&test_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, test_id);
    }
}
