# Upload Workflow Implementation Plan - Hybrid Architecture

## 🎯 **Senior Developer Decision: Option 3 - Hybrid Persistence**

**Single public workflow** with dual internal paths:

- **≤64KB files**: Inline directly in Capsule (fast path)
- **>64KB files**: Chunked → blob store → reference in Capsule
- **Sessions/chunks**: Temporary (auto-cleanup)
- **Artifacts**: Permanent (blob store)
- **Single API**: Hide complexity behind size thresholds

## 🏗️ **Architecture Overview**

### **Public Surface (Clean API)**

```rust
// Single entry point - size determines path automatically
memories_create(capsule_id, file_data, metadata) -> MemoryId

// For large files (chunked upload)
memories_begin_upload(capsule_id, metadata) -> SessionId
memories_put_chunk(session_id, chunk_idx, bytes) -> ()
memories_commit(session_id, expected_sha256, total_len) -> MemoryId
memories_abort(session_id) -> ()
```

### **Internal Storage Layout**

```rust
// Single source of truth (unchanged)
STABLE_CAPSULES: StableBTreeMap<CapsuleId, Capsule>
├── Capsule.memories: HashMap<MemoryId, Memory>
    ├── Memory.data: Option<Vec<u8>>        // ≤64KB inline
    └── Memory.blob_ref: Option<BlobRef>    // >64KB reference

// Internal upload management (temporary)
STABLE_UPLOAD_SESSIONS: StableBTreeMap<SessionId, SessionMeta>
STABLE_CHUNK_DATA: StableBTreeMap<(SessionId, u32), Vec<u8>>

// Internal blob storage (permanent)
STABLE_BLOB_STORE: StableBTreeMap<(BlobId, u32), Vec<u8>>  // Paged
STABLE_BLOB_META: StableBTreeMap<BlobId, BlobMeta>         // Metadata
```

## 📋 **Implementation Phases**

### **Phase 1: Core Infrastructure (Week 1)**

#### **1.1 Define Core Types**

```rust
// File: src/backend/src/upload/types.rs
pub const INLINE_MAX: usize = 64 * 1024; // 64KB
pub const CHUNK_SIZE: usize = 64 * 1024; // 64KB
pub const PAGE_SIZE: usize = 64 * 1024;  // 64KB

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct SessionId(pub String);

#[derive(Clone, Debug, CandidType, Deserialize, Storable)]
pub struct SessionMeta {
    pub capsule_id: CapsuleId,
    pub provisional_memory_id: MemoryId,
    pub expected_len: Option<u64>,
    pub expected_hash: Option<[u8; 32]>,
    pub created_at: u64,
    pub meta: MemoryMeta,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct BlobId(pub String);

#[derive(Clone, Debug, CandidType, Deserialize, Storable)]
pub struct BlobMeta {
    pub size: u64,
    pub checksum: [u8; 32],
    pub created_at: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct BlobRef {
    pub blob_id: BlobId,
    pub size: u64,
    pub checksum: [u8; 32],
}
```

#### **1.2 Update Memory Types**

```rust
// File: src/backend/src/types.rs
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Memory {
    pub id: MemoryId,
    pub data: Option<Vec<u8>>,        // ≤64KB inline data
    pub blob_ref: Option<BlobRef>,    // >64KB blob reference
    pub metadata: MemoryMeta,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Memory {
    pub fn inline(bytes: Vec<u8>, meta: MemoryMeta) -> Self {
        Self {
            id: MemoryId::new(),
            data: Some(bytes),
            blob_ref: None,
            metadata: meta,
            created_at: ic_cdk::api::time(),
            updated_at: ic_cdk::api::time(),
        }
    }

    pub fn from_blob(blob_id: BlobId, size: u64, checksum: [u8; 32], meta: MemoryMeta) -> Self {
        Self {
            id: MemoryId::new(),
            data: None,
            blob_ref: Some(BlobRef { blob_id, size, checksum }),
            metadata: meta,
            created_at: ic_cdk::api::time(),
            updated_at: ic_cdk::api::time(),
        }
    }
}
```

#### **1.3 Stable Memory Setup**

```rust
// File: src/backend/src/memory.rs
thread_local! {
    // Existing
    static STABLE_CAPSULES: RefCell<StableBTreeMap<CapsuleId, Capsule, Memory>> = /* ... */;

    // New upload management (temporary)
    static STABLE_UPLOAD_SESSIONS: RefCell<StableBTreeMap<SessionId, SessionMeta, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))))
    );

    static STABLE_CHUNK_DATA: RefCell<StableBTreeMap<(SessionId, u32), Vec<u8>, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))))
    );

    // New blob storage (permanent)
    static STABLE_BLOB_STORE: RefCell<StableBTreeMap<(BlobId, u32), Vec<u8>, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))))
    );

    static STABLE_BLOB_META: RefCell<StableBTreeMap<BlobId, BlobMeta, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6))))
    );
}
```

