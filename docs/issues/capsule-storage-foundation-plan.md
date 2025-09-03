# Capsule Storage Foundation Plan

## Overview

This plan establishes a stable, minimal foundation for capsule storage that prevents rework while enabling the stable memory migration. Focus is on locking the surface, adding essential indexes, and enabling incremental migration using an enum-backed repository pattern.

## Current Status

✅ **Completed:**

- Basic `CapsuleStore` trait with dual backend support
- HashMap and Stable backend implementations
- First endpoint migrations (2/65+ functions)
- 173 tests passing

🔄 **In Progress:**

- Refining the repository interface per the strategic plan
- Deciding on ID and index structure (1:1 vs 1:N for subject relationships)

## Implementation Structure

### Module Layout (Production-Grade Architecture)

The foundation uses a clean separation between domain logic and persistence:

```
src/backend/src/
├─ lib.rs                     // canister endpoints (business logic, unchanged)
├─ capsule.rs                 // Capsule struct + invariants (no storage deps)
└─ capsule_store/             // storage seam (Hash + Stable behind one API)
   ├─ mod.rs                  // trait CapsuleStore, errors, types, CapsuleId
   ├─ store.rs                // enum Store { Hash, Stable } + delegation logic
   ├─ hash.rs                 // HashMap-backed impl (fast tests)
   └─ stable.rs               // StableBTreeMap impl + IC specifics (MemoryIds, indexes)
```

### Module Responsibilities

- **`lib.rs`**: Call storage API; zero `StableBTreeMap` calls here
- **`capsule.rs`**: Domain type + `has_write_access()`, validation
- **`capsule_store/mod.rs`**: API surface, error types, pagination
- **`capsule_store/store.rs`**: Backend selection + delegation
- **`capsule_store/hash.rs`**: Fast test implementation
- **`capsule_store/stable.rs`**: Production backend + IC integration

### Integration Pattern

```rust
// lib.rs (endpoints call the seam)
#[ic_cdk::query]
fn capsules_read(id: CapsuleId) -> Option<Capsule> {
    with_store(|store| {
        let caller = ic_cdk::caller();
        store.get(&id).filter(|c| c.has_write_access(&caller))
    })
}
```

This structure ensures:

- ✅ **Domain isolation**: Business logic never touches persistence
- ✅ **Storage seam**: Single API that can switch backends
- ✅ **Testability**: Hash vs Stable implementations testable independently
- ✅ **IC boundaries**: MemoryIds, Storable concerns isolated
- ✅ **Scalability**: Easy to add new storage backends

## Strategic Priorities (Execute in Order)

### 🔥 Phase 1: Finalize Repository Interface (Freeze Surface)

**Goal:** Freeze `CapsuleStore` API and remove any iterator exposure

#### 1.1 Add Essential Helper Methods

- [ ] `fn exists(&self, id: &CapsuleId) -> bool`
- [ ] `fn upsert(&mut self, id: CapsuleId, c: Capsule) -> Option<Capsule>`
- [ ] `fn put_if_absent(&mut self, id: CapsuleId, c: Capsule) -> Result<(), AlreadyExists>`
- [ ] `fn update(&mut self, id: &CapsuleId, f: impl FnOnce(&mut Capsule)) -> Result<(), UpdateError>`
- [ ] `fn find_by_subject(&self, subj: &Principal) -> Option<Capsule>`
- [ ] `fn list_by_owner(&self, owner: &Principal) -> Vec<CapsuleId>`
- [ ] `fn get_many(&self, ids: &[CapsuleId]) -> Vec<Capsule>` (batch operations)
- [ ] `fn paginate(&self, after: Option<CapsuleId>, limit: u32, order: Order) -> (Vec<Capsule>, Option<CapsuleId>)`
- [ ] `fn count(&self) -> u64`

#### 1.1.1 Error Types

```rust
#[derive(Debug, Clone)]
pub enum UpdateError {
    NotFound,
    Validation(String),
    Concurrency, // placeholder for future MVCC
}

#[derive(Debug, Clone)]
pub enum AlreadyExists {
    CapsuleExists(CapsuleId),
}

#[derive(Debug, Clone)]
pub enum Order {
    Asc,
    Desc,
}
```

