# Memory API Refactoring: Replace `ping` with `get_memory` and `get_memory_with_assets`

## Problem

The current memory API has redundant functionality and unclear separation of concerns:

1. **`memories_read`** - Returns full Memory object (metadata + asset info)
2. **`ping`** - Returns just presence status (true/false)
3. **Both functions duplicate the same search logic** - finding memories across accessible capsules

This creates:

- **Code duplication** - Same access control and search logic in multiple places
- **Inconsistent API** - Different patterns for similar operations
- **Maintenance burden** - Changes to access logic need to be made in multiple places
- **Unclear semantics** - `ping` doesn't clearly indicate what it's checking

## Current State Analysis

### `memories_read` Function

```rust
fn memories_read(memory_id: String) -> std::result::Result<types::Memory, Error> {
    // 1. Get accessible capsules for caller
    // 2. Search for memory across all accessible capsules
    // 3. Return the full Memory object (metadata + asset info)
}
```

### `ping` Function

```rust
pub fn ping(memory_ids: Vec<String>) -> std::result::Result<Vec<MemoryPresenceResult>, Error> {
    // 1. Get accessible capsules for caller
    // 2. Search for memory across all accessible capsules
    // 3. Return just true/false for existence
}
```

### Overlap Analysis

Both functions perform **identical operations**:

- ✅ Get caller's accessible capsules
- ✅ Search across all accessible capsules
- ✅ Check if memory exists in any capsule
- ✅ Apply same access control logic

**The only difference**: Return type (full object vs. presence status)

## Proposed Solution

### New API Design

Replace the current functions with a cleaner, more semantic API:

#### 1. `get_memory` - Metadata Only

```rust
/// Get memory metadata without asset data
/// Returns just the Memory struct with metadata, no asset content
fn get_memory(memory_id: String) -> std::result::Result<types::Memory, Error> {
    // Use existing memories_read logic
    // Return Memory object but without asset content
}
```

#### 2. `get_memory_with_assets` - Full Data

```rust
/// Get memory with full asset data
/// Returns Memory struct with complete asset information
fn get_memory_with_assets(memory_id: String) -> std::result::Result<types::Memory, Error> {
    // Use existing memories_read logic
    // Return Memory object with full asset content
}
```

#### 3. Remove `ping` Function

```rust
// ❌ Remove this function entirely
// pub fn ping(memory_ids: Vec<String>) -> std::result::Result<Vec<MemoryPresenceResult>, Error>
```

### Benefits

#### 1. **Clear Semantics**

- `get_memory` - Obviously returns memory data
- `get_memory_with_assets` - Obviously returns memory with assets
- No confusion about what `ping` does

#### 2. **DRY Principle**

- Single search logic in `memories_read_core`
- Both functions reuse the same access control
- No code duplication

#### 3. **Better Performance**

- `get_memory` can be optimized to not load asset data
- `get_memory_with_assets` loads everything when needed
- No unnecessary data transfer for presence checks

#### 4. **Consistent API**

- Both functions follow the same pattern
- Same error handling
- Same access control logic

## Implementation Plan

### Phase 1: Create New Functions

1. **Implement `get_memory`**

   ```rust
   fn get_memory(memory_id: String) -> std::result::Result<types::Memory, Error> {
       use crate::memories::{CanisterEnv, StoreAdapter};
       use crate::memories_core::memories_read_core;

       let env = CanisterEnv;
       let store = StoreAdapter;

       // Get full memory but strip asset data
       let mut memory = memories_read_core(&env, &store, memory_id)?;

       // Clear asset data to return metadata only
       memory.asset = None; // or whatever field contains asset data

       Ok(memory)
   }
   ```

2. **Implement `get_memory_with_assets`**
   ```rust
   fn get_memory_with_assets(memory_id: String) -> std::result::Result<types::Memory, Error> {
       // This is just an alias for the current memories_read
       memories_read(memory_id)
   }
   ```

### Phase 2: Update Core Logic

