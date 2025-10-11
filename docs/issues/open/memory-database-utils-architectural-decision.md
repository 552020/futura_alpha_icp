# Memory Database Utils - Architectural Decision Required

## Issue Summary

**Status**: ✅ COMPLETED  
**Priority**: Medium  
**Type**: Architectural Decision  
**Assignee**: Tech Lead  
**Implementation Date**: 2024-01-XX

## Background

During the storage edges API schema mismatch fix, we discovered that `src/nextjs/src/app/api/memories/utils/memory-database.ts` was violating our service layer architecture pattern by performing direct database operations instead of using centralized service functions.

## What We Fixed

### 1. Eliminated All Direct Database Operations

**Before**: The file contained multiple direct database operations:

```typescript
// Direct database operations (REMOVED)
const insertedMemories = await db.insert(memories).values(memoryRows).returning();
const insertedAssets = await db.insert(memoryAssets).values(assetsWithMemoryIds).returning();
const memory = await db.query.memories.findFirst({ where: eq(memories.id, memoryId) });
const deletedEdges = await db.delete(storageEdges).where(conditions).returning();
```

**After**: All operations now use service layer functions:

```typescript
// Service layer operations (NEW)
const memoryResult = await createMemoryRecord(memoryParams);
const assetResult = await createAssetRecords(assetParams);
const memoryResult = await getMemoryRecord(memoryId, true);
const deletedEdgesResult = await deleteStorageEdges({ memoryId, memoryType });
```

### 2. Refactored Functions to Use Service Layer

- **`processMultipleFilesBatch`**: Now uses `createMemoryRecord` and `createAssetRecords`
- **`storeInNewDatabase`**: Now uses `createMemoryRecord` and `createAssetRecords`
- **`createStorageEdgesForMemory`**: Now uses `createStorageEdge` service
- **`cleanupStorageEdgesForMemory`**: Now uses `getAssetRecordsByMemory`, `getStorageEdges`, `deleteStorageEdges`, and `hardDeleteAssetRecord`
- **`getMemoryDataForCleanup`**: Now uses `getMemoryRecord`

### 3. Added Missing Service Function

Created `deleteStorageEdges` function in the storage edges service layer to complete the CRUD operations.

## Current File Structure

```
src/nextjs/src/
├── app/api/memories/utils/
│   └── memory-database.ts          # ⚠️  LOCATION IN QUESTION
├── services/
│   ├── memory/                     # ✅ Memory service layer
│   │   ├── index.ts
│   │   ├── memory-operations.ts    # createMemoryRecord, getMemoryRecord, etc.
│   │   └── asset-operations.ts     # createAssetRecords, getAssetRecordsByMemory, etc.
│   └── storage-edges/              # ✅ Storage edges service layer
│       ├── index.ts
│       └── storage-edge-operations.ts  # createStorageEdge, getStorageEdges, deleteStorageEdges
└── lib/                            # ✅ Utility functions
    └── ...
```

## Functions in `memory-database.ts` and Their Purpose

| Function                       | Purpose                                             | Current Usage               |
| ------------------------------ | --------------------------------------------------- | --------------------------- |
| `buildNewMemoryAndAsset`       | Helper to build memory and asset data structures    | Used by batch processing    |
| `processMultipleFilesBatch`    | Batch create memories and assets for multiple files | Used by folder uploads      |
| `storeInNewDatabase`           | Create single memory with asset and storage edges   | Used by single file uploads |
| `createStorageEdgesForMemory`  | Create storage edge records for a memory            | Used by memory creation     |
| `cleanupStorageEdgesForMemory` | Delete all storage edges and assets for a memory    | Used by memory deletion     |
| `getMemoryDataForCleanup`      | Get memory data before deletion for cleanup         | Used by memory deletion     |
| `extractS3KeyFromUrl`          | Helper to extract S3 key from URL                   | Used by cleanup functions   |

## Architectural Decision Required

### Option 1: Move to `src/nextjs/src/lib/` (Recommended)

**Pros:**

- ✅ Follows standard Next.js conventions (lib = utility functions)
- ✅ Clear separation from API routes
- ✅ Functions are pure utilities, not API-specific
- ✅ Can be imported by both API routes and other services

**Cons:**

- ❌ Requires updating import paths in API routes

### Option 2: Move to `src/nextjs/src/services/`

**Pros:**

- ✅ Groups with other service layer functions
- ✅ Clear service layer organization

**Cons:**

- ❌ These are orchestration functions, not pure service functions
- ❌ They combine multiple service calls (memory + storage edges)

### Option 3: Keep in `src/nextjs/src/app/api/memories/utils/`

**Pros:**

- ✅ No import path changes needed
- ✅ Close to where it's used

**Cons:**

- ❌ Violates separation of concerns (API routes should be thin)
- ❌ Utils in API folder suggests API-specific logic
- ❌ Harder to reuse across different API routes

## Recommendation

**Move to `src/nextjs/src/lib/memory-database.ts`**

**Reasoning:**

1. These functions are **orchestration utilities** that combine multiple service calls
2. They're **not API-specific** - could be used by other parts of the application
3. They follow the **lib pattern** of providing utility functions
4. Clear separation from API route handlers

## Implementation Plan

If approved, the move would involve:

1. **Move file**: `app/api/memories/utils/memory-database.ts` → `lib/memory-database.ts`
2. **Update imports** in API routes that use these functions
3. **Update documentation** to reflect new location
4. **No functional changes** - all service layer calls remain the same

## Files That Import These Functions

The following files would need import path updates:

- `src/nextjs/src/app/api/memories/route.ts`
- `src/nextjs/src/app/api/memories/[id]/route.ts`
- Any other API routes that use these utilities

## Success Criteria

- ✅ All direct database operations eliminated
- ✅ Service layer architecture fully implemented
- ✅ Functions moved to appropriate location
- ✅ Import paths updated
- ✅ No functional regressions

## Questions for Tech Lead

1. **Location**: Should we move `memory-database.ts` to `lib/` or keep it in `app/api/memories/utils/`?
2. **Naming**: Is `memory-database.ts` a good name, or should it be `memory-orchestration.ts` or similar?
3. **Organization**: Should we split these functions into multiple files (e.g., `memory-creation.ts`, `memory-cleanup.ts`)?
4. **Future**: Are there other similar utility files that should be moved to `lib/`?

---

**Created**: 2024-01-XX  
**Last Updated**: 2024-01-XX  
**Related Issues**: Storage Edges API Schema Mismatch Critical Bug
