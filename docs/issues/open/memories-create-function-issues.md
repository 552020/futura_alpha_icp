# Memories Create Function Issues

## Problem Summary

The `memories_create` function is experiencing multiple issues across different testing environments:

1. **JavaScript Tests**: Certificate verification errors
2. **Shell Tests**: Candid parsing errors
3. **Unit Tests**: `CheckSequenceNotMatch` errors
4. **Integration Tests**: Type mismatches (`text` vs `principal`)

## Current Status

### ✅ Working Functions

- `capsules_create()` - Works perfectly in JavaScript and shell
- `uploads_begin()`, `uploads_put_chunk()`, `uploads_finish()` - Work perfectly in JavaScript
- `capsules_read_basic()` - Works perfectly in JavaScript and shell

### ❌ Failing Functions

- `memories_create()` - Fails in ALL testing environments

## Detailed Analysis

### 1. JavaScript Test Failures

**Error**: `Certificate verification error: "Signature verification failed: TrustError: Certificate verification error: "Invalid signature"`

**Root Cause**: Initially thought to be certificate verification, but discovered to be a **Candid type mismatch**:

- Error: `type mismatch: type on the wire text, expect type principal`
- Somewhere in the `memories_create` call, we're passing a `text` where a `principal` is expected

**Evidence**:

- Other JavaScript update calls work fine (`capsules_create`, `uploads_*`)
- The issue is specific to `memories_create`
- There are existing memories in the capsule, proving `memories_create` WAS working before

### 2. Shell Test Failures

**Error**: `Candid parser error: Unrecognized token 'Semi' found at 510:511`

**Root Cause**: Candid syntax errors in the shell command arguments

- The `memories_create` function signature appears correct in the interface
- The issue is with the way we're constructing the Candid arguments

### 3. Unit Test Failures

**Error**: `CheckSequenceNotMatch`

**Root Cause**: Backend implementation issue

- The `memories_create_core` function is failing with sequence check errors
- This suggests a problem with the internal implementation, not the interface

**Affected Tests**:

- `test_memories_create_core_inline_asset`
- `test_memories_create_core_idempotency`

### 4. Integration Test Failures

**Error**: `type mismatch: type on the wire text, expect type principal`

**Root Cause**: Interface mismatch

- The backend expects a `principal` type but we're passing a `text`
- This suggests the interface has changed or there's a version mismatch

## Technical Details

### Function Signature

```candid
memories_create : (
    text,                    // capsule_id
    opt blob,               // inline_bytes
    opt BlobRef,            // blob_ref
    opt StorageEdgeBlobType, // storage_type
    opt text,               // external_storage_key
    opt text,               // external_url
    opt nat64,              // external_size
    opt blob,               // external_hash
    AssetMetadata,          // asset_metadata
    text,                   // idempotency_key
) -> Result<MemoryId, Error>
```

### Backend Implementation

- **Core Function**: `memories_create_core` in `src/backend/src/memories/core/create.rs`
- **Public API**: `memories_create` in `src/backend/src/lib.rs`
- **Unit Tests**: Located in `src/backend/src/memories/core/create.rs`

### Test Coverage

- **Unit Tests**: 2 tests failing with `CheckSequenceNotMatch`
- **Integration Tests**: Multiple tests failing with type mismatches
- **JavaScript Tests**: Certificate verification errors (actually type mismatches)
- **Shell Tests**: Candid parsing errors

## Root Cause Analysis

### 1. Backend Implementation Issue

The unit tests are failing with `CheckSequenceNotMatch`, which suggests:

- There's a problem with the internal implementation of `memories_create_core`
- The function is not handling the input parameters correctly
- There might be a bug in the sequence checking logic

### 2. Interface Mismatch

The JavaScript tests are failing with type mismatches:

- The backend expects a `principal` type but we're passing a `text`
- This suggests the interface has changed or there's a version mismatch
- The fact that there are existing memories proves the function was working before

### 3. Candid Parsing Issues

The shell tests are failing with Candid parsing errors:

- The function signature appears correct in the interface
- The issue is with the way we're constructing the Candid arguments
- This might be related to the interface mismatch

## Impact

### Development Impact

- **Bulk Memory APIs**: Cannot be tested because they depend on `memories_create` for test setup
- **Upload Functionality**: May be affected if it depends on `memories_create`
- **Integration Testing**: Cannot test memory creation workflows

### User Impact

- **Memory Creation**: Users cannot create new memories
- **Upload Workflows**: May be broken if they depend on memory creation
- **Data Persistence**: Existing memories are preserved, but new ones cannot be created

## Proposed Solutions

### 1. Fix Backend Implementation

- **Priority**: HIGH
- **Action**: Debug the `CheckSequenceNotMatch` error in `memories_create_core`
- **Location**: `src/backend/src/memories/core/create.rs`
- **Tests**: Fix the failing unit tests

### 2. Fix Interface Mismatch

- **Priority**: HIGH
- **Action**: Identify which field expects a `principal` instead of `text`
- **Location**: JavaScript test calls and Candid interface
- **Tests**: Update test calls to use correct types

### 3. Fix Candid Parsing

- **Priority**: MEDIUM
- **Action**: Fix the Candid argument construction in shell tests
- **Location**: Shell test scripts
- **Tests**: Update shell test syntax

### 4. Add Integration Tests

- **Priority**: MEDIUM
- **Action**: Add comprehensive integration tests for `memories_create`
- **Location**: New test files
- **Tests**: Cover all parameter combinations and edge cases

## Next Steps

### Immediate Actions

1. **Debug Backend Implementation**: Fix the `CheckSequenceNotMatch` error in unit tests
2. **Identify Type Mismatch**: Find which field expects `principal` instead of `text`
3. **Fix JavaScript Tests**: Update test calls to use correct types
4. **Fix Shell Tests**: Update Candid argument construction

### Long-term Actions

1. **Add Comprehensive Testing**: Create integration tests for all `memories_create` scenarios
2. **Document Interface Changes**: Document any interface changes that occurred
3. **Add Error Handling**: Improve error handling and reporting
4. **Performance Testing**: Add performance tests for memory creation

## Related Issues

- [JavaScript Tests Certificate Verification Issue](./javascript-tests-certificate-verification-issue.md)
- [Bulk Memory APIs Implementation](./backend-bulk-memory-apis-implementation.md)
- [Backend Module Refactoring](./backend-module-refactoring-upload-split.md)

## Files Affected

### Backend Code

- `src/backend/src/memories/core/create.rs` - Core implementation
- `src/backend/src/lib.rs` - Public API
- `src/backend/src/memories/core/traits.rs` - Traits and interfaces

### Test Files

- `src/backend/src/memories/core/create.rs` - Unit tests
- `tests/backend/shared-capsule/memories/bulk-apis/test_bulk_memory_apis.mjs` - JavaScript tests
- `tests/backend/shared-capsule/memories/test_memories_create.sh` - Shell tests

### Documentation

- `docs/issues/open/javascript-tests-certificate-verification-issue.md` - Related issue
- `docs/issues/open/backend-bulk-memory-apis-implementation.md` - Related issue

## Conclusion

The `memories_create` function is experiencing multiple issues across different testing environments. The root cause appears to be a combination of:

1. **Backend implementation issues** (unit test failures)
2. **Interface mismatches** (JavaScript test failures)
3. **Candid parsing issues** (shell test failures)

The fact that there are existing memories in the capsule proves that `memories_create` was working before, suggesting that something has changed in the backend or interface that's causing these issues.

**Priority**: HIGH - This is blocking multiple features and testing workflows.
