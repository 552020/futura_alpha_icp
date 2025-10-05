//! Memory update operations
//!
//! This module contains the core business logic for updating memories
//! with proper access control and post-write assertions.

use super::traits::*;
use crate::types::{Error, MemoryId, MemoryUpdateData};

/// Core memory update function - pure business logic
pub fn memories_update_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_id: MemoryId,
    updates: MemoryUpdateData,
) -> std::result::Result<crate::types::Memory, Error> {
    // Capture timestamp once for consistency
    let now = env.now();

    // Find the memory across all accessible capsules
    let accessible_capsules = store.get_accessible_capsules(&env.caller());

    for capsule_id in accessible_capsules {
        if let Some(mut memory) = store.get_memory(&capsule_id, &memory_id) {
            // TODO: Add ownership check when we have proper owner tracking
            // For now, if the caller has access to the capsule, they can update memories

            // Apply updates
            if let Some(name) = updates.name {
                memory.metadata.title = Some(name);
            }

            if let Some(metadata) = updates.metadata {
                memory.metadata = metadata;
            }

            if let Some(access) = updates.access {
                memory.access = access;
            }

            // Update timestamp with captured value
            memory.metadata.updated_at = now;

            // Save the updated memory back to the store
            store.insert_memory(&capsule_id, memory)?;

            // POST-WRITE ASSERTION: Verify memory was actually updated
            if let Some(updated_memory) = store.get_memory(&capsule_id, &memory_id) {
                if updated_memory.metadata.updated_at != now {
                    return Err(Error::Internal(
                        "Post-update readback failed: memory was not updated".to_string(),
                    ));
                }
                return Ok(updated_memory);
            } else {
                return Err(Error::Internal(
                    "Post-update readback failed: memory was not found".to_string(),
                ));
            }
        }
    }

    Err(Error::NotFound)
}