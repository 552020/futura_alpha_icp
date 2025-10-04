# Backend Bulk Memory APIs Implementation

## Overview

This document outlines the implementation of 8 new backend API endpoints for comprehensive memory and asset management, required for the "Clear All" ICP integration functionality.

## Required API Endpoints (MVP - 8 Endpoints)

### **Naming Strategy Note**

**Current Implementation**: All endpoints are implemented in the `memories/` module for MVP
**Future Refactoring**: Asset-related endpoints (`asset_*`) will be moved to the `assets/` module when the upload module is split

**Implementation Location**:

- **Memory operations** (`memories_*`) → `src/backend/src/memories/core.rs`
- **Asset operations** (`asset_*`) → `src/backend/src/memories/core.rs` (MVP) → `src/backend/src/assets/` (future)

**Endpoint Naming**:

- **Memory operations**: `memories_*` (stay in memories module)
- **Asset operations**: `asset_*` (future assets module)
- **Blob operations**: `blob_*` (future blob module)

### **Core "Clear All" Operations (4 endpoints)**

**Current Gap Analysis**:

- ✅ `memories_update(memory_id, MemoryUpdateData)` - Can update metadata
- ✅ `cleanup_memory_assets(memory)` - Deletes ALL assets (only during memory deletion)
- ❌ **No API to delete specific assets** - Only bulk cleanup during memory deletion
- ❌ **No API to remove individual asset references** from memories
- ❌ **No granular asset management** - Can't target specific `BlobRef` or `storage_key`

**Important**: When deleting memories (`memories_delete_bulk`, `memories_delete_all`), **all associated assets are automatically deleted** via `cleanup_memory_assets()`. This includes:

- **Inline assets** (stored in memory struct)
- **ICP blob assets** (deleted from ICP blob store)
- **External assets** (deleted from S3/Vercel/etc.)

### **Granular Asset Operations (4 additional endpoints)**

### 5. `asset_remove` - Remove Specific Asset

**Purpose**: Remove a specific asset from a memory by asset reference

**Signature**:

```rust
asset_remove(memory_id: String, asset_ref: String) -> Result<AssetRemovalResult, Error>
```

### 6. `asset_remove_inline` - Remove Inline Asset

**Purpose**: Remove specific inline asset by index

**Signature**:

```rust
asset_remove_inline(memory_id: String, asset_index: u32) -> Result<AssetRemovalResult, Error>
```

### 7. `asset_remove_internal` - Remove ICP Blob Asset

**Purpose**: Remove specific ICP blob asset by blob reference

**Signature**:

```rust
asset_remove_internal(memory_id: String, blob_ref: String) -> Result<AssetRemovalResult, Error>
```

### 8. `asset_remove_external` - Remove External Asset

**Purpose**: Remove specific external storage asset by storage key

**Signature**:

```rust
asset_remove_external(memory_id: String, storage_key: String) -> Result<AssetRemovalResult, Error>
```

### 9. `memories_list_assets` - List Memory Assets

**Purpose**: Get detailed list of all assets in a memory

**Signature**:

```rust
memories_list_assets(memory_id: String) -> Result<MemoryAssetsList, Error>
```

### 1. `memories_delete_bulk` - Bulk Memory Deletion

**Purpose**: Efficiently delete multiple memories in a single operation

**Signature**:

```rust
memories_delete_bulk(capsule_id: String, memory_ids: Vec<String>) -> Result<BulkDeleteResult, Error>
```

**Implementation Location**: `src/backend/src/lib.rs`

**Core Logic**:

```rust
#[ic_cdk::update]
fn memories_delete_bulk(capsule_id: String, memory_ids: Vec<String>) -> Result<BulkDeleteResult, Error> {
    use crate::memories::core::memories_delete_bulk_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    memories_delete_bulk_core(&env, &mut store, capsule_id, memory_ids)
}
```

**Core Function** (`src/backend/src/memories/core.rs`):

