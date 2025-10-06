# ICP Inline Memory Creation Flow (Frontend → Backend)

Status: draft

## Goal

Implement a reliable end-to-end flow for creating small memories (inline) on the ICP canister when:

- The user is authenticated with Internet Identity (II)
- The user’s storage setting prefers `icp-canister`

This issue focuses ONLY on inline creation (small files). Chunked upload is out of scope here.

## Scope

- Frontend (Next.js): `user-file-upload.ts`, `icp-upload.ts`, settings UI
- Backend (Rust canister): `memories_create` (inline)
- AuthN/AuthZ: Internet Identity (II) principal on canister side; NextAuth session optional for app UI

## Current State (as of this issue)

- Frontend selects storage via `use-upload-storage` and routes to `icpUploadService` when `chosen_storage === "icp-canister"`.
- `icpUploadService` ensures II auth via `AuthClient.isAuthenticated()` and builds an `HttpAgent` with II identity.
- For small files: should call inline create (`memories_create`).
- Backend endpoint exists in `lib.rs` and delegates to module logic:
  - `#[update] memories_create(capsule_id, memory_data, idem) -> MemoryId`

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
