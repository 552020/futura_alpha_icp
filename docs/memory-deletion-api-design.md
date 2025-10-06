# Memory Deletion API Design

## Current State

We currently have these memory deletion endpoints:

1. **`memories_delete(memory_id)`** - Delete single memory + all assets
2. **`memories_delete_bulk(capsule_id, memory_ids)`** - Delete multiple memories + all assets
3. **`memories_delete_all(capsule_id)`** - Delete all memories in capsule + all assets

## Current Behavior

All current deletion endpoints **automatically delete assets**:

- **Inline assets**: Removed with memory (stored in memory struct)
- **Internal blob assets**: Deleted from ICP blob store
- **External blob assets**: Deleted from external storage

## New Requirements

We need **two deletion modes**:

1. **Full Deletion** (current behavior): Delete memory + all assets
2. **Metadata-Only Deletion**: Delete memory metadata but preserve assets

### Use Cases

**Full Deletion** (current):

- User wants to completely remove a memory and all its data
- Cleanup unused assets to free storage space
- GDPR "right to be forgotten" requests

**Metadata-Only Deletion** (new):

- Move memory metadata from ICP to web2 database
- Keep assets in ICP blob store for performance
- Selective data migration scenarios
- Preserve assets for other systems while removing ICP metadata

## Proposed API Design

### Option 1: Add `delete_assets` Parameter

```rust
#[ic_cdk::update]
fn memories_delete(
    memory_id: String,
    delete_assets: bool, // true = delete assets (current behavior), false = keep assets
) -> std::result::Result<(), Error>

#[ic_cdk::update]
fn memories_delete_bulk(
    capsule_id: String,
    memory_ids: Vec<String>,
    delete_assets: bool, // true = delete assets, false = keep assets
) -> Result<BulkDeleteResult, Error>
```

### Option 2: Separate Endpoints

```rust
// Current endpoints (unchanged)
#[ic_cdk::update]
fn memories_delete(memory_id: String) -> std::result::Result<(), Error>

#[ic_cdk::update]
fn memories_delete_bulk(capsule_id: String, memory_ids: Vec<String>) -> Result<BulkDeleteResult, Error>

// New endpoints for metadata-only deletion
#[ic_cdk::update]
fn memories_delete_metadata_only(memory_id: String) -> std::result::Result<(), Error>

#[ic_cdk::update]
fn memories_delete_bulk_metadata_only(capsule_id: String, memory_ids: Vec<String>) -> Result<BulkDeleteResult, Error>
```

### Option 3: Enum-Based Approach

```rust
#[derive(CandidType, Deserialize)]
pub enum DeletionMode {
    Full,        // Delete memory + all assets (current behavior)
    MetadataOnly, // Delete memory metadata only, keep assets
}

#[ic_cdk::update]
fn memories_delete(
    memory_id: String,
    mode: DeletionMode,
) -> std::result::Result<(), Error>

#[ic_cdk::update]
fn memories_delete_bulk(
    capsule_id: String,
    memory_ids: Vec<String>,
    mode: DeletionMode,
) -> Result<BulkDeleteResult, Error>
```

## Recommendation

**Option 1 (boolean parameter)** is the cleanest approach:

### Pros:

- Simple boolean parameter
- Backward compatible (default `delete_assets: true`)
- Clear intent
- Minimal API surface

### Cons:

- Parameter name could be confusing (delete_assets vs keep_assets)

### Implementation:

```rust
#[ic_cdk::update]
fn memories_delete(
    memory_id: String,
    delete_assets: bool, // true = delete assets (default), false = keep assets
) -> std::result::Result<(), Error> {
    use crate::memories::core::memories_delete_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    memories_delete_core(&env, &mut store, memory_id, delete_assets)
}
```

## Core Logic Changes

The `memories_delete_core` function would need to be updated:

```rust
pub fn memories_delete_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_id: MemoryId,
    delete_assets: bool, // New parameter
) -> std::result::Result<(), Error> {
    // ... existing ACL checks ...

    if delete_assets {
        // Current behavior: Clean up assets before deleting the memory
        cleanup_memory_assets(&memory)?;
    }
    // If delete_assets is false, skip asset cleanup

    // Delete the memory (always happens)
    store.delete_memory(&capsule_id, &memory_id)?;

    // ... existing post-write assertions ...
}
```

## Backward Compatibility

To maintain backward compatibility, we can:

1. **Keep existing endpoints unchanged** (they default to `delete_assets: true`)
2. **Add new overloaded endpoints** with the boolean parameter
3. **Or update existing endpoints** with optional parameter (defaulting to `true`)

## Testing Strategy

1. **Test current behavior** (delete_assets: true) - should work exactly as before
2. **Test new behavior** (delete_assets: false) - memory deleted, assets preserved
3. **Test asset preservation** - verify blobs still exist after metadata-only deletion
4. **Test ACL permissions** - ensure deletion permissions work for both modes

## Migration Path

1. **Phase 1**: Implement new parameter in core functions
2. **Phase 2**: Update API endpoints with backward compatibility
3. **Phase 3**: Add comprehensive tests
4. **Phase 4**: Update documentation and examples
