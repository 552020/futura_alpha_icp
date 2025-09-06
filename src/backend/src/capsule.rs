#![allow(dead_code)]

use crate::capsule_store::{types::PaginationOrder as Order, CapsuleStore};
use crate::memory::{with_capsule_store, with_capsule_store_mut};
use crate::state::{add_canister_size, track_size_change};
use crate::types::BlobRef;
use crate::types::Result;
use crate::types::*;
use ic_stable_structures::Storable;

use ic_cdk::api::{msg_caller, time};
use std::collections::HashMap;

// ============================================================================
// SIZE TRACKING UTILITIES
// ============================================================================

/// Calculate the serialized size of a capsule
fn calculate_capsule_size(capsule: &Capsule) -> u64 {
    let bytes = capsule.to_bytes();
    bytes.len() as u64
}

// ============================================================================
// MIGRATION GUIDE: From Direct Storage Access to Trait-Based API
// ============================================================================
//
// OLD PATTERN (direct HashMap access):
//     with_capsules(|capsules: &HashMap<String, Capsule>| {
//         capsules.values().find(|c| c.subject == caller).cloned()
//     });
//
// NEW PATTERN (trait-based, object-safe):
//     with_capsule_store(|store: &dyn CapsuleStore| {
//         store.find_by_subject(&caller.into())
//     });
//
// Benefits:
// - Object-safe: Can use `dyn CapsuleStore`
// - Runtime polymorphism: Switch between HashMap/StableBTreeMap at runtime
// - Clean API: Methods like find_by_subject() handle common operations
// - Test-friendly: Easy to mock different storage backends
//
// Migration steps:
// 1. Replace with_capsules -> with_capsule_store
// 2. Use store methods instead of direct HashMap operations
// 3. Add helper methods to CapsuleStore trait as needed
//
// EXAMPLE MIGRATION:
//
// Before:
// ```rust
// pub fn find_capsule_by_subject_old(caller: Principal) -> Option<Capsule> {
//     with_hashmap_capsules(|capsules| {
//         capsules.values().find(|c| c.subject == PersonRef::Principal(caller)).cloned()
//     })
// }
//
// After:
// ```rust
// pub fn find_capsule_by_subject_new(caller: Principal) -> Option<Capsule> {
//     with_capsule_store(|store| {
//         store.find_by_subject(&PersonRef::Principal(caller))
//     })
// }
// ```
// ============================================================================

impl PersonRef {
    /// Create a PersonRef from the current caller
    pub fn from_caller() -> Self {
        PersonRef::Principal(msg_caller())
    }

    /// Create an opaque PersonRef (for deceased/non-principal subjects)
    pub fn opaque(id: String) -> Self {
        PersonRef::Opaque(id)
    }

    /// Check if this PersonRef matches the current caller
    pub fn is_caller(&self) -> bool {
        match self {
            PersonRef::Principal(p) => *p == msg_caller(),
            PersonRef::Opaque(_) => false,
        }
    }
}

impl Capsule {
    /// Create a new capsule
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

    /// Insert a new memory into the capsule
    pub fn insert_memory(
        &mut self,
        memory_id: &str,
        blob: BlobRef,
        meta: MemoryMeta,
        now: u64,
        idempotency_key: Option<String>,
    ) -> crate::types::Result<()> {
        use crate::types::{
            ImageMetadata, Memory, MemoryAccess, MemoryInfo, MemoryMetadata, MemoryMetadataBase,
            MemoryType,
        };

        let memory_info = MemoryInfo {
            memory_type: MemoryType::Image, // Default, can be updated later
            name: meta.name.clone(),
            content_type: "application/octet-stream".to_string(),
            created_at: now,
            updated_at: now,
            uploaded_at: now,
            date_of_memory: None,
        };

        let memory_metadata = MemoryMetadata::Image(ImageMetadata {
            base: MemoryMetadataBase {
                size: blob.hash.map_or(0, |_| 32), // Use hash length as size indicator, or calculate properly
                mime_type: "application/octet-stream".to_string(),
                original_name: meta.name.clone(),
                uploaded_at: now.to_string(),
                date_of_memory: None,
                people_in_memory: None,
                format: None,
                bound_to_neon: false,
            },
            dimensions: None,
        });

        let memory_access = MemoryAccess::Private;

        let memory_data = MemoryData::BlobRef {
            blob,
            meta: meta.clone(),
        };

        let memory = Memory {
            id: memory_id.to_string(),
            info: memory_info,
            metadata: memory_metadata,
            access: memory_access,
            data: memory_data,
            idempotency_key,
        };

        self.memories.insert(memory_id.to_string(), memory);
        self.updated_at = now;
        Ok(())
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
        match &memory.access {
            MemoryAccess::Public => true,
            MemoryAccess::Private => self.has_write_access(person),
            MemoryAccess::Custom {
                individuals,
                groups,
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
            } => {
                // Check if time has passed, if so use the nested access rule
                let current_time = ic_cdk::api::time();
                if current_time >= *accessible_after {
                    self.can_read_memory_access(person, access)
                } else {
                    // Not yet accessible
                    false
                }
            }
            MemoryAccess::EventTriggered { access, .. } => {
                // For now, treat as private until event system is implemented
                // TODO: Implement event checking
                self.can_read_memory_access(person, access)
            }
        }
    }

