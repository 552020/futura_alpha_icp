//! Memory management module with decoupled architecture
//!
//! This module provides the canister-facing functions that route through
//! the decoupled core functions for better testability and maintainability.

use crate::capsule_store::{types::PaginationOrder, CapsuleStore};
use crate::memory::{with_capsule_store, with_capsule_store_mut};
use crate::types::{
    AssetMetadata, BlobRef, CapsuleId, Error, Memory, MemoryId, MemoryOperationResponse,
    MemoryUpdateData, PersonRef, StorageEdgeBlobType,
};

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
    fn get_capsule_mut(&mut self, _id: &CapsuleId) -> Option<crate::memories_core::CapsuleRefMut> {
        // Unused; kept for future bulk operations
        None
    }

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
            let all_capsules = store.paginate(None, u32::MAX, PaginationOrder::Asc);
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
// THIN CANISTER WRAPPERS
// ============================================================================

/// Create a memory with any type of asset (inline, blob, or external)
pub fn memories_create(
    capsule_id: String,
    bytes: Option<Vec<u8>>,
    blob_ref: Option<BlobRef>,
    external_location: Option<StorageEdgeBlobType>,
    external_storage_key: Option<String>,
    external_url: Option<String>,
    external_size: Option<u64>,
    external_hash: Option<Vec<u8>>,
    asset_metadata: AssetMetadata,
    idem: String,
) -> std::result::Result<MemoryId, Error> {
    let env = CanisterEnv;
    let mut store = StoreAdapter;
    crate::memories_core::memories_create_core(
        &env,
        &mut store,
        capsule_id,
        bytes,
        blob_ref,
        external_location,
        external_storage_key,
        external_url,
        external_size,
        external_hash,
        asset_metadata,
        idem,
    )
}

/// Read a memory by ID from caller's accessible capsules
pub fn memories_read(memory_id: String) -> std::result::Result<Memory, Error> {
    let env = CanisterEnv;
    let store = StoreAdapter;
    crate::memories_core::memories_read_core(&env, &store, memory_id)
}

/// Update a memory's metadata
pub fn memories_update(memory_id: String, updates: MemoryUpdateData) -> MemoryOperationResponse {
    let env = CanisterEnv;
    let mut store = StoreAdapter;
    match crate::memories_core::memories_update_core(&env, &mut store, memory_id.clone(), updates) {
        Ok(()) => MemoryOperationResponse {
            memory_id: Some(memory_id),
            message: "Memory updated successfully".to_string(),
            success: true,
        },
        Err(e) => MemoryOperationResponse {
            memory_id: None,
            message: format!("Failed to update memory: {:?}", e),
            success: false,
        },
    }
}

/// Delete a memory
pub fn memories_delete(memory_id: String) -> MemoryOperationResponse {
    let env = CanisterEnv;
    let mut store = StoreAdapter;
    match crate::memories_core::memories_delete_core(&env, &mut store, memory_id.clone()) {
        Ok(()) => MemoryOperationResponse {
            memory_id: Some(memory_id),
            message: "Memory deleted successfully".to_string(),
            success: true,
        },
        Err(e) => MemoryOperationResponse {
            memory_id: None,
            message: format!("Failed to delete memory: {:?}", e),
            success: false,
        },
    }
}

// ============================================================================
// LEGACY FUNCTIONS (for backward compatibility with existing tests)
// ============================================================================

/// Check presence for multiple memories on ICP
pub fn ping(
    memory_ids: Vec<String>,
) -> std::result::Result<Vec<crate::types::MemoryPresenceResult>, Error> {
    // TODO: Implement memory presence checking using capsule system instead of artifacts
    // For now, return false for all memories since artifacts system is removed
    let results: Vec<crate::types::MemoryPresenceResult> = memory_ids
        .iter()
        .map(|memory_id| crate::types::MemoryPresenceResult {
            memory_id: memory_id.clone(),
            metadata_present: false, // Artifacts system removed
            asset_present: false,    // Artifacts system removed
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
