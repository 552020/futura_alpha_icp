use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use std::collections::HashMap;

// Memory types as enum for type safety
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq, Eq)]
pub enum MemoryType {
    Image,
    Document,
    Note,
    Video,
    Audio,
}

// Generic memory types (like database tables)
pub type MemoryId = u64;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemoryData {
    pub id: MemoryId,
    pub memory_type: MemoryType,
    pub name: String,
    pub content_type: String,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemoryInfo {
    pub id: MemoryId,
    pub memory_type: MemoryType,
    pub name: String,
    pub content_type: String,
    pub metadata: MemoryMetadata,
}

// Memory metadata
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemoryMetadata {
    pub size: u64,
    pub mime_type: String,
    pub original_name: String,
    pub uploaded_at: String,
    pub date_of_memory: Option<String>,
    pub people_in_memory: Option<Vec<String>>,
    pub format: Option<String>,
}

// Metadata structs for different memory types
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ImageMetadata {
    pub common: MemoryMetadata,
    pub dimensions: Option<(u32, u32)>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct VideoMetadata {
    pub common: MemoryMetadata,
    pub duration: Option<u32>, // Duration in seconds
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub thumbnail: Option<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct AudioMetadata {
    pub common: MemoryMetadata,
    pub duration: Option<u32>, // Duration in seconds
    pub format: Option<String>,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct DocumentMetadata {
    pub common: MemoryMetadata,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct NoteMetadata {
    pub tags: Option<Vec<String>>,
    pub date_of_memory: Option<String>,
}

// Simple extensions with just specific metadata
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ImageMemory {
    pub base: MemoryData,
    pub metadata: ImageMetadata,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct VideoMemory {
    pub base: MemoryData,
    pub metadata: VideoMetadata,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct AudioMemory {
    pub base: MemoryData,
    pub metadata: AudioMetadata,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct DocumentMemory {
    pub base: MemoryData,
    pub metadata: DocumentMetadata,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct NoteMemory {
    pub base: MemoryData,
    pub content: String, // Notes store text content instead of binary data
    pub metadata: NoteMetadata,
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

// Memory visibility levels
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum Visibility {
    Private,     // only owners/controllers
    Connections, // accepted connections can see
    Public,      // anyone can see
    Custom,      // specific allowed list
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

// Memory metadata
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, Default)]
pub struct MemoryMeta {
    pub title: Option<String>,
    pub tags: Vec<String>,
}

// Individual memory/content item
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Memory {
    pub id: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub visibility: Visibility,
    pub allowed: Vec<PersonRef>, // used when visibility is Custom
    pub blob_ref: BlobRef,
    pub meta: MemoryMeta,
}

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
