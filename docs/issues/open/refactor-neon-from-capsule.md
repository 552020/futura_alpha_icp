# Remove `bound_to_neon` from Capsule struct

## Problem

The `Capsule` struct still has a `bound_to_neon: bool` field, but this functionality has been replaced by `database_storage_edges: Vec<StorageEdgeDatabaseType>` in the `Memory` struct.

## Current State

- `Capsule` struct has `bound_to_neon: bool` field
- `Memory` struct has `database_storage_edges: Vec<StorageEdgeDatabaseType>` field
- This creates redundancy and inconsistency

## Solution

1. Remove `bound_to_neon` field from `Capsule` struct
2. Update all `Capsule` initializations to remove this field
3. Update any code that references `capsule.bound_to_neon`
4. Use `database_storage_edges` in `Memory` struct instead

## Files to Update

- `src/backend/src/types.rs` - Remove field from `Capsule` struct
- `src/backend/src/canister_factory/auth.rs` - Remove from initialization
- `src/backend/src/capsule_store/stable.rs` - Remove from initialization
- `src/backend/src/capsule_store/store.rs` - Remove from initialization
- `src/backend/src/capsule_store/integration_tests.rs` - Remove from initialization
- `src/backend/src/gallery.rs` - Remove from initialization
- Any other files that reference `bound_to_neon`

## Benefits

- Cleaner architecture
- Single source of truth for storage edges
- Consistent with the new multi-asset architecture
