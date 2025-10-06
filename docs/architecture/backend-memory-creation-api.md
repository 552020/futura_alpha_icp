# Backend Memory Creation API Documentation

## Overview

This document describes the implemented memory creation API endpoints in the ICP backend canister. The API provides flexible memory creation with support for different asset types and storage backends.

**Status**: ✅ IMPLEMENTED  
**Implementation Date**: October 6, 2025  
**Key Commits**: 
- `4d9580e` - Complete upload service refactoring with multiple asset support
- `5606b60` - Memory creation architecture documentation  
- `5f9f4a6` - Enhance memories_create API with unified asset support
**Architecture**: Hybrid approach with unified + specialized endpoints  
**Location**: `src/backend/src/lib.rs` and `src/backend/src/memories/core/create.rs`

## Architecture Overview

The memory creation API follows a decoupled architecture pattern:

- **Core Business Logic**: Pure functions in `memories/core/create.rs` (testable, no ICP dependencies)
- **API Layer**: Public endpoints in `lib.rs` that delegate to core functions
- **Environment Adapters**: `CanisterEnv` and `StoreAdapter` bridge ICP-specific concerns
- **Type Safety**: Full TypeScript/Rust type safety with compile-time validation

### Key Features

- ✅ **Pure blob upload** - `uploads_finish` returns only `blob_id`
- ✅ **Memory extraction** - Upload flow no longer creates memories
- ✅ **Memory creation endpoints** - Both unified and specialized endpoints implemented
- ✅ **Idempotency** - Deterministic memory IDs prevent duplicate creation
- ✅ **ACL Integration** - Full access control with capsule permissions
- ✅ **Post-write Verification** - Ensures data persistence integrity

## API Endpoints

### Primary Endpoint: `memories_create`

**Function Signature**: `memories_create` - Single unified endpoint handling all asset types
**Location**: `src/backend/src/lib.rs:267-302`
**Type**: `#[ic_cdk::update]` (state-changing operation)

```rust
#[ic_cdk::update]
fn memories_create(
    capsule_id: types::CapsuleId,
    bytes: Option<Vec<u8>>,                    // Inline assets
    blob_ref: Option<types::BlobRef>,         // Internal blob assets
    external_location: Option<types::StorageEdgeBlobType>, // External storage type
    external_storage_key: Option<String>,      // External storage key
    external_url: Option<String>,              // External URL
    external_size: Option<u64>,                // External file size
    external_hash: Option<Vec<u8>>,            // External file hash
    asset_metadata: types::AssetMetadata,
    idem: String,
) -> types::Result20
```

### Specialized Endpoint: `memories_create_with_internal_blobs`

**Function Signature**: `memories_create_with_internal_blobs` - Optimized for ICP blob storage
**Location**: `src/backend/src/lib.rs:305-328`
**Type**: `#[ic_cdk::update]` (state-changing operation)

```rust
#[ic_cdk::update]
fn memories_create_with_internal_blobs(
    capsule_id: types::CapsuleId,
    memory_metadata: crate::memories::types::MemoryMetadata,
    internal_blob_assets: Vec<crate::memories::types::InternalBlobAssetInput>,
    idem: String,
) -> types::Result20
```

## Data Types

### Input Types

**Location**: `src/backend/src/memories/types.rs`

```rust
// Internal blob asset input (lines 417-420)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct InternalBlobAssetInput {
    pub blob_id: String,        // From uploads_finish (ICP blob storage)
    pub metadata: AssetMetadata,
}

// External blob asset input (lines 424-431)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct ExternalBlobAssetInput {
    pub location: StorageEdgeBlobType, // S3, Vercel, Arweave, IPFS, etc.
    pub storage_key: String,
    pub url: Option<String>,
    pub size: Option<u64>,
    pub hash: Option<Vec<u8>>,
    pub metadata: AssetMetadata,
}
```

### Return Types

```rust
// Result type for memory creation operations
pub enum Result20 {
    Ok(MemoryId),
    Err(Error),
}
```

### ❌ NOT CHOSEN: Separate Endpoints (Option B)

```rust
// 1. Create memory with inline assets (small files ≤32KB)
memories_create_inline(
    capsule_id: String,
    memory_metadata: MemoryMetadata,
    inline_assets: Vec<InlineAssetInput>,
    idem: String,
) -> Result<MemoryId, Error>

// 2. Create memory with internal blob assets (ICP blob storage)
memories_create_with_internal_blobs(
    capsule_id: String,
    memory_metadata: MemoryMetadata,
    internal_blob_assets: Vec<InternalBlobAssetInput>,
    idem: String,
) -> Result<MemoryId, Error>

// 3. Create memory with external blob assets (S3, Vercel, Arweave, IPFS, etc.)
memories_create_with_external_blobs(
    capsule_id: String,
    memory_metadata: MemoryMetadata,
    external_blob_assets: Vec<ExternalBlobAssetInput>,
    idem: String,
) -> Result<MemoryId, Error>
```

## Input Types

```rust
struct InlineAssetInput {
    bytes: Vec<u8>,
    metadata: AssetMetadata,
}

struct InternalBlobAssetInput {
    blob_id: String,        // From uploads_finish (ICP blob storage)
    metadata: AssetMetadata,
}

struct ExternalBlobAssetInput {
    location: StorageEdgeBlobType, // S3, Vercel, Arweave, IPFS, etc.
    storage_key: String,
    url: Option<String>,
    size: Option<u64>,
    hash: Option<Vec<u8>>,
    metadata: AssetMetadata,
}
```

