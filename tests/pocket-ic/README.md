# PocketIC Integration Tests

This directory contains comprehensive PocketIC integration tests for the backend canister. These tests verify complete memory management functionality, HTTP module integration, and access control in a controlled environment without network dependencies.

## Overview

The tests verify:

- ✅ **Memory Management**: Create, Read, Update, Delete operations
- ✅ **Access Control**: Authorization and permission boundaries
- ✅ **HTTP Integration**: Complete flow from memory creation to asset serving
- ✅ **Token validation**: Proper authentication and authorization
- ✅ **Asset serving**: Correct content type, headers, and body
- ✅ **Error handling**: 401, 403, 404 responses for various error cases
- ✅ **Security**: Private cache control, no-store headers
- ✅ **Idempotency**: Duplicate operations work correctly

## Test Structure

### `http_integration_tests.rs`

**Main test function**: `test_http_module_integration()`

**Test flow**:

1. **Setup**: Initialize PocketIC and install backend canister
2. **Create test data**: Create capsule and memory with inline image asset
3. **Mint token**: Generate HTTP token for the memory
4. **Serve asset**: Test HTTP request with valid token
5. **Negative cases**: Test various error scenarios

**Test cases**:

- ✅ **Happy path**: Valid token → 200 OK with correct headers
- ✅ **Missing token**: No token → 401/403
- ✅ **Invalid token**: Bad token → 401
- ✅ **Wrong variant**: Token for thumbnail used on original → 403
- ✅ **Non-existent memory**: Invalid memory ID → 404

### `memories_pocket_ic.rs` (Backend Package)

**Comprehensive memory management tests**:

1. **`test_create_and_read_memory_happy_path`**: Basic memory creation and retrieval
2. **`test_memory_creation_idempotency`**: Duplicate operations return same result
3. **`test_memory_update_roundtrip`**: Update memory and verify changes
4. **`test_delete_forbidden_for_non_owner`**: Access control for delete operations
5. **`test_memory_crud_full_workflow`**: Complete CRUD operations cycle

**Test coverage**:

- ✅ **Memory Creation**: Inline assets, external assets, proper metadata
- ✅ **Memory Reading**: Retrieve and verify memory data
- ✅ **Memory Updates**: Modify metadata and verify changes
- ✅ **Memory Deletion**: Delete operations and access control
- ✅ **Access Control**: Owner vs non-owner permissions
- ✅ **Error Handling**: Proper error responses for invalid operations

## Current Status

**✅ All PocketIC Tests Working:**

- **HTTP Integration Tests**: Complete flow from memory creation to asset serving
- **Memory Management Tests**: Full CRUD operations with access control
- **Simple Tests**: Basic canister operations and setup verification
- **Hello World Tests**: Fundamental PocketIC functionality

**✅ Test Infrastructure:**

- PocketIC is installed and functional
- Backend canister installation and calls work correctly
- All API signatures match current backend.did specification
- Type definitions updated to match current backend structure
- Proper error handling for all operations

**✅ Test Results:**

- **5/5 Memory Management Tests**: All passing
- **1/1 HTTP Integration Test**: Passing
- **2/2 Simple Tests**: All passing
- **2/2 Hello World Tests**: All passing

**Total: 10/10 PocketIC tests passing** 🎉

## Running the Tests

**All Tests Working:**

```bash
# All PocketIC tests (consolidated in workspace member)
cargo test --package http-integration-tests

# Individual test binaries
cargo test --package http-integration-tests --bin http_integration_tests
cargo test --package http-integration-tests --bin memories_pocket_ic
cargo test --package http-integration-tests --bin simple_pocket_ic
cargo test --package http-integration-tests --bin hello_world_pocket_ic
cargo test --package http-integration-tests --bin simple_memory_test
```

### Workspace Integration

The PocketIC tests are now integrated as workspace members, sharing the main project's:

- **Target directory**: Build artifacts stored in `/target/` (not duplicated)
- **Dependencies**: Shared `Cargo.lock` for consistent versions
- **Build process**: Single `cargo build` builds everything

### Available Test Scripts

