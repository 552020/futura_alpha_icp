use crate::types::{AssetMetadata, CapsuleId, MemoryId, MemoryType};
use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::{storable::Bound, Storable};
use serde::Serialize;
use std::borrow::Cow;

// ============================================================================
// UPLOAD TYPES (moved from unified_types.rs)
// ============================================================================

/// Storage backend types
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq, Eq)]
pub enum StorageBackend {
    S3,
    Icp,
    VercelBlob,
    Arweave,
    Ipfs,
}

/// Processing status types
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq, Eq)]
pub enum ProcessingStatus {
    Uploading,
    Processing,
    Finalizing,
    Completed,
    Error,
}

// MemoryType moved to memories/types.rs

/// Unified upload result for all storage backends
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct UploadFinishResult {
    pub memory_id: String,
    pub blob_id: String,
    pub remote_id: Option<String>,
    pub size: u64,
    pub checksum_sha256: Option<[u8; 32]>, // 32 bytes
    pub storage_backend: StorageBackend,
    pub storage_location: String, // URL or key
    pub uploaded_at: u64,         // ms since epoch
    pub expires_at: Option<u64>,
}

/// Unified upload progress for all storage backends
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct UploadProgress {
    pub file_index: u32,
    pub total_files: u32,
    pub current_file: String,
    pub bytes_uploaded: u64,
    pub total_bytes: u64,
    pub pct_bp: u16, // 0..10000 basis points
    pub status: ProcessingStatus,
    pub message: Option<String>,
}

/// Upload configuration for client discoverability
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct UploadConfig {
    /// Maximum size for inline uploads (bytes)
    pub inline_max: u32,
    /// Recommended chunk size for chunked uploads (bytes)
    pub chunk_size: u32,
    /// Maximum inline budget per capsule (bytes)
    pub inline_budget_per_capsule: u32,
}

/// Upload session for tracking upload progress
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct UploadSession {
    pub session_id: String,
    pub memory_id: String,
    pub memory_type: MemoryType,
    pub expected_hash: String,
    pub chunk_count: u32,
    pub total_size: u64,
    pub created_at: u64,
    pub chunks_received: Vec<bool>, // Track which chunks have been received
    pub bytes_received: u64,        // Total bytes received so far
}

/// Structure for storing individual chunk data
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ChunkData {
    pub session_id: String,
    pub chunk_index: u32,
    pub data: Vec<u8>,
    pub received_at: u64,
}

/// Commit response for upload operations
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct CommitResponse {
    pub success: bool,
    pub memory_id: String,
    pub final_hash: String,
    pub total_bytes: u64,
    pub message: String,
    pub error: Option<crate::types::Error>,
}

// Size constants aligned with senior developer feedback
pub const INLINE_MAX: u64 = 32 * 1024; // 32KB (fits in Capsule bound)
pub const CHUNK_SIZE: usize = 1_800_000; // 1.8MB - ICP expert recommended optimal size
                                         // Removed unused constant: PAGE_SIZE
pub const CAPSULE_INLINE_BUDGET: u64 = 32 * 1024; // Max inline bytes per capsule

// Re-export SessionId from session module to avoid duplication
pub use crate::session::types::SessionId;

impl Storable for SessionId {
    const BOUND: Bound = Bound::Bounded {
        max_size: 8,
        is_fixed_size: true,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(self.0.to_le_bytes().to_vec())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let arr: [u8; 8] = bytes
            .as_ref()
            .try_into()
            .expect("Invalid SessionId bytes - expected 8 bytes");
        SessionId(u64::from_le_bytes(arr))
    }
}

/// Blob identifier using u64 for efficient storage
#[derive(Clone, Debug, CandidType, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlobId(pub u64);

impl Default for BlobId {
    fn default() -> Self {
        Self::new()
    }
}

impl BlobId {
    pub fn new() -> Self {
        use crate::upload::blob_store::STABLE_BLOB_COUNTER;
        let id = STABLE_BLOB_COUNTER.with(|counter| {
            let mut c = counter.borrow_mut();
            let current = c.get();
            let next_id = current + 1;
            c.set(next_id).expect("Failed to increment blob counter");
            next_id
        });
        BlobId(id)
    }
}

impl Storable for BlobId {
    const BOUND: Bound = Bound::Bounded {
        max_size: 8,
        is_fixed_size: true,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(self.0.to_le_bytes().to_vec())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let arr: [u8; 8] = bytes
            .as_ref()
            .try_into()
            .expect("Invalid BlobId bytes - expected 8 bytes");
        BlobId(u64::from_le_bytes(arr))
    }
}

// Re-export SessionStatus from session module to avoid duplication
pub use crate::session::types::SessionStatus;

/// Upload session metadata with crash safety and authorization
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct SessionMeta {
    pub capsule_id: CapsuleId,
    pub provisional_memory_id: MemoryId,
    pub caller: candid::Principal,
    pub chunk_count: u32,
    pub expected_len: Option<u64>,
    pub expected_hash: Option<[u8; 32]>,
    pub status: SessionStatus,
    pub created_at: u64,
    pub asset_metadata: AssetMetadata,
    pub idem: String,
}

impl Storable for SessionMeta {
    const BOUND: Bound = Bound::Bounded {
        max_size: 2048,
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(&(1u16, self)).expect("Failed to encode SessionMeta"))
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let (version, meta): (u16, SessionMeta) =
            Decode!(bytes.as_ref(), (u16, SessionMeta)).expect("Failed to decode SessionMeta");
        assert_eq!(version, 1, "Unsupported SessionMeta version");
        meta
    }
}

/// Blob metadata for integrity verification
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct BlobMeta {
    pub size: u64,
    pub checksum: [u8; 32],
    pub created_at: u64,
    pub pmid_hash: [u8; 32], // SHA256 of provisional_memory_id for deterministic key lookups
}

impl Storable for BlobMeta {
    const BOUND: Bound = Bound::Bounded {
        max_size: 512,
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(&(1u16, self)).expect("Failed to encode BlobMeta"))
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let (version, meta): (u16, BlobMeta) =
            Decode!(bytes.as_ref(), (u16, BlobMeta)).expect("Failed to decode BlobMeta");
        assert_eq!(version, 1, "Unsupported BlobMeta version");
        meta
    }
}

/// Result type for uploads_finish function (UploadFinishResult or Error)
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum Result_15 {
    Ok(UploadFinishResult),
    Err(crate::types::Error),
}
