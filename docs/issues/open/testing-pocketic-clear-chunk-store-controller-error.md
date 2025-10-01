# PocketIC clear_chunk_store Controller Permission Error

## Issue Summary

PocketIC integration tests are failing with a controller permission error when the canister attempts to call the ICP system method `clear_chunk_store`. This prevents all memory management integration tests from running, despite the core business logic working correctly in unit tests.

## Error Details

```
Failed to submit ingress message: UserError {
    code: CanisterInvalidController,
    description: "Only controllers of canister lxzze-o7777-77777-aaaaa-cai can call ic00 method clear_chunk_store"
}
```

## Root Cause Analysis

The canister is attempting to call the ICP system method `clear_chunk_store` which requires special controller permissions. This call is not present in our codebase directly, suggesting it originates from:

1. **Dependencies**: `ic-stable-structures = "0.6"` or `ic-cdk = "0.18"`
2. **Canister initialization**: Some cleanup or initialization code triggered during canister startup
3. **Upload service integration**: Related to chunked upload functionality

## Files Involved

### Test Files

- **`src/backend/tests/memories_pocket_ic.rs`** - Main PocketIC integration test file
  - Contains `test_create_and_read_memory_happy_path()` and other memory management tests
  - All tests fail with the same controller permission error
  - Test setup: `pic.install_canister(canister_id, wasm, vec![], Some(controller))`

### Core Implementation Files

- **`src/backend/src/lib.rs`** - Main canister entry point

  - Contains `memories_create`, `memories_read`, `memories_update`, `memories_delete` functions
  - Has `pre_upgrade` and `post_upgrade` hooks that call `with_capsule_store*` functions
  - Lines 699-774: Upgrade functions that might trigger the system call

- **`src/backend/src/memories.rs`** - Canister-facing memory functions

  - Contains `CanisterEnv` and `StoreAdapter` implementations
  - Thin wrappers that call `memories_core` functions
  - `StoreAdapter` calls `with_capsule_store` and `with_capsule_store_mut`

- **`src/backend/src/memories_core.rs`** - Pure business logic
  - Contains the decoupled core functions
  - No direct ICP dependencies (as intended)

### Dependencies

- **`Cargo.toml`** - Key dependencies that might trigger the system call:
  ```toml
  ic-cdk = "0.18"
  ic-stable-structures = "0.6"
  ```

## Test Setup Details

```rust
let mut pic = PocketIc::new();
let wasm = load_backend_wasm();
let controller = Principal::from_slice(&[1; 29]);
let canister_id = pic.create_canister();
pic.add_cycles(canister_id, 2_000_000_000_000);
pic.install_canister(canister_id, wasm, vec![], Some(controller));
```

## Impact

- **All PocketIC integration tests fail** with the same error
- **Unit tests pass** - core business logic is working correctly
- **Decoupled architecture is sound** - the issue is in the integration layer, not the core logic

## Investigation Attempts

### 1. Candid Type Issues (✅ RESOLVED)

- Fixed parameter type mismatches in test calls
- Updated test to send correct types to `memories_create`

### 2. Controller Setup (✅ ATTEMPTED)

- Tried installing canister with `Some(controller)` vs `None`
- Both approaches fail with the same error

### 3. Dependency Analysis (🔍 IN PROGRESS)

- Searched codebase for `clear_chunk_store` calls - none found
- Suspected `ic-stable-structures` or `ic-cdk` as the source
- Upgrade functions call `with_capsule_store*` which might trigger the system call

## Potential Solutions

### Option 1: Investigate System Call Source

- Add debug logging to identify exactly what triggers `clear_chunk_store`
- Check if it's related to stable memory initialization or cleanup
- Review `ic-stable-structures` documentation for controller requirements

### Option 2: Alternative Test Setup

- Research proper PocketIC controller permission setup
- Check if there's a way to grant the test user the necessary permissions
- Look into PocketIC-specific configuration for system method access

### Option 3: Focus on Unit Tests

- Since core logic is working (unit tests pass), focus on unit test coverage
- Use unit tests to validate the decoupled architecture
- Defer PocketIC integration tests until the controller issue is resolved

### Option 4: Dependency Updates

- Check if updating `ic-stable-structures` or `ic-cdk` versions resolves the issue
- Review changelogs for controller permission changes

## Current Status

- **Core functionality**: ✅ Working (unit tests pass)
- **Decoupled architecture**: ✅ Implemented correctly
- **PocketIC integration**: ❌ Blocked by controller permissions
- **Candid export**: ✅ Working (compilation successful)

## Next Steps

1. **Immediate**: Determine if this is a known issue with current PocketIC/ICP versions
2. **Short-term**: Focus on unit test coverage while investigating the controller issue
3. **Long-term**: Resolve PocketIC integration for end-to-end testing

## Related Issues

- [Candid Export Compilation Error](./candid-export-compilation-error.md) - ✅ RESOLVED
- [Production Integration of Decoupled Architecture](./production-integration-decoupled-architecture.md) - Core work completed, integration testing blocked

---

**Priority**: Medium - Core functionality works, integration testing blocked
**Assignee**: Senior developer
**Labels**: `pocketic`, `integration-tests`, `controller-permissions`, `icp-system-methods`

