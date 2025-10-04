# Clear All Button ICP Integration Analysis

## Current Implementation Analysis

### What "Clear All" Currently Does

**Location**: `src/nextjs/src/app/api/memories/delete.ts` (line 174-210)

**Current Behavior**:

1. **Neon Database Cleanup**:

   - Deletes all memories from `memories` table
   - Deletes all folders from `folders` table
   - Deletes all galleries from `galleries` table
   - Deletes all gallery items from `galleryItems` table
   - Calls `cleanupStorageEdgesForMemories()` for each memory

2. **Storage Edge Cleanup** (`cleanupStorageEdgesForMemory`):
   - Deletes S3 objects from AWS S3
   - Deletes storage edge records from `storageEdges` table
   - Deletes memory assets from `memoryAssets` table
   - **Does NOT delete ICP assets**

### What's Missing for ICP Integration

**Current Gap**: The clear all function only cleans up:

- ✅ Neon database records
- ✅ S3 storage (if used)
- ❌ **ICP memories and assets** (not implemented)

## Required ICP Integration

### 1. ICP Memory Deletion

**Backend Architecture Analysis**:

- **Memory Storage**: Memories are stored **per capsule**, not centralized (`Capsule.memories: HashMap<String, Memory>`)
- **Current API**: `memories_delete(memory_id: String)` - searches across ALL accessible capsules
- **Capsule-Scoped Operations**: Most operations require `capsule_id` parameter (`memories_list`, `memories_create`)

**Backend API Available**: `memories_delete(memory_id: String)` in `src/backend/src/lib.rs:404`

**Current Status**:

- ✅ Backend function exists (single memory deletion)
- ❌ **No bulk deletion API** - needs to be implemented
- ❌ Frontend service not implemented (see `src/nextjs/src/services/icp-gallery.ts:438` - placeholder)

**Backend Enhancement Needed**:

```rust
// Proposed new API for bulk deletion - following Result<T, Error> standard
memories_delete_bulk(capsule_id: String, memory_ids: Vec<String>) -> Result<BulkDeleteResult, Error>

// Result type following our standards
pub struct BulkDeleteResult {
    pub deleted_count: u32,
    pub failed_count: u32,
    pub message: String,
}
```

**Type Standards Compliance**:

- ✅ Uses `Result<T, Error>` pattern (not wrapper structs)
- ✅ Follows canonical Rust error handling
- ✅ No "clowny" type names like `MemoryBulkOperationResponse`
- ✅ Consistent with existing `MemoryOperationResponse` pattern

**Benefits of Bulk API**:

- More efficient (no cross-capsule search)
- Better performance (single transaction)
- Clearer semantics (capsule-scoped)
- Future-proof for multi-canister architecture
- Matches existing pattern (`memories_list(capsule_id)`)

### **"Delete All" Method Consideration**

**Question**: Should we have `memories_delete_all(capsule_id: String)` for clearing entire capsule?

**Risks**:

- ⚠️ **Destructive Operation**: Deletes ALL memories in capsule
- ⚠️ **No Undo**: Irreversible data loss
- ⚠️ **User Error**: Easy to accidentally trigger
- ⚠️ **Capsule Preservation**: Should NOT delete the capsule itself

**Proposed Implementation**:

```rust
// High-risk method - requires explicit confirmation
memories_delete_all(capsule_id: String) -> Result<BulkDeleteResult, Error>

// Safety measures:
// 1. Require explicit user confirmation
// 2. Log all deletions for audit trail
// 3. Preserve capsule structure
// 4. Return detailed deletion report
```

**Recommendation**: **YES, but with strict safety measures**