    /// Helper function to check access recursively
    fn can_read_memory_access(&self, person: &PersonRef, access: &MemoryAccess) -> bool {
        match access {
            MemoryAccess::Public => true,
            MemoryAccess::Private => self.has_write_access(person),
            MemoryAccess::Custom {
                individuals,
                groups,
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
            } => {
                let current_time = ic_cdk::api::time();
                if current_time >= *accessible_after {
                    self.can_read_memory_access(person, access)
                } else {
                    false
                }
            }
            MemoryAccess::EventTriggered { access, .. } => {
                // TODO: Implement event checking
                self.can_read_memory_access(person, access)
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

pub fn capsules_create(subject: Option<PersonRef>) -> CapsuleCreationResult {
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
                return CapsuleCreationResult {
                    success: true,
                    capsule_id: Some(capsule_id),
                    message: "Welcome back! Your capsule is ready.".to_string(),
                };
            }
        }
    }

    // MIGRATED: Create new capsule
    let actual_subject = subject.unwrap_or_else(|| caller.clone());
    let capsule = Capsule::new(actual_subject, caller);
    let capsule_id = capsule.id.clone();

    // Track size before creating capsule
    let capsule_size = calculate_capsule_size(&capsule);
    if let Err(e) = add_canister_size(capsule_size) {
        return CapsuleCreationResult {
            success: false,
            capsule_id: None,
            message: format!("Cannot create capsule: {}", e),
        };
    }

    // Use upsert to create new capsule (should succeed since we're checking for existing self-capsule above)
    with_capsule_store_mut(|store| {
        store.upsert(capsule_id.clone(), capsule);
    });

    CapsuleCreationResult {
        success: true,
        capsule_id: Some(capsule_id),
        message: if is_self_capsule {
            "Self-capsule created successfully!".to_string()
        } else {
            "Capsule created".to_string()
        },
    }
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

/// Simple user registration for II integration (idempotent)
/// Tracks basic user info: { registered_at, last_activity_at, bound: bool }
pub fn register() -> Result<()> {
    let caller_ref = PersonRef::from_caller();
    let now = time();

    // MIGRATED: Check if user already has a self-capsule
    let all_capsules = with_capsule_store(|store| store.paginate(None, u32::MAX, Order::Asc));

    let existing_self_capsule = all_capsules
        .items
        .into_iter()
        .find(|capsule| capsule.subject == caller_ref && capsule.owners.contains_key(&caller_ref));

    match existing_self_capsule {
        Some(capsule) => {
            // MIGRATED: Update activity timestamp using store.update()
            let capsule_id = capsule.id.clone();
            let update_result = with_capsule_store_mut(|store| {
                store.update(&capsule_id, |capsule| {
                    if let Some(owner_state) = capsule.owners.get_mut(&caller_ref) {
                        owner_state.last_activity_at = now;
                    }
                    capsule.updated_at = now;
                })
            });
            if update_result.is_ok() {
                Ok(())
            } else {
                Err(crate::types::Error::Internal(
                    "Failed to update capsule activity".to_string(),
                ))
            }
        }
        None => {
            // MIGRATED: Create new self-capsule with basic info
            match capsules_create(None) {
                CapsuleCreationResult { success: true, .. } => Ok(()),
                CapsuleCreationResult { success: false, .. } => Err(crate::types::Error::Internal(
                    "Failed to create capsule".to_string(),
                )),
            }
        }
    }
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
// MEMORY MANAGEMENT FUNCTIONS
// ============================================================================
// NOTE: Memory functions are kept in capsule.rs because memories are always
// stored within capsules. This could be refactored into a separate memories.rs
// module in the future if the capsule.rs file becomes too large or if we need
// more complex memory-specific logic.
// ============================================================================

/// Add a memory to the caller's capsule (deprecated - use memories_create instead)
#[deprecated(
    since = "0.7.0",
    note = "Use memories_create with capsule_id parameter instead"
)]

pub fn add_memory_to_capsule(
    memory_id: String,
    memory_data: MemoryData,
) -> MemoryOperationResponse {
    let caller = PersonRef::from_caller();

    // MIGRATED: Find caller's self-capsule
    let capsule = with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller))
    });

    match capsule {
        Some(mut capsule) => {
            // Check if memory already exists with this UUID (idempotency)
            if capsule.memories.contains_key(&memory_id) {
                return MemoryOperationResponse {
                    success: true,
                    memory_id: Some(memory_id),
                    message: "Memory already exists with this UUID".to_string(),
                };
            }

            // Use the memory ID provided by Web2 (don't generate new ID)

            // Create memory info
            let now = ic_cdk::api::time();
            let memory_info = MemoryInfo {
                memory_type: MemoryType::Image, // Default type, can be updated later
                name: format!("Memory {memory_id}"),
                content_type: "application/octet-stream".to_string(),
                created_at: now,
                updated_at: now,
                uploaded_at: now,
                date_of_memory: None,
            };

            // Create memory metadata (default to Image type)
            let memory_metadata = MemoryMetadata::Image(ImageMetadata {
                base: MemoryMetadataBase {
                    size: match &memory_data {
                        crate::types::MemoryData::Inline { bytes, .. } => bytes.len() as u64,
                        crate::types::MemoryData::BlobRef { blob, .. } => {
                            blob.hash.map_or(0, |_| 32)
                        } // Size of hash if available
                    },
                    mime_type: "application/octet-stream".to_string(),
                    original_name: format!("Memory {memory_id}"),
                    uploaded_at: now.to_string(),
                    date_of_memory: None,
                    people_in_memory: None,
                    format: None,
                    bound_to_neon: false,
                },
                dimensions: None,
            });

            // Create memory access (default to private)
            let memory_access = MemoryAccess::Private;

            // Create the memory
            let memory = Memory {
                id: memory_id.clone(),
                info: memory_info,
                metadata: memory_metadata,
                access: memory_access,
                data: memory_data,
                idempotency_key: None, // No idempotency key for legacy memories_create
            };

            // Store memory in capsule
            capsule.memories.insert(memory_id.clone(), memory);
            capsule.updated_at = ic_cdk::api::time(); // Update capsule timestamp

            // MIGRATED: Save updated capsule
            with_capsule_store_mut(|store| {
                store.upsert(capsule.id.clone(), capsule);
            });

            MemoryOperationResponse {
                success: true,
                memory_id: Some(memory_id),
                message: "Memory added successfully to capsule".to_string(),
            }
        }
        None => MemoryOperationResponse {
            success: false,
            memory_id: None,
            message: "No capsule found for caller".to_string(),
        },
    }
}

