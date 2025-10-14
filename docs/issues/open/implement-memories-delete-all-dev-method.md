# Implement memories_delete_all_dev Method

## Issue Summary

**Enhancement**: Implement a simplified, developer-focused `memories_delete_all_dev` method to replace the current complex and unreliable `memories_delete_all` function. This will solve the "always fails 1 memory" issue and provide a fast, atomic capsule-scoped clear operation.

## Current Problem

The existing `memories_delete_all` function has several issues:

1. **Complex Logic**: Iterates through accessible capsules and individual memory deletions
2. **Race Conditions**: Between listing memories and deleting them, state can change
3. **Partial Failures**: "Always fails 1 memory" due to individual deletion complexity
4. **Performance Issues**: N+1 queries and multiple backend calls
5. **Over-Engineering**: Too many failure points for a simple "clear all" operation

## Proposed Solution

Implement a **developer method** with clear naming and documentation that:

1. **Bypasses Complex ACL Checks**: Since it's a dev method
2. **Uses Atomic Operations**: Single capsule-level clear
3. **Eliminates Race Conditions**: No individual memory handling
4. **Provides Fast Performance**: Single database operation
5. **Makes Technical Debt Explicit**: Clear naming and documentation

## Existing Infrastructure

We already have most of the infrastructure needed:

### ✅ **Asset Cleanup Functions** (Already Implemented)

- **`cleanup_memory_assets(memory: &Memory)`** - Cleans up all assets for a memory
- **`cleanup_internal_blob_asset(blob_ref: &BlobRef)`** - Deletes internal blobs from ICP storage
- **`cleanup_external_blob_asset(external_asset: &MemoryAssetBlobExternal)`** - Handles external assets (no-op)

### ✅ **Store Layer Methods** (Already Implemented)

- **`get_all_memories(capsule: &CapsuleId) -> Vec<Memory>`** - Gets all memories in a capsule
- **`delete_memory(capsule: &CapsuleId, id: &MemoryId)`** - Deletes individual memories
- **`get_capsule_for_acl(capsule_id: &CapsuleId)`** - Gets capsule ACL info

### ✅ **Individual Asset Removal** (Already Implemented)

- **`asset_remove_by_id`** - Removes specific assets by ID
- **`asset_remove_internal`** - Removes internal blob assets
- **`asset_remove_external`** - Removes external blob assets

### ❌ **Missing Methods** (Need to Implement)

- **`clear_all_memories_in_capsule`** - Atomic memory clearing
- **`clear_all_internal_blobs_in_capsule`** - Atomic asset clearing
- **`capsule_exists`** - Capsule existence check
- **`_dev_clear_all_memories_in_capsule`** - Main dev function

## Implementation Plan

### Step 1: Backend Core Function

**File**: `src/backend/src/memories/core/delete.rs`

```rust
/// TEMPORARY DEV METHOD: Clear all memories in a capsule
///
/// WARNING: This is a developer method that bypasses normal ACL checks
/// and uses atomic operations. It should be refined or removed before
/// production use.
///
/// TODO: Implement proper ACL checks and individual memory deletion
/// TODO: Add proper error handling and rollback mechanisms
/// TODO: Consider if this should be a user-facing feature
pub fn _dev_clear_all_memories_in_capsule_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    capsule_id: String,
    delete_assets: bool
) -> std::result::Result<crate::memories::types::BulkDeleteResult, Error> {
    let caller = env.caller();

    // Simple check: just verify the capsule exists
    if !store.capsule_exists(&capsule_id) {
        return Err(Error::NotFound);
    }

    // Get count of memories before deletion (this is our main return value)
    let memories = store.get_all_memories(&capsule_id);
    let deleted_memory_count = memories.len();  // This is what we return

    // If delete_assets=true, delete assets first, then memories
    if delete_assets {
        // Delete internal blob assets (ICP storage)
        store.clear_all_internal_blobs_in_capsule(&capsule_id)?;

        // For external blob assets, we can't delete them (they're on external storage)
        // So we just return true and do nothing - the external storage will handle cleanup
        // This is the expected behavior for external assets
    }

    // NUCLEAR OPTION: Clear all memories in this capsule
    store.clear_all_memories_in_capsule(&capsule_id)?;

    // Return the count of deleted memories (this is the main result)
    Ok(crate::memories::types::BulkDeleteResult {
        deleted_count: deleted_memory_count,  // PRIMARY RETURN VALUE: number of memories deleted
        failed_count: 0,
        message: format!(
            "Successfully cleared {} memories from capsule {}",
            deleted_memory_count,
            capsule_id
        ),
    })
}
```