- Implement as separate high-risk method
- Require explicit user confirmation
- Add comprehensive logging
- Preserve capsule structure (don't delete capsule itself)

### **Asset-Only Deletion API (Missing)**

**Current Gap**: No endpoint to delete assets while preserving memory metadata

**Missing APIs**:

```rust
// Asset-only deletion (preserves memory metadata)
memories_cleanup_assets(memory_id: String) -> Result<AssetCleanupResult, Error>

// Bulk asset cleanup
memories_cleanup_assets_bulk(memory_ids: Vec<String>) -> Result<BulkAssetCleanupResult, Error>
```

**Use Cases**:

- **Storage Cleanup**: Remove large assets but keep memory metadata
- **Privacy**: Remove sensitive assets while preserving memory structure
- **Cost Optimization**: Delete expensive external storage assets
- **Clear All**: Remove assets but keep memory records for audit

**Current Limitation**: `cleanup_memory_assets()` only runs during full memory deletion

### 2. Memory vs Asset Deletion Separation

**Current Backend Design**: `memories_delete` handles both memory metadata AND asset cleanup

- Memory deletion: Removes memory record from capsule
- Asset cleanup: Deletes blob assets from storage systems

**Proposed Separation**:

```rust
// Separate concerns for better error handling
memories_delete_bulk(capsule_id: String, memory_ids: Vec<String>) -> MemoryBulkOperationResponse
assets_cleanup_bulk(asset_ids: Vec<String>) -> AssetCleanupResponse
```

**Benefits**:

- Better error handling (memory deletion can succeed even if asset cleanup fails)
- More granular control and debugging
- Easier testing of individual operations
- Cleaner separation of concerns

### 3. ICP Asset Cleanup

**Backend Implementation**: `cleanup_memory_assets()` in `src/backend/src/memories/core.rs:357`

**What it cleans**:

- ✅ Inline assets (automatic with memory deletion)
- ✅ Internal blob assets (deletes from ICP blob store)
- ✅ External blob assets (deletes from external storage)

### 3. Multi-Canister Future Considerations

**Current Architecture**: Single canister with all capsules
**Future Architecture**: Capsules could be separate canisters

**Impact on Clear All**:

- **Current**: All operations happen within single canister
- **Future**: May need to handle multiple canister calls
- **Frontend Responsibility**: Detect capsule location and route calls appropriately

**Proposed API Design**:

```rust
// Current: Single canister
memories_delete_bulk(capsule_id: String, memory_ids: Vec<String>)

// Future: Multi-canister aware
memories_delete_bulk(capsule_id: String, memory_ids: Vec<String>, canister_id: Option<String>)
```

**Frontend Strategy**:

- Detect if capsule is in current canister or separate canister
- Route calls to appropriate canister
- Handle cross-canister operations gracefully

### 4. ICP Capsule Cleanup

**Backend API Available**: `capsules_delete(capsule_id: String)` in `src/backend/src/capsule.rs:373`

**Current Status**:

- ✅ Backend function exists
- ❌ Frontend integration not implemented

## Implementation Plan

### Phase 0: Backend API Enhancement (Required First)

**Priority**: HIGH - Frontend implementation depends on this

**4 New Backend Endpoints Required**:

1. **`memories_delete_bulk`** - Bulk memory deletion (efficient)
2. **`memories_delete_all`** - Clear entire capsule (high-risk)
3. **`memories_cleanup_assets`** - Asset-only deletion (preserves memory)
4. **`memories_cleanup_assets_bulk`** - Bulk asset cleanup

**Implementation Tasks**:

- [ ] **Backend API 1**: Implement `memories_delete_bulk(capsule_id: String, memory_ids: Vec<String>) -> Result<BulkDeleteResult, Error>`
- [ ] **Backend API 2**: Implement `memories_delete_all(capsule_id: String) -> Result<BulkDeleteResult, Error>` (high-risk method)
- [ ] **Backend API 3**: Implement `memories_cleanup_assets(memory_id: String) -> Result<AssetCleanupResult, Error>`
- [ ] **Backend API 4**: Implement `memories_cleanup_assets_bulk(memory_ids: Vec<String>) -> Result<BulkAssetCleanupResult, Error>`
- [ ] **Result Types**: Define `BulkDeleteResult`, `AssetCleanupResult`, `BulkAssetCleanupResult`
- [ ] **Backend DID**: Update `backend.did` with new API signatures
- [ ] **Testing**: Add comprehensive tests for all 4 new bulk operations

**Benefits**:

- More efficient than multiple single calls
- Better error handling and reporting
- Future-proof for multi-canister architecture

### Phase 1: Frontend Integration

**Frontend Implementation Tasks**:

- [ ] **ICP Memory Service**: Create `src/nextjs/src/services/icp-memory.ts` with functional ICP memory operations
- [ ] **Enhanced Clear All**: Update `deleteAllMemories()` in `src/nextjs/src/services/memories.ts` with ICP cleanup
- [ ] **Storage Edges API**: Create `/api/storage/edges` route for updating Neon storage edges
- [ ] **Error Handling**: Implement graceful degradation for ICP authentication failures
- [ ] **User Experience**: Update clear all button text and success messages

### Phase 1.1: ICP Memory Deletion Service

**File**: `src/nextjs/src/services/icp-gallery.ts`

**Current State**:

```typescript
async deleteMemory(_memoryId: string): Promise<MemoryOperationResponse> {
  // TODO: Update this call when declarations are regenerated
  // const result = await actor.memories_delete(memoryId);

  return {
    success: true,
    message: 'Memory deleted successfully',
  };
}
```

**Required Implementation**:

```typescript
async deleteMemory(memoryId: string): Promise<MemoryOperationResponse> {
  try {
    const actor = await backendActor(this.identity);
    const result = await actor.memories_delete(memoryId);

    return {
      success: result.success,
      message: result.message,
    };
  } catch (error) {
    return {
      success: false,
      message: `Failed to delete memory: ${error instanceof Error ? error.message : 'Unknown error'}`,
    };
  }
}
```

### Phase 2: Enhanced Clear All Function (Frontend)

**File**: `src/nextjs/src/services/memories.ts`

**Current Flow**:

```typescript
// Frontend: deleteAllMemories()
const response = await fetch(`/api/memories?all=true`, {
  method: "DELETE",
});
// Backend handles Neon + S3 cleanup
```

**Enhanced Flow**:

```typescript
// Frontend: deleteAllMemories()
export const deleteAllMemories = async (options?: {
  all?: boolean;
}): Promise<{ success: boolean; message: string; deletedCount: number }> => {
  // 1. Call backend for Neon + S3 cleanup
  const response = await fetch(`/api/memories?all=true`, {
    method: "DELETE",
  });

  // 2. Check if user is ICP-authenticated
  const { isICPAuthenticated } = useICPIdentity();

  if (isICPAuthenticated) {
    // 3. Get user's memories from ICP
    const icpMemories = await getICPMemories();

    // 4. Delete ICP memories
    await deleteICPMemories(icpMemories);

    // 5. Update Neon storage edges (mark ICP edges as deleted)
    await updateNeonStorageEdges(icpMemories, "deleted");
  }

  return response.json();
};
```

### Phase 3: ICP Memory Service (Functional)

**New File**: `src/nextjs/src/services/icp-memory.ts`

**Functional Implementation**:

```typescript
// Get all memories from user's capsule
export async function getICPMemories(): Promise<string[]> {
  try {
    const actor = await backendActor();
    const capsuleId = await getCurrentUserCapsuleId();
    const memories = await actor.memories_list(capsuleId);
    return memories.map((m) => m.id);
  } catch (error) {
    console.error("Failed to get ICP memories:", error);
    return [];
  }
}

// Delete multiple ICP memories
export async function deleteICPMemories(memoryIds: string[]): Promise<{
  successCount: number;
  errorCount: number;
  errors: string[];
}> {
  const results = await Promise.allSettled(memoryIds.map((memoryId) => deleteICPMemory(memoryId)));

  let successCount = 0;
  let errorCount = 0;
  const errors: string[] = [];

  results.forEach((result, index) => {
    if (result.status === "fulfilled" && result.value.success) {
      successCount++;
    } else {
      errorCount++;
      const error = result.status === "rejected" ? result.reason : result.value.message;
      errors.push(`Memory ${memoryIds[index]}: ${error}`);
    }
  });

  return { successCount, errorCount, errors };
}

// Delete single ICP memory
export async function deleteICPMemory(memoryId: string): Promise<{
  success: boolean;
  message: string;
}> {
  try {
    const actor = await backendActor();
    const result = await actor.memories_delete(memoryId);

    return {
      success: result.success,
      message: result.message,
    };
  } catch (error) {
    return {
      success: false,
      message: `Failed to delete ICP memory: ${error instanceof Error ? error.message : "Unknown error"}`,
    };
  }
}

// Update Neon storage edges to mark ICP memories as deleted
export async function updateNeonStorageEdges(memoryIds: string[], status: "deleted" | "active"): Promise<void> {
  try {
    const response = await fetch("/api/storage/edges", {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        memoryIds,
        status,
        storageBackend: "icp",
      }),
    });

    if (!response.ok) {
      throw new Error("Failed to update storage edges");
    }
  } catch (error) {
    console.error("Failed to update Neon storage edges:", error);
  }
}
```

### Phase 4: Implementation Summary

**Key Changes**:

1. **Frontend-only ICP cleanup** - No server-side ICP operations
2. **Functional programming** - No OOP classes, pure functions
3. **Capsule preservation** - User's capsule is NOT deleted (represents the user)
4. **Memory-only deletion** - Only delete memories, not the capsule itself
5. **Neon edge updates** - Mark ICP storage edges as deleted in Neon database

## Technical Considerations

### 1. Authentication Requirements

**Solution**: Frontend-only ICP cleanup with user authentication

**Implementation**:

- Check `useICPIdentity()` hook for authentication status
- Only attempt ICP deletion if user is authenticated
- Graceful degradation if not authenticated
- No server-side ICP operations needed

### 2. Error Handling

**Current**: Clear all fails if any step fails

**Enhanced**:

- Neon cleanup (critical) - fail if errors
- S3 cleanup (important) - log errors, continue
- ICP cleanup (optional) - log errors, continue

### 3. Performance

**Current**: Sequential cleanup (Neon → S3)

**Enhanced**: Parallel cleanup (Neon + S3) + Sequential ICP

```typescript
// Backend: Parallel Neon + S3 cleanup
const [neonResults, s3Results] = await Promise.allSettled([deleteNeonMemories(), cleanupStorageEdgesForMemories()]);

// Frontend: ICP cleanup (after backend completes)
if (isICPAuthenticated) {
  const icpResults = await deleteICPMemories(memoryIds);
}
```

### 4. User Experience

**Current**: "Successfully deleted X memories"

**Enhanced**: "Successfully deleted X memories (X from Neon, X from S3, X from ICP)"

## Implementation Priority

### High Priority (MVP)

1. ✅ Implement `deleteMemory()` in ICP service
2. ✅ Add ICP cleanup to clear all function
3. ✅ Handle authentication gracefully

### Medium Priority

1. ✅ Parallel cleanup for performance
2. ✅ Enhanced error reporting
3. ✅ ICP capsule cleanup

### Low Priority

1. ✅ Detailed cleanup reporting
2. ✅ Rollback on partial failures
3. ✅ Cleanup verification

## Files to Modify

1. **`src/nextjs/src/services/memories.ts`** - Enhance `deleteAllMemories()` with ICP cleanup
2. **`src/nextjs/src/services/icp-memory.ts`** - NEW: Functional ICP memory service
3. **`src/nextjs/src/app/api/storage/edges/route.ts`** - NEW: API to update storage edges
4. **`src/nextjs/src/components/dashboard/dashboard-top-bar.tsx`** - Update clear all button text

## Success Criteria

- ✅ Clear all deletes Neon records
- ✅ Clear all deletes S3 assets (existing)
- ✅ Clear all deletes ICP memories and assets (new)
- ✅ Clear all handles ICP authentication gracefully
- ✅ Clear all provides detailed success/error reporting
- ✅ Clear all works for users with mixed storage (S3 + ICP)

## Testing Scenarios

1. **User with only Neon data** - Should work as before
2. **User with S3 + Neon data** - Should work as before
3. **User with ICP + Neon data** - Should delete both
4. **User with S3 + ICP + Neon data** - Should delete all
5. **User not ICP-authenticated** - Should skip ICP cleanup gracefully
6. **ICP deletion fails** - Should continue with Neon cleanup
7. **Partial ICP deletion** - Should report partial success
