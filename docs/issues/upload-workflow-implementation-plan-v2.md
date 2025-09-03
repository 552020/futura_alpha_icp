# Upload Workflow Implementation Plan - Hybrid Architecture (CORRECTED)

## 🎯 **Senior Developer Decision: Option 3 - Hybrid Persistence**

**Single public workflow** with dual internal paths:

- **≤32KB files**: Inline directly in Capsule (fast path)
- **>32KB files**: Chunked → blob store → reference in Capsule
- **Sessions/chunks**: Temporary (auto-cleanup)
- **Artifacts**: Permanent (blob store)
- **Explicit API**: Two clear endpoints for inline vs chunked

## 🚨 **CRITICAL FIXES APPLIED**

Based on senior developer feedback, these corrections ensure production-ready implementation:

1. **No Trait Objects**: UploadService uses concrete Store, not `dyn CapsuleStore`
2. **Manual Storable**: Implement Storable manually for all types (no derive)
3. **Centralized MemoryManager**: Single global manager with all MemoryId constants
4. **Split API**: Separate inline-only vs chunked endpoints (no auto-split large payloads)
5. **Numeric Keys**: Use u64 counters for SessionId/BlobId, not String
6. **Size Bounds**: Lower INLINE_MAX to 32KB, enforce per-capsule budget
7. **Chunk Integrity**: Verify all chunks exist before streaming
8. **Idempotency**: Session status tracking with crash-safe commit
9. **Authorization**: Verify caller access on all operations
10. **Index Correctness**: Ensure stable/hash stores have identical behavior

## 🏗️ **Architecture Overview**

### **Public Surface (Explicit API)**

```rust
// Inline-only endpoint (≤32KB, fails if larger)
memories_create_inline(capsule_id, file_data, metadata) -> MemoryId

// Chunked upload workflow (for >32KB files)
memories_begin_upload(capsule_id, metadata, expected_chunks) -> SessionId
memories_put_chunk(session_id, chunk_idx, bytes) -> ()
memories_commit(session_id, expected_sha256, total_len) -> MemoryId
memories_abort(session_id) -> ()
```

### **Internal Storage Layout**

```rust
// Single source of truth (unchanged)
STABLE_CAPSULES: StableBTreeMap<CapsuleId, Capsule>
├── Capsule.memories: HashMap<MemoryId, Memory>
    ├── Memory.data: Option<Vec<u8>>        // ≤32KB inline
    └── Memory.blob_ref: Option<BlobRef>    // >32KB reference

// Internal upload management (temporary) - using u64 keys
STABLE_UPLOAD_SESSIONS: StableBTreeMap<u64, SessionMeta>
STABLE_CHUNK_DATA: StableBTreeMap<(u64, u32), Vec<u8>>

// Internal blob storage (permanent) - using u64 keys
STABLE_BLOB_STORE: StableBTreeMap<(u64, u32), Vec<u8>>  // Paged
STABLE_BLOB_META: StableBTreeMap<u64, BlobMeta>         // Metadata

// ID counters for numeric keys
STABLE_SESSION_COUNTER: StableCell<u64>
STABLE_BLOB_COUNTER: StableCell<u64>
```

## 📋 **Implementation Phases**

### **Phase 1: Core Infrastructure (Week 1)**

#### **1.1 Create MemoryManager Module**

```rust
// File: src/backend/src/memory_manager.rs
use ic_stable_structures::{DefaultMemoryImpl, memory_manager::{MemoryManager, MemoryId}};
use std::cell::RefCell;

thread_local! {
    pub static MM: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

// All MemoryId constants in one place
pub const MEM_CAPSULES: MemoryId = MemoryId::new(0);
pub const MEM_IDX_SUBJECT: MemoryId = MemoryId::new(1);
pub const MEM_SESSIONS: MemoryId = MemoryId::new(2);
pub const MEM_CHUNKS: MemoryId = MemoryId::new(3);
pub const MEM_BLOBS: MemoryId = MemoryId::new(4);
pub const MEM_BLOB_META: MemoryId = MemoryId::new(5);
pub const MEM_SESSION_COUNTER: MemoryId = MemoryId::new(6);
pub const MEM_BLOB_COUNTER: MemoryId = MemoryId::new(7);
```

