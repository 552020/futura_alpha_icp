use crate::memory::{with_capsules, with_capsules_mut};
use crate::types::*;
use candid::Principal;
use ic_cdk::api::{msg_caller, time};
use std::collections::HashMap;

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
        let capsule_id = format!("capsule_{}", now);

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
            bound_to_web2: false, // Initially not bound to Web2
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

/// Create a new capsule
pub fn create_capsule(subject: PersonRef) -> CapsuleCreationResult {
    let caller = PersonRef::from_caller();
    let capsule = Capsule::new(subject, caller);
    let capsule_id = capsule.id.clone();

    with_capsules_mut(|capsules| {
        capsules.insert(capsule_id.clone(), capsule);
    });

    CapsuleCreationResult {
        success: true,
        capsule_id: Some(capsule_id),
        message: "Capsule created".to_string(),
    }
}

/// Get capsule by ID (with read access check)
pub fn get_capsule(capsule_id: String) -> Option<Capsule> {
    let caller = PersonRef::from_caller();

    with_capsules(|capsules| {
        capsules
            .get(&capsule_id)
            .filter(|capsule| capsule.has_write_access(&caller))
            .cloned()
    })
}

/// List capsules owned or controlled by caller
pub fn list_my_capsules() -> Vec<CapsuleHeader> {
    let caller = PersonRef::from_caller();

    with_capsules(|capsules| {
        capsules
            .values()
            .filter(|capsule| capsule.has_write_access(&caller))
            .map(|capsule| capsule.to_header())
            .collect()
    })
}

/// Register capsule for current user (idempotent self-registration)
/// Creates a capsule where the caller is both subject and owner
pub fn register_capsule() -> CapsuleRegistrationResult {
    let caller_ref = PersonRef::from_caller();

    // Check if caller already has a self-capsule (subject == owner == caller)
    let existing_self_capsule = with_capsules(|capsules| {
        capsules
            .values()
            .find(|capsule| {
                capsule.subject == caller_ref && capsule.owners.contains_key(&caller_ref)
            })
            .cloned()
    });

    match existing_self_capsule {
        Some(mut capsule) => {
            // Update owner activity and capsule timestamp
            let now = time();
            capsule.updated_at = now;

            if let Some(owner_state) = capsule.owners.get_mut(&caller_ref) {
                owner_state.last_activity_at = now;
            }

            // Store the updated capsule
            with_capsules_mut(|capsules| {
                capsules.insert(capsule.id.clone(), capsule.clone());
            });

            CapsuleRegistrationResult {
                success: true,
                capsule_id: Some(capsule.id),
                is_new: false,
                message: "Welcome back! Your capsule is ready.".to_string(),
            }
        }
        None => {
            // Create new self-capsule (subject = caller, owner = caller automatically)
            match create_capsule(caller_ref.clone()) {
                CapsuleCreationResult {
                    success: true,
                    capsule_id,
                    ..
                } => CapsuleRegistrationResult {
                    success: true,
                    capsule_id,
                    is_new: true,
                    message: "Capsule created successfully!".to_string(),
                },
                CapsuleCreationResult {
                    success: false,
                    message,
                    ..
                } => CapsuleRegistrationResult {
                    success: false,
                    capsule_id: None,
                    is_new: false,
                    message: format!("Failed to create capsule: {}", message),
                },
            }
        }
    }
}

/// Simple user registration for II integration (idempotent)
/// Tracks basic user info: { registered_at, last_activity_at, bound: bool }
pub fn register() -> bool {
    let caller_ref = PersonRef::from_caller();
    let now = time();

    // Check if user already has a self-capsule
    let existing_self_capsule = with_capsules(|capsules| {
        capsules
            .values()
            .find(|capsule| {
                capsule.subject == caller_ref && capsule.owners.contains_key(&caller_ref)
            })
            .cloned()
    });

    match existing_self_capsule {
        Some(mut capsule) => {
            // Update activity timestamp
            if let Some(owner_state) = capsule.owners.get_mut(&caller_ref) {
                owner_state.last_activity_at = now;
            }
            capsule.updated_at = now;

            // Store the updated capsule
            with_capsules_mut(|capsules| {
                capsules.insert(capsule.id.clone(), capsule);
            });
            true
        }
        None => {
            // Create new self-capsule with basic info
            match create_capsule(caller_ref.clone()) {
                CapsuleCreationResult { success: true, .. } => true,
                CapsuleCreationResult { success: false, .. } => false,
            }
        }
    }
}

