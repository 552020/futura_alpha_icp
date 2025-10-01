# Backend Type Organization Plan

## Current Problem

- We have duplicate types scattered across multiple files
- `unified_types.rs` has the canonical types (following tech lead spec)
- `types.rs` has duplicates and is messy
- `upload/types.rs` and `memories/types.rs` exist but may have overlaps

## Clear Plan

### STEP 1: DEFINE THE CANONICAL STRUCTURE

1.1 [x] **`unified_types.rs`** = THE CANONICAL TYPES (tech lead's Option S spec)
1.2 [x] **`types.rs`** = Re-exports + core types that aren't in unified_types
1.3 [x] **`upload/types.rs`** = Upload-specific types (SessionId, BlobId, etc.) + imports unified_types
1.4 [x] **`memories/types.rs`** = Memory-specific types + imports unified_types

### STEP 2: CLEAN UP `types.rs`

2.1 [x] Remove ALL duplicate types that are in `unified_types.rs`
2.2 [x] Keep only:
2.2.1 [x] Type aliases (CapsuleId, MemoryId)
2.2.2 [x] Core types not in unified_types (Error, User, etc.)
2.2.3 [x] Re-export statements
2.3 [x] Remove duplicate AssetMetadata, UploadFinishResult, StorageEdge types

### STEP 3: UPDATE IMPORTS

3.1 [x] Make sure `upload/types.rs` imports from `unified_types.rs`
3.2 [x] Make sure `memories/types.rs` imports from `unified_types.rs`
3.3 [ ] Update all backend functions to use the canonical types

### STEP 4: REGENERATE CANDID

4.1 [ ] Deploy with `dfx deploy` to regenerate `.did` file
4.2 [ ] Update frontend to use generated types

## Execution Order

1. ✅ Keep `unified_types.rs` (DON'T DELETE IT)
2. 🔄 Clean up `types.rs` (remove duplicates)
3. 🔄 Update imports in other files
4. 🔄 Update backend functions
5. 🔄 Deploy and regenerate Candid

**The key insight: `unified_types.rs` IS our canonical schema. Everything else should import from it.**

## Status

### ✅ COMPLETED TASKS

1. [x] Created unified_types.rs with canonical types
2. [x] Created memories/types.rs
3. [x] Updated upload/types.rs to import unified types
4. [x] Clean up types.rs (remove duplicates)

### 🔄 IN PROGRESS

5. [ ] Update backend functions to use unified types

### ⏳ PENDING TASKS

6. [ ] Deploy and regenerate Candid interface
7. [ ] Update frontend to use generated types
