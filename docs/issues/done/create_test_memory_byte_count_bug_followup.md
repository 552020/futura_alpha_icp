# Follow-up: `create_test_memory` byte count bug - Implementation attempted but still failing

## What We Tried

We implemented the suggested fix from the senior developer, but the error persists:

```
InvalidArgument = "inline bytes_len != metadata.base.bytes"
```

## Implementation Details

### 1. Added Helper Functions

We added the three helper functions to `tests/backend/test_utils.sh`:

```bash
# Extract pure base64 from Candid blob literal
extract_b64() {
  local s="$1"
  s="$(printf %s "$s" | tr -d '\r\n')"
  if [[ "$s" =~ ^blob ]]; then
    s="${s#blob }"
    s="${s#\"}"
    s="${s%\"}"
    printf %s "$s"
  else
    printf %s "$s"
  fi
}

# Decoded byte length from base64
b64_decoded_len() {
  local b64="$(extract_b64 "$1" | tr -d '\n\r\t ')"
  local n
  n=$( { printf %s "$b64" | base64 -d 2>/dev/null || printf %s "$b64" | base64 -D 2>/dev/null; } | wc -c | awk '{print $1}' )
  if [[ -n "$n" && "$n" -ge 0 ]]; then
    printf %s "$n"
    return
  fi
  # Fallback formula
  local L=${#b64}
  local pads=0
  [[ "$b64" =~ ==$ ]] && pads=2 || { [[ "$b64" =~ =$ ]] && pads=1; }
  printf %s $(( (L/4)*3 - pads ))
}

# Count bytes from vec nat8 literal
vec_nat8_len() {
  local s="$1"
  printf %s "$s" | grep -Eo '0x[0-9a-fA-F]{2}' | wc -l | awk '{print $1}'
}
```

### 2. Updated create_test_memory Function

We replaced the buggy line 369 with:

```bash
# Calculate the correct decoded byte count
local bytes_len
if [[ "$memory_bytes" =~ ^[[:space:]]*vec[[:space:]]*\{ ]]; then
    bytes_len=$(vec_nat8_len "$memory_bytes")
else
    bytes_len=$(b64_decoded_len "$memory_bytes")
fi

local asset_metadata=$(create_document_asset_metadata "$name" "$description" "$tags" "$bytes_len")
```

## Verification Tests

### Helper Function Tests

```bash
# Test extract_b64
extract_b64 'blob "SGVsbG8gV29ybGQ="'
# Output: SGVsbG8gV29ybGQ= ✅

# Test b64_decoded_len
b64_decoded_len 'blob "SGVsbG8gV29ybGQ="'
# Output: 11 ✅ (correct for "Hello World")

# Test create_document_asset_metadata
create_document_asset_metadata "test" "Test" '"test"' "11"
# Output: bytes = 11; ✅
```

### Full API Test

```bash
capsule_id="capsule_1759268213340323000"
memory_bytes='blob "SGVsbG8gV29ybGQ="'
bytes_len=$(b64_decoded_len "$memory_bytes")  # Returns 11
asset_metadata=$(create_document_asset_metadata "test" "Test" '"test"' "$bytes_len")
# Metadata shows: bytes = 11;

# API call still fails:
dfx canister call --identity default backend memories_create "(\"$capsule_id\", opt $memory_bytes, null, null, null, null, null, null, $asset_metadata, \"$idem\")"
# Result: InvalidArgument = "inline bytes_len != metadata.base.bytes"
```

## The Mystery

Even though our calculations are correct:

- Base64: `"SGVsbG8gV29ybGQ="`
- Decoded bytes: `11` (verified with `echo -n "SGVsbG8gV29ybGQ=" | base64 -d | wc -c`)
- Metadata bytes field: `11`
- The API still reports a mismatch

## Questions for Senior Developer

1. **Is there a caching issue?** The canister might be using old code or data.

2. **Are we missing something in the API call?** The error suggests the canister is comparing two different byte counts.

3. **Should we check the actual blob data being sent?** Maybe the issue is in how `dfx` is sending the blob data.

4. **Is there a different validation happening?** The error message suggests the canister is validating `inline bytes_len` against `metadata.base.bytes` - are these the right fields?

5. **Should we try a different approach?** Maybe we need to use a working test as a template and modify it step by step.

## Current Status

- ✅ Helper functions implemented and tested
- ✅ Byte count calculation verified (11 bytes for "Hello World")
- ✅ Metadata generation verified (bytes = 11)
- ❌ API call still fails with same error
- ❌ All tests using `create_test_memory` still fail

## Next Steps Requested

Could you please:

1. **Verify our implementation** is correct
2. **Suggest debugging steps** to identify where the mismatch occurs
3. **Provide a working example** if possible
4. **Check if there are other validation rules** we're missing

The fix looks correct in theory, but something is still causing the byte count mismatch at the canister level.
