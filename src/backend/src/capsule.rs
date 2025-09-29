#![allow(dead_code)]

use crate::capsule_store::{types::PaginationOrder as Order, CapsuleStore};
use crate::memory::{with_capsule_store, with_capsule_store_mut};
use crate::state::add_canister_size;
use crate::types::Result;
use crate::types::*;
use ic_stable_structures::Storable;

use ic_cdk::api::time;
use std::collections::HashMap;

// ============================================================================
// SIZE TRACKING UTILITIES
// ============================================================================

/// Calculate the serialized size of a capsule
fn calculate_capsule_size(capsule: &Capsule) -> u64 {
    let bytes = capsule.to_bytes();
    bytes.len() as u64
}

impl Capsule {
    pub fn new(subject: PersonRef, initial_owner: PersonRef) -> Self {
        let now = time();
        let capsule_id = format!("capsule_{now}");

        let mut owners = HashMap::new();
        owners.insert(
            initial_owner,
            OwnerState {
                since: now,
                last_activity_at: now,
            },
        );

        Capsule {
            id: capsule_id,
            subject,
            owners,
            controllers: HashMap::new(),
            connections: HashMap::new(),
            connection_groups: HashMap::new(),
            memories: HashMap::new(),
            galleries: HashMap::new(),
            created_at: now,
            updated_at: now,
            bound_to_neon: false, // Initially not bound to Neon
            inline_bytes_used: 0, // Start with zero inline consumption
        }
    }

    /// Check if a PersonRef is an owner
    pub fn is_owner(&self, person: &PersonRef) -> bool {
        self.owners.contains_key(person)
    }

    /// Check if a PersonRef is a controller
    pub fn is_controller(&self, person: &PersonRef) -> bool {
        self.controllers.contains_key(person)
    }

    /// Check if a PersonRef has write access (owner or controller)
    pub fn has_write_access(&self, person: &PersonRef) -> bool {
        self.is_owner(person) || self.is_controller(person)
    }

    /// Check if a PersonRef can read a specific memory
    pub fn can_read_memory(&self, person: &PersonRef, memory: &Memory) -> bool {
        self.check_memory_access(person, &memory.access)
    }

    /// Core access checking logic - handles all access types including recursive cases
    fn check_memory_access(&self, person: &PersonRef, access: &MemoryAccess) -> bool {
        match access {
            MemoryAccess::Public { owner_secure_code: _ } => true,
            MemoryAccess::Private { owner_secure_code: _ } => self.has_write_access(person),
            MemoryAccess::Custom {
                individuals,
                groups,
                owner_secure_code: _,
            } => {
                // Check if person has write access (owners/controllers always have access)
                if self.has_write_access(person) {
                    return true;
                }

                // Check direct individual access
                if individuals.contains(person) {
                    return true;
                }

                // Check group access
                for group_id in groups {
                    if let Some(group) = self.connection_groups.get(group_id) {
                        if group.members.contains(person) {
                            return true;
                        }
                    }
                }

                false
            }
            MemoryAccess::Scheduled {
                accessible_after,
                access,
                owner_secure_code: _,
            } => {
                // Check if time has passed, if so use the nested access rule
                let current_time = ic_cdk::api::time();
                if current_time >= *accessible_after {
                    self.check_memory_access(person, access) // Recursive call
                } else {
                    // Not yet accessible
                    false
                }
            }
            MemoryAccess::EventTriggered { access, .. } => {
                // For now, treat as private until event system is implemented
                // TODO: Implement event checking
                self.check_memory_access(person, access) // Recursive call
            }
        }
    }

    /// Update the capsule's last modified timestamp
    pub fn touch(&mut self) {
        self.updated_at = time();
    }

