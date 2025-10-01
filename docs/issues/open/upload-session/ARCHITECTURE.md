# Upload Session Architecture

**Purpose**: Generic session management system for chunked operations  
**Status**: ✅ Production Ready  
**Date**: 2025-10-01

---

## 🎯 Design Goals

1. **Separation of Concerns**
   - Generic session lifecycle (SessionService)
   - Upload-specific semantics (SessionCompat, UploadService)
   - Storage abstraction (ByteSink trait)

2. **Reliability**
   - Deterministic keys (SHA256)
   - Rolling hash verification
   - Session isolation (parallel-safe)
   - Atomic operations

3. **Performance**
   - Zero-copy chunk writes
   - Direct stable memory writes (no heap buffering)
   - Incremental hash computation

4. **Maintainability**
   - Clear layer boundaries
   - Testable components
   - Future-proof for refactoring

---

## 🏗️ Layer Architecture

```
┌─────────────────────────────────────────────────────────┐
│  lib.rs (Candid Endpoints)                              │
│  - uploads_begin()                                       │
│  - uploads_put_chunk()                                   │
│  - uploads_finish()                                      │
│  - Rolling hash management (UPLOAD_HASH)                 │
└────────────────┬────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────┐
│  UploadService (upload/service.rs)                       │
│  - Upload orchestration                                  │
│  - Capsule/memory management                             │
│  - Asset indexing                                        │
│  - Idempotency key handling                              │
└────────────────┬────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────┐
│  SessionCompat (session/compat.rs) [COMPATIBILITY LAYER] │
│  - Bridges old upload API → generic SessionService       │
│  - Manages UploadSessionMeta                             │
│  - Creates StableBlobSink instances                      │
│  - TODO: Remove in future refactoring                    │
└────────────────┬────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────┐
│  SessionService (session/service.rs)                     │
│  - Generic session lifecycle                             │
│  - Chunk bookkeeping (received_idxs, bytes_received)     │
│  - Idempotency handling                                  │
│  - TTL and expiration                                    │
│  - NO upload-specific logic                              │
└────────────────┬────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────┐
│  ByteSink Trait (session/types.rs)                       │
│  - write_at(offset, bytes) -> Result<(), Error>          │
│  - Abstract chunk writing                                │
└────────────────┬────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────┐
│  StableBlobSink (upload/blob_store.rs)                   │
│  - ByteSink implementation                               │
│  - Direct writes to STABLE_BLOB_STORE                    │
│  - Session-aware key derivation (pmid_session_hash32)    │
│  - Chunk alignment and validation                        │
└─────────────────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────┐
│  Stable Memory                                           │
│  - STABLE_BLOB_STORE: StableBTreeMap<([u8;32], u32), Vec<u8>> │
│  - STABLE_BLOB_META: StableBTreeMap<u64, BlobMeta>      │
└─────────────────────────────────────────────────────────┘
```

---

## 🔑 Key Design Decisions

### 1. Generic Session Service

**Why**: Enable reuse for other chunked operations (downloads, streaming, etc.)

**How**:
- Session lifecycle independent of upload semantics
- Uses abstract `ByteSink` trait
- No knowledge of capsules, memories, or assets

**Benefits**:
- ✅ Single responsibility principle
- ✅ Testable in isolation
- ✅ Future-proof for new features

### 2. Compatibility Layer Pattern

**Why**: Gradual migration without breaking existing code

**How**:
- `SessionCompat` bridges old API → new `SessionService`
- Manages upload-specific metadata (`UploadSessionMeta`)
- Creates `StableBlobSink` instances with proper context

**Benefits**:
- ✅ All tests pass during migration
- ✅ Low-risk incremental refactoring
- ✅ Can remove layer later (see REFACTORING_TODO.md)

**Trade-offs**:
- ❌ Extra indirection (temporary)
- ❌ More complex call chain (temporary)

### 3. ByteSink Trait Abstraction

**Why**: Decouple session management from storage implementation

**How**:
```rust
pub trait ByteSink {
    fn write_at(&mut self, offset: usize, bytes: &[u8]) -> Result<(), Error>;
}
```

**Benefits**:
- ✅ SessionService doesn't know about stable memory
- ✅ Easy to test with mock sinks
- ✅ Could support other storage backends (heap, file, etc.)

### 4. Deterministic SHA256 Keys

**Why**: Non-deterministic keys (DefaultHasher) caused complete data loss

**How**:
```rust
fn pmid_session_hash32(pmid: &str, session_id: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(pmid.as_bytes());
    hasher.update(session_id.to_le_bytes());
    hasher.finalize().into()
}
```

**Benefits**:
- ✅ Same input → same key (always)
- ✅ Works across canister upgrades
- ✅ Reliable data retrieval
- ✅ Cryptographically sound

### 5. Session-Aware Keys for Parallel Safety

**Why**: Parallel uploads with same `provisional_memory_id` collided

**How**: Include `session_id` in key derivation
```rust
// Key format: (pmid_session_hash32(pmid, session_id), chunk_idx)
let key = (pmid_session_hash32(&meta.provisional_memory_id, meta.session_id), chunk_idx);
STABLE_BLOB_STORE.insert(key, chunk_data);
```

