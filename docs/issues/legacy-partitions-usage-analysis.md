# Legacy Partitions Usage Analysis

## Problem Statement

We have 4 legacy memory partitions in `memory.rs` that are marked as "to be removed" but we need to verify if they're actually being used by any functions in the codebase before removing them.

## Legacy Partitions to Analyze

### **1. STABLE_CAPSULES**

```rust
static STABLE_CAPSULES: RefCell<StableBTreeMap<String, Capsule, Memory>> = RefCell::new(
    StableBTreeMap::init(
        MM.with(|m| m.borrow().get(CAPSULES_MEMORY_ID))  // Uses ID 0
    )
);
```

### **2. STABLE_UPLOAD_SESSIONS**

```rust
static STABLE_UPLOAD_SESSIONS: RefCell<StableBTreeMap<String, UploadSession, Memory>> = RefCell::new(
    StableBTreeMap::init(
        MM.with(|m| m.borrow().get(UPLOAD_SESSIONS_MEMORY_ID))  // Uses ID 3
    )
);
```

### **3. STABLE_MEMORY_ARTIFACTS**

```rust
static STABLE_MEMORY_ARTIFACTS: RefCell<StableBTreeMap<String, MemoryArtifact, Memory>> = RefCell::new(
    StableBTreeMap::init(
        MM.with(|m| m.borrow().get(MEMORY_ARTIFACTS_MEMORY_ID))  // Uses ID 4
    )
);
```

### **4. STABLE_CHUNK_DATA**

```rust
static STABLE_CHUNK_DATA: RefCell<StableBTreeMap<String, ChunkData, Memory>> = RefCell::new(
    StableBTreeMap::init(
        MM.with(|m| m.borrow().get(CHUNK_DATA_MEMORY_ID))  // Uses ID 5
    )
);
```

## Research Results

### **STABLE_CAPSULES Usage**

- **Direct references**: 3 in memory.rs
- **Access function references**: 7 total
- **Used by**:
  - `get_stable_memory_stats()` - Gets capsule count
  - `migrate_capsules_to_stable()` - Migration helper
  - Test functions in memory.rs
- **Status**: ✅ **USED**

### **STABLE_UPLOAD_SESSIONS Usage**

- **Direct references**: 11 total (3 in memory.rs, 8 in upload/sessions.rs)
- **Access function references**: 12 total
- **Used by**:
  - `get_stable_memory_stats()` - Gets session count
  - `SessionStore` functions (create, get, update, cleanup, find_pending, count_active_for)
  - `check_active_sessions()` in auth.rs
  - Test functions in memory.rs
- **Status**: ✅ **USED**

### **STABLE_MEMORY_ARTIFACTS Usage**

- **Direct references**: 4 in memory.rs
- **Access function references**: 13 total
- **Used by**:
  - `get_stable_memory_stats()` - Gets artifact count
  - `upsert_metadata()`, `get_metadata()`, `delete_metadata()`, `get_metadata_status()` in metadata.rs
  - Test functions in memory.rs
- **Status**: ✅ **USED**

### **STABLE_CHUNK_DATA Usage**

- **Direct references**: 10 total (4 in memory.rs, 6 in upload/sessions.rs)
- **Access function references**: 2 (only definitions, no actual usage)
- **Used by**:
  - `SessionStore` functions (put_chunk, get_chunk, verify_chunks_complete, cleanup)
  - `ChunkIterator::next()`
  - Test functions in memory.rs
- **Status**: ✅ **USED**

## Summary

### **Safe to Remove**

- [ ] **NONE** - All structures are actively used

### **Need Migration**

- [x] STABLE_CAPSULES → CapsuleStore (already has replacement)
- [x] STABLE_UPLOAD_SESSIONS → upload/sessions.rs (already has replacement)
- [ ] STABLE_MEMORY_ARTIFACTS → **NO REPLACEMENT** (actively used in metadata.rs)
- [x] STABLE_CHUNK_DATA → upload/sessions.rs (already has replacement)

### **Legacy Constants to Remove**

- [ ] `CAPSULES_MEMORY_ID`
- [ ] `UPLOAD_SESSIONS_MEMORY_ID`
- [ ] `MEMORY_ARTIFACTS_MEMORY_ID`
- [ ] `CHUNK_DATA_MEMORY_ID`

## Key Insight: Two Different Storage Systems

**Important Discovery**: We have **two different capsule storage systems** running in parallel:

### **1. Active CapsuleStore (in `capsule_store/stable.rs`):**

```rust
pub struct StableStore {
    capsules: StableBTreeMap<CapsuleId, Capsule, VirtualMemory<DefaultMemoryImpl>>,
    // Uses: MM.with(|m| m.borrow().get(MEM_CAPSULES))  // Memory ID 0
}
```

### **2. Legacy STABLE_CAPSULES (in `memory.rs`):**

```rust
static STABLE_CAPSULES: RefCell<StableBTreeMap<String, Capsule, Memory>> = RefCell::new(
    StableBTreeMap::init(
        MM.with(|m| m.borrow().get(CAPSULES_MEMORY_ID))  // Memory ID 0
    )
);
```

### **The Problem:**

Both are using **Memory ID 0**, but they're **different data structures**:

- **CapsuleStore**: `StableBTreeMap<CapsuleId, Capsule, ...>` (uses `CapsuleId` as key)
- **Legacy**: `StableBTreeMap<String, Capsule, ...>` (uses `String` as key)

### **The Solution:**

**REMOVE THE LEGACY ONE** - The `STABLE_CAPSULES` in `memory.rs` is useless and should be deleted. We only need the active `CapsuleStore` in `capsule_store/stable.rs`.

**Action Plan:**

1. **Delete** `STABLE_CAPSULES` from `memory.rs`
2. **Delete** `CAPSULES_MEMORY_ID` constant
3. **Delete** all access functions (`with_stable_capsules`, etc.)
4. **Update** `get_stable_memory_stats()` to use `CapsuleStore` instead
5. **Update** any test code to use `CapsuleStore`

**No collision problem** - we're just removing the dead code.
