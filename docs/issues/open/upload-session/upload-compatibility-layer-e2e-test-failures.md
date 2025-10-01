# Compatibility Layer E2E Test Failures

**Status**: 🔴 **BLOCKED** - Need tech lead guidance on parallel upload failures  
**Priority**: **HIGH** - Blocking MVP validation  
**Assignee**: Tech Lead  
**Created**: 2025-10-01  
**Updated**: 2025-10-01

## Executive Summary

The compatibility layer implementation is **complete and compiles successfully** with unit tests passing (17/17 SessionService tests). However, E2E tests show **parallel upload failures** (60% failure rate) while sequential uploads work perfectly.

## Test Results Summary

```
2-Lane + 4-Asset Upload System Test Summary:
Total tests: 5
Passed: 2 (40%)
Failed: 3 (60%)

✅ Lane A (Sequential Original Upload): PASSING
✅ Lane B (Sequential Image Processing): PASSING
❌ Parallel Lanes Execution: FAILING
❌ Complete 2-Lane + 4-Asset System: FAILING
❌ Asset Retrieval: FAILING
```

## What's Working ✅

### 1. **Unit Tests - All Passing**

- ✅ **SessionService tests**: 17/17 passing
- ✅ **Generic session management**: Working correctly
- ✅ **ByteSink interface**: Implemented and tested
- ✅ **Write-through design**: No heap buffering confirmed

### 2. **Sequential Uploads - Perfect**

```javascript
// Lane A: Original Upload (20.8 MB file)
ℹ️  ✅ Upload session started: 1
ℹ️  📦 Uploading 13 chunks (1.7 MB each)...
ℹ️    📈 100% (13/13 chunks)
ℹ️  ✅ Upload finished successfully
✅ Lane A: Original Upload - PASSED
```

### 3. **Image Processing - Perfect**

```javascript
// Lane B: Image Processing
ℹ️  🖼️ Processing derivatives for 20.8 MB file
ℹ️  📊 Derivative sizes:
ℹ️    Display: 200.0 KB (1920x1080)
ℹ️    Thumb: 50.0 KB (300x169)
ℹ️    Placeholder: 1.0 KB (32x18)
✅ Lane B: Image Processing - PASSED
```

## What's Failing ❌

### 1. **Parallel Lanes Execution**

```javascript
// Test: Upload original + process derivatives in parallel
ℹ️  Running: Parallel Lanes Execution

// 4 sessions created successfully
ℹ️  ✅ Upload session started: 4  // Original file
ℹ️  ✅ Upload session started: 2  // Display derivative
ℹ️  ✅ Upload session started: 5  // Thumb derivative
ℹ️  ✅ Upload session started: 3  // Placeholder derivative

// All chunks upload successfully
ℹ️    📈 100% (1/1 chunks)  // Derivatives complete
ℹ️    📈 100% (13/13 chunks)  // Original complete

// BUT THEN...
❌ Parallel Lanes Execution - FAILED
// Error: "Lane failed: A=rejected, B=fulfilled" or similar
```

**Pattern**: Sessions are created, chunks upload, but `Promise.allSettled()` shows rejection.

### 2. **Complete 2-Lane + 4-Asset System**

```javascript
ℹ️  Running: Complete 2-Lane + 4-Asset System

// Same pattern - sessions created, chunks uploaded
ℹ️  ✅ Upload session started: 6-9 (4 sessions)
ℹ️    📈 100% (all chunks uploaded)

❌ Complete 2-Lane + 4-Asset System: Lane failed: A=rejected, B=rejected
```

### 3. **Asset Retrieval**

```javascript
ℹ️  Running: Asset Retrieval

// Uploads complete
ℹ️  ✅ Upload finished successfully: blob_id=..., memory_id=...
ℹ️  ✅ Upload completed: display (200.0 KB) in 6.2s

// But overall test fails
❌ Asset Retrieval: Lane failed: A=rejected, B=rejected
```

## Technical Analysis

### Architecture Summary

The compatibility layer implements the **Hybrid Architecture (Option C)** from the tech lead's recommendation:

```
session/
├── types.rs          # Generic types (SessionId, SessionSpec, ByteSink, Clock)
├── service.rs        # Generic SessionService (pure Rust, no upload semantics)
├── compat.rs         # SessionCompat compatibility layer
└── adapter.rs        # IC-specific adapter (ICClock)

upload/
├── service.rs        # Upload service (uses SessionCompat)
├── blob_store.rs     # StableBlobSink (implements ByteSink)
└── types.rs          # UploadSessionMeta
```

