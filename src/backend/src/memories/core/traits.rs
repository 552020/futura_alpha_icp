//! Environment and storage abstractions
//!
//! This module defines the core traits for dependency injection,
//! allowing the business logic to be tested independently of ICP-specific APIs.

use crate::capsule_acl::CapsuleAccess;
use crate::types::{CapsuleId, Error, Memory, MemoryId, PersonRef};

/// Environment abstraction for ICP-specific APIs
pub trait Env {
    fn caller(&self) -> PersonRef;
    fn now(&self) -> u64;
}

/// Storage abstraction for capsule store operations
pub trait Store {
    fn insert_memory(
        &mut self,
        capsule: &CapsuleId,
        memory: Memory,
    ) -> std::result::Result<(), Error>;
    fn get_memory(&self, capsule: &CapsuleId, id: &MemoryId) -> Option<Memory>;
    fn delete_memory(
        &mut self,
        capsule: &CapsuleId,
        id: &MemoryId,
    ) -> std::result::Result<(), Error>;
    fn update_memory(
        &mut self,
        capsule: &CapsuleId,
        id: &MemoryId,
        memory: Memory,
    ) -> std::result::Result<(), Error>;
    fn get_accessible_capsules(&self, caller: &PersonRef) -> Vec<CapsuleId>;
    fn get_capsule_for_acl(&self, capsule_id: &CapsuleId) -> Option<CapsuleAccess>;
}
