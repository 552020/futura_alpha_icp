# Rename Binding to Neon Functions

## Overview

The current function names `capsules_bind_neon` and related functions use "Neon" in their naming, but this should be updated to reflect the evolution of our Web2 backend architecture.

## Current Functions

- `capsules_bind_neon(resource_type, resource_id, bound)`
- `galleries_bind_neon(gallery_id, bound)`
- `memories_bind_neon(memory_id, bound)`

## Problem

The naming "bind to neon" is outdated and doesn't accurately reflect:

1. **Multi-backend Architecture**: We now support multiple storage backends (S3, ICP, Vercel Blob, etc.)
2. **Centralized Registry**: The "binding" is actually about registering assets in a centralized storage edge registry
3. **Generic Storage Management**: The functions are about storage preference management, not just Neon

## Proposed Solutions

### Option 1: Generic Storage Binding

```rust
// Rename to generic storage binding
capsules_bind_storage(resource_type, resource_id, bound)
galleries_bind_storage(gallery_id, bound)
memories_bind_storage(memory_id, bound)
```

### Option 2: Storage Edge Registration

```rust
// Rename to reflect storage edge registry concept
capsules_register_storage_edge(resource_type, resource_id, bound)
galleries_register_storage_edge(gallery_id, bound)
memories_register_storage_edge(memory_id, bound)
```

### Option 3: Storage Preference Management

```rust
// Rename to reflect storage preference concept
capsules_set_storage_preference(resource_type, resource_id, bound)
galleries_set_storage_preference(gallery_id, bound)
memories_set_storage_preference(memory_id, bound)
```

## Recommended Approach

**Option 2: Storage Edge Registration** is recommended because:

1. **Accurate Terminology**: Reflects the actual architecture with storage edges
2. **Clear Purpose**: Makes it obvious this is about registering where assets are stored
3. **Future-Proof**: Works with any storage backend, not just Neon
4. **Consistent with Schema**: Matches the `storageEdges` table in our Web2 schema

## Implementation Plan

### Phase 1: Add New Functions

```rust
// Add new functions with better names
#[ic_cdk::update]
fn capsules_register_storage_edge(
    resource_type: ResourceType,
    resource_id: String,
    bound: bool
) -> Result<(), Error> {
    // Implementation
}

#[ic_cdk::update]
fn galleries_register_storage_edge(
    gallery_id: String,
    bound: bool
) -> Result<(), Error> {
    // Implementation
}

#[ic_cdk::update]
fn memories_register_storage_edge(
    memory_id: String,
    bound: bool
) -> Result<(), Error> {
    // Implementation
}
```

### Phase 2: Deprecate Old Functions

```rust
// Keep old functions for backward compatibility but mark as deprecated
#[deprecated(note = "Use capsules_register_storage_edge instead")]
#[ic_cdk::update]
fn capsules_bind_neon(
    resource_type: ResourceType,
    resource_id: String,
    bound: bool
) -> Result<(), Error> {
    // Delegate to new function
    capsules_register_storage_edge(resource_type, resource_id, bound)
}
```

### Phase 3: Update Tests

- Update all test scripts to use new function names
- Update documentation and examples
- Update frontend code

### Phase 4: Remove Old Functions

- After sufficient migration period, remove deprecated functions

## Backward Compatibility

- Keep old function names as deprecated aliases
- Ensure old functions delegate to new implementations
- Provide migration guide for developers

## Testing

Update test files:

- `test_capsules_bind_neon.sh` → `test_capsules_register_storage_edge.sh`
- Update all test function names and calls
- Ensure all tests pass with new function names

## Priority

**Medium Priority** - This is a naming/architecture improvement that doesn't affect functionality but improves code clarity and maintainability.

## Related Files

- `tests/backend/general/test_capsules_bind_neon.sh`
- `src/backend/src/capsule_store/stable.rs`
- `src/nextjs/src/db/schema.ts` (storageEdges table)
- All frontend code that calls these functions

---

_Issue created: January 2025_
_Status: Open_
_Priority: Medium_