### Step 2: Backend Function Export

**File**: `src/backend/src/memories/core.rs`

```rust
// Add to existing exports
pub use delete::{
    memories_delete_all_core,
    _dev_clear_all_memories_in_capsule_core,  // NEW
    memories_delete_bulk_core,
    memories_delete_core
};
```

### Step 3: Backend Canister Function

**File**: `src/backend/src/lib.rs`

```rust
/// TEMPORARY DEV METHOD: Clear all memories in a capsule
///
/// WARNING: This is a developer method that bypasses normal ACL checks
/// and uses atomic operations. It should be refined or removed before
/// production use.
///
/// TODO: Implement proper ACL checks and individual memory deletion
/// TODO: Add proper error handling and rollback mechanisms
/// TODO: Consider if this should be a user-facing feature
#[ic_cdk::update]
fn _dev_clear_all_memories_in_capsule(
    capsule_id: String,
    delete_assets: bool,
) -> Result<crate::memories::types::BulkDeleteResult, Error> {
    use crate::memories::core::_dev_clear_all_memories_in_capsule_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    _dev_clear_all_memories_in_capsule_core(&env, &mut store, capsule_id, delete_assets)
}
```

### Step 4: Add Missing Store Trait Methods

**File**: `src/backend/src/memories/core/traits.rs`

First, add the missing methods to the Store trait:

```rust
/// Storage abstraction for capsule store operations
pub trait Store {
    // ... existing methods ...

    /// Clear all memories in a capsule (atomic operation)
    fn clear_all_memories_in_capsule(&mut self, capsule_id: &str) -> Result<(), Error>;

    /// Clear all internal blobs in a capsule (atomic operation)
    fn clear_all_internal_blobs_in_capsule(&mut self, capsule_id: &str) -> Result<(), Error>;

    /// Check if capsule exists
    fn capsule_exists(&self, capsule_id: &str) -> bool;
}
```

### Step 5: Store Layer Implementation

**File**: `src/backend/src/memories/adapters.rs`

Implement the new methods using existing infrastructure:

```rust
impl crate::memories::core::Store for StoreAdapter {
    // ... existing methods ...

    /// Clear all memories in a capsule (atomic operation)
    fn clear_all_memories_in_capsule(&mut self, capsule_id: &str) -> Result<(), Error> {
        with_capsule_store_mut(|store| {
            match store.update_with(capsule_id, |capsule_data| {
                // Clear all memories in the capsule
                capsule_data.memories.clear();
                Ok(())
            }) {
                Ok(_) => Ok(()),
                Err(e) => Err(Error::Internal(format!("Failed to clear memories: {:?}", e))),
            }
        })
    }

    /// Clear all internal blobs in a capsule (atomic operation)
    fn clear_all_internal_blobs_in_capsule(&mut self, capsule_id: &str) -> Result<(), Error> {
        // Get all memories first to access their assets
        let memories = self.get_all_memories(capsule_id);

        // Clean up assets for each memory
        for memory in memories {
            // Use existing cleanup function for each memory's assets
            crate::memories::core::assets::cleanup_memory_assets(&memory)?;
        }

        Ok(())
    }

    /// Check if capsule exists
    fn capsule_exists(&self, capsule_id: &str) -> bool {
        with_capsule_store(|store| {
            store.get(capsule_id).is_some()
        })
    }
}
```

**Note**: We can reuse the existing `cleanup_memory_assets` function from `src/backend/src/memories/core/assets.rs` which already handles:

- Internal blob asset cleanup via `cleanup_internal_blob_asset`
- External blob asset cleanup via `cleanup_external_blob_asset`
- Proper blob store operations

### Step 6: Frontend Update

**File**: `src/nextjs/src/services/memories.ts`

