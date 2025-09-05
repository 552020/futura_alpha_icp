use crate::capsule_store::{CapsuleStore, Order, Store};
use crate::memory::{with_capsule_store, with_capsule_store_mut};
use crate::types::{CapsuleId, Error, MemoryData, MemoryId, MemoryMeta, PersonRef, Result};
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

pub fn create(capsule_id: CapsuleId, payload: MemoryData, idem: String) -> Result<MemoryId> {
    let caller = PersonRef::from_caller();

    match payload {
        MemoryData::Inline { bytes, meta } => {
            // 1) Size check
            let len_u64 = bytes.len() as u64;
            if len_u64 > INLINE_MAX {
                return Err(Error::InvalidArgument(format!(
                    "inline_too_large: {} > {}",
                    len_u64, INLINE_MAX
                )));
            }

            // 2) Persist bytes -> BlobRef (durable)
            // Trust blob store as single source of truth for hash and length
            let blob_store = BlobStore::new();
            let blob = blob_store
                .put_inline(&bytes)
                .map_err(|e| Error::Internal(format!("blob_store_put_inline: {:?}", e)))?;

            // Use the hash and length from blob store directly
            let sha256 = blob
                .hash
                .ok_or_else(|| Error::Internal("blob_store_did_not_provide_hash".to_string()))?;

            // 3) Single atomic pass: auth + budget + idempotency + insert
            with_capsule_store_mut(|store| {
                // Use update_with for proper error propagation and atomicity
                store
                    .update_with(&capsule_id, |cap| {
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

                        // Use the BlobRef directly (no conversion needed)
                        let types_blob = blob.clone();

                        // Insert the memory
                        cap.insert_memory(
                            &memory_id,
                            types_blob.clone(),
                            meta.clone(),
                            now,
                            Some(idem.clone()),
                        )
                        .map_err(|e| {
                            crate::types::Error::Internal(format!("insert_memory: {:?}", e))
                        })?;

                        // Update inline budget counter for inline uploads
                        cap.inline_bytes_used = cap.inline_bytes_used.saturating_add(blob.len);

                        cap.updated_at = now;

                        // Return the generated ID
                        Ok(memory_id)
                    })
                    .map_err(|e| match e {
                        crate::capsule_store::UpdateError::NotFound => {
                            crate::types::Error::NotFound
                        }
                        crate::capsule_store::UpdateError::Validation(msg) => {
                            crate::types::Error::InvalidArgument(msg)
                        }
                        crate::capsule_store::UpdateError::Concurrency => {
                            crate::types::Error::Internal("concurrency error".into())
                        }
                    })
            })
        }

        MemoryData::BlobRef { blob, meta } => {
            // Verify blob exists and matches provided hash/length
            {
                let blob_store = BlobStore::new();
                let blob_meta = blob_store.head(&blob.locator)?;

                match blob_meta {
                    Some(meta) => {
                        // Verify hash matches
                        if let Some(expected_hash) = &blob.hash {
                            if meta.checksum != *expected_hash {
                                return Err(crate::types::Error::InvalidArgument(
                                    "blob_hash_mismatch".to_string(),
                                ));
                            }
                        }

                        // Verify length matches
                        if meta.size != blob.len {
                            return Err(crate::types::Error::InvalidArgument(
                                "blob_length_mismatch".to_string(),
                            ));
                        }
                    }
                    None => {
                        return Err(crate::types::Error::InvalidArgument(
                            "blob_not_found".to_string(),
                        ));
                    }
                }
            }

            with_capsule_store_mut(|store| {
                // Use update_with for proper error propagation and atomicity
                store
                    .update_with(&capsule_id, |cap| {
                        // First, check for existing memory with same content (idempotency)
                        // Use the actual blob length for proper deduplication
                        if let Some(hash) = &blob.hash {
                            if let Some(existing_id) = find_existing_memory_by_content_in_capsule(
                                cap, hash, blob.len, &idem,
                            ) {
                                return Ok(existing_id); // Return existing memory ID
                            }
                        }
                        // Check authorization
                        if !cap.owners.contains_key(&caller) && cap.subject != caller {
                            return Err(crate::types::Error::Unauthorized);
                        }

                        // Pre-generate the ID
                        let memory_id = generate_memory_id();
                        let now = ic_cdk::api::time();

                        // For blob refs, we don't do budget checks since they're already persisted
                        cap.insert_memory(
                            &memory_id,
                            blob.clone(),
                            meta.clone(),
                            now,
                            Some(idem.clone()),
                        )
                        .map_err(|e| {
                            crate::types::Error::Internal(format!("insert_memory: {:?}", e))
                        })?;

                        cap.updated_at = now;

                        // Return the generated ID
                        Ok(memory_id)
                    })
                    .map_err(|e| match e {
                        crate::capsule_store::UpdateError::NotFound => {
                            crate::types::Error::NotFound
                        }
                        crate::capsule_store::UpdateError::Validation(msg) => {
                            crate::types::Error::InvalidArgument(msg)
                        }
                        crate::capsule_store::UpdateError::Concurrency => {
                            crate::types::Error::Internal("concurrency error".into())
                        }
                    })
            })
        }
    }
}

/// Generate a new unique memory ID
fn generate_memory_id() -> MemoryId {
    format!("mem_{}", ic_cdk::api::time())
}

/// Helper: Check if capsule access is authorized
fn ensure_capsule_access(cap: &crate::types::Capsule, who: &PersonRef) -> Result<()> {
    if cap.owners.contains_key(who) || cap.subject == *who {
        Ok(())
    } else {
        Err(Error::Unauthorized)
    }
}

