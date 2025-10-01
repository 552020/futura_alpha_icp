# Upload Session - Complete Refactoring Plan

**Status**: 📋 **OPEN** - Ready to start (after MVP stabilization)  
**Priority**: Medium - Technical debt cleanup  
**Impact**: Maintainability, code quality  
**Timeline**: 5-8 days (incremental, low-risk)  
**Blocker**: None (current system is green: 5/5 tests passing)

---

## 🎯 Goal

Retire the compatibility shim (`SessionCompat`) and make `upload::service` talk directly to the generic `session::service`, with a single source of truth for types and semantics.

**Short answer**: Yes—but not today. You've got a solid MVP. "Completing the refactor" should be an **incremental cleanup**, not a new fire-drill.

---

## 📋 Phase Breakdown

### Phase 0 — Freeze what works (NOW) ✅

**Status**: Already done

- ✅ Keep current public `.did` **unchanged**
- ✅ Keep `SessionCompat` but mark it `@deprecated / internal`
- ✅ Add big comment pointing to this plan
- ✅ Leave rolling hash + session-salted keys exactly as they are (battle-tested)

**Action Items**:

- [x] All tests passing (5/5 E2E)
- [ ] Add deprecation comment to `SessionCompat`
- [ ] Add link to this issue in code comments

---

### Phase 1 — Type unification (low risk)

**Goal**: Single source of truth for types

**Changes**:

1. **Use only** `session::types::{SessionId, SessionStatus, SessionMeta, SessionSpec}` across upload
2. **Move** upload-specific fields into `upload::types::UploadMeta`:
   - `capsule_id`
   - `asset_metadata`
   - `pmid` (provisional_memory_id)
   - `pmid_stem` (computed once, stored)
   - `chunk_size`
   - `idem`
   - etc.
3. **Re-export** session types from `upload::types` to avoid deep import churn
4. **Ensure one place** computes and stores `pmid_stem`; readers never recompute

**Before**:

```rust
// In SessionCompat
pub struct UploadSessionMeta {
    session_id: u64,              // DUPLICATE of SessionId
    status: SessionStatus,        // DUPLICATE of session::types
    created_at: u64,              // DUPLICATE of SessionMeta
    idem: String,                 // DUPLICATE of SessionMeta
    // ... upload-specific fields mixed in
}
```

**After**:

```rust
// In upload::types
pub use session::types::{SessionId, SessionStatus, SessionMeta, SessionSpec};

pub struct UploadMeta {
    capsule_id: CapsuleId,
    asset_metadata: AssetMetadata,
    pmid_stem: String,           // Computed ONCE, stored here
    chunk_size: usize,
    // Only upload-specific fields
}

// Store separately
thread_local! {
    static UPLOAD_META: RefCell<BTreeMap<SessionId, UploadMeta>> = ...;
}
```

**Acceptance Criteria**:

- [ ] No duplicate session types in upload module
- [ ] `pmid_stem` computed once in `begin_upload`, stored in `UploadMeta`
- [ ] Session types re-exported from `upload::types` for convenience
- [ ] All tests still passing

**Risk**: Low (type refactoring, no semantic changes)

---

### Phase 2 — Direct calls to SessionService (remove the shim)

**Goal**: Remove `SessionCompat` layer, call `SessionService` directly

**Changes**:

1. **Replace** `SessionCompat::create` with `SessionService::begin_with_id`
2. **Replace** `SessionCompat::put_chunk` with `SessionService::put_chunk`
3. **Replace** `SessionCompat::verify_chunks_complete` with `SessionService` state checks
4. **Replace** `SessionCompat::cleanup` with `SessionService::finish/abort`
5. **Pass** `ByteSink` from `upload::blob_store` directly into `SessionService::put_chunk`
6. **Keep** rolling-hash bookkeeping in **upload layer** for now (simpler)

**Before**:

```rust
// In UploadService::begin_upload
with_session_compat(|sessions| sessions.create(session_id, upload_meta))?;
```

**After**:

```rust
// In UploadService::begin_upload
let spec = SessionSpec {
    chunk_size: CHUNK_SIZE,
    bytes_expected: asset_metadata.get_base().bytes,
    owner: caller,
    idem: idem.clone(),
};
SESSION_SERVICE.with(|svc| svc.borrow_mut().begin_with_id(session_id, spec, &ICClock))?;

UPLOAD_META.with(|meta| {
    meta.borrow_mut().insert(session_id, UploadMeta {
        capsule_id,
        asset_metadata,
        pmid_stem: format!("{}#{}", capsule_id.0, path),  // Compute ONCE
        chunk_size: CHUNK_SIZE,
    });
});
```