### What We Know

1. **Session Creation**: Working (sessions 1-13 created successfully)
2. **Chunk Upload**: Working (all chunks upload to 100%)
3. **ByteSink Write**: Working (data is written through to storage)
4. **Sequential Flow**: Working perfectly (Lane A, Lane B individually)
5. **Parallel Flow**: Failing (Promise.allSettled shows rejection)

### What We Don't Know

**❓ Where is the rejection happening?**

The test output shows:

- ✅ Sessions created successfully
- ✅ Chunks uploaded successfully (100% progress)
- ❌ Promise rejects anyway

**Possible causes:**

1. **`uploads_finish()` rejecting** - Hash mismatch? Completeness check failing?
2. **Race condition in `finish()`** - Index update not atomic?
3. **Session cleanup interference** - Cleanup running during finalization?
4. **Error propagation issue** - Error happening but not logged?

## Code Flow Analysis

### Sequential Upload (Working)

```rust
// uploads_begin() -> SessionId
let sid = with_session_compat(|sessions| {
    sessions.create(session_id, upload_meta)
})?;

// uploads_put_chunk() * N -> Ok(())
for chunk in chunks {
    with_session_compat(|sessions| {
        sessions.put_chunk(&sid, idx, &data)  // Calls ByteSink factory
    })?;
}

// uploads_finish() -> Result<MemoryId, Error>
let result = self.finish_upload(...)?;  // ← WHERE DOES THIS REJECT?
```

### Parallel Upload (Failing)

```javascript
// JavaScript side
const laneAPromise = uploadOriginalToICP(backend, fileBuffer, fileName);
const laneBPromise = processImageDerivativesToICP(backend, fileBuffer, mimeType);

const [laneAResult, laneBResult] = await Promise.allSettled([
  laneAPromise, // ← REJECTS in parallel mode
  laneBPromise, // ← REJECTS in parallel mode
]);

// Error: "Lane failed: A=rejected, B=rejected"
```

## Missing Visibility

### Problem: No Error Details in Test Output

The test shows **rejection** but **not the actual error**:

```javascript
❌ Parallel Lanes Execution
// What error? Why rejection? Where in the flow?
```

### What We Need

1. **Detailed error messages** from rejected promises
2. **Backend logs** showing what's happening during parallel uploads
3. **Session state** at point of failure
4. **Hash/checksum values** if that's the issue

## Questions for Tech Lead

### 1. **Error Logging Strategy**

How should we add better error visibility without cluttering production code?

**Options:**

- Add debug logging in `SessionCompat`?
- Add error details to test output?
- Use `dfx canister logs backend` (but grep shows nothing)?

### 2. **Suspected Root Cause**

Based on the pattern (sequential works, parallel fails), likely culprits:

**A) Hash Verification Failing**

```rust
// In uploads_finish()
let computed_hash = compute_hash_from_chunks()?;
if computed_hash != expected_hash {
    return Err(Error::ChecksumMismatch);  // ← Silent rejection?
}
```

**B) Session Cleanup Race**

```rust
// cleanup_expired_sessions_for_caller() running during parallel upload
// Evicting sessions that are still in use?
```

**C) Index Update Not Atomic**

```rust
// finish() doesn't update index before returning
// Causing race condition in parallel finalization
```

Which is most likely? How to diagnose?

### 3. **Tech Lead's Original Guidance**

From `backend-session-architecture-reorganization.md`:

> **Two Concrete Technical Risks**
>
> 1. **You still buffer chunks in heap** - We removed `chunks: BTreeMap`
> 2. **Cleanup still too broad** - We have cleanup functions but unclear if they're interfering

Did we fully address cleanup scope?

### 4. **Next Debugging Steps**

What's the most efficient way to diagnose this?

**Options:**

- Add `ic_cdk::println!()` to `uploads_finish()`?
- Add error capture to `SessionCompat::put_chunk()`?
- Add session state logging before/after parallel uploads?
- Use PocketIC for local debugging?

## Proposed Debugging Approach

### Step 1: Add Error Visibility

