# ICP Memory Retrieval 500 Error Analysis

## Issue Summary

500 Internal Server Error occurring when attempting to retrieve individual ICP memory details via `/api/memories/[id]` endpoint.

## Observed Behavior

- **Error**: `GET http://localhost:3000/api/memories/[memory-id] 500 (Internal Server Error)`
- **Affected Memories**: ICP memories (e.g., `3331772a-2413-64ce-3331-0000000064ce`, `8215838d-c7ab-40d4-8215-0000000040d4`)
- **Context**: Error occurs in `use-memory-storage-status.ts` when `fetchStatus` function calls the API
- **Component**: `MemoryStorageBadge` component fails to load storage status

## Error Location

- **File**: `src/nextjs/src/hooks/use-memory-storage-status.ts`
- **Function**: `fetchStatus` (line 103)
- **API Endpoint**: `/api/memories/[id]`

## Potential Causes

1. **Backend API Issue**: The `/api/memories/[id]` endpoint may not be properly handling ICP memory IDs
2. **Data Transformation Error**: Issue in transforming ICP memory data to frontend format
3. **Missing Fields**: Required fields for ICP memories may be missing or malformed
4. **Type Mismatch**: Backend response format may not match expected frontend types

## Context

- This error occurs after successful ICP upload and memory creation
- The `memories_list_by_capsule` API call succeeds (returns 200)
- Only individual memory retrieval fails with 500 error
- Affects the storage status display in the UI

## Next Steps

1. **Investigate API Endpoint**: Check `/api/memories/[id]` implementation for ICP memory handling
2. **Check Backend Logs**: Look for specific error messages in backend logs
3. **Verify Data Format**: Ensure ICP memory data structure matches expected format
4. **Test with Different Memory Types**: Compare behavior between ICP and other memory types

## Related Files

- `src/nextjs/src/hooks/use-memory-storage-status.ts`
- `src/nextjs/src/components/common/memory-storage-badge.tsx`
- `src/nextjs/src/app/api/memories/[id]/route.ts` (if exists)
- Backend memory retrieval logic

## Root Cause Identified

The issue was in the `MemoryStorageBadge` component. Even when the `storageStatus` prop was provided (which contains the storage location information from the memory listing), the component was still calling the `useMemoryStorageStatus` hook.

The hook was detecting ICP memory IDs as UUID v7 format and trying to call `/api/memories/[id]` first, which caused the 500 error. This was unnecessary since the storage status information was already available from the memory listing.

## Fix Applied

Modified `MemoryStorageBadge` component to only call the `useMemoryStorageStatus` hook when `storageStatus` prop is not provided:

```typescript
// Only call the hook if storageStatus is not provided
const { status, data: presenceData } = useMemoryStorageStatus(
  storageStatus ? "" : memoryId,
  storageStatus ? "" : memoryType,
  dataSource
);
```

## Status

**Resolved** - Fixed by preventing unnecessary API calls when storage status is already available
