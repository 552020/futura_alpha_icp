# Fix get_stable_memory_stats() Function

## Problem

The `get_stable_memory_stats()` function in `memory.rs` has a critical issue - it's trying to use dead code that doesn't actually work.

## Current Broken Function

```rust
pub fn get_stable_memory_stats() -> (u64, u64, u64) {
    let capsule_count = with_stable_capsules(|capsules| capsules.len()); // ❌ WRONG!
    let session_count = with_stable_upload_sessions(|sessions| sessions.len());
    let artifact_count = with_stable_memory_artifacts(|artifacts| artifacts.len());

    (
        capsule_count.try_into().unwrap(),
        session_count,
        artifact_count,
    )
}
```

## Issues

1. **`with_stable_capsules()` is dead code** - Marked as ` ` and not used
2. **Wrong storage system** - Capsules are stored in CapsuleStore, not the dead StableBTreeMap
3. **Incomplete stats** - Missing other stable memory structures
4. **Potential runtime errors** - Function may panic if called

## Root Cause

The function was written when there were multiple capsule storage systems, but it's using the wrong one. The real capsule storage is in the **CapsuleStore system**, not the dead `STABLE_CAPSULES`.

## Solution

### Option 1: Fix Current Function (Recommended)

```rust
pub fn get_stable_memory_stats() -> (u64, u64, u64) {
    // Use CapsuleStore for capsules (the real system)
    let capsule_count = with_capsule_store(|store| store.len());

    // Use stable structures for other data
    let session_count = with_stable_upload_sessions(|sessions| sessions.len());
    let artifact_count = with_stable_memory_artifacts(|artifacts| artifacts.len());

    (
        capsule_count.try_into().unwrap(),
        session_count,
        artifact_count,
    )
}
```

### Option 2: Complete Stats Function

```rust
pub fn get_complete_stable_memory_stats() -> (u64, u64, u64, u64, u64, u64) {
    let capsule_count = with_capsule_store(|store| store.len());
    let session_count = with_stable_upload_sessions(|sessions| sessions.len());
    let artifact_count = with_stable_memory_artifacts(|artifacts| artifacts.len());
    let chunk_count = with_stable_chunk_data(|chunks| chunks.len());

    // Blob store stats (if needed)
    let blob_count = 0; // Would need access functions for blob store
    let blob_meta_count = 0; // Would need access functions for blob meta

    (
        capsule_count.try_into().unwrap(),
        session_count,
        artifact_count,
        chunk_count,
        blob_count,
        blob_meta_count,
    )
}
```

## Complete Analysis of Stable Memory Structures

**Note**: These are ALL the stable data structures in our codebase. ICP provides the underlying stable memory infrastructure, but these are our application-level stable structures built on top of ICP's `StableBTreeMap` and `StableCell` primitives.

### **1. ⚠️ DEAD CODE Structures (in `memory.rs`) - REMOVE THESE**

#### **CRITICAL: Memory Collision Risk**

- **`STABLE_CAPSULES`** - `StableBTreeMap<String, Capsule, Memory>` (uses `CAPSULES_MEMORY_ID` = 0) - **DEAD CODE**
- **`STABLE_UPLOAD_SESSIONS`** - `StableBTreeMap<String, UploadSession, Memory>` (uses `UPLOAD_SESSIONS_MEMORY_ID` = 3) - **DEAD CODE**
- **`STABLE_MEMORY_ARTIFACTS`** - `StableBTreeMap<String, MemoryArtifact, Memory>` (uses `MEMORY_ARTIFACTS_MEMORY_ID` = 4) - **DEAD CODE**
- **`STABLE_CHUNK_DATA`** - `StableBTreeMap<String, ChunkData, Memory>` (uses `CHUNK_DATA_MEMORY_ID` = 5) - **DEAD CODE**

### **2. ✅ Active Upload System (in `upload/sessions.rs`)**

- **`STABLE_UPLOAD_SESSIONS`** - `StableBTreeMap<u64, SessionMeta, Memory>` (uses `MEM_SESSIONS` = 3)
- **`STABLE_CHUNK_DATA`** - `StableBTreeMap<(u64, u32), Vec<u8>, Memory>` (uses `MEM_SESSIONS_CHUNKS` = 4)
- **`STABLE_SESSION_COUNTER`** - `StableCell<u64, Memory>` (uses `MEM_SESSIONS_COUNTER` = 5)

### **3. ✅ Active Blob Store (in `upload/blob_store.rs`)**

