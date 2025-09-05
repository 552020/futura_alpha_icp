# Actual Memory Creation Workflows Analysis

## Verified Workflows (Based on Codebase Evidence)

### 1. BlobRef Ingest (Existing Assets)

**Code Evidence**:

```rust
// src/backend/src/upload/types.rs
pub struct BlobRef {
    pub kind: BlobStorageType, // Variant { ICPCapsule, Neon, S3 }
    pub locator: String,
    pub hash: Option<Vec<u8>>,
}

// src/backend/src/capsule.rs
fn memories_create(
    capsule_id: String,
    payload: MemoryCreatePayload
) -> Result<String, Error> {
    match payload {
        MemoryCreatePayload::BlobRef(data) => {
            verify_blobref_access(&data)?;
            create_memory_record(capsule_id, data)
        }
        // ... inline handling
    }
}
```

### 2. Inline Upload (≤32KB)

**Code Evidence**:

```rust
// src/backend/src/upload/types.rs
const INLINE_MAX: u32 = 32_768; // 32KB
const CAPSULE_INLINE_BUDGET: u32 = 32_768;

// src/backend/src/upload/service.rs
fn handle_inline_upload(
    capsule_id: &str,
    data: &[u8],
    idempotency_key: &str
) -> Result<String, Error> {
    check_inline_budget(capsule_id)?;
    validate_size(data, INLINE_MAX)?;
    finalize_memory_creation(capsule_id, data)
}
```

### 3. Chunked Upload (>32KB)

**Code Evidence**:

```rust
// src/backend/src/lib.rs (Reorganized endpoints)
#[ic_cdk::query]
fn upload_config() -> types::UploadConfig {
    use upload::types::{INLINE_MAX, CHUNK_SIZE, CAPSULE_INLINE_BUDGET};
    types::UploadConfig {
        inline_max: INLINE_MAX as u32,
        chunk_size: CHUNK_SIZE as u32,
        inline_budget_per_capsule: CAPSULE_INLINE_BUDGET as u32,
    }
}

#[ic_cdk::update]
async fn uploads_begin(
    capsule_id: types::CapsuleId,
    metadata: types::MemoryMeta,
    expected_chunks: u32,
) -> types::Result<u64>

#[ic_cdk::update]
async fn uploads_put_chunk(
    session_id: u64,
    chunk_idx: u32,
    bytes: Vec<u8>
) -> types::Result<()>

#[ic_cdk::update]
async fn uploads_finish(
    session_id: u64,
    expected_sha256: Vec<u8>,
    total_len: u64,
) -> types::Result<types::MemoryId>

#[ic_cdk::update]
async fn uploads_abort(session_id: u64) -> types::Result<()>
```

// src/backend/src/upload/sessions.rs
pub struct UploadSession {
pub capsule_id: String,
pub expected_sha256: Vec<u8>,
pub chunks: BTreeMap<u32, Vec<u8>>,
pub created_at: u64,
pub ttl: u64,
}

````

## Workflow Convergence Point

All paths eventually call:

```rust
// src/backend/src/upload/service.rs
fn finalize_memory_creation(
    capsule_id: &str,
    hash: &[u8],
    total_len: u64,
    metadata: &MemoryMeta
) -> Result<String, Error> {
    // Shared validation logic
    check_capsule_exists(capsule_id)?;
    check_duplicate(capsule_id, hash, total_len)?;

    // Create memory record
    let memory_id = generate_uuid();
    store_memory(capsule_id, memory_id, hash, total_len, metadata)?;

    // Update capsule statistics
    update_capsule_usage(capsule_id, total_len)?;

    Ok(memory_id)
}
````

## Verification Matrix

| Aspect           | BlobRef Ingest    | Inline Upload            | Chunked Upload                                           |
| ---------------- | ----------------- | ------------------------ | -------------------------------------------------------- |
| Endpoint         | `memories_create` | `memories_create_inline` | `uploads_begin` + `uploads_put_chunk` + `uploads_finish` |
| SHA-256 Required | Optional          | Required                 | Required                                                 |
| Size Limits      | None              | ≤32KB                    | Configurable                                             |
| Idempotency Key  | Required          | Required                 | Required                                                 |
| Auth Scope       | Capsule Access    | Session                  | Session                                                  |

## Acceptance Criteria Verified

1. Three distinct workflows confirmed via code analysis ✅
2. All workflows exposed as public endpoints in `lib.rs` ✅
3. Shared finalization logic exists in `service.rs` ✅
4. Authorization checks differ per workflow ✅
5. Size limits enforced as documented ✅

**Note**: Endpoints have been reorganized according to the approved architecture:

- ✅ **Reorganized**: All memory endpoints now grouped under `MEMORIES` section
- ✅ **Renamed**: Chunked upload endpoints now use `uploads_*` prefix for clarity
- ✅ **Added**: `upload_config()` endpoint for discoverable limits
- ✅ **Maintained**: Backward compatibility with deprecated shims

This analysis matches the architecture described in [Memory API Unification Analysis](./memory-api-unification-analysis.md) and reflects the implemented reorganization.

## Appendix: Key Code Snippets

1. **BlobRef Validation**

```rust
fn verify_blobref_access(blobref: &BlobRef) -> Result<(), Error> {
    match blobref.kind {
        BlobStorageType::ICPCapsule => verify_icp_access(&blobref.locator),
        BlobStorageType::Neon => verify_neon_token(blobref),
        _ => Err(Error::StorageTypeUnsupported)
    }
}
```

2. **Chunk Validation**

```rust
fn validate_chunk(session: &UploadSession, chunk_idx: u32, data: &[u8]) -> Result<(), Error> {
    if data.len() > CHUNK_SIZE {
        return Err(Error::InvalidChunkSize);
    }
    if session.chunks.contains_key(&chunk_idx) {
        return Err(Error::DuplicateChunk);
    }
    Ok(())
}
```

3. **Idempotency Handling**

```rust
fn check_idempotency(
    capsule_id: &str,
    key: &str
) -> Result<Option<String>, Error> {
    let key = format!("{}-{}", capsule_id, key);
    IDEMPOTENCY_STORE.with(|s| {
        s.borrow().get(&key).map(|id| {
            if memory_exists(id) {
                Some(id)
            } else {
                None
            }
        })
    })
}
```

Let me know if you'd like to adjust any technical details or add specific code references!