**Before**:

```rust
// In UploadService::put_chunk
with_session_compat(|sessions| {
    sessions.put_chunk(&session_id, chunk_idx, bytes)
})?;
```

**After**:

```rust
// In UploadService::put_chunk
let meta = UPLOAD_META.with(|m| {
    m.borrow().get(&session_id).cloned().ok_or(Error::NotFound)
})?;

let mut sink = StableBlobSink::for_meta(&meta, session_id)?;  // Use stored pmid_stem

SESSION_SERVICE.with(|svc| {
    svc.borrow_mut().put_chunk(session_id, chunk_idx, bytes, &mut sink, &ICClock)
})?;
```

**Acceptance Criteria**:

- [ ] No calls to `SessionCompat` in `UploadService`
- [ ] `ByteSink` created from `UploadMeta` (uses stored `pmid_stem`)
- [ ] Rolling hash still in upload layer (lib.rs)
- [ ] All tests still passing

**Risk**: Medium (wiring changes, but semantics unchanged)

---

### Phase 3 — Semantics in the right layer

**Goal**: Move policies to appropriate layers

**Changes**:

1. **Ordering policy**: Enforce in-order in **session layer** (MVP)

   - Upload stays dumb
   - If later allowing out-of-order, flip a `needs_full_rehash` flag
   - Let finish do a single readback if needed

2. **Limits**: Per-principal / per-capsule active sessions → **session layer**

   ```rust
   pub fn begin_with_id(&mut self, ...) -> Result<SessionId, Error> {
       let active_count = self.sessions.values()
           .filter(|s| s.owner == owner && s.session_meta.status == Active)
           .count();
       if active_count >= MAX_ACTIVE_PER_PRINCIPAL {
           return Err(Error::TooManySessions);
       }
       // ...
   }
   ```

3. **TTL cleanup**: **Session layer** (since it knows last_seen, bytes_received, etc.)
   ```rust
   pub fn tick_ttl(&mut self, now: u64) {
       self.sessions.retain(|id, s| {
           if now - s.session_meta.last_seen > TTL_NANOS {
               ic_cdk::println!("SESSION_EXPIRED sid={}", id.0);
               false  // Remove
           } else {
               true   // Keep
           }
       });
   }
   ```

**Acceptance Criteria**:

- [ ] In-order chunk enforcement in `SessionService::put_chunk`
- [ ] Active session limits in `SessionService::begin_with_id`
- [ ] TTL cleanup in `SessionService::tick_ttl` (called periodically)
- [ ] Upload layer only knows about upload semantics (capsules, assets, etc.)
- [ ] All tests still passing

**Risk**: Low (moving existing logic to better homes)

---

### Phase 4 — Observability & hardening

**Goal**: Clean up debug code, add production-ready observability

**Changes**:

1. **Keep the three finish logs** (START / HASH_OK / COMMITTED)

   ```rust
   ic_cdk::println!("FINISH_START sid={}", session_id.0);
   ic_cdk::println!("FINISH_HASH_OK sid={} hash={}", session_id.0, hex::encode(&hash));
   ic_cdk::println!("FINISH_INDEX_COMMITTED sid={} mid={}", session_id.0, memory_id.0);
   ```

2. **Remove the rest** (BLOB_WRITE, BLOB_READ, UPLOAD_HASH_INIT, etc.)

3. **Add tiny metric/probe** behind `cfg!(debug_assertions)`:

   ```rust
   #[cfg(debug_assertions)]
   pub fn debug_session_stats() -> SessionStats {
       SESSION_SERVICE.with(|svc| {
           let sessions = svc.borrow();
           let by_principal = sessions.sessions.values()
               .fold(HashMap::new(), |mut acc, s| {
                   *acc.entry(s.owner.clone()).or_insert(0) += 1;
                   acc
               });
           SessionStats {
               total_active: sessions.sessions.len(),
               by_principal,
               total_chunk_writes: sessions.total_chunk_writes,
               finish_ok_count: sessions.finish_ok_count,
               finish_err_count: sessions.finish_err_count,
           }
       })
   }
   ```