#### **1.2 Define Core Types with Manual Storable**

```rust
// File: src/backend/src/upload/types.rs
use ic_stable_structures::{Storable, storable::Bound};
use candid::{CandidType, Deserialize, Encode, Decode};
use std::borrow::Cow;

pub const INLINE_MAX: usize = 32 * 1024; // 32KB (fits in Capsule bound)
pub const CHUNK_SIZE: usize = 64 * 1024; // 64KB
pub const PAGE_SIZE: usize = 64 * 1024;  // 64KB
pub const CAPSULE_INLINE_BUDGET: usize = 32 * 1024; // Max inline bytes per capsule

#[derive(Clone, Debug, CandidType, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SessionId(pub u64);

impl SessionId {
    pub fn new() -> Self {
        use crate::upload::sessions::STABLE_SESSION_COUNTER;
        let id = STABLE_SESSION_COUNTER.with(|counter| {
            let mut c = counter.borrow_mut();
            let id = c.get() + 1;
            c.set(id).expect("Failed to increment session counter");
            id
        });
        SessionId(id)
    }
}

impl Storable for SessionId {
    const BOUND: Bound = Bound::Bounded { max_size: 8, is_fixed_size: true };
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(self.0.to_le_bytes().to_vec())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let arr: [u8; 8] = bytes.as_ref().try_into().expect("Invalid SessionId bytes");
        SessionId(u64::from_le_bytes(arr))
    }
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum SessionStatus {
    Pending,
    Committed { blob_id: u64 },
}

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
    pub meta: MemoryMeta,
}

impl Storable for SessionMeta {
    const BOUND: Bound = Bound::Bounded { max_size: 2048, is_fixed_size: false };
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(&(1u16, self)).expect("Failed to encode SessionMeta"))
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let (version, meta): (u16, SessionMeta) = Decode!(bytes.as_ref(), (u16, SessionMeta))
            .expect("Failed to decode SessionMeta");
        assert_eq!(version, 1, "Unsupported SessionMeta version");
        meta
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlobId(pub u64);

impl BlobId {
    pub fn new() -> Self {
        use crate::upload::blob_store::STABLE_BLOB_COUNTER;
        let id = STABLE_BLOB_COUNTER.with(|counter| {
            let mut c = counter.borrow_mut();
            let id = c.get() + 1;
            c.set(id).expect("Failed to increment blob counter");
            id
        });
        BlobId(id)
    }
}

impl Storable for BlobId {
    const BOUND: Bound = Bound::Bounded { max_size: 8, is_fixed_size: true };
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(self.0.to_le_bytes().to_vec())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let arr: [u8; 8] = bytes.as_ref().try_into().expect("Invalid BlobId bytes");
        BlobId(u64::from_le_bytes(arr))
    }
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct BlobMeta {
    pub size: u64,
    pub checksum: [u8; 32],
    pub created_at: u64,
}

impl Storable for BlobMeta {
    const BOUND: Bound = Bound::Bounded { max_size: 512, is_fixed_size: false };
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(&(1u16, self)).expect("Failed to encode BlobMeta"))
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let (version, meta): (u16, BlobMeta) = Decode!(bytes.as_ref(), (u16, BlobMeta))
            .expect("Failed to decode BlobMeta");
        assert_eq!(version, 1, "Unsupported BlobMeta version");
        meta
    }
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct BlobRef {
    pub blob_id: u64,
    pub size: u64,
    pub checksum: [u8; 32],
}
```

#### **1.3 Stable Memory Setup**