- **`STABLE_BLOB_STORE`** - `StableBTreeMap<(u64, u32), Vec<u8>, Memory>` (uses `MEM_BLOBS` = 6)
- **`STABLE_BLOB_META`** - `StableBTreeMap<u64, BlobMeta, Memory>` (uses `MEM_BLOB_META` = 7)
- **`STABLE_BLOB_COUNTER`** - `StableCell<u64, Memory>` (uses `MEM_BLOB_COUNTER` = 8)

### **4. ✅ Active CapsuleStore System (in `capsule_store/stable.rs`)**

- **`capsules`** - `StableBTreeMap<CapsuleId, Capsule, VirtualMemory>` (uses `MEM_CAPSULES` = 0)
- **`subject_index`** - `StableBTreeMap<Vec<u8>, CapsuleId, VirtualMemory>` (uses `MEM_CAPSULES_IDX_SUBJECT` = 1)
- **`owner_index`** - `StableBTreeMap<OwnerIndexKey, (), VirtualMemory>` (uses `MEM_CAPSULES_IDX_OWNER` = 2)

### **📊 Summary: All Stable Data Structures in Our Codebase**

**Total: 10 Stable Structures**

- **4 Dead Code** (in `memory.rs`) - **REMOVE IMMEDIATELY**
- **6 Active Code** (in working modules)

**ICP Stable Memory Primitives Available (ic-stable-structures v0.6.9):**

**We Currently Use:**

- **`StableBTreeMap<K, V, M>`** - Persistent key-value storage (8 structures)
- **`StableCell<T, M>`** - Persistent single value storage (2 structures)
- **`VirtualMemory<DefaultMemoryImpl>`** - Memory abstraction layer
- **`MemoryManager<DefaultMemoryImpl>`** - Memory ID management

**Other ICP Stable Structures Available (Not Used):**

- **`StableVec<T, M>`** - Persistent vector storage
- **`StableLog<T, M>`** - Persistent append-only log
- **`StableUnboundedMap<K, V, M>`** - Unbounded key-value storage
- **`StableUnboundedVec<T, M>`** - Unbounded vector storage
- **`StableMultimap<K, V, M>`** - Persistent multimap storage
- **`StableBTreeSet<T, M>`** - Persistent set storage

## ⚠️ CRITICAL: Memory ID Collision Analysis

### **Memory ID Constants (in `memory_manager.rs`)**

```rust
// Capsule storage (existing)
pub const MEM_CAPSULES: MemoryId = MemoryId::new(0);
pub const MEM_CAPSULES_IDX_SUBJECT: MemoryId = MemoryId::new(1);
pub const MEM_CAPSULES_IDX_OWNER: MemoryId = MemoryId::new(2);

// Upload workflow (new)
pub const MEM_SESSIONS: MemoryId = MemoryId::new(3);
pub const MEM_SESSIONS_CHUNKS: MemoryId = MemoryId::new(4);
pub const MEM_SESSIONS_COUNTER: MemoryId = MemoryId::new(5);

// Blob storage
pub const MEM_BLOBS: MemoryId = MemoryId::new(6);
pub const MEM_BLOB_META: MemoryId = MemoryId::new(7);
pub const MEM_BLOB_COUNTER: MemoryId = MemoryId::new(8);
```

### **🚨 CRITICAL COLLISIONS DETECTED:**

| Memory ID | Dead Code (memory.rs)     | Active Code                                   | Status           |
| --------- | ------------------------- | --------------------------------------------- | ---------------- |
| **0**     | `STABLE_CAPSULES`         | `capsules` (CapsuleStore)                     | ⚠️ **COLLISION** |
| **3**     | `STABLE_UPLOAD_SESSIONS`  | `STABLE_UPLOAD_SESSIONS` (upload/sessions.rs) | ⚠️ **COLLISION** |
| **4**     | `STABLE_MEMORY_ARTIFACTS` | `STABLE_CHUNK_DATA` (upload/sessions.rs)      | ⚠️ **COLLISION** |
| **5**     | `STABLE_CHUNK_DATA`       | `STABLE_SESSION_COUNTER` (upload/sessions.rs) | ⚠️ **COLLISION** |

**Note**: The active structures are still present in their respective files and using the correct memory IDs from `memory_manager.rs`.

### **🔥 IMMEDIATE ACTION REQUIRED:**

**The dead code in `memory.rs` is using the same Memory IDs as active code, creating a memory collision risk that could corrupt your data!**

