# Issue: Memory Creation – Backend/Frontend Analysis, Decision Tree, and Plan

## Status

- Draft for senior review
- Externally greenfield, but operationally the Next.js routes are our current user-facing interface
- Goal: Clarify memory creation flows, align backend organization, preserve UX, and create a safe path to gradual improvements

## Executive Summary

- There are three distinct memory creation workflows today:
  1. BlobRef ingest (existing assets) via `memories_create` in `capsule.rs`
  2. Inline uploads (≤32KB) via `memories_create` with inline payload
  3. Chunked uploads (>32KB) via `uploads_begin/put_chunk/finish` workflow
- The Next.js API routes handle upload UX, validation, auth, DB and storage orchestration; they are not trivial to replace and should remain the primary interface for end-users in the near term.
- We will:
  - Consolidate `lib.rs` grouping into a single "MEMORY MANAGEMENT" section with subheaders (Core/Metadata/Upload)
  - Rename chunking endpoints to `uploads_*` for clarity and treat uploads as a first-class resource
  - Unify creation paths through a shared core finalization function
  - Add config discoverability, idempotency, and proper auth scoping
  - Introduce a small TypeScript client used by the Next.js routes to choose inline vs chunked and drive the hybrid flow

---

## Current State (Backend)

### Public endpoints in `lib.rs`

- Core (CRUD): `memories_create`, `memories_read`, `memories_update`, `memories_delete`, `memories_list`
- Metadata & Presence: `upsert_metadata`, `memories_presence` (renamed from `memories_ping`)
- Upload (Advanced): `uploads_begin`, `uploads_put_chunk`, `uploads_finish`, `uploads_abort`
- Config: `upload_config` (returns `{ inline_max, chunk_size, inline_budget_per_capsule }`)

### Upload architecture

- `upload/types.rs`: `INLINE_MAX = 32KB`, `CHUNK_SIZE = 256KB` (configurable 256-1024KB), `CAPSULE_INLINE_BUDGET = 32KB`; session/blob IDs and metadata
- `upload/sessions.rs`: stable sessions with TTL, owner binding, and GC; chunks with verification
- `upload/blob_store.rs`: paged blob storage with hash and length verification; metadata, read/delete/stats
- `upload/service.rs`: unified business logic for inline and chunked flows; shared finalization; authorization; idempotent commit and cleanup

### Unified creation path

- `memories_create`: Routes to inline flow or BlobRef ingest; both delegate to shared core finalization
- `uploads_finish`: Delegates to same shared core finalization as `memories_create`
- Shared core: validation, dedupe by `(capsule_id, sha256, len)`, indexing, auditing

### Authorization (backend)

- Upload service: caller principal authorization, session owner binding, capsule access checks
- Capsule operations: owner/controller checks via `PersonRef`
- Session scoping: `session.capsule_id`, `session.owner_principal`, `created_at`, `ttl`

---

## Current State (Frontend)

- Next.js routes (`/api/memories/upload/*`) are the primary user interface for uploads and are non-trivial:
  - Multi-layer file validation (file-type, mime, size)
  - NextAuth-based authentication and DB lookups
  - Vercel Blob/storage orchestration and Drizzle ORM integration
  - Rich error handling, onboarding flows, and progress/UX
- Direct ICP actor calls for uploads are not wired yet; placeholders exist in `icp-gallery.ts`.
- Presence checks use server routes (not direct `memories_presence`).

Implication: any change to upload flows must preserve route behavior; introducing a TS client under these routes is the safest way to abstract hybrid logic without breaking UX.

---

## Problem Statement

- Fragmentation across Core, Metadata/Presence, and Upload can confuse consumers.
- The three workflows serve different use-cases but are not clearly documented.
- Authorization semantics differ between canister (principal/owner/controller) and frontend (session/DB); we must document and bridge them without weakening canister checks.
- Current chunking endpoints expose implementation details; uploads should be a first-class resource.

---

## Decision Tree: Choosing the Right Workflow

- If you already have a BlobRef (external/pre-staged asset):
  - Use `memories_create(capsule_id, memory_data)` (ingest path)
- Else if the file size ≤ 32KB:
  - Use inline flow: `memories_create(capsule_id, payload: Inline { data, metadata, idempotency_key })`
- Else (file size > 32KB):
  - Use chunked flow: `uploads_begin` → `uploads_put_chunk*` → `uploads_finish` with SHA256 and total_len

Notes:

- The canister continues to enforce authorization and integrity checks regardless of path.
- Per-capsule inline budget is enforced (`CAPSULE_INLINE_BUDGET`).
- All paths require `idempotency_key` for deduplication and retry safety.
- Dedupe by `(capsule_id, sha256, len)` prevents duplicates across different idempotency keys.

---

## API Surface Design

### Memory Creation

```rust
// Single creation endpoint for inline and BlobRef ingest
memories_create(capsule_id: CapsuleId, payload: MemoryCreatePayload) -> Result<MemoryId, Error>

type MemoryCreatePayload = variant {
  Inline: record { data: blob; metadata: MemoryMeta; idempotency_key: string };
  BlobRef: record { blob_ref: BlobRef; metadata: MemoryMeta; idempotency_key: string };
};
```

