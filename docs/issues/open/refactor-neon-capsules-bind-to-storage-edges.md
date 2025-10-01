# Rename `capsules_bind_neon` to Reflect New Database Storage Edges Architecture

## Problem

The function `capsules_bind_neon` has an outdated name that doesn't reflect the current architecture:

1. **Outdated naming** - References "neon" specifically when the system now supports multiple database storage options
2. **Misleading scope** - Name suggests it only works with capsules, but it handles capsules, galleries, AND memories
3. **Architecture mismatch** - The underlying data structure has evolved from `bound_to_neon: bool` to `database_storage_edges: Vec<StorageEdgeDatabaseType>`

## Current State

### Function Name

```rust
fn capsules_bind_neon(
    resource_type: types::ResourceType,
    resource_id: String,
    bind: bool,
) -> std::result::Result<(), Error>
```

### Current Data Structure

```rust
// Memory type now uses:
pub struct MemoryMetadata {
    // ... other fields ...
    pub database_storage_edges: Vec<StorageEdgeDatabaseType>,
}

pub enum StorageEdgeDatabaseType {
    Icp,  // ICP canister storage
    Neon, // Neon database
}
```

### What the Function Actually Does

- Handles `ResourceType::Capsule`, `ResourceType::Gallery`, and `ResourceType::Memory`
- Manages `database_storage_edges` field (not just a boolean "bound to neon")
- Supports multiple storage options (ICP, Neon, etc.)

## Proposed Solution

### Option 1: Generic Resource Function

```rust
fn resources_set_database_storage(
    resource_type: types::ResourceType,
    resource_id: String,
    storage_edges: Vec<StorageEdgeDatabaseType>,
) -> std::result::Result<(), Error>
```

### Option 2: Specific Storage Management

```rust
fn resources_manage_storage_edges(
    resource_type: types::ResourceType,
    resource_id: String,
    add_storage: Option<StorageEdgeDatabaseType>,
    remove_storage: Option<StorageEdgeDatabaseType>,
) -> std::result::Result<(), Error>
```

### Option 3: Simple and Clear

```rust
fn resources_set_hosting_preferences(
    resource_type: types::ResourceType,
    resource_id: String,
    database_hosting: Vec<StorageEdgeDatabaseType>,
) -> std::result::Result<(), Error>
```

## Recommended Approach

**Option 3** is recommended because:

- ✅ **Clear intent** - "hosting preferences" is intuitive
- ✅ **Matches frontend terminology** - Aligns with user-facing concepts
- ✅ **Future-proof** - Can easily extend to blob hosting preferences
- ✅ **Consistent** - Follows the established pattern of resource functions

## Implementation Plan

### Phase 1: Function Rename

1. **Rename function** from `capsules_bind_neon` to `resources_set_hosting_preferences`
2. **Update function signature** to accept `Vec<StorageEdgeDatabaseType>` instead of `bool`
3. **Update both lib.rs and capsule.rs** implementations

### Phase 2: Update Function Logic

1. **Modify the function** to work with the new `database_storage_edges` field
2. **Update ResourceType handling** to use the new storage edges system
3. **Ensure backward compatibility** during transition

### Phase 3: Update Documentation

1. **Update API documentation**
2. **Update Candid interface** (if needed)
3. **Create migration guide** for frontend

## Function Signature Changes

### Before (Current)

```rust
fn capsules_bind_neon(
    resource_type: types::ResourceType,
    resource_id: String,
    bind: bool,  // ❌ Boolean - too restrictive
) -> std::result::Result<(), Error>
```

### After (Proposed)

```rust
fn resources_set_hosting_preferences(
    resource_type: types::ResourceType,
    resource_id: String,
    database_hosting: Vec<StorageEdgeDatabaseType>,  // ✅ Flexible - supports multiple options
) -> std::result::Result<(), Error>
```

## Frontend Impact

### Current Frontend Usage

```typescript
// Current: unclear what this does
await actor.capsules_bind_neon(ResourceType.Memory, memoryId, true);
```

### New Frontend Usage

```typescript
// New: clear intent - set hosting preferences
await actor.resources_set_hosting_preferences(ResourceType.Memory, memoryId, [
  StorageEdgeDatabaseType.Neon,
  StorageEdgeDatabaseType.Icp,
]);
```

## Benefits

### 1. **Clear Semantics**

- Function name clearly indicates it manages hosting preferences
- No confusion about what "bind neon" means
- Aligns with user-facing terminology

### 2. **Architecture Alignment**

- Matches the current `database_storage_edges` data structure
- Supports multiple storage options, not just boolean binding
- Future-proof for additional storage providers

### 3. **Better API Design**

- Generic function name that works for all resource types
- Flexible parameter that supports multiple storage options
- Consistent with other resource management functions

### 4. **Improved Maintainability**

- Function name reflects actual functionality
- Easier to understand and maintain
- Better documentation and onboarding

## Migration Strategy

### Backward Compatibility

- **Keep old function** as deprecated alias during transition
- **Add deprecation warning** to guide developers to new function
- **Provide migration guide** with examples

### Gradual Migration

1. **Add new function** alongside existing one
2. **Update frontend** to use new function
3. **Remove old function** after migration is complete

## Testing Strategy

### Unit Tests

1. **Test new function** with different resource types
2. **Test multiple storage edges** (ICP + Neon combinations)
3. **Test error handling** for invalid resource IDs

### Integration Tests

1. **Test with PocketIC** to ensure function works in canister context
2. **Test with real data** to verify storage edges are set correctly
3. **Test frontend integration** with new function signature

## Related Issues

- [Memory API Refactoring: Replace `ping` with `get_memory` and `get_memory_with_assets`](./memory-api-refactoring-ping-to-get-functions.md)
- [Hosting Preferences: Non-Exclusive Storage Options](./hosting-preferences-non-exclusive-storage.md)

## Acceptance Criteria

- [ ] Function renamed to `resources_set_hosting_preferences`
- [ ] Function signature updated to use `Vec<StorageEdgeDatabaseType>`
- [ ] Function logic updated to work with `database_storage_edges` field
- [ ] All tests pass with new function
- [ ] Frontend migration guide created
- [ ] API documentation updated
- [ ] Backward compatibility maintained during transition
- [ ] Old function deprecated and eventually removed

## Priority

**Medium** - This improves API clarity and architecture alignment, but doesn't fix critical bugs.

## Estimated Effort

**Small** - Function rename and signature update, but requires careful testing to ensure no breaking changes.

## Dependencies

- None - This is a pure refactoring that doesn't depend on external changes.

## Notes

This refactoring aligns the function name with:

- **Current architecture** - Database storage edges instead of boolean binding
- **User terminology** - "Hosting preferences" is more intuitive than "bind neon"
- **API consistency** - Generic resource functions follow established patterns
- **Future extensibility** - Easy to add new storage providers

The new name makes the API more self-documenting and easier to understand for both developers and users.
