# Memory Creation API Design

## Current State

- ✅ **Pure blob upload** - `uploads_finish` returns only `blob_id`
- ✅ **Memory extraction** - Upload flow no longer creates memories
- 🎯 **Next**: Create memory creation endpoints

## Proposed API Signatures

### Option A: Single Endpoint with Multiple Asset Types

```rust
// Single endpoint that handles all asset types
memories_create_with_assets(
    capsule_id: String,
    memory_metadata: MemoryMetadata,
    assets: Vec<MemoryAssetInput>,
    idem: String,
) -> Result<MemoryId, Error>

// Asset input types
enum MemoryAssetInput {
    Inline {
        bytes: Vec<u8>,
        metadata: AssetMetadata,
    },
    BlobInternal {
        blob_id: String,
        metadata: AssetMetadata,
    },
    BlobExternal {
        location: StorageEdgeBlobType,
        storage_key: String,
        url: Option<String>,
        size: Option<u64>,
        hash: Option<Vec<u8>>,
        metadata: AssetMetadata,
    },
}
```

### Option B: Separate Endpoints (Recommended)

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

## Benefits of Option B (Separate Endpoints)

### ✅ **Clear Separation of Concerns**

- Each endpoint handles one asset type
- Easier to understand and use
- Better error handling per asset type

### ✅ **Performance Optimization**

- Inline endpoint: Fast for small files (≤32KB)
- Internal blob endpoint: Optimized for ICP blob storage
- External blob endpoint: Handles external storage complexity (S3, Vercel, etc.)

### ✅ **Type Safety**

- Each endpoint has specific input types
- Compile-time validation of asset types
- No runtime asset type confusion

### ✅ **Frontend Flexibility**

- Frontend can choose appropriate endpoint
- Easy to implement different upload flows
- Clear API contracts

## Usage Examples

### 1. Simple Internal Blob Memory (ICP Storage)

```javascript
// After pure blob upload
const blobId = await backend.uploads_finish(sessionId, hash, size);
const memoryId = await backend.memories_create_with_internal_blobs(
  capsuleId,
  {
    title: "My Photo",
    description: "A beautiful sunset",
    tags: ["photo", "sunset"],
  },
  [
    {
      blob_id: blobId.blob_id,  // From ICP blob storage
      metadata: {
        name: "sunset.jpg",
        mime_type: "image/jpeg",
        size: 3623604,
      },
    },
  ],
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
      blob_id: originalBlobId,  // From ICP blob storage
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

## Implementation Strategy

### Phase 1: Core Endpoints

1. `memories_create_with_blobs` - ICP blob assets
2. `memories_create_inline` - Inline assets

### Phase 2: External Support

3. `memories_create_with_external_blobs` - External storage
4. `memories_create_mixed` - All asset types

### Phase 3: Optimization

5. Batch operations
6. Asset linking/updating
7. Performance improvements

## Migration Path

### Current → New

```rust
// OLD: uploads_finish created memory
let (blob_id, memory_id) = uploads_finish(session_id, hash, size);

// NEW: uploads_finish returns only blob_id
let blob_id = uploads_finish(session_id, hash, size);
let memory_id = memories_create_with_blobs(capsule_id, metadata, [blob_id], idem);
```

This design provides maximum flexibility while maintaining clear separation of concerns and excellent performance characteristics.