### Upload Resource (First-Class)

```rust
// Uploads as a first-class resource with clear naming
uploads_begin(capsule_id: CapsuleId, expected_len: nat64, sha256: blob, metadata: MemoryMeta, idempotency_key: string) -> Result<UploadSessionId, Error>
uploads_put_chunk(session_id: UploadSessionId, idx: nat32, bytes: blob) -> Result<(), Error>
uploads_finish(session_id: UploadSessionId) -> Result<MemoryId, Error>
uploads_abort(session_id: UploadSessionId) -> Result<(), Error>
```

### Configuration and Discovery

```rust
upload_config() -> record { inline_max: nat32; chunk_size: nat32; inline_budget_per_capsule: nat32 }
memories_presence(memory_ids: vec string) -> Result<vec MemoryPresenceResult, Error>
```

### Error Normalization

```rust
type Error = variant {
  Unauthorized: null;
  NotFound: null;
  InvalidArgument: string;
  Conflict: string;
  ResourceExhausted: null;
  Internal: string;
};
```

---

## Recommendations (Reprioritized)

### Phase 1 – Documentation & Organization (now)

- Consolidate `lib.rs` Groups 5–7 under one section: "MEMORY MANAGEMENT" with subheaders:
  - `// === Core (CRUD)`
  - `// === Metadata & Presence`
  - `// === Upload (Advanced)`
- Rename endpoints: `memories_ping` → `memories_presence`, `memories_*upload*` → `uploads_*`
- Add this decision tree to docs; clearly document each workflow's purpose and constraints.
- Expand the authorization section to capture: principal checks (canister), owner/controller (capsules), and session-based auth (Next.js routes).

### Phase 2 – Unified Creation Path & TS Client (near term)

- Unify creation paths:
  - Make `capsule.rs::memories_create` delegate to shared core finalization
  - Make `uploads_finish` delegate to same shared core finalization
  - Add dedupe by `(capsule_id, sha256, len)`
- Implement a small TypeScript client used by the Next.js routes:
  - Call `upload_config()` to get current limits
  - Detect size and choose inline vs chunked
  - Stream SHA-256 in browser; retry with exponential backoff
  - Support out-of-order chunk sends; progress callbacks; abort controller
  - Use `idempotency_key` per file; persist in local state to survive reload
  - Keep existing route validation, DB, and UX; no breaking changes

### Phase 3 – Advanced Features & Observability

- Add session TTL, periodic GC, hard caps on concurrent sessions per principal/capsule
- Emit lightweight audit events: begin, chunk (rate-limited), finish/abort with sizes and sha prefix
- Add `uploads_stats()` (debug-only/feature-gated) for ops
- Consider resumable uploads and progress callbacks if needed

---

## Acceptance Criteria

- `lib.rs` regrouped: single "MEMORY MANAGEMENT" section with Core/Metadata/Upload subheaders
- Endpoints renamed: `uploads_*` for chunking, `memories_presence` for presence checks
- Documentation includes: three workflows, decision tree, and auth model mapping
- Unified creation path: shared core finalization for both `memories_create` and `uploads_finish`
- TypeScript upload client exists and is used by Next.js routes (no route behavioral regressions)
- Config endpoint `upload_config()` provides discoverable limits
- Idempotency keys required for all creation paths
- CI candid-diff passes and `.did` baseline updated when façade is added

---

## Risks and Mitigations

- Risk: Breaking current upload UX if route behavior changes
  - Mitigation: Keep routes as the primary interface; insert TS client under the routes only
- Risk: Confusion over which path to use
  - Mitigation: Decision tree + examples in docs; mark advanced endpoints as advanced
- Risk: Auth mismatches between layers
  - Mitigation: Document model boundaries; keep canister strict; perform session/DB auth in routes
- Risk: Session management complexity
  - Mitigation: TTL enforcement, GC, hard caps, proper error mapping

---

## Open Questions for Senior Review

1. Do you allow resumable finish after partial commit window? If yes, document "resumable" explicitly and retain chunk index map on session TTL reset.
2. Will `memories_update` ever replace the binary payload, or is payload immutable (new Memory on re-upload)? Decide and document.
3. Are the proposed chunk sizes (256-1024KB configurable) appropriate for IC message limits and overhead reduction?
4. Should we add feature-gating for advanced endpoints (`uploads_stats`, debug-only operations)?

---

## Immediate Action Items

- Regroup `lib.rs` memory endpoints under one section
- Rename endpoints: `memories_ping` → `memories_presence`, chunking endpoints → `uploads_*`
- Add `upload_config()` endpoint for discoverable limits
- Unify creation paths through shared core finalization
- Add the decision tree and auth model documentation to this doc
- Implement the small TS upload client inside Next.js routes with config discovery and idempotency
- Add session TTL, GC, and proper error normalization
