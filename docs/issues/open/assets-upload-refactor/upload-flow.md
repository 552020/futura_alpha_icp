# Upload Flow - Functions and Data Flow

## Overview

The system supports two upload strategies: **inline** (small files) and **chunked** (large files), with parallel processing for image derivatives.

## Upload Functions

### Frontend Functions

- `uploadFileToICP()` - Main entry point, routes to inline/chunked
- `uploadInlineToICP()` - Small files (< 2MB), single API call
- `uploadChunkedToICP()` - Large files, multi-step process
- `processImageDerivativesForICP()` - Parallel image processing

### Backend API Functions

- `uploads_begin()` - Start chunked upload session
- `uploads_put_chunk()` - Upload individual chunks
- `uploads_finish()` - Commit chunks and create memory
- `uploads_abort()` - Cancel upload session
- `memories_create()` - Direct inline memory creation

## Data Flow

### Inline Upload (Small Files)

```
File → uploadInlineToICP() → memories_create() → Memory with asset_id
```

**Functions:**

1. `uploadInlineToICP(file, actor, capsuleId, idem, onProgress)`
2. `actor.memories_create(capsuleId, [bytes], [], [], [title], [description], [], [], assetMetadata, idem)`

### Chunked Upload (Large Files)

```
File → uploadChunkedToICP() → uploads_begin() → uploads_put_chunk() → uploads_finish() → Memory with asset_id
```

**Functions:**

1. `uploadChunkedToICP(file, actor, capsuleId, idem, limits, onProgress)`
2. `actor.uploads_begin(capsuleId, assetMetadata, expectedChunks, idem)`
3. `actor.uploads_put_chunk(sessionId, chunkIndex, chunkBytes)` (repeat for each chunk)
4. `actor.uploads_finish(sessionId, sha256Hash, totalLength)`

### Parallel Processing (Images)

```
File → Lane A (Original) + Lane B (Derivatives) → Combined Memory
```

**Functions:**

- Lane A: `uploadInlineToICP()` or `uploadChunkedToICP()`
- Lane B: `processImageDerivativesForICP()` → `uploadProcessedAssetsToICP()`

## Key Data Structures

### Asset Metadata

```typescript
AssetMetadata = {
  Image: {
    base: {
      name: string,
      mime_type: string,
      bytes: bigint,
      // ... other fields
    },
  },
};
```

### Upload Result

```typescript
UploadResult = {
  memoryId: string,
  blobId: string,
  size: number,
  checksum_sha256: string,
};
```

## Upload Limits

- **Inline max**: 2MB (files ≤ 2MB use inline)
- **Chunk size**: 1MB per chunk
- **Max chunks**: 1000 chunks per file
- **Strategy**: File size determines inline vs chunked

## Session Management

- Each chunked upload gets a unique `sessionId`
- Sessions track: `capsuleId`, `caller`, `expectedChunks`, `assetMetadata`
- Rolling SHA256 hash calculated during chunk upload
- Sessions can be aborted for cleanup
