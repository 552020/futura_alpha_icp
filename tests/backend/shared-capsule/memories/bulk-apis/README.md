# Bulk Memory APIs Testing

This directory contains comprehensive tests for the new bulk memory API endpoints that provide efficient memory and asset management operations.

## New API Endpoints Tested

### 1. Bulk Memory Operations

- **`memories_delete_bulk(capsule_id, memory_ids[])`** - Delete multiple memories in a single operation
- **`memories_delete_all(capsule_id)`** - Delete ALL memories in a capsule (high-risk operation)

### 2. Asset Cleanup Operations

- **`memories_cleanup_assets_all(memory_id)`** - Clean up all assets from a memory while preserving the memory record
- **`memories_cleanup_assets_bulk(memory_ids[])`** - Bulk cleanup assets from multiple memories

### 3. Granular Asset Operations

- **`asset_remove(memory_id, asset_ref)`** - Remove a specific asset by its reference
- **`asset_remove_inline(memory_id, asset_index)`** - Remove a specific inline asset by its index
- **`asset_remove_internal(memory_id, blob_ref)`** - Remove a specific internal blob asset by its blob reference
- **`asset_remove_external(memory_id, storage_key)`** - Remove a specific external asset by its storage key

## Test Files

### Core Test Files

- **`test_bulk_memory_apis.mjs`** - Main test suite covering all 8 new endpoints
- **`bulk_test_helpers.mjs`** - Helper functions for test data creation and validation
- **`run_bulk_tests.mjs`** - Test runner script with orchestration and reporting

### Test Coverage

#### ✅ Bulk Delete Operations

- Success scenarios with valid memory IDs
- Partial failure scenarios with mixed valid/invalid IDs
- Error handling with invalid inputs
- Performance testing with large batches

#### ✅ Delete All Operations

- Complete capsule memory deletion
- Safety verification and error handling
- Performance impact assessment

#### ✅ Asset Cleanup Operations

- Individual memory asset cleanup
- Bulk asset cleanup across multiple memories
- Asset type validation (inline, internal, external)
- Cleanup result verification

#### ✅ Granular Asset Operations

- Inline asset removal by index
- Internal blob asset removal by reference
- External asset removal by storage key
- Error handling for invalid asset references

## Running the Tests

### Prerequisites

```bash
# Ensure dfx is installed and running
dfx --version

# Start the local replica
dfx start

# Deploy the backend canister
dfx deploy backend
```

### Run All Tests

```bash
# Run all bulk memory API tests
node run_bulk_tests.mjs

# Run with debug output
DEBUG=true node run_bulk_tests.mjs

# Run specific test file
node test_bulk_memory_apis.mjs
```

### Environment Variables

```bash
# Set ICP host (default: http://127.0.0.1:4943)
export IC_HOST="http://127.0.0.1:4943"

# Set backend canister ID
export BACKEND_CANISTER_ID="your-canister-id"

# Enable debug output
export DEBUG="true"
```

## Test Structure

### Test Data Setup

- **Capsule Creation**: Automatic capsule creation if none exists
- **Memory Creation**: Batch creation of test memories with different asset types
- **Asset Setup**: Inline, internal, and external asset configuration
- **Cleanup**: Automatic cleanup of test data after each test

### Validation Patterns

- **Result Validation**: Type-safe validation of API responses
- **State Verification**: Post-operation state verification
- **Error Handling**: Comprehensive error scenario testing
- **Performance**: Execution time and resource usage monitoring

### Helper Functions

- **`createTestMemory()`** - Create individual test memories
- **`createTestMemoriesBatch()`** - Create multiple memories for bulk operations
- **`verifyMemoriesDeleted()`** - Verify memory deletion
- **`validateBulkDeleteResult()`** - Validate bulk operation results
- **`cleanupTestMemories()`** - Clean up test data

## Test Results

### Success Criteria

- ✅ All 8 new endpoints tested
- ✅ Success and error scenarios covered
- ✅ Authentication/authorization validated
- ✅ Performance benchmarks established
- ✅ Test execution time < 10 seconds (vs 30+ for PocketIC)

### Quality Metrics

- **Test Coverage**: 100% endpoint coverage
- **Error Scenarios**: Comprehensive error handling validation
- **Performance**: Fast execution with real ICP environment
- **Maintainability**: Well-structured, documented test code

## Integration with Existing Tests

### Test Orchestration

- **`run_all_memory_tests.sh`** - Integrates with existing memory test suite
- **Parallel Execution** - Runs alongside existing memory tests
- **Shared Infrastructure** - Uses existing test utilities and patterns

### Backend Integration

- **Real ICP Environment** - Tests actual canister behavior
- **Type Safety** - Uses exact backend types via `backend.did.js`
- **Authentication** - Tests real Principal-based auth via `ic-identity.js`

## Future Enhancements

### Planned Additions

- **Performance Testing** - Large dataset testing (1000+ memories)
- **Concurrency Testing** - Parallel operation testing
- **Stress Testing** - Resource limit validation
- **Integration Testing** - End-to-end workflow testing

### Test Maintenance

- **Automated Updates** - Keep tests in sync with API changes
- **Documentation** - Maintain comprehensive test documentation
- **CI/CD Integration** - Automated test execution in CI pipeline

## Troubleshooting

### Common Issues

1. **dfx not found** - Install dfx and ensure it's in PATH
2. **Backend canister not running** - Run `dfx start && dfx deploy backend`
3. **Authentication failures** - Check dfx identity configuration
4. **Test timeouts** - Increase timeout values for slow operations

### Debug Mode

```bash
# Enable detailed debug output
DEBUG=true node test_bulk_memory_apis.mjs

# Check dfx status
dfx canister status backend

# Verify canister deployment
dfx canister call backend health_check
```

## Contributing

### Adding New Tests

1. Follow existing test patterns in `test_bulk_memory_apis.mjs`
2. Use helper functions from `bulk_test_helpers.mjs`
3. Add comprehensive error handling and validation
4. Update this README with new test coverage

### Test Standards

- **Naming**: Use descriptive test function names
- **Documentation**: Include comprehensive comments
- **Error Handling**: Test both success and failure scenarios
- **Cleanup**: Always clean up test data
- **Performance**: Monitor and report execution times


