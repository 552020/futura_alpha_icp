# Separate Upload and Memory Management Endpoints

**Status:** Open Issue  
**Priority:** High  
**Created:** January 2025  
**Type:** Architecture Improvement

## Problem Description

The current `memories_create` endpoint has mixed responsibilities, handling both blob storage and memory metadata creation in a single operation. This creates several architectural issues:

1. **Mixed Responsibilities**: Upload logic is coupled with memory management
2. **No Frontend Control**: Frontend cannot choose optimal upload strategy based on file size
3. **Redundant Endpoints**: Both `memories_create` and `uploads_*` endpoints exist with overlapping functionality
4. **Poor Error Handling**: Upload failures affect memory metadata creation
5. **Limited Flexibility**: Cannot implement proper retry logic or progress tracking

## Current Architecture

### Endpoints in `src/backend/src/lib.rs`

#### Memory Management Endpoints:

```rust
// Core memory CRUD operations
#[ic_cdk::update] async fn memories_create(capsule_id: CapsuleId, memory_data: MemoryData, idem: String) -> Result<MemoryId>
#[ic_cdk::query] fn memories_read(memory_id: String) -> Result<Memory>
#[ic_cdk::update] async fn memories_update(memory_id: String, updates: MemoryUpdateData) -> MemoryOperationResponse
#[ic_cdk::update] async fn memories_delete(memory_id: String) -> MemoryOperationResponse
#[ic_cdk::query] fn memories_list(capsule_id: String) -> MemoryListResponse
#[ic_cdk::query] fn memories_ping(memory_ids: Vec<String>) -> Result<Vec<MemoryPresenceResult>>
```

#### Upload Endpoints:

```rust
// Upload configuration
#[ic_cdk::query] fn upload_config() -> UploadConfig

// Chunked upload workflow
#[ic_cdk::update] fn uploads_begin(capsule_id: CapsuleId, meta: MemoryMeta, expected_chunks: u32, idem: String) -> Result<SessionId>
#[ic_cdk::update] async fn uploads_put_chunk(session_id: u64, chunk_idx: u32, bytes: Vec<u8>) -> Result<()>
#[ic_cdk::update] async fn uploads_finish(session_id: u64, expected_sha256: Vec<u8>, total_len: u64) -> Result<MemoryId>
#[ic_cdk::update] async fn uploads_abort(session_id: u64) -> Result<()>
```

### Related Implementation Files

#### Memory Management:

- **`src/backend/src/memories.rs`**: Core memory CRUD operations
  - `create()`: Handles both `MemoryData::Inline` and `MemoryData::BlobRef`
  - `read()`, `update()`, `delete()`, `list()`: Standard CRUD operations
  - `create_memory_object()`: Memory object construction
  - `find_existing_memory_by_content_in_capsule()`: Idempotency logic

#### Upload System:

- **`src/backend/src/upload/service.rs`**: Upload service implementation

  - `UploadService::new()`: Service initialization
  - `create_inline()`: Inline upload for small files
  - `begin_upload()`: Start chunked upload session
  - `put_chunk()`: Upload individual chunks
  - `commit()`: Finish upload and create memory record
  - `abort()`: Cancel upload session

- **`src/backend/src/upload/sessions.rs`**: Session management

  - `SessionStore`: Manages upload sessions
  - `SessionMeta`: Session metadata structure
  - `SessionStatus`: Session state tracking

- **`src/backend/src/upload/blob_store.rs`**: Blob storage

  - `BlobStore`: Manages blob storage and retrieval
  - `put_inline()`: Store small blobs inline
  - `store_from_chunks()`: Store large blobs from chunks

- **`src/backend/src/upload/types.rs`**: Upload type definitions
  - `SessionId`, `BlobRef`, `MemoryMeta`: Core types
  - `INLINE_MAX`, `CHUNK_SIZE`, `CAPSULE_INLINE_BUDGET`: Constants

#### Data Types:

- **`src/backend/src/types.rs`**: Core type definitions
  - `Memory`: Complete memory structure
  - `MemoryData`: Enum for inline vs blob reference
  - `MemoryMeta`: Memory metadata
  - `BlobRef`: Blob storage reference
  - `MemoryInfo`, `MemoryMetadata`: Memory information structures

#### Storage:

- **`src/backend/src/capsule_store/stable.rs`**: Capsule storage implementation

  - `CapsuleStore`: Stable storage for capsules and memories
  - `update()`, `get()`: Storage operations

- **`src/backend/src/memory.rs`**: Memory management infrastructure
  - `MM`: Global memory manager
  - Memory ID constants and management

## Current Architecture Issues

### Current Flow:

```
Frontend → memories_create(MemoryData) → Backend creates Memory record immediately
```

### Problems:

- **Single Responsibility Violation**: One endpoint does too much
- **No Size-Based Optimization**: Frontend can't choose between inline vs chunked upload
- **Poor Error Boundaries**: Upload and metadata errors are mixed
- **Limited Retry Logic**: Cannot retry uploads without losing metadata

## Proposed Solution

### Separated Endpoint Architecture

#### 1. Upload Endpoints (Blob Storage Only)

```rust
// For small files (≤32KB) - direct blob creation
uploads_create_inline(bytes: Vec<u8>, meta: MemoryMeta) -> BlobRef

// For large files (>32KB) - chunked upload workflow
uploads_begin(capsule_id: CapsuleId, meta: MemoryMeta, expected_chunks: u32, idem: String) -> SessionId
uploads_put_chunk(session_id: SessionId, chunk_idx: u32, bytes: Vec<u8>) -> Result<()>
uploads_finish(session_id: SessionId, expected_sha256: Vec<u8>, total_len: u64) -> BlobRef
uploads_abort(session_id: SessionId) -> Result<()>
```