4. **Add asserts/guards**:

   ```rust
   // In SessionService::begin_with_id
   const VALUE_BOUND: usize = 1_000_000;  // From ic-stable-structures
   assert!(spec.chunk_size <= VALUE_BOUND, "chunk_size exceeds stable memory limit");

   // In SessionService::finish
   assert_eq!(
       session.bytes_received,
       session.bytes_expected,
       "bytes_received must match bytes_expected at finish"
   );

   // In SessionService::put_chunk (already done in StableBlobSink)
   if offset % chunk_size != 0 {
       return Err(Error::UnalignedOffset);
   }
   if bytes.len() > chunk_size {
       return Err(Error::OversizedChunk);
   }
   ```

**Acceptance Criteria**:

- [ ] Only 3 finish logs remain
- [ ] Debug probe endpoints available in `cfg!(debug_assertions)`
- [ ] Static guards for chunk_size, bytes_expected, alignment
- [ ] All debug canary endpoints removed
- [ ] All tests still passing

**Risk**: Low (removing debug code, adding safety checks)

---

### Phase 5 — Delete compat

**Goal**: Final cleanup, delete all compatibility code

**Changes**:

1. **Delete** `src/backend/src/session/compat.rs`
2. **Delete** `UploadSessionMeta` struct (now just `UploadMeta`)
3. **Delete** any duplicate enums/aliases
4. **Update** `src/backend/src/session/mod.rs` to remove compat exports
5. **Run** full E2E + stress test suite

**Files to delete**:

- `src/backend/src/session/compat.rs`

**Files to update**:

- `src/backend/src/session/mod.rs` (remove `pub mod compat;`)
- `src/backend/src/upload/service.rs` (remove any compat imports)
- `src/backend/src/lib.rs` (remove any compat references)

**Acceptance Criteria**:

- [ ] No references to `SessionCompat` anywhere
- [ ] No references to `UploadSessionMeta` anywhere
- [ ] Only `session::types` used for shared session concepts
- [ ] Upload keeps `UploadMeta` for upload-specific fields
- [ ] All tests pass (5/5 E2E)
- [ ] `cargo build --release` succeeds with no warnings

**Risk**: Low (just deletion after previous phases work)

---

## 🚫 Risks to Avoid

### 1. API Churn

- ❌ **Don't** change `.did` during this cleanup
- ✅ Keep public API stable
- ✅ All changes are internal refactoring

### 2. Key Semantics Drift

- ❌ **Don't** change `pmid_session_hash32(pmid, session_id)` semantics
- ✅ Keep frozen as-is (battle-tested)
- ✅ Store `pmid_stem` in meta, always use stored value
- ✅ Never recompute on read

### 3. Stable Storage Schema

- ❌ **Don't** change `STABLE_BLOB_STORE` key/value types
- ✅ If you ever need to, version the memory IDs
- ✅ Write a migration or do a dev uninstall
- ✅ Document schema version in types

---

## ✅ Acceptance Criteria (Final)

### Code Quality

- [ ] No references to `SessionCompat` in `upload::service`
- [ ] Only `session::types` used for shared session concepts
- [ ] Upload keeps `UploadMeta` for upload-specific fields
- [ ] All duplicate types removed

### Testing

- [ ] All tests pass (5/5 E2E)
- [ ] Stress test passes (parallel uploads)
- [ ] No regressions in performance (≥0.62 MB/s single, ≥0.50 MB/s parallel)

### Observability

- [ ] Logs trimmed to 3 finish checkpoints
- [ ] Debug probes available behind `cfg!(debug_assertions)`
- [ ] Static/compile-time bounds/guards in place

### Documentation

- [ ] Update ARCHITECTURE.md to reflect new direct structure
- [ ] Update IMPLEMENTATION_GUIDE.md to remove compat references
- [ ] Update CHANGELOG.md with refactoring completion

---

## 📅 Order of Work (Fast Path)

| Phase                                | Duration     | Risk           | Dependencies |
| ------------------------------------ | ------------ | -------------- | ------------ |
| Phase 0: Freeze & mark deprecated    | 30 min       | None           | None         |
| Phase 1: Type unification            | 1-2 days     | Low            | Phase 0      |
| Phase 2: Direct SessionService calls | 2-3 days     | Medium         | Phase 1      |
| Phase 3: Semantics in right layer    | 1 day        | Low            | Phase 2      |
| Phase 4: Observability & hardening   | 1 day        | Low            | Phase 3      |
| Phase 5: Delete compat               | 30 min       | Low            | Phase 4      |
| **Total**                            | **5-8 days** | **Low-Medium** | Incremental  |