```rust
pub fn memories_delete_bulk_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    capsule_id: CapsuleId,
    memory_ids: Vec<MemoryId>,
) -> std::result::Result<BulkDeleteResult, Error> {
    let caller = env.caller();

    // Check capsule access permissions
    let capsule_access = store
        .get_capsule_for_acl(&capsule_id)
        .ok_or(Error::NotFound)?;

    if !capsule_access.can_delete(&caller) {
        return Err(Error::Unauthorized);
    }

    let mut deleted_count = 0;
    let mut failed_count = 0;
    let mut errors = Vec::new();

    for memory_id in memory_ids {
        match memories_delete_core(env, store, memory_id.clone()) {
            Ok(()) => deleted_count += 1,
            Err(e) => {
                failed_count += 1;
                errors.push(format!("Memory {}: {}", memory_id, e));
            }
        }
    }

    Ok(BulkDeleteResult {
        deleted_count,
        failed_count,
        message: format!("Deleted {} memories, {} failed", deleted_count, failed_count),
    })
}
```

### 2. `memories_delete_all` - Clear Entire Capsule

**Purpose**: Delete ALL memories in a capsule (high-risk operation)

**Signature**:

```rust
memories_delete_all(capsule_id: String) -> Result<BulkDeleteResult, Error>
```

**Safety Measures**:

- Require explicit user confirmation
- Log all deletions for audit trail
- Preserve capsule structure
- Return detailed deletion report

**Implementation**:

```rust
#[ic_cdk::update]
fn memories_delete_all(capsule_id: String) -> Result<BulkDeleteResult, Error> {
    use crate::memories::core::memories_delete_all_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    memories_delete_all_core(&env, &mut store, capsule_id)
}
```

**Core Function**:

```rust
pub fn memories_delete_all_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    capsule_id: CapsuleId,
) -> std::result::Result<BulkDeleteResult, Error> {
    let caller = env.caller();

    // Check capsule access permissions
    let capsule_access = store
        .get_capsule_for_acl(&capsule_id)
        .ok_or(Error::NotFound)?;

    if !capsule_access.can_delete(&caller) {
        return Err(Error::Unauthorized);
    }

    // Get all memories in the capsule
    let capsule = store.get(&capsule_id).ok_or(Error::NotFound)?;
    let memory_ids: Vec<String> = capsule.memories.keys().cloned().collect();

    // Delete all memories using bulk operation
    memories_delete_bulk_core(env, store, capsule_id, memory_ids)
}
```

### 3. `memories_cleanup_assets_all` - Remove ALL Assets

**Purpose**: Remove ALL assets (inline + internal + external) while **preserving memory metadata**

**Note**: This is different from memory deletion - it only removes assets but keeps the memory record intact.

**Signature**:

```rust
memories_cleanup_assets_all(memory_id: String) -> Result<AssetCleanupResult, Error>
```

**Implementation**:

```rust
#[ic_cdk::update]
fn memories_cleanup_assets(memory_id: String) -> Result<AssetCleanupResult, Error> {
    use crate::memories::core::memories_cleanup_assets_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    memories_cleanup_assets_core(&env, &mut store, memory_id)
}
```

**Core Function**:

```rust
pub fn memories_cleanup_assets_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_id: MemoryId,
) -> std::result::Result<AssetCleanupResult, Error> {
    let caller = env.caller();

    // Find the memory across all accessible capsules
    let accessible_capsules = store.get_accessible_capsules(&caller);

    for capsule_id in accessible_capsules {
        if let Some(memory) = store.get_memory(&capsule_id, &memory_id) {
            // Check permissions
            let capsule_access = store
                .get_capsule_for_acl(&capsule_id)
                .ok_or(Error::NotFound)?;

            if !capsule_access.can_write(&caller) {
                return Err(Error::Unauthorized);
            }

            // Clean up assets but preserve memory
            let cleanup_result = cleanup_memory_assets(&memory)?;

            // Update memory to remove asset references
            let mut updated_memory = memory.clone();
            updated_memory.blob_internal_assets.clear();
            updated_memory.blob_external_assets.clear();
            updated_memory.inline_assets.clear();

            store.insert_memory(&capsule_id, updated_memory)?;

            return Ok(AssetCleanupResult {
                memory_id: memory_id.clone(),
                assets_cleaned: cleanup_result.assets_cleaned,
                message: format!("Cleaned {} assets from memory", cleanup_result.assets_cleaned),
            });
        }
    }

    Err(Error::NotFound)
}
```

### 4. `memories_cleanup_assets_bulk` - Bulk Asset Cleanup

**Purpose**: Clean up assets from multiple memories

**Signature**:

```rust
memories_cleanup_assets_bulk(memory_ids: Vec<String>) -> Result<BulkAssetCleanupResult, Error>
```

