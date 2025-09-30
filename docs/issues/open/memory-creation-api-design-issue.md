# Memory Creation API Design Issue

## Summary

The `memories_create` function uses a poor API design with three separate `Option<T>` parameters instead of a proper union type, leading to confusing validation logic and unclear mutual exclusivity.

## Status

🔍 **OPEN** - API design improvement needed

## Problem Description

### Current API Design

```rust
pub fn memories_create_core(
    env: &dyn Env,
    store: &mut dyn Store,
    capsule_id: String,
    bytes: Option<Vec<u8>>,                    // ❌ Confusing
    blob_ref: Option<BlobRef>,                 // ❌ Confusing
    external_location: Option<StorageEdgeBlobType>, // ❌ Confusing
    external_storage_key: Option<String>,
    external_url: Option<String>,
    external_size: Option<u64>,
    external_hash: Option<Vec<u8>>,
    asset_metadata: AssetMetadata,
    idem: String,
) -> std::result::Result<MemoryId, Error>
```

### Current Validation Logic

```rust
// ❌ Manual counting and validation
let asset_count = bytes.is_some() as u8 + blob_ref.is_some() as u8 + external_location.is_some() as u8;
if asset_count != 1 {
    return Err(Error::InvalidArgument(
        "Exactly one asset type must be provided".to_string(),
    ));
}
```

## Issues with Current Design

### 1. **Unclear Mutual Exclusivity**

- Parameter names don't indicate they're mutually exclusive
- Users might think they can provide multiple asset types
- No compile-time safety

### 2. **Confusing Validation**

- Manual counting of `Option` values
- Unnecessary `as u8` casting
- Runtime validation instead of type safety

### 3. **Poor API Clarity**

- Three separate parameters for what should be one concept
- No indication that these represent different storage strategies
- Hard to understand the business rule

### 4. **Maintenance Burden**

- Validation logic scattered throughout codebase
- Easy to forget the mutual exclusivity rule
- Error-prone when adding new asset types

## Proposed Solution

### Union Type Design

```rust
// ✅ Better API design
pub enum AssetData {
    Inline {
        bytes: Vec<u8>,
    },
    BlobRef {
        blob_ref: BlobRef,
    },
    External {
        location: StorageEdgeBlobType,
        storage_key: String,
        url: Option<String>,
        size: u64,
        hash: Option<Vec<u8>>,
    },
}

pub fn memories_create_core(
    env: &dyn Env,
    store: &mut dyn Store,
    capsule_id: String,
    asset_data: AssetData,           // ✅ Single parameter
    asset_metadata: AssetMetadata,
    idem: String,
) -> std::result::Result<MemoryId, Error>
```

### Benefits of Union Type

1. **Type Safety**: Compile-time guarantee of exactly one asset type
2. **Clear Intent**: API clearly shows mutual exclusivity
3. **No Validation**: No need for runtime counting/validation
4. **Extensible**: Easy to add new asset types
5. **Self-Documenting**: Code clearly shows the business rule

## Impact

### Files to Update

- `src/backend/src/memories_core.rs` - Core function signature
- `src/backend/src/memories.rs` - Canister wrapper
- `src/backend/src/lib.rs` - Public canister function
- `src/backend/src/types.rs` - Add `AssetData` enum
- All test files - Update test calls
- Shell test scripts - Update Candid calls

### Breaking Changes

- **Public API**: Canister function signature changes
- **Candid Interface**: `.did` file needs updates
- **Client Code**: All callers need updates

## Implementation Plan

### Phase 1: Add Union Type

1. Define `AssetData` enum in `types.rs`
2. Add helper methods for conversion
3. Update internal functions to use union type

### Phase 2: Update Core Functions

1. Modify `memories_create_core` signature
2. Remove manual validation logic
3. Use pattern matching instead of counting

### Phase 3: Update Canister Layer

1. Update `memories_create` wrapper
2. Add conversion from old parameters to union type
3. Maintain backward compatibility during transition

### Phase 4: Update Tests

1. Update unit tests to use new API
2. Update integration tests
3. Update shell test scripts

### Phase 5: Cleanup

1. Remove old parameter-based functions
2. Update documentation
3. Update Candid interface

## Alternative: Gradual Migration

If breaking changes are too disruptive, consider a gradual migration:

1. **Add new union-based function** alongside old one
2. **Deprecate old function** with warnings
3. **Migrate callers** to new function
4. **Remove old function** in next major version

## Priority

**Medium** - This is an API design improvement that would make the code more maintainable and type-safe, but it's not blocking current functionality.

## Related Issues

- None currently, but this could be part of a larger API cleanup effort

---

_This issue was identified during the decoupled architecture refactoring. While the architecture separation was successful, the underlying API design issues remain._

