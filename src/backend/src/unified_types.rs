/**
 * Unified types for all storage backends
 *
 * This file implements the tech lead's recommended Option S (snake_case everywhere)
 * approach for unified type system across frontend and backend.
 */
use candid::{CandidType, Deserialize};
use serde::Serialize;

// ============================================================================
// CORE ENUMS
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

// AssetType moved to memories/types.rs
// MemoryType moved to upload/types.rs

/// Database storage edge types - where memory metadata/records are stored
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq, Eq)]
pub enum StorageEdgeDatabaseType {
    Icp,  // ICP canister storage
    Neon, // Neon database
}

/// Blob storage edge types - where asset data is stored
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq, Eq)]
pub enum StorageEdgeBlobType {
    Icp,        // ICP canister storage
    VercelBlob, // Vercel Blob storage
    S3,         // AWS S3 storage
    Arweave,    // Arweave storage
    Ipfs,       // IPFS storage
    Neon,       // Neon database - for small assets
}

// ============================================================================
// UPLOAD TYPES - MOVED TO upload/types.rs
// ============================================================================

// Upload types have been moved to upload/types.rs to avoid duplication
// The following types are now defined in upload/types.rs:
// - StorageBackend
// - ProcessingStatus
// - MemoryType
// - UploadFinishResult
// - UploadProgress
// - UploadConfig
// - UploadSession
// - ChunkData
// - CommitResponse

// ============================================================================
// ASSET METADATA TYPES - MOVED TO memories/types.rs
// ============================================================================

// Asset metadata types have been moved to memories/types.rs to avoid duplication
// The following types are now defined in memories/types.rs:
// - AssetType
// - AssetMetadataBase
// - ImageAssetMetadata
// - VideoAssetMetadata
// - AudioAssetMetadata
// - DocumentAssetMetadata
// - NoteAssetMetadata
// - AssetMetadata

// ============================================================================
// RESULT TYPES
// ============================================================================

/// Use standard Rust Result<T, Error> - no custom enum needed
/// The tech lead's Candid spec will map to std::result::Result<T, Error>
// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

// UploadProgress impl moved to upload/types.rs
// UploadFinishResult impl moved to upload/types.rs
// AssetMetadata impl moved to memories/types.rs

// ============================================================================
// STORABLE IMPLEMENTATIONS
// ============================================================================
use ic_stable_structures::Storable;
use std::borrow::Cow;

// Storable implementations moved to upload/types.rs
// - impl Storable for UploadSession
// - impl Storable for ChunkData