```rust
// In uploads_finish()
pub async fn uploads_finish(
    session_id: u64,
    expected_sha256: Vec<u8>,
    total_len: u64,
) -> Result<MemoryId, Error> {
    ic_cdk::println!("FINISH: sid={}, expected_len={}", session_id, total_len);

    // Verify chunks
    let result = with_session_compat(|sessions| {
        sessions.verify_chunks_complete(&SessionId(session_id), chunk_count)
    });

    if let Err(e) = result {
        ic_cdk::println!("FINISH ERROR: Chunks incomplete: {:?}", e);
        return Err(e);
    }

    // ... rest of finish logic with logging
}
```

### Step 2: Capture Rejection Details in Test

```javascript
const [laneAResult, laneBResult] = await Promise.allSettled([laneAPromise, laneBPromise]);

if (laneAResult.status === "rejected") {
  console.error("❌ Lane A rejected:", laneAResult.reason);
}
if (laneBResult.status === "rejected") {
  console.error("❌ Lane B rejected:", laneBResult.reason);
}
```

### Step 3: Check Backend Logs

```bash
dfx canister logs backend | grep -E "(FINISH|ERROR|UPLOAD)" > debug.log
```

## Files Affected

### Backend Implementation

- `src/backend/src/session/service.rs` - Generic session service (✅ 17/17 tests passing)
- `src/backend/src/session/compat.rs` - Compatibility layer (⚠️ 12 tests IC-dependent)
- `src/backend/src/upload/service.rs` - Upload service (❓ finish logic unclear)
- `src/backend/src/upload/blob_store.rs` - ByteSink implementation (✅ working)
- `src/backend/src/lib.rs` - Upload endpoints (❓ error handling unclear)

### Test Files

- `tests/backend/shared-capsule/upload/test_upload_2lane_4asset_system.mjs` - E2E test (❌ 3/5 failing)
- `tests/backend/shared-capsule/upload/helpers.mjs` - Test utilities

## Business Impact

- **MVP Delivery**: Blocked until parallel uploads work
- **User Experience**: Sequential uploads work, but parallel uploads fail
- **Performance**: Can't leverage parallel processing for image derivatives
- **Confidence**: 40% test success rate is not production-ready

## Timeline Estimate

With proper error visibility:

- **1-2 hours**: Add logging and capture errors
- **1-2 hours**: Diagnose root cause
- **2-4 hours**: Implement fix
- **Total**: 4-8 hours to resolution

Without visibility:

- **Unknown**: Blind debugging could take days

## Tech Lead Review & Action Items

**Status**: ✅ **GUIDANCE RECEIVED** - Clear action plan provided  
**Reviewer**: Tech Lead  
**Review Date**: 2025-10-01

### Verdict

- **Architecture:** ✅ Matches the plan (generic `session` core + upload-specific compat + `ByteSink`)
- **Type hygiene:** ✅ Re-exporting `SessionId/SessionStatus` avoids duplication
- **No buffering:** ✅ `ByteSink` write-through is the right direction

**We're very close. The remaining issues are concrete and quick to fix.**

---

## Must-Fix Before Merge (5 Items)

### 1. ❌ `bytes_expected` Source of Truth

**Location**: `SessionCompat::create`  
**Issue**: Deriving `bytes_expected` as `chunk_count * chunk_size` is wrong for partial last chunks  
**Impact**: HIGH - Causes hash/length mismatches

**Fix**:

```rust
// WRONG (current):
bytes_expected: (m.chunk_count as u64) * (m.chunk_size as u64)

// RIGHT (use actual byte length):
bytes_expected: m.asset_metadata.base.bytes
```

**File**: `src/backend/src/session/compat.rs:68`

---

### 2. ❌ `StableBlobSink::write_at` Uses Undefined `blob_id`

**Location**: `upload/blob_store.rs`  
**Issue**: Code references `blob_id` but sink only has `capsule_id`, `provisional_memory_id`, `chunk_size`  
**Impact**: HIGH - Critical correctness bug

**Fix Options**:

**A) Chunk-indexed store (current shape):**

```rust
fn write_at(&mut self, offset: u64, data: &[u8]) -> Result<(), Error> {
    let chunk_idx = (offset / self.chunk_size as u64) as u32;
    if offset % (self.chunk_size as u64) != 0 {
        return Err(Error::InvalidArgument("unaligned offset".into()));
    }
    if data.len() > self.chunk_size && (offset > 0) {
        return Err(Error::InvalidArgument("oversized chunk".into()));
    }
    STABLE_BLOB_STORE.with(|store| {
        store.borrow_mut().insert(
            (self.provisional_memory_id.clone(), chunk_idx),
            data.to_vec(),
        );
    });
    Ok(())
}
```