    /// Get capsule header for listing
    pub fn to_header(&self) -> CapsuleHeader {
        CapsuleHeader {
            id: self.id.clone(),
            subject: self.subject.clone(),
            owner_count: self.owners.len() as u32,
            controller_count: self.controllers.len() as u32,
            memory_count: self.memories.len() as u32,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

// Note: CAPSULES storage moved to memory.rs for centralized storage

/// Create a new capsule with optional subject
/// If subject is None, creates a self-capsule (subject = caller)
/// If subject is provided, creates a capsule for that subject

pub fn capsules_create(subject: Option<PersonRef>) -> Result<Capsule> {
    let caller = PersonRef::from_caller();

    // Check if caller already has a self-capsule when creating self-capsule
    let is_self_capsule = subject.is_none();

    if is_self_capsule {
        // MIGRATED: Check if caller already has a self-capsule
        let all_capsules = with_capsule_store(|store| store.paginate(None, u32::MAX, Order::Asc));

        let existing_self_capsule = all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller));

        if let Some(capsule) = existing_self_capsule {
            // MIGRATED: Update existing self-capsule activity
            let capsule_id = capsule.id.clone();
            let update_result = with_capsule_store_mut(|store| {
                store.update(&capsule_id, |capsule| {
                    let now = time();
                    capsule.updated_at = now;

                    if let Some(owner_state) = capsule.owners.get_mut(&caller) {
                        owner_state.last_activity_at = now;
                    }
                })
            });

            if update_result.is_ok() {
                // Return the existing capsule
                return Ok(capsule);
            } else {
                return Err(Error::Internal(
                    "Failed to update capsule activity".to_string(),
                ));
            }
        }
    }

    // MIGRATED: Create new capsule
    let actual_subject = subject.unwrap_or_else(|| caller.clone());
    let capsule = Capsule::new(actual_subject, caller);
    let capsule_id = capsule.id.clone();

    // Track size before creating capsule
    let capsule_size = calculate_capsule_size(&capsule);
    if let Err(_e) = add_canister_size(capsule_size) {
        return Err(Error::ResourceExhausted);
    }

    // Use upsert to create new capsule (should succeed since we're checking for existing self-capsule above)
    with_capsule_store_mut(|store| {
        store.upsert(capsule_id.clone(), capsule.clone());
    });

    Ok(capsule)
}

/// Get capsule by ID (with read access check)

pub fn capsules_read(capsule_id: String) -> Result<Capsule> {
    let caller = PersonRef::from_caller();

    // MIGRATED: Using new trait-based API
    with_capsule_store(|store| {
        store
            .get(&capsule_id)
            .filter(|capsule| capsule.has_write_access(&caller))
            .ok_or(Error::NotFound)
    })
}

/// Get capsule info by ID (basic version with read access check)

pub fn capsules_read_basic(capsule_id: String) -> Result<CapsuleInfo> {
    let caller = PersonRef::from_caller();

    // MIGRATED: Using new trait-based API
    with_capsule_store(|store| {
        store
            .get(&capsule_id)
            .filter(|capsule| capsule.has_write_access(&caller))
            .map(|capsule| CapsuleInfo {
                capsule_id: capsule.id.clone(),
                subject: capsule.subject.clone(),
                is_owner: capsule.owners.contains_key(&caller),
                is_controller: capsule.controllers.contains_key(&caller),
                is_self_capsule: capsule.subject == caller,
                bound_to_neon: capsule.bound_to_neon,
                created_at: capsule.created_at,
                updated_at: capsule.updated_at,

                // Add lightweight counts for summary information
                memory_count: capsule.memories.len() as u32,
                gallery_count: capsule.galleries.len() as u32,
                connection_count: capsule.connections.len() as u32,
            })
            .ok_or(Error::NotFound)
    })
}

/// Get caller's self-capsule (where caller is the subject)

pub fn capsule_read_self() -> Result<Capsule> {
    let caller = PersonRef::from_caller();

    // MIGRATED: Find caller's self-capsule
    with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == caller)
            .ok_or(Error::NotFound)
    })
}

/// Get caller's self-capsule info (basic version)
pub fn capsule_read_self_basic() -> Result<CapsuleInfo> {
    let caller = PersonRef::from_caller();

    // MIGRATED: Find caller's self-capsule and create basic info
    with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == caller)
            .map(|capsule| CapsuleInfo {
                capsule_id: capsule.id.clone(),
                subject: capsule.subject.clone(),
                is_owner: capsule.owners.contains_key(&caller),
                is_controller: capsule.controllers.contains_key(&caller),
                is_self_capsule: true,
                bound_to_neon: capsule.bound_to_neon,
                created_at: capsule.created_at,
                updated_at: capsule.updated_at,

                // Add lightweight counts for summary information
                memory_count: capsule.memories.len() as u32,
                gallery_count: capsule.galleries.len() as u32,
                connection_count: capsule.connections.len() as u32,
            })
            .ok_or(Error::NotFound)
    })
}

