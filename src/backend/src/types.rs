use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use std::collections::HashMap;

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

// Response types
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemoryResponse {
    pub success: bool,
    pub data: Option<String>,
    pub error: Option<String>,
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

// Connection status for peer relationships
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum ConnectionStatus {
    Pending,
    Accepted,
    Blocked,
    Revoked,
}

// Connection between persons
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Connection {
    pub peer: PersonRef,
    pub status: ConnectionStatus,
    pub created_at: u64,
    pub updated_at: u64,
}

// Connection groups for organizing relationships
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ConnectionGroup {
    pub id: String,
    pub name: String, // "Family", "Close Friends", etc.
    pub description: Option<String>,
    pub members: Vec<PersonRef>,
    pub created_at: u64,
    pub updated_at: u64,
}

// Controller state tracking (simplified - full control except ownership transfer)
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ControllerState {
    pub granted_at: u64,
    pub granted_by: PersonRef,
}

// Owner state tracking
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
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
    pub bound_to_web2: bool, // Web2 (NextAuth) binding status
}

// Capsule creation result
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct CapsuleCreationResult {
    pub success: bool,
    pub capsule_id: Option<String>,
    pub message: String,
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
    pub bound_to_web2: bool,
    pub created_at: u64,
    pub updated_at: u64,
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

// New unified memory system
// Base metadata that all memory types share
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemoryMetadataBase {
    pub size: u64,
    pub mime_type: String,
    pub original_name: String,
    pub uploaded_at: String,
    pub date_of_memory: Option<String>,
    pub people_in_memory: Option<Vec<String>>,
    pub format: Option<String>,
}

// Extended metadata for specific memory types
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ImageMetadata {
    pub base: MemoryMetadataBase,
    pub dimensions: Option<(u32, u32)>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct VideoMetadata {
    pub base: MemoryMetadataBase,
    pub duration: Option<u32>, // Duration in seconds
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub thumbnail: Option<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct AudioMetadata {
    pub base: MemoryMetadataBase,
    pub duration: Option<u32>, // Duration in seconds
    pub format: Option<String>,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct DocumentMetadata {
    pub base: MemoryMetadataBase,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct NoteMetadata {
    pub base: MemoryMetadataBase,
    pub tags: Option<Vec<String>>,
}

// Enum for different metadata types
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub enum MemoryMetadata {
    Image(ImageMetadata),
    Video(VideoMetadata),
    Audio(AudioMetadata),
    Document(DocumentMetadata),
    Note(NoteMetadata),
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub enum MemoryType {
    Image,
    Video,
    Audio,
    Document,
    Note,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub enum MemoryAccess {
    Public,
    Private,
    Custom {
        individuals: Vec<PersonRef>, // direct individual access
        groups: Vec<String>,         // group access (group IDs)
    },

    // Time-based access
    Scheduled {
        accessible_after: u64,     // nanoseconds since Unix epoch
        access: Box<MemoryAccess>, // what it becomes after the time
    },

    // Event-based access
    EventTriggered {
        trigger_event: AccessEvent,
        access: Box<MemoryAccess>, // what it becomes after the event
    },
}

// Events that can trigger access changes
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
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
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemoryInfo {
    pub memory_type: MemoryType,
    pub name: String,
    pub content_type: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub uploaded_at: u64,
    pub date_of_memory: Option<u64>, // when the actual event happened
}

// External blob storage types
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum MemoryBlobKindExternal {
    Http,    // HTTP URL
    Ipfs,    // IPFS CID
    Arweave, // Arweave CID
    AWS,     // AWS S3
}

// Blob storage reference
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum MemoryBlobKind {
    ICPCapsule,             // stored in IC canister
    MemoryBlobKindExternal, // external reference
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct BlobRef {
    pub kind: MemoryBlobKind,
    pub locator: String,        // canister+key, URL, CID, etc.
    pub hash: Option<[u8; 32]>, // optional integrity hash
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemoryData {
    pub blob_ref: BlobRef,     // where the data is stored
    pub data: Option<Vec<u8>>, // inline data (for IcCanister storage)
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Memory {
    pub id: String,               // unique identifier
    pub info: MemoryInfo,         // basic info (name, type, timestamps)
    pub metadata: MemoryMetadata, // rich metadata (size, dimensions, etc.)
    pub access: MemoryAccess,     // who can access + temporal rules
    pub data: MemoryData,         // actual data + storage location
}

// ============================================================================
// GALLERY SYSTEM - Minimal Abstraction Approach
// ============================================================================

// Gallery storage status for tracking where gallery data is stored
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum GalleryStorageStatus {
    Web2Only,  // Stored only in Web2 database
    ICPOnly,   // Stored only in ICP canister
    Both,      // Stored in both Web2 and ICP
    Migrating, // Currently being migrated from Web2 to ICP
    Failed,    // Migration or storage failed
}

// Minimal extra data for gallery memory entries
// Only stores what memories don't already have
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct GalleryMemoryEntry {
    pub memory_id: String,               // Reference to existing memory
    pub position: u32,                   // Gallery-specific ordering
    pub gallery_caption: Option<String>, // Only if different from memory caption
    pub is_featured: bool,               // Gallery-specific highlighting
    pub gallery_metadata: String,        // JSON for gallery-specific annotations
}

// Main gallery structure with embedded memory entries
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Gallery {
    pub id: String,                              // unique gallery identifier
    pub owner_principal: Principal,              // who owns this gallery
    pub title: String,                           // gallery title
    pub description: Option<String>,             // gallery description
    pub is_public: bool,                         // whether gallery is publicly accessible
    pub created_at: u64,                         // creation timestamp (nanoseconds)
    pub updated_at: u64,                         // last update timestamp (nanoseconds)
    pub storage_status: GalleryStorageStatus,    // where this gallery is stored
    pub memory_entries: Vec<GalleryMemoryEntry>, // minimal extra data for each memory
}

// Gallery creation result
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct GalleryCreationResult {
    pub success: bool,
    pub gallery_id: Option<String>,
    pub message: String,
}

// Gallery storage result for "Store Forever" feature
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct StoreGalleryResponse {
    pub success: bool,
    pub gallery_id: Option<String>,
    pub icp_gallery_id: Option<String>, // ID in ICP canister
    pub message: String,
    pub storage_status: GalleryStorageStatus,
}

// Gallery update result
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct UpdateGalleryResponse {
    pub success: bool,
    pub gallery: Option<Gallery>,
    pub message: String,
}

// Gallery deletion result
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct DeleteGalleryResponse {
    pub success: bool,
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
    pub bound_to_web2: bool,          // Whether linked to Web2 account
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
    pub memories: Vec<Memory>,
    pub message: String,
}

// ============================================================================
// TYPE MAPPING FUNCTIONS - Web2 ↔ ICP Conversion
// ============================================================================

impl Gallery {
    /// Convert Web2 gallery data to ICP gallery format
    #[allow(dead_code)]
    pub fn from_web2(
        web2_gallery: &Web2Gallery,
        web2_items: &[Web2GalleryItem],
        owner_principal: Principal,
    ) -> Self {
        let memory_entries: Vec<GalleryMemoryEntry> = web2_items
            .iter()
            .map(|item| GalleryMemoryEntry::from_web2(item))
            .collect();

        Self {
            id: web2_gallery.id.clone(),
            owner_principal,
            title: web2_gallery.title.clone(),
            description: web2_gallery.description.clone(),
            is_public: web2_gallery.is_public,
            created_at: Self::timestamp_to_nanoseconds(web2_gallery.created_at),
            updated_at: Self::timestamp_to_nanoseconds(web2_gallery.updated_at),
            storage_status: GalleryStorageStatus::Web2Only,
            memory_entries,
        }
    }

    /// Convert ICP gallery to Web2 format
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    fn timestamp_to_nanoseconds(timestamp: u64) -> u64 {
        timestamp * 1_000_000_000 // Convert seconds to nanoseconds
    }

    /// Helper: Convert nanoseconds to timestamp
    #[allow(dead_code)]
    fn nanoseconds_to_timestamp(nanoseconds: u64) -> u64 {
        nanoseconds / 1_000_000_000 // Convert nanoseconds to seconds
    }
}

impl GalleryMemoryEntry {
    /// Convert Web2 gallery item to ICP gallery memory entry
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
