use crate::memory::{with_capsules, with_capsules_mut};
use crate::types::*;
use ic_cdk::api::{msg_caller, time};
use std::cell::RefCell;
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
                capsules
                    .insert(capsule.id.clone(), capsule.clone());
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
