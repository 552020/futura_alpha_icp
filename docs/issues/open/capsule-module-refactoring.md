# Capsule Module Refactoring

**Status**: Open  
**Priority**: Medium  
**Created**: 2024-10-10  
**Assignee**: TBD

## Problem Statement

Currently, the capsule functionality is implemented as a single monolithic file `src/backend/src/capsule.rs` (650 lines). This creates several issues:

1. **Maintainability**: Large single file is difficult to navigate and maintain
2. **Modularity**: All capsule-related functionality is mixed together
3. **Extensibility**: Adding new features requires modifying the large file
4. **Code Organization**: No clear separation of concerns

## Current Structure

```
src/backend/src/
├── capsule.rs (650 lines - monolithic)
├── capsule_acl.rs
├── capsule_store/
└── capsule_mess/ (previous work preserved)
```

## Proposed Solution

Refactor to a proper module structure with both a main file and submodules:

```
src/backend/src/
├── capsule.rs (main module file - core functionality)
└── capsule/
    ├── access.rs (access control system)
    ├── types.rs (capsule-specific types)
    ├── api.rs (API endpoints)
    ├── storage.rs (storage management)
    └── time.rs (time utilities)
```

## Benefits

1. **Better Organization**: Clear separation of concerns
2. **Easier Maintenance**: Smaller, focused files
3. **Improved Extensibility**: Easy to add new modules
4. **Better Testing**: Each module can be tested independently
5. **Code Reusability**: Modules can be imported individually

## Implementation Plan

### Phase 1: Create Module Structure

- [ ] Create `src/backend/src/capsule/` directory
- [ ] Move existing `capsule.rs` to `capsule/core.rs` as reference
- [ ] Create new `capsule.rs` as main module file
- [ ] Update `lib.rs` to use new module structure

### Phase 2: Extract Core Functionality

- [ ] Keep essential capsule functions in main `capsule.rs`
- [ ] Extract access control logic to `capsule/access.rs`
- [ ] Extract type definitions to `capsule/types.rs`
- [ ] Extract API functions to `capsule/api.rs`

### Phase 3: Extract Supporting Modules

- [ ] Move storage utilities to `capsule/storage.rs`
- [ ] Move time utilities to `capsule/time.rs`
- [ ] Update imports and dependencies

### Phase 4: Cleanup and Testing

- [ ] Remove redundant files
- [ ] Update all imports across the codebase
- [ ] Run comprehensive tests
- [ ] Update documentation

## Technical Considerations

### Module Declaration

```rust
// In lib.rs
mod capsule {
    pub mod access;
    pub mod types;
    pub mod api;
    pub mod storage;
    pub mod time;

    // Re-export for backward compatibility
    pub use access::*;
    pub use types::*;
    pub use api::*;
}
```

### Backward Compatibility

- Maintain all existing public APIs
- Use re-exports to avoid breaking changes
- Gradual migration approach

## Files to Refactor

### Main Files

- `src/backend/src/capsule.rs` → Split into multiple modules

### Dependencies to Update

- `src/backend/src/lib.rs` (module declarations)
- `src/backend/src/gallery.rs` (imports)
- `src/backend/src/user.rs` (imports)
- Any other files importing capsule functions

## Success Criteria

- [ ] All existing functionality preserved
- [ ] Code compiles without errors
- [ ] All tests pass
- [ ] No breaking changes to public APIs
- [ ] Improved code organization and maintainability

## Risks and Mitigation

### Risk: Breaking Changes

**Mitigation**: Use re-exports and gradual migration

### Risk: Import Errors

**Mitigation**: Update all imports systematically

### Risk: Lost Functionality

**Mitigation**: Comprehensive testing and code review

## Related Work

- Previous work preserved in `capsule_mess/` directory
- Access control system design in `gallery-type-refactor-implementation.md`
- Existing `capsule_store/` module structure as reference

## Notes

- This refactoring should be done incrementally
- Preserve all existing work in backup directories
- Consider this as preparation for implementing the access control system
- Follow Rust module best practices (no `mod.rs`, use `module_name.rs` pattern)
