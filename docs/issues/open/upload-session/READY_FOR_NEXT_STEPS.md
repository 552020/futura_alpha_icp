# Ready for Next Steps - Deterministic Keys Complete

**Date**: 2025-10-01 23:30  
**Status**: ✅ **CRITICAL FIX COMPLETE** - Deterministic SHA256 keys unified  
**Next**: Add logging + rolling hash (30-45 minutes)

---

## ✅ COMPLETED: Deterministic Key Unification

### What We Fixed

**Created unified `pmid_hash32()` function** that ALL code now uses:

```rust
/// Deterministic hash of provisional_memory_id for stable chunk keys
/// CRITICAL: This MUST be used everywhere chunks are written/read
pub fn pmid_hash32(pmid: &str) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(pmid.as_bytes());
    h.finalize().into()
}
```

**Location**: `src/backend/src/upload/blob_store.rs:12-18`

### All Usage Points Updated

1. ✅ **StableBlobSink::for_meta()** - Uses `pmid_hash32()`
2. ✅ **store_from_chunks()** - Uses `pmid_hash32()`
3. ✅ **read_blob()** - Uses stored `pmid_hash` from BlobMeta
4. ✅ **delete_blob()** - Uses stored `pmid_hash` from BlobMeta
5. ✅ **read_blob_chunk()** - Uses stored `pmid_hash` from BlobMeta
6. ✅ **blob_get_meta()** - Uses stored `pmid_hash` from BlobMeta

**NO MORE `DefaultHasher` ANYWHERE!** 🎉

### Compilation Status

```bash
✅ cargo build --target wasm32-unknown-unknown --release -p backend
    Finished `release` profile [optimized] target(s) in 10.51s
```

**0 errors, ready to deploy!**

---

## 🎯 Next Steps (Tech Lead's Action Plan)

### Step 1: Add Structured Logging (15 min) ⏭️

```rust
// In src/backend/src/lib.rs::uploads_finish()
#[ic_cdk::update]
async fn uploads_finish(session_id: u64, expected_sha256: Vec<u8>, total_len: u64) -> Result_15 {
    ic_cdk::println!("FINISH_START sid={}", session_id);

    let hash: [u8; 32] = match expected_sha256.clone().try_into() {
        Ok(h) => h,
        Err(_) => {
            ic_cdk::println!("FINISH_ERROR sid={} err=invalid_hash_length", session_id);
            return Result_15::Err(Error::InvalidArgument(...));
        }
    };

    let result = memory::with_capsule_store_mut(|store| {
        let mut upload_service = upload::service::UploadService::new();
        let session_id = upload::types::SessionId(session_id);
        upload_service.commit(store, session_id, hash, total_len)
    });

    match result {
        Ok((blob_id, memory_id)) => {
            ic_cdk::println!("FINISH_OK sid={} blob={} mem={}", session_id, blob_id, memory_id);
            Result_15::Ok(UploadFinishResult { ... })
        }
        Err(e) => {
            ic_cdk::println!("FINISH_ERROR sid={} err={:?}", session_id, e);
            Result_15::Err(e)
        }
    }
}
```

**Then check logs**:

```bash
dfx canister logs backend | egrep 'FINISH_|ERROR|Checksum'
```

### Step 2: Implement Rolling Hash (30 min) ⏭️

```rust
// In src/backend/src/lib.rs (top of file)
use sha2::{Digest, Sha256};
use std::{cell::RefCell, collections::BTreeMap};

thread_local! {
    static UPLOAD_HASH: RefCell<BTreeMap<u64, Sha256>> = RefCell::new(BTreeMap::new());
}

// In uploads_begin()
#[ic_cdk::update]
fn uploads_begin(...) -> Result_13 {
    // ... existing code ...
    let sid = /* session_id value */;

    // Initialize rolling hash
    UPLOAD_HASH.with(|m| { m.borrow_mut().insert(sid, Sha256::new()); });

    Ok(Result_13::Ok(sid))
}

// In uploads_put_chunk()
#[ic_cdk::update]
async fn uploads_put_chunk(session_id: u64, chunk_idx: u32, bytes: Vec<u8>) -> Result<(), Error> {
    // Update rolling hash FIRST
    UPLOAD_HASH.with(|m| {
        m.borrow_mut()
            .get_mut(&session_id)
            .ok_or(Error::NotFound)?
            .update(&bytes);
        Ok::<(), Error>(())
    })?;

    // Then write chunk
    memory::with_capsule_store_mut(|store| {
        let mut upload_service = upload::service::UploadService::new();
        let session_id = upload::types::SessionId(session_id);
        upload_service.put_chunk(store, &session_id, chunk_idx, bytes)
    })
}

// In uploads_finish()
#[ic_cdk::update]
async fn uploads_finish(session_id: u64, expected_sha256: Vec<u8>, total_len: u64) -> Result_15 {
    ic_cdk::println!("FINISH_START sid={}", session_id);

    // Get rolling hash
    let computed = UPLOAD_HASH.with(|m| {
        m.borrow_mut()
            .remove(&session_id)
            .ok_or(Error::NotFound)
            .map(|h| h.finalize().to_vec())
    }).map_err(|e| {
        ic_cdk::println!("FINISH_ERROR sid={} err=hash_not_found", session_id);
        e
    })?;

    // Verify hash
    if computed != expected_sha256 {
        ic_cdk::println!("FINISH_ERROR sid={} err=checksum_mismatch computed={:?} expected={:?}",
            session_id, &computed[..8], &expected_sha256[..8]);
        return Result_15::Err(Error::ChecksumMismatch);
    }

    ic_cdk::println!("FINISH_HASH_OK sid={} len={}", session_id, total_len);

    // ... rest of finish logic ...
}
```

