# BUG: Storage Edges Are Not Being Saved

## What's happening

When the app uploads a file to ICP, it tries to save info about where that file is stored (a "storage edge").
But nothing is actually being saved in the database.

That's why:

- Storage badges show wrong info
- The tracking system looks empty
- We lose all record of where files are stored

Basically: uploads "work" — but no data is stored.

## Why it happens

The API sends the wrong field name to the database.

```ts
backend: "icp-canister"; // ❌ does not exist in the DB table
```

The database table uses these names instead:

```ts
locationMetadata;
locationAsset;
```

Drizzle ORM ignores unknown fields instead of throwing an error.
So it silently skips `backend` and inserts nothing — no error, no warning.

## How to fix it (step by step)

### 1. Use the correct column names

Replace the wrong code:

```ts
backend: backend as "neon-db" | "vercel-blob" | "icp-canister",
```

with this:

```ts
// Choose the correct field based on artifact type
locationMetadata: artifact === "metadata"
  ? (backend === "icp-canister" ? "icp" : "neon")
  : undefined,

locationAsset: artifact === "asset"
  ? (backend === "icp-canister" ? "icp" : backend)
  : undefined,
```

### 2. Use the generated type from Drizzle

This makes TypeScript check for wrong field names automatically.

```ts
import { storageEdges, type NewDBStorageEdge } from "@/db/schema";

const edgeData: NewDBStorageEdge = {
  memoryId,
  memoryType,
  artifact,
  locationMetadata,
  locationAsset,
  present,
  locationUrl: location,
  updatedAt: new Date(),
};

await db.insert(storageEdges).values(edgeData);
```

Now if you add a field like `backend`, TypeScript will show an error.

### 3. Add a simple check after inserting

So we don't silently fail again:

```ts
const result = await db.insert(storageEdges).values(edgeData).returning();

if (!result.length) {
  throw new Error("Failed to create storage edge");
}
```

### 4. Optional: Add runtime validation (Zod)

To make sure the API input is valid:

```ts
import { z } from "zod";

const StorageEdgeInput = z.object({
  memoryId: z.string().uuid(),
  memoryType: z.enum(["image", "video", "note", "document", "audio"]),
  artifact: z.enum(["metadata", "asset"]),
  backend: z.enum(["neon-db", "vercel-blob", "icp-canister"]),
  location: z.string().url(),
});
```

Use it at the start of the API route:

```ts
const data = StorageEdgeInput.parse(await req.json());
```

### 5. Add a quick test

Create one small test that uploads and checks that a row appears in `storage_edges`.
If it doesn't — fail the test. This prevents the same bug from coming back.

## Summary (TL;DR)

| Problem                        | Fix                                      |
| ------------------------------ | ---------------------------------------- |
| Wrong field name (`backend`)   | Use `locationMetadata` / `locationAsset` |
| Drizzle ignores unknown fields | Use `NewDBStorageEdge` type for inserts  |
| Silent failure                 | Check `.returning()` result              |
| No input validation            | Add Zod schema                           |
| No test                        | Add a simple integration test            |

**In short:**
The bug happened because we didn't use Drizzle's type safety.
Fix the field names, type the insert properly, and TypeScript will protect you next time.

## Status

**CRITICAL** - Immediate fix required, affects all ICP uploads

## Priority

**P0** - This is a data integrity issue that breaks core functionality

---

## New Architecture Consideration

We're currently moving database operations from API endpoints to service layers (following the pattern used in `src/nextjs/src/services/memory/`).

### Current Pattern (Memories):

- **API Layer**: `src/nextjs/src/app/api/memories/` - handles HTTP requests/responses
- **Service Layer**: `src/nextjs/src/services/memory/` - handles database operations
- **Clean Separation**: API routes import and call service functions

### Should We Apply This to Storage Edges?

**Current**: Storage edges API does database operations directly in the route handler
**Proposed**: Create `src/nextjs/src/services/storage-edges/` with functions like:

- `createStorageEdge()`
- `getStorageEdges()`
- `updateStorageEdge()`

**Benefits**:

- Consistent architecture across the codebase
- Easier testing (service functions are pure)
- Better separation of concerns
- Reusable functions across different API endpoints

**Example Service Structure**:

```ts
// src/nextjs/src/services/storage-edges/storage-edge-operations.ts
export const createStorageEdge = async (params: CreateStorageEdgeParams): Promise<StorageEdgeOperationResult> => {
  // Database logic here with proper typing
};

// src/nextjs/src/app/api/storage/edges/route.ts
export async function PUT(request: NextRequest) {
  const data = await request.json();
  const result = await createStorageEdge(data);
  return NextResponse.json(result);
}
```

## Tech Lead Review & Answers

### 1. Should we refactor storage edges to use the service layer pattern before fixing the schema bug?

**Answer:** No — fix the bug **first**, then refactor.
The schema mismatch is a **P0 data integrity issue**, so we must restore correct writes immediately.
After we confirm the inserts work correctly and data is consistent again, we can safely refactor to the service layer.