/// List capsules owned or controlled by caller
pub fn capsules_list() -> Vec<CapsuleHeader> {
    let caller = PersonRef::from_caller();

    // MIGRATED: Using new trait-based API with pagination
    // Note: Using large limit to get all capsules, maintaining backward compatibility
    with_capsule_store(|store| {
        let page = store.paginate(None, u32::MAX, Order::Asc);
        page.items
            .into_iter()
            .filter(|capsule| capsule.has_write_access(&caller))
            .map(|capsule| capsule.to_header())
            .collect()
    })
}

/// Update a capsule with the provided data
/// Only allows updates to mutable fields (binding status, timestamps)
pub fn capsules_update(capsule_id: String, updates: CapsuleUpdateData) -> Result<Capsule> {
    let caller = PersonRef::from_caller();

    // First check if the capsule exists and caller has access
    let _capsule = with_capsule_store(|store| {
        store
            .get(&capsule_id)
            .filter(|capsule| capsule.has_write_access(&caller))
            .ok_or(Error::NotFound)
    })?;

    // Update the capsule
    with_capsule_store_mut(|store| {
        store.update(&capsule_id, |capsule| {
            // Update mutable fields
            if let Some(bound_to_neon) = updates.bound_to_neon {
                capsule.bound_to_neon = bound_to_neon;
            }

            // Update timestamp
            capsule.updated_at = time();

            // Update owner activity
            if let Some(owner_state) = capsule.owners.get_mut(&caller) {
                owner_state.last_activity_at = time();
            }
        })
    })?;

    // Return the updated capsule
    with_capsule_store(|store| store.get(&capsule_id).ok_or(Error::NotFound))
}

/// Delete a capsule (permanent deletion)
/// Only allows deletion by capsule owners
pub fn capsules_delete(capsule_id: String) -> Result<()> {
    let caller = PersonRef::from_caller();

    // First, get the capsule to check ownership and calculate size for tracking
    let capsule = with_capsule_store(|store| {
        store
            .get(&capsule_id)
            .filter(|capsule| capsule.has_write_access(&caller))
            .ok_or(Error::NotFound)
    })?;

    // Calculate size for tracking removal
    let _capsule_size = calculate_capsule_size(&capsule);

    // Remove from storage
    with_capsule_store_mut(|store| store.remove(&capsule_id).ok_or(Error::NotFound))?;

    // Track size removal (note: we don't have remove_canister_size implemented yet)
    // For now, we'll just log this - in a full implementation, we'd subtract from total size
    // TODO: Implement remove_canister_size in state.rs

    Ok(())
}

