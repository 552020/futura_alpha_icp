# Fix Database Type Imports After Schema Migration

## Problem Description

The build is failing because multiple files are importing database types from incorrect paths after a schema migration from `schema.ts` to a modular structure (`tables.ts`, `types.ts`, `enums.ts`).

## Root Cause

The project underwent a database schema migration:

- **Before**: Single `schema.ts` file with all types and tables
- **After**: Modular structure:
  - `tables.ts` - Drizzle table definitions
  - `types.ts` - Drizzle-inferred types (`DBMemory`, `NewDBMemory`, etc.)
  - `enums.ts` - Enum definitions and constants

However, many files still have old import paths that no longer exist.

## Current State

### ❌ Broken Imports

```typescript
// These paths don't exist anymore:
import { NewDBMemory, NewDBMemoryAsset, DBMemory } from "@/db/schema";
import { users, allUsers } from "@/db/types"; // Tables not in types.ts
```

### ✅ Correct Imports

```typescript
// Tables from tables.ts:
import { users, allUsers, storageEdges } from "@/db/tables";

// Types from types.ts:
import { type DBUser, type NewDBUser, type DBMemory, type NewDBMemory } from "@/db/types";

// Enums from enums.ts:
import { type MemoryType, type AssetType, type ProcessingStatus } from "@/db/enums";
```

## Files That Need Fixing

### ✅ Fixed

- `src/services/memory/types.ts` - Now imports from `@/db/enums`
- `src/app/api/memories/utils/memory-database.ts` - Now imports from `@/db/types`
- `src/services/storage-edges/storage-edge-operations.ts` - Split imports correctly
- `src/lib/usecases/memory/build-new-memory-and-asset.ts` - Fixed import path
- `src/lib/usecases/memory/create-multiple-memories.ts` - Fixed import path
- `src/lib/usecases/memory/create-memory-with-asset.ts` - Fixed import path

### ❌ Still Broken

- `src/services/user/user-operations.ts` - Needs to import tables from `@/db/tables`
- `src/services/user/types.ts` - Still imports from `@/db/schema`
- `src/utils/memory-type.ts` - Likely has old imports

## The Fix Pattern

### For Files That Need Both Tables and Types

```typescript
// Import tables from tables.ts
import { users, allUsers, storageEdges } from "@/db/tables";

// Import types from types.ts
import { type DBUser, type NewDBUser, type DBStorageEdge, type NewDBStorageEdge } from "@/db/types";
```

### For Files That Only Need Types

```typescript
// Import only types from types.ts
import { type DBMemory, type NewDBMemory, type NewDBMemoryAsset } from "@/db/types";
```

### For Files That Only Need Enums

```typescript
// Import enums from enums.ts
import { type MemoryType, type AssetType, type ProcessingStatus } from "@/db/enums";
```

## Build Status

- **Current**: Build fails with import errors
- **Target**: Build passes with correct imports
- **Progress**: ~70% of files fixed

## Next Steps

1. Fix remaining import errors in user-operations.ts
2. Fix user/types.ts import path
3. Check and fix any other files with old import paths
4. Verify build passes completely

## Impact

- **Build fails** until all imports are fixed
- **Type safety** compromised with incorrect imports
- **Developer experience** degraded with broken imports

## Related

- Database schema migration from monolithic to modular structure
- Drizzle ORM type inference system
- TypeScript module resolution