### **Phase 2: Blob Store Module (Week 1)**

#### **2.1 Blob Store Implementation**

```rust
// File: src/backend/src/upload/blob_store.rs
pub struct BlobStore;

impl BlobStore {
    pub fn store_from_chunks(&self, session_id: &SessionId, expected_len: u64, expected_hash: [u8; 32]) -> Result<BlobId, Error> {
        let blob_id = BlobId::new();
        let mut hasher = Sha256::new();
        let mut total_written = 0u64;
        let mut page_idx = 0u32;

        // Stream chunks into blob store pages
        loop {
            let chunk_key = (session_id.clone(), page_idx);
            let chunk_data = STABLE_CHUNK_DATA.with(|chunks| {
                chunks.borrow().get(&chunk_key)
            });

            match chunk_data {
                Some(data) => {
                    hasher.update(&data);
                    total_written += data.len() as u64;

                    // Store as blob page
                    let page_key = (blob_id.clone(), page_idx);
                    STABLE_BLOB_STORE.with(|store| {
                        store.borrow_mut().insert(page_key, data);
                    });

                    page_idx += 1;
                }
                None => break, // No more chunks
            }
        }

        // Verify integrity
        let actual_hash = hasher.finalize().into();
        if actual_hash != expected_hash {
            return Err(Error::ChecksumMismatch);
        }
        if total_written != expected_len {
            return Err(Error::SizeMismatch);
        }

        // Store blob metadata
        let meta = BlobMeta {
            size: total_written,
            checksum: actual_hash,
            created_at: ic_cdk::api::time(),
        };

        STABLE_BLOB_META.with(|metas| {
            metas.borrow_mut().insert(blob_id.clone(), meta);
        });

        Ok(blob_id)
    }

    pub fn read_blob(&self, blob_id: &BlobId) -> Result<Vec<u8>, Error> {
        let meta = STABLE_BLOB_META.with(|metas| {
            metas.borrow().get(blob_id)
        }).ok_or(Error::BlobNotFound)?;

        let mut result = Vec::with_capacity(meta.size as usize);
        let mut page_idx = 0u32;

        loop {
            let page_key = (blob_id.clone(), page_idx);
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
            metas.borrow_mut().remove(blob_id)
        });

        // Delete all pages
        let mut page_idx = 0u32;
        loop {
            let page_key = (blob_id.clone(), page_idx);
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

#### **3.1 Upload Service Implementation**

```rust
// File: src/backend/src/upload/service.rs
pub struct UploadService<'a> {
    store: &'a mut dyn CapsuleStore,
    blob_store: BlobStore,
}

impl<'a> UploadService<'a> {
    pub fn new(store: &'a mut dyn CapsuleStore) -> Self {
        Self {
            store,
            blob_store: BlobStore,
        }
    }

    /// Single entry point - automatically chooses inline vs chunked
    pub fn create_memory(&mut self, capsule_id: &CapsuleId, bytes: Vec<u8>, meta: MemoryMeta) -> Result<MemoryId, Error> {
        if bytes.len() <= INLINE_MAX {
            self.put_inline(capsule_id, bytes, meta)
        } else {
            // For large files, use chunked upload
            let session_id = self.begin_upload(capsule_id.clone(), meta)?;

            // Split into chunks and upload
            for (idx, chunk) in bytes.chunks(CHUNK_SIZE).enumerate() {
                self.put_chunk(&session_id, idx as u32, chunk.to_vec())?;
            }

            // Commit with checksum
            let hash = Self::compute_sha256(&bytes);
            self.commit(session_id, hash, bytes.len() as u64)
        }
    }

