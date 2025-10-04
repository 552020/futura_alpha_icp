//! Memory creation operations
//!
//! This module contains the core business logic for creating memories
//! with various asset types and storage backends.

use super::{model_helpers::*, traits::*};
use crate::capsule_acl::CapsuleAcl;
use crate::types::{AssetMetadata, BlobRef, CapsuleId, Error, MemoryId, StorageEdgeBlobType};

/// Core memory creation function - pure business logic
///
/// This function contains all the business logic for memory creation
/// without any ICP-specific dependencies. It can be fully unit tested.
pub fn memories_create_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    capsule_id: CapsuleId,
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
    // Validate that exactly one asset type is provided
    let asset_count =
        bytes.is_some() as u8 + blob_ref.is_some() as u8 + external_location.is_some() as u8;
    if asset_count != 1 {
        return Err(Error::InvalidArgument(
            "Exactly one asset type must be provided: bytes, blob_ref, or external_location"
                .to_string(),
        ));
    }

    // Enforce deep asset consistency
    let base = asset_metadata.get_base();
    match (&bytes, &blob_ref, &external_location) {
        (Some(b), None, None) => {
            // inline: base.bytes must equal bytes.len()
            if base.bytes != b.len() as u64 {
                return Err(Error::InvalidArgument(
                    "inline bytes_len != metadata.base.bytes".to_string(),
                ));
            }
        }
        (None, Some(br), None) => {
            // internal blob: base.bytes must equal blob_ref.len
            if base.bytes != br.len {
                return Err(Error::InvalidArgument(
                    "blob_ref.len != metadata.base.bytes".to_string(),
                ));
            }
        }
        (None, None, Some(_loc)) => {
            // external: storage_key must be present
            if external_storage_key.as_deref().unwrap_or("").is_empty() {
                return Err(Error::InvalidArgument(
                    "external_storage_key is required".to_string(),
                ));
            }
            // size consistency (prefer both present and equal)
            if let Some(sz) = external_size {
                if base.bytes != sz {
                    return Err(Error::InvalidArgument(
                        "external_size != metadata.base.bytes".to_string(),
                    ));
                }
            }
            // optional hash consistency
            if let (Some(h), Some(meta_hash)) = (&external_hash, &base.sha256) {
                if h != meta_hash {
                    return Err(Error::InvalidArgument(
                        "external_hash != metadata.base.sha256".to_string(),
                    ));
                }
            }
        }
        _ => {} // already handled by asset_count != 1 above
    }

    // Check if capsule exists and caller has write access using centralized ACL
    let caller = env.caller();
    let capsule_access = store
        .get_capsule_for_acl(&capsule_id)
        .ok_or(Error::NotFound)?;

    if !capsule_access.can_write(&caller) {
        ic_cdk::println!(
            "[ACL] op=create caller={} cap={} read={} write={} delete={} - UNAUTHORIZED",
            caller,
            capsule_id,
            capsule_access.can_read(&caller),
            capsule_access.can_write(&caller),
            capsule_access.can_delete(&caller)
        );
        return Err(Error::Unauthorized);
    }

    // Log successful ACL check
    ic_cdk::println!(
        "[ACL] op=create caller={} cap={} read={} write={} delete={} - AUTHORIZED",
        caller,
        capsule_id,
        capsule_access.can_read(&caller),
        capsule_access.can_write(&caller),
        capsule_access.can_delete(&caller)
    );

    // Capture timestamp once for consistency
    let now = env.now();

    // Generate deterministic memory ID
    let memory_id = format!("mem:{}:{}", &capsule_id, idem);

    // Check for existing memory (idempotency)
    if let Some(_existing) = store.get_memory(&capsule_id, &memory_id) {
        return Ok(memory_id); // Return existing ID for idempotency
    }

    // Create memory based on asset type
    let memory = if let Some(bytes_data) = bytes {
        create_inline_memory(
            &memory_id,
            &capsule_id,
            bytes_data,
            asset_metadata,
            now,
            &caller,
        )
    } else if let Some(blob) = blob_ref {
        create_blob_memory(&memory_id, &capsule_id, blob, asset_metadata, now, &caller)
    } else if let Some(location) = external_location {
        create_external_memory(
            &memory_id,
            &capsule_id,
            location,
            external_storage_key,
            external_url,
            external_size,
            external_hash,
            asset_metadata,
            now,
            &caller,
        )
    } else {
        return Err(Error::InvalidArgument(
            "No valid asset type provided".to_string(),
        ));
    };

    // Insert memory into store
    store.insert_memory(&capsule_id, memory)?;

    // POST-WRITE ASSERTION: Verify memory was actually created
    // This catches silent failures where the function returns Ok but no data is persisted
    if store.get_memory(&capsule_id, &memory_id).is_none() {
        return Err(Error::Internal(
            "Post-write readback failed: memory was not persisted".to_string(),
        ));
    }

    // Debug: Log successful memory creation
    ic_cdk::println!(
        "[DEBUG] memories_create: successfully created memory {} in capsule {}",
        memory_id,
        capsule_id
    );

    Ok(memory_id)
}