1. **Modify `memories_read_core`** to support metadata-only mode
2. **Add asset loading control** to the core function
3. **Ensure consistent access control** across both functions

### Phase 3: Remove Old Functions

1. **Remove `ping` function** from `memories.rs`
2. **Remove `ping` function** from `lib.rs`
3. **Update any tests** that use `ping`

### Phase 4: Update Documentation

1. **Update API documentation**
2. **Update Candid interface** (if needed)
3. **Create migration guide** for frontend

## API Comparison

### Before (Current)

```rust
// Unclear what this returns
fn memories_read(memory_id: String) -> Result<Memory, Error>

// Unclear what this checks
fn ping(memory_ids: Vec<String>) -> Result<Vec<MemoryPresenceResult>, Error>
```

### After (Proposed)

```rust
// Clear: returns memory metadata only
fn get_memory(memory_id: String) -> Result<Memory, Error>

// Clear: returns memory with full asset data
fn get_memory_with_assets(memory_id: String) -> Result<Memory, Error>
```

## Frontend Impact

### Current Frontend Usage

```typescript
// Current: unclear what this returns
const memory = await actor.memories_read(memoryId);

// Current: unclear what this checks
const presence = await actor.ping([memoryId]);
```

### New Frontend Usage

```typescript
// New: clear intent - get metadata only
const memory = await actor.get_memory(memoryId);

// New: clear intent - get full data with assets
const memoryWithAssets = await actor.get_memory_with_assets(memoryId);

// New: check existence by trying to get metadata
const exists = (await actor.get_memory(memoryId)).isOk();
```

## Testing Strategy

### Unit Tests

1. **Test `get_memory`** returns metadata without asset data
2. **Test `get_memory_with_assets`** returns full memory data
3. **Test access control** works consistently across both functions
4. **Test error handling** for non-existent memories

### Integration Tests

1. **Test with PocketIC** to ensure functions work in canister context
2. **Test with real memory data** to verify asset loading behavior
3. **Test performance** difference between metadata-only and full data

## Migration Strategy

### Backward Compatibility

- **Keep `memories_read`** as an alias to `get_memory_with_assets` during transition
- **Deprecate `ping`** with clear migration path
- **Provide migration guide** for frontend developers

### Gradual Migration

1. **Add new functions** alongside existing ones
2. **Update frontend** to use new functions
3. **Remove old functions** after migration is complete

## Performance Considerations

### Memory Usage

- **`get_memory`** - Lower memory usage (no asset data)
- **`get_memory_with_assets`** - Higher memory usage (full data)

### Network Transfer

- **`get_memory`** - Smaller response size
- **`get_memory_with_assets`** - Larger response size

### Use Cases

- **`get_memory`** - For listing, searching, metadata operations
- **`get_memory_with_assets`** - For viewing, downloading, full operations

## Acceptance Criteria

- [ ] `get_memory` function implemented and tested
- [ ] `get_memory_with_assets` function implemented and tested
- [ ] `ping` function removed from codebase
- [ ] All tests pass with new functions
- [ ] Frontend migration guide created
- [ ] API documentation updated
- [ ] Performance benchmarks show improvement
- [ ] No breaking changes for existing functionality

## Priority

**Medium** - This improves API design and reduces code duplication, but doesn't fix critical bugs.

## Estimated Effort

**Small** - Most of the logic already exists, just needs to be reorganized and optimized.

## Dependencies

- None - This is a pure refactoring that doesn't depend on external changes.

## Related Issues

- [Memory Creation API Design Issue](./memory-creation-api-design-issue.md)
- [Backend Memories Auto-Capsule Creation](./backend-memories-auto-capsule-creation.md)

## Notes

This refactoring aligns with modern API design principles:

- **Clear function names** that indicate their purpose
- **Consistent patterns** across similar functions
- **Separation of concerns** between metadata and asset data
- **DRY principle** to reduce code duplication

The new API will be more intuitive for developers and easier to maintain for the team.
