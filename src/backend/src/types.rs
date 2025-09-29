use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::Storable;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashMap;

// ============================================================================
// TYPE ALIASES
// ============================================================================

/// Type alias for capsule identifiers (used throughout the codebase)
pub type CapsuleId = String;

/// Type alias for memory identifiers
pub type MemoryId = String;

// ============================================================================
// STORAGE EDGE TYPES
// ============================================================================

/// Database storage edge types - where memory metadata/records are stored
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum StorageEdgeDatabaseType {
    Icp,  // ICP canister storage
    Neon, // Neon database
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum StorageEdgeBlobType {
    Icp,        // ICP canister storage
    VercelBlob, // Vercel Blob storage
    S3,         // AWS S3 storage
    Arweave,    // Arweave storage
    Ipfs,       // IPFS storage
    Neon,       // Neon database - for small assets
}

/// Type alias for unified error handling - see Error enum below
/// Enhanced asset metadata with all fields from database memory_assets table
// Base asset metadata (shared across all asset types)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct AssetMetadataBase {
    // Basic info
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,

    // Asset classification
    pub asset_type: AssetType, // Moved from asset struct to metadata

    // File properties
    pub bytes: u64,                  // File size
    pub mime_type: String,           // MIME type
    pub sha256: Option<[u8; 32]>,    // File hash
    pub width: Option<u32>,          // Image/video width
    pub height: Option<u32>,         // Image/video height
    pub url: Option<String>,         // External URL
    pub storage_key: Option<String>, // Storage identifier
    pub bucket: Option<String>,      // Storage bucket

    // Processing status
    pub processing_status: Option<String>,
    pub processing_error: Option<String>,

    // Timestamps
    pub created_at: u64,
    pub updated_at: u64,
    pub deleted_at: Option<u64>,

    // Storage location
    pub asset_location: Option<String>, // Where the asset is stored
}