**Implementation**:

```rust
#[ic_cdk::update]
fn memories_cleanup_assets_bulk(memory_ids: Vec<String>) -> Result<BulkAssetCleanupResult, Error> {
    use crate::memories::core::memories_cleanup_assets_bulk_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    memories_cleanup_assets_bulk_core(&env, &mut store, memory_ids)
}
```

**Core Function**:

```rust
pub fn memories_cleanup_assets_bulk_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_ids: Vec<MemoryId>,
) -> std::result::Result<BulkAssetCleanupResult, Error> {
    let mut cleaned_count = 0;
    let mut failed_count = 0;
    let mut total_assets_cleaned = 0;
    let mut errors = Vec::new();

    for memory_id in memory_ids {
        match memories_cleanup_assets_core(env, store, memory_id.clone()) {
            Ok(result) => {
                cleaned_count += 1;
                total_assets_cleaned += result.assets_cleaned;
            },
            Err(e) => {
                failed_count += 1;
                errors.push(format!("Memory {}: {}", memory_id, e));
            }
        }
    }

    Ok(BulkAssetCleanupResult {
        cleaned_count,
        failed_count,
        total_assets_cleaned,
        message: format!("Cleaned {} memories, {} failed, {} total assets cleaned",
                        cleaned_count, failed_count, total_assets_cleaned),
    })
}
```

## Result Types

### `BulkDeleteResult`

```rust
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct BulkDeleteResult {
    pub deleted_count: u32,
    pub failed_count: u32,
    pub message: String,
}
```

### `AssetCleanupResult`

```rust
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct AssetCleanupResult {
    pub memory_id: String,
    pub assets_cleaned: u32,
    pub message: String,
}
```

### `BulkAssetCleanupResult`

```rust
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct BulkAssetCleanupResult {
    pub cleaned_count: u32,
    pub failed_count: u32,
    pub total_assets_cleaned: u32,
    pub message: String,
}
```

## Implementation Steps

### Step 1: Add Result Types

**File**: `src/backend/src/types.rs`

Add the three result types to the types module.

### Step 2: Implement Core Functions

**File**: `src/backend/src/memories/core.rs`

Add the four core functions following the existing pattern:

- `memories_delete_bulk_core`
- `memories_delete_all_core`
- `memories_cleanup_assets_core`
- `memories_cleanup_assets_bulk_core`

### Step 3: Add Public API Endpoints

**File**: `src/backend/src/lib.rs`

Add the four public API endpoints following the existing pattern.

### Step 4: Update Candid Interface

**File**: `src/backend/backend.did`

Add the new endpoint signatures to the Candid interface.

### Step 5: Add Tests

**File**: `src/backend/tests/memories_pocket_ic.rs`

Add comprehensive tests for all four endpoints.

## Error Handling

All endpoints follow the established error handling pattern:

- Use `Result<T, Error>` return type
- Check permissions using existing ACL system
- Provide detailed error messages
- Handle partial failures gracefully

## Security Considerations

### Permission Checks

- All endpoints check capsule access permissions
- `memories_delete_all` requires delete permissions (high-risk)
- `memories_cleanup_assets` requires write permissions
- Bulk operations check permissions for each memory

### Audit Logging

- Log all bulk operations for audit trail
- Track success/failure counts
- Record error details for debugging

## Performance Considerations

### Bulk Operations

- Process multiple items in single transaction
- Return detailed success/failure counts
- Continue processing even if individual items fail

### Memory Efficiency

- Use iterator patterns for large datasets
- Avoid loading all memories into memory at once
- Process items in batches if needed

## Testing Strategy

### Unit Tests

- Test each core function individually
- Test permission checks
- Test error handling scenarios

### Integration Tests

- Test with PocketIC for realistic scenarios
- Test bulk operations with large datasets
- Test partial failure scenarios

### Edge Cases

- Empty memory lists
- Non-existent memories
- Permission denied scenarios
- Network/connection failures

## Files to Modify

1. **`src/backend/src/types.rs`** - Add result types
2. **`src/backend/src/memories/core.rs`** - Add core functions
3. **`src/backend/src/lib.rs`** - Add public API endpoints
4. **`src/backend/backend.did`** - Update Candid interface
5. **`src/backend/tests/memories_pocket_ic.rs`** - Add tests