/// Helper: Find existing memory by content hash, length, and idempotency key within a capsule
/// This version works on a single capsule for use within update_with closures
fn find_existing_memory_by_content_in_capsule(
    cap: &crate::types::Capsule,
    sha256: &[u8; 32],
    len: u64,
    idem: &str,
) -> Option<MemoryId> {
    cap.memories
        .values()
        .find(|memory| {
            // Check idempotency key first (most specific)
            if let Some(ref memory_idem) = memory.idempotency_key {
                if memory_idem == idem {
                    return true; // Same idempotency key = same request
                }
            }

            // Fallback to content-based deduplication
            match &memory.data {
                MemoryData::Inline { bytes, .. } => {
                    let memory_sha256 = compute_sha256(bytes);
                    memory_sha256 == *sha256 && bytes.len() as u64 == len
                }
                MemoryData::BlobRef { blob: mem_blob, .. } => {
                    // For existing blob refs, compare hash AND length
                    if let Some(ref hash) = mem_blob.hash {
                        *hash == *sha256 && mem_blob.len == len
                    } else {
                        false
                    }
                }
            }
        })
        .map(|memory| memory.id.clone())
}

/// Helper: Find existing memory by content hash, length, and idempotency key
fn find_existing_memory_by_content(
    store: &mut Store,
    capsule_id: &CapsuleId,
    sha256: &[u8; 32],
    len: u64,
    idem: &str,
) -> Option<MemoryId> {
    if let Some(capsule) = store.get(capsule_id) {
        capsule
            .memories
            .values()
            .find(|memory| {
                // Check idempotency key first (most specific)
                if let Some(ref memory_idem) = memory.idempotency_key {
                    if memory_idem == idem {
                        return true; // Same idempotency key = same request
                    }
                }

                // Fallback to content-based deduplication
                match &memory.data {
                    MemoryData::Inline { bytes, .. } => {
                        let memory_sha256 = compute_sha256(bytes);
                        memory_sha256 == *sha256 && bytes.len() as u64 == len
                    }
                    MemoryData::BlobRef { blob: mem_blob, .. } => {
                        // For existing blob refs, compare hash AND length
                        if let Some(ref hash) = mem_blob.hash {
                            *hash == *sha256 && mem_blob.len == len
                        } else {
                            false
                        }
                    }
                }
            })
            .map(|memory| memory.id.clone())
    } else {
        None
    }
}

/// Create a memory structure from blob data
fn create_memory_from_blob(
    memory_id: MemoryId,
    blob: crate::types::BlobRef,
    meta: MemoryMeta,
    now: u64,
) -> crate::types::Memory {
    use crate::types::{
        ImageMetadata, Memory, MemoryAccess, MemoryInfo, MemoryMetadata, MemoryMetadataBase,
        MemoryType,
    };

    let memory_info = MemoryInfo {
        memory_type: MemoryType::Image, // Default, can be updated later
        name: meta.name.clone(),
        content_type: "application/octet-stream".to_string(),
        created_at: now,
        updated_at: now,
        uploaded_at: now,
        date_of_memory: None,
    };

    let memory_metadata = MemoryMetadata::Image(ImageMetadata {
        base: MemoryMetadataBase {
            size: blob.len,
            mime_type: "application/octet-stream".to_string(),
            original_name: meta.name.clone(),
            uploaded_at: now.to_string(),
            date_of_memory: None,
            people_in_memory: None,
            format: None,
            bound_to_neon: false,
        },
        dimensions: None,
    });

    let memory_access = MemoryAccess::Private;

    // Use the BlobRef directly (no conversion needed)
    let types_blob_ref = blob.clone();

    let memory_data = MemoryData::BlobRef {
        blob: types_blob_ref,
        meta: meta.clone(),
    };

    Memory {
        id: memory_id,
        info: memory_info,
        metadata: memory_metadata,
        access: memory_access,
        data: memory_data,
        idempotency_key: None, // No idempotency key for helper function
    }
}

pub fn read(memory_id: String) -> crate::types::Result<crate::types::Memory> {
    let caller = PersonRef::from_caller();
    with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.owners.contains_key(&caller) || capsule.subject == caller)
            .and_then(|capsule| capsule.memories.get(&memory_id).cloned())
            .ok_or(crate::types::Error::NotFound)
    })
}

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
                    Some(capsule.memories.values().cloned().collect::<Vec<_>>())
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

// === Metadata & Presence ===

pub fn upsert_metadata(
    memory_id: String,
    memory_type: crate::types::MemoryType,
    metadata: crate::types::SimpleMemoryMetadata,
    idempotency_key: String,
) -> crate::types::Result<crate::types::MetadataResponse> {
    crate::metadata::upsert_metadata(memory_id, memory_type, metadata, idempotency_key)
}

pub fn memories_ping(
    memory_ids: Vec<String>,
) -> crate::types::Result<Vec<crate::types::MemoryPresenceResult>> {
    crate::metadata::memories_ping(memory_ids)
}
pub fn memories_presence(
    memory_ids: Vec<String>,
) -> crate::types::Result<Vec<crate::types::MemoryPresenceResult>> {
    crate::metadata::memories_ping(memory_ids)
}

#[allow(dead_code)]
fn finalize_new_memory(
    _capsule_id: &str,
    _memory: &crate::types::Memory,
) -> crate::types::Result<crate::types::MemoryId> {
    Ok("".to_string())
}
