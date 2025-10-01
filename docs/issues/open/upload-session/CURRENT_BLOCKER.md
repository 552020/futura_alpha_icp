# Current Blocker - Stable Storage Not Persisting

**Date**: 2025-10-01 21:05  
**Status**: 🔴 **CRITICAL BLOCKER** - Chunks write but can't be read back  
**Progress**: 4/5 fixes complete, rolling hash implemented

---

## 🎯 What We've Implemented

### ✅ Completed (4/5 Critical Fixes)

1. **✅ Fix #1: `bytes_expected` source of truth**
   - Uses `meta.asset_metadata.get_base().bytes`
2. **✅ Fix #2: Deterministic SHA256 keys**
   - Created unified `pmid_hash32()` function
   - Changed `STABLE_BLOB_STORE` key type to `([u8; 32], u32)`
   - All read/write paths use same keys
3. **✅ Fix #3: Rolling hash**
   - Added `UPLOAD_HASH` thread-local storage
   - Hash updates in `uploads_put_chunk()`
   - Hash verified in `uploads_finish()` BEFORE reading chunks
4. **✅ Fix #4: Box sink_factory**
   - Was already correct

### ⏭️ Remaining

5. **⏭️ Fix #5: Atomic index commit**
   - Needs verification after blocker resolved

---

## 🔴 THE BLOCKER

### Symptom

Chunks are **written successfully** but **cannot be read back** in the same `finish()` call:

```
[BLOB_WRITE chunk_idx=0 len=1800000 pmid_hash=[227, 176, 196, 66, 152, 252, 28, 20]]
[BLOB_WRITE chunk_idx=1 len=1800000 pmid_hash=[227, 176, 196, 66, 152, 252, 28, 20]]
...
[BLOB_WRITE chunk_idx=12 len=227484 pmid_hash=[227, 176, 196, 66, 152, 252, 28, 20]]

[FINISH_HASH_OK sid=18 len=21827484]  ← Rolling hash passed!
[COMMIT: sid=18 chunks_verified]       ← Chunk count verified!

[BLOB_READ sid=18 chunk_idx=0 found=false len=0 pmid_hash=[227, 176, 196, 66, 152, 252, 28, 20]]
[BLOB_READ_NOTFOUND sid=18 chunk_idx=0 pmid_hash=[227, 176, 196, 66, 152, 252, 28, 20]]
[FINISH_ERROR sid=18 err=NotFound]
```

**Key observation**: SAME `pmid_hash` used for write and read, but chunks not found!

---

## 🔍 Root Cause Analysis

### What's Happening

1. ✅ `StableBlobSink::write_at()` called → writes to `STABLE_BLOB_STORE`
2. ✅ Logging confirms write with `pmid_hash=[227, 176, 196, 66, 152, 252, 28, 20]`
3. ✅ `verify_chunks_complete()` passes (checks `received_idxs`)
4. ❌ `store_from_chunks()` tries to read → chunks NOT FOUND
5. ❌ Same `pmid_hash` but data disappeared!

### Possible Causes

#### A) StableBTreeMap Not Actually Persisting Writes ⚠️

The `StableBTreeMap::init()` creates the map, but writes might not be flushing to stable memory properly.

**Evidence**:

- Writes happen in `put_chunk()` calls (separate messages)
- Reads happen in `finish()` call (different message)
- Data doesn't persist between messages

**Code**:

```rust
thread_local! {
    static STABLE_BLOB_STORE: RefCell<StableBTreeMap<([u8; 32], u32), Vec<u8>, Memory>> = RefCell::new(
        StableBTreeMap::init(MM.with(|m| m.borrow().get(MEM_BLOBS)))
    );
}
```

#### B) Key Type Change Corrupted Memory 🎯 LIKELY

When we changed the key type from `(u64, u32)` to `([u8; 32], u32)`, the underlying stable memory structure might be corrupted because:

- Old data written with `(u64, u32)` keys
- New code tries to read with `([u8; 32], u32)` keys
- `StableBTreeMap` internal structure is inconsistent

**We tried**: `dfx canister uninstall-code backend` to clear memory
**Result**: Still failing (but fewer logs, so might need more testing)

#### C) Memory Manager Issue

