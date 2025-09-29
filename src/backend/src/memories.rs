use crate::capsule_store::{types::PaginationOrder as Order, CapsuleStore};
use crate::memory::{with_capsule_store, with_capsule_store_mut};
use crate::types::{
    AssetType, BlobRef, CapsuleId, Error, Memory, MemoryAssetBlob, MemoryAssetInline, MemoryId,
    MemoryMeta, MemoryType, PersonRef, Result, StorageEdgeDatabaseType,
};
use crate::upload::blob_store::BlobStore;
use crate::upload::types::{CAPSULE_INLINE_BUDGET, INLINE_MAX};
use ic_cdk::api::time;
use sha2::{Digest, Sha256};

/// Utility function to compute SHA256 hash
fn compute_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Create a Memory object from the given parameters
/// This function handles all the memory construction logic
pub fn create_memory_object(memory_id: &str, blob: BlobRef, meta: MemoryMeta, now: u64) -> Memory {
    use crate::types::{
        ImageMetadata, MemoryAccess, MemoryInfo, MemoryMetadata, MemoryMetadataBase,
    };

    let memory_info = MemoryInfo {
        memory_type: MemoryType::Image, // Default, can be updated later
        name: meta.name.clone(),
        content_type: "application/octet-stream".to_string(),
        created_at: now,
        updated_at: now,
        uploaded_at: now,
        date_of_memory: None,
        parent_folder_id: None, // Default to root folder
        deleted_at: None,       // Default to not deleted
        database_storage_edges: vec![StorageEdgeDatabaseType::Icp], // Default to ICP only
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
            // bound_to_neon removed - now tracked in database_storage_edges
            storage_duration: None, // Default to permanent storage
        },
        dimensions: None,
    });

    let memory_access = MemoryAccess::Private {
        owner_secure_code: format!("mem_{}_{:x}", memory_id, now % 0xFFFF), // Generate secure code
    };

    let blob_asset = MemoryAssetBlob {
        blob,
        meta: meta.clone(),
        asset_type: AssetType::Original,
    };

    Memory {
        id: memory_id.to_string(),
        info: memory_info,
        metadata: memory_metadata,
        access: memory_access,
        inline_assets: vec![],
        blob_assets: vec![blob_asset],
    }
}

pub fn create_inline(
    capsule_id: CapsuleId,
    payload: MemoryAssetInline,
    idem: String,
) -> Result<MemoryId> {
    let caller = PersonRef::from_caller();

    // 1) Size check
    let len_u64 = payload.bytes.len() as u64;
    if len_u64 > INLINE_MAX {
        return Err(Error::InvalidArgument(format!(
            "inline_too_large: {len_u64} > {INLINE_MAX}"
        )));
    }

    // 2) Persist bytes -> BlobRef (durable)
    // Trust blob store as single source of truth for hash and length
    let blob_store = BlobStore::new();
    let blob = blob_store
        .put_inline(&payload.bytes)
        .map_err(|e| Error::Internal(format!("blob_store_put_inline: {e:?}")))?;

    // Use the hash and length from blob store directly
    let sha256 = blob
        .hash
        .ok_or_else(|| Error::Internal("blob_store_did_not_provide_hash".to_string()))?;

    // 3) Single atomic pass: auth + budget + idempotency + insert
    with_capsule_store_mut(|store| {
        // Use update_with for proper error propagation and atomicity
        store.update_with(&capsule_id, |cap| {
            // First, check for existing memory with same content (idempotency)
            // Move this inside the closure for full atomicity
            if let Some(existing_id) =
                find_existing_memory_by_content_in_capsule(cap, &sha256, len_u64, &idem)
            {
                return Ok(existing_id); // Return existing memory ID
            }
            // Check authorization
            if !cap.owners.contains_key(&caller) && cap.subject != caller {
                return Err(crate::types::Error::Unauthorized);
            }

            // Check budget (use maintained counter)
            let used = cap.inline_bytes_used;
            if used.saturating_add(len_u64) > CAPSULE_INLINE_BUDGET {
                return Err(crate::types::Error::ResourceExhausted);
            }

            // Pre-generate the ID
            let memory_id = generate_memory_id();
            let now = ic_cdk::api::time();

            // Create the memory object
            let memory = create_memory_object(&memory_id, blob.clone(), payload.meta.clone(), now);

            // Insert the memory into the capsule
            cap.memories.insert(memory_id.clone(), memory);

            // Update inline budget counter for inline uploads
            cap.inline_bytes_used = used.saturating_add(len_u64);

            // Note: idempotency tracking not supported in current Capsule struct

            // Update timestamps
            cap.updated_at = now;

            // Update owner activity
            if let Some(owner_state) = cap.owners.get_mut(&caller) {
                owner_state.last_activity_at = now;
            }

            // Return the generated ID
            Ok(memory_id)
        })
    })
}

/// Check presence for multiple memories on ICP (consolidated from get_memory_presence_icp and get_memory_list_presence_icp)
pub fn ping(memory_ids: Vec<String>) -> Result<Vec<crate::types::MemoryPresenceResult>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memories_ping_not_found() {
        let memory_id = "nonexistent_memory".to_string();
        let result = ping(vec![memory_id]);

        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(response.len(), 1);
            assert!(!response[0].metadata_present);
            assert!(!response[0].asset_present);
        }
    }

    #[test]
    fn test_memories_ping_empty() {
        let memory_ids = vec![];
        let result = ping(memory_ids);

        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(response.len(), 0);
        }
    }

    #[test]
    fn test_memories_ping_multiple() {
        let memory_ids = vec![
            "mem1".to_string(),
            "mem2".to_string(),
            "mem3".to_string(),
            "mem4".to_string(),
            "mem5".to_string(),
        ];

        let result = ping(memory_ids);
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(response.len(), 5);
            // All memories should not be present (nonexistent)
            for memory_presence in response {
                assert!(!memory_presence.metadata_present);
                assert!(!memory_presence.asset_present);
            }
        }
    }

    #[test]
    fn test_memories_ping_single() {
        let memory_ids = vec!["mem1".to_string()];

        let result = ping(memory_ids);
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(response.len(), 1);
            assert!(!response[0].metadata_present);
            assert!(!response[0].asset_present);
        }
    }
}

