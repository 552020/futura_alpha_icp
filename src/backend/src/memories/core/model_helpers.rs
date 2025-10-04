//! Model helper functions for memory operations
//!
//! This module contains helper functions used across different memory operations,
//! providing common functionality for creating memories and managing assets.

use crate::types::{
    AssetMetadata, BlobRef, CapsuleId, Memory, MemoryAccess, MemoryAssetBlobExternal,
    MemoryAssetBlobInternal, MemoryAssetInline, MemoryMetadata, MemoryType, PersonRef,
    StorageEdgeBlobType,
};

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
    _capsule_id: &CapsuleId,
    bytes: Vec<u8>,
    asset_metadata: AssetMetadata,
    now: u64,
    caller: &PersonRef,
) -> Memory {
    let inline_assets = vec![MemoryAssetInline {
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
    _capsule_id: &CapsuleId,
    blob_ref: BlobRef,
    asset_metadata: AssetMetadata,
    now: u64,
    caller: &PersonRef,
) -> Memory {
    let blob_internal_assets = vec![MemoryAssetBlobInternal {
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
    _capsule_id: &CapsuleId,
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
        },
        access: MemoryAccess::Private {
            owner_secure_code: "test_code".to_string(), // TODO: Generate proper secure code
        },
        inline_assets: vec![],
        blob_internal_assets: vec![],
        blob_external_assets,
    }
}
