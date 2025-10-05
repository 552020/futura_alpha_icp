# Bulk Memory APIs - End-to-End Testing Strategy

## Problem Statement

Current tests are **meaningless** because they test endpoints with non-existent data, returning only `NotFound` errors. This doesn't validate that the endpoints actually work correctly.

**Bad Example:**

```bash
dfx canister call backend memories_delete_bulk '("non-existent-capsule", vec {"non-existent-memory"})'
# Result: (variant { Err = variant { NotFound } })
# This tells us NOTHING about whether the function works!
```

## Solution: Proper E2E Testing Strategy

### Phase 1: Data Setup (Prerequisites)

Before testing any bulk operations, we need to create **real test data**:

1. **Create a test capsule**
2. **Create multiple test memories** with different asset types:
   - Inline assets (small files)
   - Internal assets (ICP blob storage)
   - External assets (S3/Vercel storage)
3. **Verify data exists** before testing bulk operations

### Phase 2: Meaningful Test Scenarios

#### Test 1: `memories_delete_bulk` - Real Data

```bash
# Setup: Create capsule and memories
dfx canister call backend capsules_create '(null)'
# Get capsule_id from response

dfx canister call backend memories_create '(
  "capsule_id",
  opt blob {0; 1; 2; 3; 4},
  null,
  null,
  null,
  null,
  null,
  null,
  record {
    Document = record {
      base = record {
        name = "test-doc-1";
        mime_type = "text/plain";
        bytes = 5;
        tags = vec {"test"};
        description = opt "Test document 1";
        created_at = 0;
        updated_at = 0;
        asset_type = variant { Original };
        url = null;
        height = null;
        sha256 = null;
        storage_key = null;
        processing_error = null;
        deleted_at = null;
        asset_location = null;
        width = null;
        processing_status = null;
        bucket = null;
      };
      document_type = null;
      language = null;
      page_count = null;
      word_count = null;
    };
  },
  "test-idempotency-1"
)'

# Create second memory
dfx canister call backend memories_create '(...)' # Similar call for memory 2

# NOW test bulk delete with REAL data
dfx canister call backend memories_delete_bulk '("capsule_id", vec {"memory_id_1"; "memory_id_2"})'
# Expected: (variant { Ok = record { deleted_count = 2; failed_count = 0; message = "..." } })
```

#### Test 2: `memories_delete_all` - Real Data

```bash
# Setup: Create capsule with multiple memories
# ... create capsule and 3-4 memories ...

# Test delete all
dfx canister call backend memories_delete_all '("capsule_id")'
# Expected: (variant { Ok = record { deleted_count = 4; failed_count = 0; message = "..." } })

# Verify: Check that capsule has no memories left
dfx canister call backend memories_list '("capsule_id")'
# Expected: Empty memories list
```

#### Test 3: `memories_cleanup_assets_all` - Real Data

```bash
# Setup: Create memory with assets
# ... create memory with inline/blob/external assets ...

# Test cleanup assets (preserves memory, removes assets)
dfx canister call backend memories_cleanup_assets_all '("memory_id")'
# Expected: (variant { Ok = record { memory_id = "memory_id"; assets_cleaned = 3; message = "..." } })

# Verify: Memory still exists but has no assets
dfx canister call backend memories_read '("memory_id")'
# Expected: Memory exists but inline_assets, blob_internal_assets, blob_external_assets are empty
```

#### Test 4: `memories_cleanup_assets_bulk` - Real Data

```bash
# Setup: Create multiple memories with assets
# ... create 3 memories with different asset types ...

# Test bulk cleanup
dfx canister call backend memories_cleanup_assets_bulk '(vec {"memory_1"; "memory_2"; "memory_3"})'
# Expected: (variant { Ok = 6 : nat64 }) # 6 total assets cleaned

# Verify: All memories exist but have no assets
```

#### Test 5-8: Asset Removal Endpoints - Real Data