**B) Byte-addressed store (preferred):**

```rust
// Key: (provisional_memory_id, offset)
// No chunk_idx translation needed
```

**File**: `src/backend/src/upload/blob_store.rs`

---

### 3. ❌ Missing Rolling Hash Update

**Location**: `uploads_put_chunk` and `uploads_finish`  
**Issue**: No SHA-256 hash update visible in code  
**Impact**: HIGH - Hash verification will fail

**Fix Pattern**:

```rust
// Add thread-local hash storage
thread_local! {
    static UPLOAD_HASH: RefCell<BTreeMap<u64, Sha256>> = RefCell::new(BTreeMap::new());
}

#[ic_cdk::update]
fn uploads_put_chunk(sid: u64, idx: u32, data: Vec<u8>) -> Result<(), Error> {
    // Update hash BEFORE or AFTER write
    UPLOAD_HASH.with(|h| {
        h.borrow_mut()
            .get_mut(&sid)
            .ok_or(Error::NotFound)?
            .update(&data)
    });

    with_session_compat(|c| c.put_chunk(&SessionId(sid), idx, &data))?;
    Ok(())
}

#[ic_cdk::update]
fn uploads_finish(sid: u64, client_sha: Vec<u8>, total_len: u64) -> Result<MemoryId, Error> {
    let computed = UPLOAD_HASH.with(|h| {
        h.borrow_mut()
            .remove(&sid)
            .ok_or(Error::NotFound)?
            .finalize()
            .to_vec()
    });

    if computed != client_sha {
        return Err(Error::ChecksumMismatch);
    }

    // ... rest of finish logic
}
```

**Files**: `src/backend/src/lib.rs` (upload endpoints)

---

### 4. ❌ `SessionCompat` Should Use Trait Object

**Location**: `SessionCompat` struct definition  
**Issue**: Generic parameter causes monomorphization issues in thread-locals  
**Impact**: MEDIUM - Compilation/storage complexity

**Fix**:

```rust
pub struct SessionCompat {
    svc: RefCell<SessionService>,
    meta: RefCell<BTreeMap<u64, UploadSessionMeta>>,
    idem: RefCell<BTreeMap<IdemKey, SessionId>>,
    sink_factory: Box<dyn Fn(&UploadSessionMeta) -> Result<Box<dyn ByteSink>, Error>>,
    //            ^^^ Add Box wrapper
}

impl SessionCompat {
    pub fn new(
        sink_factory: impl Fn(&UploadSessionMeta) -> Result<Box<dyn ByteSink>, Error> + 'static,
    ) -> Self {
        Self {
            svc: RefCell::new(SessionService::new()),
            meta: RefCell::new(BTreeMap::new()),
            idem: RefCell::new(BTreeMap::new()),
            sink_factory: Box::new(sink_factory),  // Box the closure
        }
    }
}
```

**File**: `src/backend/src/session/compat.rs:26-47`

---

### 5. ❌ `uploads_finish` Must Commit Index Before Returning

**Location**: `uploads_finish`  
**Issue**: Index update not atomic with finish - causes retrieval flakes  
**Impact**: HIGH - Causes the E2E test failures we're seeing

**Fix**:

```rust
fn uploads_finish(...) -> Result<MemoryId, Error> {
    // 1. Verify hash
    let computed = UPLOAD_HASH.with(...);
    if computed != client_sha { return Err(Error::ChecksumMismatch); }

    // 2. Finalize session
    with_session_compat(|c| c.finish(&SessionId(sid)))?;

    // 3. Update index/metadata/counters BEFORE returning ← CRITICAL
    let memory_id = commit_and_index(...)?;

    // 4. Only then return success
    Ok(memory_id)
}
```

**File**: `src/backend/src/upload/service.rs` or `src/backend/src/lib.rs`

---

## Should-Fix Soon (Not Blockers)

### 6. ⚠️ Verify Chunk Coverage (No Gaps)

**Current**: Comparing `received_count == chunk_count`  
**Better**: Use `BitSet` to verify all expected indices are set

```rust
// Check coverage (no gaps)
let expected_chunks = (bytes_expected + chunk_size - 1) / chunk_size;
if session.received_idxs.len() != expected_chunks as usize {
    return Err(Error::InvalidArgument("Incomplete chunks".into()));
}
// Also verify indices are contiguous: 0, 1, 2, ..., expected_chunks-1
```

### 7. ⚠️ Invariants in `put_chunk`