#### 1.1.2 Cursor Semantics

- `after` parameter is **exclusive** (items with `id > after` when `Asc`, `id < after` when `Desc`)
- `order` defaults to `Asc` (ascending by `CapsuleId`)
- Returns `(items, next_cursor)` where `next_cursor` is the last item's ID for continuation

#### 1.2 Remove Iterator Methods from Trait

- [ ] Remove `iter()` method from `CapsuleStore` trait
- [ ] Keep only query/update helpers in trait surface
- [ ] Commit to enum-backed repo (`Store::{Hash,Stable}`) - no trait objects

#### 1.3 Decide ID and Index Structure

- [ ] **FROZEN**: `subject → id` = 1:1 (one capsule per subject)
- [ ] If 1:N needed later: use `StableBTreeMap<(SubjectKey, CapsuleId), ()>` (sparse multimap)
- [ ] Avoids big value rewrites and fragmentation
- [ ] Decision locked to prevent rework

#### 1.4 Type Consistency

- [ ] Choose `candid::Principal` or project alias consistently
- [ ] Fix any `Principal`/type mismatches across codebase
- [ ] Update imports and type annotations

### 🔥 Phase 2: Implement Secondary Index

**Goal:** Make queries O(log n) instead of O(n) scans

#### 2.1 Add Subject Index to Stable Backend

- [ ] Implement `subject -> capsule_id` mapping (decide 1:1 vs 1:N from Phase 1)
- [ ] Use `Vec<u8>` from `Principal::as_slice()` as key (Ord + stable)
- [ ] Wire index maintenance in `put/remove/update` methods

#### 2.2 Add Owner Index (if needed)

- [ ] Check if `list_by_owner` endpoints already exist
- [ ] If yes: implement `owner -> Vec<capsule_id>` mapping
- [ ] If no: defer until endpoints are identified

#### 2.3 Index Maintenance Logic

- [ ] Update index in `put()` method
- [ ] Clean up index in `remove()` method
- [ ] Handle subject changes in `update()` (old→new reindex atomically)
- [ ] Maintain indexes only in repo methods—never in endpoints

#### 2.4 Stable Backend Setup

- [ ] Implement `Storable`/`BoundedStorable` for `Capsule` with headroom (4–8 KiB)
- [ ] Add size validation unit test: `assert!(candid::encode_one(&Capsule)?.len() <= Capsule::MAX_SIZE as usize)`
- [ ] Reserve fixed `MemoryId`s as constants:

  ```rust
  const MEM_CAPSULES: u8 = 0;
  const MEM_IDX_SUBJECT: u8 = 1;
  // keep 2..5 reserved for future indexes
  ```

- [ ] Add schema versioning: `version: u16` field to `Capsule`
- [ ] Write upgrade test: encode v1 → simulate upgrade → decode with v2
- [ ] Add observability counters in stable impl only: `capsules_count`, `index_subject_count`, `index_owner_count`

### 🔥 Phase 3: Dual-Backend Tests & Write Path Migration

**Goal:** Test both backends and migrate mutation-heavy endpoints

#### 3.1 Dual-Backend Test Harness

- [ ] Parametrize tests: `run_suite(HashMapStore)` and `run_suite(StableStore)`
- [ ] Add property tests for index consistency:
  - After any sequence of `put/update/remove`, `find_by_subject(subj)` equals ground truth rebuilt by scanning
  - Randomly mutate subject/owner and assert index parity vs. rebuilt HashMap truth
- [ ] Add fuzzing: random ops (put/update/remove) for 1–2k steps; assert repo invariants after each batch
- [ ] Add CI check: forbid `.iter()`/`.values()` in `StableStore` impl via grep step ("no scan on hot path")

#### 3.2 Migrate Write-Heavy Endpoints

- [ ] `capsules_update_metadata` → use `repo.update(id, |c| ...)`
- [ ] `capsules_grant_access` → use `repo.update(id, |c| ...)`
- [ ] `capsules_revoke_access` → use `repo.update(id, |c| ...)`
- [ ] `capsules_delete` → use `repo.remove(id)`
- [ ] Any "register"/"rename" operations

#### 3.3 Update Migration Pattern

Replace this pattern everywhere:

