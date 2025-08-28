use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use std::collections::HashMap;

// // Memory types as enum for type safety
// #[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq, Eq)]
// pub enum MemoryType {
//     Image,
//     Document,
//     Note,
//     Video,
//     Audio,
// }

// // Generic memory types (like database tables)
// pub type MemoryId = u64;

// #[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
// pub struct MemoryData {
//     pub id: MemoryId,
//     pub memory_type: MemoryType,
//     pub name: String,
//     pub content_type: String,
//     pub data: Vec<u8>,
// }

// #[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
// pub struct MemoryInfo {
//     pub id: MemoryId,
//     pub memory_type: MemoryType,
//     pub name: String,
//     pub content_type: String,
//     pub metadata: MemoryMetadata,
// }

// Memory metadata
// #[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
// pub struct MemoryMetadata {
//     pub size: u64,
//     pub mime_type: String,
//     pub original_name: String,
//     pub uploaded_at: String,
//     pub date_of_memory: Option<String>,
//     pub people_in_memory: Option<Vec<String>>,
//     pub format: Option<String>,
// }

// // Metadata structs for different memory types
// #[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
// pub struct ImageMetadata {
//     pub common: MemoryMetadata,
//     pub dimensions: Option<(u32, u32)>,
// }

// #[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
// pub struct VideoMetadata {
//     pub common: MemoryMetadata,
//     pub duration: Option<u32>, // Duration in seconds
//     pub width: Option<u32>,
//     pub height: Option<u32>,
//     pub thumbnail: Option<String>,
// }

// #[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
// pub struct AudioMetadata {
//     pub common: MemoryMetadata,
//     pub duration: Option<u32>, // Duration in seconds
//     pub format: Option<String>,
//     pub bitrate: Option<u32>,
//     pub sample_rate: Option<u32>,
//     pub channels: Option<u8>,
// }

// #[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
// pub struct DocumentMetadata {
//     pub common: MemoryMetadata,
// }

// #[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
// pub struct NoteMetadata {
//     pub tags: Option<Vec<String>>,
//     pub date_of_memory: Option<String>,
// }

// Simple extensions with just specific metadata
// #[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
// pub struct ImageMemory {
//     pub base: MemoryData,
//     pub metadata: ImageMetadata,
// }

// #[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
// pub struct VideoMemory {
//     pub base: MemoryData,
//     pub metadata: VideoMetadata,
// }

// #[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
// pub struct AudioMemory {
//     pub base: MemoryData,
//     pub metadata: AudioMetadata,
// }

// #[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
// pub struct DocumentMemory {
//     pub base: MemoryData,
//     pub metadata: DocumentMetadata,
// }

// #[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
// pub struct NoteMemory {
//     pub base: MemoryData,
//     pub content: String, // Notes store text content instead of binary data
//     pub metadata: NoteMetadata,
// }

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
}

// Blob storage reference
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum BlobKind {
    IcCanister, // stored in IC canister
    Http,       // HTTP URL
    Ipfs,       // IPFS CID
    External,   // external reference
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct BlobRef {
    pub kind: BlobKind,
    pub locator: String,        // canister+key, URL, CID, etc.
    pub hash: Option<[u8; 32]>, // optional integrity hash
}

// // Individual memory/content item (enhanced with capsule fields) - OLD VERSION
// #[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
// pub struct Memory {
//     pub id: String,
//     pub memory_type: MemoryType,
//     pub name: String,
//     pub content_type: String,
//     pub metadata: MemoryMetadata, // Rich metadata system we already have
//     pub created_at: u64,
//     pub updated_at: u64,
//     pub visibility: Visibility,
//     pub allowed: Vec<PersonRef>, // used when visibility is Custom
//     pub blob_ref: BlobRef,
// }

// Main capsule structure
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Capsule {
    pub id: String,                                       // unique capsule identifier
    pub subject: PersonRef,                               // who this capsule is about
    pub owners: HashMap<PersonRef, OwnerState>,           // 1..n owners (usually 1)
    pub controllers: HashMap<PersonRef, ControllerState>, // delegated admins (full control)
    pub connections: HashMap<PersonRef, Connection>,      // social graph
    pub memories: HashMap<String, Memory>,                // content
    pub created_at: u64,
    pub updated_at: u64,
}

// Capsule creation result
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct CapsuleCreationResult {
    pub success: bool,
    pub capsule_id: Option<String>,
    pub message: String,
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
    Custom(Vec<PersonRef>),

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
