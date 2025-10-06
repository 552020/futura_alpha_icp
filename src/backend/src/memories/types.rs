use candid::{CandidType, Deserialize};
use ic_stable_structures::Storable;
use serde::Serialize;
use std::borrow::Cow;

// Import types from the main types module
use crate::types::{PersonRef, StorageEdgeBlobType, StorageEdgeDatabaseType};

// ============================================================================
// ASSET METADATA TYPES (moved from unified_types.rs)
// ============================================================================

/// Asset type for categorizing different asset variants
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq, Eq)]
pub enum AssetType {
    Original,
    Thumbnail,
    Preview,
    Derivative,
    Metadata,
}

/// Base asset metadata shared across all asset types
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct AssetMetadataBase {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub asset_type: AssetType,

    pub bytes: u64,
    pub mime_type: String,
    pub sha256: Option<[u8; 32]>, // 32 bytes
    pub width: Option<u32>,
    pub height: Option<u32>,

    pub url: Option<String>,
    pub storage_key: Option<String>,
    pub bucket: Option<String>,
    pub asset_location: Option<String>,

    pub processing_status: Option<String>,
    pub processing_error: Option<String>,

    pub created_at: u64,
    pub updated_at: u64,
    pub deleted_at: Option<u64>,
}

/// Image-specific asset metadata
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct ImageAssetMetadata {
    pub base: AssetMetadataBase,
    pub color_space: Option<String>,
    pub exif_data: Option<String>,
    pub compression_ratio: Option<f32>,
    pub dpi: Option<u32>,
    pub orientation: Option<u8>,
}

/// Video-specific asset metadata
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct VideoAssetMetadata {
    pub base: AssetMetadataBase,
    pub duration: Option<u64>, // ms
    pub frame_rate: Option<f32>,
    pub codec: Option<String>,
    pub bitrate: Option<u64>,
    pub resolution: Option<String>, // "1920x1080"
    pub aspect_ratio: Option<f32>,
}

/// Audio-specific asset metadata
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct AudioAssetMetadata {
    pub base: AssetMetadataBase,
    pub duration: Option<u64>, // ms
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
    pub bitrate: Option<u64>,
    pub codec: Option<String>,
    pub bit_depth: Option<u8>,
}

/// Document-specific asset metadata
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct DocumentAssetMetadata {
    pub base: AssetMetadataBase,
    pub page_count: Option<u32>,
    pub document_type: Option<String>,
    pub language: Option<String>,
    pub word_count: Option<u32>,
}

/// Note-specific asset metadata
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct NoteAssetMetadata {
    pub base: AssetMetadataBase,
    pub word_count: Option<u32>,
    pub language: Option<String>,
    pub format: Option<String>,
}

/// Unified asset metadata for all asset types
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum AssetMetadata {
    Image(ImageAssetMetadata),
    Video(VideoAssetMetadata),
    Audio(AudioAssetMetadata),
    Document(DocumentAssetMetadata),
    Note(NoteAssetMetadata),
}

// ============================================================================
// MEMORY-RELATED TYPE ALIASES
// ============================================================================

/// Type alias for memory identifiers
pub type MemoryId = String;

// ============================================================================
// MEMORY TYPES AND STRUCTURES
// ============================================================================

/// Memory type classification
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum MemoryType {
    Image,
    Video,
    Audio,
    Document,
    Note,
}

/// Memory access control
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum MemoryAccess {
    Public {
        owner_secure_code: String, // secure code for owner access control
    },
    Private {
        owner_secure_code: String, // secure code for owner access control
    },
    Custom {
        individuals: Vec<PersonRef>, // direct individual access
        groups: Vec<String>,         // group access (group IDs)
        owner_secure_code: String,   // secure code for owner access control
    },

    // Time-based access
    Scheduled {
        accessible_after: u64,     // nanoseconds since Unix epoch
        access: Box<MemoryAccess>, // what it becomes after the time
        owner_secure_code: String, // secure code for owner access control
    },

    // Event-based access
    EventTriggered {
        trigger_event: AccessEvent,
        access: Box<MemoryAccess>, // what it becomes after the event
        owner_secure_code: String, // secure code for owner access control
    },
}