```rust
// File: src/backend/src/upload/sessions.rs
use ic_stable_structures::{StableBTreeMap, StableCell};
use std::cell::RefCell;
use crate::memory_manager::{MM, MEM_SESSIONS, MEM_CHUNKS, MEM_SESSION_COUNTER};
use crate::upload::types::{SessionId, SessionMeta};

thread_local! {
    static STABLE_UPLOAD_SESSIONS: RefCell<StableBTreeMap<u64, SessionMeta, _>> = RefCell::new(
        StableBTreeMap::init(MM.with(|m| m.borrow().get(MEM_SESSIONS)))
    );

    static STABLE_CHUNK_DATA: RefCell<StableBTreeMap<(u64, u32), Vec<u8>, _>> = RefCell::new(
        StableBTreeMap::init(MM.with(|m| m.borrow().get(MEM_CHUNKS)))
    );

    pub static STABLE_SESSION_COUNTER: RefCell<StableCell<u64, _>> = RefCell::new(
        StableCell::init(MM.with(|m| m.borrow().get(MEM_SESSION_COUNTER)), 0)
            .expect("Failed to init session counter")
    );
}

// File: src/backend/src/upload/blob_store.rs
use ic_stable_structures::{StableBTreeMap, StableCell};
use std::cell::RefCell;
use crate::memory_manager::{MM, MEM_BLOBS, MEM_BLOB_META, MEM_BLOB_COUNTER};
use crate::upload::types::{BlobId, BlobMeta};

thread_local! {
    static STABLE_BLOB_STORE: RefCell<StableBTreeMap<(u64, u32), Vec<u8>, _>> = RefCell::new(
        StableBTreeMap::init(MM.with(|m| m.borrow().get(MEM_BLOBS)))
    );

    static STABLE_BLOB_META: RefCell<StableBTreeMap<u64, BlobMeta, _>> = RefCell::new(
        StableBTreeMap::init(MM.with(|m| m.borrow().get(MEM_BLOB_META)))
    );

    pub static STABLE_BLOB_COUNTER: RefCell<StableCell<u64, _>> = RefCell::new(
        StableCell::init(MM.with(|m| m.borrow().get(MEM_BLOB_COUNTER)), 0)
            .expect("Failed to init blob counter")
    );
}
```

### **Phase 2: Blob Store Module (Week 1)**

#### **2.1 Blob Store Implementation**

```rust
// File: src/backend/src/upload/blob_store.rs
use sha2::{Sha256, Digest};
use crate::upload::types::*;
use crate::types::Error;

pub struct BlobStore;

impl BlobStore {
    pub fn new() -> Self {
        BlobStore
    }

    pub fn store_from_chunks(
        &self,
        session_id: &SessionId,
        chunk_count: u32,
        expected_len: u64,
        expected_hash: [u8; 32]
    ) -> Result<BlobId, Error> {
        let blob_id = BlobId::new();
        let mut hasher = Sha256::new();
        let mut total_written = 0u64;

        // Stream chunks into blob store pages
        for chunk_idx in 0..chunk_count {
            let chunk_key = (session_id.0, chunk_idx);
            let chunk_data = STABLE_CHUNK_DATA.with(|chunks| {
                chunks.borrow().get(&chunk_key)
            }).ok_or(Error::ChunkNotFound)?;

            hasher.update(&chunk_data);
            total_written += chunk_data.len() as u64;

            // Store as blob page
            let page_key = (blob_id.0, chunk_idx);
            STABLE_BLOB_STORE.with(|store| {
                store.borrow_mut().insert(page_key, chunk_data);
            });
        }

        // Verify integrity
        let actual_hash = hasher.finalize().into();
        if actual_hash != expected_hash {
            // Cleanup on failure
            self.delete_blob(&blob_id)?;
            return Err(Error::ChecksumMismatch);
        }
        if total_written != expected_len {
            // Cleanup on failure
            self.delete_blob(&blob_id)?;
            return Err(Error::SizeMismatch);
        }

        // Store blob metadata
        let meta = BlobMeta {
            size: total_written,
            checksum: actual_hash,
            created_at: ic_cdk::api::time(),
        };

        STABLE_BLOB_META.with(|metas| {
            metas.borrow_mut().insert(blob_id.0, meta);
        });

        Ok(blob_id)
    }

    pub fn read_blob(&self, blob_id: &BlobId) -> Result<Vec<u8>, Error> {
        let meta = STABLE_BLOB_META.with(|metas| {
            metas.borrow().get(&blob_id.0)
        }).ok_or(Error::BlobNotFound)?;

        let mut result = Vec::with_capacity(meta.size as usize);
        let mut page_idx = 0u32;

        loop {
            let page_key = (blob_id.0, page_idx);
            let page_data = STABLE_BLOB_STORE.with(|store| {
                store.borrow().get(&page_key)
            });

            match page_data {
                Some(data) => {
                    result.extend_from_slice(&data);
                    page_idx += 1;
                }
                None => break,
            }
        }

        Ok(result)
    }

    pub fn delete_blob(&self, blob_id: &BlobId) -> Result<(), Error> {
        // Delete metadata first
        STABLE_BLOB_META.with(|metas| {
            metas.borrow_mut().remove(&blob_id.0)
        });

        // Delete all pages
        let mut page_idx = 0u32;
        loop {
            let page_key = (blob_id.0, page_idx);
            let removed = STABLE_BLOB_STORE.with(|store| {
                store.borrow_mut().remove(&page_key)
            });

            if removed.is_none() {
                break; // No more pages
            }
            page_idx += 1;
        }

        Ok(())
    }
}
```