## ✅ IMPLEMENTED: Benefits of Chosen Hybrid Approach

### ✅ **Unified Endpoint Benefits**

- **Single API**: One endpoint handles all asset types (inline, internal blob, external)
- **Flexible Parameters**: Optional parameters allow different asset types
- **Type Safety**: Compile-time validation with Option<T> parameters
- **Simplified Frontend**: One endpoint to learn and use

### ✅ **Specialized Endpoint Benefits**

- **ICP Optimization**: `memories_create_with_internal_blobs` optimized for ICP blob storage
- **Batch Operations**: Can create memories with multiple internal blob assets
- **Performance**: Direct path for common ICP blob use case
- **Type Safety**: Specific input types for internal blob assets

### ✅ **Hybrid Approach Benefits**

- **Best of Both**: Unified flexibility + specialized optimization
- **Backward Compatibility**: Can add more specialized endpoints as needed
- **Clear Migration Path**: Easy to understand and implement
- **Future Extensibility**: Can add more specialized endpoints without breaking changes

## ✅ IMPLEMENTED: Usage Examples

### 1. ✅ IMPLEMENTED: Simple Internal Blob Memory (ICP Storage)

```javascript
// ✅ IMPLEMENTED: After pure blob upload
const blobId = await backend.uploads_finish(sessionId, hash, size);

// ✅ IMPLEMENTED: Using specialized endpoint
const memoryId = await backend.memories_create_with_internal_blobs(
  capsuleId,
  {
    title: "My Photo",
    description: "A beautiful sunset",
    tags: ["photo", "sunset"],
  },
  [
    {
      blob_id: blobId.blob_id, // From ICP blob storage
      metadata: {
        name: "sunset.jpg",
        mime_type: "image/jpeg",
        size: 3623604,
      },
    },
  ],
  "unique-idempotency-key"
);

// ✅ IMPLEMENTED: Alternative using unified endpoint
const memoryId2 = await backend.memories_create(
  capsuleId,
  null, // no inline bytes
  { locator: blobId.blob_id, len: size, hash: Some(hash) }, // blob_ref
  null, // no external location
  null, // no external storage key
  null, // no external URL
  null, // no external size
  null, // no external hash
  {
    name: "sunset.jpg",
    mime_type: "image/jpeg",
    size: 3623604,
  },
  "unique-idempotency-key"
);
```

### 2. Separate Memories for Different Asset Types

```javascript
// Create separate memories for different asset types (recommended approach)

// 1. Inline memory for small thumbnail
const thumbnailMemoryId = await backend.memories_create_inline(
  capsuleId,
  { title: "Thumbnail", description: "Small version" },
  [
    {
      bytes: thumbnailBytes,
      metadata: { name: "thumb.jpg", mime_type: "image/jpeg" },
    },
  ],
  "thumbnail-idempotency-key"
);

// 2. Internal blob memory for large original (ICP storage)
const originalMemoryId = await backend.memories_create_with_internal_blobs(
  capsuleId,
  { title: "Original Photo", description: "Full resolution" },
  [
    {
      blob_id: originalBlobId, // From ICP blob storage
      metadata: { name: "original.jpg", mime_type: "image/jpeg" },
    },
  ],
  "original-idempotency-key"
);

// 3. External blob memory for processed version (S3 storage)
const processedMemoryId = await backend.memories_create_with_external_blobs(
  capsuleId,
  { title: "Processed Photo", description: "Filtered version" },
  [
    {
      location: "S3",
      storage_key: "processed/sunset.jpg",
      url: "https://s3.amazonaws.com/...",
      size: 2048000,
      metadata: { name: "processed.jpg", mime_type: "image/jpeg" },
    },
  ],
  "processed-idempotency-key"
);
```

## ✅ COMPLETED: Implementation Strategy

### ✅ COMPLETED: Phase 1: Core Endpoints

1. ✅ `memories_create` - Unified endpoint handling all asset types
2. ✅ `memories_create_with_internal_blobs` - Specialized ICP blob assets endpoint

### ✅ COMPLETED: Phase 2: External Support

3. ✅ External storage support via unified `memories_create` endpoint
4. ✅ All asset types supported through optional parameters

### ✅ COMPLETED: Phase 3: Optimization

5. ✅ Idempotency support with deterministic memory IDs
6. ✅ ACL integration for security
7. ✅ Post-write verification for data integrity

## ✅ COMPLETED: Migration Path

### ✅ IMPLEMENTED: Current → New

```rust
// ✅ IMPLEMENTED: OLD: uploads_finish created memory
let (blob_id, memory_id) = uploads_finish(session_id, hash, size);

// ✅ IMPLEMENTED: NEW: uploads_finish returns only blob_id
let blob_id = uploads_finish(session_id, hash, size);
let memory_id = memories_create_with_internal_blobs(capsule_id, metadata, [blob_id], idem);

// ✅ IMPLEMENTED: Alternative: Using unified endpoint
let memory_id = memories_create(capsule_id, None, Some(blob_ref), None, None, None, None, None, metadata, idem);
```

## ✅ IMPLEMENTATION SUMMARY

**Chosen Solution**: Hybrid approach with unified + specialized endpoints

- **Primary**: `memories_create` - Single endpoint for all asset types
- **Specialized**: `memories_create_with_internal_blobs` - Optimized for ICP blobs
- **Benefits**: Maximum flexibility + performance optimization
- **Status**: ✅ FULLY IMPLEMENTED AND DEPLOYED