## Implementation Todo List

### **Phase 1: Backend Core Implementation**

#### **1.1 Result Types (Priority: HIGH)**

- [ ] **Add `BulkDeleteResult`** to `src/backend/src/memories/types.rs`
- [ ] **Add `AssetCleanupResult`** to `src/backend/src/memories/types.rs`
- [ ] **Add `BulkAssetCleanupResult`** to `src/backend/src/memories/types.rs`
- [ ] **Add `AssetRemovalResult`** to `src/backend/src/memories/types.rs`
- [ ] **Add `MemoryAssetsList`** to `src/backend/src/memories/types.rs`

#### **1.2 Core Functions (Priority: HIGH)**

- [ ] **`memories_delete_bulk_core`** in `src/backend/src/memories/core.rs`
- [ ] **`memories_delete_all_core`** in `src/backend/src/memories/core.rs`
- [ ] **`memories_cleanup_assets_all_core`** in `src/backend/src/memories/core.rs`
- [ ] **`memories_cleanup_assets_bulk_core`** in `src/backend/src/memories/core.rs`

#### **1.3 Granular Asset Functions (Priority: HIGH)**

- [ ] **`asset_remove_core`** in `src/backend/src/memories/core.rs`
- [ ] **`asset_remove_inline_core`** in `src/backend/src/memories/core.rs`
- [ ] **`asset_remove_internal_core`** in `src/backend/src/memories/core.rs`
- [ ] **`asset_remove_external_core`** in `src/backend/src/memories/core.rs`
- [ ] **`memories_list_assets_core`** in `src/backend/src/memories/core.rs`

#### **1.4 Public API Endpoints (Priority: HIGH)**

- [ ] **`memories_delete_bulk`** in `src/backend/src/lib.rs`
- [ ] **`memories_delete_all`** in `src/backend/src/lib.rs`
- [ ] **`memories_cleanup_assets_all`** in `src/backend/src/lib.rs`
- [ ] **`memories_cleanup_assets_bulk`** in `src/backend/src/lib.rs`
- [ ] **`asset_remove`** in `src/backend/src/lib.rs`
- [ ] **`asset_remove_inline`** in `src/backend/src/lib.rs`
- [ ] **`asset_remove_internal`** in `src/backend/src/lib.rs`
- [ ] **`asset_remove_external`** in `src/backend/src/lib.rs`
- [ ] **`memories_list_assets`** in `src/backend/src/lib.rs`

### **Phase 2: Interface & Testing**

#### **2.1 Candid Interface (Automatic)**

- [ ] **Candid interface auto-updates** when new endpoints are added to `lib.rs`
- [ ] **TypeScript declarations** will be regenerated automatically

#### **2.2 Testing (Priority: HIGH)**

- [ ] **Unit tests** for all core functions in `src/backend/tests/memories_pocket_ic.rs`
- [ ] **Integration tests** for bulk operations
- [ ] **Permission tests** for ACL checks
- [ ] **Error handling tests** for partial failures
- [ ] **Edge case tests** (empty lists, non-existent memories)

### **Phase 3: Documentation & Validation**

#### **3.1 Documentation (Priority: MEDIUM)**

- [ ] **Update API documentation** with new endpoints
- [ ] **Add usage examples** for each endpoint
- [ ] **Document security considerations**

#### **3.2 Validation (Priority: HIGH)**

- [ ] **Compilation check** - All endpoints compile successfully
- [ ] **Type safety check** - All endpoints follow `Result<T, Error>` pattern
- [ ] **Permission check** - ACL system works correctly
- [ ] **Bulk operation check** - Handle partial failures gracefully
- [ ] **Test coverage** - All scenarios pass

## Success Criteria

- [ ] All 8 endpoints compile successfully
- [ ] All endpoints follow `Result<T, Error>` pattern
- [ ] Permission checks work correctly
- [ ] Bulk operations handle partial failures
- [ ] Tests pass for all scenarios
- [ ] Candid interface updated
- [ ] Documentation updated

## Future Enhancements

### Multi-Canister Support

- Add `canister_id` parameter for future multi-canister architecture
- Route calls to appropriate canister
- Handle cross-canister operations

### Advanced Features

- Batch processing for very large datasets
- Progress callbacks for long-running operations
- Rollback capabilities for failed operations
- Detailed cleanup reporting
