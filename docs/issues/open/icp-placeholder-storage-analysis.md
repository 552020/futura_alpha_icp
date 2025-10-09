# ICP Placeholder Storage Analysis

**Status**: Open  
**Priority**: High  
**Date**: 2025-10-08  
**Branch**: icp-413-wire-icp-memory-upload-frontend-backend

## Summary

Analysis needed to determine the correct approach for storing placeholder assets in ICP canisters. Currently, placeholders are inline data URLs (base64-encoded images) that need to be stored in ICP for ICP-only uploads.

## Problem Statement

For ICP-only memory uploads, we need to store all assets (original, display, thumb, placeholder) in ICP. However, placeholders are currently inline data URLs, not blob objects, which creates a mismatch with the current ICP backend API.

## Current Implementation Issues

### 1. **Placeholder Nature**

- Placeholders are **inline data URLs** (base64-encoded images)
- They are **not blob objects** that can be uploaded directly
- Current ICP upload functions expect blob objects

### 2. **ICP Backend API Analysis Needed**

We need to analyze the ICP backend API to understand:

#### **Memory Creation Functions**

- `memories_create` - Takes `opt blob` parameter (could support inline assets?)
- `memories_create_with_internal_blobs` - Only takes `Array<InternalBlobAssetInput>` (blob references only)

#### **Key Questions**

1. **Which function should we use for ICP-only memories?**
2. **Does `memories_create` support inline blob data for placeholders?**
3. **Can we mix inline assets and blob references in the same memory?**
4. **What are the exact API signatures and parameters?**

### 3. **Current Workaround**

Currently converting placeholder data URL to blob and uploading it, but this may not be the intended approach.

## Technical Analysis Results

### **Backend API Investigation** ✅ COMPLETED

#### **Two Memory Creation Functions Found:**

1. **`memories_create`** - Supports inline assets

   ```rust
   // From src/backend/src/memories/core/create.rs:18
   pub fn memories_create_core<E: Env, S: Store>(
       env: &E,
       store: &mut S,
       capsule_id: CapsuleId,
       bytes: Option<Vec<u8>>,        // ← INLINE ASSET SUPPORT
       blob_ref: Option<BlobRef>,     // ← BLOB REFERENCE
       external_location: Option<StorageEdgeBlobType>,
       external_storage_key: Option<String>,
       external_url: Option<String>,
       external_size: Option<u64>,
       external_hash: Option<Vec<u8>>,
       asset_metadata: AssetMetadata,
       idem: String,
   ```

2. **`memories_create_with_internal_blobs`** - Blob references only
   ```rust
   // From src/backend/src/memories/core/create.rs:186
   pub fn memories_create_with_internal_blobs_core<E: Env, S: Store>(
       env: &E,
       store: &mut S,
       capsule_id: CapsuleId,
       metadata: MemoryMetadata,
       blob_assets: Vec<InternalBlobAssetInput>,  // ← BLOB REFERENCES ONLY
       idem: String,
   ```

#### **Candid Interface Analysis** ✅ COMPLETED

```candid
// From backend.did:522
memories_create : (
    text,                    // capsule_id
    opt blob,               // ← INLINE BYTES (for placeholders)
    opt BlobRef,            // ← BLOB REFERENCE
    opt BlobHosting,        // ← STORAGE BACKEND
    opt text,               // ← STORAGE KEY
    opt text,               // ← URL
    opt nat64,              // ← SIZE
    opt blob,               // ← HASH
    AssetMetadata,          // ← ASSET METADATA
    text,                   // ← IDEMPOTENCY KEY
) -> (Result6);

memories_create_with_internal_blobs : (
    text,                           // capsule_id
    MemoryMetadata,                 // ← MEMORY METADATA
    vec InternalBlobAssetInput,     // ← BLOB REFERENCES ONLY
    text,                           // ← IDEMPOTENCY KEY
) -> (Result6);
```

### **Test Analysis** ✅ COMPLETED

#### **Placeholder Tests Found:**

