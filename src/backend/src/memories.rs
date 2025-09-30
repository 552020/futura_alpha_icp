//! Memory management module with decoupled architecture
//!
//! This module provides the canister-facing functions that route through
//! the decoupled core functions for better testability and maintainability.

use crate::capsule_store::{types::PaginationOrder as Order, CapsuleStore};
use crate::memory::{with_capsule_store, with_capsule_store_mut};
use crate::types::{CapsuleId, Error, Memory, MemoryId, PersonRef};

// ============================================================================
// CANISTER ENVIRONMENT AND STORE ADAPTER
// ============================================================================

/// Canister environment implementation for production use
pub struct CanisterEnv;

impl crate::memories_core::Env for CanisterEnv {
    fn caller(&self) -> PersonRef {
        PersonRef::Principal(ic_cdk::api::msg_caller())
    }

    fn now(&self) -> u64 {
        ic_cdk::api::time()
    }
}

/// Production store adapter that bridges the Store trait with CapsuleStore
pub struct StoreAdapter;

impl crate::memories_core::Store for StoreAdapter {
    // Removed unused method: get_capsule_mut

    fn insert_memory(
        &mut self,
        capsule: &CapsuleId,
        memory: Memory,
    ) -> std::result::Result<(), Error> {
        with_capsule_store_mut(|store| {
            match store.update_with(capsule, |capsule_data| {
                // Check if memory already exists
                if capsule_data.memories.contains_key(&memory.id) {
                    return Err(Error::Conflict(format!(
                        "Memory {} already exists in capsule {}",
                        memory.id, capsule
                    )));
                }

                // Insert the memory
                capsule_data.memories.insert(memory.id.clone(), memory);
                Ok(())
            }) {
                Ok(_) => Ok(()),
                Err(e) => Err(Error::Internal(format!("Failed to insert memory: {:?}", e))),
            }
        })
    }

    fn get_memory(&self, capsule: &CapsuleId, id: &MemoryId) -> Option<Memory> {
        with_capsule_store(|store| {
            store
                .get(capsule)
                .and_then(|capsule| capsule.memories.get(id).cloned())
        })
    }

    fn delete_memory(
        &mut self,
        capsule: &CapsuleId,
        id: &MemoryId,
    ) -> std::result::Result<(), Error> {
        with_capsule_store_mut(|store| {
            match store.update_with(capsule, |capsule_data| {
                match capsule_data.memories.remove(id) {
                    Some(_) => Ok(()),
                    None => Err(Error::NotFound),
                }
            }) {
                Ok(_) => Ok(()),
                Err(e) => Err(Error::Internal(format!("Failed to delete memory: {:?}", e))),
            }
        })
    }

    fn get_accessible_capsules(&self, caller: &PersonRef) -> Vec<CapsuleId> {
        with_capsule_store(|store| {
            let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
            all_capsules
                .items
                .into_iter()
                .filter(|capsule| capsule.has_write_access(caller))
                .map(|capsule| capsule.id)
                .collect()
        })
    }
}

// ============================================================================
// MEMORY UTILITY FUNCTIONS
// ============================================================================

/// Check presence for multiple memories on ICP
/// 
/// Returns presence status for the given memory IDs by checking if they exist
/// in the caller's accessible capsules.
pub fn ping(
    memory_ids: Vec<String>,
) -> std::result::Result<Vec<crate::types::MemoryPresenceResult>, Error> {
    let caller = PersonRef::from_caller();
    
    let results: Vec<crate::types::MemoryPresenceResult> = memory_ids
        .iter()
        .map(|memory_id| {
            // Check if memory exists in any of the caller's accessible capsules
            let exists = with_capsule_store(|store| {
                let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
                all_capsules
                    .items
                    .iter()
                    .any(|capsule| {
                        // Check if caller has read access to this capsule
                        capsule.has_read_access(&caller) && 
                        // Check if memory exists in this capsule
                        capsule.memories.contains_key(memory_id)
                    })
            });
            
            crate::types::MemoryPresenceResult {
                memory_id: memory_id.clone(),
                metadata_present: exists,
                asset_present: exists, // For now, assume if metadata exists, asset exists
            }
        })
        .collect();

    Ok(results)
}

/// List memories in a capsule
pub fn list(capsule_id: String) -> crate::types::MemoryListResponse {
    let caller = PersonRef::from_caller();
    let memories = with_capsule_store(|store| {
        store
            .get(&capsule_id)
            .and_then(|capsule| {
                if capsule.owners.contains_key(&caller) || capsule.subject == caller {
                    Some(
                        capsule
                            .memories
                            .values()
                            .map(|memory| memory.to_header())
                            .collect::<Vec<_>>(),
                    )
                } else {
                    None
                }
            })
            .unwrap_or_default()
    });
    crate::types::MemoryListResponse {
        success: true,
        memories,
        message: "Memories retrieved successfully".to_string(),
    }
}