// Type-specific asset metadata extensions
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct ImageAssetMetadata {
    pub base: AssetMetadataBase,
    pub color_space: Option<String>,
    pub exif_data: Option<String>,
    pub compression_ratio: Option<f32>,
    pub dpi: Option<u32>,
    pub orientation: Option<u8>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct VideoAssetMetadata {
    pub base: AssetMetadataBase,
    pub duration: Option<u64>,      // Duration in milliseconds
    pub frame_rate: Option<f32>,    // Frames per second
    pub codec: Option<String>,      // Video codec (H.264, VP9, etc.)
    pub bitrate: Option<u64>,       // Bitrate in bits per second
    pub resolution: Option<String>, // Resolution string (e.g., "1920x1080")
    pub aspect_ratio: Option<f32>,  // Aspect ratio
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct AudioAssetMetadata {
    pub base: AssetMetadataBase,
    pub duration: Option<u64>,    // Duration in milliseconds
    pub sample_rate: Option<u32>, // Sample rate in Hz
    pub channels: Option<u8>,     // Number of audio channels
    pub bitrate: Option<u64>,     // Bitrate in bits per second
    pub codec: Option<String>,    // Audio codec (MP3, AAC, etc.)
    pub bit_depth: Option<u8>,    // Bit depth (16, 24, 32)
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct DocumentAssetMetadata {
    pub base: AssetMetadataBase,
    pub page_count: Option<u32>,       // Number of pages
    pub document_type: Option<String>, // PDF, DOCX, etc.
    pub language: Option<String>,      // Document language
    pub word_count: Option<u32>,       // Word count
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct NoteAssetMetadata {
    pub base: AssetMetadataBase,
    pub word_count: Option<u32>,  // Word count
    pub language: Option<String>, // Note language
    pub format: Option<String>,   // Markdown, plain text, etc.
}

// Unified asset metadata enum
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum AssetMetadata {
    Image(ImageAssetMetadata),
    Video(VideoAssetMetadata),
    Audio(AudioAssetMetadata),
    Document(DocumentAssetMetadata),
    Note(NoteAssetMetadata),
}

/// Upload configuration for TS client discoverability
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct UploadConfig {
    /// Maximum size for inline uploads (bytes)
    pub inline_max: u32,
    /// Recommended chunk size for chunked uploads (bytes)
    pub chunk_size: u32,
    /// Maximum inline budget per capsule (bytes)
    pub inline_budget_per_capsule: u32,
}

// ============================================================================
// MVP ICP ERROR MODEL - Essential error types for ICP operations
// ============================================================================

// Core error types for all ICP operations
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum Error {
    // Core reusable variants
    Unauthorized,
    NotFound,
    InvalidArgument(String), // validation/config/parse errors
    Conflict(String),        // already exists, concurrent update
    ResourceExhausted,       // quotas/size/cycles
    Internal(String),        // redact in prod logs
}

// Canonical Rust Result type
pub type Result<T> = std::result::Result<T, Error>;

// Result<T> removed - using canonical Result<T> = std::result::Result<T, Error>

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Unauthorized => write!(f, "unauthorized access"),
            Error::NotFound => write!(f, "resource not found"),
            Error::InvalidArgument(msg) => write!(f, "invalid argument: {}", msg.to_lowercase()),
            Error::Conflict(msg) => write!(f, "conflict: {}", msg.to_lowercase()),
            Error::ResourceExhausted => write!(f, "resource exhausted"),
            Error::Internal(msg) => write!(f, "internal error: {}", msg.to_lowercase()),
        }
    }
}

impl Error {
    pub fn code(&self) -> u16 {
        match self {
            Error::Unauthorized => 401,
            Error::NotFound => 404,
            Error::InvalidArgument(_) => 422,
            Error::Conflict(_) => 409,
            Error::ResourceExhausted => 429,
            Error::Internal(_) => 500,
        }
    }
}

// Helper functions to convert between error patterns
impl CommitResponse {
    pub fn ok(memory_id: String, final_hash: String, total_bytes: u64, message: String) -> Self {
        Self {
            success: true,
            memory_id,
            final_hash,
            total_bytes,
            message,
            error: None,
        }
    }

    pub fn err(error: Error, memory_id: String, message: String) -> Self {
        Self {
            success: false,
            memory_id,
            final_hash: String::new(),
            total_bytes: 0,
            message,
            error: Some(error),
        }
    }
}

// HTTP types for serving content
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpHeader(pub String, pub String);

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HttpHeader>,
    pub body: Vec<u8>,
}

// Memory artifacts system removed - metadata stored in capsules instead

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct UploadSession {
    pub session_id: String,
    pub memory_id: String,
    pub memory_type: MemoryType, // Added to support different memory types
    pub expected_hash: String,
    pub chunk_count: u32,
    pub total_size: u64,
    pub created_at: u64,
    pub chunks_received: Vec<bool>, // Track which chunks have been received
    pub bytes_received: u64,        // Total bytes received so far
}

