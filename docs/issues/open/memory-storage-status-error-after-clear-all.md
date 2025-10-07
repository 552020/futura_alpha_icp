# Memory Storage Status Error After Clear All

## Issue Summary

When users clear all memories and switch database preferences, the app attempts to fetch storage status for non-existent memories, causing console errors.

## Error Details

- **Error Type**: Console Error
- **Error Message**: `Error fetching memory storage status:`
- **Location**: `src/hooks/use-memory-storage-status.ts:59:16`
- **Frequency**: Multiple errors (8 reported)

## Reproduction Steps

1. Set database preferences to include both ICP and Neon
2. Have memories visible in dashboard
3. Click "Clear All" to delete all memories
4. Try to upload an image (gets "please connect to ICP" error)
5. Switch back to Web2-only preferences in profile settings
6. **Result**: Console errors appear for non-existent memory storage status

## Root Cause Analysis

The issue occurs because:

1. **Stale Memory References**: After clearing all memories, the frontend may still have references to memory IDs in React state or cached data
2. **Storage Status Hook**: The `useMemoryStorageStatus` hook continues to attempt fetching storage status for these non-existent memories
3. **No Error Handling**: The hook doesn't properly handle 404 responses when memories don't exist
4. **State Cleanup**: The app doesn't properly clean up memory references when memories are deleted

## Technical Details

### Error Location

```typescript
// src/hooks/use-memory-storage-status.ts:59
const fetchStatus = async () => {
  try {
    // ... fetch logic
  } catch (error) {
    logger.error("Error fetching memory storage status:", error); // <- Error occurs here
  }
};
```

### Related Files

- `src/hooks/use-memory-storage-status.ts` - Main error location
- `src/app/api/memories/[id]/route.ts` - API endpoint that returns 404
- Dashboard components that use the storage status hook

## Impact

- **User Experience**: Console errors create noise and confusion
- **Performance**: Unnecessary API calls to non-existent resources
- **Debugging**: Makes it harder to identify real issues

## Proposed Solutions

### Phase 1: Immediate Fix (Error Handling)

1. **Add 404 Handling**: Update `useMemoryStorageStatus` to handle 404 responses gracefully
2. **Silent Failures**: Don't log errors for expected 404s when memories don't exist
3. **State Cleanup**: Clear storage status state when memories are deleted

### Phase 2: Prevention (State Management)

1. **Memory Reference Cleanup**: Ensure all memory references are removed from state when memories are deleted
2. **Cache Invalidation**: Properly invalidate React Query cache for deleted memories
3. **Component Unmounting**: Clean up storage status hooks when memory components unmount

### Phase 3: Robustness (Defensive Programming)

1. **Memory Existence Check**: Verify memory exists before fetching storage status
2. **Debounced Requests**: Prevent rapid-fire requests for the same memory
3. **Error Boundaries**: Add error boundaries around storage status components

## Implementation Plan

### Step 1: Fix Error Handling

```typescript
// src/hooks/use-memory-storage-status.ts
const fetchStatus = async () => {
  try {
    const response = await fetch(`/api/memories/${memoryId}`);

    if (response.status === 404) {
      // Memory doesn't exist, clear state silently
      setStatus("error");
      return;
    }

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }

    // ... rest of logic
  } catch (error) {
    // Only log unexpected errors, not 404s
    if (error instanceof Error && !error.message.includes("404")) {
      logger.error("Error fetching memory storage status:", error);
    }
    setStatus("error");
  }
};
```

### Step 2: Add Memory Existence Check

```typescript
// Before fetching storage status, verify memory exists
const checkMemoryExists = async (memoryId: string): Promise<boolean> => {
  try {
    const response = await fetch(`/api/memories/${memoryId}`);
    return response.ok;
  } catch {
    return false;
  }
};
```

### Step 3: Update Dashboard State Management

- Clear memory references from React state when memories are deleted
- Invalidate React Query cache for deleted memories
- Ensure storage status hooks are properly cleaned up

## Testing Strategy

1. **Reproduce Issue**: Follow exact reproduction steps
2. **Verify Fix**: Confirm no console errors after clearing memories
3. **Edge Cases**: Test with various database preference combinations
4. **Performance**: Ensure no unnecessary API calls

## Priority

**High** - This affects user experience and creates console noise that makes debugging difficult.

## Related Issues

- Database switching functionality
- Memory deletion cleanup
- Storage status display

## Notes

- The error occurs specifically when switching from dual storage (ICP + Neon) back to Web2-only
- Multiple errors suggest the hook is being called for multiple non-existent memories
- This is likely a state management issue where memory references persist after deletion