```bash
# Backend unit tests (fast, no PocketIC)
./src/backend/run_backend_unit_tests.sh

# All PocketIC integration tests
./tests/pocket-ic/run_all_pocket_ic_tests.sh

# Memory management tests (with optimizations)
./tests/pocket-ic/run_pocket_ic_tests.sh

# Run specific memory test
./tests/pocket-ic/run_pocket_ic_tests.sh test_create_and_read_memory_happy_path
```

### Check compilation

```bash
cargo check
```

````

**Test Results:**

```bash
# Memory management tests - ALL PASSING
running 5 tests
test test_create_and_read_memory_happy_path ... ok
test test_memory_creation_idempotency ... ok
test test_memory_update_roundtrip ... ok
test test_delete_forbidden_for_non_owner ... ok
test test_memory_crud_full_workflow ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

# HTTP integration test - PASSING
running 1 test
test test_http_module_integration ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
````

**Available Test Scripts:**

```bash
# Run all working tests
./tests/pocket-ic/run_working_tests.sh

# Run unit tests (fast)
./tests/pocket-ic/run_unit_tests.sh

# Run PocketIC tests with optimized settings
./tests/pocket-ic/run_pocket_ic_tests.sh
```

## Test Data

### Test Image

- **Format**: 1x1 PNG (68 bytes)
- **Purpose**: Minimal valid image for testing
- **Content**: Standard PNG signature with minimal valid structure

### Test Metadata

- **Name**: `test_image.png`
- **MIME type**: `image/png`
- **Dimensions**: 1x1 pixels
- **Tags**: `["test", "image", "http"]`

## Expected Results

### Successful Response (200 OK)

```
Status: 200 OK
Content-Type: image/png
Cache-Control: private, no-store
Body: [68 bytes of PNG data]
```

### Error Responses

- **401 Unauthorized**: Invalid or missing token
- **403 Forbidden**: Valid token but wrong permissions/variant
- **404 Not Found**: Non-existent memory or asset

## Integration with Main Test Suite

These tests complement the existing test suite:

- **Unit tests**: Test core logic in isolation
- **PocketIC tests**: Test complete integration flow ✅ **WORKING**
- **Local replica tests**: Test with real HTTP requests
- **Browser tests**: Test end-to-end user experience

## Key Fixes Applied

**API Alignment:**

- Updated all type definitions to match current `backend.did`
- Fixed `memories_create`, `memories_update`, `memories_delete` function signatures
- Corrected `capsules_create` to use `Option<PersonRef>` parameter
- Updated return types: `Result5`/`Result11` → `Result6`/`Result20`

**Type Structure Updates:**

- Added missing `access_entries` field to `Memory` type
- Updated `MemoryMetadata` with all current fields
- Fixed `MemoryUpdateData` structure (removed `access`, added `access_entries`)
- Added missing types: `AccessEntry`, `ResourceRole`, `GrantSource`, etc.

**Error Handling:**

- Proper handling of `NotFound` vs `Unauthorized` errors
- Correct Candid decoding for all operation results
- Fixed backend panic issues in Candid type mismatches

## Dependencies

- `pocket-ic`: ICP testing framework
- `candid`: Interface description language
- `ic-cdk`: Internet Computer Development Kit
- `tokio`: Async runtime
- `serde`: Serialization framework
- `chrono`: Date/time handling

## Notes

- Tests run in isolated PocketIC environment
- No network dependencies or external services required
- Fast execution compared to local replica tests
- Ideal for CI/CD pipelines
- Tests real canister logic with controlled inputs
- All tests use actual compiled WASM modules
- Comprehensive coverage of memory management operations
- Proper access control and security testing

## Success Metrics

**✅ Complete Test Coverage:**

- Memory CRUD operations: Create, Read, Update, Delete
- Access control: Owner vs non-owner permissions
- HTTP integration: Token minting and asset serving
- Error handling: All error scenarios covered
- Idempotency: Duplicate operations work correctly

**✅ Production Ready:**

- All tests pass consistently
- No flaky or intermittent failures
- Proper error handling and edge cases
- Real canister logic testing
- Comprehensive API coverage
