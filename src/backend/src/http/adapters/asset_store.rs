use crate::http::core_types::{AssetStore, InlineAsset};
use crate::memories::core::traits::{Env, Store};
use crate::memories::{CanisterEnv, StoreAdapter};

/// Asset store adapter that bridges to existing asset storage APIs
pub struct FuturaAssetStore;

impl AssetStore for FuturaAssetStore {
    fn get_inline(&self, memory_id: &str, asset_id: &str) -> Option<InlineAsset> {
        // Use existing asset_get_by_id_core logic
        let env = CanisterEnv;
        let store = StoreAdapter;

        match crate::memories::core::assets::asset_get_by_id_core(
            &env,
            &store,
            memory_id.to_string(),
            asset_id.to_string(),
        ) {
            Ok(crate::types::MemoryAssetData::Inline {
                bytes,
                content_type,
                ..
            }) => Some(InlineAsset {
                bytes,
                content_type,
            }),
            _ => None,
        }
    }

    fn get_blob_len(&self, memory_id: &str, asset_id: &str) -> Option<(u64, String)> {
        // Get blob metadata without loading the full blob
        let env = CanisterEnv;
        let store = StoreAdapter;

        match crate::memories::core::assets::asset_get_by_id_core(
            &env,
            &store,
            memory_id.to_string(),
            asset_id.to_string(),
        ) {
            Ok(crate::types::MemoryAssetData::InternalBlob { blob_id, size, .. }) => {
                // Get chunk size from blob metadata
                let blob_store = crate::upload::blob_store::BlobStore::new();
                match blob_store
                    .get_blob_meta(&crate::upload::types::BlobId(blob_id.parse().unwrap_or(0)))
                {
                    Ok(Some(meta)) => Some((size, meta.size.to_string())), // Use size as chunk size for now
                    _ => None,
                }
            }
            Ok(crate::types::MemoryAssetData::ExternalUrl { size, .. }) => {
                // External URLs - assume standard chunk size
                Some((size.unwrap_or(0), "1048576".to_string())) // 1MB default
            }
            _ => None,
        }
    }

    fn read_blob_chunk(
        &self,
        memory_id: &str,
        asset_id: &str,
        offset: u64,
        len: u64,
    ) -> Option<Vec<u8>> {
        // Read chunk from blob store
        let env = CanisterEnv;
        let store = StoreAdapter;

        match crate::memories::core::assets::asset_get_by_id_core(
            &env,
            &store,
            memory_id.to_string(),
            asset_id.to_string(),
        ) {
            Ok(crate::types::MemoryAssetData::InternalBlob { blob_id, .. }) => {
                // Read chunk from internal blob store
                let _blob_store = crate::upload::blob_store::BlobStore::new();
                match crate::upload::blob_store::blob_read_chunk(
                    blob_id.clone(),
                    (offset / 1024) as u32,
                ) {
                    Ok(data) => Some(data),
                    Err(_) => None,
                }
            }
            Ok(crate::types::MemoryAssetData::ExternalUrl { url, .. }) => {
                // For external URLs, we'd need to implement HTTP range requests
                // For now, return None (external URL streaming not implemented)
                let _ = (url, offset, len);
                None
            }
            _ => None,
        }
    }

    fn exists(&self, memory_id: &str, asset_id: &str) -> bool {
        // Check existence without loading full data
        let env = CanisterEnv;
        let store = StoreAdapter;

        // Get all accessible capsules for the caller
        let accessible_capsules = store.get_accessible_capsules(&env.caller());

        // Search for the asset across all accessible capsules
        for capsule_id in accessible_capsules {
            if let Some(memory) = store.get_memory(&capsule_id, &memory_id.to_string()) {
                // Check inline assets
                if memory
                    .inline_assets
                    .iter()
                    .any(|asset| asset.asset_id == asset_id)
                {
                    return true;
                }
                // Check blob internal assets
                if memory
                    .blob_internal_assets
                    .iter()
                    .any(|asset| asset.asset_id == asset_id)
                {
                    return true;
                }
                // Check blob external assets
                if memory
                    .blob_external_assets
                    .iter()
                    .any(|asset| asset.asset_id == asset_id)
                {
                    return true;
                }
            }
        }
        false
    }
}
