# Backend and Frontend Types Reference

This document contains the current type definitions from both backend (backend.did + Rust source) and frontend (TypeScript) for AssetMetadata and UploadResult types.

## Backend Types

### From backend.did (Candid Interface)

### AssetMetadata

```did
type AssetMetadata = variant {
  Note : NoteAssetMetadata;
  Image : ImageAssetMetadata;
  Document : DocumentAssetMetadata;
  Audio : AudioAssetMetadata;
  Video : VideoAssetMetadata;
};
```

### AssetMetadataBase

```did
type AssetMetadataBase = record {
  url : opt text;
  height : opt nat32;
  updated_at : nat64;
  asset_type : AssetType;
  sha256 : opt blob;
  name : text;
  storage_key : opt text;
  tags : vec text;
  processing_error : opt text;
  mime_type : text;
  description : opt text;
  created_at : nat64;
  deleted_at : opt nat64;
  bytes : nat64;
  asset_location : opt text;
  width : opt nat32;
  processing_status : opt text;
  bucket : opt text;
};
```

### AssetType

```did
type AssetType = variant { Preview; Metadata; Derivative; Original; Thumbnail };
```

### NoteAssetMetadata

```did
type NoteAssetMetadata = record {
  base : AssetMetadataBase;
  language : opt text;
  word_count : opt nat32;
  format : opt text;
};
```

### ImageAssetMetadata

```did
type ImageAssetMetadata = record {
  dpi : opt nat32;
  color_space : opt text;
  base : AssetMetadataBase;
  exif_data : opt text;
  compression_ratio : opt float32;
  orientation : opt nat8;
};
```

### DocumentAssetMetadata

```did
type DocumentAssetMetadata = record {
  document_type : opt text;
  base : AssetMetadataBase;
  language : opt text;
  page_count : opt nat32;
  word_count : opt nat32;
};
```

### AudioAssetMetadata

```did
type AudioAssetMetadata = record {
  duration : opt nat64;
  base : AssetMetadataBase;
  codec : opt text;
  channels : opt nat8;
  sample_rate : opt nat32;
  bit_depth : opt nat8;
  bitrate : opt nat64;
};
```

### VideoAssetMetadata

```did
type VideoAssetMetadata = record {
  duration : opt nat64;
  base : AssetMetadataBase;
  codec : opt text;
  frame_rate : opt float32;
  resolution : opt text;
  bitrate : opt nat64;
  aspect_ratio : opt float32;
};
```

### UploadResult (UploadFinishResult)

```did
type UploadFinishResult = record { blob_id : text; memory_id : text };
```

### UploadConfig

```did
type UploadConfig = record {
  inline_max : nat32;
  chunk_size : nat32;
  inline_budget_per_capsule : nat32;
};
```

### From Rust Source Code (src/backend/src/types.rs)

### AssetMetadataBase (Rust)

```rust
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
```

### AssetMetadata (Rust)

```rust
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum AssetMetadata {
    Image(ImageAssetMetadata),
    Video(VideoAssetMetadata),
    Audio(AudioAssetMetadata),
    Document(DocumentAssetMetadata),
    Note(NoteAssetMetadata),
}
```

### ImageAssetMetadata (Rust)

```rust
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct ImageAssetMetadata {
    pub base: AssetMetadataBase,
    pub color_space: Option<String>,
    pub exif_data: Option<String>,
    pub compression_ratio: Option<f32>,
    pub dpi: Option<u32>,
    pub orientation: Option<u8>,
}
```

### VideoAssetMetadata (Rust)

```rust
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
```

### AudioAssetMetadata (Rust)

```rust
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
```

### DocumentAssetMetadata (Rust)

```rust
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct DocumentAssetMetadata {
    pub base: AssetMetadataBase,
    pub page_count: Option<u32>,       // Number of pages
    pub document_type: Option<String>, // PDF, DOCX, etc.
    pub language: Option<String>,      // Document language
    pub word_count: Option<u32>,       // Word count
}
```

### NoteAssetMetadata (Rust)

```rust
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct NoteAssetMetadata {
    pub base: AssetMetadataBase,
    pub word_count: Option<u32>,  // Word count
    pub language: Option<String>, // Note language
    pub format: Option<String>,   // Markdown, plain text, etc.
}
```

### AssetType (Rust)

```rust
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum AssetType {
    Original,   // Original file
    Thumbnail,  // Small preview/thumbnail
    Preview,    // Medium preview
    Derivative, // Processed/derived version
    Metadata,   // Metadata-only asset
}
```

### UploadFinishResult (Rust)

```rust
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct UploadFinishResult {
    pub blob_id: String,
    pub memory_id: String,
}
```

### UploadConfig (Rust)

```rust
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct UploadConfig {
    /// Maximum size for inline uploads (bytes)
    pub inline_max: u32,
    /// Recommended chunk size for chunked uploads (bytes)
    pub chunk_size: u32,
    /// Maximum inline budget per capsule (bytes)
    pub inline_budget_per_capsule: u32,
}
```

