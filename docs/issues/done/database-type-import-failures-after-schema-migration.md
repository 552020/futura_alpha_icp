# Database Type Import Failures After Schema Migration

## Problem Description

The build is failing due to database type import errors after a successful merge that brought in a new modular database schema structure. Files are trying to import types from paths that no longer exist.

## Root Cause Analysis

### What Happened During the Merge

**Before Merge (c08da71~1):**

- **Database Structure**: Single monolithic `schema.ts` file
- **Type Exports**: All types exported from `@/db/schema`
- **Available Types**:
  ```typescript
  // From old schema.ts
  export type DBMemory = typeof memories.$inferSelect;
  export type NewDBMemory = typeof memories.$inferInsert;
  export type NewDBMemoryAsset = typeof memoryAssets.$inferInsert;
  export type DBStorageEdge = typeof storageEdges.$inferSelect;
  export type NewDBStorageEdge = typeof storageEdges.$inferInsert;
  ```

**After Merge (c08da71):**

- **Database Structure**: Modular approach with separate files:
  - `tables.ts` - Drizzle table definitions
  - `types.ts` - Drizzle-inferred types
  - `enums.ts` - Enum definitions and constants
  - `relations.ts` - Table relations
  - `index.ts` - Main export file
- **Old File**: `schema.ts` → `schema_old.ts.md` (renamed)

### The Import Path Problem

**❌ Broken Imports (causing build failures):**

```typescript
// These paths no longer exist:
import { NewDBMemory, NewDBMemoryAsset, DBMemory } from "@/db/schema";
import { users, allUsers } from "@/db/types"; // Tables not in types.ts
```

**✅ Correct Imports (working):**

```typescript
// Tables from tables.ts:
import { users, allUsers, storageEdges, memories } from "@/db/tables";

// Types from types.ts:
import { type DBUser, type NewDBUser, type DBMemory, type NewDBMemory } from "@/db/types";

// Enums from enums.ts:
import { type MemoryType, type AssetType, type ProcessingStatus } from "@/db/enums";
```

## Type Availability Analysis

### Types That Exist in New Structure

**In `@/db/types`:**

- `DBMemory` ✅ (line 53)
- `NewDBMemory` ✅ (line 54)
- `DBMemoryAsset` ✅ (line 56)
- `NewDBMemoryAsset` ✅ (line 57)
- `DBStorageEdge` ✅ (line 143)
- `NewDBStorageEdge` ✅ (line 144)
- `DBUser`, `NewDBUser` ✅ (lines 36-37)
- `DBAllUser`, `NewDBAllUser` ✅ (lines 39-40)

**In `@/db/enums`:**

- `MemoryType` ✅ (line 154)
- `AssetType` ✅ (line 162)
- `ProcessingStatus` ✅ (line 163)

**In `@/db/tables`:**

- `users` ✅ (table definition)
- `allUsers` ✅ (table definition)
- `storageEdges` ✅ (table definition)
- `memories` ✅ (table definition)

### Types That Are Missing

**❌ Not Found:**

- Some files expect tables to be exported from `@/db/types` but they're in `@/db/tables`
- Some files expect types to be exported from `@/db/tables` but they're in `@/db/types`

## Current Build Failures

### Files with Import Errors

1. **`src/services/user/user-operations.ts`**

   - **Error**: `'users' is not exported from '@/db/types'`
   - **Fix**: Import tables from `@/db/tables`, types from `@/db/types`

2. **`src/services/user/types.ts`**

   - **Error**: `Cannot find module '@/db/schema'`
   - **Fix**: Update import path to `@/db/types`

3. **`src/utils/memory-type.ts`**
   - **Likely Error**: Old import paths
   - **Fix**: Update to new modular imports

### Build Status

- **Current**: Build fails with TypeScript import errors
- **Target**: Build passes with correct modular imports
- **Progress**: ~70% of files fixed

## The Solution Pattern

### For Files That Need Both Tables and Types

```typescript
// Import tables from tables.ts
import { users, allUsers, storageEdges, memories } from "@/db/tables";

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

## Impact Assessment

### Why This Happened

1. **Main branch**: Had the new modular database structure
2. **This branch**: Had files using old import paths (`@/db/schema`)
3. **Merge**: Successfully brought in new structure but didn't update import paths
4. **Result**: Build fails because `@/db/schema` no longer exists

### Current State

- **Database structure**: ✅ Correctly migrated to modular approach
- **Type definitions**: ✅ All types exist in new locations
- **Import paths**: ❌ Many files still use old paths
- **Build status**: ❌ Failing due to import errors

## Next Steps

1. **Systematically fix all import paths** from old `@/db/schema` to new modular imports
2. **Verify all types are available** in their new locations
3. **Test build passes** after all imports are fixed
4. **Document the new import patterns** for future development

## Related Files

- `src/db/tables.ts` - Table definitions
- `src/db/types.ts` - Drizzle-inferred types
- `src/db/enums.ts` - Enum definitions
- `src/db/index.ts` - Main export file
- `src/db/schema_old.ts.md` - Old schema (renamed)

## Related

- Database schema migration from monolithic to modular structure
- Drizzle ORM type inference system
- TypeScript module resolution
- Git merge conflict resolution