The memory manager might not be properly initialized or the memory ID might be wrong.

---

## 📋 Next Steps to Debug

### Option 1: Add More Logging to StableBTreeMap Operations

```rust
STABLE_BLOB_STORE.with(|store| {
    let mut store = store.borrow_mut();
    ic_cdk::println!("BLOB_STORE_INSERT: key=({:?}, {}) len={}",
        &self.pmid_hash[..8], chunk_idx, data.len());
    store.insert((self.pmid_hash, chunk_idx), data.to_vec());

    // Immediately read back to verify
    if let Some(readback) = store.get(&(self.pmid_hash, chunk_idx)) {
        ic_cdk::println!("BLOB_STORE_VERIFY: readback successful len={}", readback.len());
    } else {
        ic_cdk::println!("BLOB_STORE_VERIFY: FAILED - can't read immediately after insert!");
    }
});
```

### Option 2: Test With Simpler Key Type

Temporarily change back to `(u64, u32)` to see if the issue is specific to the `([u8; 32], u32)` key type.

### Option 3: Check StableBTreeMap Size

Add a query to check how many entries are in `STABLE_BLOB_STORE`:

```rust
#[ic_cdk::query]
fn debug_blob_store_size() -> u64 {
    STABLE_BLOB_STORE.with(|store| store.borrow().len())
}
```

### Option 4: Use Different Memory for Testing

Create a completely new memory ID to ensure no corruption from old data.

---

## 💡 Recommended Action

**IMMEDIATE**: Add read-back verification right after write to see if the issue is:

- Write not actually persisting to `StableBTreeMap`
- OR read happening from a different `StableBTreeMap` instance

**Code to add** in `StableBlobSink::write_at_impl()` after line 564:

```rust
// Store chunk
STABLE_BLOB_STORE.with(|store| {
    let mut store = store.borrow_mut();
    store.insert((self.pmid_hash, chunk_idx), data.to_vec());
});

// IMMEDIATE READBACK TEST
let verify = STABLE_BLOB_STORE.with(|store| {
    store.borrow().get(&(self.pmid_hash, chunk_idx)).map(|d| d.len())
});

match verify {
    Some(len) if len == data.len() => {
        ic_cdk::println!("BLOB_VERIFY_OK chunk_idx={} len={}", chunk_idx, len);
    }
    Some(len) => {
        ic_cdk::println!("BLOB_VERIFY_SIZE_MISMATCH chunk_idx={} wrote={} read={}",
            chunk_idx, data.len(), len);
    }
    None => {
        ic_cdk::println!("BLOB_VERIFY_FAILED chunk_idx={} - NOT FOUND IMMEDIATELY AFTER WRITE!",
            chunk_idx);
    }
}
```

This will tell us if the problem is:

- ❌ Write not working → will see `BLOB_VERIFY_FAILED`
- ❌ Read from different instance → will see success in write but fail in finish
- ✅ Both work → problem is elsewhere

---

## 📊 Current Test Results

| Test                        | Status | Issue                     |
| --------------------------- | ------ | ------------------------- |
| Lane A (single 21MB upload) | ❌     | finish() returns NotFound |
| Lane B (image processing)   | ✅     | Works (no upload)         |
| Parallel Lanes              | ❌     | Multiple uploads fail     |
| Complete 2-Lane + 4-Asset   | ❌     | Uploads fail              |
| Asset Retrieval             | ❌     | Uploads fail              |

**Success Rate**: 1/5 (20%)  
**Blocker**: Cannot read chunks from stable storage

---

## 🎯 Goal

Get chunks to persist in `STABLE_BLOB_STORE` so they can be read back in `store_from_chunks()`.

**When this works**, we expect:

- Rolling hash ✅ (already works)
- Chunk verification ✅ (already works)
- Chunk readback ✅ (currently blocked)
- Upload finish ✅ (will work after readback fixes)

---

**Status**: 🔴 **BLOCKED** - Need to debug StableBTreeMap persistence  
**Next Action**: Add immediate read-back verification after write  
**ETA**: Unknown until blocker resolved

---

**Created**: 2025-10-01 21:05  
**Blocker**: StableBTreeMap writes not persisting or reads using wrong instance  
**Action**: Add verification logging to pinpoint exact issue