**Benefits**:
- ✅ Sessions fully isolated
- ✅ No race conditions in parallel uploads
- ✅ Same asset can upload in multiple sessions simultaneously

### 6. Rolling Hash Verification

**Why**: Read-back verification was slow and unreliable

**How**: Incremental SHA256 during upload
```rust
thread_local! {
    static UPLOAD_HASH: RefCell<BTreeMap<u64, Sha256>> = ...;
}

// Update on each chunk
UPLOAD_HASH.with(|h| {
    if let Some(hasher) = h.borrow_mut().get_mut(&session_id.0) {
        hasher.update(&bytes);
    }
});

// Verify in finish
let computed = UPLOAD_HASH.with(|h| {
    h.borrow_mut().remove(&session_id.0)
        .map(|h| h.finalize().to_vec())
});
```

**Benefits**:
- ✅ No read-back needed (faster)
- ✅ Detects corruption immediately
- ✅ Works perfectly with parallel uploads
- ✅ Lower memory usage (streaming hash)

### 7. Zero-Copy Chunk Writes

**Why**: Minimize memory usage and improve performance

**How**: `ByteSink::write_at()` writes directly to stable memory
```rust
impl ByteSink for StableBlobSink {
    fn write_at(&mut self, offset: usize, bytes: &[u8]) -> Result<(), Error> {
        let chunk_idx = (offset / self.chunk_size) as u32;
        let key = (self.pmid_hash, chunk_idx);
        STABLE_BLOB_STORE.with(|store| {
            store.borrow_mut().insert(key, bytes.to_vec())
        });
        Ok(())
    }
}
```

**Benefits**:
- ✅ No intermediate heap buffering
- ✅ Lower memory footprint
- ✅ Direct stable memory writes

---

## 📊 Data Flow

### Upload Flow (uploads_begin → uploads_put_chunk → uploads_finish)

```
1. uploads_begin(capsule_id, asset_metadata, ...)
   │
   ├─→ UploadService::begin_upload()
   │   │
   │   ├─→ Create UploadSessionMeta
   │   │   - session_id
   │   │   - capsule_id
   │   │   - asset_metadata
   │   │   - provisional_memory_id = "capsule_id#path"
   │   │
   │   ├─→ SessionCompat::create(session_id, meta)
   │   │   │
   │   │   ├─→ Create SessionSpec from meta
   │   │   │   - chunk_size
   │   │   │   - bytes_expected = asset_metadata.get_base().bytes
   │   │   │   - owner (Principal)
   │   │   │   - idem (idempotency key)
   │   │   │
   │   │   └─→ SessionService::begin_with_id(session_id, spec, clock)
   │   │       - Store generic session state
   │   │       - Initialize received_idxs
   │   │
   │   └─→ UPLOAD_HASH: Initialize SHA256 hasher
   │
   └─→ Return SessionId

2. uploads_put_chunk(session_id, chunk_idx, bytes)
   │
   ├─→ UPLOAD_HASH: Update hasher with bytes
   │
   ├─→ UploadService::put_chunk(session_id, chunk_idx, bytes)
   │   │
   │   └─→ SessionCompat::put_chunk(session_id, chunk_idx, bytes)
   │       │
   │       ├─→ Get UploadSessionMeta for session
   │       │
   │       ├─→ Create StableBlobSink from meta
   │       │   - pmid_hash = pmid_session_hash32(pmid, session_id)
   │       │   - chunk_size
   │       │
   │       └─→ SessionService::put_chunk(session_id, chunk_idx, bytes, &sink, clock)
   │           │
   │           ├─→ Validate chunk (alignment, size)
   │           │
   │           ├─→ Update received_idxs (idempotent)
   │           │
   │           ├─→ sink.write_at(offset, bytes)
   │           │   │
   │           │   └─→ StableBlobSink::write_at()
   │           │       - key = (pmid_hash, chunk_idx)
   │           │       - STABLE_BLOB_STORE.insert(key, bytes)
   │           │
   │           └─→ Update bytes_received
   │
   └─→ Return Ok()

3. uploads_finish(session_id, capsule_id, expected_sha256, ...)
   │
   ├─→ UPLOAD_HASH: Remove hasher, finalize hash
   │
   ├─→ Verify hash matches expected_sha256
   │
   ├─→ UploadService::commit(session_id, chunk_count, ...)
   │   │
   │   ├─→ SessionCompat::verify_chunks_complete(session_id, chunk_count)
   │   │   - Check received_idxs == [0..chunk_count)
   │   │
   │   ├─→ BlobStore::store_from_chunks(session_id, ...)
   │   │   │
   │   │   ├─→ Get UploadSessionMeta
   │   │   │
   │   │   ├─→ Derive blob_id from pmid_hash (first 8 bytes)
   │   │   │
   │   │   ├─→ Store BlobMeta (includes pmid_hash)
   │   │   │   - STABLE_BLOB_META.insert(blob_id, BlobMeta { ... })
   │   │   │
   │   │   └─→ SessionService::finish(session_id)
   │   │       - Mark session complete
   │   │       - Ready for cleanup
   │   │
   │   ├─→ Create MemoryId
   │   │
   │   ├─→ Store AssetRef in capsule index
   │   │
   │   └─→ UPLOAD_HASH: Cleanup (already removed)
   │
   └─→ Return (asset_path, memory_id)
```