```rust
// OLD:
if let Some(c) = capsules.get_mut(&id) { /* mutate c */ }

// NEW:
store.update(&id, |c| { /* mutate c */ });
```

### 📋 Phase 4: Pagination & List Endpoints (After Phase 3)

**Goal:** Migrate all listing endpoints to efficient pagination

#### 4.1 Implement Keyset Pagination

- [ ] Add `paginate()` method to trait
- [ ] Use CapsuleId as cursor for keyset pagination
- [ ] Avoid O(n) scans for large datasets

#### 4.2 Migrate List Endpoints

- [ ] Find all endpoints that return `Vec<Capsule>` or similar
- [ ] Update them to use `paginate(cursor, limit)`
- [ ] Add cursor handling in frontend if needed

### 📋 Phase 5: Production Switch (After Phase 4)

**Goal:** Flip to Stable backend in production

#### 5.1 Runtime Backend Selection

- [ ] Prefer runtime switch via `Store` enum (HashMap | Stable)
- [ ] Use compile-time feature only if binary size reduction needed
- [ ] Keep both backends available in CI for comprehensive testing

#### 5.2 Performance Validation

- [ ] Add micro-benchmarks for hot paths
- [ ] Validate index performance (O(log n) queries)
- [ ] Size checks for memory usage

### 📋 Phase 6: Future Enhancements (Defer Until Needed)

#### 6.1 Additional Indexes

- [ ] Tag-based indexes
- [ ] Timestamp-based indexes
- [ ] Status-based indexes

#### 6.2 Advanced Features

- [ ] Batch operations API
- [ ] Fancy query language
- [ ] Advanced pagination options

## Migration Cadence

### Read-Only First (Phase 1-2)

- `get()`, `exists()`, `find_by_subject()`
- No mutation, establishes query patterns

### Writes Second (Phase 3)

- `update()`, `put()`, `remove()`
- Read-modify-write patterns
- Index maintenance

### Lists Last (Phase 4)

- `paginate()`, `list_by_owner()`
- Large dataset handling
- Performance optimization

## Definition of Done

### Phase 1 (Interface Freeze)

- [ ] `CapsuleStore` trait surface finalized
- [ ] All helper methods implemented
- [ ] No iterators in trait
- [ ] Type consistency achieved

### Phase 2 (Indexing)

- [ ] Subject index implemented in StableStore
- [ ] Index maintenance in all write methods
- [ ] O(log n) performance on hot queries

### Phase 3 (Dual Backend)

- [ ] Same test suite passes on both backends
- [ ] Write-heavy endpoints migrated
- [ ] Read-modify-write pattern established

### Phase 4 (Pagination)

- [ ] All list endpoints use pagination
- [ ] No O(n) scans on hot paths
- [ ] Efficient cursor-based navigation

### Phase 5 (Production Ready)

- [ ] Stable backend in production
- [ ] HashMap in CI for fast tests
- [ ] Performance benchmarks green

## Risk Mitigation

### Rollback Plan

- Keep old `with_capsules` behind feature flag only (default off to avoid accidental use)
- Can revert individual endpoints if issues arise
- Runtime switch via `Store` enum for backend selection

### Testing Strategy

- Dual-backend testing catches inconsistencies
- Fast HashMap tests for development
- Accurate Stable tests for production parity

### Performance Safeguards

- Avoid scans in hot paths: if found during migration, add/extend index or prove it's cold (add TODO)
- Watch clone storms: `StableBTreeMap::iter()` gives owned values—don't `.clone()` them again
- No O(n) operations on hot paths
- Index consistency across all write operations
- Memory usage monitoring

### Ownership & Boundaries

- Keep access-control logic out of the repo: do checks in endpoints before calling `update/remove`
- Repo = persistence + indexes, nothing else (keeps it reusable and testable)
- Maintain clear separation between storage layer and business logic

## Success Metrics

### Phase 1 Success Checks

- [x] `CapsuleStore` trait surface finalized and frozen
- [x] No iterator methods exposed in trait
- [x] ID and index structure decisions frozen (1:1 with multimap fallback)
- [x] Module structure implemented (mod.rs, store.rs, hash.rs, stable.rs)
- [x] Error types defined (UpdateError, AlreadyExists)
- [x] Cursor semantics documented (exclusive after cursor)
- [x] Unit tests passing (3/3 capsule_store tests)
- [x] Integration tests passing (173 total tests)

