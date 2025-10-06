use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::Storable;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashMap;

// Re-export types from specialized modules
pub use crate::memories::types::*;
pub use crate::unified_types::*;
pub use crate::upload::types::*;

// ============================================================================
// TYPE ALIASES
// ============================================================================

/// Type alias for capsule identifiers (used throughout the codebase)
pub type CapsuleId = String;
pub type MemoryId = String;

// ============================================================================
// CANDID RESULT TYPES
// ============================================================================

/// Result type for uploads_begin function (SessionId or Error)
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum Result13 {
    Ok(u64),
    Err(Error),
}

/// Result type for verify_nonce function (Principal or Error)
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum Result14 {
    Ok(Principal),
    Err(Error),
}

/// Result type for uploads_finish function (text or Error)
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum Result6 {
    Ok(String),
    Err(Error),
}

/// Result type for memories_create function (MemoryId or Error)
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum Result20 {
    Ok(String), // MemoryId
    Err(Error),
}

/// Discriminated union for structured asset data
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum MemoryAssetData {
    Inline {
        bytes: Vec<u8>,
        content_type: String,
        size: u64,
        sha256: Option<Vec<u8>>,
    },
    InternalBlob {
        blob_id: String,
        size: u64,
        sha256: Option<Vec<u8>>,
    },
    ExternalUrl {
        url: String,
        size: Option<u64>,
        sha256: Option<Vec<u8>>,
    },
}

/// Standardized bulk operation result with per-item failure tracking
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct BulkResult<TId> {
    pub ok: Vec<TId>,
    pub failed: Vec<BulkFailure<TId>>,
}

/// Individual failure in bulk operation
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct BulkFailure<TId> {
    pub id: TId,
    pub err: Error,
}

// ============================================================================
// ID HYGIENE - FUTURE NEWTYPE WRAPPERS (PHASE 5 DEFERRED)
// ============================================================================

// NOTE: Phase 5 (ID Hygiene) is deferred due to extensive codebase changes required.
// The newtype wrappers would require updating all function signatures and trait implementations.
// For now, we keep the existing type aliases for backward compatibility.
//
// Future implementation would include:
// - MemoryId(String) newtype wrapper
// - CapsuleId(String) newtype wrapper
// - AssetId(String) newtype wrapper
// - Proper trait implementations (Display, Ord, etc.)
// - Conversion helpers between Rust and DID types

// UploadFinishResult moved to upload/types.rs

// Result_15 moved to upload/types.rs (uses UploadFinishResult from there)

// Storage edge types moved to unified_types.rs

// Asset metadata types moved to memories/types.rs

// All asset metadata types moved to memories/types.rs

// AssetMetadata impl moved to memories/types.rs

// UploadConfig moved to upload/types.rs

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
    NotImplemented(String),  // feature not yet implemented
}

// Canonical Rust Result type
// Removed ApiResult and UnitResult aliases - use std::result::Result<T, Error> directly

// std::result::Result<T> removed - using canonical Result<T> = std::result::Result<T, Error>

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Unauthorized => write!(f, "unauthorized access"),
            Error::NotFound => write!(f, "resource not found"),
            Error::InvalidArgument(msg) => write!(f, "invalid argument: {}", msg.to_lowercase()),
            Error::Conflict(msg) => write!(f, "conflict: {}", msg.to_lowercase()),
            Error::ResourceExhausted => write!(f, "resource exhausted"),
            Error::Internal(msg) => write!(f, "internal error: {}", msg.to_lowercase()),
            Error::NotImplemented(msg) => write!(f, "not implemented: {}", msg.to_lowercase()),
        }
    }
}

impl Error {
    // Removed unused method: code
}

// Helper functions to convert between error patterns
impl CommitResponse {
    #[allow(dead_code)] // Used in tests
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

    #[allow(dead_code)] // Used in tests
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

// UploadSession and ChunkData moved to upload/types.rs

// MemoryPresenceResult moved to memories/types.rs

// SimpleMemoryMetadata moved to memories/types.rs

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct CommitResponse {
    pub success: bool,
    pub memory_id: String,
    pub final_hash: String,
    pub total_bytes: u64,
    pub message: String,
    pub error: Option<Error>,
}

// Memory sync types - REMOVED: unused
// #[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
// pub struct MemorySyncRequest { ... }
// pub struct MemorySyncResult { ... }
// pub struct BatchMemorySyncResponse { ... }

// UserMemoriesResponse - REMOVED: unused
// #[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
// pub struct UserMemoriesResponse { ... }

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

// UserRegistrationResult - REMOVED: unused
// #[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
// pub struct UserRegistrationResult { ... }

// Capsule types for user-owned data architecture
// Core person reference - can be a live principal or opaque identifier
#[derive(
    CandidType, Deserialize, Serialize, Clone, Eq, PartialEq, Hash, Debug, PartialOrd, Ord,
)]
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

