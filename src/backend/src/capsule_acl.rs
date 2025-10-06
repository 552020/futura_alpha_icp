//! Centralized Access Control Logic for Capsules
//!
//! This module provides a single source of truth for capsule access control,
//! eliminating inconsistencies between different memory operations.

use crate::types::{ControllerState, OwnerState, PersonRef};
use std::collections::HashMap;

/// Capsule access control methods
pub trait CapsuleAcl {
    /// Check if a person can read from this capsule
    ///
    /// Read access: owners ∨ controllers ∨ subject
    fn can_read(&self, person: &PersonRef) -> bool;

    /// Check if a person can write/create in this capsule
    ///
    /// Write access: owners ∨ controllers ∨ subject
    fn can_write(&self, person: &PersonRef) -> bool;

    /// Check if a person can delete from this capsule
    ///
    /// Delete access: owners ∨ controllers (subject cannot delete by default)
    fn can_delete(&self, person: &PersonRef) -> bool;
}

/// Helper struct for capsule access control
///
/// This can be implemented by any struct that has the necessary fields
/// for access control (subject, owners, controllers)
#[derive(Clone)]
pub struct CapsuleAccess {
    pub subject: PersonRef,
    pub owners: HashMap<PersonRef, OwnerState>,
    pub controllers: HashMap<PersonRef, ControllerState>,
}

impl CapsuleAccess {
    pub fn new(
        subject: PersonRef,
        owners: HashMap<PersonRef, OwnerState>,
        controllers: HashMap<PersonRef, ControllerState>,
    ) -> Self {
        Self {
            subject,
            owners,
            controllers,
        }
    }
}

impl CapsuleAcl for CapsuleAccess {
    #[inline]
    fn can_read(&self, person: &PersonRef) -> bool {
        self.owners.contains_key(person)
            || self.controllers.contains_key(person)
            || self.subject == *person
    }

    #[inline]
    fn can_write(&self, person: &PersonRef) -> bool {
        // Subject can create memories
        self.owners.contains_key(person)
            || self.controllers.contains_key(person)
            || self.subject == *person
    }

    #[inline]
    fn can_delete(&self, person: &PersonRef) -> bool {
        // Keep delete stricter - only owners and controllers can delete
        // Subject cannot delete by default (can be changed if needed)
        self.owners.contains_key(person) || self.controllers.contains_key(person)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PersonRef;
    use candid::Principal;
    use std::collections::BTreeMap;

    fn create_test_person(id: &str) -> PersonRef {
        // Create a valid Principal from a simple string
        let principal = Principal::from_text(id).unwrap_or_else(|_| {
            // If parsing fails, create a Principal from bytes
            let bytes = id.as_bytes();
            let mut principal_bytes = [0u8; 29];
            let len = bytes.len().min(29);
            principal_bytes[..len].copy_from_slice(&bytes[..len]);
            Principal::from_slice(&principal_bytes)
        });
        PersonRef::Principal(principal)
    }

    #[test]
    fn test_owner_access() {
        let owner = create_test_person("owner-principal");
        let subject = create_test_person("subject-principal");
        let other = create_test_person("other-principal");

        let mut owners = HashMap::new();
        owners.insert(
            owner.clone(),
            OwnerState {
                since: 0,
                last_activity_at: 0,
            },
        );

        let access = CapsuleAccess::new(subject, owners, HashMap::new());

        assert!(access.can_read(&owner));
        assert!(access.can_write(&owner));
        assert!(access.can_delete(&owner));

        assert!(!access.can_read(&other));
        assert!(!access.can_write(&other));
        assert!(!access.can_delete(&other));
    }

    #[test]
    fn test_subject_access() {
        let owner = create_test_person("owner-principal");
        let subject = create_test_person("subject-principal");
        let other = create_test_person("other-principal");

        let mut owners = HashMap::new();
        owners.insert(
            owner,
            OwnerState {
                since: 0,
                last_activity_at: 0,
            },
        );

        let access = CapsuleAccess::new(subject.clone(), owners, HashMap::new());

        // Subject can read and write but not delete
        assert!(access.can_read(&subject));
        assert!(access.can_write(&subject));
        assert!(!access.can_delete(&subject));

        assert!(!access.can_read(&other));
        assert!(!access.can_write(&other));
        assert!(!access.can_delete(&other));
    }

    #[test]
    fn test_controller_access() {
        let owner = create_test_person("owner-principal");
        let controller = create_test_person("controller-principal");
        let subject = create_test_person("subject-principal");

        let mut owners = HashMap::new();
        owners.insert(
            owner,
            OwnerState {
                since: 0,
                last_activity_at: 0,
            },
        );

        let mut controllers = HashMap::new();
        controllers.insert(
            controller.clone(),
            ControllerState {
                granted_at: 0,
                granted_by: create_test_person("owner-principal"),
            },
        );

        let access = CapsuleAccess::new(subject, owners, controllers);

        // Controller has full access
        assert!(access.can_read(&controller));
        assert!(access.can_write(&controller));
        assert!(access.can_delete(&controller));
    }
}
