/*! HashMap Backend Implementation - LEGACY CODE (COMMENTED OUT)
//!
//! This module provides a HashMap-based implementation of CapsuleStore
//! for fast testing and development. It implements all the same operations
//! as the stable backend but uses in-memory HashMap storage.
//!
//! Key characteristics:
//! - Fast operations (O(1) average case)
//! - In-memory only (data lost on restart)
//! - Great for unit tests and CI
//! - No persistence or indexing complexity

use super::{AlreadyExists, CapsuleId, CapsuleStore, Order, Page, UpdateError};
use crate::types::Capsule;
use candid::Principal;
use std::collections::HashMap;

/// HashMap-based storage implementation for testing
#[derive(Debug, Clone)]
pub struct HashStore {
    /// Main capsule storage
    capsules: HashMap<CapsuleId, Capsule>,
    /// Subject → CapsuleId index (1:1 relationship - frozen)
    subject_index: HashMap<Principal, CapsuleId>,
    /// Owner → Vec<CapsuleId> index (1:N relationship)
    owner_index: HashMap<Principal, Vec<CapsuleId>>,
}

impl Default for HashStore {
    fn default() -> Self {
        Self::new()
    }
}

impl HashStore {
    /// Create a new empty HashStore
    pub fn new() -> Self {
        Self {
            capsules: HashMap::new(),
            subject_index: HashMap::new(),
            owner_index: HashMap::new(),
        }
    }

    /// Internal helper: Update indexes when a capsule is added/modified
    fn update_indexes(&mut self, id: &CapsuleId, capsule: &Capsule) {
        // Update subject index (1:1 - add new)
        if let Some(subject_principal) = capsule.subject.principal() {
            self.subject_index.insert(*subject_principal, id.clone());
        }

        // Update owner index (1:N - deduplicate per owner)
        for owner_ref in capsule.owners.keys() {
            if let crate::types::PersonRef::Principal(owner_principal) = owner_ref {
                let owner_capsules = self.owner_index.entry(*owner_principal).or_default();

                // Only add if not already present (prevent duplicates)
                if !owner_capsules.contains(id) {
                    owner_capsules.push(id.clone());
                }
            }
        }
    }

    /// Internal helper: Remove from indexes when a capsule is deleted
    fn remove_from_indexes(&mut self, id: &CapsuleId, capsule: &Capsule) {
        // Remove from subject index
        if let Some(subject_principal) = capsule.subject.principal() {
            self.subject_index.remove(subject_principal);
        }

        // Remove from owner index
        for owner_ref in capsule.owners.keys() {
            if let crate::types::PersonRef::Principal(owner_principal) = owner_ref {
                if let Some(owner_capsules) = self.owner_index.get_mut(owner_principal) {
                    owner_capsules.retain(|capsule_id| capsule_id != id);
                    // Remove empty vectors
                    if owner_capsules.is_empty() {
                        self.owner_index.remove(owner_principal);
                    }
                }
            }
        }
    }

    /// Internal helper: Get principal from PersonRef
    fn principal_from_person_ref(person_ref: &crate::types::PersonRef) -> Option<&Principal> {
        match person_ref {
            crate::types::PersonRef::Principal(p) => Some(p),
            crate::types::PersonRef::Opaque(_) => None,
        }
    }
}

impl CapsuleStore for HashStore {
    fn exists(&self, id: &CapsuleId) -> bool {
        self.capsules.contains_key(id)
    }

    fn get(&self, id: &CapsuleId) -> Option<Capsule> {
        self.capsules.get(id).cloned()
    }

    fn upsert(&mut self, id: CapsuleId, capsule: Capsule) -> Option<Capsule> {
        // Key-record coherence: store key must equal capsule.id
        debug_assert_eq!(id, capsule.id, "CapsuleId key must match Capsule.id");

        // If replacing existing capsule, clean old indexes first
        if let Some(old_capsule) = self.capsules.get(&id).cloned() {
            self.remove_from_indexes(&id, &old_capsule);
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
        if let Some(capsule) = self.capsules.get_mut(id) {
            let old_subject = capsule.subject.principal().cloned();
            let old_owners: Vec<_> = capsule
                .owners
                .keys()
                .filter_map(Self::principal_from_person_ref)
                .cloned()
                .collect();

            f(capsule);

            // Update indexes if subject or owners changed
            let new_subject = capsule.subject.principal().cloned();
            let new_owners: Vec<_> = capsule
                .owners
                .keys()
                .filter_map(Self::principal_from_person_ref)
                .cloned()
                .collect();

            // Update subject index if changed
            if old_subject != new_subject {
                if let Some(old_principal) = old_subject {
                    self.subject_index.remove(&old_principal);
                }
                if let Some(new_principal) = &new_subject {
                    self.subject_index.insert(*new_principal, id.clone());
                }
            }

            // Update owner index for added/removed owners
            for owner in &old_owners {
                if !new_owners.contains(owner) {
                    if let Some(owner_capsules) = self.owner_index.get_mut(owner) {
                        owner_capsules.retain(|capsule_id| capsule_id != id);
                        if owner_capsules.is_empty() {
                            self.owner_index.remove(owner);
                        }
                    }
                }
            }
            for owner in &new_owners {
                if !old_owners.contains(owner) {
                    self.owner_index.entry(*owner).or_default().push(id.clone());
                }
            }

            Ok(())
        } else {
            Err(UpdateError::NotFound)
        }
    }

    fn update_with<R, F>(&mut self, id: &CapsuleId, f: F) -> Result<R, UpdateError>
    where
        F: FnOnce(&mut Capsule) -> Result<R, crate::types::Error>,
    {
        if let Some(capsule) = self.capsules.get_mut(id) {
            let old_subject = capsule.subject.principal().cloned();
            let old_owners: Vec<_> = capsule
                .owners
                .keys()
                .filter_map(Self::principal_from_person_ref)
                .cloned()
                .collect();

            let result = f(capsule)?;

            // Update indexes if subject or owners changed
            let new_subject = capsule.subject.principal().cloned();
            let new_owners: Vec<_> = capsule
                .owners
                .keys()
                .filter_map(Self::principal_from_person_ref)
                .cloned()
                .collect();

            // Update subject index if changed
            if old_subject != new_subject {
                if let Some(old_principal) = old_subject {
                    self.subject_index.remove(&old_principal);
                }
                if let Some(new_principal) = &new_subject {
                    self.subject_index.insert(*new_principal, id.clone());
                }
            }

            // Update owner index for added/removed owners
            for owner in &old_owners {
                if !new_owners.contains(owner) {
                    if let Some(owner_capsules) = self.owner_index.get_mut(owner) {
                        owner_capsules.retain(|capsule_id| capsule_id != id);
                        if owner_capsules.is_empty() {
                            self.owner_index.remove(owner);
                        }
                    }
                }
            }
            for owner in &new_owners {
                if !old_owners.contains(owner) {
                    self.owner_index.entry(*owner).or_default().push(id.clone());
                }
            }

            Ok(result)
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
        // Extract Principal from PersonRef for indexing
        if let Some(principal) = Self::principal_from_person_ref(subject) {
            self.subject_index
                .get(principal)
                .and_then(|id| self.capsules.get(id))
                .cloned()
        } else {
            None
        }
    }

    fn list_by_owner(&self, owner: &crate::types::PersonRef) -> Vec<CapsuleId> {
        if let Some(principal) = Self::principal_from_person_ref(owner) {
            self.owner_index.get(principal).cloned().unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    fn get_many(&self, ids: &[CapsuleId]) -> Vec<Capsule> {
        ids.iter()
            .filter_map(|id| self.capsules.get(id))
            .cloned()
            .collect()
    }

    fn paginate(&self, after: Option<CapsuleId>, limit: u32, order: Order) -> Page<Capsule> {
        let mut all_ids: Vec<&CapsuleId> = self.capsules.keys().collect();

        // Sort based on order
        match order {
            Order::Asc => all_ids.sort(),
            Order::Desc => {
                all_ids.sort();
                all_ids.reverse();
            }
        }

        // Find start position (exclusive after cursor)
        let start_pos = if let Some(after_id) = &after {
            match order {
                Order::Asc => all_ids
                    .iter()
                    .position(|id| **id > *after_id)
                    .unwrap_or(all_ids.len()),
                Order::Desc => all_ids
                    .iter()
                    .position(|id| **id < *after_id)
                    .unwrap_or(all_ids.len()),
            }
        } else {
            0
        };

        // Take the requested number of items
        let end_pos = (start_pos + limit as usize).min(all_ids.len());
        let page_ids = &all_ids[start_pos..end_pos];

        // Convert to capsules
        let items: Vec<Capsule> = page_ids
            .iter()
            .filter_map(|id| self.capsules.get(*id))
            .cloned()
            .collect();

        // Next cursor is the last item's ID (keyset pagination standard)
        let next_cursor = if items.len() == limit as usize {
            items.last().map(|capsule| capsule.id.clone())
        } else {
            None
        };

        Page { items, next_cursor }
    }

    fn count(&self) -> u64 {
        self.capsules.len() as u64
    }

    fn stats(&self) -> (u64, u64, u64) {
        // Hash store doesn't have separate indexes, so return (capsules, 0, 0)
        (self.capsules.len() as u64, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::OwnerState;
    use crate::types::PersonRef;

    #[test]
    fn test_hash_store_basic_operations() {
        let mut store = HashStore::new();

        // Test empty store
        assert_eq!(store.count(), 0);
        assert!(!store.exists(&"test".to_string()));

        // Test put_if_absent
        let id = "test-123".to_string();

        // Create test capsule
        let capsule = create_test_capsule(id.clone());
        assert!(store.put_if_absent(id.clone(), capsule.clone()).is_ok());
        assert_eq!(store.count(), 1);
        assert!(store.exists(&id));

        // Test duplicate put_if_absent fails
        assert!(store.put_if_absent(id.clone(), capsule.clone()).is_err());

        // Test get
        let retrieved = store.get(&id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, id);

        // Test upsert (update existing)
        let updated_capsule = create_test_capsule(id.clone());
        let old = store.upsert(id.clone(), updated_capsule);
        assert!(old.is_some());
        assert_eq!(store.count(), 1);

        // Test remove
        let removed = store.remove(&id);
        assert!(removed.is_some());
        assert_eq!(store.count(), 0);
        assert!(!store.exists(&id));
    }

    #[test]
    fn test_hash_store_indexes() {
        let mut store = HashStore::new();
        let id = "test-123".to_string();
        let capsule = create_test_capsule(id.clone());

        store.upsert(id.clone(), capsule.clone());

        // Test subject index
        let subject_principal = capsule.subject.principal().unwrap();
        let found = store.find_by_subject(&PersonRef::Principal(subject_principal.clone()));
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, id);

        // Test owner index
        let owner_principal = subject_principal; // In this case, subject is also owner
        let owner_capsules = store.list_by_owner(&PersonRef::Principal(owner_principal.clone()));
        assert_eq!(owner_capsules.len(), 1);
        assert_eq!(owner_capsules[0], id);
    }

    #[test]
    fn test_hash_store_pagination() {
        let mut store = HashStore::new();

        // Add multiple capsules
        for i in 0..5 {
            let capsule_id = format!("capsule-{}", i);
            let capsule = create_test_capsule(capsule_id.clone());
            store.upsert(capsule_id, capsule);
        }

        // Test pagination with limit
        let page = store.paginate(None, 2, Order::Asc);
        assert_eq!(page.items.len(), 2);
        assert!(page.next_cursor.is_some());

        // Test pagination with cursor
        let next_page = store.paginate(page.next_cursor, 2, Order::Asc);
        assert_eq!(next_page.items.len(), 2);

        // Test pagination descending
        let desc_page = store.paginate(None, 3, Order::Desc);
        assert_eq!(desc_page.items.len(), 3);
    }

    #[test]
    fn test_hash_store_upsert_subject_change() {
        let mut store = HashStore::new();
        let id = "test-subject-change".to_string();

        // Create initial capsule
        #[allow(unused_mut)]
        let mut capsule1 = create_test_capsule(id.clone());
        store.upsert(id.clone(), capsule1.clone());

        // Verify initial subject
        let found = store.find_by_subject(&capsule1.subject);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, id);

        // Create new capsule with different subject
        let new_subject = PersonRef::Principal(Principal::from_text("2vxsx-fae").unwrap());
        let capsule2 = Capsule {
            id: id.clone(),
            subject: new_subject.clone(),
            owners: HashMap::new(), // Clear owners for simplicity
            controllers: HashMap::new(),
            connections: HashMap::new(),
            has_advanced_settings: false, // Default to simple settings
            connection_groups: HashMap::new(),
            memories: HashMap::new(),
            galleries: HashMap::new(),
            created_at: 1234567890,
            updated_at: 1234567890,
            // bound_to_neon removed - now tracked in database_storage_edges
            inline_bytes_used: 0,
        };

        // Upsert with new subject
        store.upsert(id.clone(), capsule2.clone());

        // Verify old subject no longer finds anything
        let old_found = store.find_by_subject(&capsule1.subject);
        assert!(old_found.is_none());

        // Verify new subject finds the capsule
        let new_found = store.find_by_subject(&new_subject);
        assert!(new_found.is_some());
        assert_eq!(new_found.unwrap().id, id);
    }

    #[test]
    fn test_hash_store_upsert_owner_change() {
        let mut store = HashStore::new();
        let id = "test-owner-change".to_string();

        // Create initial capsule with one owner
        let mut capsule1 = create_test_capsule(id.clone());
        let owner1 = PersonRef::Principal(Principal::from_text("aaaaa-aa").unwrap());
        let mut owners1 = HashMap::new();
        owners1.insert(
            owner1.clone(),
            OwnerState {
                since: 1234567890,
                last_activity_at: 1234567890,
            },
        );
        capsule1.owners = owners1;

        store.upsert(id.clone(), capsule1.clone());

        // Verify initial owner
        let owner_list1 = store.list_by_owner(&owner1);
        assert_eq!(owner_list1.len(), 1);
        assert_eq!(owner_list1[0], id);

        // Create new capsule with different owner
        let owner2 = PersonRef::Principal(Principal::from_text("w7x7r-cok77-xa").unwrap());
        let mut owners2 = HashMap::new();
        owners2.insert(
            owner2.clone(),
            OwnerState {
                since: 1234567890,
                last_activity_at: 1234567890,
            },
        );

        let capsule2 = Capsule {
            id: id.clone(),
            subject: capsule1.subject.clone(),
            owners: owners2,
            controllers: HashMap::new(),
            connections: HashMap::new(),
            has_advanced_settings: false, // Default to simple settings
            connection_groups: HashMap::new(),
            memories: HashMap::new(),
            galleries: HashMap::new(),
            created_at: 1234567890,
            updated_at: 1234567890,
            // bound_to_neon removed - now tracked in database_storage_edges
            inline_bytes_used: 0,
        };

        // Upsert with new owner
        store.upsert(id.clone(), capsule2.clone());

        // Verify old owner no longer has this capsule
        let owner_list1_after = store.list_by_owner(&owner1);
        assert_eq!(owner_list1_after.len(), 0);

        // Verify new owner has the capsule
        let owner_list2 = store.list_by_owner(&owner2);
        assert_eq!(owner_list2.len(), 1);
        assert_eq!(owner_list2[0], id);
    }

    fn create_test_capsule(id: String) -> Capsule {
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
            has_advanced_settings: false, // Default to simple settings
            created_at: 1234567890,
            updated_at: 1234567890,
            // bound_to_neon removed - now tracked in database_storage_edges
            inline_bytes_used: 0,
        }
    }
}
*/

// LEGACY CODE - COMMENTED OUT
// This HashMap implementation is kept for reference but not used in production
// We only use the StableStore implementation now