```typescript
// Update the deleteAllMemoriesFromICP function
const deleteAllMemoriesFromICP = async (options?: {
  type?: "image" | "document" | "note" | "video" | "audio";
  folder?: string;
  all?: boolean;
}): Promise<{ success: boolean; message: string; deletedCount: number }> => {
  try {
    // Import ICP dependencies
    const { getAuthClient } = await import("@/ic/ii");
    const { backendActor } = await import("@/ic/backend");

    // Get authenticated actor
    const authClient = await getAuthClient();
    if (!authClient.isAuthenticated()) {
      throw new Error("Please connect your Internet Identity to delete ICP memories");
    }

    const identity = authClient.getIdentity();
    const backend = await backendActor(identity);

    // Get capsule ID
    const capsuleResult = await backend.capsules_read_basic([]);
    if (!("Ok" in capsuleResult)) {
      throw new Error("Failed to get user capsule");
    }
    const capsuleId = capsuleResult.Ok.capsule_id;

    // Use the new dev method for efficient deletion
    console.log("🔍 [Delete All Dev] Calling _dev_clear_all_memories_in_capsule for capsule:", capsuleId);
    const deleteAllResult = await backend._dev_clear_all_memories_in_capsule(capsuleId, true); // true = delete assets

    if ("Ok" in deleteAllResult) {
      const result = deleteAllResult.Ok;
      console.log("🔍 [Delete All Dev] Success:", result);
      return {
        success: true,
        message: result.message,
        deletedCount: result.deleted_count,
      };
    } else {
      console.error("🔍 [Delete All Dev] Failed:", deleteAllResult.Err);
      throw new Error(`Failed to delete all memories: ${JSON.stringify(deleteAllResult.Err)}`);
    }
  } catch (error) {
    console.error("Failed to delete memories from ICP:", error);
    throw new Error(`Failed to delete ICP memories: ${error instanceof Error ? error.message : "Unknown error"}`);
  }
};
```

## Return Value

The function returns a `BulkDeleteResult` with:

- **`deleted_count`**: Number of **memories** that were deleted (this is the main return value)
- **`failed_count`**: Number of failed deletions (should always be 0 for this dev method)
- **`message`**: Human-readable success message

**Key Point**: The **primary return value is the count of deleted memories**, not assets. Assets are deleted as a side effect when `delete_assets=true`.

**Example Return Values**:

```rust
// Empty capsule
BulkDeleteResult { deleted_count: 0, failed_count: 0, message: "Successfully cleared 0 memories from capsule capsule_123" }

// Single memory
BulkDeleteResult { deleted_count: 1, failed_count: 0, message: "Successfully cleared 1 memories from capsule capsule_123" }

// Multiple memories
BulkDeleteResult { deleted_count: 5, failed_count: 0, message: "Successfully cleared 5 memories from capsule capsule_123" }
```

## Asset Deletion Strategy

### **Internal Blob Assets (ICP Storage)**

- **When `delete_assets=true`**: Delete all internal blobs from ICP storage
- **Implementation**: `clear_all_internal_blobs_in_capsule()` - atomic operation
- **Storage**: ICP blob store (canister storage)

### **External Blob Assets (S3, Vercel Blob, etc.)**

- **When `delete_assets=true`**: Return `true` and do nothing
- **Reason**: We can't delete external assets from ICP canister
- **Expected Behavior**: External storage providers handle cleanup
- **Implementation**: No-op (always succeeds)

### **Asset Deletion Order**

1. **First**: Delete internal blob assets (if `delete_assets=true`)
2. **Second**: Delete memory records
3. **External assets**: Always return success (no-op)

### **Flag Behavior**

- **`delete_assets=true`**: Delete internal assets + memories
- **`delete_assets=false`**: Delete only memories (keep assets)

## Benefits

### ✅ **Solves Current Issues**

- **No more "always fails 1 memory"**: Atomic operation eliminates race conditions
- **Better Performance**: Single operation instead of N+1 queries
- **Simpler Logic**: No complex individual memory handling
- **Reliable**: All-or-nothing operation

### ✅ **Developer Experience**

- **Fast Testing**: Quick clear operations for development
- **Predictable**: Clear success/failure behavior
- **Debuggable**: Simple logic, easy to trace issues

### ✅ **Technical Debt Management**

- **Clear Naming**: `_dev` suffix makes it obvious
- **Documentation**: WARNING and TODO comments
- **Future Refactoring**: Easy to find and improve

## Implementation TODO List

### **Phase 1: Backend Store Layer**

- [ ] **Add Store Trait Methods** (`src/backend/src/memories/core/traits.rs`)

  - [ ] Add `clear_all_memories_in_capsule(&mut self, capsule_id: &str) -> Result<(), Error>`
  - [ ] Add `clear_all_internal_blobs_in_capsule(&mut self, capsule_id: &str) -> Result<(), Error>`
  - [ ] Add `capsule_exists(&self, capsule_id: &str) -> bool`

- [ ] **Implement Store Adapter Methods** (`src/backend/src/memories/adapters.rs`)
  - [ ] Implement `clear_all_memories_in_capsule` using `with_capsule_store_mut`
  - [ ] Implement `clear_all_internal_blobs_in_capsule` using existing `cleanup_memory_assets`
  - [ ] Implement `capsule_exists` using `with_capsule_store`

