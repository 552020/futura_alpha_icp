use crate::http::core_types::{AssetStore, InlineAsset};
use crate::memories::core::traits::{Env, Store};
use crate::memories::types::AssetType;
use crate::memories::{CanisterEnv, StoreAdapter};
use crate::types::PersonRef;
use candid::Principal;

/// Asset store adapter that bridges to existing asset storage APIs
pub struct FuturaAssetStore;

/// Helper function to map variant strings to AssetType enums
fn variant_to_asset_type(variant: &str) -> Option<AssetType> {
    match variant {
        "display" => Some(AssetType::Display),
        "thumbnail" => Some(AssetType::Thumbnail),
        "original" => Some(AssetType::Original),
        "placeholder" => Some(AssetType::Placeholder),
        "metadata" => Some(AssetType::Metadata),
        _ => None,
    }
}

impl AssetStore for FuturaAssetStore {
    fn get_inline(&self, memory_id: &str, asset_id: &str) -> Option<InlineAsset> {
        // Add debug logging to trace the lookup
        ic_cdk::println!(
            "[ASSET-LOOKUP] memory_id={} asset_id={}",
            memory_id,
            asset_id
        );

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
            }) => {
                ic_cdk::println!(
                    "[ASSET-LOOKUP] ✅ Found inline asset: {} bytes, content_type={}",
                    bytes.len(),
                    content_type
                );
                Some(InlineAsset {
                    bytes,
                    content_type,
                })
            }
            Ok(crate::types::MemoryAssetData::InternalBlob { .. }) => {
                ic_cdk::println!("[ASSET-LOOKUP] ⚠️ Asset found but is blob, not inline");
                None
            }
            Ok(crate::types::MemoryAssetData::ExternalUrl { .. }) => {
                ic_cdk::println!("[ASSET-LOOKUP] ⚠️ Asset found but is external URL, not inline");
                None
            }
            Err(e) => {
                ic_cdk::println!("[ASSET-LOOKUP] ❌ Asset lookup failed: {:?}", e);
                None
            }
        }
    }

    /// Get inline asset using the token's subject principal (not HTTP caller)
    fn get_inline_with_principal(
        &self,
        who: &Principal,
        memory_id: &str,
        asset_id: &str,
    ) -> Option<InlineAsset> {
        ic_cdk::println!(
            "[ASSET-LOOKUP] principal={} mem={} id={}",
            who,
            memory_id,
            asset_id
        );

        let store = StoreAdapter;
        let caps = store.get_accessible_capsules(&PersonRef::Principal(*who));
        ic_cdk::println!("[ASSET-LOOKUP] accessible_capsules={:?}", caps);

        for cap in &caps {
            if let Some(m) = store.get_memory(cap, &memory_id.to_string()) {
                ic_cdk::println!("[ASSET-LOOKUP] found memory in cap={}", cap);
                ic_cdk::println!(
                    "[ASSET-LOOKUP] counts inline={} blob_int={} blob_ext={}",
                    m.inline_assets.len(),
                    m.blob_internal_assets.len(),
                    m.blob_external_assets.len()
                );

                // Check inline assets first
                for asset in &m.inline_assets {
                    ic_cdk::println!(
                        "[ASSET-LOOKUP] inline.id={} ct={}",
                        asset.asset_id,
                        asset.metadata.get_base().mime_type
                    );
                    if asset.asset_id == asset_id {
                        ic_cdk::println!("[ASSET-LOOKUP] ✅ Found matching inline asset");
                        return Some(InlineAsset {
                            bytes: asset.bytes.clone(),
                            content_type: asset.metadata.get_base().mime_type.clone(),
                        });
                    }
                }

                // Check blob internal assets
                for asset in &m.blob_internal_assets {
                    ic_cdk::println!(
                        "[ASSET-LOOKUP] blob_int.id={} ct={}",
                        asset.asset_id,
                        asset.metadata.get_base().mime_type
                    );
                    if asset.asset_id == asset_id {
                        ic_cdk::println!(
                            "[ASSET-LOOKUP] ⚠️ Found matching blob internal asset, but need inline"
                        );
                        return None; // This is a blob, not inline
                    }
                }

                // Check blob external assets
                for asset in &m.blob_external_assets {
                    ic_cdk::println!(
                        "[ASSET-LOOKUP] blob_ext.id={} ct={}",
                        asset.asset_id,
                        asset.metadata.get_base().mime_type
                    );
                    if asset.asset_id == asset_id {
                        ic_cdk::println!(
                            "[ASSET-LOOKUP] ⚠️ Found matching blob external asset, but need inline"
                        );
                        return None; // This is a blob, not inline
                    }
                }
            }
        }
        ic_cdk::println!("[ASSET-LOOKUP] ❌ Asset not found in any accessible capsule");
        None
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

    /// Check if asset exists using the token's subject principal (not HTTP caller)
    fn exists_with_principal(&self, who: &Principal, memory_id: &str, asset_id: &str) -> bool {
        let store = StoreAdapter;
        let caps = store.get_accessible_capsules(&PersonRef::Principal(*who));

        for cap in &caps {
            if let Some(memory) = store.get_memory(cap, &memory_id.to_string()) {
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

    /// Resolve asset for a specific variant, handling variant-to-asset-id mapping
    fn resolve_asset_for_variant(
        &self,
        who: &Principal,
        memory_id: &str,
        variant: &str,
        id_param: Option<&str>,
    ) -> Option<String> {
        ic_cdk::println!(
            "[VARIANT-RESOLVE] principal={} mem={} variant={} id_param={:?}",
            who,
            memory_id,
            variant,
            id_param
        );

        let store = StoreAdapter;
        let caps = store.get_accessible_capsules(&PersonRef::Principal(*who));

        for cap in &caps {
            if let Some(memory) = store.get_memory(cap, &memory_id.to_string()) {
                ic_cdk::println!("[VARIANT-RESOLVE] found memory in cap={}", cap);

                // If specific ID provided, try exact match first
                if let Some(id) = id_param {
                    // Check if the provided ID exists in any asset type
                    if memory.inline_assets.iter().any(|a| a.asset_id == id)
                        || memory.blob_internal_assets.iter().any(|a| a.asset_id == id)
                        || memory.blob_external_assets.iter().any(|a| a.asset_id == id)
                    {
                        ic_cdk::println!("[VARIANT-RESOLVE] ✅ Found exact match for id={}", id);
                        return Some(id.to_string());
                    }

                    // If no exact match, try variant-specific mapping
                    // This is where you'd implement base_id -> variant_id mapping
                    // For now, we'll return None if no exact match
                    ic_cdk::println!("[VARIANT-RESOLVE] ❌ No exact match for id={}", id);
                    return None;
                }

                // If no ID provided, find asset of the requested variant type
                // First, convert variant string to AssetType
                let target_asset_type = match variant_to_asset_type(variant) {
                    Some(asset_type) => {
                        ic_cdk::println!(
                            "[VARIANT-RESOLVE] Looking for asset type: {:?}",
                            asset_type
                        );
                        asset_type
                    }
                    None => {
                        ic_cdk::println!("[VARIANT-RESOLVE] ❌ Unknown variant: {}", variant);
                        return None;
                    }
                };

                // Search for assets with the correct type
                // Priority: inline -> blob_internal -> blob_external

                // Check inline assets first
                for asset in &memory.inline_assets {
                    let asset_type = match &asset.metadata {
                        crate::memories::types::AssetMetadata::Image(img) => &img.base.asset_type,
                        crate::memories::types::AssetMetadata::Video(vid) => &vid.base.asset_type,
                        crate::memories::types::AssetMetadata::Audio(audio) => {
                            &audio.base.asset_type
                        }
                        crate::memories::types::AssetMetadata::Document(doc) => {
                            &doc.base.asset_type
                        }
                        crate::memories::types::AssetMetadata::Note(note) => &note.base.asset_type,
                    };

                    if asset_type == &target_asset_type {
                        ic_cdk::println!(
                            "[VARIANT-RESOLVE] ✅ Found matching inline asset: {} (type: {:?})",
                            asset.asset_id,
                            asset_type
                        );
                        return Some(asset.asset_id.clone());
                    }
                }

                // Check blob internal assets
                for asset in &memory.blob_internal_assets {
                    let asset_type = match &asset.metadata {
                        crate::memories::types::AssetMetadata::Image(img) => &img.base.asset_type,
                        crate::memories::types::AssetMetadata::Video(vid) => &vid.base.asset_type,
                        crate::memories::types::AssetMetadata::Audio(audio) => {
                            &audio.base.asset_type
                        }
                        crate::memories::types::AssetMetadata::Document(doc) => {
                            &doc.base.asset_type
                        }
                        crate::memories::types::AssetMetadata::Note(note) => &note.base.asset_type,
                    };

                    if asset_type == &target_asset_type {
                        ic_cdk::println!(
                            "[VARIANT-RESOLVE] ✅ Found matching blob internal asset: {} (type: {:?})",
                            asset.asset_id, asset_type
                        );
                        return Some(asset.asset_id.clone());
                    }
                }

                // Check blob external assets
                for asset in &memory.blob_external_assets {
                    let asset_type = match &asset.metadata {
                        crate::memories::types::AssetMetadata::Image(img) => &img.base.asset_type,
                        crate::memories::types::AssetMetadata::Video(vid) => &vid.base.asset_type,
                        crate::memories::types::AssetMetadata::Audio(audio) => {
                            &audio.base.asset_type
                        }
                        crate::memories::types::AssetMetadata::Document(doc) => {
                            &doc.base.asset_type
                        }
                        crate::memories::types::AssetMetadata::Note(note) => &note.base.asset_type,
                    };

                    if asset_type == &target_asset_type {
                        ic_cdk::println!(
                            "[VARIANT-RESOLVE] ✅ Found matching blob external asset: {} (type: {:?})",
                            asset.asset_id, asset_type
                        );
                        return Some(asset.asset_id.clone());
                    }
                }

                ic_cdk::println!(
                    "[VARIANT-RESOLVE] ❌ No asset found for variant: {} (type: {:?})",
                    variant,
                    target_asset_type
                );
            }
        }

        ic_cdk::println!("[VARIANT-RESOLVE] ❌ No memory found or no assets in memory");
        None
    }

    /// Get blob asset using the token's subject principal (not HTTP caller)
    fn get_blob_with_principal(
        &self,
        who: &Principal,
        memory_id: &str,
        asset_id: &str,
    ) -> Option<(Vec<u8>, String)> {
        ic_cdk::println!(
            "[BLOB-LOOKUP] principal={} mem={} id={}",
            who,
            memory_id,
            asset_id
        );

        let store = StoreAdapter;
        let caps = store.get_accessible_capsules(&PersonRef::Principal(*who));

        for cap in &caps {
            if let Some(memory) = store.get_memory(cap, &memory_id.to_string()) {
                // Check blob internal assets first
                for asset in &memory.blob_internal_assets {
                    if asset.asset_id == asset_id {
                        ic_cdk::println!("[BLOB-LOOKUP] ✅ Found matching blob internal asset");

                        // For Phase 1, only handle blobs <= 2MB
                        if asset.blob_ref.len <= 2 * 1024 * 1024 {
                            // Read the full blob
                            match crate::upload::blob_store::blob_read_chunk(
                                asset.blob_ref.locator.clone(),
                                0, // Read from beginning
                            ) {
                                Ok(data) => {
                                    ic_cdk::println!(
                                        "[BLOB-LOOKUP] ✅ Read {} bytes from blob",
                                        data.len()
                                    );
                                    return Some((
                                        data,
                                        asset.metadata.get_base().mime_type.clone(),
                                    ));
                                }
                                Err(e) => {
                                    ic_cdk::println!(
                                        "[BLOB-LOOKUP] ❌ Failed to read blob: {:?}",
                                        e
                                    );
                                    return None;
                                }
                            }
                        } else {
                            ic_cdk::println!(
                                "[BLOB-LOOKUP] ⚠️ Blob too large for inline serving: {} bytes",
                                asset.blob_ref.len
                            );
                            return None; // Too large for Phase 1
                        }
                    }
                }

                // Check blob external assets
                for asset in &memory.blob_external_assets {
                    if asset.asset_id == asset_id {
                        ic_cdk::println!(
                            "[BLOB-LOOKUP] ⚠️ Found blob external asset, not supported in Phase 1"
                        );
                        return None; // External URLs not supported in Phase 1
                    }
                }
            }
        }

        ic_cdk::println!("[BLOB-LOOKUP] ❌ Blob not found in any accessible capsule");
        None
    }
}