```bash
# Setup: Create memory with specific asset types
# ... create memory with inline, internal, external assets ...

# Test asset_remove (generic)
dfx canister call backend asset_remove '("memory_id", "asset_ref")'
# Expected: (variant { Ok = record { memory_id = "memory_id"; asset_removed = true; message = "..." } })

# Test asset_remove_inline
dfx canister call backend asset_remove_inline '("memory_id", 0)'
# Expected: (variant { Ok = record { memory_id = "memory_id"; asset_removed = true; message = "..." } })

# Test asset_remove_internal
dfx canister call backend asset_remove_internal '("memory_id", "blob_ref")'
# Expected: (variant { Ok = record { memory_id = "memory_id"; asset_removed = true; message = "..." } })

# Test asset_remove_external
dfx canister call backend asset_remove_external '("memory_id", "storage_key")'
# Expected: (variant { Ok = record { memory_id = "memory_id"; asset_removed = true; message = "..." } })
```

### Phase 3: Error Handling Tests

Test meaningful error scenarios:

```bash
# Test 1: Delete non-existent memories (partial failure)
dfx canister call backend memories_delete_bulk '("capsule_id", vec {"existing_memory"; "non_existent_memory"})'
# Expected: (variant { Ok = record { deleted_count = 1; failed_count = 1; message = "..." } })

# Test 2: Cleanup assets from non-existent memory
dfx canister call backend memories_cleanup_assets_all '("non_existent_memory")'
# Expected: (variant { Err = variant { NotFound } })

# Test 3: Remove asset from non-existent memory
dfx canister call backend asset_remove '("non_existent_memory", "asset_ref")'
# Expected: (variant { Err = variant { NotFound } })
```

### Phase 4: Performance Tests

Test with larger datasets:

```bash
# Create 100 memories in a capsule
# Test bulk operations on all 100
# Measure performance and verify all operations complete
```

## Test Data Requirements

### Memory Types to Test:

1. **Inline Memory**: Small files stored directly in memory struct
2. **Blob Internal Memory**: Large files stored in ICP blob store
3. **Blob External Memory**: Files stored in external storage (S3, Vercel)

### Asset Types to Test:

1. **Inline Assets**: Text files, small images
2. **Internal Assets**: Large images, videos, documents
3. **External Assets**: Files stored in S3, Vercel Blob, etc.

### Test Scenarios:

1. **Single Memory Operations**: Test each endpoint with one memory
2. **Multiple Memory Operations**: Test bulk endpoints with 2-5 memories
3. **Large Dataset Operations**: Test with 10-100 memories
4. **Mixed Asset Types**: Test with memories containing different asset types
5. **Partial Failures**: Test with some existing and some non-existing memories

## Success Criteria

### Functional Tests:

- ✅ All 8 endpoints work with real data
- ✅ Bulk operations handle multiple items correctly
- ✅ Error handling works for non-existent data
- ✅ Partial failures are handled gracefully

### Performance Tests:

- ✅ Bulk operations complete within reasonable time
- ✅ Memory usage doesn't spike during bulk operations
- ✅ Canister doesn't run out of cycles during large operations

### Data Integrity Tests:

- ✅ After bulk delete, memories are actually gone
- ✅ After asset cleanup, assets are removed but memories remain
- ✅ After asset removal, specific assets are removed but others remain

## Implementation Plan

### Phase 1: Setup Test Infrastructure

1. Create test data setup scripts
2. Create test data verification scripts
3. Create test cleanup scripts

### Phase 2: Implement Test Suite

1. Create comprehensive test script for all 8 endpoints
2. Test with real data scenarios
3. Test error handling scenarios
4. Test performance with larger datasets

### Phase 3: Automation

1. Integrate tests into CI/CD pipeline
2. Add performance benchmarks
3. Add regression testing

## Conclusion

**Current Status**: 4/8 endpoints tested with meaningless data
**Target Status**: 8/8 endpoints tested with meaningful, real-world scenarios

This approach will give us **confidence** that the bulk memory APIs actually work correctly in production scenarios, not just return `NotFound` errors for non-existent data.