/// Generate a new unique memory ID
fn generate_memory_id() -> MemoryId {
    format!("mem_{}", ic_cdk::api::time())
}

// ensure_capsule_access helper removed - authorization logic is inline in create()

/// Helper: Find existing memory by content hash and length within a capsule
/// This version works on a single capsule for use within update_with closures
fn find_existing_memory_by_content_in_capsule(
    cap: &crate::types::Capsule,
    sha256: &[u8; 32],
    len: u64,
    _idem: &str, // Keep parameter for API compatibility but don't use it
) -> Option<MemoryId> {
    cap.memories
        .values()
        .find(|memory| {
            // Content-based deduplication
            // Check inline assets first
            for inline_asset in &memory.inline_assets {
                let memory_sha256 = compute_sha256(&inline_asset.bytes);
                if memory_sha256 == *sha256 && inline_asset.bytes.len() as u64 == len {
                    return true;
                }
            }

            // Check blob assets
            for blob_asset in &memory.blob_assets {
                if let Some(ref hash) = blob_asset.blob.hash {
                    if *hash == *sha256 && blob_asset.blob.len == len {
                        return true;
                    }
                }
            }

            false
        })
        .map(|memory| memory.id.clone())
}

// Legacy find_existing_memory_by_content function removed - using find_existing_memory_by_content_in_capsule instead

// create_memory_from_blob helper removed - not used anywhere

// read function removed - duplicate of memories_read in lib.rs

pub fn update(
    memory_id: String,
    updates: crate::types::MemoryUpdateData,
) -> crate::types::MemoryOperationResponse {
    let caller = PersonRef::from_caller();
    let memory_id_clone = memory_id.clone();
    let mut capsule_found = false;
    let mut memory_found = false;
    with_capsule_store_mut(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        if let Some(capsule) = all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.owners.contains_key(&caller) || capsule.subject == caller)
            .filter(|capsule| capsule.memories.contains_key(&memory_id))
        {
            capsule_found = true;
            let capsule_id = capsule.id.clone();
            let update_result = store.update(&capsule_id, |capsule| {
                if let Some(memory) = capsule.memories.get(&memory_id) {
                    memory_found = true;
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
                    updated_memory.info.updated_at = time();
                    capsule.memories.insert(memory_id.clone(), updated_memory);
                    capsule.updated_at = time();
                }
            });
            if update_result.is_err() {
                capsule_found = false;
            }
        }
    });
    if !capsule_found {
        return crate::types::MemoryOperationResponse {
            success: false,
            memory_id: None,
            message: "No accessible capsule found for caller".to_string(),
        };
    }
    if !memory_found {
        return crate::types::MemoryOperationResponse {
            success: false,
            memory_id: None,
            message: "Memory not found in any accessible capsule".to_string(),
        };
    }
    crate::types::MemoryOperationResponse {
        success: true,
        memory_id: Some(memory_id_clone),
        message: "Memory updated successfully".to_string(),
    }
}

pub fn delete(memory_id: String) -> crate::types::MemoryOperationResponse {
    let caller = PersonRef::from_caller();
    let memory_id_clone = memory_id.clone();
    let mut memory_found = false;
    let mut capsule_found = false;
    with_capsule_store_mut(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        if let Some(capsule) = all_capsules.items.into_iter().find(|capsule| {
            capsule.has_write_access(&caller) && capsule.memories.contains_key(&memory_id)
        }) {
            capsule_found = true;
            let capsule_id = capsule.id.clone();
            let update_result = store.update(&capsule_id, |capsule| {
                if capsule.memories.contains_key(&memory_id) {
                    capsule.memories.remove(&memory_id);
                    capsule.updated_at = time();
                    memory_found = true;
                }
            });
            if update_result.is_err() {
                capsule_found = false;
            }
        }
    });
    if !capsule_found {
        return crate::types::MemoryOperationResponse {
            success: false,
            memory_id: None,
            message: "No accessible capsule found for caller".to_string(),
        };
    }
    if !memory_found {
        return crate::types::MemoryOperationResponse {
            success: false,
            memory_id: None,
            message: "Memory not found in any accessible capsule".to_string(),
        };
    }
    crate::types::MemoryOperationResponse {
        success: true,
        memory_id: Some(memory_id_clone),
        message: "Memory deleted successfully".to_string(),
    }
}

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

/// Read a memory by ID from caller's accessible capsules
pub fn read(memory_id: String) -> Result<crate::types::Memory> {
    use crate::capsule_store::types::PaginationOrder as Order;
    use crate::types::PersonRef;

    let caller = PersonRef::from_caller();

    // Find memory across caller's accessible capsules
    crate::memory::with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .find(|capsule| {
                // Check if caller has access to this capsule
                capsule.owners.contains_key(&caller) || capsule.subject == caller
            })
            .and_then(|capsule| capsule.memories.get(&memory_id).cloned())
            .ok_or(crate::types::Error::NotFound)
    })
}
