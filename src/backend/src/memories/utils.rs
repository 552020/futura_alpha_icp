use crate::http::token_service::TokenService;
use crate::memories::types::Memory;
use crate::types::MemoryHeader;
use base64::{engine::general_purpose, Engine as _};
use candid::CandidType;
use serde::{Deserialize, Serialize};

/// Convert AssetKind to string for URL paths
fn asset_kind_to_string(kind: AssetKind) -> &'static str {
    match kind {
        AssetKind::Thumbnail => "thumbnail",
        AssetKind::Display => "display",
        AssetKind::Original => "original",
    }
}

/// Canonical enum for asset types the frontend actually uses
#[derive(Serialize, Deserialize, CandidType, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetKind {
    Thumbnail,
    Display,
    Original,
}

/// Lightweight summary per asset kind exposed to the frontend
#[derive(Serialize, Deserialize, CandidType, Clone, Debug)]
pub struct AssetLink {
    pub path: String,         // e.g. "/asset/{memory_id}/{asset_id}"
    pub token: String,        // opaque, audience/resource-bound
    pub expires_at_ns: u128,  // for client refresh strategy
    pub content_type: String, // e.g. "image/webp"
    pub width: Option<u32>,   // hint for layout
    pub height: Option<u32>,  // hint for layout
    pub bytes: Option<u64>,   // size hint
    pub asset_kind: AssetKind,
    pub asset_id: String,     // allows later direct fetch of full asset record
    pub etag: Option<String>, // hex hash if available (from BlobRef.hash)
}

/// Grouped by kind for simple frontend access patterns
#[derive(Serialize, Deserialize, CandidType, Clone, Debug, Default)]
pub struct AssetLinks {
    pub thumbnail: Option<AssetLink>,
    pub display: Option<AssetLink>,
    pub original: Option<AssetLink>,
}

/// Build a single asset link with token metadata
fn build_asset_link(
    memory_id: &str,
    asset_id: &str,
    asset_kind: AssetKind,
    content_type: &str,
    width: Option<u32>,
    height: Option<u32>,
    bytes: Option<u64>,
    etag: Option<String>,
    ttl_seconds: u32,
) -> AssetLink {
    let path = format!("/asset/{}/{}", memory_id, asset_kind_to_string(asset_kind));

    ic_cdk::println!(
        "[TOKEN-GEN] Minting token for memory_id={} asset_id={} asset_kind={:?} ttl={}",
        memory_id,
        asset_id,
        asset_kind,
        ttl_seconds
    );

    let token = TokenService::mint_token(
        memory_id.to_string(),
        vec![asset_kind_to_string(asset_kind).to_string()],
        Some(vec![asset_id.to_string()]),
        ttl_seconds,
    );

    ic_cdk::println!(
        "[TOKEN-GEN] ✅ Token minted successfully: {}... (first 50 chars)",
        &token[..token.len().min(50)]
    );
    let expires_at_ns = (ic_cdk::api::time() + (ttl_seconds as u64 * 1_000_000_000)) as u128;

    AssetLink {
        path,
        token,
        expires_at_ns,
        content_type: content_type.to_string(),
        width,
        height,
        bytes,
        asset_kind,
        asset_id: asset_id.to_string(),
        etag,
    }
}