## Key Insights

1. **Capsules use CapsuleStore** - The real capsule storage is in `capsule_store/stable.rs`, not the dead `STABLE_CAPSULES`
2. **Multiple storage systems** - There are overlapping storage systems that need cleanup
3. **Dead code exists** - `with_stable_capsules()` and related functions are unused
4. **Memory ID conflicts** - Some memory IDs are used by multiple systems

## Implementation Steps

1. **Fix the function** to use `with_capsule_store()` instead of `with_stable_capsules()`
2. **Test the function** to ensure it works correctly
3. **Consider cleanup** of dead stable storage code
4. **Add comprehensive stats** if needed for monitoring

## Files to Update

- `src/backend/src/memory.rs` - Fix the `get_stable_memory_stats()` function
- Consider removing dead `with_stable_capsules()` functions

## Priority

**High** - This function is likely used for monitoring and debugging, and it's currently broken.

## Timeline

**1 day** - Simple fix, just need to change one function call.

---

## Appendix: Complete Analysis of Stable Memory Structures

### **Purpose and Usage of Each Stable Structure**

#### **1. Capsule Storage**

- **Structure**: TWO separate systems using the SAME memory ID!
  - `STABLE_CAPSULES` (in `memory.rs`) - **DEAD CODE**
  - `CapsuleStore` (in `capsule_store/stable.rs`) - **ACTIVE CODE**
- **Purpose**: Store user capsules (personal data containers)
- **Who produces**: Capsule creation/management functions
- **Why needed**: Core data structure for the entire application
- **Status**: ⚠️ **MEMORY COLLISION RISK** - Both use `MemoryId::new(0)`

#### **2. Upload Sessions**

- **Structure**: `STABLE_UPLOAD_SESSIONS` (in `upload/sessions.rs`)
- **Purpose**: Track file upload sessions for chunked uploads
- **Who produces**: `UploadService::begin_upload()` function
- **Why needed**: Handle large file uploads in chunks with session management
- **Status**: ✅ **NECESSARY** - Required for file upload functionality

#### **3. Chunk Data**

- **Structure**: `STABLE_CHUNK_DATA` (in `upload/sessions.rs`)
- **Purpose**: Store individual chunks of files being uploaded
- **Who produces**: `UploadService::put_chunk()` function
- **Why needed**: Temporary storage for file chunks during upload
- **Status**: ✅ **NECESSARY** - Required for chunked uploads

#### **4. Memory Artifacts**

- **Structure**: `STABLE_MEMORY_ARTIFACTS` (in `memory.rs`)
- **Purpose**: Store metadata artifacts for memories (JSON metadata)
- **Who produces**: `metadata::upsert_metadata()` function
- **Why needed**: Store metadata separately from memory content
- **Status**: ❓ **QUESTIONABLE** - Could be merged with capsule storage

#### **5. Blob Store**

- **Structure**: `STABLE_BLOB_STORE` (in `upload/blob_store.rs`)
- **Purpose**: Store large files in paged format
- **Who produces**: `BlobStore::put_inline()` and `BlobStore::store_from_chunks()`
- **Why needed**: Handle large files that don't fit in memory
- **Status**: ✅ **NECESSARY** - Required for large file storage

#### **6. Blob Metadata**

- **Structure**: `STABLE_BLOB_META` (in `upload/blob_store.rs`)
- **Purpose**: Store metadata for blobs (size, checksum, creation time)
- **Who produces**: `BlobStore` when storing blobs
- **Why needed**: Track blob information without reading content
- **Status**: ✅ **NECESSARY** - Required for blob management

#### **7. Session Counter**

- **Structure**: `STABLE_SESSION_COUNTER` (in `upload/sessions.rs`)
- **Purpose**: Generate unique session IDs
- **Who produces**: Session creation functions
- **Why needed**: Ensure unique session identifiers
- **Status**: ✅ **NECESSARY** - Required for session management

#### **8. Blob Counter**

- **Structure**: `STABLE_BLOB_COUNTER` (in `upload/blob_store.rs`)
- **Purpose**: Generate unique blob IDs
- **Who produces**: Blob creation functions
- **Why needed**: Ensure unique blob identifiers
- **Status**: ✅ **NECESSARY** - Required for blob management

### **Evaluation: Which Structures Are Necessary?**

#### **✅ NECESSARY (Keep These)**

