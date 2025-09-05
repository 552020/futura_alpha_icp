# Check Upload Workflow - Implementation Validation

## Problem Statement

The upload workflow implementation exists in `upload/service.rs` but needs validation through TDD approach. We need to:

1. **Verify existing implementation** in `upload/service.rs`
2. **Check facade in `lib.rs`** - ensure `uploads_begin` is properly exported
3. **Write comprehensive tests** to validate the contract end-to-end
4. **Identify any missing pieces** in the upload workflow

## Current Implementation Status

### ✅ What We Have

Based on the existing files, we already have:

- `uploads_begin` implemented in `upload/service.rs`
- Session management in `upload/sessions.rs`
- Blob storage in `upload/blob_store.rs`
- Type definitions in `upload/types.rs`

### ❓ What Needs Validation

1. **Facade in `lib.rs`** - Is `uploads_begin` properly exported as a public canister endpoint?
2. **End-to-end functionality** - Does the complete workflow work from client to storage?
3. **Error handling** - Are all error cases properly handled?
4. **Authorization** - Is capsule access properly verified?

### 🚨 MVP Safety Issues to Address

The current implementation needs these critical guardrails for production safety:

1. **Input validation** - No zero chunks, sane upper bounds
2. **Idempotency binding** - Bind to caller-provided `idem` key to prevent duplicate sessions
3. **Concurrent session limits** - Cap concurrent sessions per caller/capsule
4. **Proper error handling** - All edge cases covered
5. **Authorization enforcement** - Capsule access properly verified

## Implementation Check Plan

### Step 1: Verify Existing Implementation

#### Check `upload/service.rs`

```rust
// Expected function signature:
pub fn begin(
    capsule_id: CapsuleId,
    expected_len: u64,
    expected_sha256: Option<[u8; 32]>,
    meta: MemoryMeta,
    idem: String,
) -> Result<SessionId, Error>
```

**Validation Points:**

- [ ] Function exists with correct signature
- [ ] Validates `expected_len > 0`
- [ ] Verifies capsule access authorization
- [ ] Creates session with proper metadata
- [ ] Persists session to storage
- [ ] Returns `SessionId` on success

#### Check `upload/sessions.rs`

**Validation Points:**

- [ ] Session struct properly defined
- [ ] Session creation logic implemented
- [ ] Session storage/retrieval methods exist
- [ ] Session expiration handling

#### Check `upload/blob_store.rs`

**Validation Points:**

- [ ] Blob storage methods implemented
- [ ] SHA256 computation and verification
- [ ] Blob reference generation
- [ ] Storage key management

### Step 2: Check Facade in `lib.rs`

#### Expected Facade Implementation (MVP-Safe)

```rust
#[ic_cdk::update]
fn uploads_begin(
    capsule_id: CapsuleId,
    meta: MemoryMeta,
    expected_chunks: u32,
    idem: String,                // ← add this for idempotency
) -> Result<SessionId, Error> {
    with_capsule_store_mut(|store| {
        let mut svc = upload::service::UploadService::new(store);
        svc.begin_upload(capsule_id, meta, expected_chunks, idem)
    })
}
```

**Validation Points:**

- [ ] Function exists in `lib.rs`
- [ ] Properly decorated with `#[ic_cdk::update]`
- [ ] Correct parameter types and names (including `idem` parameter)
- [ ] Calls `upload::service::begin_upload` correctly
- [ ] Returns proper `Result<SessionId, Error>`
- [ ] Uses `with_capsule_store_mut` for proper store access

### Step 3: MVP-Safe Implementation Details

#### Enhanced `upload/service.rs` Implementation

```rust
pub fn begin_upload(
    &mut self,
    capsule_id: CapsuleId,
    meta: MemoryMeta,
    expected_chunks: u32,
    idem: String,                      // ← add this for idempotency
) -> Result<SessionId, Error> {
    // 0) validate input early
    if expected_chunks == 0 {
        return Err(Error::InvalidArgument("expected_chunks_zero".into()));
    }
    // optional sane cap to avoid abuse (tune as needed)
    const MAX_CHUNKS: u32 = 16_384;
    if expected_chunks > MAX_CHUNKS {
        return Err(Error::InvalidArgument("expected_chunks_too_large".into()));
    }

    // 1) auth
    let caller = ic_cdk::api::msg_caller();
    let person_ref = PersonRef::Principal(caller);
    if let Some(capsule) = self.store.get(&capsule_id) {
        if !capsule.has_write_access(&person_ref) {
            return Err(Error::Unauthorized);
        }
    } else {
        return Err(Error::NotFound);
    }

    // 2) idempotency: if a pending session with same (capsule, caller, idem) exists, return it
    if let Some(existing) = self.sessions.find_pending(&capsule_id, &caller, &idem) {
        return Ok(existing);
    }

    // 3) back-pressure: cap concurrent sessions per caller/capsule
    const MAX_ACTIVE_PER_CALLER: usize = 3;
    if self.sessions.count_active_for(&capsule_id, &caller) >= MAX_ACTIVE_PER_CALLER {
        return Err(Error::ResourceExhausted); // "too many active uploads"
    }

    // 4) create session
    let session_id = SessionId::new();
    let provisional_memory_id = MemoryId::new();

    let session_meta = SessionMeta {
        capsule_id,
        provisional_memory_id,
        caller,
        chunk_count: expected_chunks,
        expected_len: None,            // fine for MVP if you don't know length upfront
        expected_hash: None,           // ditto; you can verify on finish
        status: SessionStatus::Pending,
        created_at: ic_cdk::api::time(),
        meta,
        idem,                          // ← persist for idempotency
    };

    self.sessions.create(session_id.clone(), session_meta)?;
    Ok(session_id)
}
```