/// Flexible resource binding function for Neon database
/// Can bind capsules, galleries, or memories to Neon
pub fn capsules_bind_neon(
    resource_type: ResourceType,
    resource_id: String,
    bind: bool,
) -> Result<()> {
    let caller_ref = PersonRef::from_caller();

    match resource_type {
        ResourceType::Capsule => {
            // MIGRATED: Bind specific capsule if caller owns it
            with_capsule_store_mut(|store| {
                let update_result = store.update(&resource_id, |capsule| {
                    if capsule.owners.contains_key(&caller_ref) {
                        capsule.bound_to_neon = bind;
                        capsule.updated_at = time();

                        // Update owner activity
                        if let Some(owner_state) = capsule.owners.get_mut(&caller_ref) {
                            owner_state.last_activity_at = time();
                        }
                    }
                });
                if update_result.is_ok() {
                    Ok(())
                } else {
                    Err(crate::types::Error::Internal(
                        "Failed to update capsule".to_string(),
                    ))
                }
            })
        }
        ResourceType::Gallery => {
            // MIGRATED: Bind specific gallery if caller owns the capsule containing it
            with_capsule_store_mut(|store| {
                let all_capsules = store.paginate(None, u32::MAX, Order::Asc);

                for capsule in all_capsules.items {
                    if capsule.owners.contains_key(&caller_ref)
                        && capsule.galleries.contains_key(&resource_id)
                    {
                        // Found the capsule containing the gallery
                        let update_result = store.update(&capsule.id, |capsule| {
                            if let Some(gallery) = capsule.galleries.get_mut(&resource_id) {
                                gallery.bound_to_neon = bind;
                                capsule.updated_at = time();

                                // Update owner activity
                                if let Some(owner_state) = capsule.owners.get_mut(&caller_ref) {
                                    owner_state.last_activity_at = time();
                                }
                            }
                        });
                        return if update_result.is_ok() {
                            Ok(())
                        } else {
                            Err(crate::types::Error::Internal(
                                "Failed to update gallery".to_string(),
                            ))
                        };
                    }
                }
                Err(crate::types::Error::NotFound)
            })
        }
        ResourceType::Memory => {
            // MIGRATED: Bind specific memory if caller owns the capsule containing it
            with_capsule_store_mut(|store| {
                let all_capsules = store.paginate(None, u32::MAX, Order::Asc);

                for capsule in all_capsules.items {
                    if capsule.owners.contains_key(&caller_ref)
                        && capsule.memories.contains_key(&resource_id)
                    {
                        // Found the capsule containing the memory
                        let update_result = store.update(&capsule.id, |capsule| {
                            if let Some(memory) = capsule.memories.get_mut(&resource_id) {
                                // Update the bound_to_neon field in the memory's metadata
                                match &mut memory.metadata {
                                    MemoryMetadata::Image(meta) => meta.base.bound_to_neon = bind,
                                    MemoryMetadata::Video(meta) => meta.base.bound_to_neon = bind,
                                    MemoryMetadata::Audio(meta) => meta.base.bound_to_neon = bind,
                                    MemoryMetadata::Document(meta) => {
                                        meta.base.bound_to_neon = bind
                                    }
                                    MemoryMetadata::Note(meta) => meta.base.bound_to_neon = bind,
                                }

                                capsule.updated_at = time();

                                // Update owner activity
                                if let Some(owner_state) = capsule.owners.get_mut(&caller_ref) {
                                    owner_state.last_activity_at = time();
                                }
                            }
                        });
                        return if update_result.is_ok() {
                            Ok(())
                        } else {
                            Err(crate::types::Error::Internal(
                                "Failed to update memory".to_string(),
                            ))
                        };
                    }
                }
                Err(crate::types::Error::NotFound)
            })
        }
    }
}

/// Export all capsules for upgrade persistence
pub fn export_capsules_for_upgrade() -> Vec<(String, Capsule)> {
    // MIGRATED: Export all capsules using pagination
    with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .map(|capsule| (capsule.id.clone(), capsule))
            .collect()
    })
}

/// Import capsules from upgrade persistence
pub fn import_capsules_from_upgrade(capsule_data: Vec<(String, Capsule)>) {
    with_capsule_store_mut(|store| {
        for (id, capsule) in capsule_data {
            store.upsert(id, capsule);
        }
    })
}

// ============================================================================
// CAPSULE HELPER FUNCTIONS
// ============================================================================

/// Find a self-capsule (where subject == owner == caller)
pub fn find_self_capsule(caller: &PersonRef) -> Option<Capsule> {
    use crate::capsule_store::types::PaginationOrder as Order;
    use crate::memory::with_capsule_store;

    let all_capsules = with_capsule_store(|store| store.paginate(None, u32::MAX, Order::Asc));
    all_capsules
        .items
        .into_iter()
        .find(|capsule| capsule.subject == *caller && capsule.owners.contains_key(caller))
}

/// Update a capsule's activity timestamp for a specific owner
pub fn update_capsule_activity(capsule_id: &str, caller: &PersonRef) -> crate::types::Result<()> {
    let now = ic_cdk::api::time();
    use crate::memory::with_capsule_store_mut;

    with_capsule_store_mut(|store| {
        store.update(&capsule_id.to_string(), |capsule| {
            if let Some(owner_state) = capsule.owners.get_mut(caller) {
                owner_state.last_activity_at = now;
            }
            capsule.updated_at = now;
        })
    })
    .map_err(|_| crate::types::Error::Internal("Failed to update capsule activity".to_string()))
}
