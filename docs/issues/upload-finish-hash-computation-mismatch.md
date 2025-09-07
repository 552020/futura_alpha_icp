# Upload Finish Hash Computation Mismatch

## Issue Summary

The `uploads_finish` function is computing a different SHA256 hash than what the client expects, causing `checksum_mismatch` errors even when the correct hash is provided.

## Problem Description

After fixing the Candid blob decoding issue, we discovered that the backend computes a different SHA256 hash than what the client calculates, leading to hash validation failures.

### Expected Behavior

- Client uploads chunks with known data
- Client computes SHA256 hash of the uploaded data
- Backend computes the same SHA256 hash during `uploads_finish`
- Hash validation succeeds

### Actual Behavior

- Client computes hash: `134e6543ddc35b40abb4f2f8aaaa2d0513a27e267beaf9081e29d84eba94017d`
- Backend computes hash: `75b4538b54f44331bd862a76c245952095ebe421e9ee70a12cb4f1838b979b9b`
- Hash validation fails with `checksum_mismatch`

## Technical Details

### Test Case

```bash
# Create 100 bytes of '0' characters
echo -n "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" > /tmp/test_chunk.bin

# Upload chunk successfully
dfx canister call backend uploads_put_chunk '(4, 0, blob "'$(base64 -i /tmp/test_chunk.bin)'")'
# Result: (variant { Ok })

# Compute expected hash
HASH_HEX=$(shasum -a 256 /tmp/test_chunk.bin | cut -d' ' -f1)
# Result: 134e6543ddc35b40abb4f2f8aaaa2d0513a27e267beaf9081e29d84eba94017d

# Finish upload with expected hash
dfx canister call backend uploads_finish "(4, blob \"$BLOB_ARG\", 100)"
# Result: checksum_mismatch: expected=134e6543ddc35b40abb4f2f8aaaa2d0513a27e267beaf9081e29d84eba94017d, actual=75b4538b54f44331bd862a76c245952095ebe421e9ee70a12cb4f1838b979b9b
```

### Backend Hash Computation

The backend computes the hash in `store_from_chunks`:

```rust
// src/backend/src/upload/blob_store.rs:89-106
let mut hasher = Sha256::new();
let mut total_written = 0u64;

// Stream chunks into blob store pages
let chunk_iter = session_store.iter_chunks(session_id, chunk_count);
for (page_idx, chunk_data) in chunk_iter.enumerate() {
    hasher.update(&chunk_data);  // <-- This is where the hash is computed
    total_written += chunk_data.len() as u64;
    // ... store chunk data
}

// Verify integrity
let actual_hash: [u8; 32] = hasher.finalize().into();
```

### Client Hash Computation

The client computes the hash using standard SHA256:

```bash
# Method 1: Direct file hash
shasum -a 256 /tmp/test_chunk.bin

# Method 2: Base64 decode then hash
echo -n "$CHUNK_DATA" | base64 -d | shasum -a 256

# Both methods produce: 134e6543ddc35b40abb4f2f8aaaa2d0513a27e267beaf9081e29d84eba94017d
```

## Root Cause Analysis

### Hypothesis 1: Chunk Data Corruption

The backend might be receiving different data than what the client sends.

**Investigation:**

- Client sends: 100 bytes of '0' characters (0x30)
- Backend receives: Unknown (need to verify)

### Hypothesis 2: Hash Algorithm Difference

The backend might be using a different hash algorithm or implementation.

**Investigation:**

- Backend uses: `sha2::Sha256` from Rust
- Client uses: `shasum -a 256` (OpenSSL SHA256)
- Both should produce identical results

### Hypothesis 3: Data Processing Difference

The backend might be processing the chunk data differently before hashing.

**Investigation:**

- Backend: `hasher.update(&chunk_data)` where `chunk_data` comes from `session_store.iter_chunks()`
- Client: Direct hash of the raw bytes
- Need to verify what `chunk_data` contains

### Hypothesis 4: Session Store Issue

The `session_store.iter_chunks()` might be returning different data than what was stored.

**Investigation:**

- Need to verify what data is actually stored in the session store
- Need to verify what data is returned by `iter_chunks()`

## Investigation Steps

### Step 1: Verify Chunk Data Storage

Add logging to see what data is actually stored in the session store:

```rust
// In put_chunk function
println!("Storing chunk {}: {:?}", chunk_idx, &bytes[..10]); // First 10 bytes

// In iter_chunks function
println!("Retrieved chunk {}: {:?}", page_idx, &chunk_data[..10]); // First 10 bytes
```

### Step 2: Verify Hash Computation

Add logging to see the hash computation process:

```rust
// In store_from_chunks function
let mut hasher = Sha256::new();
for (page_idx, chunk_data) in chunk_iter.enumerate() {
    println!("Hashing chunk {}: {:?}", page_idx, &chunk_data[..10]);
    hasher.update(&chunk_data);
}
let actual_hash: [u8; 32] = hasher.finalize().into();
println!("Computed hash: {}", hex::encode(actual_hash));
```

### Step 3: Compare with Client Hash

Verify that the client is computing the hash of the same data:

```bash
# Verify the exact data being hashed
echo -n "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" | shasum -a 256
# Should produce: 134e6543ddc35b40abb4f2f8aaaa2d0513a27e267beaf9081e29d84eba94017d
```

## Impact Assessment

### High Priority

- **Upload workflow broken** - Users cannot complete file uploads
- **Data integrity compromised** - Hash validation is failing
- **Test suite failures** - Upload tests cannot pass

### Affected Components

- `uploads_finish` function
- Upload workflow completion
- File upload test suite
- User file upload experience

## Questions for Senior Review

1. **Data Integrity**: Is the chunk data being stored and retrieved correctly in the session store?

2. **Hash Algorithm**: Are we using the correct SHA256 implementation? Should we use a different hash library?

3. **Data Processing**: Is there any data transformation happening between `put_chunk` and `store_from_chunks`?

4. **Session Store**: Is the `SessionStore.iter_chunks()` method returning the correct data?

5. **Candid Handling**: Is there any issue with how Candid handles the blob data in `uploads_put_chunk`?

## Proposed Solutions

### Option 1: Add Debug Logging

Add comprehensive logging to track data flow and hash computation:

```rust
// In put_chunk
println!("PUT_CHUNK: session_id={}, chunk_idx={}, data_len={}, first_10_bytes={:?}",
    session_id.0, chunk_idx, bytes.len(), &bytes[..10.min(bytes.len())]);

// In store_from_chunks
println!("STORE_FROM_CHUNKS: session_id={}, chunk_count={}, expected_len={}",
    session_id.0, chunk_count, expected_len);
```

### Option 2: Add Hash Verification

Add a test endpoint to verify hash computation:

```rust
#[ic_cdk::query]
fn debug_compute_hash(data: Vec<u8>) -> String {
    let hash = UploadService::compute_sha256(&data);
    hex::encode(hash)
}
```

### Option 3: Fix Data Processing

If the issue is in data processing, fix the root cause in the session store or blob store.

## Test Environment

- **Platform**: macOS 23.4.0
- **DFX Version**: Latest
- **Backend**: Deployed and running
- **Test Data**: 100 bytes of '0' characters (0x30)

## Expected vs Actual Hashes

| Method             | Hash                                                               |
| ------------------ | ------------------------------------------------------------------ |
| Client (shasum)    | `134e6543ddc35b40abb4f2f8aaaa2d0513a27e267beaf9081e29d84eba94017d` |
| Backend (computed) | `75b4538b54f44331bd862a76c245952095ebe421e9ee70a12cb4f1838b979b9b` |

## Next Steps

1. **Senior Developer Review**: Investigate the hash computation mismatch
2. **Add Debug Logging**: Track data flow through the upload process
3. **Verify Data Integrity**: Ensure chunk data is stored and retrieved correctly
4. **Fix Root Cause**: Address the underlying issue
5. **Test Validation**: Ensure all upload tests pass

## Related Files

- `src/backend/src/upload/blob_store.rs` - Hash computation logic
- `src/backend/src/upload/sessions.rs` - Session store implementation
- `src/backend/src/upload/service.rs` - Upload service logic
- `scripts/tests/backend/shared-capsule/upload/` - Test suite

## Priority

**CRITICAL** - This blocks the entire upload workflow and affects data integrity.

---

_Created: $(date)_  
_Status: Open_  
_Assigned: Senior Developer_