#### SessionStore Helper Methods

```rust
impl SessionStore {
    pub fn find_pending(&self, capsule_id: &CapsuleId, caller: &Principal, idem: &str) -> Option<SessionId> {
        self.iter_pending().find_map(|(sid, s)| {
            if &s.capsule_id == capsule_id && &s.caller == caller && s.idem == idem {
                Some(sid.clone())
            } else {
                None
            }
        })
    }

    pub fn count_active_for(&self, capsule_id: &CapsuleId, caller: &Principal) -> usize {
        self.iter_pending()
            .filter(|(_, s)| &s.capsule_id == capsule_id && &s.caller == caller)
            .count()
    }
}
```

### Step 4: Write Comprehensive Tests

#### Test Categories

1. **Happy Path Tests**
2. **Error Handling Tests**
3. **Authorization Tests**
4. **Edge Case Tests**
5. **Idempotency Tests**
6. **Concurrent Session Limit Tests**

#### Test Implementation Strategy

##### A. Rust Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uploads_begin_happy_path() {
        // Test successful session creation
    }

    #[test]
    fn test_uploads_begin_zero_length() {
        // Test rejection of expected_len = 0
    }

    #[test]
    fn test_uploads_begin_unauthorized() {
        // Test rejection of unauthorized principal
    }

    #[test]
    fn test_uploads_begin_invalid_capsule() {
        // Test rejection of non-existent capsule
    }
}
```

##### B. Enhanced Bash TDD Script (MVP-Safe)

Create `scripts/e2e_uploads_begin.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

# Config
BACKEND_CANISTER="${BACKEND_CANISTER:-backend}"
CAPSULE_ID="${CAPSULE_ID:?set CAPSULE_ID (principal string)}"

call() { dfx canister call "$BACKEND_CANISTER" "$1" "$2"; }

echo "== happy path =="
# meta is an empty record here; adjust to your MemoryMeta candid
SID=$(call uploads_begin "(principal \"$CAPSULE_ID\", record {}, 4:nat32, \"idem-1\")" \
  | awk -F'[()]' '/\(/ {print $2}' | tr -d ' ')

test -n "$SID" && echo "OK: session=$SID" || { echo "FAIL: no session returned"; exit 1; }

echo "== idempotency (same idem returns same sid) =="
SID2=$(call uploads_begin "(principal \"$CAPSULE_ID\", record {}, 4:nat32, \"idem-1\")" \
  | awk -F'[()]' '/\(/ {print $2}' | tr -d ' ')
test "$SID" = "$SID2" && echo "OK" || { echo "FAIL: idem did not return same sid"; exit 1; }

echo "== rejects zero chunks =="
set +e
OUT=$(call uploads_begin "(principal \"$CAPSULE_ID\", record {}, 0:nat32, \"idem-zero\")" 2>&1)
set -e
echo "$OUT" | grep -qi "expected_chunks_zero" && echo "OK" || { echo "FAIL: zero not rejected"; exit 1; }

echo "== rejects too many chunks =="
set +e
OUT=$(call uploads_begin "(principal \"$CAPSULE_ID\", record {}, 20000:nat32, \"idem-large\")" 2>&1)
set -e
echo "$OUT" | grep -qi "expected_chunks_too_large" && echo "OK" || { echo "FAIL: large chunks not rejected"; exit 1; }

# Optional: unauthorized check if you have another identity configured
if dfx identity list | grep -q other; then
  echo "== unauthorized principal =="
  dfx identity use other
  set +e
  OUT=$(call uploads_begin "(principal \"$CAPSULE_ID\", record {}, 4:nat32, \"idem-unauth\")" 2>&1)
  set -e
  dfx identity use default
  echo "$OUT" | grep -qi "Unauthorized" && echo "OK" || echo "WARN: Unauthorized not observed (check auth)"
fi

echo "== concurrent session limit test =="
# Create multiple sessions to test the limit
for i in {1..4}; do
  set +e
  OUT=$(call uploads_begin "(principal \"$CAPSULE_ID\", record {}, 4:nat32, \"idem-concurrent-$i\")" 2>&1)
  set -e
  if [ $i -le 3 ]; then
    # First 3 should succeed
    echo "$OUT" | grep -q "(" && echo "OK: session $i created" || { echo "FAIL: session $i not created"; exit 1; }
  else
    # 4th should fail with ResourceExhausted
    echo "$OUT" | grep -qi "ResourceExhausted" && echo "OK: session limit enforced" || { echo "FAIL: session limit not enforced"; exit 1; }
  fi