- Reject duplicate `idx` (or treat as idempotent if bytes match)
- Enforce `data.len() <= chunk_size`
- Track `bytes_received += data.len()`
- On finish: assert `bytes_received == bytes_expected`

### 8. ⚠️ TTL Cleanup Implementation

`cleanup_expired_sessions_for_caller` doesn't actually evict  
**Decision**: Wire to tick or remove call sites for now

### 9. ⚠️ Warnings (34 unused items)

Add `#[allow(dead_code)]` on compat module temporarily, then prune

---

## Required Tests (Before Merge)

### Test 1: Immediate Retrieval (100x loop)

```javascript
for (let i = 0; i < 100; i++) {
  const sid = await begin();
  await putAllChunks(sid);
  await finish(sid);
  const asset = await list(); // Must succeed immediately
  assert(asset.present);
}
```

### Test 2: Sparse Interleaving

```javascript
// Two sessions A/B with non-sequential chunks
BeginA,
  BeginB,
  PutA0,
  PutB0,
  PutA2,
  PutB2, // Skip chunk 1
  PutA1,
  PutB1, // Fill gap
  FinishA,
  FinishB;
// Assert no NotFound, final hashes match
```

### Test 3: Large File Memory Check

```javascript
// Upload 20-50 MB
const heapBefore = getHeapSize();
await upload50MB();
const heapAfter = getHeapSize();
assert((heapAfter - heapBefore) < 5MB);  // Prove no chunk buffering
```

### Test 4: Duplicate Chunk Handling

```javascript
await putChunk(sid, 0, data);
await putChunk(sid, 0, data); // Same bytes → allow (idempotent)
await putChunk(sid, 0, differentData); // Different bytes → error
```

---

## Root Cause Analysis (Updated)

### **Likely Cause of E2E Failures**

Based on tech lead review, the **most likely causes** are:

1. **Missing hash updates** (Item #3) → Hash verification fails silently
2. **Index not committed before return** (Item #5) → Retrieval immediately after finish fails
3. **Wrong `bytes_expected`** (Item #1) → Length mismatch causes rejection

### **Why Parallel Fails But Sequential Works**

Sequential mode likely has timing slack that hides the race condition:

- Sequential: Hash computed, index updated, then next upload starts
- Parallel: Multiple finish() calls race, index updates conflict or get skipped

---

## Action Plan

### Phase 1: Critical Fixes (Day 1 - 4 hours)

1. ✅ Fix `bytes_expected` calculation (Item #1) - 15 min
2. ✅ Fix `StableBlobSink::write_at` key scheme (Item #2) - 30 min
3. ✅ Add rolling hash updates (Item #3) - 1 hour
4. ✅ Box `sink_factory` trait object (Item #4) - 15 min
5. ✅ Ensure index commit before return (Item #5) - 1 hour

**Total**: ~3-4 hours of focused work

### Phase 2: Validation (Day 1-2 - 2 hours)

1. Run E2E tests again
2. Verify all 5 tests pass
3. Check for hash/length errors in logs
4. Validate retrieval works immediately

### Phase 3: Additional Tests (Day 2 - 2 hours)

1. Implement Test 1: Immediate retrieval (100x loop)
2. Implement Test 2: Sparse interleaving
3. Implement Test 3: Large file memory check
4. Implement Test 4: Duplicate chunk handling

### Phase 4: Cleanup (Day 2-3 - 1 hour)

1. Address should-fix items
2. Clean up warnings
3. Update documentation

**Total Timeline**: 2-3 days to production-ready

---

## Files Requiring Changes

### Critical Fixes

1. `src/backend/src/session/compat.rs` - Fix bytes_expected, box sink_factory
2. `src/backend/src/upload/blob_store.rs` - Fix write_at key scheme
3. `src/backend/src/lib.rs` - Add hash updates to put_chunk/finish
4. `src/backend/src/upload/service.rs` - Ensure index commit atomicity

### Test Files

1. `tests/backend/shared-capsule/upload/test_upload_2lane_4asset_system.mjs` - Should pass after fixes
2. `tests/backend/shared-capsule/upload/session/test_*.mjs` - Validate with new architecture

---

**Status**: ✅ **READY TO IMPLEMENT** - Clear action plan provided  
**Next Step**: Implement 5 must-fix items (estimated 3-4 hours)  
**Expected Outcome**: E2E tests pass, parallel uploads work  
**Confidence**: High (tech lead identified exact issues)
