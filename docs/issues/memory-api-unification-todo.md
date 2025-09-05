# Memory API Unification – TODO Checklist

## Phase 0 – Critical Memory Creation Fixes ✅ COMPLETE

0. [x] **Fix Memory Creation Implementation** (See `memory-create-implementation.md` TODO section):
       0.1. [x] Fix MemoryId return issue - capture actual ID from closure instead of generating new one ✅ DONE
       0.2. [x] Fix store.update closure return pattern - use get_mut or capture in outer mutable ✅ DONE
       0.3. [x] Implement idempotency/dedupe logic using (capsule_id, sha256, len, idem) tuple ✅ DONE
       0.4. [x] Consolidate to single BlobRef type in types.rs - remove upload::types::BlobRef ✅ DONE
       0.5. [x] Move inline budget accounting to shared finalize_new_memory_locked function ✅ DONE
       0.6. [x] Fix double hashing - use single source of truth for SHA256 computation ✅ DONE
       0.7. [x] Remove unused imports/helpers (compute_sha256, Order, time) ✅ DONE
       0.8. [x] Ensure error consistency - return Result<MemoryId, Error> everywhere, avoid expect() ✅ DONE
       0.9. [x] Remove public memories_create_inline endpoint (replaced by unified create) ✅ DONE
       0.10. [x] Update lib.rs facade to use new Result<MemoryId, Error> signature ✅ DONE
       0.11. [x] Fix all 77 compiler warnings (unused imports, dead code, deprecated functions) ✅ DONE

## Phase 1 – Documentation & Organization 🚧 IN PROGRESS

1. [ ] Regroup `lib.rs` memory endpoints under one section:
       1.1. [x] Create single section header: `MEMORIES` ✅ DONE
       1.2. [x] Add subheaders: `Core`, `Metadata & Presence` ✅ DONE (Upload needs work)
       1.3. [x] **Core subheader** - Move these functions: 1. [x] `memories_create` (unified Inline/BlobRef) 2. [x] `memories_read` 3. [x] `memories_update` 4. [x] `memories_delete` 5. [x] `memories_list` ✅ DONE
       1.4. [ ] **Metadata & Presence subheader** - Move these functions: 1. [ ] Keep `memories_ping` as primary endpoint (remove `memories_presence`) 2. [x] `upsert_metadata` ✅ DONE
       1.5. [ ] **Upload subheader** - Rename and move these functions: 1. [ ] `memories_begin_upload` → `uploads_begin` 2. [ ] `memories_put_chunk` → `uploads_put_chunk` 3. [ ] `memories_commit` → `uploads_finish` 4. [ ] `memories_abort` → `uploads_abort`
       1.6. [ ] Verify function counts and comments are accurate
2. [ ] Update docs:
       2.1. [ ] Add decision tree (BlobRef ingest vs inline ≤32KB vs chunked >32KB)
       2.2. [ ] Expand authorization section (canister principal, capsule owner/controller, NextAuth/session)
       2.3. [ ] Cross-link from `docs/issues/lib-rs-reorganization-plan.md`
       2.4. [ ] Ensure `docs/issues/memory-api-unification-analysis.md` reflects latest structure
3. [ ] Sanity checks:
       3.1. [ ] `cargo check` passes with zero warnings
       3.2. [ ] Regenerate Candid interface (`.did` file) after endpoint renaming
       3.3. [ ] Update frontend service calls to use new `uploads_*` endpoint names
       3.4. [ ] Prepare PR notes (scope: endpoint renaming + structure/docs)

## Phase 2 – Client Abstraction in Next.js (Near Term)

4. [ ] Implement small TypeScript upload client (used by Next.js routes):
       4.1. [ ] Detect file size and choose inline vs chunked
       4.2. [ ] Inline path: call unified `memories_create` with Inline payload
       4.3. [ ] Chunked path: `uploads_begin` → `uploads_put_chunk*` (64KB) → `uploads_finish` with SHA-256 and total_len
       4.4. [ ] Robust error handling/retry/backoff for chunk uploads
       4.5. [ ] Return typed results and normalized errors to routes
5. [ ] Wire Next.js routes to use the client:
       5.1. [ ] Keep existing validation (file-type, mime, size) intact
       5.2. [ ] Preserve NextAuth/DB flows and responses
       5.3. [ ] Ensure onboarding and UX remain unchanged
6. [ ] Tests & QA:
       6.1. [ ] Unit tests for client logic (size branching, retries)
       6.2. [ ] Route integration test (happy/large/invalid cases)
       6.3. [ ] Manual QA: big files, network interruptions, auth errors

## Phase 3 – Optional Unified Façade for Programmatic Consumers

7. [ ] Add façade endpoint:
       7.1. [ ] `memories_create(capsule_id, payload: variant { Inline; Large })`
       7.2. [ ] Map `Inline` → inline flow; `Large` → return session or orchestrate server-side
8. [ ] Keep advanced endpoints public but document as "advanced"
9. [ ] Consider unifying internal creation path:
       9.1. [ ] Make `capsule.rs::memories_create` delegate to `UploadService` (single code path)
10. [ ] Update Candid and CI:
        10.1. [ ] Regenerate DID and update baseline
        10.2. [ ] Ensure candid-diff passes
11. [ ] Docs:
        11.1. [ ] Update analysis and API docs to include façade

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