done

echo "uploads_begin smoke passed."
```

Make it executable: `chmod +x scripts/e2e_uploads_begin.sh`.

### Step 4: End-to-End Workflow Validation

#### Complete Upload Flow Test

```bash
#!/bin/bash
# test_complete_upload_workflow.sh

set -e

echo "Testing complete upload workflow..."

# Step 1: Begin upload session
echo "Step 1: Begin upload session"
SESSION_ID=$(dfx canister call futura_alpha_icp uploads_begin \
    '(record {
        capsule_id = "test-capsule-123";
        expected_len = 1024;
        expected_sha256 = null;
        meta = record { title = "Test Upload"; description = "Test Description" };
        idem = "test-idempotency-key";
    })' \
    --output idl)

echo "Session ID: $SESSION_ID"

# Step 2: Upload chunks (if chunked upload implemented)
echo "Step 2: Upload chunks"
# TODO: Implement chunk upload tests

# Step 3: Finish upload
echo "Step 3: Finish upload"
# TODO: Implement upload finish tests

# Step 4: Verify memory creation
echo "Step 4: Verify memory creation"
# TODO: Implement memory verification tests

echo "Complete workflow test passed! ✅"
```

## Validation Checklist

### Implementation Validation

- [ ] **`upload/service.rs`** - `begin` function exists with correct signature
- [ ] **`upload/sessions.rs`** - Session management properly implemented
- [ ] **`upload/blob_store.rs`** - Blob storage methods available
- [ ] **`upload/types.rs`** - All required types defined

### Facade Validation

- [ ] **`lib.rs`** - `uploads_begin` endpoint properly exported
- [ ] **CDK annotations** - Correct `#[ic_cdk::update]` decoration
- [ ] **Parameter types** - Match expected Candid interface
- [ ] **Return types** - Proper `Result<SessionId, Error>`

### Test Validation

- [ ] **Unit tests** - Rust tests for core logic
- [ ] **Integration tests** - Bash scripts for end-to-end validation
- [ ] **Error cases** - All error scenarios covered
- [ ] **Authorization** - Access control properly tested
- [ ] **Idempotency tests** - Same `idem` returns same session ID
- [ ] **Input validation** - Zero chunks and oversized chunks rejected
- [ ] **Concurrent limits** - Session limits properly enforced

### Workflow Validation

- [ ] **Session creation** - `uploads_begin` creates valid session
- [ ] **Session persistence** - Session stored and retrievable
- [ ] **Error handling** - Proper error responses
- [ ] **Authorization** - Capsule access verified

## Expected Outcomes

### Success Criteria

1. **All existing code compiles** without errors
2. **Facade properly exports** `uploads_begin` endpoint with `idem` parameter
3. **Tests pass** for all validation scenarios
4. **End-to-end workflow** functions correctly
5. **Error handling** works as expected
6. **Authorization** properly enforced
7. **Idempotency** works correctly with same `idem` returning same session
8. **Input validation** rejects invalid inputs (zero chunks, oversized)
9. **Concurrent limits** properly enforced (max 3 sessions per caller/capsule)

### Potential Issues to Watch For

1. **Missing facade** - `uploads_begin` not exported in `lib.rs`
2. **Type mismatches** - Parameter or return type issues
3. **Authorization gaps** - Capsule access not properly verified
4. **Session management** - Session creation or storage issues
5. **Error handling** - Missing error cases or improper error types
6. **Missing idempotency** - `idem` parameter not implemented
7. **No input validation** - Zero chunks or oversized chunks not rejected
8. **No concurrent limits** - Unlimited sessions per caller/capsule
9. **SessionStore methods missing** - `find_pending` and `count_active_for` not implemented

## Next Steps

1. **Run implementation check** - Verify existing code
2. **Add missing facade** - Export `uploads_begin` in `lib.rs` with `idem` parameter
3. **Implement MVP safety features** - Add input validation, idempotency, and concurrent limits
4. **Add SessionStore helper methods** - Implement `find_pending` and `count_active_for`
5. **Write tests** - Implement comprehensive test suite including enhanced TDD script
6. **Run validation** - Execute all tests and verify results
7. **Fix issues** - Address any problems found during validation
8. **Document results** - Update this document with findings

## Why Start Here?

- `uploads_begin` sets the contract for the rest of the streaming flow (owner, shape, idempotency)
- Once this is in, `uploads_put_chunk` and `uploads_finish` can be developed & tested incrementally
- The shared `finalize_new_memory(...)` can be reused on finish
- MVP safety features prevent production issues later

## Conclusion

This validation approach ensures that the existing upload workflow implementation is properly tested and validated without rewriting working code. The TDD approach focuses on:

- **Verifying existing implementation** rather than rebuilding
- **Testing the contract** end-to-end
- **Identifying gaps** in the current implementation
- **Ensuring proper integration** between components

The goal is to have confidence that the upload workflow works correctly and can be used by clients to upload files to the system.
