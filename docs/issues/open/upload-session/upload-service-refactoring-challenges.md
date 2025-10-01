# Backend Upload Service Refactoring Challenges

**Status**: 🔴 **BLOCKED** - Need tech lead guidance for practical implementation  
**Priority**: **HIGH** - Required to complete hybrid architecture  
**Assignee**: Tech Lead  
**Created**: 2024-12-19  
**Updated**: 2024-12-19

## Problem Statement

We've successfully implemented the **generic session module** according to your hybrid architecture recommendation, but we're **blocked** on refactoring the upload service to work with the new generic session interface. The upload service has **20+ compilation errors** due to method signature mismatches.

## What We've Completed ✅

### Generic Session Module (Phase 1 Complete)

- ✅ **Generic types**: `SessionId`, `SessionSpec`, `SessionMeta`, `ByteSink`, `Clock`
- ✅ **Generic SessionService**: Pure Rust, no upload semantics
- ✅ **SessionAdapter**: IC-specific wrapper
- ✅ **Removed chunk buffering**: No `BTreeMap<u32, Vec<u8>>` in sessions
- ✅ **ByteSink trait**: For direct chunk writing

### Current Architecture

```
session/
├── types.rs          # Generic session types (SessionId, SessionSpec, etc.)
├── service.rs        # Generic SessionService (no upload semantics)
└── adapter.rs        # IC SessionAdapter wrapper
```

## What's Blocking Us ❌

### Upload Service Method Mismatches (20+ errors)

The upload service expects these methods on `SessionAdapter`:

```rust
// MISSING METHODS in SessionAdapter:
.find_pending(&capsule_id, &caller, &idem)           // Find existing session
.create(session_id, session_meta)                    // Create session
.get(session_id)                                      // Get session
.put_chunk(session_id, chunk_idx, bytes)              // Put chunk (old signature)
.verify_chunks_complete(session_id, chunk_count)     // Verify chunks
.cleanup(session_id)                                  // Cleanup session
.update(session_id, session)                         // Update session
.iter_chunks(session_id, chunk_count)                // Iterate chunks
.cleanup_expired_sessions_for_caller(...)            // Caller-specific cleanup
```

### Type Conflicts

- **SessionStatus enum**: Both `upload::types::SessionStatus` and `session::types::SessionStatus` exist
- **SessionMeta fields**: Upload service expects `caller`, `capsule_id`, `created_at` fields that don't exist in generic `SessionMeta`

### ByteSink Implementation Gap

- Upload service needs to provide `ByteSink` implementation for stable/blob storage
- Current `put_chunk` calls don't provide `ByteSink` parameter

## Specific Compilation Errors

### 1. Missing Methods (15+ errors)

```rust
error[E0599]: no method named `find_pending` found for struct `SessionAdapter`
error[E0599]: no method named `create` found for struct `SessionAdapter`
error[E0599]: no method named `get` found for struct `SessionAdapter`
error[E0599]: no method named `verify_chunks_complete` found for struct `SessionAdapter`
error[E0599]: no method named `cleanup` found for struct `SessionAdapter`
error[E0599]: no method named `iter_chunks` found for struct `SessionAdapter`
```

### 2. Method Signature Mismatches (5+ errors)

```rust
error[E0061]: this method takes 1 argument but 2 arguments were supplied
// .count_active_for(&capsule_id, &caller) - expects only &caller

error[E0061]: this method takes 4 arguments but 3 arguments were supplied
// .put_chunk(session_id, chunk_idx, bytes) - missing ByteSink parameter
```

### 3. Type Conflicts (3+ errors)

```rust
error[E0308]: mismatched types
// upload::types::SessionStatus vs session::types::SessionStatus

error[E0609]: no field `caller` on type `session::types::SessionMeta`
// Upload service expects fields that don't exist in generic SessionMeta
```

## Questions for Tech Lead

### 1. **Method Compatibility Strategy**

Should we:

- **A)** Add all missing methods to `SessionAdapter` to maintain compatibility?
- **B)** Refactor upload service to use new generic interface?
- **C)** Create a compatibility layer between old and new interfaces?