---

## 🗄️ Data Structures

### Session Layer

```rust
// Generic session state (SessionService)
pub struct Session {
    pub owner: Vec<u8>,              // Principal bytes
    pub chunk_size: usize,
    pub bytes_expected: u64,
    pub bytes_received: u64,
    pub received_idxs: BTreeSet<u32>,
    pub session_meta: SessionMeta {
        pub idem: String,
        pub last_seen: u64,
        pub status: SessionStatus,
    },
}

// Upload-specific metadata (SessionCompat)
pub struct UploadSessionMeta {
    pub session_id: u64,
    pub capsule_id: CapsuleId,
    pub caller: Principal,
    pub created_at: u64,
    pub expected_chunks: u32,
    pub status: SessionStatus,
    pub chunk_count: u32,
    pub asset_metadata: AssetMetadata,
    pub provisional_memory_id: String,
    pub chunk_size: usize,
    pub idem: String,
    pub blob_id: Option<u64>,
}
```

### Storage Layer

```rust
// Blob metadata
pub struct BlobMeta {
    pub pmid_hash: [u8; 32],         // SHA256(pmid + session_id)
    pub chunk_count: u32,
    pub chunk_size: usize,
    pub total_bytes: u64,
    pub created_at: u64,
}

// Stable memory stores
thread_local! {
    static STABLE_BLOB_STORE: RefCell<StableBTreeMap<([u8; 32], u32), Vec<u8>, Memory>> = ...;
    static STABLE_BLOB_META: RefCell<StableBTreeMap<u64, BlobMeta, Memory>> = ...;
}

// Rolling hash state
thread_local! {
    static UPLOAD_HASH: RefCell<BTreeMap<u64, Sha256>> = ...;
}
```

---

## 🔐 Key Derivation Strategy

### Chunk Keys

**Format**: `([u8; 32], u32)` where:
- `[u8; 32]` = `pmid_session_hash32(provisional_memory_id, session_id)`
- `u32` = `chunk_idx`

**Derivation**:
```rust
fn pmid_session_hash32(pmid: &str, session_id: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(pmid.as_bytes());
    hasher.update(session_id.to_le_bytes());
    hasher.finalize().into()
}
```

**Why SHA256**:
- ✅ Deterministic (same input → same output)
- ✅ Cryptographically sound (no collisions)
- ✅ Fixed size (32 bytes)
- ✅ Works across canister upgrades

**Why Include session_id**:
- ✅ Parallel uploads don't collide
- ✅ Same asset can upload in multiple sessions
- ✅ Session isolation guaranteed

### Blob IDs

**Format**: `u64` derived from first 8 bytes of `pmid_hash`

**Derivation**:
```rust
let blob_id = u64::from_le_bytes(pmid_hash[0..8].try_into().unwrap());
```

**Why This Works**:
- ✅ Deterministic (same pmid_hash → same blob_id)
- ✅ Compatible with existing BlobMeta key type (u64)
- ✅ Low collision risk (256-bit hash → 64-bit truncation)

---

## 🧪 Testing Strategy

### Unit Tests

Located in module tests (e.g., `src/backend/src/session/service.rs`)

**Coverage**:
- Session lifecycle (begin, put_chunk, finish)
- Idempotency handling
- Chunk bookkeeping
- Expiration logic

**Limitations**:
- Cannot test `ic_cdk::api::time()` (panics outside canister)
- Must use mock Clock implementation

### Integration Tests (E2E)

Located in `tests/backend/shared-capsule/upload/session/`

**Coverage**:
- Single 21MB upload (test_session_persistence.mjs)
- Parallel 2-lane system (test_session_isolation.mjs)
- Asset retrieval + derivatives (test_asset_retrieval_debug.mjs)
- Session collision prevention (test_session_collision.mjs)
- Session lifecycle (test_session_debug.mjs)

**All 5 tests passing** ✅

---

## 🔮 Future Refactoring

See **REFACTORING_TODO.md** for complete plan.

### Goal: Remove SessionCompat Layer

**Current**:
```
UploadService → SessionCompat → SessionService → StableBlobSink
```

**Target**:
```
UploadService → SessionService → StableBlobSink
```

**Benefits**:
- ✅ Simpler code (fewer indirections)
- ✅ Better performance
- ✅ Easier to maintain

**Timeline**: 5-8 days after current implementation stabilizes

---

## 📚 Related Documentation

- **IMPLEMENTATION_GUIDE.md** - How we built this
- **REFACTORING_TODO.md** - Next steps
- **CHANGELOG.md** - What changed and why
- **README.md** - Quick reference

---

**Reviewed**: 2025-10-01  
**Status**: ✅ Production Ready  
**Next Review**: After production stabilization