// Structure for storing individual chunk data
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ChunkData {
    pub session_id: String,
    pub chunk_index: u32,
    pub data: Vec<u8>,
    pub received_at: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemoryPresenceResult {
    pub memory_id: String,
    pub metadata_present: bool,
    pub asset_present: bool,
}

// Simple memory metadata structure for ICP storage (task 1.3)
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

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct CommitResponse {
    pub success: bool,
    pub memory_id: String,
    pub final_hash: String,
    pub total_bytes: u64,
    pub message: String,
    pub error: Option<Error>,
}

// Memory sync request for batch operations
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemorySyncRequest {
    pub memory_id: String,
    pub memory_type: MemoryType,
    pub metadata: SimpleMemoryMetadata,
    pub asset_url: String, // URL to fetch asset from (e.g., Vercel Blob)
    pub expected_asset_hash: String, // Expected hash of the asset
    pub asset_size: u64,   // Size of the asset in bytes
}

// Response for individual memory sync operation
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemorySyncResult {
    pub memory_id: String,
    pub success: bool,
    pub metadata_stored: bool,
    pub asset_stored: bool,
    pub message: String,
    pub error: Option<Error>,
}

// Response for batch memory sync operation
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct BatchMemorySyncResponse {
    pub gallery_id: String,
    pub success: bool,
    pub total_memories: u32,
    pub successful_memories: u32,
    pub failed_memories: u32,
    pub results: Vec<MemorySyncResult>,
    pub message: String,
    pub error: Option<Error>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct UserMemoriesResponse {
    pub images: Vec<String>,
    pub notes: Vec<String>,
    pub videos: Vec<String>,
    pub documents: Vec<String>,
    pub audio: Vec<String>,
}

// User management types for Internet Identity integration
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct User {
    pub principal: Principal,
    /// Registration timestamp (nanoseconds since Unix epoch)
    pub registered_at: u64,
    /// Last activity timestamp (nanoseconds since Unix epoch)  
    pub last_activity_at: u64,
    /// Whether user is bound to Web2 session (optional convenience flag)
    pub bound: bool,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct UserRegistrationResult {
    pub success: bool,
    pub user: Option<User>,
    pub message: String,
}

// Capsule types for user-owned data architecture
// Core person reference - can be a live principal or opaque identifier
#[derive(CandidType, Deserialize, Serialize, Clone, Eq, PartialEq, Hash, Debug)]
pub enum PersonRef {
    Principal(Principal), // live II user
    Opaque(String),       // non-principal subject (e.g., deceased), UUID-like
}

impl PersonRef {
    /// Extract the Principal if this is a Principal variant, None otherwise
    pub fn principal(&self) -> Option<&Principal> {
        match self {
            PersonRef::Principal(p) => Some(p),
            PersonRef::Opaque(_) => None,
        }
    }
}

// Connection status for peer relationships
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum ConnectionStatus {
    Pending,
    Accepted,
    Blocked,
    Revoked,
}

// Connection between persons
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Connection {
    pub peer: PersonRef,
    pub status: ConnectionStatus,
    pub created_at: u64,
    pub updated_at: u64,
}

// Connection groups for organizing relationships
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct ConnectionGroup {
    pub id: String,
    pub name: String, // "Family", "Close Friends", etc.
    pub description: Option<String>,
    pub members: Vec<PersonRef>,
    pub created_at: u64,
    pub updated_at: u64,
}

// Controller state tracking (simplified - full control except ownership transfer)
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ControllerState {
    pub granted_at: u64,
    pub granted_by: PersonRef,
}

// Owner state tracking
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct OwnerState {
    pub since: u64,
    pub last_activity_at: u64, // Track owner activity
}

// Main capsule structure
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Capsule {
    pub id: String,                                          // unique capsule identifier
    pub subject: PersonRef,                                  // who this capsule is about
    pub owners: HashMap<PersonRef, OwnerState>,              // 1..n owners (usually 1)
    pub controllers: HashMap<PersonRef, ControllerState>,    // delegated admins (full control)
    pub connections: HashMap<PersonRef, Connection>,         // social graph
    pub connection_groups: HashMap<String, ConnectionGroup>, // organized connection groups
    pub memories: HashMap<String, Memory>,                   // content
    pub galleries: HashMap<String, Gallery>,                 // galleries (collections of memories)
    pub created_at: u64,
    pub updated_at: u64,
    pub bound_to_neon: bool,    // Neon database binding status
    pub inline_bytes_used: u64, // Track inline storage consumption
}

// Capsule registration result (minimal response for user registration)
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct CapsuleRegistrationResult {
    pub success: bool,
    pub capsule_id: Option<String>, // Just the ID
    pub is_new: bool,               // true if just created, false if existing
    pub message: String,
}

// Capsule information for user queries
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct CapsuleInfo {
    pub capsule_id: String,
    pub subject: PersonRef,
    pub is_owner: bool,
    pub is_controller: bool,
    pub is_self_capsule: bool, // true if subject == caller
    pub bound_to_neon: bool,
    pub created_at: u64,
    pub updated_at: u64,

    // Lightweight counts for summary information
    pub memory_count: u32,     // Number of memories in this capsule
    pub gallery_count: u32,    // Number of galleries in this capsule
    pub connection_count: u32, // Number of connections to other people
}

// Capsule header for listing
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct CapsuleHeader {
    pub id: String,
    pub subject: PersonRef,
    pub owner_count: u32,
    pub controller_count: u32,
    pub memory_count: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

// Capsule update data for partial updates
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct CapsuleUpdateData {
    pub bound_to_neon: Option<bool>, // Update binding status
                                     // Note: Most capsule fields (id, subject, owners, etc.) are immutable
                                     // Only binding status and timestamps can be updated
}

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

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct GalleryHeader {
    pub id: String,
    pub name: String,
    pub memory_count: u32,
    pub created_at: u64,
    pub updated_at: u64,
    pub storage_location: GalleryStorageLocation,
}

// New unified memory system
// Old metadata structures removed - replaced with new AssetMetadata enum and AssetMetadataBase

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum MemoryType {
    Image,
    Video,
    Audio,
    Document,
    Note,
}

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

// Events that can trigger access changes
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

// New memory structures
// Enhanced MemoryMetadata (Memory-Level Metadata)
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

// External blob storage types
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum MemoryBlobKindExternal {
    Http,    // HTTP URL
    Ipfs,    // IPFS CID
    Arweave, // Arweave CID
    AWS,     // AWS S3
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct BlobRef {
    pub locator: String,        // canister+key, URL, CID, etc.
    pub hash: Option<[u8; 32]>, // optional integrity hash
    pub len: u64,               // size in bytes
}

// #[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
// pub struct MemoryData {
//     pub blob_ref: BlobRef,     // where the data is stored
//     pub data: Option<Vec<u8>>, // inline data (for IcCanister storage)
// }

// Asset type for categorizing different asset variants
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum AssetType {
    Original,   // Original file
    Thumbnail,  // Small preview/thumbnail
    Preview,    // Medium preview
    Derivative, // Processed/derived version
    Metadata,   // Metadata-only asset
}

// Inline asset (stored directly in memory)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct MemoryAssetInline {
    pub bytes: Vec<u8>,
    pub metadata: AssetMetadata,
}

// Blob asset (reference to ICP blob store)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct MemoryAssetBlobInternal {
    pub blob_ref: BlobRef,
    pub metadata: AssetMetadata,
}

// External blob asset (reference to external storage)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct MemoryAssetBlobExternal {
    pub location: StorageEdgeBlobType, // Where the asset is stored externally
    pub storage_key: String,           // Key/ID in external storage system
    pub url: Option<String>,           // Public URL (if available)
    pub metadata: AssetMetadata,       // Type-specific metadata
}

// Legacy struct for backward compatibility (will be removed)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct MemoryAssetBlob {
    pub blob: BlobRef,
    pub metadata: AssetMetadata,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Memory {
    pub id: String,                                         // unique identifier
    pub metadata: MemoryMetadata, // memory-level metadata (title, description, etc.)
    pub access: MemoryAccess,     // who can access + temporal rules
    pub inline_assets: Vec<MemoryAssetInline>, // 0 or more inline assets
    pub blob_internal_assets: Vec<MemoryAssetBlobInternal>, // 0 or more ICP blob assets
    pub blob_external_assets: Vec<MemoryAssetBlobExternal>, // 0 or more external blob assets
}

impl Memory {
    /// Create a new memory with inline data (≤32KB)
    pub fn inline(bytes: Vec<u8>, asset_metadata: AssetMetadata) -> Self {
        let now = ic_cdk::api::time();
        Self {
            id: format!("mem_{now}"), // Simple ID generation
            metadata: MemoryMetadata {
                memory_type: MemoryType::Note, // Default type for inline
                title: None,
                description: None,
                content_type: match &asset_metadata {
                    AssetMetadata::Image(img) => img.base.mime_type.clone(),
                    AssetMetadata::Video(vid) => vid.base.mime_type.clone(),
                    AssetMetadata::Audio(audio) => audio.base.mime_type.clone(),
                    AssetMetadata::Document(doc) => doc.base.mime_type.clone(),
                    AssetMetadata::Note(note) => note.base.mime_type.clone(),
                },
                created_at: now,
                updated_at: now,
                uploaded_at: now,
                date_of_memory: None,
                file_created_at: None,
                parent_folder_id: None, // Default to root folder
                tags: match &asset_metadata {
                    AssetMetadata::Image(img) => img.base.tags.clone(),
                    AssetMetadata::Video(vid) => vid.base.tags.clone(),
                    AssetMetadata::Audio(audio) => audio.base.tags.clone(),
                    AssetMetadata::Document(doc) => doc.base.tags.clone(),
                    AssetMetadata::Note(note) => note.base.tags.clone(),
                },
                deleted_at: None,
                people_in_memory: None,
                location: None,
                memory_notes: None,
                created_by: None,
                database_storage_edges: vec![StorageEdgeDatabaseType::Icp],
            },
            access: MemoryAccess::Private {
                owner_secure_code: format!("mem_{now}_{:x}", now % 0xFFFF), // Generate secure code
            }, // Default to private access
            inline_assets: vec![MemoryAssetInline {
                bytes,
                metadata: asset_metadata,
            }],
            blob_internal_assets: vec![],
            blob_external_assets: vec![],
        }
    }

    /// Create a new memory with blob reference (>32KB)
    pub fn from_blob(
        blob_id: u64,
        size: u64,
        checksum: [u8; 32],
        asset_metadata: AssetMetadata,
    ) -> Self {
        let now = ic_cdk::api::time();
        Self {
            id: format!("mem_{now}"), // Simple ID generation
            metadata: MemoryMetadata {
                memory_type: MemoryType::Note, // Default type for blob
                title: None,
                description: None,
                content_type: match &asset_metadata {
                    AssetMetadata::Image(img) => img.base.mime_type.clone(),
                    AssetMetadata::Video(vid) => vid.base.mime_type.clone(),
                    AssetMetadata::Audio(audio) => audio.base.mime_type.clone(),
                    AssetMetadata::Document(doc) => doc.base.mime_type.clone(),
                    AssetMetadata::Note(note) => note.base.mime_type.clone(),
                },
                created_at: now,
                updated_at: now,
                uploaded_at: now,
                date_of_memory: None,
                file_created_at: None,
                parent_folder_id: None, // Default to root folder
                tags: match &asset_metadata {
                    AssetMetadata::Image(img) => img.base.tags.clone(),
                    AssetMetadata::Video(vid) => vid.base.tags.clone(),
                    AssetMetadata::Audio(audio) => audio.base.tags.clone(),
                    AssetMetadata::Document(doc) => doc.base.tags.clone(),
                    AssetMetadata::Note(note) => note.base.tags.clone(),
                },
                deleted_at: None,
                people_in_memory: None,
                location: None,
                memory_notes: None,
                created_by: None,
                database_storage_edges: vec![StorageEdgeDatabaseType::Icp],
            },
            access: MemoryAccess::Private {
                owner_secure_code: format!("blob_{blob_id}_{:x}", now % 0xFFFF), // Generate secure code
            }, // Default to private access
            inline_assets: vec![],
            blob_internal_assets: vec![MemoryAssetBlobInternal {
                blob_ref: BlobRef {
                    locator: format!("blob_{blob_id}"),
                    hash: Some(checksum),
                    len: size,
                },
                metadata: asset_metadata,
            }],
            blob_external_assets: vec![],
        }
    }

    /// Get memory header for listing
    pub fn to_header(&self) -> MemoryHeader {
        // Calculate total size from all assets
        let inline_size: u64 = self
            .inline_assets
            .iter()
            .map(|asset| asset.bytes.len() as u64)
            .sum();
        let blob_internal_size: u64 = self
            .blob_internal_assets
            .iter()
            .map(|asset| asset.blob_ref.len)
            .sum();
        let blob_external_size: u64 = self
            .blob_external_assets
            .iter()
            .map(|asset| match &asset.metadata {
                AssetMetadata::Image(img) => img.base.bytes,
                AssetMetadata::Video(vid) => vid.base.bytes,
                AssetMetadata::Audio(audio) => audio.base.bytes,
                AssetMetadata::Document(doc) => doc.base.bytes,
                AssetMetadata::Note(note) => note.base.bytes,
            })
            .sum();
        let size = inline_size + blob_internal_size + blob_external_size;

        MemoryHeader {
            id: self.id.clone(),
            name: self
                .metadata
                .title
                .clone()
                .unwrap_or_else(|| "Untitled".to_string()),
            memory_type: self.metadata.memory_type.clone(),
            size,
            created_at: self.metadata.created_at,
            updated_at: self.metadata.updated_at,
            access: self.access.clone(),
        }
    }
}

// ============================================================================
// GALLERY SYSTEM - Minimal Abstraction Approach
// ============================================================================

// Gallery storage status for tracking where gallery data is stored
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum GalleryStorageLocation {
    Web2Only,  // Stored only in Web2 database
    ICPOnly,   // Stored only in ICP canister
    Both,      // Stored in both Web2 and ICP
    Migrating, // Currently being migrated from Web2 to ICP
    Failed,    // Migration or storage failed
}

// Minimal extra data for gallery memory entries
// Only stores what memories don't already have
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct GalleryMemoryEntry {
    pub memory_id: String,               // Reference to existing memory
    pub position: u32,                   // Gallery-specific ordering
    pub gallery_caption: Option<String>, // Only if different from memory caption
    pub is_featured: bool,               // Gallery-specific highlighting
    pub gallery_metadata: String,        // JSON for gallery-specific annotations
}

// Main gallery structure with embedded memory entries
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Gallery {
    pub id: String,                               // unique gallery identifier
    pub owner_principal: Principal,               // who owns this gallery
    pub title: String,                            // gallery title
    pub description: Option<String>,              // gallery description
    pub is_public: bool,                          // whether gallery is publicly accessible
    pub created_at: u64,                          // creation timestamp (nanoseconds)
    pub updated_at: u64,                          // last update timestamp (nanoseconds)
    pub storage_location: GalleryStorageLocation, // where this gallery is stored
    pub memory_entries: Vec<GalleryMemoryEntry>,  // minimal extra data for each memory
    pub bound_to_neon: bool,                      // whether linked to Neon database
}

// Gallery creation result
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct GalleryCreationResult {
    pub success: bool,
    pub gallery_id: Option<String>,
    pub message: String,
}

// Gallery data for storage operations
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct GalleryData {
    pub gallery: Gallery,
    pub owner_principal: Principal,
}

// Gallery update data
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct GalleryUpdateData {
    pub title: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
    pub memory_entries: Option<Vec<GalleryMemoryEntry>>,
}

// User principal management types
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct UserPrincipalData {
    pub principal: Principal,
    pub web2_user_id: Option<String>, // Link to Web2 user ID
    pub registered_at: u64,           // Registration timestamp
    pub last_activity_at: u64,        // Last activity timestamp
    pub bound_to_neon: bool,          // Whether linked to Neon database
}

// User principal registration result
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct RegisterUserResponse {
    pub success: bool,
    pub user_data: Option<UserPrincipalData>,
    pub message: String,
}

// User principal linking result
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct LinkUserResponse {
    pub success: bool,
    pub user_data: Option<UserPrincipalData>,
    pub message: String,
}

// Gallery verification result
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct VerificationResult {
    pub success: bool,
    pub verified_memories: u32,        // Number of memories verified
    pub total_memories: u32,           // Total memories in gallery
    pub missing_memories: Vec<String>, // Memory IDs that couldn't be verified
    pub message: String,
}

// Memory operation response types
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct MemoryOperationResponse {
    pub success: bool,
    pub memory_id: Option<String>,
    pub message: String,
}

// Memory update data
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct MemoryUpdateData {
    pub name: Option<String>,
    pub metadata: Option<MemoryMetadata>,
    pub access: Option<MemoryAccess>,
}

// Memory list response
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct MemoryListResponse {
    pub success: bool,
    pub memories: Vec<MemoryHeader>,
    pub message: String,
}

// ============================================================================
// RESOURCE BINDING TYPES
// ============================================================================

/// Resource types that can be bound to Neon database
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum ResourceType {
    Capsule,
    Gallery,
    Memory,
}

// ============================================================================
// TYPE MAPPING FUNCTIONS - Web2 ↔ ICP Conversion
// ============================================================================

impl Gallery {
    /// Convert Web2 gallery data to ICP gallery format

    pub fn from_web2(
        web2_gallery: &Web2Gallery,
        web2_items: &[Web2GalleryItem],
        owner_principal: Principal,
    ) -> Self {
        let memory_entries: Vec<GalleryMemoryEntry> = web2_items
            .iter()
            .map(GalleryMemoryEntry::from_web2)
            .collect();

        Self {
            id: web2_gallery.id.clone(),
            owner_principal,
            title: web2_gallery.title.clone(),
            description: web2_gallery.description.clone(),
            is_public: web2_gallery.is_public,
            created_at: Self::timestamp_to_nanoseconds(web2_gallery.created_at),
            updated_at: Self::timestamp_to_nanoseconds(web2_gallery.updated_at),
            storage_location: GalleryStorageLocation::Web2Only,
            memory_entries,
            bound_to_neon: false,
        }
    }

    /// Convert ICP gallery to Web2 format

    pub fn to_web2(&self) -> (Web2Gallery, Vec<Web2GalleryItem>) {
        let web2_gallery = Web2Gallery {
            id: self.id.clone(),
            owner_id: self.owner_principal.to_string(), // This would need proper mapping
            title: self.title.clone(),
            description: self.description.clone(),
            is_public: self.is_public,
            created_at: Self::nanoseconds_to_timestamp(self.created_at),
            updated_at: Self::nanoseconds_to_timestamp(self.updated_at),
        };

        let web2_items: Vec<Web2GalleryItem> = self
            .memory_entries
            .iter()
            .map(|entry| entry.to_web2(&self.id))
            .collect();

        (web2_gallery, web2_items)
    }

    /// Helper: Convert timestamp to nanoseconds

    fn timestamp_to_nanoseconds(timestamp: u64) -> u64 {
        timestamp * 1_000_000_000 // Convert seconds to nanoseconds
    }

    /// Helper: Convert nanoseconds to timestamp

    fn nanoseconds_to_timestamp(nanoseconds: u64) -> u64 {
        nanoseconds / 1_000_000_000 // Convert nanoseconds to seconds
    }

    /// Get gallery header for listing
    pub fn to_header(&self) -> GalleryHeader {
        GalleryHeader {
            id: self.id.clone(),
            name: self.title.clone(),
            memory_count: self.memory_entries.len() as u32,
            created_at: self.created_at,
            updated_at: self.updated_at,
            storage_location: self.storage_location.clone(),
        }
    }
}

impl GalleryMemoryEntry {
    /// Convert Web2 gallery item to ICP gallery memory entry

    pub fn from_web2(web2_item: &Web2GalleryItem) -> Self {
        Self {
            memory_id: web2_item.memory_id.clone(),
            position: web2_item.position as u32,
            gallery_caption: web2_item.caption.clone(),
            is_featured: web2_item.is_featured,
            gallery_metadata: serde_json::to_string(&web2_item.metadata)
                .unwrap_or_else(|_| "{}".to_string()),
        }
    }

    /// Convert ICP gallery memory entry to Web2 format

    pub fn to_web2(&self, gallery_id: &str) -> Web2GalleryItem {
        Web2GalleryItem {
            id: format!("web2_item_{}", ic_cdk::api::time()), // Generate new ID for Web2
            gallery_id: gallery_id.to_string(),
            memory_id: self.memory_id.clone(),
            memory_type: "image".to_string(), // This would need proper mapping
            position: self.position as i32,
            caption: self.gallery_caption.clone(),
            is_featured: self.is_featured,
            metadata: serde_json::from_str(&self.gallery_metadata)
                .unwrap_or_else(|_| serde_json::json!({})),
            created_at: 0, // This would need proper mapping
            updated_at: 0, // This would need proper mapping
        }
    }
}

// Web2 data structures for type mapping (these would typically be in a separate module)
// These are placeholder structures that match the Web2 database schema

#[derive(Clone, Debug)]

pub struct Web2Gallery {
    pub id: String,
    pub owner_id: String,
    pub title: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, Debug)]

pub struct Web2GalleryItem {
    pub id: String,
    pub gallery_id: String,
    pub memory_id: String,
    pub memory_type: String,
    pub position: i32,
    pub caption: Option<String>,
    pub is_featured: bool,
    pub metadata: serde_json::Value,
    pub created_at: u64,
    pub updated_at: u64,
}

// ============================================================================
// STORABLE TRAIT IMPLEMENTATIONS FOR STABLE MEMORY
// ============================================================================

impl Storable for UploadSession {
    fn to_bytes(&self) -> Cow<[u8]> {
        let bytes = candid::encode_one(self).expect("Failed to encode UploadSession");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).expect("Failed to decode UploadSession")
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 2048, // Increased to 2KB for additional fields
            is_fixed_size: false,
        };
}

