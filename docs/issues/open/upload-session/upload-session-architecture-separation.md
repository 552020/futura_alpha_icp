# Backend Upload vs Session Architecture Separation

**Status**: Open  
**Priority**: Medium  
**Assigned**: Tech Lead  
**Created**: 2024-12-19

## Problem Statement

We have a **logical separation issue** between `upload` and `session` modules in our backend architecture. The current organization mixes responsibilities and creates confusion about where different functionalities should live.

## Current Architecture

### Upload Functions (in `lib.rs`):

```rust
fn uploads_begin(capsule_id, asset_metadata, expected_chunks, idem) -> Result_13
async fn uploads_put_chunk(session_id, chunk_idx, bytes) -> Result<(), Error>
async fn uploads_finish(session_id, expected_sha256, total_len) -> Result_15
async fn uploads_abort(session_id) -> Result<(), Error>
```

### Session Functions (in `lib.rs`):

```rust
fn sessions_clear_all() -> Result<String, Error>
fn sessions_stats() -> Result<String, Error>
fn sessions_list() -> Result<String, Error>
```

## The Core Question

**Should `uploads_*` functions be moved to the `session` module?**

### Arguments FOR Moving to Session Module:

1. **`uploads_begin`** - Creates a session, manages session state
2. **`uploads_put_chunk`** - Stores data in a session
3. **`uploads_finish`** - Commits session data
4. **`uploads_abort`** - Cleans up session

These are fundamentally **session operations** that happen to be used for file uploads.

### Arguments AGAINST Moving to Session Module:

1. **Sessions are abstract** - Could be used for many things (uploads, downloads, processing, etc.)
2. **Upload is the business domain** - File upload is the primary use case
3. **API clarity** - `uploads_*` clearly indicates upload functionality
4. **Separation of concerns** - Upload logic should stay in upload module

## Alternative Approaches

### Option A: Keep Current Structure

- Keep `uploads_*` functions in `upload` module
- Session module provides generic session management
- Upload module uses session module internally

### Option B: Move to Session Module

- Rename `uploads_*` to `session_*`
- Session module handles all session operations
- Upload module focuses on file processing only

### Option C: Hybrid Approach

- `session_begin/put_chunk/finish/abort` - Generic session operations
- `uploads_*` - Upload-specific business logic that uses sessions
- Clear separation between session management and upload business logic

## Technical Considerations

### Current Session Module Structure:

```
session/
├── core.rs          # Pure Rust session logic
├── adapter.rs       # IC-specific session adapter
├── service.rs       # Session service interface
└── types.rs         # Session types
```

### Current Upload Module Structure:

```
upload/
├── service.rs       # Upload service (uses sessions)
├── blob_store.rs    # Blob storage
└── types.rs         # Upload types
```

## Questions for Tech Lead

1. **Should sessions be generic/abstract** or **upload-specific**?
2. **What's the primary use case** for sessions in our system?
3. **How do we balance** between generic session management and upload-specific needs?
4. **What's the long-term vision** for session usage beyond uploads?
5. **Should we have** `session_*` functions that `uploads_*` functions call internally?

## Business Impact

- **Code maintainability** - Clear separation of concerns
- **API consistency** - Logical function organization
- **Future extensibility** - Sessions for other use cases
- **Developer experience** - Easy to find and modify functionality

## Tech Lead Decision & Recommendation

**Status**: ✅ **DECIDED** - Tech Lead provided clear architectural guidance  
**Decision Date**: 2024-12-19  
**Decision**: **Hybrid (Option C)** - Keep `uploads_*` API, make sessions generic

### 🎯 **Tech Lead Recommendation: Hybrid Approach**

> **Keep the external API as `uploads_*` (clear business naming). Keep `session` as a generic module (no upload semantics). Make `uploads_*` call a small, generic session service that does just session lifecycle + chunk book-keeping; all upload semantics stay in `upload`.**

This gives you clean separation, future flexibility, and minimal churn for the team.

## Who Owns What (Single Page Reference)

| Concern                                        | **session** (generic)  | **upload** (business)                 |
| ---------------------------------------------- | ---------------------- | ------------------------------------- |
| ID allocation, TTL, last_seen                  | ✅                     |                                       |
| Begin / Put / Finish / Abort (generic)         | ✅                     |                                       |
| Received index/bitset, bytes_received/expected | ✅                     |                                       |
| Concurrency/isolation guarantees (sid-scoped)  | ✅                     |                                       |
| Where bytes go (sink interface)                | **provided by caller** | ✅ provides sink to stable/blob store |
| Rolling SHA-256 / checksum                     |                        | ✅ (update while writing)             |
| ACL (subject/owner/controller)                 |                        | ✅                                    |
| Asset metadata validation (mime, size, etc.)   |                        | ✅                                    |
| Indexing / catalog / counters                  |                        | ✅                                    |
| API endpoints (`uploads_*`)                    |                        | ✅ (call into session)                |
| Admin ops (`sessions_*` stats/clear)           | ✅ (ops only)          |                                       |

**Key Rule**: The **session** layer knows **nothing** about assets, mime, or hashing. It only manages session lifecycle and per-chunk book-keeping. The **upload** layer decides where bytes are written and updates the rolling hash as the bytes stream.

## Minimal Interfaces (What to Standardize)

### `session::service` (generic, sync helpers)

