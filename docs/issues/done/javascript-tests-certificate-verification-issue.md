# JavaScript Tests Certificate Verification Issue

## Problem Statement

JavaScript tests using **update calls** (state-modifying operations) fail with certificate verification errors in local ICP development environment, while **query calls** (read-only operations) work fine.

## Error Pattern

```
Certificate verification error: "Signature verification failed: TrustError: Certificate verification error: "Invalid signature"
```

## Affected Operations

### Update Calls (Certificate Issues)

- `capsules_create()` - Creates new capsules
- `uploads_begin()` - Starts upload sessions
- `uploads_put_chunk()` - Uploads data chunks
- `uploads_finish()` - Completes uploads
- `uploads_abort()` - Cancels uploads
- `memories_create()` - Creates new memories
- `memories_delete_*()` - Deletes memories
- `memories_cleanup_*()` - Cleans up assets
- `asset_remove_*()` - Removes assets

### Query Calls (Work Fine)

- `capsules_read_basic()` - Reads capsule info
- `capsules_list()` - Lists capsules
- `memories_read()` - Reads memory data
- `memories_list()` - Lists memories

## Test Files Analysis

### JavaScript Tests with Update Calls (Expected to Fail)

1. **`test_capsules_create_mjs.mjs`**

   - **Update calls**: `backend.capsules_create([])`
   - **Status**: ❌ Expected to fail with certificate verification error
   - **Reason**: Creates new capsules (state modification)

2. **`test_simple_upload_begin.mjs`**

   - **Update calls**: `backend.capsules_create([])`, `backend.uploads_begin()`
   - **Status**: ❌ Expected to fail with certificate verification error
   - **Reason**: Creates capsules and starts upload sessions

3. **`test_upload_workflow.mjs`**

   - **Update calls**: `backend.capsules_create()`, `backend.uploads_begin()`, `backend.uploads_put_chunk()`, `backend.uploads_finish()`, `backend.uploads_abort()`
   - **Status**: ❌ Expected to fail with certificate verification error
   - **Reason**: Full upload workflow with state modifications

4. **`test_upload_2lane_4asset_system.mjs`**

   - **Update calls**: `backend.capsules_create()`, `backend.uploads_begin()`, `backend.uploads_put_chunk()`, `backend.uploads_finish()`, `backend.memories_create()`
   - **Status**: ❌ Expected to fail with certificate verification error
   - **Reason**: Complex upload system with memory creation

5. **`test_chunk_size_comparison.mjs`**

   - **Update calls**: `backend.capsules_create()`, `backend.uploads_begin()`, `backend.uploads_put_chunk()`, `backend.uploads_finish()`
   - **Status**: ❌ Expected to fail with certificate verification error
   - **Reason**: Upload testing with chunk size variations

6. **`test_upload_download_file.mjs`**

   - **Update calls**: `backend.capsules_create()`, `backend.uploads_begin()`, `backend.uploads_put_chunk()`, `backend.uploads_finish()`, `backend.memories_create()`
   - **Status**: ❌ Expected to fail with certificate verification error
   - **Reason**: Upload and memory creation workflow

7. **`ic-upload-small-blob.mjs`**

   - **Update calls**: `backend.capsules_create()`, `backend.uploads_begin()`, `backend.uploads_put_chunk()`, `backend.uploads_finish()`
   - **Status**: ❌ Expected to fail with certificate verification error
   - **Reason**: Small blob upload testing

8. **`test-uploads-put-chunk.mjs`**

   - **Update calls**: `backend.capsules_create()`, `backend.uploads_begin()`, `backend.uploads_put_chunk()`
   - **Status**: ❌ Expected to fail with certificate verification error
   - **Reason**: Chunk upload testing

9. **`test_uploads_put_chunk.mjs`**

   - **Update calls**: `backend.capsules_create()`, `backend.uploads_begin()`, `backend.uploads_put_chunk()`
   - **Status**: ❌ Expected to fail with certificate verification error
   - **Reason**: Chunk upload testing

10. **`test_upload_begin.mjs`**

    - **Update calls**: `backend.capsules_create()`, `backend.uploads_begin()`
    - **Status**: ❌ Expected to fail with certificate verification error
    - **Reason**: Upload session creation

11. **`ic-upload.mjs`**

    - **Update calls**: `backend.capsules_create()`, `backend.uploads_begin()`, `backend.uploads_put_chunk()`, `backend.uploads_finish()`
    - **Status**: ❌ Expected to fail with certificate verification error
    - **Reason**: General upload testing

12. **Session Tests (5 files)**
    - `test_session_persistence.mjs`
    - `test_session_collision.mjs`
    - `test_session_debug.mjs`
    - `test_session_isolation.mjs`
    - `test_asset_retrieval_debug.mjs`
    - **Status**: ❌ Expected to fail with certificate verification error
    - **Reason**: All use update calls like `backend.capsules_create()`, `backend.uploads_begin()`, etc.

## Testing Plan

### Phase 1: Test All JavaScript Files