/// Read a memory by its ID (searches across all capsules the caller has access to)
pub fn memories_read(memory_id: String) -> Result<Memory> {
    let caller = PersonRef::from_caller();

    // MIGRATED: Find memory across caller's accessible capsules
    with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .find(|capsule| {
                // Check if caller has access to this capsule
                capsule.owners.contains_key(&caller) || capsule.subject == caller
            })
            .and_then(|capsule| capsule.memories.get(&memory_id).cloned())
            .ok_or(Error::NotFound)
    })
}

/// Update a memory by its ID (searches across all capsules the caller has access to)
pub fn memories_update(memory_id: String, updates: MemoryUpdateData) -> MemoryOperationResponse {
    let caller = PersonRef::from_caller();
    let memory_id_clone = memory_id.clone();

    // MIGRATED: Find and update memory across caller's accessible capsules
    let mut capsule_found = false;
    let mut memory_found = false;

    with_capsule_store_mut(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);

        // Find the capsule containing the memory
        if let Some(capsule) = all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.owners.contains_key(&caller) || capsule.subject == caller)
            .filter(|capsule| capsule.memories.contains_key(&memory_id))
        {
            capsule_found = true;
            let capsule_id = capsule.id.clone();

            // Update the capsule with the modified memory
            let update_result = store.update(&capsule_id, |capsule| {
                if let Some(memory) = capsule.memories.get(&memory_id) {
                    memory_found = true;

                    // Update memory fields
                    let mut updated_memory = memory.clone();
                    if let Some(name) = updates.name.clone() {
                        updated_memory.info.name = name;
                    }
                    if let Some(metadata) = updates.metadata.clone() {
                        updated_memory.metadata = metadata;
                    }
                    if let Some(access) = updates.access.clone() {
                        updated_memory.access = access;
                    }

                    updated_memory.info.updated_at = ic_cdk::api::time();

                    // Store updated memory
                    capsule.memories.insert(memory_id, updated_memory);
                    capsule.updated_at = ic_cdk::api::time(); // Update capsule timestamp
                }
            });

            if update_result.is_err() {
                capsule_found = false;
            }
        }
    });

    if !capsule_found {
        return MemoryOperationResponse {
            success: false,
            memory_id: None,
            message: "No accessible capsule found for caller".to_string(),
        };
    }

    if !memory_found {
        return MemoryOperationResponse {
            success: false,
            memory_id: None,
            message: "Memory not found in any accessible capsule".to_string(),
        };
    }

    MemoryOperationResponse {
        success: true,
        memory_id: Some(memory_id_clone),
        message: "Memory updated successfully".to_string(),
    }
}