/// Generate asset links with tokens for memory headers
/// Always returns all available asset links (thumbnail, display, original) if present
pub fn generate_asset_links_for_memory_header(
    mut header: MemoryHeader,
    memory: &Memory,
) -> MemoryHeader {
    // Use extended TTL for memory listings (30 minutes = 1800 seconds)
    // This allows users to browse memories for longer periods
    const MEMORY_LISTING_TTL: u32 = 1800; // 30 minutes

    // Helper function to find asset by type
    fn find_asset_by_type(
        memory: &Memory,
        asset_type: crate::memories::types::AssetType,
    ) -> Option<(&str, &crate::memories::types::AssetMetadata)> {
        // Check blob_internal_assets first
        for asset in &memory.blob_internal_assets {
            let matches_type = match &asset.metadata {
                crate::memories::types::AssetMetadata::Image(img) => {
                    img.base.asset_type == asset_type
                }
                crate::memories::types::AssetMetadata::Video(vid) => {
                    vid.base.asset_type == asset_type
                }
                crate::memories::types::AssetMetadata::Audio(audio) => {
                    audio.base.asset_type == asset_type
                }
                crate::memories::types::AssetMetadata::Document(doc) => {
                    doc.base.asset_type == asset_type
                }
                crate::memories::types::AssetMetadata::Note(note) => {
                    note.base.asset_type == asset_type
                }
            };
            if matches_type {
                return Some((&asset.asset_id, &asset.metadata));
            }
        }

        // Then check inline_assets
        for asset in &memory.inline_assets {
            let matches_type = match &asset.metadata {
                crate::memories::types::AssetMetadata::Image(img) => {
                    img.base.asset_type == asset_type
                }
                crate::memories::types::AssetMetadata::Video(vid) => {
                    vid.base.asset_type == asset_type
                }
                crate::memories::types::AssetMetadata::Audio(audio) => {
                    audio.base.asset_type == asset_type
                }
                crate::memories::types::AssetMetadata::Document(doc) => {
                    doc.base.asset_type == asset_type
                }
                crate::memories::types::AssetMetadata::Note(note) => {
                    note.base.asset_type == asset_type
                }
            };
            if matches_type {
                return Some((&asset.asset_id, &asset.metadata));
            }
        }

        None
    }

    // Helper function to extract metadata from asset
    fn extract_asset_metadata(
        metadata: &crate::memories::types::AssetMetadata,
    ) -> (
        String,
        Option<u32>,
        Option<u32>,
        Option<u64>,
        Option<String>,
    ) {
        match metadata {
            crate::memories::types::AssetMetadata::Image(img) => (
                img.base.mime_type.clone(),
                img.base.width,
                img.base.height,
                Some(img.base.bytes),
                img.base.sha256.map(|hash| hex::encode(hash)),
            ),
            crate::memories::types::AssetMetadata::Video(vid) => (
                vid.base.mime_type.clone(),
                vid.base.width,
                vid.base.height,
                Some(vid.base.bytes),
                vid.base.sha256.map(|hash| hex::encode(hash)),
            ),
            crate::memories::types::AssetMetadata::Audio(audio) => (
                audio.base.mime_type.clone(),
                None,
                None,
                Some(audio.base.bytes),
                audio.base.sha256.map(|hash| hex::encode(hash)),
            ),
            crate::memories::types::AssetMetadata::Document(doc) => (
                doc.base.mime_type.clone(),
                None,
                None,
                Some(doc.base.bytes),
                doc.base.sha256.map(|hash| hex::encode(hash)),
            ),
            crate::memories::types::AssetMetadata::Note(note) => (
                note.base.mime_type.clone(),
                None,
                None,
                Some(note.base.bytes),
                note.base.sha256.map(|hash| hex::encode(hash)),
            ),
        }
    }

    // Generate thumbnail link if available
    if let Some((asset_id, metadata)) =
        find_asset_by_type(memory, crate::memories::types::AssetType::Thumbnail)
    {
        let (content_type, width, height, bytes, etag) = extract_asset_metadata(metadata);
        header.assets.thumbnail = Some(build_asset_link(
            &memory.id,
            asset_id,
            AssetKind::Thumbnail,
            &content_type,
            width,
            height,
            bytes,
            etag,
            MEMORY_LISTING_TTL,
        ));
    }

    // Generate display link if available (Display type)
    if let Some((asset_id, metadata)) =
        find_asset_by_type(memory, crate::memories::types::AssetType::Display)
    {
        let (content_type, width, height, bytes, etag) = extract_asset_metadata(metadata);
        header.assets.display = Some(build_asset_link(
            &memory.id,
            asset_id,
            AssetKind::Display,
            &content_type,
            width,
            height,
            bytes,
            etag,
            MEMORY_LISTING_TTL,
        ));
    }

    // Generate original link if available
    if let Some((asset_id, metadata)) =
        find_asset_by_type(memory, crate::memories::types::AssetType::Original)
    {
        let (content_type, width, height, bytes, etag) = extract_asset_metadata(metadata);
        header.assets.original = Some(build_asset_link(
            &memory.id,
            asset_id,
            AssetKind::Original,
            &content_type,
            width,
            height,
            bytes,
            etag,
            MEMORY_LISTING_TTL,
        ));
    }

    // Extract placeholder data from inline assets if available
    // Look for a placeholder asset (small base64-encoded low-quality image for LQIP)
    for asset in &memory.inline_assets {
        let is_placeholder = match &asset.metadata {
            crate::memories::types::AssetMetadata::Image(img) => {
                // Check if this is a placeholder asset (very small size indicates placeholder)
                img.base.asset_type == crate::memories::types::AssetType::Derivative
                    && img.base.bytes < 1000 // Placeholders are typically very small
            }
            _ => false,
        };

        if is_placeholder {
            // Convert bytes to base64 for frontend LQIP
            header.placeholder_data = Some(general_purpose::STANDARD.encode(&asset.bytes));
            break; // Use the first placeholder found
        }
    }

    header
}
