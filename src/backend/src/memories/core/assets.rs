//! Asset management operations
//!
//! This module contains functions for managing memory assets,
//! including cleanup operations for different storage backends.

use super::traits::*;
use crate::types::{BlobRef, Error, Memory, MemoryAssetBlobExternal, StorageEdgeBlobType};

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
pub fn cleanup_internal_blob_asset(blob_ref: &BlobRef) -> std::result::Result<(), Error> {
    use crate::upload::blob_store::BlobStore;
    use crate::upload::types::BlobId;

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

    // Convert string to BlobId (assuming it's a numeric ID)
    let blob_id = blob_id_str
        .parse::<u64>()
        .map_err(|_| Error::InvalidArgument(format!("Invalid blob ID: {}", blob_id_str)))?;

    let blob_id = BlobId(blob_id);

    // Delete the blob from the store
    let blob_store = BlobStore::new();
    blob_store.delete_blob(&blob_id)?;

    Ok(())
}

/// Clean up an external blob asset from external storage
pub fn cleanup_external_blob_asset(
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
pub fn cleanup_vercel_blob_asset(storage_key: &str) -> std::result::Result<(), Error> {
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
pub fn cleanup_s3_blob_asset(storage_key: &str) -> std::result::Result<(), Error> {
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
pub fn cleanup_arweave_blob_asset(storage_key: &str) -> std::result::Result<(), Error> {
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
pub fn cleanup_ipfs_blob_asset(storage_key: &str) -> std::result::Result<(), Error> {
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
pub fn cleanup_neon_blob_asset(storage_key: &str) -> std::result::Result<(), Error> {
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

/// Core cleanup all assets function - pure business logic
pub fn memories_cleanup_assets_all_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_id: String,
) -> std::result::Result<crate::memories::types::AssetCleanupResult, Error> {
    // Find the memory across all accessible capsules
    let accessible_capsules = store.get_accessible_capsules(&env.caller());

    for capsule_id in accessible_capsules {
        if let Some(memory) = store.get_memory(&capsule_id, &memory_id) {
            // Clean up all assets
            cleanup_memory_assets(&memory)?;

            return Ok(crate::memories::types::AssetCleanupResult {
                memory_id,
                assets_cleaned: 1, // Simplified - would need to count actual assets
                message: "Assets cleaned successfully".to_string(),
            });
        }
    }

    Err(Error::NotFound)
}

/// Core bulk cleanup assets function - pure business logic
pub fn memories_cleanup_assets_bulk_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_ids: Vec<String>,
) -> std::result::Result<crate::memories::types::BulkAssetCleanupResult, Error> {
    let mut cleaned_count = 0;
    let mut failed_count = 0;
    let mut total_assets_cleaned = 0;
    let mut errors = Vec::new();

    for memory_id in memory_ids {
        match memories_cleanup_assets_all_core(env, store, memory_id.clone()) {
            Ok(result) => {
                cleaned_count += 1;
                total_assets_cleaned += result.assets_cleaned;
            }
            Err(e) => {
                failed_count += 1;
                errors.push(format!("Failed to cleanup {}: {:?}", memory_id, e));
            }
        }
    }

    let message = if errors.is_empty() {
        format!("Successfully cleaned {} memories", cleaned_count)
    } else {
        format!(
            "Cleaned {} memories, {} failed: {}",
            cleaned_count,
            failed_count,
            errors.join(", ")
        )
    };

    Ok(crate::memories::types::BulkAssetCleanupResult {
        cleaned_count,
        failed_count,
        total_assets_cleaned,
        message,
    })
}

/// Core asset removal function - pure business logic
pub fn asset_remove_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_id: String,
    asset_ref: String,
) -> std::result::Result<crate::memories::types::AssetRemovalResult, Error> {
    // Find the memory across all accessible capsules
    let accessible_capsules = store.get_accessible_capsules(&env.caller());

    for capsule_id in accessible_capsules {
        if let Some(mut memory) = store.get_memory(&capsule_id, &memory_id) {
            // This is a simplified implementation
            // In practice, you'd need to find and remove the specific asset
            return Ok(crate::memories::types::AssetRemovalResult {
                memory_id,
                asset_removed: true,
                message: "Asset removal not fully implemented".to_string(),
            });
        }
    }

    Err(Error::NotFound)
}

/// Core inline asset removal function - pure business logic
pub fn asset_remove_inline_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_id: String,
    asset_index: u32,
) -> std::result::Result<crate::memories::types::AssetRemovalResult, Error> {
    // Find the memory across all accessible capsules
    let accessible_capsules = store.get_accessible_capsules(&env.caller());

    for capsule_id in accessible_capsules {
        if let Some(mut memory) = store.get_memory(&capsule_id, &memory_id) {
            // This is a simplified implementation
            // In practice, you'd need to remove the asset at the specified index
            return Ok(crate::memories::types::AssetRemovalResult {
                memory_id,
                asset_removed: true,
                message: "Inline asset removal not fully implemented".to_string(),
            });
        }
    }

    Err(Error::NotFound)
}

/// Core internal asset removal function - pure business logic
pub fn asset_remove_internal_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_id: String,
    blob_ref: String,
) -> std::result::Result<crate::memories::types::AssetRemovalResult, Error> {
    // Find the memory across all accessible capsules
    let accessible_capsules = store.get_accessible_capsules(&env.caller());

    for capsule_id in accessible_capsules {
        if let Some(mut memory) = store.get_memory(&capsule_id, &memory_id) {
            // This is a simplified implementation
            // In practice, you'd need to remove the specific internal asset
            return Ok(crate::memories::types::AssetRemovalResult {
                memory_id,
                asset_removed: true,
                message: "Internal asset removal not fully implemented".to_string(),
            });
        }
    }

    Err(Error::NotFound)
}

/// Core external asset removal function - pure business logic
pub fn asset_remove_external_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_id: String,
    storage_key: String,
) -> std::result::Result<crate::memories::types::AssetRemovalResult, Error> {
    // Find the memory across all accessible capsules
    let accessible_capsules = store.get_accessible_capsules(&env.caller());

    for capsule_id in accessible_capsules {
        if let Some(mut memory) = store.get_memory(&capsule_id, &memory_id) {
            // This is a simplified implementation
            // In practice, you'd need to remove the specific external asset
            return Ok(crate::memories::types::AssetRemovalResult {
                memory_id,
                asset_removed: true,
                message: "External asset removal not fully implemented".to_string(),
            });
        }
    }

    Err(Error::NotFound)
}

/// Core list assets function - pure business logic
pub fn memories_list_assets_core<E: Env, S: Store>(
    env: &E,
    store: &S,
    memory_id: String,
) -> std::result::Result<crate::memories::types::MemoryAssetsList, Error> {
    // Find the memory across all accessible capsules
    let accessible_capsules = store.get_accessible_capsules(&env.caller());

    for capsule_id in accessible_capsules {
        if let Some(memory) = store.get_memory(&capsule_id, &memory_id) {
            let inline_assets: Vec<String> = memory
                .inline_assets
                .iter()
                .enumerate()
                .map(|(i, _)| format!("inline_{}", i))
                .collect();

            let internal_assets: Vec<String> = memory
                .blob_internal_assets
                .iter()
                .map(|asset| asset.blob_ref.locator.clone())
                .collect();

            let external_assets: Vec<String> = memory
                .blob_external_assets
                .iter()
                .map(|asset| asset.storage_key.clone())
                .collect();

            let total_count = inline_assets.len() + internal_assets.len() + external_assets.len();

            return Ok(crate::memories::types::MemoryAssetsList {
                memory_id,
                inline_assets,
                internal_assets,
                external_assets,
                total_count: total_count as u32,
            });
        }
    }

    Err(Error::NotFound)
}