### **Phase 2: Backend Core Logic**

- [ ] **Add Core Function** (`src/backend/src/memories/core/delete.rs`)

  - [ ] Implement `_dev_clear_all_memories_in_capsule_core`
  - [ ] Add proper error handling and logging
  - [ ] Add WARNING and TODO documentation comments

- [ ] **Export Core Function** (`src/backend/src/memories/core.rs`)
  - [ ] Add `_dev_clear_all_memories_in_capsule_core` to exports

### **Phase 3: Backend Canister Function**

- [ ] **Add Canister Function** (`src/backend/src/lib.rs`)
  - [ ] Implement `_dev_clear_all_memories_in_capsule`
  - [ ] Add `#[ic_cdk::update]` annotation
  - [ ] Add WARNING and TODO documentation comments

### **Phase 4: Frontend Integration**

- [ ] **Update Frontend Service** (`src/nextjs/src/services/memories.ts`)
  - [ ] Update `deleteAllMemoriesFromICP` to use `_dev_clear_all_memories_in_capsule`
  - [ ] Add proper error handling and logging
  - [ ] Update console log messages

### **Phase 5: Testing & Validation**

- [ ] **Backend Testing**

  - [ ] Test with empty capsule (should return 0 deleted count)
  - [ ] Test with single memory (should delete 1 memory)
  - [ ] Test with multiple memories (should delete all memories)
  - [ ] Test with `delete_assets=true` (should delete assets and memories)
  - [ ] Test with `delete_assets=false` (should delete only memories)
  - [ ] Test with non-existent capsule (should return NotFound error)

- [ ] **Frontend Testing**
  - [ ] Test delete all button functionality
  - [ ] Verify dashboard refreshes after deletion
  - [ ] Check console logs for proper success/error messages
  - [ ] Verify no more "always fails 1 memory" issues

### **Phase 6: Documentation & Cleanup**

- [ ] **Update Documentation**

  - [ ] Add function to API documentation
  - [ ] Update issue status to resolved
  - [ ] Document the dev method warning

- [ ] **Code Review**
  - [ ] Review implementation for security implications
  - [ ] Verify proper error handling
  - [ ] Check for any memory leaks or edge cases

## Implementation Steps (Summary)

1. **Add Missing Store Trait Methods** (clear_all_memories_in_capsule, clear_all_internal_blobs_in_capsule, capsule_exists)
2. **Implement Store Layer Methods** (using existing infrastructure)
3. **Add Core Function** (\_dev_clear_all_memories_in_capsule_core)
4. **Add Backend Function** (\_dev_clear_all_memories_in_capsule)
5. **Update Frontend** (use \_dev_clear_all_memories_in_capsule)
6. **Test Implementation** (verify it works correctly)
7. **Update Documentation** (add to API docs)

## Testing

### Test Cases

1. **Empty Capsule**: Should return 0 deleted count
2. **Single Memory**: Should delete 1 memory successfully
3. **Multiple Memories**: Should delete all memories successfully
4. **With Internal Assets (`delete_assets=true`)**: Should delete memories and internal assets
5. **With External Assets (`delete_assets=true`)**: Should delete memories, return success for external assets (no-op)
6. **Without Asset Deletion (`delete_assets=false`)**: Should delete only memories, keep all assets
7. **Non-existent Capsule**: Should return NotFound error

### Success Criteria

- [ ] No more "always fails 1 memory" issues
- [ ] Fast execution (single operation)
- [ ] All memories deleted successfully
- [ ] All assets deleted when delete_assets=true
- [ ] Clear error messages for failures
- [ ] Proper logging for debugging

## Future Considerations

### Production Readiness

- **ACL Checks**: Add proper permission validation
- **Error Handling**: Add rollback mechanisms
- **User Interface**: Consider if this should be user-facing
- **Audit Logging**: Add proper audit trails

### Alternative Approaches

- **Batch Operations**: Implement proper batch deletion
- **Soft Delete**: Consider soft delete with cleanup
- **Archive**: Consider archiving instead of deletion

## Status

🔧 **IMPLEMENTATION** - Ready for development

**Date**: 2025-01-14  
**Priority**: High (solves critical delete all issues)  
**Effort**: Medium (requires store layer implementation)

## Related Issues

- **Delete All Memories Not Working**: This issue will resolve the root cause
- **Placeholder Mystery**: May be related if old memories are persisting
- **Asset Type Tagging Mismatch**: Separate issue, but affects asset cleanup
