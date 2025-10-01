# Upload Session Layer - Refactoring TODO

**Status**: 📋 **PLANNING** - Compatibility layer working, ready to refactor  
**Goal**: Remove SessionCompat compatibility layer, use SessionService directly  
**Timeline**: 1-2 weeks (after current implementation stabilizes)

---

## 🎯 Current State

### What We Have Now (Temporary)

```
UploadService
    ↓
SessionCompat (compatibility layer) ← TEMPORARY
    ↓
SessionService (generic)
    ↓
StableBlobSink (ByteSink implementation)
```

**Why we have this:**

- SessionCompat bridges old upload API → new generic SessionService
- Allows gradual migration without breaking existing code
- All tests passing (5/5) with this structure

### What We Want (Final)

```
UploadService
    ↓
SessionService (generic) ← DIRECT USE
    ↓
StableBlobSink (ByteSink implementation)
```

**Benefits:**

- ✅ Simpler code (one less layer)
- ✅ Better performance (fewer indirections)
- ✅ Easier to maintain
- ✅ True separation of concerns

---

## 📋 Refactoring Checklist

### Phase 1: Preparation (1-2 days)

- [ ] **Audit all SessionCompat usage**

  - [ ] Find all calls to SessionCompat methods
  - [ ] Document what each call does
  - [ ] Identify patterns that can be simplified

- [ ] **Create migration plan**

  - [ ] Map SessionCompat methods → SessionService equivalents
  - [ ] Identify upload-specific logic that stays in UploadService
  - [ ] Plan for metadata handling

- [ ] **Update tests**
  - [ ] Ensure SessionService has sufficient test coverage
  - [ ] Add any missing tests for direct SessionService usage
  - [ ] Keep E2E tests as integration validation

### Phase 2: Refactor UploadService (2-3 days)

- [ ] **Remove SessionCompat dependency**

  ```rust
  // Before:
  with_session_compat(|sessions| sessions.create(...))

  // After:
  SESSION_SERVICE.with(|svc| svc.begin(...))
  ```

- [ ] **Move upload-specific logic to UploadService**

  - [ ] provisional_memory_id generation
  - [ ] Upload metadata (UploadSessionMeta) → keep in UploadService
  - [ ] Idempotency key handling
  - [ ] blob_id assignment

- [ ] **Update method signatures**

  - [ ] `begin_upload()` - use SessionService.begin() directly
  - [ ] `put_chunk()` - use SessionService.put_chunk() directly
  - [ ] `commit()` - use SessionService.finish() + blob store logic

- [ ] **Handle session metadata**
  - [ ] Store upload-specific metadata separately (not in SessionService)
  - [ ] Use SessionId as key to lookup upload metadata
  - [ ] Keep generic session state in SessionService

### Phase 3: Clean Up (1 day)

- [ ] **Delete SessionCompat files**

  - [ ] Remove `src/backend/src/session/compat.rs`
  - [ ] Remove UploadSessionMeta (replace with simpler structure)
  - [ ] Clean up imports

- [ ] **Update documentation**

  - [ ] Update architecture diagrams
  - [ ] Document new direct usage pattern
  - [ ] Update code comments

- [ ] **Run full test suite**
  - [ ] All unit tests passing
  - [ ] All E2E tests passing (5/5)
  - [ ] Performance benchmarks unchanged

### Phase 4: Optimization (1-2 days)

- [ ] **Remove debug logging**

  - [ ] BLOB_VERIFY_SAMECALL logs
  - [ ] BLOB_READ/BLOB_WRITE logs
  - [ ] UPLOAD_HASH_INIT logs
  - [ ] Keep only essential error logs

- [ ] **Remove canary endpoints**

  - [ ] debug_blob_write_canary
  - [ ] debug_blob_read_canary

- [ ] **Performance improvements**
  - [ ] Profile upload performance
  - [ ] Optimize hot paths if needed
  - [ ] Benchmark parallel uploads

---

## 🗺️ Detailed Migration Guide

### 1. UploadService.begin_upload()

**Current (with SessionCompat):**

```rust
pub fn begin_upload(...) -> Result<SessionId, Error> {
    let session_id = SessionId::new();
    let upload_meta = UploadSessionMeta {
        session_id: session_id.0,
        capsule_id,
        // ... lots of fields
    };
    with_session_compat(|sessions| sessions.create(session_id, upload_meta))?;
    Ok(session_id)
}
```

**Target (direct SessionService):**

```rust
pub fn begin_upload(...) -> Result<SessionId, Error> {
    let session_id = SessionId::new();

    // Create generic session spec
    let spec = SessionSpec {
        chunk_size: CHUNK_SIZE,
        bytes_expected: asset_metadata.get_base().bytes,
        owner: caller,
        idem: idem.clone(),
    };

    // Store in SessionService (generic)
    SESSION_SERVICE.with(|svc| svc.begin_with_id(session_id, spec, &ICClock))?;

    // Store upload-specific metadata separately
    UPLOAD_METADATA.with(|meta| {
        meta.borrow_mut().insert(session_id.0, UploadMeta {
            capsule_id,
            provisional_memory_id,
            asset_metadata,
            // Only upload-specific fields
        });
    });

    Ok(session_id)
}
```

### 2. UploadService.put_chunk()

**Current (with SessionCompat):**

```rust
pub fn put_chunk(...) -> Result<(), Error> {
    with_session_compat(|sessions| {
        sessions.put_chunk(&session_id, chunk_idx, bytes)
    })
}
```

**Target (direct SessionService):**