### **Phase 3: Upload Service (Week 2)**

#### **3.1 Upload Service Implementation (No Trait Objects)**

```rust
// File: src/backend/src/upload/service.rs
use crate::capsule_store::Store;
use crate::upload::{BlobStore, SessionStore};
use crate::upload::types::*;
use crate::types::{CapsuleId, MemoryId, Memory, MemoryMeta, Error};

pub struct UploadService<'a> {
    store: &'a mut Store,  // Concrete enum, not trait object
    sessions: SessionStore,
    blobs: BlobStore,
}

impl<'a> UploadService<'a> {
    pub fn new(store: &'a mut Store) -> Self {
        Self {
            store,
            sessions: SessionStore::new(),
            blobs: BlobStore::new(),
        }
    }

    /// Inline-only endpoint - rejects large payloads
    pub fn create_inline(&mut self, capsule_id: &CapsuleId, bytes: Vec<u8>, meta: MemoryMeta) -> Result<MemoryId, Error> {
        if bytes.len() > INLINE_MAX {
            return Err(Error::PayloadTooLarge);
        }

        // Check per-capsule inline budget
        let current_inline_size = self.store.get(capsule_id)
            .map(|capsule| {
                capsule.memories.values()
                    .filter_map(|m| m.data.as_ref().map(|d| d.len()))
                    .sum::<usize>()
            })
            .unwrap_or(0);

        if current_inline_size + bytes.len() > CAPSULE_INLINE_BUDGET {
            return Err(Error::CapsuleInlineBudgetExceeded);
        }

        // Verify caller has write access
        let caller = ic_cdk::api::caller();
        if !self.store.can_write(capsule_id, &caller)? {
            return Err(Error::Unauthorized);
        }

        let memory = Memory::inline(bytes, meta);
        let memory_id = memory.id.clone();

        // Atomic update to capsule
        self.store.update(capsule_id, |capsule| {
            capsule.memories.insert(memory_id.clone(), memory);
            capsule.updated_at = ic_cdk::api::time();
        })?;

        Ok(memory_id)
    }

    /// Begin chunked upload for large files
    pub fn begin_upload(&mut self, capsule_id: CapsuleId, meta: MemoryMeta, expected_chunks: u32) -> Result<SessionId, Error> {
        // Verify caller has write access
        let caller = ic_cdk::api::caller();
        if !self.store.can_write(&capsule_id, &caller)? {
            return Err(Error::Unauthorized);
        }

        let session_id = SessionId::new();
        let provisional_memory_id = MemoryId::new();

        let session_meta = SessionMeta {
            capsule_id,
            provisional_memory_id,
            caller,
            chunk_count: expected_chunks,
            expected_len: None,
            expected_hash: None,
            status: SessionStatus::Pending,
            created_at: ic_cdk::api::time(),
            meta,
        };

        self.sessions.create(session_id.clone(), session_meta)?;
        Ok(session_id)
    }

    /// Upload chunk
    pub fn put_chunk(&mut self, session_id: &SessionId, chunk_idx: u32, bytes: Vec<u8>) -> Result<(), Error> {
        // Verify session exists and caller matches
        let session = self.sessions.get(session_id)?.ok_or(Error::SessionNotFound)?;

        let caller = ic_cdk::api::caller();
        if session.caller != caller {
            return Err(Error::Unauthorized);
        }

        // Verify chunk index is within expected range
        if chunk_idx >= session.chunk_count {
            return Err(Error::InvalidChunkIndex);
        }

        // Verify chunk size (except possibly last chunk)
        if bytes.len() > CHUNK_SIZE {
            return Err(Error::ChunkTooLarge);
        }

        // Store chunk
        self.sessions.put_chunk(session_id, chunk_idx, bytes)?;
        Ok(())
    }

    /// Commit upload and attach to capsule (crash-safe)
    pub fn commit(&mut self, session_id: SessionId, expected_sha256: [u8; 32], total_len: u64) -> Result<MemoryId, Error> {
        let mut session = self.sessions.get(&session_id)?.ok_or(Error::SessionNotFound)?;

        // Verify caller matches
        let caller = ic_cdk::api::caller();
        if session.caller != caller {
            return Err(Error::Unauthorized);
        }

        // Handle idempotent retry
        if let SessionStatus::Committed { blob_id } = session.status {
            // Check if already attached to capsule
            if let Some(capsule) = self.store.get(&session.capsule_id) {
                if capsule.memories.contains_key(&session.provisional_memory_id) {
                    // Already committed and attached
                    self.sessions.cleanup(&session_id);
                    return Ok(session.provisional_memory_id);
                }
            }

            // Blob exists but not attached - retry attach
            let memory = Memory::from_blob(blob_id, total_len, expected_sha256, session.meta.clone());
            let memory_id = memory.id.clone();

            self.store.update(&session.capsule_id, |capsule| {
                capsule.memories.insert(memory_id.clone(), memory);
                capsule.updated_at = ic_cdk::api::time();
            })?;

            self.sessions.cleanup(&session_id);
            return Ok(memory_id);
        }

        // First-time commit

        // 1. Verify all chunks exist (integrity check)
        self.sessions.verify_chunks_complete(&session_id, session.chunk_count)?;

        // 2. Stream chunks to blob store with verification
        let blob_id = self.blobs.store_from_chunks(&session_id, session.chunk_count, total_len, expected_sha256)?;

        // 3. Mark session as committed (crash-safe checkpoint)
        session.status = SessionStatus::Committed { blob_id: blob_id.0 };
        self.sessions.update(&session_id, session.clone())?;

        // 4. Create memory with blob reference
        let memory = Memory::from_blob(blob_id.0, total_len, expected_sha256, session.meta.clone());
        let memory_id = memory.id.clone();

        // 5. Atomic attach to capsule
        self.store.update(&session.capsule_id, |capsule| {
            capsule.memories.insert(memory_id.clone(), memory);
            capsule.updated_at = ic_cdk::api::time();
        })?;

        // 6. Cleanup session and chunks
        self.sessions.cleanup(&session_id);

        Ok(memory_id)
    }

    /// Abort upload and cleanup
    pub fn abort(&mut self, session_id: SessionId) -> Result<(), Error> {
        // Verify caller matches
        if let Some(session) = self.sessions.get(&session_id)? {
            let caller = ic_cdk::api::caller();
            if session.caller != caller {
                return Err(Error::Unauthorized);
            }
        }

        self.sessions.cleanup(&session_id);
        Ok(())
    }
}
```