1. **CapsuleStore system** - Core application data (ACTIVE)
2. **Upload Sessions** - File upload management
3. **Chunk Data** - Temporary upload storage
4. **Blob Store** - Large file storage
5. **Blob Metadata** - Blob management
6. **Session Counter** - Unique session IDs
7. **Blob Counter** - Unique blob IDs

#### **❓ QUESTIONABLE (Consider Merging)**

1. **Memory Artifacts** - Could be stored within capsules instead of separate structure

#### **❌ DEAD CODE (Remove These)**

1. **`STABLE_CAPSULES`** (in `memory.rs`) - Dead code, replaced by CapsuleStore system
2. **`STABLE_CHUNK_DATA`** (in `memory.rs`) - Duplicate of the one in `upload/sessions.rs`

#### **⚠️ CRITICAL ISSUE**

1. **Memory ID Collision** - Both `STABLE_CAPSULES` and `CapsuleStore` use `MemoryId::new(0)`

### **Recommendations**

#### **1. Remove Dead Code (CRITICAL)**

- **URGENT**: Delete `STABLE_CAPSULES` and related functions in `memory.rs` - **MEMORY COLLISION RISK**
- Delete duplicate `STABLE_CHUNK_DATA` in `memory.rs`
- Keep only the working versions in their respective modules

#### **2. Consider Memory Artifacts Consolidation**

- **Option A**: Keep separate (current approach) - Good for metadata-only operations
- **Option B**: Merge into capsules - Simpler architecture, but less efficient for metadata queries

#### **3. Memory ID Cleanup**

- Remove unused memory IDs from `memory_manager.rs`
- Ensure all remaining IDs are properly documented

### **Current Memory ID Usage**

```rust
// ACTIVE (Keep)
pub const MEM_CAPSULES: MemoryId = MemoryId::new(0);              // CapsuleStore
pub const MEM_CAPSULES_IDX_SUBJECT: MemoryId = MemoryId::new(1);  // CapsuleStore
pub const MEM_CAPSULES_IDX_OWNER: MemoryId = MemoryId::new(2);    // CapsuleStore
pub const MEM_SESSIONS: MemoryId = MemoryId::new(3);              // Upload sessions
pub const MEM_SESSIONS_CHUNKS: MemoryId = MemoryId::new(4);       // Chunk data
pub const MEM_SESSIONS_COUNTER: MemoryId = MemoryId::new(5);      // Session counter
pub const MEM_BLOBS: MemoryId = MemoryId::new(6);                 // Blob store
pub const MEM_BLOB_META: MemoryId = MemoryId::new(7);             // Blob metadata
pub const MEM_BLOB_COUNTER: MemoryId = MemoryId::new(8);          // Blob counter

// DEAD CODE (Remove)
// CAPSULES_MEMORY_ID = 0 (collision with MEM_CAPSULES)
// UPLOAD_SESSIONS_MEMORY_ID = 3 (collision with MEM_SESSIONS)
// MEMORY_ARTIFACTS_MEMORY_ID = 4 (collision with MEM_SESSIONS_CHUNKS)
// CHUNK_DATA_MEMORY_ID = 5 (collision with MEM_SESSIONS_COUNTER)
```

### **Final Assessment**

The stable memory architecture has **CRITICAL MEMORY COLLISION ISSUES** that need immediate attention:

1. **🚨 4 MEMORY ID COLLISIONS** - Dead code using same Memory IDs as active code
2. **⚠️ DATA CORRUPTION RISK** - Multiple structures competing for same memory regions
3. **Dead code** that needs immediate removal
4. **Memory artifacts** that could be consolidated
5. **Duplicate structures** that need removal

**🔥 URGENT PRIORITY**: Remove ALL dead code in `memory.rs` immediately to prevent memory corruption:

- `STABLE_CAPSULES` (collision with CapsuleStore)
- `STABLE_UPLOAD_SESSIONS` (collision with subject_index)
- `STABLE_MEMORY_ARTIFACTS` (collision with owner_index)
- `STABLE_CHUNK_DATA` (collision with upload/sessions.rs)

**This is a critical bug that could corrupt your production data!**

Alright — here’s how I’d expose **stats accessors** so your `get_stable_memory_stats()` can stay clean and decoupled. No internal thread-locals leaking out, just public methods on the right modules.

---

### 1. CapsuleStore (`capsule_store/stable.rs`)

You already have a trait + backends. Just add:

