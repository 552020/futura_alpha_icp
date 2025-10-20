### Refactor `api/memories/[id]` to use services instead of direct DB access

We currently perform database operations directly in the route `src/nextjs/src/app/api/memories/[id]/route.ts`, which violates our policy that services should encapsulate DB access. This issue tracks extracting those concerns into service/usecase layers and updating the route to consume them.

#### Scope

- Affected file: `src/nextjs/src/app/api/memories/[id]/route.ts`
- New/updated service locations (suggested): `@/services/user`, `@/services/memory`, `@/services/folder`, `@/services/storage` (or corresponding usecase directories under `@/lib/usecases/...`)
- Preserve existing logging via `fatLogger` and existing API response shapes.

---

### Sub-issues

1. Create user service for `allUsers` lookup

- Description: Add `getAllUserByAuthUserId(userId)` in `@/services/user`.
- Replace direct `db.query.allUsers.findFirst` calls in GET/PUT/DELETE handlers.
- Acceptance criteria:
  - Route no longer imports `allUsers`/`db` for user lookup.
  - Errors and not-found cases are preserved and logged consistently.

Status: Completed

- Implemented `getAllUserRecord(session.user.id)` usage in GET/PUT/DELETE of `route.ts`.
- Removed direct `db.query.allUsers` in the route; kept logging/behavior intact.

2. Create memory read services

- Description: Add memory read APIs in `@/services/memory`, including:
  - `getMemoryWithAssetsByOwner(memoryId, ownerAllUserId)`
  - A generic `getMemoryWithRelations(memoryId, ownerAllUserId, relations)` that supports `{ assets: true }` and `{ folder: true, assets: true }`.
- Replace direct `db.query.memories.findFirst/findMany` calls in GET/PUT/DELETE.
- Acceptance criteria:
  - Route no longer calls `db.query.memories` directly.
  - Relation fetching parity is maintained.

Status: Completed

- Added `getMemoryWithRelations` and `getMemoryWithAssetsByOwner` in `@/services/memory` and exported via `index.ts`.
- Updated `route.ts` GET/PUT/DELETE to use service; parity for `{ assets }` and `{ folder, assets }` maintained; no direct `db.query.memories` calls remain in the route.

3. Move storage status computation to a service

- Description: Add `getStorageStatusForMemory(memoryId)` in `@/services/storage` and a helper `attachStorageStatus(memory)` in the memory service.
- Replace the inline `addStorageStatusToMemory` and direct `storageEdges` queries.
- Acceptance criteria:
  - Route does not import `storageEdges` or touch `db` for storage lookups.
  - Returned shape is unchanged: `{ storageStatus: { storageLocations: string[] } }`.

Status: Completed

- Added `getStorageStatusForMemory(memoryId)` in `@/services/storage-edges/storage-status.ts` and re-exported via `@/services/storage-edges/index.ts`.
- Added `attachStorageStatus(memory)` in `@/services/memory/memory-operations.ts` and exported via `@/services/memory/index.ts`.
- Updated `route.ts` GET to use `attachStorageStatus(memory)` and removed inline `addStorageStatusToMemory`, removed direct `storageEdges`/`db` usage.

4. Extract folder deletion orchestration to service/usecase

- Description: Add `deleteFolderAndContents(folderId, ownerAllUserId)` in `@/services/folder` (or `@/lib/usecases/folder`). Should:
  - Validate ownership.
  - List child memories.
  - Delete memories via memory service.
  - Run storage edge cleanup via existing cleanup utility.
  - Delete the folder.
- Replace inline `handleFolderDeletion` logic in the route.
- Acceptance criteria:
  - Route calls a single service function for folder deletion.
  - Response shape and logging behavior preserved.

Status: Completed

- Implemented `@/lib/usecases/folder/delete-folder-and-contents.ts` orchestrating:
  - Ownership validation, child memory listing, DB deletion, and `cleanupMemoryAndStorage` per memory, then folder deletion.
- Updated `route.ts` DELETE to call `deleteFolderAndContents` and removed inline `handleFolderDeletion` logic.
- Preserved response shape and logging.

5. Extract memory deletion orchestration to service/usecase

- Description: Add `deleteMemoryWithCleanup(memoryId, ownerAllUserId)` in `@/services/memory` (or `@/lib/usecases/memory`). Should:
  - Pre-read memory with needed relations.
  - Perform DB deletion (respecting cascade behavior).
  - Invoke `cleanupMemoryAndStorage`.
- Replace direct `db.delete(memories)` and pre-read in the route.
- Acceptance criteria:
  - Route calls a single service function for memory deletion.
  - Response shape and logging behavior preserved.

Status: Completed

- Implemented `@/lib/usecases/memory/delete-memory-with-cleanup.ts` to pre-read relations, delete the memory, and invoke `cleanupMemoryAndStorage`.
- Updated `route.ts` DELETE to call the usecase and removed direct DB delete/inline cleanup; response shape and logging preserved.

6. Update route to depend solely on services and add tests

- Description:
  - Replace remaining direct `db` imports/queries in `route.ts` with calls from the services defined above.
  - Add unit tests for service modules; add integration tests for GET/PUT/DELETE to ensure behavior parity.
- Acceptance criteria:
  - `route.ts` contains only: auth, input parsing/validation, service calls, and HTTP response mapping.
  - Tests cover success and error paths with no regressions in API behavior.

---

### Current direct DB usages to replace (for reference)

- User lookup: `db.query.allUsers.findFirst(...)`
- Memory reads: `db.query.memories.findFirst(...)`, `db.query.memories.findMany(...)`
- Storage status: `db.query.storageEdges.findMany(...)`
- Folder deletion: `db.delete(folders).where(...)` and related inline memory deletions
- Memory deletion: `db.delete(memories).where(...).returning()`

### Desired route structure

- Authenticate user.
- Parse and validate inputs.
- Call service functions only.
- Map service results to HTTP responses.

### Notes

- Keep `fatLogger` logging in place through service layers to preserve observability.
- Maintain response shapes to avoid frontend regressions.