---

## 🎯 Success Metrics

### Before Refactoring

- ✅ 5/5 tests passing
- ✅ 0.62 MB/s single upload
- ✅ 0.50 MB/s parallel upload (79% efficiency)
- ⚠️ Extra indirection through `SessionCompat`
- ⚠️ Duplicate types (SessionStatus, SessionMeta, etc.)
- ⚠️ Upload-specific logic in compat layer

### After Refactoring

- ✅ 5/5 tests passing (no regressions)
- ✅ ≥0.62 MB/s single upload (should be faster)
- ✅ ≥0.50 MB/s parallel upload (should improve)
- ✅ Direct `UploadService → SessionService` (one less layer)
- ✅ Single source of truth for types
- ✅ Clean separation: generic session vs upload-specific

---

## 📝 Implementation Checklist

### Phase 0 ✅

- [ ] Add `#[deprecated]` to `SessionCompat`
- [ ] Add comment linking to this issue
- [ ] Commit: `refactor(backend): mark SessionCompat as deprecated`

### Phase 1

- [ ] Create `upload::types::UploadMeta` struct
- [ ] Re-export session types from `upload::types`
- [ ] Add `pmid_stem: String` to `UploadMeta`
- [ ] Update `begin_upload` to compute `pmid_stem` once
- [ ] Store `UploadMeta` in thread_local
- [ ] All tests passing
- [ ] Commit: `refactor(backend): unify session types, create UploadMeta`

### Phase 2

- [ ] Replace `SessionCompat::create` with `SessionService::begin_with_id`
- [ ] Replace `SessionCompat::put_chunk` with `SessionService::put_chunk`
- [ ] Replace `SessionCompat::verify_chunks_complete` with state checks
- [ ] Update `StableBlobSink::for_meta` to accept `UploadMeta` + `SessionId`
- [ ] Use stored `pmid_stem` (never recompute)
- [ ] All tests passing
- [ ] Commit: `refactor(backend): remove SessionCompat, call SessionService directly`

### Phase 3

- [ ] Move in-order enforcement to `SessionService::put_chunk`
- [ ] Add active session limits to `SessionService::begin_with_id`
- [ ] Add `SessionService::tick_ttl` for cleanup
- [ ] Call `tick_ttl` periodically (heartbeat or timer)
- [ ] All tests passing
- [ ] Commit: `refactor(backend): move policies to session layer`

### Phase 4

- [ ] Remove all debug logs except 3 finish checkpoints
- [ ] Add `debug_session_stats` behind `cfg!(debug_assertions)`
- [ ] Add static guards (chunk_size, bytes_expected, alignment)
- [ ] Remove canary endpoints
- [ ] All tests passing
- [ ] Commit: `refactor(backend): clean up logging, add observability`

### Phase 5

- [ ] Delete `src/backend/src/session/compat.rs`
- [ ] Remove compat exports from `session/mod.rs`
- [ ] Remove all compat imports
- [ ] Update ARCHITECTURE.md
- [ ] Update IMPLEMENTATION_GUIDE.md
- [ ] Update CHANGELOG.md
- [ ] All tests passing
- [ ] Commit: `refactor(backend): delete SessionCompat, refactoring complete`

---

## 📚 Related Documentation

- **[upload-session/REFACTORING_TODO.md](./upload-session/REFACTORING_TODO.md)** - Original plan (less detailed)
- **[upload-session/ARCHITECTURE.md](./upload-session/ARCHITECTURE.md)** - Current architecture
- **[upload-session/IMPLEMENTATION_GUIDE.md](./upload-session/IMPLEMENTATION_GUIDE.md)** - How it works now

---

## 🔗 References

- Current implementation: `src/backend/src/session/compat.rs`
- Target architecture: `src/backend/src/session/service.rs`
- Upload service: `src/backend/src/upload/service.rs`
- Tests: `tests/backend/shared-capsule/upload/session/`

---

**Created**: 2025-10-01  
**Status**: 📋 OPEN - Ready to start  
**Priority**: Medium (technical debt, not blocking)  
**Next Step**: Phase 0 (mark deprecated, add comment)  
**Estimated Completion**: 1-2 weeks (incremental, low-risk)

This gets you a clean, maintainable architecture without risking your now-green system.