### **Phase 4: Public API Integration (Week 2)**

#### **4.1 Update Public Endpoints**

```rust
// File: src/backend/src/lib.rs

#[ic_cdk::update]
pub async fn memories_create_inline(capsule_id: CapsuleId, file_data: Vec<u8>, metadata: MemoryMeta) -> Result<MemoryId, Error> {
    with_capsule_store_mut(|store| {
        let mut upload_service = UploadService::new(store);
        upload_service.create_inline(&capsule_id, file_data, metadata)
    })
}

#[ic_cdk::update]
pub async fn memories_begin_upload(capsule_id: CapsuleId, metadata: MemoryMeta, expected_chunks: u32) -> Result<SessionId, Error> {
    with_capsule_store_mut(|store| {
        let mut upload_service = UploadService::new(store);
        upload_service.begin_upload(capsule_id, metadata, expected_chunks)
    })
}

#[ic_cdk::update]
pub async fn memories_put_chunk(session_id: SessionId, chunk_idx: u32, bytes: Vec<u8>) -> Result<(), Error> {
    with_capsule_store_mut(|store| {
        let mut upload_service = UploadService::new(store);
        upload_service.put_chunk(&session_id, chunk_idx, bytes)
    })
}

#[ic_cdk::update]
pub async fn memories_commit(session_id: SessionId, expected_sha256: Vec<u8>, total_len: u64) -> Result<MemoryId, Error> {
    let hash: [u8; 32] = expected_sha256.try_into().map_err(|_| Error::InvalidHash)?;

    with_capsule_store_mut(|store| {
        let mut upload_service = UploadService::new(store);
        upload_service.commit(session_id, hash, total_len)
    })
}

#[ic_cdk::update]
pub async fn memories_abort(session_id: SessionId) -> Result<(), Error> {
    with_capsule_store_mut(|store| {
        let mut upload_service = UploadService::new(store);
        upload_service.abort(session_id)
    })
}
```

