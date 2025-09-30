# Upload API Blob Format Confusion - Need Senior Developer Guidance

## Issue Summary

We're experiencing a **hash mismatch** in our upload system where the backend receives base64 strings instead of decoded bytes, even when using the `blob` type in Candid calls.

## Current API Design

Our current upload API in `.did` file:

```candid
service {
  uploads_begin      : (text, AssetMetadata, nat32, text) -> (Result_12);
  uploads_put_chunk  : (nat64, nat32, blob) -> (Result);
  uploads_finish     : (nat64, blob, nat64) -> (Result_5);
}
```

**Parameters:**

- `uploads_begin(capsule_id, asset_metadata, chunk_count, idempotency_key)`
- `uploads_put_chunk(session_id, chunk_idx, data)` - where `data` is `blob`
- `uploads_finish(session_id, sha256_hash, total_length)` - where `sha256_hash` is `blob`

## The Problem

**Expected behavior**: When we send `blob "QQ=="` (base64 for "A"), the backend should receive the decoded byte `[65]` (ASCII for "A").

**Actual behavior**: The backend receives the base64 string `[81, 81, 61, 61]` (ASCII for "QQ==").

## Evidence from Backend Logs

```
[21. 2025-09-30T19:53:37.884726Z]: PUT_CHUNK: session_id=10, chunk_idx=0, data_len=4, first_10_bytes=[81, 81, 61, 61]
[22. 2025-09-30T19:53:39.136483Z]: STORE_FROM_CHUNKS: session_id=10, page_idx=0, data_len=4, first_10_bytes=[81, 81, 61, 61]
```

**Analysis:**

- `[81, 81, 61, 61]` = ASCII for `"QQ=="` (base64 string)
- `[65]` = ASCII for `"A"` (the actual byte we want)

## Our Client Implementation

We're using this shell script approach:

```bash
# 1) Encode data to base64
chunk_b64=$(printf %s "A" | base64 | tr -d '\n')  # Results in "QQ=="

# 2) Send as blob
dfx canister call backend uploads_put_chunk "($session_id, 0:nat32, blob \"$chunk_b64\")"
```

## Hash Mismatch Result

- **Our expected hash**: `559aead08264d5795d3909718cdd05abd49572e84fe55590eef31a88a08fdffd` (SHA256 of "A")
- **Backend actual hash**: `ee0b13692453f0f83c3c9bfa207ef7a6b1927f6dedaf5d900239e1b17762b3ea` (SHA256 of "QQ==")

## Questions for Senior Developer

1. **Is our API design correct?** Should `uploads_put_chunk` accept `blob` type for chunk data?

2. **Is our client implementation correct?** Should we use `blob "QQ=="` or a different format?

3. **Is there a backend bug?** Should the backend automatically decode base64 when receiving `blob` type?

4. **What's the canonical way** to send raw bytes via Candid `blob` type?

## Alternative Approaches We've Tried

1. **Using `vec nat8` format**: `vec { 0x41; }` for byte "A" - still getting base64 strings
2. **Using debug endpoints**: `debug_put_chunk_b64` - works but bypasses normal flow
3. **Different base64 encoding**: `echo -n`, `printf %s` - same result

## Expected Resolution

We need clarity on:

- **Correct Candid format** for sending raw bytes
- **Backend expectations** for `blob` type handling
- **Whether our API design** needs changes

## Diagnostic Checklist Results

Following your diagnostic checklist:

1. ✅ **Deployed .did confirmed**: `uploads_put_chunk : (nat64, nat32, blob) -> (Result)`
2. ✅ **Rust signature confirmed**: `bytes: Vec<u8>` (not `String`)
3. ✅ **No text conversions found**: No `String::from_utf8`, `base64::encode/decode`, or `.as_bytes()` in upload paths
4. ❌ **Smoke test blocked**: Can't test with raw bytes because session doesn't exist (authorization fails)

## The Mystery

- **Deployed interface**: Correct (`blob` type)
- **Rust signature**: Correct (`Vec<u8>` type)
- **No text conversions**: Found none in upload code paths
- **Backend logs**: Still show `[81, 81, 61, 61]` ("QQ==") instead of `[65]` ("A")

