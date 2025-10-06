//! Memory deletion operations
//!
//! This module contains the core business logic for deleting memories
//! with proper asset cleanup and post-write assertions.

use super::traits::*;
use crate::capsule_acl::CapsuleAcl;
use crate::types::{
    BlobRef, Error, Memory, MemoryAssetBlobExternal, MemoryId, StorageEdgeBlobType,
};

/// Core memory deletion function - pure business logic
pub fn memories_delete_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_id: MemoryId,
    delete_assets: bool,
) -> std::result::Result<(), Error> {
    let caller = env.caller();

    // Find the memory across all accessible capsules
    let accessible_capsules = store.get_accessible_capsules(&caller);

    for capsule_id in accessible_capsules {
        if let Some(memory) = store.get_memory(&capsule_id, &memory_id) {
            // Check delete permissions using centralized ACL
            let capsule_access = store
                .get_capsule_for_acl(&capsule_id)
                .ok_or(Error::NotFound)?;

            if !capsule_access.can_delete(&caller) {
                ic_cdk::println!(
                    "[ACL] op=delete caller={} cap={} read={} write={} delete={} - UNAUTHORIZED",
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
                "[ACL] op=delete caller={} cap={} read={} write={} delete={} - AUTHORIZED",
                caller,
                capsule_id,
                capsule_access.can_read(&caller),
                capsule_access.can_write(&caller),
                capsule_access.can_delete(&caller)
            );

            // CRITICAL: Clean up assets before deleting the memory (if requested)
            if delete_assets {
                cleanup_memory_assets(&memory)?;
            }

            // Delete the memory
            store.delete_memory(&capsule_id, &memory_id)?;

            // POST-WRITE ASSERTION: Verify memory was actually deleted
            if store.get_memory(&capsule_id, &memory_id).is_some() {
                return Err(Error::Internal(
                    "Post-delete readback failed: memory was not removed".to_string(),
                ));
            }

            return Ok(());
        }
    }

    Err(Error::NotFound)
}

/// Core bulk memory deletion function - pure business logic
pub fn memories_delete_bulk_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    capsule_id: String,
    memory_ids: Vec<String>,
    delete_assets: bool,
) -> std::result::Result<crate::memories::types::BulkDeleteResult, Error> {
    let caller = env.caller();
    let mut deleted_count = 0;
    let mut failed_count = 0;
    let mut errors = Vec::new();

    // Check if capsule exists and caller has write access
    let capsule_access = store
        .get_capsule_for_acl(&capsule_id)
        .ok_or(Error::NotFound)?;

    if !capsule_access.can_write(&caller) {
        return Err(Error::Unauthorized);
    }

    // Delete each memory
    for memory_id in memory_ids {
        match memories_delete_core(env, store, memory_id.clone(), delete_assets) {
            Ok(_) => {
                deleted_count += 1;
            }
            Err(e) => {
                failed_count += 1;
                errors.push(format!("Failed to delete {}: {:?}", memory_id, e));
            }
        }
    }

    let message = if errors.is_empty() {
        format!("Successfully deleted {} memories", deleted_count)
    } else {
        format!(
            "Deleted {} memories, {} failed: {}",
            deleted_count,
            failed_count,
            errors.join(", ")
        )
    };

    Ok(crate::memories::types::BulkDeleteResult {
        deleted_count,
        failed_count,
        message,
    })
}

/// Core delete all memories function - pure business logic
pub fn memories_delete_all_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    capsule_id: String,
    delete_assets: bool,
) -> std::result::Result<crate::memories::types::BulkDeleteResult, Error> {
    let caller = env.caller();

    // Check if capsule exists and caller has write access
    let capsule_access = store
        .get_capsule_for_acl(&capsule_id)
        .ok_or(Error::NotFound)?;

    if !capsule_access.can_write(&caller) {
        return Err(Error::Unauthorized);
    }

    // Get all accessible capsules for the caller
    let accessible_capsules = store.get_accessible_capsules(&caller);

    let mut deleted_count = 0;
    let mut failed_count = 0;
    let mut errors: Vec<String> = Vec::new();

    // Find and delete all memories in the capsule
    for accessible_capsule_id in accessible_capsules {
        if accessible_capsule_id == capsule_id {
            // Get all memories in the capsule and delete them
            let memories = store.get_all_memories(&capsule_id);
            let memory_ids: Vec<String> = memories.iter().map(|m| m.id.clone()).collect();

            return memories_delete_bulk_core(env, store, capsule_id, memory_ids, delete_assets);
        }
    }

    Ok(crate::memories::types::BulkDeleteResult {
        deleted_count,
        failed_count,
        message: format!("Deleted {} memories", deleted_count),
    })
}

/// Clean up all assets associated with a memory before deletion
/// This prevents memory leaks and storage bloat
pub fn cleanup_memory_assets(memory: &Memory) -> std::result::Result<(), Error> {
    // 1. Inline assets: No cleanup needed - they're stored directly in the memory struct
    // When the memory is deleted, inline assets are automatically removed

    // 2. Internal blob assets: Delete from ICP blob store
    for blob_asset in &memory.blob_internal_assets {
        cleanup_internal_blob_asset(&blob_asset.blob_ref)?;
    }

    // 3. External blob assets: Delete from external storage
    for external_asset in &memory.blob_external_assets {
        cleanup_external_blob_asset(external_asset)?;
    }

    Ok(())
}