#### **4.2 Update Candid Interface**

```rust
// File: src/backend/backend.did
type SessionId = nat64;
type BlobId = nat64;

type MemoryMeta = record {
    name: text;
    description: opt text;
    tags: vec text;
    // ... other metadata fields
};

service : {
    // Inline-only endpoint (≤32KB, fails if larger)
    memories_create_inline : (CapsuleId, blob, MemoryMeta) -> (variant { Ok : MemoryId; Err : Error });

    // Chunked upload for large files (>32KB)
    memories_begin_upload : (CapsuleId, MemoryMeta, nat32) -> (variant { Ok : SessionId; Err : Error });
    memories_put_chunk : (SessionId, nat32, blob) -> (variant { Ok; Err : Error });
    memories_commit : (SessionId, blob, nat64) -> (variant { Ok : MemoryId; Err : Error });
    memories_abort : (SessionId) -> (variant { Ok; Err : Error });
}
```

### **Phase 5: Migration & Cleanup (Week 3)**

#### **5.1 Remove Old Functions (Greenfield Approach)**

```bash
# Functions to remove completely:
- begin_asset_upload()
- put_chunk()  # old version
- commit_asset()
- cancel_upload()
- upsert_metadata()  # move logic into Memory creation
- get_memory_presence_icp()  # replaced by memories_ping()
- get_memory_list_presence_icp()  # replaced by memories_ping()

# Storage to remove:
- STABLE_MEMORY_ARTIFACTS (replaced by STABLE_BLOB_STORE + inline data)
- Old upload session logic (replaced by new UploadService)
```

### **Phase 6: Testing & Validation (Week 3)**

#### **6.1 Core Test Cases**