/// Simple binding function for II integration
/// Sets bound=true for the caller's self-capsule
pub fn mark_bound() -> bool {
    let caller_ref = PersonRef::from_caller();

    with_capsules_mut(|capsules| {
        // Find caller's self-capsule (where caller is both subject and owner)
        for capsule in capsules.values_mut() {
            if capsule.subject == caller_ref && capsule.owners.contains_key(&caller_ref) {
                capsule.bound_to_web2 = true;
                capsule.updated_at = time();

                // Update owner activity too
                if let Some(owner_state) = capsule.owners.get_mut(&caller_ref) {
                    owner_state.last_activity_at = time();
                }

                return true;
            }
        }
        false // No self-capsule found
    })
}

/// Mark caller's self-capsule as bound to Web2
/// Called after successful NextAuth authentication
pub fn mark_capsule_bound_to_web2() -> bool {
    let caller_ref = PersonRef::from_caller();

    with_capsules_mut(|capsules| {
        // Find caller's self-capsule (where caller is both subject and owner)
        for capsule in capsules.values_mut() {
            if capsule.subject == caller_ref && capsule.owners.contains_key(&caller_ref) {
                capsule.bound_to_web2 = true;
                capsule.updated_at = time();

                // Update owner activity too
                if let Some(owner_state) = capsule.owners.get_mut(&caller_ref) {
                    owner_state.last_activity_at = time();
                }

                return true;
            }
        }
        false // No self-capsule found
    })
}

/// Export all capsules for upgrade persistence
pub fn export_capsules_for_upgrade() -> Vec<(String, Capsule)> {
    with_capsules(|capsules| {
        capsules
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    })
}

/// Import capsules from upgrade persistence
pub fn import_capsules_from_upgrade(capsule_data: Vec<(String, Capsule)>) {
    with_capsules_mut(|capsules| {
        *capsules = capsule_data.into_iter().collect();
    })
}

// ============================================================================
// GALLERY MANAGEMENT FUNCTIONS
// ============================================================================

/// Store a gallery in the caller's capsule
pub fn store_gallery_forever(gallery_data: GalleryData) -> StoreGalleryResponse {
    let caller = PersonRef::from_caller();

    // Use the gallery ID provided by Web2 (don't generate new ID)
    let gallery_id = gallery_data.gallery.id.clone();

    // Find caller's self-capsule (where caller is both subject and owner)
    let capsule = with_capsules(|capsules| {
        capsules
            .values()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller))
            .cloned()
    });

    match capsule {
        Some(mut capsule) => {
            // Check if gallery already exists with this UUID (idempotency)
            if let Some(_existing_gallery) = capsule.galleries.get(&gallery_id) {
                return StoreGalleryResponse {
                    success: true,
                    gallery_id: Some(gallery_id.clone()),
                    icp_gallery_id: Some(gallery_id),
                    message: "Gallery already exists with this UUID".to_string(),
                    storage_status: GalleryStorageStatus::ICPOnly,
                };
            }

            // Create gallery from data (don't overwrite gallery.id - it's already set by Web2)
            let mut gallery = gallery_data.gallery;
            gallery.owner_principal = match caller {
                PersonRef::Principal(p) => p,
                PersonRef::Opaque(_) => {
                    return StoreGalleryResponse {
                        success: false,
                        gallery_id: None,
                        icp_gallery_id: None,
                        message: "Only principals can store galleries".to_string(),
                        storage_status: GalleryStorageStatus::Failed,
                    }
                }
            };
            gallery.created_at = ic_cdk::api::time();
            gallery.updated_at = ic_cdk::api::time();
            gallery.storage_status = GalleryStorageStatus::ICPOnly;

            // Store gallery in capsule
            capsule.galleries.insert(gallery_id.clone(), gallery);
            capsule.touch(); // Update capsule timestamp

            // Save updated capsule
            with_capsules_mut(|capsules| {
                capsules.insert(capsule.id.clone(), capsule);
            });

            StoreGalleryResponse {
                success: true,
                gallery_id: Some(gallery_id.clone()),
                icp_gallery_id: Some(gallery_id),
                message: "Gallery stored successfully in capsule".to_string(),
                storage_status: GalleryStorageStatus::ICPOnly,
            }
        }
        None => StoreGalleryResponse {
            success: false,
            gallery_id: None,
            icp_gallery_id: None,
            message: "No capsule found for caller".to_string(),
            storage_status: GalleryStorageStatus::Failed,
        },
    }
}

/// Get all galleries for a user principal
pub fn get_user_galleries(user_principal: Principal) -> Vec<Gallery> {
    let person_ref = PersonRef::Principal(user_principal);

    with_capsules(|capsules| {
        capsules
            .values()
            .filter(|capsule| {
                capsule.subject == person_ref && capsule.owners.contains_key(&person_ref)
            })
            .flat_map(|capsule| capsule.galleries.values().cloned().collect::<Vec<_>>())
            .collect()
    })
}