    /// Fast path for small files (≤64KB)
    pub fn put_inline(&mut self, capsule_id: &CapsuleId, bytes: Vec<u8>, meta: MemoryMeta) -> Result<MemoryId, Error> {
        if bytes.len() > INLINE_MAX {
            return Err(Error::TooLargeForInline);
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
    pub fn begin_upload(&mut self, capsule_id: CapsuleId, meta: MemoryMeta) -> Result<SessionId, Error> {
        let session_id = SessionId::new();
        let provisional_memory_id = MemoryId::new();

        let session_meta = SessionMeta {
            capsule_id,
            provisional_memory_id,
            expected_len: None,
            expected_hash: None,
            created_at: ic_cdk::api::time(),
            meta,
        };

        STABLE_UPLOAD_SESSIONS.with(|sessions| {
            sessions.borrow_mut().insert(session_id.clone(), session_meta);
        });

        Ok(session_id)
    }

    /// Upload chunk
    pub fn put_chunk(&mut self, session_id: &SessionId, chunk_idx: u32, bytes: Vec<u8>) -> Result<(), Error> {
        // Verify session exists
        let _session = STABLE_UPLOAD_SESSIONS.with(|sessions| {
            sessions.borrow().get(session_id)
        }).ok_or(Error::SessionNotFound)?;

        // Store chunk
        let chunk_key = (session_id.clone(), chunk_idx);
        STABLE_CHUNK_DATA.with(|chunks| {
            chunks.borrow_mut().insert(chunk_key, bytes);
        });

        Ok(())
    }

    /// Commit upload and attach to capsule
    pub fn commit(&mut self, session_id: SessionId, expected_sha256: [u8; 32], total_len: u64) -> Result<MemoryId, Error> {
        // Get session metadata
        let session = STABLE_UPLOAD_SESSIONS.with(|sessions| {
            sessions.borrow().get(&session_id)
        }).ok_or(Error::SessionNotFound)?;

        // 1. Store chunks as blob
        let blob_id = self.blob_store.store_from_chunks(&session_id, total_len, expected_sha256)?;

        // 2. Create memory with blob reference
        let memory = Memory::from_blob(blob_id, total_len, expected_sha256, session.meta.clone());
        let memory_id = memory.id.clone();

        // 3. Atomic attach to capsule
        self.store.update(&session.capsule_id, |capsule| {
            capsule.memories.insert(memory_id.clone(), memory);
            capsule.updated_at = ic_cdk::api::time();
        })?;

        // 4. Cleanup session and chunks
        self.cleanup_session(&session_id);

        Ok(memory_id)
    }

    /// Abort upload and cleanup
    pub fn abort(&mut self, session_id: SessionId) -> Result<(), Error> {
        self.cleanup_session(&session_id);
        Ok(())
    }

    fn cleanup_session(&self, session_id: &SessionId) {
        // Remove session
        STABLE_UPLOAD_SESSIONS.with(|sessions| {
            sessions.borrow_mut().remove(session_id);
        });

        // Remove all chunks for this session
        let mut chunk_idx = 0u32;
        loop {
            let chunk_key = (session_id.clone(), chunk_idx);
            let removed = STABLE_CHUNK_DATA.with(|chunks| {
                chunks.borrow_mut().remove(&chunk_key)
            });

            if removed.is_none() {
                break; // No more chunks
            }
            chunk_idx += 1;
        }
    }

    fn compute_sha256(data: &[u8]) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }
}
```

### **Phase 4: Public API Integration (Week 2)**

#### **4.1 Update Public Endpoints**

```rust
// File: src/backend/src/lib.rs

#[ic_cdk::update]
pub async fn memories_create(capsule_id: CapsuleId, file_data: Vec<u8>, metadata: MemoryMeta) -> Result<MemoryId, Error> {
    with_capsule_store_mut(|store| {
        let mut upload_service = UploadService::new(store);
        upload_service.create_memory(&capsule_id, file_data, metadata)
    })
}

#[ic_cdk::update]
pub async fn memories_begin_upload(capsule_id: CapsuleId, metadata: MemoryMeta) -> Result<SessionId, Error> {
    with_capsule_store_mut(|store| {
        let mut upload_service = UploadService::new(store);
        upload_service.begin_upload(capsule_id, metadata)
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
type SessionId = text;
type BlobId = text;

type MemoryMeta = record {
    name: text;
    description: opt text;
    tags: vec text;
    // ... other metadata fields
};

service : {
    // Single entry point (auto-detects inline vs chunked)
    memories_create : (CapsuleId, blob, MemoryMeta) -> (variant { Ok : MemoryId; Err : Error });

    // Chunked upload for large files
    memories_begin_upload : (CapsuleId, MemoryMeta) -> (variant { Ok : SessionId; Err : Error });
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

#### **5.2 Update Related Functions**

```rust
// File: src/backend/src/capsule.rs

// Update sync_gallery_memories to use new API
pub fn sync_gallery_memories(gallery_id: GalleryId) -> Result<(), Error> {
    with_capsule_store_mut(|store| {
        let mut upload_service = UploadService::new(store);

        // Get gallery memories from external source
        let external_memories = fetch_external_gallery_memories(&gallery_id)?;

        for ext_memory in external_memories {
            // Use new unified API
            upload_service.create_memory(
                &ext_memory.capsule_id,
                ext_memory.data,
                ext_memory.metadata
            )?;
        }

        Ok(())
    })
}
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
        // Test ≤64KB files go inline
        let small_data = vec![0u8; 32 * 1024]; // 32KB
        let memory_id = upload_service.create_memory(&capsule_id, small_data, meta).unwrap();

        // Verify stored inline
        let memory = store.get_memory(&capsule_id, &memory_id).unwrap();
        assert!(memory.data.is_some());
        assert!(memory.blob_ref.is_none());
    }

    #[test]
    fn test_chunked_upload_large_file() {
        // Test >64KB files go to blob store
        let large_data = vec![0u8; 128 * 1024]; // 128KB
        let memory_id = upload_service.create_memory(&capsule_id, large_data, meta).unwrap();

        // Verify stored as blob reference
        let memory = store.get_memory(&capsule_id, &memory_id).unwrap();
        assert!(memory.data.is_none());
        assert!(memory.blob_ref.is_some());
    }

    #[test]
    fn test_chunked_workflow() {
        // Test manual chunked upload
        let session_id = upload_service.begin_upload(capsule_id, meta).unwrap();

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
    fn test_upload_failure_cleanup() {
        let session_id = upload_service.begin_upload(capsule_id, meta).unwrap();
        upload_service.put_chunk(&session_id, 0, vec![1u8; 1024]).unwrap();

        // Abort upload
        upload_service.abort(session_id.clone()).unwrap();

        // Verify cleanup
        assert!(get_session(&session_id).is_none());
        assert!(get_chunk(&session_id, 0).is_none());
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
SMALL_DATA=$(printf '{"data": "%s"}' "$(head -c 32768 /dev/zero | base64)")
RESULT=$(dfx canister call backend memories_create "(\"test-capsule-1\", $SMALL_DATA, record { name = \"small-file\"; description = opt \"Test small file\" })")
echo "✅ Small file result: $RESULT"

# Test 2: Large file (chunked - auto)
echo "📝 Test 2: Large file upload (auto-chunked)"
LARGE_DATA=$(printf '{"data": "%s"}' "$(head -c 131072 /dev/zero | base64)")
RESULT=$(dfx canister call backend memories_create "(\"test-capsule-2\", $LARGE_DATA, record { name = \"large-file\"; description = opt \"Test large file\" })")
echo "✅ Large file result: $RESULT"

# Test 3: Manual chunked upload
echo "📝 Test 3: Manual chunked upload"
SESSION=$(dfx canister call backend memories_begin_upload "(\"test-capsule-3\", record { name = \"manual-chunks\"; description = opt \"Manual chunked upload\" })")
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
- [ ] Small file uploads (<64KB) complete in <100ms
- [ ] Large file upload throughput >1MB/s
- [ ] No memory leaks in upload sessions

### **Functionality Targets**

- [ ] Single API handles both small and large files
- [ ] Automatic size-based routing works correctly
- [ ] Upload failures clean up properly (no orphaned data)
- [ ] Blob storage integrity verified (checksums match)

### **Code Quality Targets**

- [ ] No old upload functions remain in codebase
- [ ] Clean separation between public API and internal storage
- [ ] Comprehensive test coverage (>95%)
- [ ] No linter errors or warnings

## 🗂️ **File Structure**

```
src/backend/src/
├── upload/
│   ├── mod.rs              # Public module interface
│   ├── types.rs            # SessionId, BlobId, etc.
│   ├── service.rs          # UploadService implementation
│   ├── blob_store.rs       # Blob storage management
│   └── tests.rs            # Unit tests
├── memory.rs               # Updated stable memory setup
├── types.rs                # Updated Memory type
├── lib.rs                  # Updated public endpoints
└── capsule.rs              # Updated functions using new API
```

## 🚀 **Implementation Order**

1. **Week 1**: Core types + Blob store module
2. **Week 2**: Upload service + Public API integration
3. **Week 3**: Migration, cleanup, testing

**Total Timeline**: 3 weeks for complete implementation

---

**Status**: 🟢 READY TO IMPLEMENT  
**Priority**: HIGH - Critical for unified storage architecture  
**Assignee**: Development Team  
**Dependencies**: Capsule Storage Foundation (Phase 1 Complete)  
**Created**: Current Session

**Key Benefits**:
✅ Single source of truth (Capsule storage)  
✅ Clean public API (size complexity hidden)  
✅ Optimal performance (inline small, chunked large)  
✅ Greenfield implementation (no legacy baggage)  
✅ Crash-safe with proper cleanup  
✅ Senior developer approved architecture