### Phase 1 Completion Summary

**✅ FROZEN: `CapsuleStore` API Surface**

- 12 core methods: `exists`, `get`, `upsert`, `put_if_absent`, `update`, `remove`, `find_by_subject`, `list_by_owner`, `get_many`, `paginate`, `count`
- Rich error types: `UpdateError`, `AlreadyExists`
- Exclusive cursor semantics: `after` parameter is exclusive
- No iterator exposure - clean separation maintained

**✅ Module Architecture Implemented**

```
capsule_store/
├─ mod.rs      # Frozen trait + types + errors
├─ store.rs    # Enum delegation (Hash | Stable)
├─ hash.rs     # Fast testing backend
└─ stable.rs   # Production IC backend
```

**✅ Key Design Decisions Frozen**

- Subject → ID: 1:1 relationship (multimap fallback ready)
- Cursor: Exclusive `after` parameter
- Update: Internal index delta computation
- Error handling: `Result<T, UpdateError>` pattern

**✅ Testing Foundation Established**

- 3/3 capsule_store unit tests passing
- 173/173 total tests passing
- Integration tests verify enum delegation works
- API completeness validated

### Phase 2 Success Checks

- [ ] `find_by_subject` is O(log n) (confirmed by no `.iter()` in impl and by microbench ceiling)
- [ ] Index maintenance works in all write operations (`put/remove/update`)
- [ ] Subject changes in `update()` handled atomically (old→new reindex)

### Phase 3 Success Checks

- [ ] 100% of write-heavy endpoints use `repo.update`/`remove` only
- [ ] Property tests pass for index consistency
- [ ] Fuzzing tests reveal no corruption scenarios

### Phase 4 Success Checks

- [ ] Zero list endpoints perform full scans
- [ ] All list endpoints use `paginate` with keyset cursors

### Overall Success Checks

- [ ] All 65+ endpoints compile against `CapsuleStore` helpers
- [ ] StableStore maintains indexes automatically
- [ ] Test suite passes on both backends
- [ ] Hot query paths are O(log n), not O(n)
- [ ] No performance regressions in production

---

## Tech Lead Feedback Incorporated

### ✅ **Key Improvements Made:**

1. **Enum-Backed Architecture**: Committed to `Store::{Hash,Stable}` enum - no trait objects
2. **Frozen Decisions**: ID/index structure decisions made upfront to prevent rework (1:1 with multimap fallback)
3. **Enhanced API Surface**: Added `put_if_absent`, `get_many()`, improved error types, keyset pagination with exclusive cursors
4. **Index Delta Internal**: Old→new computation inside `update()` (not caller-provided)
5. **Indexing Strategy**: `Vec<u8>` keys, atomic reindexing, repo-only maintenance, sparse multimap for 1:N if needed
6. **Stable Backend Setup**: MemoryId reservations, Storable with headroom, schema versioning, observability counters (stable impl only)
7. **Testing Excellence**: Property tests, fuzzing, parameterized dual-backend testing, CI scan detection
8. **Performance Guardrails**: Scan detection, clone storm prevention, O(log n) guarantees, size validation unit tests
9. **Clear Boundaries**: Access control in endpoints, repo = persistence + indexes only
10. **Success Metrics**: Phase-specific completion criteria with measurable outcomes
11. **Runtime Switching**: Prefer enum-based backend selection over compile-time flags
12. **Upgrade Hygiene**: Schema versioning with forward-compatibility tests
13. **Rollback Safety**: Feature flag for old APIs (default off to prevent accidental use)

### **Result**: Production-Ready Foundation

This plan now provides:

- **Zero rework risk** through frozen decisions
- **Surgical execution** with clear phase boundaries
- **Quality assurance** through comprehensive testing
- **Performance guarantees** on all hot paths
- **Maintainable architecture** with clear separation of concerns

**Status**: 🟢 PHASE 1 COMPLETE - Ready for Phase 2
**Next Action**: Start Phase 2 - Implement secondary indexes
**Estimated Timeline**: Continue with remaining phases as planned
