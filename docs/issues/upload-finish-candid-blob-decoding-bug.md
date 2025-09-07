# Upload Finish Candid Blob Decoding Bug

## Issue Summary

The `uploads_finish` function has a critical bug where Candid blob parameters are not being properly decoded, causing hash validation to fail with incorrect length errors.

## Problem Description

When calling `uploads_finish` with a 32-byte hash blob, the backend receives the base64-encoded string instead of the decoded bytes, leading to hash length validation failures.

### Expected Behavior

- Pass a 32-byte hash blob to `uploads_finish`
- Backend should receive exactly 32 bytes for hash validation
- Function should succeed with valid hash

### Actual Behavior

- Backend receives 44 characters (base64-encoded 32-byte data)
- Hash validation fails with: `"invalid_hash_length: expected 32 bytes, got 44"`
- Function returns `InvalidArgument` error

## Technical Details

### Candid Interface

```candid
uploads_finish : (nat64, blob, nat64) -> (Result_8);
```

### Rust Function Signature

```rust
async fn uploads_finish(
    session_id: u64,
    expected_sha256: Vec<u8>,  // Should receive decoded bytes
    total_len: u64,
) -> types::Result<types::MemoryId>
```

### Error Location

```rust
// src/backend/src/lib.rs:339-347
let hash: [u8; 32] = match expected_sha256.clone().try_into() {
    Ok(h) => h,
    Err(_) => {
        return Err(types::Error::InvalidArgument(format!(
            "invalid_hash_length: expected 32 bytes, got {}",
            expected_sha256.len()  // Shows 44 instead of 32
        )))
    }
};
```

## Test Cases That Fail

### Test 1: Base64 Encoded 32-byte Hash

```bash
# Create 32 bytes of data
echo -n "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA" | base64
# Output: QUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUE=

# Call uploads_finish
dfx canister call backend uploads_finish '(3, blob "QUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUE=", 250)'

# Result: InvalidArgument("invalid_hash_length: expected 32 bytes, got 44")
```

### Test 2: Hex Encoded 32-byte Hash

```bash
# 32 bytes in hex (64 characters)
dfx canister call backend uploads_finish '(3, blob "4141414141414141414141414141414141414141414141414141414141414141", 250)'

# Result: InvalidArgument("invalid_hash_length: expected 32 bytes, got 64")
```

### Test 3: File-based Blob

```bash
# Create binary file
echo -n "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA" > /tmp/test_hash.bin
dfx canister call backend uploads_finish --argument-file <(echo '(3, blob "'$(base64 -i /tmp/test_hash.bin)'", 250)')

# Result: InvalidArgument("invalid_hash_length: expected 32 bytes, got 44")
```

## Comparison with Working Function

The `uploads_put_chunk` function works correctly with blob parameters:

### Candid Interface

```candid
uploads_put_chunk : (nat64, nat32, blob) -> (Result);
```

### Rust Implementation

```rust
async fn uploads_put_chunk(session_id: u64, chunk_idx: u32, bytes: Vec<u8>) -> types::Result<()>
```

### Working Test

```bash
# This works correctly
dfx canister call backend uploads_put_chunk '(3, 0, blob "QUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUE=")'
# Result: Ok
```

## Root Cause Analysis

The issue appears to be a mismatch between:

1. **Candid blob type** - Should automatically decode base64 to bytes
2. **Rust Vec<u8> parameter** - Should receive the decoded bytes
3. **Actual behavior** - Receives the base64 string instead of decoded bytes

## Impact Assessment

### High Priority

- **Upload workflow broken** - Users cannot complete file uploads
- **Test suite failures** - 5 out of 10 upload tests failing
- **Production blocking** - Core functionality unusable

### Affected Components

- `uploads_finish` function
- Upload workflow completion
- File upload test suite
- User file upload experience

## Investigation Questions for Senior Review

1. **Candid Blob Handling**: Is there a difference in how Candid handles blob parameters between `uploads_put_chunk` and `uploads_finish`?

2. **Function Signature**: Should `expected_sha256` be a different type (e.g., `[u8; 32]` instead of `Vec<u8>`)?

3. **Candid Interface**: Is the Candid interface definition correct for blob parameters?

4. **IC CDK Integration**: Are there any IC CDK specific requirements for blob parameter handling?

5. **Alternative Approaches**: Should we use a different parameter type (e.g., `text` with manual base64 decoding)?

## Proposed Solutions

### Option 1: Fix Candid Blob Decoding

- Investigate why blob parameters aren't being decoded properly
- Ensure consistent behavior across all blob parameters

### Option 2: Change Parameter Type

- Change `expected_sha256` from `Vec<u8>` to `text`
- Add manual base64 decoding in the function
- Update Candid interface accordingly

### Option 3: Use Fixed-Size Array

- Change parameter to `[u8; 32]` if Candid supports it
- This would enforce exact 32-byte requirement at the type level

## Test Environment

- **Platform**: macOS 23.4.0
- **DFX Version**: Latest
- **Backend**: Deployed and running
- **Test Data**: 32-byte hash (all 'A' characters for simplicity)

## Next Steps

1. **Senior Developer Review**: Investigate Candid blob handling differences
2. **ICP Expert Consultation**: Verify IC-specific blob parameter requirements
3. **Fix Implementation**: Apply the correct solution
4. **Test Validation**: Ensure all upload tests pass
5. **Documentation Update**: Document the correct blob parameter usage

## Related Files

- `src/backend/src/lib.rs` - Main implementation
- `src/backend/backend.did` - Candid interface
- `scripts/tests/backend/shared-capsule/upload/` - Test suite
- `scripts/tests/backend/shared-capsule/upload/upload_test_utils.sh` - Test utilities

## Priority

**CRITICAL** - This blocks the entire upload workflow and affects user experience.

---

_Created: $(date)_  
_Status: Open_  
_Assigned: Senior Developer + ICP Expert_
