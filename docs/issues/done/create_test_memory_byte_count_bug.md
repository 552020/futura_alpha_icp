# Bug: `create_test_memory` utility function calculates byte count incorrectly

## Problem Description

The `create_test_memory` utility function in `tests/backend/test_utils.sh` has a bug in its byte count calculation that causes memory creation to fail with the error:

```
InvalidArgument = "inline bytes_len != metadata.base.bytes"
```

## Root Cause

In `tests/backend/test_utils.sh` line 369:

```bash
local asset_metadata=$(create_document_asset_metadata "$name" "$description" "$tags" "$(echo -n "$base64_content" | wc -c)")
```

The function is using `wc -c` on the **base64-encoded content** instead of the **decoded byte count**. This causes a mismatch between:

- The actual byte count of the decoded data
- The byte count reported in the metadata

## Example

For the string `"Hello World"` (11 bytes):

- Base64: `"SGVsbG8gV29ybGQ="` (16 characters)
- `wc -c` on base64: `16` ❌ (wrong)
- Actual decoded bytes: `11` ✅ (correct)

## Impact

- All tests using `create_test_memory` fail
- Cannot create test memories for comprehensive testing
- Blocks development of new test cases

## Current Workaround

Tests that work use hardcoded byte counts or bypass the utility function entirely.

## Proposed Fix

The byte count calculation should be:

```bash
# Option 1: Calculate decoded byte count
local decoded_bytes=$(echo -n "$base64_content" | base64 -d | wc -c)
local asset_metadata=$(create_document_asset_metadata "$name" "$description" "$tags" "$decoded_bytes")

# Option 2: Use a more robust calculation
local byte_count=$(echo -n "$base64_content" | base64 -d | wc -c)
local asset_metadata=$(create_document_asset_metadata "$name" "$description" "$tags" "$byte_count")
```

## Test Case

```bash
# This should work but currently fails:
local memory_bytes='blob "SGVsbG8gV29ybGQ="'  # "Hello World" in base64
local memory_id=$(create_test_memory "$capsule_id" "test" "Test" '"test"' "$memory_bytes" "$CANISTER_ID" "$IDENTITY")
```

## Files Affected

- `tests/backend/test_utils.sh` (line 369)
- All test files using `create_test_memory`
- New test development blocked

## Priority

**High** - This blocks test development and makes the test suite unreliable.

## Additional Context

The `create_document_asset_metadata` function expects the actual byte count of the decoded data, not the base64 string length. This is a common mistake when working with base64 encoding.

## Verification

After fixing, this test should pass:

```bash
# Test with "Hello World" (11 bytes)
local memory_bytes='blob "SGVsbG8gV29ybGQ="'
local memory_id=$(create_test_memory "$capsule_id" "test" "Test" '"test"' "$memory_bytes" "$CANISTER_ID" "$IDENTITY")
# Should return a valid memory ID, not fail with byte count mismatch
```