### UploadSession (Rust)

```rust
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
```

### StorageEdgeBlobType (Rust)

```rust
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum StorageEdgeBlobType {
    Icp,        // ICP canister storage
    VercelBlob, // Vercel Blob storage
    S3,         // AWS S3 storage
    Arweave,    // Arweave storage
    Ipfs,       // IPFS storage
    Neon,       // Neon database - for small assets
}
```

### StorageEdgeDatabaseType (Rust)

```rust
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum StorageEdgeDatabaseType {
    Icp,  // ICP canister storage
    Neon, // Neon database
}
```

## Frontend Types (TypeScript)

### UploadResult

```typescript
export interface UploadResult {
  // Core identifiers
  memoryId: string;
  blobId: string;
  remoteId: string;

  // File metadata
  size: number;
  checksumSha256: string | null;

  // Storage backend info
  storageBackend: "s3" | "icp" | "vercel-blob" | "arweave" | "ipfs";
  storageLocation: string; // URL or storage key

  // Timestamps
  uploadedAt: Date;
  expiresAt?: Date;
}
```

### UploadProgress

```typescript
export interface UploadProgress {
  // File tracking
  fileIndex: number;
  totalFiles: number;
  currentFile: string;

  // Progress metrics
  bytesUploaded: number;
  totalBytes: number;
  percentage: number;

  // Status
  status: "uploading" | "processing" | "finalizing" | "completed" | "error";
  message?: string;
}
```

### UploadServiceResult

```typescript
export interface UploadServiceResult {
  // Core data
  data: { id: string };
  results: UploadResult[];
  userId: string;

  // Metadata
  totalFiles: number;
  totalSize: number;
  processingTime: number;

  // Storage info
  storageBackend: "s3" | "icp" | "vercel-blob" | "arweave" | "ipfs";
  databaseBackend: "neon" | "icp";
}
```

### UploadLimits

```typescript
export interface UploadLimits {
  // File size limits
  maxFileSizeBytes: number;
  maxTotalSizeBytes: number;
  maxFilesPerUpload: number;

  // Storage-specific limits
  inlineMaxBytes: number;
  chunkSizeBytes?: number;
  maxChunks?: number;

  // Time limits
  uploadTimeoutMs: number;
  sessionTimeoutMs: number;
}
```

### ProcessedBlobs

```typescript
export interface ProcessedBlobs {
  display?: {
    blob: Blob;
    mimeType: string;
    width: number;
    height: number;
    bytes: number;
  };
  thumb?: {
    blob: Blob;
    mimeType: string;
    width: number;
    height: number;
    bytes: number;
  };
  placeholder?: {
    dataUrl: string;
    width: number;
    height: number;
    bytes: number;
  };
}
```

### ProcessedAssets

```typescript
export interface ProcessedAssets {
  display?: {
    url: string;
    storageKey: string;
    assetLocation: string;
  };
  thumb?: {
    url: string;
    storageKey: string;
    assetLocation: string;
  };
  placeholder?: {
    url: string;
    storageKey: string;
    assetLocation: string;
  };
}
```

### UploadError

```typescript
export interface UploadError {
  code: string;
  message: string;
  details?: Record<string, any>;
  retryable: boolean;
  timestamp: Date;
}
```

### Type Definitions

```typescript
export type UploadMode = "single" | "multiple-files" | "directory";
export type StorageBackend = "s3" | "icp" | "vercel-blob" | "arweave" | "ipfs";
export type DatabaseBackend = "neon" | "icp";
export type AssetType = "original" | "display" | "thumb" | "placeholder";
export type AssetLocation = "s3" | "icp" | "vercel-blob" | "arweave" | "ipfs" | "neon";
```

## Storage-Specific Configuration Types

### S3UploadConfig

```typescript
export interface S3UploadConfig {
  bucket: string;
  region: string;
  accessKey: string;
  secretKey: string;
  presignedUrl: string;
  expiresAt: Date;
}
```

### ICPUploadConfig

```typescript
export interface ICPUploadConfig {
  canisterId: string;
  network: "local" | "mainnet";
  capsuleId: string;
  sessionId: string;
  chunkSize: number;
  maxChunks: number;
}
```

### VercelBlobUploadConfig

```typescript
export interface VercelBlobUploadConfig {
  token: string;
  url: string;
  expiresAt: Date;
}
```

### ArweaveUploadConfig

```typescript
export interface ArweaveUploadConfig {
  wallet: string;
  gateway: string;
  tags: Record<string, string>;
}
```

### IPFSUploadConfig

```typescript
export interface IPFSUploadConfig {
  gateway: string;
  pinningService: string;
  apiKey: string;
}
```

## Type Guards

```typescript
export function isS3UploadResult(result: UploadResult): boolean {
  return result.storageBackend === "s3";
}

export function isICPUploadResult(result: UploadResult): boolean {
  return result.storageBackend === "icp";
}

export function isVercelBlobUploadResult(result: UploadResult): boolean {
  return result.storageBackend === "vercel-blob";
}
```