/// Clean up an internal blob asset from ICP blob store
fn cleanup_internal_blob_asset(blob_ref: &BlobRef) -> std::result::Result<(), Error> {
    use crate::upload::blob_store::BlobStore;
    use crate::upload::types::BlobId;
    use crate::util::blob_id::parse_blob_id;

    // Parse the blob locator to get the BlobId
    // Format: "canister_id:blob_id" or just "blob_id"
    let blob_id_str = if blob_ref.locator.contains(':') {
        blob_ref
            .locator
            .split(':')
            .nth(1)
            .unwrap_or(&blob_ref.locator)
    } else {
        &blob_ref.locator
    };

    // Add loud, temporary logging to debug exact blob ID strings
    ic_cdk::println!(
        "[cleanup_internal_blob_asset] raw='{}' (len={}) bytes={:?}",
        blob_id_str,
        blob_id_str.len(),
        blob_id_str.as_bytes()
    );

    // Use the bulletproof parser
    let blob_id_num = parse_blob_id(blob_id_str).map_err(|e| Error::InvalidArgument(e))?;

    let blob_id = BlobId(blob_id_num);

    // Delete the blob from the store
    let blob_store = BlobStore::new();
    blob_store.delete_blob(&blob_id)?;

    Ok(())
}

/// Clean up an external blob asset from external storage
fn cleanup_external_blob_asset(
    external_asset: &MemoryAssetBlobExternal,
) -> std::result::Result<(), Error> {
    match &external_asset.location {
        StorageEdgeBlobType::Icp => {
            // This shouldn't happen - ICP assets should be in blob_internal_assets
            return Err(Error::InvalidArgument(
                "ICP assets should be in blob_internal_assets".to_string(),
            ));
        }
        StorageEdgeBlobType::VercelBlob => {
            cleanup_vercel_blob_asset(&external_asset.storage_key)?;
        }
        StorageEdgeBlobType::S3 => {
            cleanup_s3_blob_asset(&external_asset.storage_key)?;
        }
        StorageEdgeBlobType::Arweave => {
            cleanup_arweave_blob_asset(&external_asset.storage_key)?;
        }
        StorageEdgeBlobType::Ipfs => {
            cleanup_ipfs_blob_asset(&external_asset.storage_key)?;
        }
        StorageEdgeBlobType::Neon => {
            cleanup_neon_blob_asset(&external_asset.storage_key)?;
        }
    }

    Ok(())
}

/// Clean up a Vercel Blob asset
fn cleanup_vercel_blob_asset(storage_key: &str) -> std::result::Result<(), Error> {
    // TODO: Implement Vercel Blob deletion via HTTP outcall
    // This would require:
    // 1. Making an HTTP outcall to Vercel Blob API
    // 2. Using the storage_key to delete the blob
    // 3. Handling authentication and error responses

    // For now, log the deletion attempt
    ic_cdk::println!("TODO: Delete Vercel Blob asset: {}", storage_key);

    // Return success for now to avoid breaking the deletion flow
    // In production, this should be implemented properly
    Ok(())
}

/// Clean up an S3 asset
fn cleanup_s3_blob_asset(storage_key: &str) -> std::result::Result<(), Error> {
    // TODO: Implement S3 deletion via HTTP outcall
    // This would require:
    // 1. Making an HTTP outcall to S3 API
    // 2. Using the storage_key to delete the object
    // 3. Handling AWS authentication and error responses

    // For now, log the deletion attempt
    ic_cdk::println!("TODO: Delete S3 asset: {}", storage_key);

    // Return success for now to avoid breaking the deletion flow
    // In production, this should be implemented properly
    Ok(())
}

/// Clean up an Arweave asset
fn cleanup_arweave_blob_asset(storage_key: &str) -> std::result::Result<(), Error> {
    // TODO: Implement Arweave deletion
    // Note: Arweave is designed to be permanent storage
    // Deletion might not be possible or might require special handling

    // For now, log the deletion attempt
    ic_cdk::println!("TODO: Delete Arweave asset: {}", storage_key);

    // Return success for now to avoid breaking the deletion flow
    // In production, this should be implemented properly
    Ok(())
}

/// Clean up an IPFS asset
fn cleanup_ipfs_blob_asset(storage_key: &str) -> std::result::Result<(), Error> {
    // TODO: Implement IPFS deletion
    // Note: IPFS is designed to be permanent storage
    // Deletion might not be possible or might require special handling

    // For now, log the deletion attempt
    ic_cdk::println!("TODO: Delete IPFS asset: {}", storage_key);

    // Return success for now to avoid breaking the deletion flow
    // In production, this should be implemented properly
    Ok(())
}

/// Clean up a Neon database asset
fn cleanup_neon_blob_asset(storage_key: &str) -> std::result::Result<(), Error> {
    // TODO: Implement Neon database asset deletion
    // This would require:
    // 1. Making an HTTP outcall to Neon API
    // 2. Using the storage_key to delete the asset record
    // 3. Handling database authentication and error responses

    // For now, log the deletion attempt
    ic_cdk::println!("TODO: Delete Neon asset: {}", storage_key);

    // Return success for now to avoid breaking the deletion flow
    // In production, this should be implemented properly
    Ok(())
}
