# PocketIC Candid Decoding Error: "Fail to decode argument 0 from table0 to text"

## Issue Summary

PocketIC integration tests are failing with a Candid decoding error when calling `memories_create`. The error occurs during argument decoding, specifically failing to decode the first argument from a table to text.

## Error Details

```
Error: Update call failed: RejectResponse {
  reject_code: CanisterError,
  reject_message: "Error from Canister lxzze-o7777-77777-aaaaa-cai: Canister called `ic0.trap` with message: 'Panicked at 'called `Result::unwrap()` on an `Err` value: Custom(Fail to decode argument 0 from table0 to text\n\nCaused by:\n    Subtyping error: text)', src/backend/src/lib.rs:250:1'.\nConsider gracefully handling failures from this canister or altering the canister to handle exceptions. See documentation: https://internetcomputer.org/docs/current/references/execution-errors#trapped-explicitly",
  error_code: CanisterCalledTrap,
  certified: true
}
```

## Affected Tests

- `test_memory_crud_full_workflow`
- `test_memory_update_roundtrip`
- `test_delete_forbidden_for_non_owner`
- `test_memory_creation_idempotency`

## Root Cause Analysis

### 1. Function Signature Mismatch

The `memories_create` function in `lib.rs` has this signature:

```rust
fn memories_create(
    capsule_id: types::CapsuleId,  // This is a String
    bytes: Option<Vec<u8>>,
    blob_ref: Option<types::BlobRef>,
    external_location: Option<types::StorageEdgeBlobType>,
    external_storage_key: Option<String>,
    external_url: Option<String>,
    external_size: Option<u64>,
    external_hash: Option<Vec<u8>>,
    asset_metadata: types::AssetMetadata,
    idem: String,
) -> std::result::Result<types::MemoryId, Error>
```

### 2. Test Call Pattern

The tests are calling it like this:

```rust
let args = (
    "cap_crud".to_string(),  // capsule_id as String
    Some(vec![1, 2, 3, 4, 5]), // bytes
    Option::<BlobRef>::None, // blob_ref
    Option::<StorageEdgeBlobType>::None, // external_location
    Option::<String>::Some("crud-test.jpg".into()), // external_storage_key
    Option::<String>::None, // external_url
    Option::<u64>::None, // external_size
    Option::<Vec<u8>>::None, // external_hash
    image_meta_now("crud-test.jpg", "image/jpeg", 5, 1, 1, 1_695_000_000_000), // asset_metadata
    "crud-workflow".to_string(), // idem
);

let raw = pic.update_call(canister_id, controller, "memories_create", Encode!(&args)?)
```

### 3. The Problem

The error "Fail to decode argument 0 from table0 to text" suggests that:

- The Candid decoder is receiving a table/record structure for the first argument
- But it expects a simple `text` (String) type
- This indicates a mismatch between what the test is sending and what the canister expects

## Investigation Needed

### 1. Check .did File

Verify the actual Candid interface definition:

```bash
# Generate the .did file and check the memories_create signature
cargo build
# Check the generated .did file for memories_create signature
```

### 2. Verify Type Definitions

Check if `types::CapsuleId` is properly defined as a String alias:

```rust
// In types.rs, verify:
pub type CapsuleId = String;
```

### 3. Check Candid Serialization

The issue might be in how the test is encoding the arguments. The error suggests the first argument is being encoded as a table instead of a simple string.

## Potential Solutions

### Solution 1: Fix Test Encoding

If the issue is in the test, ensure proper argument encoding:

```rust
// Instead of encoding a tuple, encode individual arguments
let raw = pic.update_call(
    canister_id,
    controller,
    "memories_create",
    Encode!(
        &"cap_crud".to_string(),  // capsule_id
        &Some(vec![1, 2, 3, 4, 5]), // bytes
        &Option::<BlobRef>::None, // blob_ref
        // ... rest of arguments
    )?
)
```

### Solution 2: Fix Function Signature

If the issue is in the function signature, ensure it matches the .did file:

```rust
// Make sure the function signature exactly matches the Candid interface
#[ic_cdk::update]
fn memories_create(
    capsule_id: String,  // Not types::CapsuleId
    // ... rest of parameters
) -> Result_5 {
    // Implementation
}
```

### Solution 3: Check Candid Export

Ensure the `ic_cdk::export_candid!()` macro is working correctly and generating the right interface.

## Files Involved

- `src/backend/src/lib.rs` - Function definition
- `src/backend/tests/memories_pocket_ic.rs` - Failing tests
- `src/backend/src/types.rs` - Type definitions
- Generated `.did` file - Candid interface

## Priority

**High** - This blocks integration testing and prevents verification of the decoupled architecture implementation.

## Next Steps

1. Generate and examine the `.did` file to see the actual Candid interface
2. Compare the test encoding with the expected interface
3. Fix the mismatch between test and canister interface
4. Re-run the PocketIC tests to verify the fix

## Related Context

This issue appeared after implementing the decoupled architecture and removing the `ApiResult` anti-pattern. The core functionality works (unit tests pass), but the integration tests are failing due to Candid interface mismatches.

## Investigation Results

### .did File Analysis ✅

The generated `.did` file shows the correct interface:

```candid
memories_create : (
    text,                    // capsule_id (String)
    opt blob,               // bytes
    opt BlobRef,            // blob_ref
    opt StorageEdgeBlobType, // external_location
    opt text,               // external_storage_key
    opt text,               // external_url
    opt nat64,              // external_size
    opt blob,               // external_hash
    AssetMetadata,          // asset_metadata
    text,                   // idem
) -> (Result_5);
```

The interface is correct - the first parameter is indeed `text` (String).

### Root Cause Identified

The issue is likely in the test's argument encoding. The error "Fail to decode argument 0 from table0 to text" suggests that when the test encodes the tuple `&args`, the first element is being encoded as a table/record structure instead of a simple string.

### Most Likely Solution

The test should encode arguments individually rather than as a tuple:

```rust
// Current (problematic):
let raw = pic.update_call(canister_id, controller, "memories_create", Encode!(&args)?)

// Should be:
let raw = pic.update_call(canister_id, controller, "memories_create", Encode!(
    &"cap_crud".to_string(),  // capsule_id
    &Some(vec![1, 2, 3, 4, 5]), // bytes
    &Option::<BlobRef>::None, // blob_ref
    &Option::<StorageEdgeBlobType>::None, // external_location
    &Option::<String>::Some("crud-test.jpg".into()), // external_storage_key
    &Option::<String>::None, // external_url
    &Option::<u64>::None, // external_size
    &Option::<Vec<u8>>::None, // external_hash
    &image_meta_now("crud-test.jpg", "image/jpeg", 5, 1, 1, 1_695_000_000_000), // asset_metadata
    &"crud-workflow".to_string(), // idem
)?)
```

## Status

🔴 **OPEN** - Root cause identified, needs test fix