```rust
pub struct SessionId(pub u64);

pub struct SessionSpec {
  pub chunk_size: usize,
  pub bytes_expected: u64,
  pub idem: String,
  pub owner: candid::Principal, // opaque for session; not interpreted
}

pub trait ByteSink {
  fn write_at(&mut self, offset: u64, data: &[u8]) -> Result<(), Error>;
}

pub trait Clock { fn now_ms(&self) -> u64; }

pub struct SessionService { /* in-heap map + bitset + ttl */ }

impl SessionService {
  pub fn begin(&mut self, spec: SessionSpec) -> SessionId;
  pub fn put_chunk(
    &mut self,
    sid: SessionId,
    idx: u32,
    data: &[u8],
    sink: &mut dyn ByteSink,
  ) -> Result<(), Error>;
  pub fn finish(&mut self, sid: SessionId) -> Result<(), Error>; // just closes session
  pub fn abort(&mut self, sid: SessionId) -> Result<(), Error>;
  pub fn tick_ttl(&mut self, now_ms: u64) -> usize;
}
```

**Notes**:

- `put_chunk` **never** buffers whole chunks; it immediately calls the provided **`ByteSink`** with `offset = idx * chunk_size`
- Session keeps **only**: `bytes_expected`, `bytes_received`, `received_idxs`, `last_seen`, `chunk_size`
- No hashing, no metadata, no index updates here

### `upload::service` (business logic)

```rust
#[ic_cdk::update]
fn uploads_begin(capsule_id: String, meta: AssetMetadata, expected_chunks: u32, idem: String)
  -> Result<SessionId, Error>
{
  // ACL, validate meta, compute bytes_expected, pick chunk_size
  let spec = SessionSpec { chunk_size, bytes_expected, idem, owner: ic_cdk::caller() };
  let sid = SESSION.with(|s| s.borrow_mut().begin(spec));
  // initialize rolling hash state & allocate writer (per-sid) in your upload layer
  Ok(sid)
}

#[ic_cdk::update]
fn uploads_put_chunk(sid: u64, idx: u32, data: Vec<u8>) -> Result<(), Error> {
  let mut sink = StableBlobSink::for_asset(/* capsule_id, asset key, sid */)?;
  // update rolling hash **before/after** write (your choice; usually before)
  SESSION.with(|s| s.borrow_mut().put_chunk(SessionId(sid), idx, &data, &mut sink))?;
  UPLOAD_CTX.with(|u| u.borrow_mut().hasher_for(sid).update(&data));
  Ok(())
}

#[ic_cdk::update]
fn uploads_finish(sid: u64, client_sha256: Vec<u8>, total_len: u64) -> Result<MemoryId, Error> {
  // verify lengths first in upload layer if you prefer (or rely on session counters)
  SESSION.with(|s| s.borrow_mut().finish(SessionId(sid)))?;
  let computed = UPLOAD_CTX.with(|u| u.borrow_mut().finalize_hash_for(sid))?;
  if computed.as_slice() != client_sha256.as_slice() { return Err(Error::ChecksumMismatch); }
  // index the asset, bump counters, write metadata — **before** returning
  commit_and_index(...)?;
  Ok(memory_id)
}
```

## Endpoints: What Stays vs. What Moves

- **Keep** public API names in `lib.rs` as `uploads_*`. That's what clients understand.
- **Keep** admin utilities (`sessions_list`, `sessions_stats`) under a `sessions_*` namespace (ops-only; not used by frontend).
- Internally, have `uploads_*` call into `session::service` with a `ByteSink` and your hashing context.

This satisfies both camps: uploads remain the "face," sessions are the generic engine.

## Migration Plan (No Churn for Juniors)

1. **Freeze names**: do **not** rename public endpoints.
2. **Refactor internals**:
   - Move chunk book-keeping and TTL into `session::service` (if not already).
   - Strip any upload-specific fields out of session state (e.g., mime, sha256, capsule_id). Keep only what sessions need.
   - Ensure `uploads_put_chunk` passes a `ByteSink` and updates the rolling hash outside the session.
3. **Kill heap chunk buffers**: remove `BTreeMap<u32, Vec<u8>>` from sessions.
4. **Add two tests**:
   - Interleaved two-session property test (Begin/Put/Finish/TTL).
   - Post-`finish` retrieval test (index visibility is immediate).
5. **Docs**: one README page with the table above. That's the "source of truth".

## Long-term Vision (Future Session Workflows)

- You can reuse the **same `session::service`** for other workflows (e.g., streamed processing, chunked downloads) by swapping the `ByteSink` and the business logic in those modules.
- Uploads remain clean, and developers know where to look:
  - "session lifecycle or chunk ordering? → session"
  - "asset rules, ACL, hashing, index? → upload"

## Answers to Core Questions

1. **Generic or upload-specific sessions?** → **Generic.** Keep sessions agnostic and slim.
2. **Primary use today?** → Uploads. But don't bake that into session state; you'll reuse it later.
3. **Balance?** → Sessions do lifecycle + book-keeping; Uploads do domain logic + storage + integrity.
4. **Long-term vision?** → Reuse sessions for other streamed workflows; swap sinks and business rules.
5. **Have `session_*` called by `uploads_*`?** → **Yes.** Keep `uploads_*` as the public API; call `session` internally with a `ByteSink`.

## Next Steps

1. **Implement Tech Lead Architecture**: Apply the hybrid approach with clear separation
2. **Refactor Session Module**: Make it generic with `ByteSink` interface
3. **Update Upload Module**: Use session service internally
4. **Add Property Tests**: Interleaved session tests and retrieval tests
5. **Documentation**: Create single-page reference with ownership table

---

**Status**: ✅ **RESOLVED** - Clear architectural direction provided by tech lead
