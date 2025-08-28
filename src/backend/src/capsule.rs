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
        owners.insert(initial_owner, OwnerState { since: now });

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
            MemoryAccess::Custom { individuals, groups } => {
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
            MemoryAccess::Custom { individuals, groups } => {
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

// Thread-local storage for capsules (centralized for now)
thread_local! {
    static CAPSULES: RefCell<HashMap<String, Capsule>> = RefCell::new(HashMap::new());
}

/// Create a new capsule
pub fn create_capsule(subject: PersonRef) -> CapsuleCreationResult {
    let caller = PersonRef::from_caller();
    let capsule = Capsule::new(subject, caller);
    let capsule_id = capsule.id.clone();

    CAPSULES.with(|capsules| {
        capsules.borrow_mut().insert(capsule_id.clone(), capsule);
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

    CAPSULES.with(|capsules| {
        capsules
            .borrow()
            .get(&capsule_id)
            .filter(|capsule| capsule.has_write_access(&caller))
            .cloned()
    })
}

/// List capsules owned or controlled by caller
pub fn list_my_capsules() -> Vec<CapsuleHeader> {
    let caller = PersonRef::from_caller();

    CAPSULES.with(|capsules| {
        capsules
            .borrow()
            .values()
            .filter(|capsule| capsule.has_write_access(&caller))
            .map(|capsule| capsule.to_header())
            .collect()
    })
}

/// Export all capsules for upgrade persistence
pub fn export_capsules_for_upgrade() -> Vec<(String, Capsule)> {
    CAPSULES.with(|capsules| {
        capsules
            .borrow()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    })
}

/// Import capsules from upgrade persistence
pub fn import_capsules_from_upgrade(capsule_data: Vec<(String, Capsule)>) {
    CAPSULES.with(|capsules| {
        *capsules.borrow_mut() = capsule_data.into_iter().collect();
    })
}
