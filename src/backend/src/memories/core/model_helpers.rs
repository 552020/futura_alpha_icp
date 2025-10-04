//! Shared model helpers and utilities
//!
//! This module contains shared helper functions used across multiple
//! memory operations.

use crate::types::{AssetMetadata, MemoryType, StorageEdgeBlobType};

/// Determine memory type from asset metadata
pub(crate) fn memory_type_from_asset(metadata: &AssetMetadata) -> MemoryType {
    match metadata {
        AssetMetadata::Image(_) => MemoryType::Image,
        AssetMetadata::Video(_) => MemoryType::Video,
        AssetMetadata::Audio(_) => MemoryType::Audio,
        AssetMetadata::Document(_) => MemoryType::Document,
        AssetMetadata::Note(_) => MemoryType::Note,
    }
}

/// Validate storage edge blob type
pub(crate) fn is_valid_storage_type(storage_type: &StorageEdgeBlobType) -> bool {
    match storage_type {
        StorageEdgeBlobType::S3 => true,
        StorageEdgeBlobType::Icp => true,
        StorageEdgeBlobType::VercelBlob => true,
        StorageEdgeBlobType::Ipfs => true,
        StorageEdgeBlobType::Neon => true,
        StorageEdgeBlobType::Arweave => true,
    }
}

/// Create a default memory metadata structure
pub(crate) fn create_default_metadata(
    memory_type: MemoryType,
    title: Option<String>,
    description: Option<String>,
    now: u64,
) -> crate::types::MemoryMetadata {
    crate::types::MemoryMetadata {
        memory_type,
        title,
        description,
        content_type: "application/octet-stream".to_string(),
        created_at: now,
        updated_at: now,
        uploaded_at: now,
        date_of_memory: None,
        file_created_at: None,
        parent_folder_id: None,
        tags: vec![],
        deleted_at: None,
        people_in_memory: None,
        location: None,
        memory_notes: None,
        created_by: None,
        database_storage_edges: vec![],
    }
}
