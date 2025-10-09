//! Model helper functions for memory operations
//!
//! This module contains helper functions used across different memory operations,
//! providing common functionality for creating memories and managing assets.

use crate::types::{
    AssetMetadata, BlobRef, CapsuleId, Memory, MemoryAccess, MemoryAssetBlobExternal,
    MemoryAssetBlobInternal, MemoryAssetInline, MemoryMetadata, MemoryType, PersonRef,
    StorageEdgeBlobType,
};
use uuid::Uuid;

/// Generate a UUID v7 (time-ordered) for memory IDs
pub fn generate_uuid_v7() -> String {
    // For now, use a simple UUID generation until we can properly implement v7
    // TODO: Implement proper UUID v7 with timestamp context
    // Using a combination of timestamp and random data for uniqueness
    let timestamp = if cfg!(test) {
        // In test context, use a mock timestamp
        1234567890
    } else {
        // In canister context, use real time
        ic_cdk::api::time()
    };
    let random_data = format!("{}-{}", timestamp, timestamp % 1000);
    Uuid::new_v5(&Uuid::NAMESPACE_DNS, random_data.as_bytes()).to_string()
}

/// Generate a UUID for asset IDs using tech lead's recommended pattern
/// Uses v5 UUID (deterministic per unique seed) for ICP safety
pub fn generate_asset_id(caller: &PersonRef, timestamp: u64) -> String {
    // v5 UUID (deterministic per unique seed)
    let caller_str = match caller {
        PersonRef::Principal(p) => p.to_text(),
        PersonRef::Opaque(s) => s.clone(),
    };
    let seed = format!("{}-{}", caller_str, timestamp);
    Uuid::new_v5(&Uuid::NAMESPACE_OID, seed.as_bytes()).to_string()
}

/// Validate if a string is a valid UUID v7 format
/// Also accepts UUID v4 for early development compatibility
pub fn is_uuid_v7(id: &str) -> bool {
    match Uuid::parse_str(id) {
        Ok(uuid) => {
            // Check if it's UUID v7 (preferred) or v4 (early dev compatibility)
            uuid.get_version() == Some(uuid::Version::Random) || // v4
            uuid.get_version() == Some(uuid::Version::SortRand) // v7
        }
        Err(_) => false,
    }
}

/// Derive MemoryType from AssetMetadata variant
pub fn memory_type_from_asset(meta: &AssetMetadata) -> MemoryType {
    match meta {
        AssetMetadata::Note(_) => MemoryType::Note,
        AssetMetadata::Image(_) => MemoryType::Image,
        AssetMetadata::Document(_) => MemoryType::Document,
        AssetMetadata::Audio(_) => MemoryType::Audio,
        AssetMetadata::Video(_) => MemoryType::Video,
    }
}

/// Create an inline memory (small assets stored directly)
pub fn create_inline_memory(
    memory_id: &str,
    capsule_id: &CapsuleId,
    bytes: Vec<u8>,
    asset_metadata: AssetMetadata,
    now: u64,
    caller: &PersonRef,
) -> Memory {
    let inline_assets = vec![MemoryAssetInline {
        asset_id: generate_asset_id(caller, now),
        bytes: bytes.clone(),
        metadata: asset_metadata.clone(),
    }];

    let base = asset_metadata.get_base();
    let created_by = match caller {
        PersonRef::Principal(p) => Some(p.to_text()),
        PersonRef::Opaque(s) => Some(s.clone()),
    };

    Memory {
        id: memory_id.to_string(),
        capsule_id: capsule_id.clone(),
        metadata: MemoryMetadata {
            memory_type: memory_type_from_asset(&asset_metadata),
            title: Some(base.name.clone()),
            description: base.description.clone(),
            content_type: base.mime_type.clone(),
            created_at: now,
            updated_at: now,
            uploaded_at: now,
            date_of_memory: None,
            file_created_at: None,
            parent_folder_id: None,
            tags: base.tags.clone(),
            deleted_at: None,
            people_in_memory: None,
            location: None,
            memory_notes: None,
            created_by,
            database_storage_edges: vec![],

            // NEW: Pre-computed dashboard fields (defaults)
            is_public: false,
            shared_count: 0,
            sharing_status: "private".to_string(),
            total_size: base.bytes,
            asset_count: 1,
            thumbnail_url: None,
            primary_asset_url: None,
            has_thumbnails: false,
            has_previews: false,
        },
        access: MemoryAccess::Private {
            owner_secure_code: "test_code".to_string(), // TODO: Generate proper secure code
        },
        inline_assets,
        blob_internal_assets: vec![],
        blob_external_assets: vec![],
    }
}