impl Storable for ChunkData {
    fn to_bytes(&self) -> Cow<[u8]> {
        let bytes = candid::encode_one(self).expect("Failed to encode ChunkData");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).expect("Failed to decode ChunkData")
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 1_048_576 + 1024, // 1MB for chunk data + 1KB for metadata
            is_fixed_size: false,
        };
}

// MemoryArtifact Storable implementation removed
// ============================================================================
// TESTS FOR ICP ERROR MODEL
// ============================================================================

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_error_to_string() {
        assert_eq!(Error::Unauthorized.to_string(), "unauthorized access");
        assert_eq!(
            Error::Conflict("test".to_string()).to_string(),
            "conflict: test"
        );
        assert_eq!(Error::NotFound.to_string(), "resource not found");
        assert_eq!(
            Error::InvalidArgument("test".to_string()).to_string(),
            "invalid argument: test"
        );
        assert_eq!(
            Error::Internal("test error".to_string()).to_string(),
            "internal error: test error"
        );
    }

    #[test]
    fn test_result_ok() {
        let result: Result<String> = Ok("test data".to_string());
        assert!(result.is_ok());
        assert!(!result.is_err());
        assert_eq!(result.unwrap(), "test data".to_string());
    }

    #[test]
    fn test_result_err() {
        let result: Result<String> = Err(Error::NotFound);
        assert!(!result.is_ok());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::NotFound);
    }

    #[test]
    fn test_commit_response() {
        let ok_response = CommitResponse::ok(
            "memory_789".to_string(),
            "final_hash_xyz".to_string(),
            2048,
            "Upload completed".to_string(),
        );
        assert!(ok_response.success);
        assert_eq!(ok_response.memory_id, "memory_789");
        assert_eq!(ok_response.final_hash, "final_hash_xyz");
        assert_eq!(ok_response.total_bytes, 2048);
        assert_eq!(ok_response.message, "Upload completed");
        assert_eq!(ok_response.error, None);

        let err_response = CommitResponse::err(
            Error::InvalidArgument("invalid hash".to_string()),
            "memory_789".to_string(),
            "Hash mismatch".to_string(),
        );
        assert!(!err_response.success);
        assert_eq!(err_response.memory_id, "memory_789");
        assert_eq!(err_response.final_hash, "");
        assert_eq!(err_response.total_bytes, 0);
        assert_eq!(err_response.message, "Hash mismatch");
        assert_eq!(
            err_response.error,
            Some(Error::InvalidArgument("invalid hash".to_string()))
        );
    }
}

// ============================================================================
// ERROR CONVERSIONS - Bridge between storage layer and ICP layer
// ============================================================================