### 2. **Type Conflicts Resolution**

How should we handle:

- **SessionStatus enums**: Merge into one, or keep separate with conversion?
- **SessionMeta fields**: Add missing fields to generic `SessionMeta`, or create upload-specific wrapper?

### 3. **ByteSink Implementation**

Where should we implement `ByteSink` for upload service:

- **A)** In `upload::service` module?
- **B)** In `upload::blob_store` module?
- **C)** As a separate `upload::storage` module?

### 4. **Migration Strategy**

Should we:

- **A)** Implement all missing methods first (quick fix)?
- **B)** Refactor upload service to match new architecture (proper solution)?
- **C)** Create a hybrid approach with compatibility layer?

## Current Upload Service Dependencies

The upload service currently depends on these session operations:

```rust
// Session lifecycle
.find_pending(capsule_id, caller, idem) -> Option<SessionId>
.create(session_id, session_meta) -> Result<(), Error>
.get(session_id) -> Result<Option<SessionMeta>, Error>
.update(session_id, session) -> Result<(), Error>

// Chunk operations
.put_chunk(session_id, chunk_idx, bytes) -> Result<(), Error>
.verify_chunks_complete(session_id, chunk_count) -> Result<(), Error>
.iter_chunks(session_id, chunk_count) -> ChunkIterator

// Cleanup operations
.cleanup(session_id) -> ()
.cleanup_expired_sessions_for_caller(capsule_id, caller, expiry_ms) -> ()
.count_active_for(capsule_id, caller) -> usize
```

## Proposed Solutions

### Option A: Compatibility Layer (Quick Fix)

Add all missing methods to `SessionAdapter` that delegate to generic `SessionService`:

```rust
impl SessionAdapter {
    pub fn find_pending(&self, capsule_id: &CapsuleId, caller: &Principal, idem: &str) -> Option<SessionId> {
        // Implementation using generic SessionService
    }

    pub fn create(&self, session_id: SessionId, session_meta: SessionMeta) -> Result<(), Error> {
        // Implementation using generic SessionService
    }

    // ... all other missing methods
}
```

### Option B: Refactor Upload Service (Proper Solution)

Update upload service to use new generic interface:

```rust
// Instead of: self.sessions.find_pending(&capsule_id, &caller, &idem)
// Use: self.sessions.begin(spec) where spec contains all needed info

// Instead of: self.sessions.put_chunk(session_id, chunk_idx, bytes)
// Use: self.sessions.put_chunk(session_id, chunk_idx, &bytes, &mut sink)
```

### Option C: Hybrid Approach

Create a compatibility wrapper that implements old interface using new generic interface.

## Files Affected

### Backend Files

- `src/backend/src/upload/service.rs` - Main upload service (891 lines)
- `src/backend/src/upload/blob_store.rs` - Blob storage (494 lines)
- `src/backend/src/session/adapter.rs` - Session adapter (152 lines)
- `src/backend/src/lib.rs` - Main canister endpoints (967 lines)

### Test Files

- `tests/backend/shared-capsule/upload/test_upload_2lane_4asset_system.mjs` - Main test
- All other upload tests

## Business Impact

- **MVP Delivery**: Blocked until upload service refactoring is complete
- **2-Lane + 4-Asset System**: Cannot test until compilation errors are resolved
- **Session Management**: Generic architecture is ready, but upload integration is incomplete

## Decision Required

**Tech Lead Guidance Needed**:

1. **Which approach** should we take for method compatibility?
2. **How to resolve** type conflicts between upload and session modules?
3. **Where to implement** ByteSink for upload service?
4. **Migration strategy** for minimal disruption?

## Timeline Impact

- **Option A (Compatibility)**: 1-2 days to implement missing methods
- **Option B (Refactor)**: 3-5 days to refactor upload service
- **Option C (Hybrid)**: 2-3 days to create compatibility layer

---

**Context**: We've successfully implemented the generic session architecture per your recommendation, but need practical guidance on integrating it with the existing upload service without breaking functionality.
