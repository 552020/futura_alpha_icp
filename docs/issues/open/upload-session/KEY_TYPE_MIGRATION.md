# Key Type Migration - Need Tech Lead Guidance

**Date**: 2025-10-01 23:00  
**Status**: 🔴 **BLOCKED** - Compilation errors from key type change  
**Priority**: CRITICAL - Blocking Fix #2 completion

---

## TL;DR

Following your recommendation, we changed `STABLE_BLOB_STORE` key from `(u64, u32)` to `([u8; 32], u32)` for deterministic hashing. The new upload flow (write_at/store_from_chunks) now uses deterministic SHA256, but old blob management methods still use u64 keys and have compilation errors.

**Need**: Exact code for migrating/updating these 4 methods OR permission to disable them temporarily.

---

## ✅ What We Fixed

### 1. StableBlobSink (Write Path) ✅

```rust
pub struct StableBlobSink {
    pmid_hash: [u8; 32], // SHA256 of provisional_memory_id
    chunk_size: usize,
}

impl ByteSink for StableBlobSink {
    fn write_at(&mut self, offset: u64, data: &[u8]) -> Result<(), Error> {
        let chunk_idx = (offset / self.chunk_size as u64) as u32;
        STABLE_BLOB_STORE.with(|store| {
            store.borrow_mut().insert((self.pmid_hash, chunk_idx), data.to_vec());
        });
        Ok(())
    }
}
```

**Status**: ✅ Uses deterministic `pmid_hash: [u8; 32]`

### 2. store_from_chunks (Read Path) ✅

```rust
pub fn store_from_chunks(...) -> Result<BlobId, Error> {
    // Derive pmid_hash EXACTLY like StableBlobSink
    let pmid_hash: [u8; 32] = {
        let mut h = Sha256::new();
        h.update(session_meta.provisional_memory_id.as_bytes());
        h.finalize().into()
    };

    // Create blob_id from first 8 bytes for metadata storage
    let blob_id = BlobId(u64::from_be_bytes([...]));

    for page_idx in 0..chunk_count {
        let page_key = (pmid_hash, page_idx); // ← Uses [u8; 32] key
        let chunk_data = STABLE_BLOB_STORE.with(|store| store.borrow().get(&page_key));
        hasher.update(&chunk_data);
    }
}
```

**Status**: ✅ Uses deterministic `pmid_hash: [u8; 32]`

### 3. STABLE_BLOB_STORE Type Changed ✅

```rust
thread_local! {
    // CHANGED: Key is now ([u8; 32], u32) instead of (u64, u32)
    static STABLE_BLOB_STORE: RefCell<StableBTreeMap<([u8; 32], u32), Vec<u8>, Memory>> = ...
}
```

**Status**: ✅ Type changed to `([u8; 32], u32)`

---

## ❌ Compilation Errors (4 Methods)

### Error Pattern

```
error[E0308]: mismatched types
expected `&([u8; 32], u32)`, found `&(u64, u32)`
```

### Affected Methods

#### 1. `read_blob()` - Line 152

```rust
pub fn read_blob(&self, blob_id: &BlobId) -> Result<Vec<u8>, Error> {
    loop {
        let page_key = (blob_id.0, page_idx); // ← ERROR: u64, needs [u8; 32]
        let page_data = STABLE_BLOB_STORE.with(|store| store.borrow().get(&page_key));
    }
}
```

**Used by**: Old blob retrieval (not used by new upload flow)

#### 2. `delete_blob()` - Line 181

```rust
pub fn delete_blob(&self, blob_id: &BlobId) -> Result<(), Error> {
    loop {
        let page_key = (blob_id.0, page_idx); // ← ERROR: u64, needs [u8; 32]
        let removed = STABLE_BLOB_STORE.with(|store| store.borrow_mut().remove(&page_key));
    }
}
```

**Used by**: Blob cleanup (not used by new upload flow, but called in store_from_chunks on hash mismatch)

#### 3. `read_blob_chunk()` - Line 286

```rust
fn read_blob_chunk(blob_id: &BlobId, chunk_index: u32) -> Result<Vec<u8>, Error> {
    let page_key = (blob_id.0, chunk_index); // ← ERROR: u64, needs [u8; 32]
    let page_data = STABLE_BLOB_STORE.with(|store| store.borrow().get(&page_key));
}
```

**Used by**: Tests

#### 4. `get_blob_chunk_count()` - Line 346

```rust
fn get_blob_chunk_count(blob_id: &BlobId) -> u32 {
    loop {
        let page_key = (blob_id.0, chunk_count); // ← ERROR: u64, needs [u8; 32]
        let exists = STABLE_BLOB_STORE.with(|store| store.borrow().contains_key(&page_key));
    }
}
```

**Used by**: Tests