impl std::fmt::Display for PersonRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersonRef::Principal(p) => write!(f, "{}", p),
            PersonRef::Opaque(s) => write!(f, "{}", s),
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

// Hosting preference enums - mirrors Web2 structure
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum FrontendHosting {
    Vercel,
    Icp,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum BackendHosting {
    Vercel,
    Icp,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum DatabaseHosting {
    Neon,
    Icp,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum BlobHosting {
    S3,
    VercelBlob,
    Icp,
    Arweave,
    Ipfs,
    Neon,
}

// Hosting preferences for capsule - mirrors Web2 user_hosting_preferences
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct HostingPreferences {
    pub frontend_hosting: FrontendHosting,
    pub backend_hosting: BackendHosting,
    pub database_hosting: DatabaseHosting,
    pub blob_hosting: BlobHosting,
}

impl Default for HostingPreferences {
    fn default() -> Self {
        Self {
            frontend_hosting: FrontendHosting::Icp,
            backend_hosting: BackendHosting::Icp,
            database_hosting: DatabaseHosting::Icp,
            blob_hosting: BlobHosting::Icp,
        }
    }
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
    pub bound_to_neon: bool,         // Neon database binding status
    pub inline_bytes_used: u64,      // Track inline storage consumption
    pub has_advanced_settings: bool, // Controls whether user sees advanced settings panels
    pub hosting_preferences: HostingPreferences, // User's preferred hosting providers
}

// CapsuleRegistrationResult - REMOVED: unused
// #[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
// pub struct CapsuleRegistrationResult { ... }

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
    pub memory_count: u64,     // Number of memories in this capsule
    pub gallery_count: u64,    // Number of galleries in this capsule
    pub connection_count: u64, // Number of connections to other people
}

// Capsule header for listing
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct CapsuleHeader {
    pub id: String,
    pub subject: PersonRef,
    pub owner_count: u64,
    pub controller_count: u64,
    pub memory_count: u64,
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

/// User settings data for updating capsule settings
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct UserSettingsUpdateData {
    pub has_advanced_settings: Option<bool>,
}

/// User settings response for reading capsule settings
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct UserSettingsResponse {
    pub has_advanced_settings: bool,
    pub hosting_preferences: HostingPreferences,
}

// MemoryHeader moved to memories/types.rs

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct GalleryHeader {
    pub id: String,
    pub name: String,
    pub memory_count: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub storage_location: GalleryStorageLocation,
}

// New unified memory system
// Old metadata structures removed - replaced with new AssetMetadata enum and AssetMetadataBase

// MemoryType moved to upload/types.rs

// MemoryAccess moved to memories/types.rs

// AccessEvent moved to memories/types.rs

// MemoryMetadata moved to memories/types.rs

// External blob storage types - REMOVED: unused
// #[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
// pub enum MemoryBlobKindExternal { ... }

// BlobRef moved to memories/types.rs

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct BlobMeta {
    pub size: u64,        // total size in bytes
    pub chunk_count: u32, // number of chunks
}

// #[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
// pub struct MemoryData {
//     pub blob_ref: BlobRef,     // where the data is stored
//     pub data: Option<Vec<u8>>, // inline data (for IcCanister storage)
// }

// AssetType moved to memories/types.rs

// Inline asset (stored directly in memory)
// MemoryAssetInline moved to memories/types.rs

// MemoryAssetBlobInternal moved to memories/types.rs

// MemoryAssetBlobExternal moved to memories/types.rs

// MemoryAssetBlob moved to memories/types.rs

// Memory moved to memories/types.rs

// Memory implementation moved to memory.rs

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

// GalleryMemoryEntry moved to memories/types.rs

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

// User principal management types - REMOVED: unused
// #[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
// pub struct UserPrincipalData { ... }
// pub struct RegisterUserResponse { ... }
// pub struct LinkUserResponse { ... }

// VerificationResult - REMOVED: unused
// #[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
// pub struct VerificationResult { ... }

// MemoryOperationResponse moved to memories/types.rs

// MemoryUpdateData moved to memories/types.rs

// MemoryListResponse moved to memories/types.rs

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

// Gallery and GalleryMemoryEntry implementations moved to gallery.rs

// Note: Web2 data structures removed - not currently used

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
        let result: std::result::Result<String, Error> = Ok("test data".to_string());
        assert!(result.is_ok());
        assert!(!result.is_err());
        assert_eq!(result.unwrap(), "test data".to_string());
    }

    #[test]
    fn test_result_err() {
        let result: std::result::Result<String, Error> = Err(Error::NotFound);
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
