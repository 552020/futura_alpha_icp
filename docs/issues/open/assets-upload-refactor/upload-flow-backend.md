# Backend Upload Flow - Deep Dive

## Overview

The backend implements a robust chunked upload system with three core functions: `uploads_begin`, `uploads_put_chunk`, and `uploads_finish`. This system provides crash-safe, idempotent uploads with integrity verification.

## Core Upload Functions

### 1. `uploads_begin()` - Session Initialization

**Signature**:

```rust
async fn uploads_begin(
    capsule_id: String,
    asset_metadata: AssetMetadata,
    expected_chunks: u32,
    idem: String,
) -> Result<u64, Error>
```

**API Wrapper**: `src/backend/src/lib.rs` (lines 472-494)
**Implementation**: `src/backend/src/upload/service.rs` (lines 45-128) - `UploadService::begin_upload()`

**Purpose**: Creates a new upload session and validates permissions.

**Internal Process**:

1. **Input Validation**

   - Validates `expected_chunks > 0` and `< 16,384` (max limit)
   - Checks caller authentication via `ic_cdk::api::msg_caller()`

2. **Authorization Check**

   - Verifies caller has write access to the capsule
   - Returns `Error::Unauthorized` if access denied
   - Returns `Error::NotFound` if capsule doesn't exist

3. **Idempotency Check**

   - Searches for existing pending session with same `(capsule_id, caller, idem)`
   - Returns existing session ID if found (prevents duplicate sessions)

4. **Session Cleanup**

   - Removes expired sessions (older than 2 hours)
   - Prevents session accumulation

5. **Rate Limiting**

   - Enforces max 100 active sessions per caller/capsule
   - Returns `Error::ResourceExhausted` if limit exceeded

6. **Session Creation**
   - Generates unique `SessionId` and `MemoryId`
   - Creates `UploadSessionMeta` with:
     - `session_id`, `capsule_id`, `caller`
     - `expected_chunks`, `asset_metadata`
     - `provisional_memory_id` (for chunk storage keys)
     - `status: Pending`, `blob_id: None`

**Returns**: `SessionId` for subsequent chunk uploads

---

### 2. `uploads_put_chunk()` - Chunk Storage

**Signature**:

```rust
async fn uploads_put_chunk(
    session_id: u64,
    chunk_idx: u32,
    bytes: Vec<u8>,
) -> Result<(), Error>
```

**API Wrapper**: `src/backend/src/lib.rs` (lines 497-540)
**Implementation**: `src/backend/src/upload/service.rs` (lines 143-200) - `UploadService::put_chunk()`

**Purpose**: Stores individual file chunks with validation and integrity checks.

**Internal Process**:

1. **Session Validation**

   - Verifies session exists and caller matches
   - Ensures session is in `Pending` state (not committed)
   - Validates `chunk_idx < session.chunk_count`

2. **Chunk Size Validation**

   - Enforces max chunk size (1MB except last chunk)
   - Validates chunk index is within expected range

3. **Chunk Storage**

   - Stores chunk in stable storage using deterministic keys
   - Key format: `(pmid_hash, chunk_idx)` where `pmid_hash = SHA256(provisional_memory_id + session_id)`
   - Enables parallel uploads without key collisions

4. **Rolling Hash Update**

   - Updates SHA256 hash incrementally for each chunk
   - Enables integrity verification at commit time

5. **Idempotent Behavior**
   - Duplicate chunk uploads overwrite silently
   - Enables safe retry on network failures

**Storage Details**:

```rust
// Chunk storage key generation
fn pmid_session_hash32(pmid: &str, session_id: u64) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(pmid.as_bytes());
    h.update(b"#"); // Separator
    h.update(&session_id.to_le_bytes());
    h.finalize().into()
}

// Store chunk with deterministic key
STABLE_BLOB_STORE.insert((pmid_hash, chunk_idx), chunk_bytes);
```

---

### 3. `uploads_finish()` - Commit and Memory Creation

**Signature**:

```rust
async fn uploads_finish(
    session_id: u64,
    expected_sha256: Vec<u8>,
    total_len: u64,
) -> Result<UploadFinishResult, Error>
```

**API Wrapper**: `src/backend/src/lib.rs` (lines 543-628)
**Implementation**: `src/backend/src/upload/service.rs` (lines 210-319) - `UploadService::commit()`

**Purpose**: Commits all chunks, verifies integrity, and creates the final memory.

**Internal Process**:

1. **Session State Check**

   - Verifies caller matches session creator
   - Handles idempotent retry for already-committed sessions
   - Returns existing result if already attached to capsule

2. **Input Validation**

   - Validates `total_len` is within expected bounds
   - Ensures `0 < total_len <= (chunk_count * CHUNK_SIZE)`