---

## 🤔 Questions for Tech Lead

### Q1: How to Fix These Methods?

**Option A**: Store `pmid_hash` in `BlobMeta` and use that for lookups

```rust
pub struct BlobMeta {
    size: u64,
    checksum: [u8; 32],
    created_at: u64,
    pmid_hash: [u8; 32], // ← Add this
}

// Then in read_blob():
pub fn read_blob(&self, blob_id: &BlobId) -> Result<Vec<u8>, Error> {
    let meta = STABLE_BLOB_META.with(|metas| metas.borrow().get(&blob_id.0))?;
    let pmid_hash = meta.pmid_hash; // ← Use stored hash

    loop {
        let page_key = (pmid_hash, page_idx); // ← Now correct type
        // ...
    }
}
```

**Option B**: Create reverse mapping `blob_id → pmid_hash`

```rust
thread_local! {
    static BLOB_ID_TO_HASH: RefCell<BTreeMap<u64, [u8; 32]>> = ...;
}
```

**Option C**: Disable these methods temporarily (tests only)

```rust
#[cfg(test)]
pub fn read_blob(...) { unimplemented!("Migrating to new key type") }
```

**Which approach do you recommend?**

### Q2: Is This Breaking Change Acceptable?

The key type change from `(u64, u32)` to `([u8; 32], u32)` means:

- ✅ **New uploads** will work with deterministic keys
- ❌ **Old blobs** (if any exist) will be inaccessible

**Is this acceptable for current development** stage, or do we need migration?

### Q3: Should We Store pmid_hash in store_from_chunks?

Currently we derive blob_id from first 8 bytes of pmid_hash:

```rust
let blob_id = BlobId(u64::from_be_bytes([
    pmid_hash[0], pmid_hash[1], pmid_hash[2], pmid_hash[3],
    pmid_hash[4], pmid_hash[5], pmid_hash[6], pmid_hash[7],
]));
```

**Should we**:

- Store full `pmid_hash` in `BlobMeta` for later retrieval?
- Or is the u64 derivation sufficient?

---

## 📋 Recommended Quick Fix

**Our recommendation**: Add `pmid_hash` to `BlobMeta` (Option A)

**Pros**:

- Clean solution
- All methods can access pmid_hash
- No reverse mapping needed

**Cons**:

- Adds 32 bytes to each BlobMeta
- Need to update store_from_chunks to save it

**Implementation** (15 minutes):

```rust
// 1. Add to BlobMeta
pub struct BlobMeta {
    size: u64,
    checksum: [u8; 32],
    created_at: u64,
    pmid_hash: [u8; 32], // ← NEW
}

// 2. Save in store_from_chunks
let meta = BlobMeta {
    size: total_written,
    checksum: actual_hash,
    created_at: ic_cdk::api::time(),
    pmid_hash, // ← Save it
};

// 3. Use in read_blob/delete_blob
let meta = STABLE_BLOB_META.with(|metas| metas.borrow().get(&blob_id.0))?;
for page_idx in 0..num_chunks {
    let page_key = (meta.pmid_hash, page_idx); // ← Use stored hash
    // ...
}
```

---

## 🎯 Impact on Testing

**Current state**:

- ✅ New upload flow (write_at, store_from_chunks) will work
- ❌ Old blob methods don't compile
- ❌ Tests that use old methods will fail

**After fix**:

- ✅ All methods compile
- ✅ Tests pass
- ✅ Can run E2E tests

---

## 📊 Files Modified

1. ✅ `src/backend/src/upload/blob_store.rs:15` - Changed STABLE_BLOB_STORE key type
2. ✅ `src/backend/src/upload/blob_store.rs:510-532` - StableBlobSink uses pmid_hash
3. ✅ `src/backend/src/upload/blob_store.rs:534-565` - write_at uses pmid_hash key
4. ✅ `src/backend/src/upload/blob_store.rs:66-77` - store_from_chunks derives pmid_hash
5. ❌ `src/backend/src/upload/blob_store.rs:152,181,286,346` - Need fixes

---

## ⏱️ Time Estimate

**Option A (Add pmid_hash to BlobMeta)**: 15-30 minutes

- Update BlobMeta struct
- Update store_from_chunks to save it
- Update 4 methods to use it
- Test compilation

**Option B (Reverse mapping)**: 30-45 minutes  
**Option C (Disable methods)**: 5 minutes (but tests won't run)

---

**Status**: 🔴 **BLOCKED** - Need guidance on which option  
**Recommendation**: Option A (add pmid_hash to BlobMeta)  
**Next Step**: Awaiting tech lead decision  
**Impact**: Blocks E2E testing until resolved

---

**Created**: 2025-10-01 23:00  
**Context**: Implementing tech lead's deterministic key recommendation