/// Events that can trigger access changes
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum AccessEvent {
    // Memorial events
    AfterDeath,       // revealed after subject's death is recorded
    Anniversary(u32), // revealed on specific anniversary (Nth year)

    // Life events
    Birthday(u32), // revealed on Nth birthday
    Graduation,    // revealed after graduation
    Wedding,       // revealed after wedding

    // Capsule events
    CapsuleMaturity(u32), // revealed when capsule reaches N years old
    ConnectionCount(u32), // revealed when capsule has N connections

    // Custom events
    Custom(String), // custom event identifier
}

/// Enhanced MemoryMetadata (Memory-Level Metadata)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct MemoryMetadata {
    // Basic info
    pub memory_type: MemoryType,
    pub title: Option<String>,       // Optional title (matches database)
    pub description: Option<String>, // Optional description (matches database)
    pub content_type: String,

    // Timestamps
    pub created_at: u64,
    pub updated_at: u64,
    pub uploaded_at: u64,
    pub date_of_memory: Option<u64>, // when the actual event happened
    pub file_created_at: Option<u64>, // when the original file was created

    // Organization
    pub parent_folder_id: Option<String>,
    pub tags: Vec<String>, // Memory tags
    pub deleted_at: Option<u64>,

    // Content info
    pub people_in_memory: Option<Vec<String>>, // People in the memory
    pub location: Option<String>,              // Where the memory was taken
    pub memory_notes: Option<String>,          // Additional notes

    // System info
    pub created_by: Option<String>, // Who created this memory
    pub database_storage_edges: Vec<StorageEdgeDatabaseType>,
}

/// Blob reference for external storage
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct BlobRef {
    pub locator: String,        // canister+key, URL, CID, etc.
    pub hash: Option<[u8; 32]>, // optional integrity hash
    pub len: u64,               // size in bytes
}

/// Blob metadata
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct BlobMeta {
    pub size: u64,        // total size in bytes
    pub chunk_count: u32, // number of chunks
}

/// Inline asset (stored directly in memory)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct MemoryAssetInline {
    pub asset_id: String, // Unique identifier for this asset
    pub bytes: Vec<u8>,
    pub metadata: AssetMetadata,
}

/// Blob asset (reference to ICP blob store)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct MemoryAssetBlobInternal {
    pub asset_id: String, // Unique identifier for this asset
    pub blob_ref: BlobRef,
    pub metadata: AssetMetadata,
}

/// External blob asset (reference to external storage)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct MemoryAssetBlobExternal {
    pub asset_id: String,              // Unique identifier for this asset
    pub location: StorageEdgeBlobType, // Where the asset is stored externally
    pub storage_key: String,           // Key/ID in external storage system
    pub url: Option<String>,           // Public URL (if available)
    pub metadata: AssetMetadata,       // Type-specific metadata
}

/// Legacy struct for backward compatibility (will be removed)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct MemoryAssetBlob {
    pub blob: BlobRef,
    pub metadata: AssetMetadata,
}

/// Main memory structure
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Memory {
    pub id: String,                                         // unique identifier
    pub metadata: MemoryMetadata, // memory-level metadata (title, description, etc.)
    pub access: MemoryAccess,     // who can access + temporal rules
    pub inline_assets: Vec<MemoryAssetInline>, // 0 or more inline assets
    pub blob_internal_assets: Vec<MemoryAssetBlobInternal>, // 0 or more ICP blob assets
    pub blob_external_assets: Vec<MemoryAssetBlobExternal>, // 0 or more external blob assets
}

/// Memory header for listings
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct MemoryHeader {
    pub id: String,
    pub name: String,
    pub memory_type: MemoryType,
    pub size: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub access: MemoryAccess,
}

/// Memory operation response
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct MemoryOperationResponse {
    pub success: bool,
    pub memory_id: Option<String>,
    pub message: String,
}

/// Memory update data
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct MemoryUpdateData {
    pub name: Option<String>,
    pub metadata: Option<MemoryMetadata>,
    pub access: Option<MemoryAccess>,
}