/// Get gallery by ID from caller's capsule
pub fn get_gallery_by_id(gallery_id: String) -> Option<Gallery> {
    let caller = PersonRef::from_caller();

    with_capsules(|capsules| {
        capsules
            .values()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller))
            .and_then(|capsule| capsule.galleries.get(&gallery_id).cloned())
    })
}

/// Update a gallery in caller's capsule
pub fn update_gallery(gallery_id: String, update_data: GalleryUpdateData) -> UpdateGalleryResponse {
    let caller = PersonRef::from_caller();

    // Find caller's self-capsule and get a mutable reference
    let mut capsule_found = false;
    let mut updated_gallery: Option<Gallery> = None;

    with_capsules_mut(|capsules| {
        if let Some(capsule) = capsules
            .values_mut()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller))
        {
            capsule_found = true;

            // Check if gallery exists
            if let Some(gallery) = capsule.galleries.get(&gallery_id) {
                // Update gallery fields
                let mut gallery_clone = gallery.clone();
                if let Some(title) = update_data.title.clone() {
                    gallery_clone.title = title;
                }
                if let Some(description) = update_data.description.clone() {
                    gallery_clone.description = Some(description);
                }
                if let Some(is_public) = update_data.is_public {
                    gallery_clone.is_public = is_public;
                }
                if let Some(memory_entries) = update_data.memory_entries.clone() {
                    gallery_clone.memory_entries = memory_entries;
                }

                gallery_clone.updated_at = ic_cdk::api::time();

                // Store updated gallery
                capsule.galleries.insert(gallery_id, gallery_clone.clone());
                capsule.touch(); // Update capsule timestamp

                updated_gallery = Some(gallery_clone);
            }
        }
    });

    if !capsule_found {
        return UpdateGalleryResponse {
            success: false,
            gallery: None,
            message: "No capsule found for caller".to_string(),
        };
    }

    match updated_gallery {
        Some(gallery) => UpdateGalleryResponse {
            success: true,
            gallery: Some(gallery),
            message: "Gallery updated successfully".to_string(),
        },
        None => UpdateGalleryResponse {
            success: false,
            gallery: None,
            message: "Gallery not found".to_string(),
        },
    }
}

/// Delete a gallery from caller's capsule
pub fn delete_gallery(gallery_id: String) -> DeleteGalleryResponse {
    let caller = PersonRef::from_caller();

    // Find caller's self-capsule and perform deletion
    let mut capsule_found = false;
    let mut gallery_found = false;

    with_capsules_mut(|capsules| {
        if let Some(capsule) = capsules
            .values_mut()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller))
        {
            capsule_found = true;

            // Check if gallery exists and remove it
            if capsule.galleries.contains_key(&gallery_id) {
                capsule.galleries.remove(&gallery_id);
                capsule.touch(); // Update capsule timestamp
                gallery_found = true;
            }
        }
    });

    if !capsule_found {
        return DeleteGalleryResponse {
            success: false,
            message: "No capsule found for caller".to_string(),
        };
    }

    if !gallery_found {
        return DeleteGalleryResponse {
            success: false,
            message: "Gallery not found".to_string(),
        };
    }

    DeleteGalleryResponse {
        success: true,
        message: "Gallery deleted successfully".to_string(),
    }
}

// ============================================================================
// MEMORY MANAGEMENT FUNCTIONS
// ============================================================================
// NOTE: Memory functions are kept in capsule.rs because memories are always
// stored within capsules. This could be refactored into a separate memories.rs
// module in the future if the capsule.rs file becomes too large or if we need
// more complex memory-specific logic.
// ============================================================================