3. **Chunk Integrity Verification**

   - Verifies all expected chunks are present
   - Returns `Error::InvalidArgument` if any chunks missing

4. **Hash Verification**

   - Streams all chunks to blob store with SHA256 verification
   - Compares computed hash with `expected_sha256`
   - Returns `Error::InvalidArgument` if hash mismatch

5. **Blob Storage**

   - Creates final blob from all chunks in stable storage
   - Generates unique `BlobId` for the complete file
   - Stores blob metadata for future retrieval

6. **Session State Update**

   - Marks session as `Committed` with completion timestamp
   - Stores `blob_id` in session metadata
   - Creates crash-safe checkpoint

7. **Memory Creation**

   - Creates `Memory` object with blob reference
   - Includes asset metadata and file information
   - Generates unique `MemoryId`

8. **Capsule Attachment**

   - Atomically adds memory to capsule's memory collection
   - Updates capsule's `updated_at` timestamp
   - Ensures data consistency

9. **Cleanup**
   - Removes session and temporary chunks
   - Frees up storage space

**Returns**: `(blob_id, memory_id)` for the created memory

---

## Data Structures

### UploadSessionMeta

```rust
pub struct UploadSessionMeta {
    pub session_id: u64,
    pub capsule_id: String,
    pub caller: Principal,
    pub created_at: u64,
    pub expected_chunks: u32,
    pub status: SessionStatus,
    pub chunk_count: u32,
    pub asset_metadata: AssetMetadata,
    pub provisional_memory_id: String,
    pub chunk_size: u32,
    pub idem: String,
    pub blob_id: Option<u64>,
}
```

### SessionStatus

```rust
pub enum SessionStatus {
    Pending,
    Committed { completed_at: u64 },
    Aborted { aborted_at: u64 },
}
```

### UploadFinishResult

```rust
pub struct UploadFinishResult {
    pub memory_id: String,
    pub blob_id: String,
    pub remote_id: Option<String>,
    pub size: u64,
    pub checksum_sha256: Option<[u8; 32]>,
    pub storage_backend: StorageBackend,
    pub storage_location: String,
    pub uploaded_at: u64,
    pub expires_at: Option<u64>,
}
```

## Storage Architecture

### Chunk Storage

- **Key Format**: `(pmid_hash, chunk_idx)` where `pmid_hash = SHA256(provisional_memory_id + session_id)`
- **Storage**: Stable BTreeMap for crash-safe persistence
- **Parallel Safety**: Deterministic keys prevent collisions during parallel uploads

### Blob Storage

- **Final Storage**: Complete file stored as blob with unique `BlobId`
- **Metadata**: Blob metadata stored separately for retrieval
- **Integrity**: SHA256 hash verification ensures data integrity

### Session Management

- **Thread-Local Storage**: Sessions stored in thread-local RefCell
- **Cleanup**: Automatic cleanup of expired sessions (2-hour TTL)
- **Rate Limiting**: Max 100 active sessions per caller/capsule

## Error Handling

### Common Errors

- `Error::Unauthorized` - Caller lacks write access to capsule
- `Error::NotFound` - Session or capsule doesn't exist
- `Error::InvalidArgument` - Invalid chunk size, index, or total length
- `Error::ResourceExhausted` - Too many active sessions

### Crash Recovery

- **Idempotent Retry**: Duplicate commits return existing result
- **Session State**: Committed sessions can be safely retried
- **Chunk Integrity**: Missing chunks cause commit failure
- **Hash Verification**: Mismatched hashes prevent data corruption

## Performance Characteristics

### Chunk Size

- **Default**: 1MB per chunk (configurable via `CHUNK_SIZE`)
- **Max Chunks**: 16,384 chunks per file
- **Max File Size**: ~16GB (16,384 × 1MB)

### Storage Efficiency

- **Chunk Storage**: Temporary storage during upload
- **Blob Storage**: Final persistent storage
- **Cleanup**: Automatic cleanup after successful commit

### Parallel Uploads

- **Session Isolation**: Each session has unique keys
- **Concurrent Safety**: Deterministic key generation prevents collisions
- **Rate Limiting**: Prevents resource exhaustion

## Security Features

### Authentication

- **Caller Verification**: Only session creator can upload chunks
- **Capsule Access**: Write access required for uploads
- **Session Ownership**: Sessions tied to specific caller

### Integrity

- **Hash Verification**: SHA256 verification at commit time
- **Chunk Validation**: All chunks must be present
- **Size Validation**: Total length must match expected size

### Rate Limiting

- **Session Limits**: Max 100 active sessions per caller/capsule
- **Chunk Limits**: Max 16,384 chunks per file
- **Size Limits**: Configurable chunk and file size limits