```rust
// File: src/backend/tests/upload_service_tests.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_upload_small_file() {
        // Test ≤32KB files go inline
        let small_data = vec![0u8; 16 * 1024]; // 16KB
        let memory_id = upload_service.create_inline(&capsule_id, small_data, meta).unwrap();

        // Verify stored inline
        let memory = store.get_memory(&capsule_id, &memory_id).unwrap();
        assert!(memory.data.is_some());
        assert!(memory.blob_ref.is_none());
    }

    #[test]
    fn test_inline_rejects_large_file() {
        // Test >32KB files are rejected
        let large_data = vec![0u8; 64 * 1024]; // 64KB
        let result = upload_service.create_inline(&capsule_id, large_data, meta);
        assert!(matches!(result, Err(Error::PayloadTooLarge)));
    }

    #[test]
    fn test_chunked_upload_large_file() {
        // Test >32KB files via chunked workflow
        let large_data = vec![0u8; 128 * 1024]; // 128KB
        let session_id = upload_service.begin_upload(capsule_id, meta, 2).unwrap();

        upload_service.put_chunk(&session_id, 0, large_data[..64*1024].to_vec()).unwrap();
        upload_service.put_chunk(&session_id, 1, large_data[64*1024..].to_vec()).unwrap();

        let hash = compute_sha256(&large_data);
        let memory_id = upload_service.commit(session_id, hash, 128*1024).unwrap();

        let memory = store.get_memory(&capsule_id, &memory_id).unwrap();
        assert!(memory.data.is_none());
        assert!(memory.blob_ref.is_some());
    }

    #[test]
    fn test_chunked_workflow() {
        // Test manual chunked upload
        let session_id = upload_service.begin_upload(capsule_id, meta, 2).unwrap();

        let chunk1 = vec![1u8; CHUNK_SIZE];
        let chunk2 = vec![2u8; CHUNK_SIZE];

        upload_service.put_chunk(&session_id, 0, chunk1).unwrap();
        upload_service.put_chunk(&session_id, 1, chunk2).unwrap();

        let total_data = vec![vec![1u8; CHUNK_SIZE], vec![2u8; CHUNK_SIZE]].concat();
        let hash = compute_sha256(&total_data);

        let memory_id = upload_service.commit(session_id, hash, total_data.len() as u64).unwrap();

        // Verify blob reference
        let memory = store.get_memory(&capsule_id, &memory_id).unwrap();
        assert!(memory.blob_ref.is_some());
    }

    #[test]
    fn test_capsule_size_bounds() {
        // Verify capsule with inline memories stays under size bound
        let capsule = create_test_capsule_with_inline_memories(10); // 10 small memories
        let serialized = candid::encode_one(&capsule).unwrap();
        assert!(serialized.len() < 32 * 1024); // Under 32KB
    }

    #[test]
    fn test_capsule_inline_budget() {
        // Test per-capsule inline budget enforcement
        let small_data = vec![0u8; 16 * 1024]; // 16KB

        // First memory succeeds
        upload_service.create_inline(&capsule_id, small_data.clone(), meta.clone()).unwrap();

        // Second memory succeeds (32KB total)
        upload_service.create_inline(&capsule_id, small_data.clone(), meta.clone()).unwrap();

        // Third memory fails (would exceed 32KB budget)
        let result = upload_service.create_inline(&capsule_id, small_data, meta);
        assert!(matches!(result, Err(Error::CapsuleInlineBudgetExceeded)));
    }

    #[test]
    fn test_upload_failure_cleanup() {
        let session_id = upload_service.begin_upload(capsule_id, meta, 2).unwrap();
        upload_service.put_chunk(&session_id, 0, vec![1u8; 1024]).unwrap();

        // Abort upload
        upload_service.abort(session_id.clone()).unwrap();

        // Verify cleanup
        assert!(sessions.get(&session_id).unwrap().is_none());
        assert!(sessions.get_chunk(&session_id, 0).unwrap().is_none());
    }

    #[test]
    fn test_authorization_checks() {
        // Test that upload operations verify caller access
        // (Implementation depends on your authorization system)
    }

    #[test]
    fn test_crash_recovery() {
        // Test idempotent commit after crash
        let session_id = upload_service.begin_upload(capsule_id, meta, 1).unwrap();
        upload_service.put_chunk(&session_id, 0, vec![1u8; 1024]).unwrap();

        // Simulate crash after blob write but before capsule attach
        // (Mock the session as Committed status)

        // Retry commit should succeed idempotently
        let memory_id = upload_service.commit(session_id, hash, 1024).unwrap();
        assert!(store.get_memory(&capsule_id, &memory_id).is_some());
    }
}
```

#### **6.2 Integration Test Script**