/// Delete a memory by its ID (searches across all capsules the caller has access to)
pub fn memories_delete(memory_id: String) -> MemoryOperationResponse {
    let caller = PersonRef::from_caller();
    let memory_id_clone = memory_id.clone();

    // MIGRATED: Search across all capsules the caller has access to
    let mut memory_found = false;
    let mut capsule_found = false;

    with_capsule_store_mut(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);

        // Find the capsule containing the memory
        if let Some(capsule) = all_capsules.items.into_iter().find(|capsule| {
            capsule.has_write_access(&caller) && capsule.memories.contains_key(&memory_id)
        }) {
            capsule_found = true;
            let capsule_id = capsule.id.clone();

            // Update the capsule to remove the memory
            let update_result = store.update(&capsule_id, |capsule| {
                if capsule.memories.contains_key(&memory_id) {
                    capsule.memories.remove(&memory_id);
                    capsule.updated_at = ic_cdk::api::time(); // Update capsule timestamp
                    memory_found = true;
                }
            });

            if update_result.is_err() {
                capsule_found = false;
            }
        }
    });

    if !capsule_found {
        return MemoryOperationResponse {
            success: false,
            memory_id: None,
            message: "No accessible capsule found for caller".to_string(),
        };
    }

    if !memory_found {
        return MemoryOperationResponse {
            success: false,
            memory_id: None,
            message: "Memory not found in any accessible capsule".to_string(),
        };
    }

    MemoryOperationResponse {
        success: true,
        memory_id: Some(memory_id_clone),
        message: "Memory deleted successfully".to_string(),
    }
}

/// List all memories in a specific capsule by ID
pub fn memories_list(capsule_id: String) -> MemoryListResponse {
    let caller = PersonRef::from_caller();

    // MIGRATED: Get memories from specified capsule
    let memories = with_capsule_store(|store| {
        store
            .get(&capsule_id)
            .and_then(|capsule| {
                // Check if caller has access to this capsule
                if capsule.owners.contains_key(&caller) || capsule.subject == caller {
                    Some(capsule.memories.values().cloned().collect::<Vec<_>>())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    });

    MemoryListResponse {
        success: true,
        memories,
        message: "Memories retrieved successfully".to_string(),
    }
}

// ============================================================================
// TESTS FOR GALLERY ENHANCEMENTS
// ============================================================================

#[cfg(test)]
mod gallery_tests {
    use super::*;

    #[test]
    fn test_gallery_storage_location_logic() {
        // Test the logic for different storage status values
        // This test doesn't call the actual functions to avoid canister time() calls

        // Test storage status enum values
        let location_icp = GalleryStorageLocation::ICPOnly;
        let location_both = GalleryStorageLocation::Both;
        let location_web2 = GalleryStorageLocation::Web2Only;
        let location_failed = GalleryStorageLocation::Failed;

        // Verify enum values are different
        assert_ne!(location_icp, location_both);
        assert_ne!(location_web2, location_failed);
        assert_ne!(location_icp, location_web2);
    }

    #[test]
    fn test_gallery_data_structure() {
        // Test that we can create gallery data structures correctly
        let gallery_data = create_test_gallery_data();

        assert_eq!(gallery_data.gallery.title, "Test Gallery");
        assert_eq!(
            gallery_data.gallery.storage_location,
            GalleryStorageLocation::Web2Only
        );
        assert!(!gallery_data.gallery.is_public);
        assert!(gallery_data.gallery.memory_entries.is_empty());
    }

    #[test]
    fn test_gallery_memory_entry_structure() {
        // Test gallery memory entry structure
        let entry = GalleryMemoryEntry {
            memory_id: "test_memory_123".to_string(),
            position: 1,
            gallery_caption: Some("Test Caption".to_string()),
            is_featured: true,
            gallery_metadata: "{}".to_string(),
        };

        assert_eq!(entry.memory_id, "test_memory_123");
        assert_eq!(entry.position, 1);
        assert!(entry.is_featured);
        assert_eq!(entry.gallery_caption, Some("Test Caption".to_string()));
    }

    // Helper function to create test gallery data
    fn create_test_gallery_data() -> GalleryData {
        let mock_time = 1234567890u64; // Mock timestamp for tests
        GalleryData {
            gallery: Gallery {
                id: format!("test_gallery_{}", mock_time),
                owner_principal: candid::Principal::anonymous(),
                title: "Test Gallery".to_string(),
                description: Some("Test Description".to_string()),
                is_public: false,
                created_at: mock_time,
                updated_at: mock_time,
                storage_location: GalleryStorageLocation::Web2Only,
                memory_entries: vec![],
                bound_to_neon: false,
            },
            owner_principal: candid::Principal::anonymous(),
        }
    }
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