/// Add a memory to the caller's capsule
pub fn add_memory_to_capsule(
    memory_id: String,
    memory_data: MemoryData,
) -> MemoryOperationResponse {
    let caller = PersonRef::from_caller();

    // Find caller's self-capsule
    let capsule = with_capsules(|capsules| {
        capsules
            .values()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller))
            .cloned()
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
                name: format!("Memory {}", memory_id),
                content_type: "application/octet-stream".to_string(),
                created_at: now,
                updated_at: now,
                uploaded_at: now,
                date_of_memory: None,
            };

            // Create memory metadata (default to Image type)
            let memory_metadata = MemoryMetadata::Image(ImageMetadata {
                base: MemoryMetadataBase {
                    size: memory_data
                        .data
                        .as_ref()
                        .map(|d| d.len() as u64)
                        .unwrap_or(0),
                    mime_type: "application/octet-stream".to_string(),
                    original_name: format!("Memory {}", memory_id),
                    uploaded_at: now.to_string(),
                    date_of_memory: None,
                    people_in_memory: None,
                    format: None,
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
            };

            // Store memory in capsule
            capsule.memories.insert(memory_id.clone(), memory);
            capsule.touch(); // Update capsule timestamp

            // Save updated capsule
            with_capsules_mut(|capsules| {
                capsules.insert(capsule.id.clone(), capsule);
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

/// Get a memory from the caller's capsule
pub fn get_memory_from_capsule(memory_id: String) -> Option<Memory> {
    let caller = PersonRef::from_caller();

    with_capsules(|capsules| {
        capsules
            .values()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller))
            .and_then(|capsule| capsule.memories.get(&memory_id).cloned())
    })
}

/// Update a memory in the caller's capsule
pub fn update_memory_in_capsule(
    memory_id: String,
    updates: MemoryUpdateData,
) -> MemoryOperationResponse {
    let caller = PersonRef::from_caller();

    // Find caller's self-capsule and perform update
    let mut capsule_found = false;
    let mut memory_found = false;
    let memory_id_clone = memory_id.clone();

    with_capsules_mut(|capsules| {
        if let Some(capsule) = capsules
            .values_mut()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller))
        {
            capsule_found = true;

            // Check if memory exists
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
                capsule.touch(); // Update capsule timestamp
            }
        }
    });

    if !capsule_found {
        return MemoryOperationResponse {
            success: false,
            memory_id: None,
            message: "No capsule found for caller".to_string(),
        };
    }

    if !memory_found {
        return MemoryOperationResponse {
            success: false,
            memory_id: None,
            message: "Memory not found".to_string(),
        };
    }

    MemoryOperationResponse {
        success: true,
        memory_id: Some(memory_id_clone),
        message: "Memory updated successfully".to_string(),
    }
}

/// Delete a memory from the caller's capsule
pub fn delete_memory_from_capsule(memory_id: String) -> MemoryOperationResponse {
    let caller = PersonRef::from_caller();

    // Find caller's self-capsule and perform deletion
    let mut capsule_found = false;
    let mut memory_found = false;
    let memory_id_clone = memory_id.clone();

    with_capsules_mut(|capsules| {
        if let Some(capsule) = capsules
            .values_mut()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller))
        {
            capsule_found = true;

            // Check if memory exists and remove it
            if capsule.memories.contains_key(&memory_id) {
                capsule.memories.remove(&memory_id);
                capsule.touch(); // Update capsule timestamp
                memory_found = true;
            }
        }
    });

    if !capsule_found {
        return MemoryOperationResponse {
            success: false,
            memory_id: None,
            message: "No capsule found for caller".to_string(),
        };
    }

    if !memory_found {
        return MemoryOperationResponse {
            success: false,
            memory_id: None,
            message: "Memory not found".to_string(),
        };
    }

    MemoryOperationResponse {
        success: true,
        memory_id: Some(memory_id_clone),
        message: "Memory deleted successfully".to_string(),
    }
}

/// List all memories in the caller's capsule
pub fn list_capsule_memories() -> MemoryListResponse {
    let caller = PersonRef::from_caller();

    let memories = with_capsules(|capsules| {
        capsules
            .values()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller))
            .map(|capsule| capsule.memories.values().cloned().collect::<Vec<_>>())
            .unwrap_or_default()
    });

    MemoryListResponse {
        success: true,
        memories,
        message: "Memories retrieved successfully".to_string(),
    }
}

/// Get capsule information for the caller
/// Returns capsule info if caller is owner/controller of any capsule
pub fn get_capsule_info() -> Option<CapsuleInfo> {
    let caller_ref = PersonRef::from_caller();

    with_capsules(|capsules| {
        capsules
            .values()
            .find(|capsule| {
                // Check if caller is owner or controller
                capsule.owners.contains_key(&caller_ref)
                    || capsule.controllers.contains_key(&caller_ref)
            })
            .map(|capsule| {
                // Check if this is caller's self-capsule (subject == caller)
                let is_self_capsule = capsule.subject == caller_ref;

                CapsuleInfo {
                    capsule_id: capsule.id.clone(),
                    subject: capsule.subject.clone(),
                    is_owner: capsule.owners.contains_key(&caller_ref),
                    is_controller: capsule.controllers.contains_key(&caller_ref),
                    is_self_capsule,
                    bound_to_web2: capsule.bound_to_web2,
                    created_at: capsule.created_at,
                    updated_at: capsule.updated_at,
                }
            })
    })
}