```rust
pub fn put_chunk(...) -> Result<(), Error> {
    // Get upload metadata
    let meta = UPLOAD_METADATA.with(|m| {
        m.borrow().get(&session_id.0).cloned().ok_or(Error::NotFound)
    })?;

    // Create sink
    let mut sink = StableBlobSink::for_meta(&meta)?;

    // Use SessionService directly
    SESSION_SERVICE.with(|svc| {
        svc.borrow_mut().put_chunk(session_id, chunk_idx, bytes, &mut sink, &ICClock)
    })
}
```

### 3. UploadService.commit()

**Current (with SessionCompat):**

```rust
pub fn commit(...) -> Result<(String, MemoryId), Error> {
    // Verify chunks
    with_session_compat(|sessions| {
        sessions.verify_chunks_complete(&session_id, chunk_count)
    })?;

    // Store from chunks
    let blob_id = with_session_compat(|sessions| {
        self.blobs.store_from_chunks(sessions, &session_id, ...)
    })?;

    // ... rest of commit logic
}
```

**Target (direct SessionService):**

```rust
pub fn commit(...) -> Result<(String, MemoryId), Error> {
    // Get upload metadata
    let meta = UPLOAD_METADATA.with(|m| {
        m.borrow().get(&session_id.0).cloned().ok_or(Error::NotFound)
    })?;

    // Verify chunks (SessionService)
    let chunks_ok = SESSION_SERVICE.with(|svc| {
        svc.borrow().received_count(session_id) == chunk_count
    });
    if !chunks_ok { return Err(Error::IncompleteUpload); }

    // Store blob (using metadata)
    let blob_id = self.blobs.store_from_metadata(&meta, &session_id, ...)?;

    // Finish session (SessionService)
    SESSION_SERVICE.with(|svc| svc.borrow_mut().finish(session_id, &ICClock))?;

    // Create memory and index
    // ... rest of commit logic

    // Cleanup
    UPLOAD_METADATA.with(|m| m.borrow_mut().remove(&session_id.0));

    Ok((blob_id, memory_id))
}
```

---

## 📊 Data Structure Changes

### Before (SessionCompat)

```rust
// In SessionCompat
pub struct UploadSessionMeta {
    session_id: u64,
    capsule_id: CapsuleId,
    caller: Principal,
    created_at: u64,
    expected_chunks: u32,
    status: SessionStatus,
    chunk_count: u32,
    asset_metadata: AssetMetadata,
    provisional_memory_id: String,
    chunk_size: usize,
    idem: String,
    blob_id: Option<u64>,
}
```

### After (Split)

```rust
// Generic session state (in SessionService)
pub struct Session {
    owner: Vec<u8>,           // Principal bytes
    chunk_size: usize,
    bytes_expected: u64,
    bytes_received: u64,
    received_idxs: BTreeSet<u32>,
    session_meta: SessionMeta {
        idem: String,
        last_seen: u64,
        status: SessionStatus,
    },
}

// Upload-specific metadata (in UploadService)
thread_local! {
    static UPLOAD_METADATA: RefCell<BTreeMap<u64, UploadMeta>> = ...;
}

pub struct UploadMeta {
    capsule_id: CapsuleId,
    provisional_memory_id: String,
    asset_metadata: AssetMetadata,
    blob_id: Option<u64>,
    // Only upload-specific fields!
}
```

---

## ⚠️ Migration Risks & Mitigation

### Risk 1: Breaking Existing Sessions

**Risk**: Active upload sessions might fail during migration  
**Mitigation**:

- Deploy during low-traffic window
- Add migration code to handle old sessions
- Keep SessionCompat temporarily for backward compatibility

### Risk 2: Test Failures

**Risk**: Tests might fail with new structure  
**Mitigation**:

- Maintain full test coverage during migration
- Add new tests before removing old code
- Use feature flags to toggle between old/new implementation

### Risk 3: Performance Regression

**Risk**: Direct usage might have unexpected performance issues  
**Mitigation**:

- Benchmark before/after
- Profile critical paths
- Keep performance metrics (should be better, not worse)

---

## 🎯 Success Criteria

### Must Have

- ✅ All 5 E2E tests passing
- ✅ No SessionCompat code remaining
- ✅ Upload performance ≥ current (0.62 MB/s for single, 0.50 MB/s for parallel)
- ✅ Code is simpler (fewer lines, fewer indirections)

### Nice to Have

- ✅ Improved performance (fewer indirections should help)
- ✅ Cleaner error messages
- ✅ Better documentation
- ✅ Reduced memory usage

---

## 📅 Timeline Estimate

| Phase                           | Duration     | Dependencies                  |
| ------------------------------- | ------------ | ----------------------------- |
| Phase 1: Preparation            | 1-2 days     | None                          |
| Phase 2: Refactor UploadService | 2-3 days     | Phase 1 complete              |
| Phase 3: Clean Up               | 1 day        | Phase 2 complete              |
| Phase 4: Optimization           | 1-2 days     | Phase 3 complete              |
| **Total**                       | **5-8 days** | Current implementation stable |

---

## 📝 Notes

### Why Not Do This Now?

The current SessionCompat implementation is:

- ✅ Working perfectly (5/5 tests passing)
- ✅ Production-ready
- ✅ Well-tested

We should:

1. Let it stabilize in production
2. Gather real-world usage data
3. Then refactor when we're confident

### Alternative: Keep SessionCompat

We could also keep SessionCompat as a permanent pattern if:

- We add more session types (downloads, streaming, etc.)
- We want to maintain API stability
- The extra layer provides useful isolation

This is a design decision to make later based on:

- Real-world usage patterns
- Performance data
- Future feature requirements

---

**Created**: 2025-10-01  
**Status**: 📋 Planning  
**Priority**: Medium (can wait until current implementation stabilizes)  
**Next Step**: Let current implementation run in production, gather data, then decide