```rust
impl StableCapsuleStore {
    pub fn capsule_count(&self) -> u64 {
        self.capsules.len() as u64
    }

    pub fn subject_index_count(&self) -> u64 {
        self.subject_index.len() as u64
    }

    pub fn owner_index_count(&self) -> u64 {
        self.owner_index.len() as u64
    }
}
```

---

### 2. SessionStore (`upload/sessions.rs`)

Add public accessors to count sessions and chunks:

```rust
impl SessionStore {
    pub fn session_count(&self) -> u64 {
        STABLE_UPLOAD_SESSIONS.with(|s| s.borrow().len() as u64)
    }

    pub fn chunk_count(&self) -> u64 {
        STABLE_CHUNK_DATA.with(|c| c.borrow().len() as u64)
    }
}
```

---

### 3. BlobStore (`upload/blob_store.rs`)

Expose counts for blobs and metadata:

```rust
impl BlobStore {
    pub fn blob_count(&self) -> u64 {
        STABLE_BLOB_STORE.with(|s| s.borrow().len() as u64)
    }

    pub fn blob_meta_count(&self) -> u64 {
        STABLE_BLOB_META.with(|m| m.borrow().len() as u64)
    }
}
```

---

### 4. Global `get_stable_memory_stats()`

Then your high-level stats function is just orchestration:

```rust
pub fn get_stable_memory_stats() -> (u64, u64, u64, u64, u64, u64, u64, u64) {
    with_capsule_store(|s| {
        let cap_count = s.capsule_count();
        let subj_idx = s.subject_index_count();
        let owner_idx = s.owner_index_count();

        let sessions = SessionStore::session_count();
        let chunks = SessionStore::chunk_count();

        let blobs = BlobStore::blob_count();
        let blob_meta = BlobStore::blob_meta_count();

        // counters (cells) you can read directly too
        let session_id_counter = STABLE_SESSION_COUNTER.with(|c| c.borrow().get());
        let blob_id_counter = STABLE_BLOB_COUNTER.with(|c| c.borrow().get());

        (
            cap_count, subj_idx, owner_idx,
            sessions, chunks,
            blobs, blob_meta,
            session_id_counter + blob_id_counter
        )
    })
}
```

---

### Why this way?

- Each module owns its own stable structures.
- Stats only require **one public method per module**.
- `get_stable_memory_stats()` stays a thin coordinator.
- Future additions (new IDs) are easy: just add a new accessor and update the tuple.

---

Do you want me to also propose a **canonical struct** (instead of a long tuple) so your stats are self-describing when you query them? For MVP it’s optional, but it prevents “what was the 3rd field again?” later.

Great—let’s switch to a **named struct** for clarity.

# What to return

Use a single, self-describing type:

```rust
// Put this in `types.rs` (or a `stats.rs` if you prefer), so it’s easy to import.
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct StableMemoryStats {
    pub capsule_count: u64,
    pub subject_index_count: u64,
    pub owner_index_count: u64,

    pub session_count: u64,
    pub chunk_count: u64,

    pub blob_count: u64,
    pub blob_meta_count: u64,

    pub session_id_counter: u64,
    pub blob_id_counter: u64,
}
```

# How to build it (high-level)

Keep the stats **decoupled** by calling small accessors from each module:

- `with_capsule_store(|s| s.capsule_count())`
- `with_capsule_store(|s| s.subject_index_count())`
- `with_capsule_store(|s| s.owner_index_count())`
- `SessionStore::session_count()`
- `SessionStore::chunk_count()`
- `BlobStore::blob_count()`
- `BlobStore::blob_meta_count()`
- `STABLE_SESSION_COUNTER.with(|c| c.borrow().get())`
- `STABLE_BLOB_COUNTER.with(|c| c.borrow().get())`

Return `StableMemoryStats` from your façade endpoint (e.g., `#[ic_cdk::query] fn get_stable_memory_stats() -> StableMemoryStats`).

# Where to put things

- **Struct**: `types.rs` (keeps your API shapes centralized).
- **Accessors**: in their respective modules (`capsule_store`, `upload/sessions.rs`, `upload/blob_store.rs`).
- **Façade**: `lib.rs` exposes `get_stable_memory_stats()` and just assembles the struct via those accessors.

# Why this is better (MVP-friendly)

- Human-readable output (no guessing tuple positions).
- Easy to evolve (add a field without breaking existing ones).
- Keeps module boundaries clean (the stats function doesn’t reach into thread-locals directly).

If you want, I can list the exact accessor function names to add in each module (just signatures) so your team can implement them quickly.