## What I Need Help With

I've confirmed all the checkpoints in your diagnostic checklist, but the issue persists. Where else should I look? Is there:

- A Candid middleware/wrapper I'm missing?
- An issue with how `dfx` encodes `blob` literals?
- A bug in the IC Candid deserialization?
- Something else I'm not seeing?

## Alternative Approach: IC Agent and Actor for Proper Serialization

**Question**: Are we hitting a wall because of bash/Candid text encoding issues?

We already have a working Node.js upload agent (`tests/backend/shared-capsule/upload/ic-upload.mjs`) that successfully uploads files using **proper IC agent and actor serialization**. Should we:

1. **Test with Node.js agent** to bypass shell encoding issues?
2. **Use the existing `ic-upload.mjs`** as a reference for proper blob handling?
3. **Compare what the Node.js agent sends** vs. what our shell script sends?

The Node.js agent uses **proper IC serialization**:

```javascript
// Create IC agent with proper serialization
const agent = new HttpAgent({ host: HOST, fetch });
const backend = Actor.createActor(idlFactory, { agent, canisterId: CANISTER_ID });

// Send raw bytes with proper IC serialization
const chunk = new Uint8Array(buf.subarray(0, read));
const put = await backend.uploads_put_chunk(session, index, chunk);
```

This uses the **IC agent and actor** for proper Candid serialization, which:

- ✅ **Handles binary data correctly** (no base64 encoding/decoding issues)
- ✅ **Uses proper Candid serialization** (not shell text encoding)
- ✅ **Sends `Uint8Array` directly** (becomes `Vec<u8>` in Rust)
- ✅ **Bypasses shell escaping issues** (no quote/newline problems)

**Hypothesis**: The issue might be in how `dfx` handles `blob` literals in shell commands vs. proper IC agent serialization. The Node.js agent uses the **official IC serialization** while our shell script relies on `dfx`'s text-based Candid parsing.

## BREAKTHROUGH: Root Cause Identified! 🎯

**The issue is `dfx`'s text-based Candid parsing vs. proper IC serialization.**

### Evidence from Testing

**Node.js Agent (Proper IC Serialization):**

```javascript
const put = await backend.uploads_put_chunk(session, 0, new Uint8Array(dataBuffer));
```

- ✅ **Backend logs**: `data_len=1, first_10_bytes=[65]` (byte "A")
- ✅ **Hash matches**: `559aead08264d5795d3909718cdd05abd49572e84fe55590eef31a88a08fdffd`
- ✅ **Upload succeeds**: `mem_1759263546500382000`

**Shell Script (dfx text-based parsing):**

```bash
dfx canister call backend uploads_put_chunk "(0, 0:nat32, blob \"QQ==\")"
```

- ❌ **Backend logs**: `data_len=4, first_10_bytes=[81, 81, 61, 61]` (base64 string "QQ==")
- ❌ **Hash mismatch**: Backend hashes "QQ==" instead of "A"
- ❌ **Upload fails**: `checksum_mismatch`

### Root Cause

The issue is **NOT** in the backend or API design. The problem is that `dfx`'s text-based Candid parsing doesn't properly handle `blob` literals - it sends the base64 string instead of the decoded bytes.

### Solution

**Use Node.js agent with proper IC serialization** for binary data handling instead of relying on `dfx`'s text-based Candid parsing.

## Current Status

- ✅ **Backend logging** shows exact bytes received
- ✅ **Client utilities** are hardened and portable
- ✅ **Deployed interface** matches source code
- ✅ **Rust signatures** are correct
- ✅ **No text conversions** in upload paths
- ✅ **Root cause identified**: dfx text-based parsing issue
- ✅ **Solution confirmed**: Node.js agent with proper IC serialization works

## Priority

**RESOLVED** - Root cause identified and solution confirmed. The Node.js agent approach solves the hash mismatch issue.

---

**Context**: We're a greenfield project, so we can change the API if needed. We just need to understand the correct approach for handling binary data in Candid.

**Recommendation**: Use Node.js agent with proper IC serialization for all binary data handling instead of shell scripts with `dfx`.
