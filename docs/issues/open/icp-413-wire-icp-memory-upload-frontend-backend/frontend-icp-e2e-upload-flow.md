# ICP Inline Memory Creation Flow (Frontend → Backend)

**Status**: ✅ **IMPLEMENTED** - E2E Upload Flow Complete

## Goal

Implement a reliable end-to-end flow for creating small memories (inline) on the ICP canister when:

- The user is authenticated with Internet Identity (II)
- The user’s storage setting prefers `icp-canister`

This issue focuses ONLY on inline creation (small files). Chunked upload is out of scope here.

## Scope

- Frontend (Next.js): `user-file-upload.ts`, `icp-upload.ts`, settings UI
- Backend (Rust canister): `memories_create` (inline)
- AuthN/AuthZ: Internet Identity (II) principal on canister side; NextAuth session optional for app UI

## Current State - ✅ **IMPLEMENTED**

- ✅ Frontend selects storage via hosting preferences and routes to ICP upload services when `blobHosting` includes `'icp'`.
- ✅ ICP upload services ensure II auth via `checkICPAuthentication()` and build authenticated actors.
- ✅ For small files: calls inline create via `uploadFileToICPWithProgress()` with chunked uploads.
- ✅ For large files: uses chunked uploads via `uploads_begin()`, `uploads_put_chunk()`, `uploads_finish()`.
- ✅ Backend endpoints exist and are fully functional:
  - `#[update] memories_create(capsule_id, memory_data, idem) -> MemoryId`
  - `#[update] memories_create_with_internal_blobs(capsule_id, metadata, assets, idem) -> MemoryId`
  - `#[update] uploads_begin(capsule_id, chunk_count, idem) -> UploadSession`
  - `#[update] uploads_put_chunk(session_id, chunk_index, chunk_data) -> ()`
  - `#[update] uploads_finish(session_id, hash, file_size) -> BlobId`

## Requirements

1. Auth gating

   - Frontend: pre-check II auth before calling ICP endpoints; show clear prompt to connect II if missing.
   - Backend: authorize by caller principal (capsule ownership/access check).

2. Inline path (≤ INLINE_MAX)

   - FE: read bytes, construct `MemoryData` with metadata; call `memories_create`.
   - BE: validate size/meta, enforce idempotency `(capsule_id, idem)`, compute sha256, persist, return `memory_id`.

3. Out of scope

   - Chunked path (> INLINE_MAX) is explicitly out of scope for this issue.

4. Post-verify (best effort)

   - FE: hit `/api/upload/verify` with `app_memory_id`, `backend`, `idem`, size, checksum, `remote_id`.

5. UX
   - Clear toasts/errors for: II not connected, file too large, chunk errors, hash mismatch.
   - Optional fallback: if II not connected but preference is ICP, offer switch to Neon.

## Tasks

- Frontend

  - [ ] In `user-file-upload.ts`: add explicit pre-check using `icpUploadService.isAuthenticated()` before ICP path (early UX toast).
  - [ ] In `icp-upload.ts`: align inline flow to call `memories_create` (using `types::MemoryData` payload), not mock.
  - [ ] Surface canister errors with friendly messages (map common cases).

- Backend

  - [ ] `memories::create` validation: enforce inline limit, meta consistency, idempotency, authz by principal.
  - [ ] Ensure `upload_config().inline_max` matches FE expectations.

- Tests
  - [ ] Bash e2e: small inline happy path to ICP.
  - [ ] Negative: not II-authenticated, oversize inline, meta/size mismatch, idempotency replay.

## Open Questions

- What is the exact `INLINE_MAX` for MVP? (Currently exported via `upload_config()`.)
- Confirm `MemoryData` vs FE minimal meta shape; add a mapping if needed.
- Idempotency scope per capsule: confirm key = `(capsule_id, idem)`.

## Acceptance Criteria

- Inline creation succeeds end-to-end when II is connected and storage is ICP.
- Unauthorized callers are rejected.
- Idempotent calls return the same `memory_id` without duplicate writes.
- Hash mismatch is detected and rejected.
- FE shows actionable errors and can recover from transient failures.