### 2. How can we prevent this type of schema mismatch in the future?

**Answer:** Always use **Drizzle's generated types** (`NewDBStorageEdge`, `StorageEdge`) for inserts and updates.
This ensures compile-time checks catch missing or extra fields.
Also, enable these strict TS options in `tsconfig.json`:

```json
{
  "strict": true,
  "noImplicitAny": true,
  "noUncheckedIndexedAccess": true,
  "exactOptionalPropertyTypes": true
}
```

Optionally, add a **CI check** that regenerates Drizzle types and fails if type mismatches appear.

### 3. Should we add runtime schema validation to all database operations?

**Answer:** Yes — at least for **public API inputs**.
Use **Zod** at the route level to validate request payloads before they touch the DB.
Inside service layers, rely on **Drizzle types** for compile-time validation (no need for runtime overhead there).

So:

- **API boundary → Zod (runtime validation)**
- **Service/DB boundary → Drizzle types (compile-time validation)**

### 4. How can we improve type safety to catch these issues at compile time?

**Answer:**

- Always type insert/update objects explicitly (`const edge: NewDBStorageEdge = { ... }`)
- Never use untyped objects or `as any`
- Disallow unsafe casts in ESLint/TS rules (`no-explicit-any`, `no-unsafe-assignment`)
- Use discriminated unions for fields like `artifact` to ensure correct column is set (`locationAsset` vs `locationMetadata`)

This guarantees that if someone adds the wrong field (`backend`), TypeScript will block the build.

### 5. What's the best way to add integration tests for database operations?

**Answer:** Use **Vitest** or **Jest** with a temporary Neon branch (or SQLite memory DB for local).
Each test should:

1. Insert mock data using the service function (`createStorageEdge`)
2. Query back the inserted data
3. Assert that all required fields (`locationAsset` / `locationMetadata`) exist and are correct

Example pattern:

```ts
const edge = await createStorageEdge(mockInput);
const result = await db.query.storageEdges.findFirst({ where: eq(storageEdges.id, edge.id) });
expect(result).not.toBeNull();
```

We'll later add these tests to CI to prevent silent schema drift.

### 6. Should we add database migration validation to catch schema mismatches?

**Answer:** Yes — absolutely.
Add a **CI job** that runs:

```bash
drizzle-kit generate
drizzle-kit push
pnpm typecheck
```

This ensures migrations, schema, and generated types stay in sync.
If someone changes the table but forgets to regenerate types, the pipeline will fail.

## Final Recommendation Summary

| Goal                      | Action                                                                  |
| ------------------------- | ----------------------------------------------------------------------- |
| Fix bug fast              | Patch API now, test, deploy                                             |
| Prevent schema mismatches | Use Drizzle-generated types                                             |
| Validate inputs           | Use Zod on API layer                                                    |
| Enforce type safety       | Tighten TS & ESLint rules                                               |
| Strengthen tests          | Add integration test for storage edges                                  |
| Ensure long-term safety   | Add CI migration/type sync step                                         |
| Refactor later            | Move logic to `services/storage-edges` after data integrity is restored |

**Verdict:** Fix-first, refactor-second.
Proceed with the implementation as described, then create a follow-up issue for the service layer refactor once the hotfix is deployed and verified.

---

## Action Plan

### Phase 1: Immediate Fix (P0 - Deploy Today)

1. **Fix the API endpoint** (`src/nextjs/src/app/api/storage/edges/route.ts`)

   - Replace `backend` field with `locationMetadata`/`locationAsset`
   - Add proper type annotations using `NewDBStorageEdge`
   - Add error handling for failed inserts

2. **Test the fix**

   - Upload a file to ICP
   - Verify storage edge records are created in database
   - Check that storage badges show correct status

3. **Deploy and verify**
   - Deploy to staging first
   - Test upload flow end-to-end
   - Deploy to production
   - Monitor for 24 hours

### Phase 2: Prevention (This Week)

4. **Add TypeScript strict mode**

   - Update `tsconfig.json` with strict settings
   - Fix any new type errors that appear

5. **Add Zod validation**

   - Create input validation schema for storage edges API
   - Validate all request payloads

6. **Add integration test**
   - Create test that verifies storage edge creation
   - Add to CI pipeline

### Phase 3: Long-term (Next Sprint)

7. **Create service layer**

   - Move database logic to `src/nextjs/src/services/storage-edges/`
   - Update API routes to use service functions
   - Add comprehensive tests

8. **Add CI checks**
   - Add migration/type sync validation to CI
   - Ensure schema changes trigger type regeneration

### Success Criteria

- [ ] Storage edges are being created in database
- [ ] Storage badges show correct status
- [ ] No more 500 errors on memory retrieval
- [ ] TypeScript catches schema mismatches at compile time
- [ ] Integration tests prevent regression
- [ ] Service layer architecture is consistent

### Rollback Plan

If the fix causes issues:

1. Revert the API changes
2. Investigate database state
3. Fix any data inconsistencies
4. Re-deploy with corrected fix