/// Create a blob memory (large assets stored as blobs)
pub fn create_blob_memory(
    memory_id: &str,
    capsule_id: &CapsuleId,
    blob_ref: BlobRef,
    asset_metadata: AssetMetadata,
    now: u64,
    caller: &PersonRef,
) -> Memory {
    let blob_internal_assets = vec![MemoryAssetBlobInternal {
        asset_id: generate_asset_id(caller, now),
        blob_ref,
        metadata: asset_metadata.clone(),
    }];

    let base = asset_metadata.get_base();
    let created_by = match caller {
        PersonRef::Principal(p) => Some(p.to_text()),
        PersonRef::Opaque(s) => Some(s.clone()),
    };

    Memory {
        id: memory_id.to_string(),
        capsule_id: capsule_id.clone(),
        metadata: MemoryMetadata {
            memory_type: memory_type_from_asset(&asset_metadata),
            title: Some(base.name.clone()),
            description: base.description.clone(),
            content_type: base.mime_type.clone(),
            created_at: now,
            updated_at: now,
            uploaded_at: now,
            date_of_memory: None,
            file_created_at: None,
            parent_folder_id: None,
            tags: base.tags.clone(),
            deleted_at: None,
            people_in_memory: None,
            location: None,
            memory_notes: None,
            created_by,
            database_storage_edges: vec![],

            // NEW: Pre-computed dashboard fields (defaults)
            is_public: false,
            shared_count: 0,
            sharing_status: "private".to_string(),
            total_size: base.bytes,
            asset_count: 1,
            thumbnail_url: None,
            primary_asset_url: None,
            has_thumbnails: false,
            has_previews: false,
        },
        access: MemoryAccess::Private {
            owner_secure_code: "test_code".to_string(), // TODO: Generate proper secure code
        },
        inline_assets: vec![],
        blob_internal_assets,
        blob_external_assets: vec![],
    }
}

/// Create an external memory (assets stored outside ICP)
pub fn create_external_memory(
    memory_id: &str,
    capsule_id: &CapsuleId,
    location: StorageEdgeBlobType,
    storage_key: Option<String>,
    url: Option<String>,
    _size: Option<u64>,
    _hash: Option<Vec<u8>>,
    asset_metadata: AssetMetadata,
    now: u64,
    caller: &PersonRef,
) -> Memory {
    let blob_external_assets = vec![MemoryAssetBlobExternal {
        asset_id: generate_asset_id(caller, now),
        location,
        storage_key: storage_key.unwrap_or_default(),
        url,
        metadata: asset_metadata.clone(),
    }];

    let base = asset_metadata.get_base();
    let created_by = match caller {
        PersonRef::Principal(p) => Some(p.to_text()),
        PersonRef::Opaque(s) => Some(s.clone()),
    };

    Memory {
        id: memory_id.to_string(),
        capsule_id: capsule_id.clone(),
        metadata: MemoryMetadata {
            memory_type: memory_type_from_asset(&asset_metadata),
            title: Some(base.name.clone()),
            description: base.description.clone(),
            content_type: base.mime_type.clone(),
            created_at: now,
            updated_at: now,
            uploaded_at: now,
            date_of_memory: None,
            file_created_at: None,
            parent_folder_id: None,
            tags: base.tags.clone(),
            deleted_at: None,
            people_in_memory: None,
            location: None,
            memory_notes: None,
            created_by,
            database_storage_edges: vec![],

            // NEW: Pre-computed dashboard fields (defaults)
            is_public: false,
            shared_count: 0,
            sharing_status: "private".to_string(),
            total_size: base.bytes,
            asset_count: 1,
            thumbnail_url: None,
            primary_asset_url: None,
            has_thumbnails: false,
            has_previews: false,
        },
        access: MemoryAccess::Private {
            owner_secure_code: "test_code".to_string(), // TODO: Generate proper secure code
        },
        inline_assets: vec![],
        blob_internal_assets: vec![],
        blob_external_assets,
    }
}