#### 2. Memory Management Endpoints (Metadata Only)

```rust
// Create memory record with existing blob reference
memories_create_from_blob(capsule_id: CapsuleId, blob_ref: BlobRef, meta: MemoryMeta, idem: String) -> MemoryId

// Direct inline creation (for very small files - convenience method)
memories_create_inline(capsule_id: CapsuleId, bytes: Vec<u8>, meta: MemoryMeta, idem: String) -> MemoryId

// Standard CRUD operations (unchanged)
memories_read(memory_id: String) -> Result<Memory>
memories_update(memory_id: String, updates: MemoryUpdateData) -> MemoryOperationResponse
memories_delete(memory_id: String) -> MemoryOperationResponse
memories_list(capsule_id: String) -> MemoryListResponse
```

### Frontend Decision Logic

```typescript
async function uploadMemory(file: File, capsuleId: string, meta: MemoryMeta) {
  const idem = generateIdempotencyKey();

  if (file.size <= 32_000) {
    // 32KB threshold
    // Option 1: Direct inline creation (simplest, 1 call)
    return await memories_create_inline(capsuleId, fileBytes, meta, idem);

    // Option 2: Upload first, then create memory (more control, 2 calls)
    // const blobRef = await uploads_create_inline(fileBytes, meta);
    // return await memories_create_from_blob(capsuleId, blobRef, meta, idem);
  } else {
    // Large file: chunked upload workflow
    const expectedChunks = Math.ceil(file.size / CHUNK_SIZE);
    const sessionId = await uploads_begin(capsuleId, meta, expectedChunks, idem);

    // Upload chunks with progress tracking
    await uploadChunksWithProgress(sessionId, file);

    // Finish upload and get blob reference
    const expectedHash = await computeFileHash(file);
    const blobRef = await uploads_finish(sessionId, expectedHash, file.size);

    // Create memory record
    return await memories_create_from_blob(capsuleId, blobRef, meta, idem);
  }
}
```

## Benefits

### 1. **Clear Separation of Concerns**

- Upload endpoints = blob storage only
- Memory endpoints = metadata management only
- Single responsibility principle

### 2. **Frontend Flexibility**

- Choose optimal upload strategy based on file size
- Implement custom retry logic
- Show upload progress for large files
- Batch operations as needed

### 3. **Better Error Handling**

- Upload failures don't affect memory metadata
- Can retry uploads without losing metadata
- Clear error boundaries and recovery paths

### 4. **Performance Optimization**

- Small files: direct inline creation (1 call)
- Large files: chunked upload + metadata creation (N+1 calls)
- Frontend can optimize based on use case

### 5. **Atomicity Where Needed**

- Upload operations are atomic within their scope
- Memory creation is atomic
- Frontend controls the overall transaction

## Implementation Plan

### Phase 1: Add New Endpoints (Backward Compatible)

- [ ] Add `uploads_create_inline` endpoint
- [ ] Add `memories_create_from_blob` endpoint
- [ ] Add `memories_create_inline` convenience endpoint
- [ ] Keep existing `memories_create` for backward compatibility

### Phase 2: Update Frontend

- [ ] Implement size-based upload strategy selection
- [ ] Add progress tracking for chunked uploads
- [ ] Implement proper error handling and retry logic
- [ ] Add upload cancellation support

### Phase 3: Testing and Validation

- [ ] Test all upload scenarios (small, medium, large files)
- [ ] Test error scenarios and recovery
- [ ] Performance testing for different file sizes
- [ ] Integration testing with existing memory CRUD operations

### Phase 4: Migration and Cleanup

- [ ] Update all frontend code to use new endpoints
- [ ] Deprecate old `memories_create` endpoint
- [ ] Remove deprecated code after migration period
- [ ] Update documentation

## Technical Considerations

### File Size Thresholds

- **Inline**: ≤32KB (current `INLINE_MAX`)
- **Chunked**: >32KB (current chunked upload system)
- **Threshold**: Configurable via `upload_config()` endpoint

### Idempotency

- All endpoints support idempotency keys
- Upload sessions are idempotent
- Memory creation is idempotent
- Frontend controls idempotency across the workflow

### Error Recovery

- Upload failures: can retry without affecting metadata
- Memory creation failures: can retry with existing blob
- Session cleanup: automatic cleanup on abort/timeout

### Performance Metrics

- **Small files (≤32KB)**: 1 call, ~100ms
- **Large files (>32KB)**: N+1 calls, progress tracking
- **Chunked upload**: 0.02 MB/s (current mainnet performance)

## Related Issues

- [Memory CRUD Tests](memory_crud_tests.md) - Current test failures related to mixed responsibilities
- [Upload Performance](upload_performance.md) - Chunked upload performance on mainnet
- [Blob Store Architecture](blob_store_architecture.md) - Underlying storage system

## Success Criteria

- [ ] Frontend can choose optimal upload strategy based on file size
- [ ] Upload failures don't affect memory metadata
- [ ] Clear separation between blob storage and memory management
- [ ] Backward compatibility maintained during migration
- [ ] Performance improved for different file sizes
- [ ] Better error handling and recovery paths

## Notes

This architectural change aligns with the principle of separation of concerns and provides the frontend with the flexibility to implement optimal upload strategies. The current mixed-responsibility approach in `memories_create` is a common anti-pattern that should be addressed for better maintainability and user experience.

The proposed separation also enables better testing, as upload logic and memory management can be tested independently, and provides clearer error boundaries for debugging and user feedback.