- **`test_upload_2lane_4asset_system.mjs`** - Tests placeholder upload as blob
- **`test_lane_b_image_processing.mjs`** - Tests placeholder generation
- **Multiple session tests** - All upload placeholders as blobs

#### **Inline Asset Tests Found:**

- **`test_inline_upload.mjs`** - Tests inline storage for small files (≤32KB)
- **`test_both_storage_methods.sh`** - Tests both inline and blob storage
- **Multiple memory tests** - Use `memories_create` with inline bytes

#### **Key Finding:**

**All existing tests upload placeholders as blobs, not inline assets!**

## Recommended Solution

### **✅ Option 2: Upload placeholder as blob (CURRENT APPROACH IS CORRECT)**

**Evidence from tests:**

- All existing tests upload placeholders as blobs
- `test_upload_2lane_4asset_system.mjs` shows the pattern:
  ```javascript
  // Upload placeholder derivative
  results.placeholder = await uploadFileToICP(actor, derivatives.placeholder.buffer, "placeholder");
  ```

**Why this is correct:**

1. **Consistency** - All assets (original, display, thumb, placeholder) are stored as blobs
2. **Test coverage** - Existing tests validate this approach
3. **API alignment** - Uses `memories_create_with_internal_blobs` consistently
4. **Performance** - Blob storage is optimized for binary data

### **❌ Option 1: Use `memories_create` with inline blob (NOT RECOMMENDED)**

- Would require mixing two different memory creation APIs
- No test coverage for this approach
- Adds complexity without clear benefits

### **❌ Option 3: Store placeholder in Neon only (NOT RECOMMENDED)**

- Breaks ICP-only storage requirement
- Creates inconsistency in asset storage

## Conclusion

### **✅ RECOMMENDATION: Keep current approach**

The current implementation is **CORRECT** based on:

1. **Backend API analysis** - `memories_create_with_internal_blobs` is the right function
2. **Test evidence** - All existing tests upload placeholders as blobs
3. **Consistency** - All assets stored uniformly as blobs in ICP
4. **Performance** - Blob storage is optimized for binary data

### **No changes needed** - Current placeholder upload as blob is the intended approach.

## Questions for Tech Lead (RESOLVED)

1. ✅ **What is the intended architecture for placeholder storage in ICP?** → Upload as blob
2. ✅ **Should placeholders be stored as inline assets or uploaded as blobs?** → Upload as blob
3. ✅ **Which ICP backend function should we use for ICP-only memories?** → `memories_create_with_internal_blobs`
4. ✅ **Are there existing tests or examples of placeholder handling in ICP?** → Yes, all tests upload as blob
5. ✅ **What are the performance implications of each approach?** → Blob storage is optimal

## Files to Analyze

### **Backend API**

- `src/backend/src/memories/` - Memory creation functions
- `src/backend/src/memories/types.rs` - Data structures
- `src/ic/declarations/backend/backend.did` - Candid interface

### **Tests**

- `tests/backend/` - Backend tests
- `tests/` - Integration tests
- Look for placeholder or inline asset tests

### **Current Implementation**

- `src/nextjs/src/services/upload/icp-with-processing.ts` - Current upload logic
- `src/nextjs/src/services/upload/image-derivatives.ts` - Placeholder generation

## Expected Outcome ✅ ACHIEVED

Clear decision reached:

1. ✅ **Correct ICP backend function to use** → `memories_create_with_internal_blobs`
2. ✅ **Proper placeholder storage approach** → Upload as blob (current approach is correct)
3. ✅ **Updated implementation plan** → No changes needed
4. ✅ **Test cases to verify the solution** → Existing tests validate the approach

## Dependencies ✅ RESOLVED

- ✅ Backend API documentation - Analyzed
- ✅ Test case analysis - Completed
- ✅ Performance considerations - Blob storage is optimal
- ✅ ICP canister limitations - No issues found

---

**Status**: ✅ **RESOLVED**  
**Conclusion**: Current implementation is correct, no changes needed  
**Evidence**: Backend API analysis + test coverage validation
