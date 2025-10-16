# Missing Database Types Import Issue

## Problem Description

The build is failing because several files are importing database types from the wrong location. The types `DBMemory`, `NewDBMemory`, and `NewDBMemoryAsset` exist but are not being imported correctly.

## Root Cause Analysis

### What's Happening

1. **Types exist**: The types are properly defined in `src/db/types.ts`:

   - `DBMemory` (line 53)
   - `NewDBMemory` (line 54)
   - `NewDBMemoryAsset` (line 57)

2. **Wrong import paths**: Files are trying to import from `@/db/schema` or `@/db/tables` instead of `@/db/types`

3. **Schema migration**: The project appears to have migrated from `schema.ts` to `tables.ts` + `types.ts` + `enums.ts`, but imports weren't updated

### Files Affected

- `src/services/storage-edges/storage-edge-operations.ts` ✅ Fixed
- `src/services/user/user-operations.ts` ✅ Fixed
- `src/app/api/memories/utils/memory-database.ts` ❌ Still broken
- `src/services/memory/types.ts` ✅ Fixed (now imports from `@/db/enums`)

## Current State

### ✅ Working Imports

```typescript
// src/services/memory/types.ts
export type { MemoryType, AssetType, ProcessingStatus } from "@/db/enums";
```

### ❌ Broken Imports

```typescript
// src/app/api/memories/utils/memory-database.ts
// import { NewDBMemory, NewDBMemoryAsset, DBMemory } from '@/db/tables';
```

## Solution

### Fix the Import Path

Change the import in `memory-database.ts` from:

```typescript
// import { NewDBMemory, NewDBMemoryAsset, DBMemory } from '@/db/tables';
```

To:

```typescript
import { NewDBMemory, NewDBMemoryAsset, DBMemory } from "@/db/types";
```

### Remove Temporary Type Definitions

The file currently has temporary type definitions that should be removed:

```typescript
// Remove these temporary definitions:
type DBMemory = { ... };
type NewDBMemory = { ... };
type NewDBMemoryAsset = { ... };
```

## Impact

- **Build fails** with TypeScript errors
- **Type safety compromised** with temporary type definitions
- **Code duplication** with manual type definitions instead of using proper Drizzle-inferred types

## Files to Update

1. `src/app/api/memories/utils/memory-database.ts` - Fix import path
2. Check for any other files importing from `@/db/schema` or `@/db/tables` that should import from `@/db/types`

## Verification

After fixing, the build should pass:

```bash
npm run build
```

## Related

- Database schema migration from `schema.ts` to modular structure (`tables.ts`, `types.ts`, `enums.ts`)
- Drizzle ORM type inference system