/// Memory list response
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct MemoryListResponse {
    pub success: bool,
    pub memories: Vec<MemoryHeader>,
    pub message: String,
}

/// Memory presence check result
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemoryPresenceResult {
    pub memory_id: String,
    pub metadata_present: bool,
    pub asset_present: bool,
}

/// Simple memory metadata structure for ICP storage
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct SimpleMemoryMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub size: Option<u64>,
    pub content_type: Option<String>,
    pub custom_fields: std::collections::HashMap<String, String>,
}

// ============================================================================
// STORABLE IMPLEMENTATIONS FOR MEMORY TYPES
// ============================================================================

impl Storable for Memory {
    fn to_bytes(&self) -> Cow<[u8]> {
        let bytes = candid::encode_one(self).expect("Failed to encode Memory");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).expect("Failed to decode Memory")
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 2_097_152, // 2MB for memory with assets
            is_fixed_size: false,
        };
}

impl Storable for MemoryMetadata {
    fn to_bytes(&self) -> Cow<[u8]> {
        let bytes = candid::encode_one(self).expect("Failed to encode MemoryMetadata");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).expect("Failed to decode MemoryMetadata")
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 8192, // 8KB for metadata
            is_fixed_size: false,
        };
}

impl AssetMetadata {
    /// Get the base metadata that's common to all asset types
    pub fn get_base(&self) -> &AssetMetadataBase {
        match self {
            AssetMetadata::Image(img) => &img.base,
            AssetMetadata::Video(vid) => &vid.base,
            AssetMetadata::Audio(audio) => &audio.base,
            AssetMetadata::Document(doc) => &doc.base,
            AssetMetadata::Note(note) => &note.base,
        }
    }
}

// ============================================================================
// BULK OPERATION RESULT TYPES
// ============================================================================

/// Result type for bulk memory deletion operations
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct BulkDeleteResult {
    pub deleted_count: u32,
    pub failed_count: u32,
    pub message: String,
}

/// Result type for asset cleanup operations
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct AssetCleanupResult {
    pub memory_id: String,
    pub assets_cleaned: u32,
    pub message: String,
}

// ============================================================================
// MEMORY CREATION INPUT TYPES
// ============================================================================

/// Input type for creating memories with inline assets (small files ≤32KB)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct InlineAssetInput {
    pub bytes: Vec<u8>,
    pub metadata: AssetMetadata,
}

/// Input type for creating memories with internal blob assets (ICP blob storage)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct InternalBlobAssetInput {
    pub blob_id: String,        // From uploads_finish (ICP blob storage)
    pub metadata: AssetMetadata,
}

/// Input type for creating memories with external blob assets (S3, Vercel, Arweave, IPFS, etc.)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct ExternalBlobAssetInput {
    pub location: StorageEdgeBlobType, // S3, Vercel, Arweave, IPFS, etc.
    pub storage_key: String,
    pub url: Option<String>,
    pub size: Option<u64>,
    pub hash: Option<Vec<u8>>,
    pub metadata: AssetMetadata,
}

/// Result type for bulk asset cleanup operations
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct BulkAssetCleanupResult {
    pub cleaned_count: u32,
    pub failed_count: u32,
    pub total_assets_cleaned: u32,
    pub message: String,
}

/// Result type for individual asset removal operations
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct AssetRemovalResult {
    pub memory_id: String,
    pub asset_removed: bool,
    pub message: String,
}

/// Result type for listing memory assets
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct MemoryAssetsList {
    pub memory_id: String,
    pub inline_assets: Vec<String>, // Asset references for inline assets
    pub internal_assets: Vec<String>, // Blob references for ICP assets
    pub external_assets: Vec<String>, // Storage keys for external assets
    pub total_count: u32,
}

/// Gallery memory entry (for gallery-specific memory references)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct GalleryMemoryEntry {
    pub memory_id: String,               // Reference to existing memory
    pub position: u32,                   // Gallery-specific ordering
    pub gallery_caption: Option<String>, // Only if different from memory caption
    pub is_featured: bool,               // Gallery-specific highlighting
    pub gallery_metadata: String,        // JSON for gallery-specific annotations
}