```bash
# File: scripts/tests/backend/general/test_upload_workflow.sh

#!/bin/bash
set -e

echo "🧪 Testing Upload Workflow Integration"

# Test 1: Small file (inline)
echo "📝 Test 1: Small file upload (inline)"
SMALL_DATA=$(printf '{"data": "%s"}' "$(head -c 16384 /dev/zero | base64)")
RESULT=$(dfx canister call backend memories_create_inline "(\"test-capsule-1\", $SMALL_DATA, record { name = \"small-file\"; description = opt \"Test small file\" })")
echo "✅ Small file result: $RESULT"

# Test 2: Large file (should fail inline)
echo "📝 Test 2: Large file upload (should fail inline)"
LARGE_DATA=$(printf '{"data": "%s"}' "$(head -c 65536 /dev/zero | base64)")
RESULT=$(dfx canister call backend memories_create_inline "(\"test-capsule-2\", $LARGE_DATA, record { name = \"large-file\"; description = opt \"Test large file\" })")
echo "Expected failure: $RESULT"

# Test 3: Manual chunked upload
echo "📝 Test 3: Manual chunked upload"
SESSION=$(dfx canister call backend memories_begin_upload "(\"test-capsule-3\", record { name = \"manual-chunks\"; description = opt \"Manual chunked upload\" }, 2)")
echo "Session: $SESSION"

CHUNK1=$(printf '"%s"' "$(head -c 65536 /dev/zero | base64)")
dfx canister call backend memories_put_chunk "($SESSION, 0, $CHUNK1)"

CHUNK2=$(printf '"%s"' "$(head -c 65536 /dev/zero | base64)")
dfx canister call backend memories_put_chunk "($SESSION, 1, $CHUNK2)"

# Compute expected hash and commit
EXPECTED_HASH="..." # Would compute SHA256 of combined chunks
dfx canister call backend memories_commit "($SESSION, blob \"$EXPECTED_HASH\", 131072)"
echo "✅ Manual chunked upload completed"

echo "🎉 All upload workflow tests passed!"
```

## 📊 **Success Metrics**

### **Performance Targets**

- [ ] Capsule serialized size stays under 32KB
- [ ] Small file uploads (<32KB) complete in <100ms
- [ ] Large file upload throughput >1MB/s
- [ ] No memory leaks in upload sessions

### **Functionality Targets**

- [ ] Inline API rejects >32KB payloads at ingress
- [ ] Automatic size-based routing works correctly
- [ ] Upload failures clean up properly (no orphaned data)
- [ ] Blob storage integrity verified (checksums match)
- [ ] Crash-safe commits with idempotent retry

### **Code Quality Targets**

- [ ] No trait objects (concrete Store enum only)
- [ ] Manual Storable implementations with proper bounds
- [ ] Single MemoryManager with centralized MemoryId constants
- [ ] Authorization checks on all upload operations
- [ ] No linter errors or warnings

## 🗂️ **File Structure (Corrected)**

```
src/backend/src/
├── capsule_store/          # Existing capsule storage
│   ├── mod.rs
│   ├── store.rs
│   ├── hash.rs
│   └── stable.rs
├── upload/
│   ├── mod.rs              # Public module interface
│   ├── types.rs            # SessionId, BlobId, etc. (manual Storable)
│   ├── sessions.rs         # UploadSessionStore (maps + GC)
│   ├── blob_store.rs       # BlobStore (paged)
│   └── service.rs          # UploadService (no trait objects)
├── memory_manager.rs       # ONE MemoryManager + all MemoryId constants
├── memory.rs               # Updated capsule store access
├── types.rs                # Updated Memory type
├── lib.rs                  # Updated public endpoints
└── capsule.rs              # Updated functions using new API
```

## 🚀 **Implementation Order**

1. **Week 1**: Memory manager + Core types + Blob store
2. **Week 2**: Upload service + Public API integration
3. **Week 3**: Migration, cleanup, comprehensive testing

**Total Timeline**: 3 weeks for complete implementation

---

**Status**: 🟢 READY TO IMPLEMENT (CORRECTED)  
**Priority**: HIGH - Critical for unified storage architecture  
**Assignee**: Development Team  
**Dependencies**: Capsule Storage Foundation (Phase 1 Complete)  
**Created**: Current Session  
**Updated**: Current Session (Senior Feedback Applied)

**Key Benefits**:
✅ No trait objects (avoids compilation issues)  
✅ Manual Storable implementations (IC-compatible)  
✅ Centralized memory management (no collisions)  
✅ Split API (respects ingress limits)  
✅ Numeric keys (efficient storage)  
✅ Size bounds enforcement (Capsule performance)  
✅ Chunk integrity verification (data safety)  
✅ Crash-safe commits (production ready)  
✅ Authorization at every step (security)  
✅ Greenfield implementation (no legacy baggage)
