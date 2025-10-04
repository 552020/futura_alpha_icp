//! Memory reading operations
//!
//! This module contains the core business logic for reading memories
//! with proper access control and error handling.

use super::traits::*;
use crate::types::{Error, Memory, MemoryId};

/// Core memory reading function - pure business logic
pub fn memories_read_core<E: Env, S: Store>(
    env: &E,
    store: &S,
    memory_id: MemoryId,
) -> std::result::Result<Memory, Error> {
    // Get all accessible capsules for the caller
    let accessible_capsules = store.get_accessible_capsules(&env.caller());

    // Search for the memory across all accessible capsules
    for capsule_id in accessible_capsules {
        if let Some(memory) = store.get_memory(&capsule_id, &memory_id) {
            return Ok(memory);
        }
    }

    Err(Error::NotFound)
}