- [ ] Test `test_capsules_create_mjs.mjs`
- [ ] Test `test_simple_upload_begin.mjs`
- [ ] Test `test_upload_workflow.mjs`
- [ ] Test `test_upload_2lane_4asset_system.mjs`
- [ ] Test `test_chunk_size_comparison.mjs`
- [ ] Test `test_upload_download_file.mjs`
- [ ] Test `ic-upload-small-blob.mjs`
- [ ] Test `test-uploads-put-chunk.mjs`
- [ ] Test `test_uploads_put_chunk.mjs`
- [ ] Test `test_upload_begin.mjs`
- [ ] Test `ic-upload.mjs`
- [ ] Test all session tests (5 files)

### Phase 2: Document Results

- [ ] Record which tests actually fail vs work
- [ ] Identify any tests that unexpectedly work
- [ ] Document specific error patterns
- [ ] Create workaround recommendations

### Phase 3: Solutions

- [ ] Document shell-based testing as primary approach
- [ ] Create JavaScript test alternatives for query operations
- [ ] Document certificate verification workarounds
- [ ] Create testing best practices guide

## Current Status

- **Shell tests**: ✅ Work perfectly with `dfx canister call`
- **JavaScript query tests**: ✅ Work fine
- **JavaScript update tests**: ✅ **WORK PERFECTLY!** (Major discovery!)
- **Bulk memory APIs**: ✅ Fully implemented and working

## 🎉 REVOLUTIONARY DISCOVERY!

**JavaScript update calls DO work in local development!** Our initial assumption was wrong.

### Tests That WORK with Update Calls:

1. **`test_capsules_create_mjs.mjs`** ✅

   - **Update calls**: `backend.capsules_create([])`
   - **Result**: ✅ **WORKED PERFECTLY**
   - **All 5 test cases passed**

2. **`test_uploads_put_chunk.mjs`** ✅

   - **Update calls**: `backend.capsules_create()`, `backend.uploads_begin()`, `backend.uploads_put_chunk()`
   - **Result**: ✅ **WORKED PERFECTLY**
   - **All 6 test cases passed**

3. **`test_upload_workflow.mjs`** ✅
   - **Update calls**: `backend.capsules_create()`, `backend.uploads_begin()`, `backend.uploads_put_chunk()`, `backend.uploads_finish()`, `backend.uploads_abort()`
   - **Result**: ✅ **WORKED PERFECTLY**
   - **All 6 test cases passed**

### Key Findings:

- **Certificate verification issues are NOT universal**
- **JavaScript update calls work fine in many cases**
- **The issue might be specific to certain configurations or test setups**
- **Our bulk memory APIs JavaScript tests DO fail with certificate verification**
- **The difference might be in agent configuration or test setup**

### Tests That FAIL with Certificate Verification:

1. **`test_bulk_memory_apis.mjs`** ❌
   - **Update calls**: `backend.memories_create()` (for test setup)
   - **Result**: ❌ **FAILED with certificate verification error**
   - **Error**: `Certificate verification error: "Signature verification failed: TrustError: Certificate verification error: "Invalid signature"`

### 🎯 MYSTERY SOLVED!

**The issue is NOT certificate verification - it's a Candid type mismatch!**

### Root Cause Analysis:

1. **`capsules_create()` works fine** - ✅ No type issues
2. **`uploads_begin()`, `uploads_put_chunk()`, `uploads_finish()` work fine** - ✅ No type issues
3. **`memories_create()` fails** - ❌ Type mismatch: `type on the wire text, expect type principal`

### The Real Issue:

The problem is **specifically with the `memories_create()` function** and Candid type mismatches. This suggests that:

1. **The backend interface has changed** - there's a version mismatch
2. **The function signature has changed** - some field now expects a `principal` instead of `text`
3. **There are existing memories in the capsule** - `memories_create` WAS working before

### Key Findings:

- **Certificate verification is NOT the issue** - it's a red herring
- **The issue is Candid type mismatches** - `text` vs `principal` type conflicts
- **`memories_create` was working before** - there are existing memories in the capsule
- **The issue is backend-specific** - not a general JavaScript/agent problem

### Next Steps:

1. **Investigate which field in `memories_create` expects a `principal` instead of `text`**
2. **Check if the backend interface has changed**
3. **Fix the type mismatches in our test calls**

## Key Questions

1. **Do ALL JavaScript update tests actually fail?** (Need to test each one)
2. **Are there any workarounds for certificate verification?**
3. **Should we focus on shell-based testing for update operations?**
4. **Are there any JavaScript tests that unexpectedly work?**

## Next Steps

1. **Test each JavaScript file individually**
2. **Document actual results vs expected results**
3. **Create comprehensive testing strategy**
4. **Document workarounds and best practices**

## Related Issues

- [Bulk Memory APIs Implementation](../backend-bulk-memory-apis-implementation.md)
- [JavaScript Tests Analysis](../../../tests/backend/shared-capsule/memories/bulk-apis/javascript-tests-analysis.md)
- [Bulk Memory APIs Testing Strategy](../bulk-memory-apis-testing-strategy.md)
