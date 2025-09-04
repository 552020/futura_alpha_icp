# Memory API Unification – TODO Checklist

## Phase 1 – Documentation & Organization (Now)

- [ ] Regroup `lib.rs` memory endpoints under one section:
  - [ ] Create single section header: `MEMORY MANAGEMENT`
  - [ ] Add subheaders: `Core (CRUD)`, `Metadata & Presence`, `Upload (Advanced)`
  - [ ] Verify function counts and comments are accurate
- [ ] Update docs:
  - [ ] Add decision tree (BlobRef ingest vs inline ≤32KB vs chunked >32KB)
  - [ ] Expand authorization section (canister principal, capsule owner/controller, NextAuth/session)
  - [ ] Cross-link from `docs/issues/lib-rs-reorganization-plan.md`
  - [ ] Ensure `docs/issues/memory-api-unification-analysis.md` reflects latest structure
- [ ] Sanity checks:
  - [ ] `cargo check` passes with zero warnings
  - [ ] No public API changes in this phase (no DID delta)
  - [ ] Prepare PR notes (scope: structure/docs only)

## Phase 2 – Client Abstraction in Next.js (Near Term)

- [ ] Implement small TypeScript upload client (used by Next.js routes):
  - [ ] Detect file size and choose inline vs chunked
  - [ ] Inline path: call `memories_create_inline` with metadata
  - [ ] Chunked path: `begin_upload` → `put_chunk*` (64KB) → `commit` with SHA-256 and total_len
  - [ ] Robust error handling/retry/backoff for chunk uploads
  - [ ] Return typed results and normalized errors to routes
- [ ] Wire Next.js routes to use the client:
  - [ ] Keep existing validation (file-type, mime, size) intact
  - [ ] Preserve NextAuth/DB flows and responses
  - [ ] Ensure onboarding and UX remain unchanged
- [ ] Tests & QA:
  - [ ] Unit tests for client logic (size branching, retries)
  - [ ] Route integration test (happy/large/invalid cases)
  - [ ] Manual QA: big files, network interruptions, auth errors

## Phase 3 – Optional Unified Façade for Programmatic Consumers

- [ ] Add façade endpoint:
  - [ ] `memories_create(capsule_id, payload: variant { Inline; Large })`
  - [ ] Map `Inline` → inline flow; `Large` → return session or orchestrate server-side
- [ ] Keep advanced endpoints public but document as “advanced”
- [ ] Consider unifying internal creation path:
  - [ ] Make `capsule.rs::memories_create` delegate to `UploadService` (single code path)
- [ ] Update Candid and CI:
  - [ ] Regenerate DID and update baseline
  - [ ] Ensure candid-diff passes
- [ ] Docs:
  - [ ] Update analysis and API docs to include façade

## Acceptance Criteria

- [ ] `lib.rs` shows a single `MEMORY MANAGEMENT` section with clear subheaders
- [ ] Decision tree and auth model mapping are documented
- [ ] Next.js routes use the TS upload client without UX regressions
- [ ] (Optional) Unified façade exists; advanced endpoints remain available
- [ ] CI candid-diff clean when façade is added; `.did` baseline updated

## Risks & Mitigations

- Upload UX regressions → keep routes as primary interface; introduce client under routes only
- Auth mismatches → document boundaries; keep canister strict; perform session/DB auth in routes
- Confusion over path choice → decision tree + examples in docs; label advanced endpoints clearly

## References

- Backend: `src/backend/src/lib.rs`, `src/backend/src/upload/{service.rs,sessions.rs,blob_store.rs,types.rs}`, `src/backend/src/capsule.rs`
- Frontend: `src/nextjs/src/services/upload.ts`, `src/nextjs/src/hooks/user-file-upload.ts`, `src/nextjs/src/services/icp-gallery.ts`
- Docs: `docs/issues/memory-api-unification-analysis.md`, `docs/issues/lib-rs-reorganization-plan.md`, `docs/issues/upload-workflow-implementation-plan-v2.md`