### Step 3: Run Test Suite (5 min) ⏭️

```bash
# Deploy
dfx deploy backend

# Run tests
./tests/backend/shared-capsule/upload/run_2lane_4asset_test.sh

# Check logs
dfx canister logs backend | egrep 'FINISH_|ERROR|Checksum'
```

### Step 4: Debug Probes (Optional - 10 min) ⏭️

```rust
#[ic_cdk::query]
fn debug_session_probe(sid: u64) -> (u32, u64, u64, Vec<u32>) {
    // Returns: (received_count, bytes_received, bytes_expected, missing_idxs)
    with_session_compat(|sessions| {
        if let Ok(Some(session)) = sessions.get(&SessionId(sid)) {
            let received = /* get received count */;
            let bytes_recv = /* get bytes received */;
            let bytes_exp = /* get bytes expected */;
            let missing = /* calculate missing indices */;
            (received, bytes_recv, bytes_exp, missing)
        } else {
            (0, 0, 0, vec![])
        }
    })
}

#[ic_cdk::query]
fn debug_blob_probe(pmid: String, chunk_count: u32) -> Vec<bool> {
    use crate::upload::blob_store::pmid_hash32;
    let stem = pmid_hash32(&pmid);
    (0..chunk_count).map(|idx| {
        STABLE_BLOB_STORE.with(|store| {
            store.borrow().contains_key(&(stem, idx))
        })
    }).collect()
}
```

---

## 📊 Current State

| Component              | Status      | Notes                              |
| ---------------------- | ----------- | ---------------------------------- |
| **Deterministic Keys** | ✅ Complete | Unified `pmid_hash32()` everywhere |
| **Compilation**        | ✅ Success  | 0 errors                           |
| **bytes_expected Fix** | ✅ Complete | Uses actual bytes from metadata    |
| **BlobMeta.pmid_hash** | ✅ Complete | Stored for all operations          |
| **Rolling Hash**       | ⏭️ Next     | ~30 min to implement               |
| **Logging**            | ⏭️ Next     | ~15 min to add                     |
| **Tests**              | ⏭️ Pending  | Run after logging added            |

---

## 🎯 Expected Outcome

After completing Steps 1-3:

**Current**: 2/5 tests passing (40%)  
**Expected**: 5/5 tests passing (100%)

**Why we're confident**:

1. ✅ Deterministic keys eliminate hash lookup failures
2. ✅ Rolling hash will catch any chunk corruption immediately
3. ✅ Logging will show exact failure points
4. ✅ One upload already succeeds (proof system works)

---

## 📁 Files Ready for Next Edits

1. **`src/backend/src/lib.rs`** - Add logging + rolling hash
2. **`src/backend/src/upload/service.rs`** - May need to add logs in `commit()`

---

## ⏱️ Time Estimate

- **Step 1** (Logging): 15 minutes
- **Step 2** (Rolling hash): 30 minutes
- **Step 3** (Test + Debug): 15 minutes
- **Total**: ~1 hour to completion

---

## 💡 Key Achievements

1. ✅ **Eliminated ALL non-deterministic hashing**
2. ✅ **Unified key derivation** in single function
3. ✅ **All read/write paths use same keys**
4. ✅ **Code compiles with 0 errors**
5. ✅ **System architecture is sound**

---

**Status**: 🟢 **READY** - Deterministic keys complete, ready for logging + rolling hash  
**Confidence**: VERY HIGH - Core architecture fixed  
**Blocker**: None - clear path to completion  
**ETA to 5/5 tests passing**: 1 hour

---

**Created**: 2025-10-01 23:30  
**Ready for**: Logging implementation → Rolling hash → Test validation  
**Developer**: Standing by to implement Steps 1-3
