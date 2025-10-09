# ICP Memory Title "Placeholder" Display Issue

## Issue Summary

**Problem**: When uploading images to ICP, the dashboard displays "placeholder" as the memory title instead of the actual image title or filename.

**Status**: `OPEN` - Bug Investigation Required  
**Priority**: `HIGH` - User Experience Issue  
**Assigned**: Tech Lead  
**Created**: 2024-12-19

## Problem Description

### User Experience

- User uploads an image file (e.g., `vacation-photo.jpg`)
- Image is successfully uploaded to ICP
- Dashboard displays the memory with title "placeholder" instead of the filename or a proper title
- This creates confusion and poor user experience

### Debug Evidence

**Actual ICP Memory Data from `memories_list_by_capsule`:**

```javascript
memories_list_by_capsule result: {
  Ok: {
    items: [{
      access: {Private: {...}},
      asset_count: 1,
      capsule_id: "capsule_1760009961375823001",
      created_at: 1760010062140007001n,
      description: [],
      has_previews: true,
      has_thumbnails: false,
      id: "0199c8c6-493c-7c92-99be-433400003c92",
      is_public: false,
      memory_type: {Image: null},
      name: "placeholder",                    // ← PROBLEM: Hardcoded "placeholder"
      parent_folder_id: [],
      primary_asset_url: [],
      shared_count: 0,
      sharing_status: "private",
      size: 1000n,
      tags: [],
      thumbnail_url: [],
      title: ["placeholder"],                 // ← PROBLEM: Array with "placeholder"
      updated_at: 1760010062140007001n
    }]
  }
}
```

**Key Issues Identified:**

- `name: "placeholder"` - The memory name is hardcoded to "placeholder"
- `title: ["placeholder"]` - The title array contains "placeholder" string
- No filename extraction or user-provided title is being used

### Expected Behavior

- Memory should display the original filename as title (e.g., "vacation-photo.jpg")
- Or display a user-friendly title if one was provided during upload
- Or display "Untitled" if no title is available

## Technical Analysis

### Current Title Handling Flow

#### 1. ICP Backend Memory Creation

```rust
// From backend/src/memories/types.rs
pub struct MemoryMetadata {
    pub title: Option<String>,       // Optional title
    pub description: Option<String>, // Optional description
    // ... other fields
}

pub struct MemoryHeader {
    pub title: Option<String>,       // From metadata
    pub name: String,               // Memory name
    // ... other fields
}
```

#### 2. Frontend Title Transformation

```typescript
// From src/nextjs/src/services/memories.ts
const transformICPMemoryHeaderToNeon = (header: MemoryHeader): MemoryWithFolder => {
  return {
    // ...
    title: (header.title.length > 0 ? header.title[0] : null) || header.name || "Untitled",
    // ...
  };
};
```

#### 3. Title Display Logic

```typescript
// From src/nextjs/src/components/common/content-card.tsx
function renderTitle(item: FlexibleItem) {
  if ("title" in item) {
    return shortenTitle(item.title);
  }
  if ("memory" in item && item.memory.title) {
    return item.memory.title;
  }
  return "Untitled";
}
```

### Root Cause Analysis

#### 1. ICP Memory Creation - WRONG API CALL

**ROOT CAUSE FOUND**: The frontend is calling the wrong ICP backend API function:

```typescript
// From src/nextjs/src/services/upload/icp-with-processing.ts
// ❌ WRONG: Using memories_create (no title parameter)
const result = await backend.memories_create(
  capsuleId,
  [placeholderBytes], // inline bytes
  [] // no blob ref
  // ... other parameters
  // ❌ NO TITLE PARAMETER - this API doesn't accept title!
);

// ✅ CORRECT: Should use memories_create_with_internal_blobs
const result = await backend.memories_create_with_internal_blobs(
  capsuleId,
  memoryMetadata, // ← This contains the title!
  blobAssets,
  trackingMemoryId
);
```

#### 2. Title Field Handling - CORRECT EXTRACTION

The frontend is **correctly extracting** the title from filename:

```typescript
// From src/nextjs/src/services/upload/icp-with-processing.ts line 208
const memoryMetadata: MemoryMetadata = {
  memory_type: { Image: null } as MemoryType,
  title: [file.name.split(".")[0] || "Untitled"], // ← CORRECT: Extracts filename
  // ... other fields
};
```

#### 3. Backend API Mismatch

The issue is that:

- **Frontend**: Correctly extracts title and passes it in `MemoryMetadata`
- **Frontend**: Calls `memories_create` (wrong API - doesn't accept title)
- **Backend**: `memories_create` ignores the title and uses hardcoded "placeholder"
- **Backend**: `memories_create_with_internal_blobs` would use the title from `MemoryMetadata`

#### 4. API Design Problem - Mixed Asset Types

**CRITICAL ARCHITECTURAL ISSUE**: The current ICP upload flow creates a **mixed asset scenario** that doesn't fit either API:

```typescript
// Current ICP Upload Flow:
// ✅ 3 Blob Assets: original, display, thumb (uploaded to ICP blob storage)
// ✅ 1 Inline Asset: placeholder (stored inline in memory record)
// ❌ Problem: No single API handles both blob + inline assets
```

**Current API Limitations:**

- `memories_create`: Only handles **inline assets** (no title parameter)
- `memories_create_with_internal_blobs`: Only handles **blob assets** (no inline support)

**Result**: Frontend is forced to choose between:

1. Use `memories_create` → Lose title, lose blob assets
2. Use `memories_create_with_internal_blobs` → Lose inline placeholder asset
3. Make **two separate API calls** → Complex, inefficient, error-prone

### Data Flow Comparison

#### S3 Flow (Working)

```typescript
// S3 upload likely extracts title from filename
const title = file.name.replace(/\.[^/.]+$/, ""); // Remove extension
// Result: "vacation-photo.jpg" → "vacation-photo"
```

#### ICP Flow (Broken)

```typescript
// ICP upload might not extract title from filename
const title = []; // Empty array
// Result: No title → falls back to "placeholder"
```

## Investigation Areas

### 1. Title Extraction Logic

- **Question**: Is the filename being extracted and used as title during ICP upload?
- **Check**: `icp-with-processing.ts` memory metadata creation
- **Expected**: `title: [file.name.replace(/\.[^/.]+$/, "")]` or similar

### 2. ICP Backend Title Handling

- **Question**: How does the ICP backend handle empty title arrays?
- **Check**: `backend/src/memories/core/create.rs`
- **Expected**: Should fall back to a default or extract from filename

### 3. Frontend Title Transformation

- **Question**: Is the title transformation logic correct for ICP memories?
- **Check**: `transformICPMemoryHeaderToNeon` function
- **Expected**: Should handle empty arrays and fall back to `name` field

### 4. Memory Header Generation

- **Question**: How is the `MemoryHeader.name` field populated?
- **Check**: ICP backend memory header creation
- **Expected**: Should use filename or generated name

## Code Investigation Points

### 1. ICP Upload Memory Creation

```typescript
// File: src/nextjs/src/services/upload/icp-with-processing.ts
// Function: createICPMemoryRecordAndEdges
// Check: How is memoryMetadata.title populated?
```

### 2. ICP Backend Memory Creation

```rust
// File: src/backend/src/memories/core/create.rs
// Function: create_memory
// Check: How is title handled when creating Memory struct?
```

### 3. Memory Header Generation

```rust
// File: src/backend/src/memories/core/model_helpers.rs
// Function: create_memory_header
// Check: How is MemoryHeader.name populated?
```

### 4. Frontend Title Display

```typescript
// File: src/nextjs/src/components/common/content-card.tsx
// Function: renderTitle
// Check: Is the title transformation working correctly?
```

## Potential Fixes

### Fix 1: Use Correct ICP Backend API (Frontend) - **PRIMARY FIX**

**ROOT CAUSE**: Frontend is calling wrong API function. Change from `memories_create` to `memories_create_with_internal_blobs`:

```typescript
// In icp-with-processing.ts - REPLACE the memories_create call (lines 860-871)
// ❌ REMOVE: Current memories_create call
const result = await backend.memories_create(
  capsuleId,
  [placeholderBytes],
  []
  // ... other parameters
);

// ✅ ADD: Use memories_create_with_internal_blobs instead
const result = await backend.memories_create_with_internal_blobs(
  capsuleId,
  memoryMetadata, // ← Contains the extracted title!
  blobAssets,
  trackingMemoryId
);
```

### Fix 2: Verify Title Extraction (Frontend)

The title extraction is already correct, but verify it's working:

```typescript
// In icp-with-processing.ts - VERIFY this is working (line 208)
const memoryMetadata: MemoryMetadata = {
  memory_type: { Image: null } as MemoryType,
  title: [file.name.split(".")[0] || "Untitled"], // ← Should extract filename
  // ... other fields
};
```

### Fix 3: Update Asset Handling (Frontend)

Since we're switching to `memories_create_with_internal_blobs`, we need to handle both inline and blob assets:

```typescript
// Convert placeholder from inline to blob asset
const placeholderBlobAsset = {
  blobId: placeholderBlobId, // Upload placeholder as blob first
  assetType: "placeholder" as const,
  size: placeholderData.size,
  hash: placeholderHash,
  mimeType: placeholderData.mimeType,
};

// Add to blobAssets array
blobAssets.push(placeholderBlobAsset);
```

## Architectural Solution - New ICP Backend API

### Problem: Missing Unified API

The current ICP backend APIs are **too restrictive** for real-world use cases:

```rust
// Current APIs:
memories_create(..., inline_bytes, ...)                    // Only inline assets
memories_create_with_internal_blobs(..., blob_assets, ...) // Only blob assets

// Missing: Unified API for mixed assets
memories_create_unified(..., inline_assets, blob_assets, ...) // Both types
```

### Proposed Solution: `memories_create_unified`

**New ICP Backend API** that handles both asset types in a single call:

```rust
#[ic_cdk::update]
fn memories_create_unified(
    capsule_id: types::CapsuleId,
    memory_metadata: types::MemoryMetadata,  // ← Includes title!
    inline_assets: Vec<types::InlineAssetInput>,    // ← Placeholder data
    blob_assets: Vec<types::InternalBlobAssetInput>, // ← Original, display, thumb
    idem: String,
) -> types::Result20 {
    // Single API call handles:
    // 1. Title from memory_metadata.title
    // 2. Inline placeholder asset
    // 3. Multiple blob assets (original, display, thumb)
    // 4. Proper MemoryHeader generation with correct title
}
```

### Benefits of Unified API

1. **Single API Call**: No need for multiple backend calls
2. **Title Support**: Accepts `MemoryMetadata` with title
3. **Mixed Assets**: Handles both inline and blob assets
4. **Consistency**: Matches S3/Vercel Blob workflow (single memory creation)
5. **Efficiency**: No extra network round-trips
6. **Atomicity**: All assets created in single transaction

### Implementation Priority

**Option A: Quick Fix (Recommended)**

- Convert placeholder to blob asset
- Use existing `memories_create_with_internal_blobs`
- **Pros**: Immediate fix, no backend changes
- **Cons**: Slightly less efficient (4 blobs instead of 3+1 inline)

**Option B: Proper Fix (Long-term)**

- Implement `memories_create_unified` in ICP backend
- **Pros**: Optimal architecture, handles all asset types
- **Cons**: Requires backend development, testing, deployment

## Testing Scenarios

### 1. Filename Extraction

- Upload `vacation-photo.jpg` → Should display "vacation-photo"
- Upload `IMG_2024_12_19.jpg` → Should display "IMG_2024_12_19"
- Upload `file with spaces.png` → Should display "file with spaces"

### 2. Edge Cases

- Upload file with no extension → Should display filename
- Upload file with very long name → Should truncate appropriately
- Upload file with special characters → Should handle safely

### 3. Fallback Behavior

- No title provided → Should use filename
- Empty title → Should use filename
- Title is "placeholder" → Should use filename instead

## Related Files

- `src/nextjs/src/services/upload/icp-with-processing.ts` - ICP upload logic
- `src/backend/src/memories/core/create.rs` - ICP memory creation
- `src/backend/src/memories/types.rs` - ICP memory structures
- `src/nextjs/src/services/memories.ts` - Memory transformation
- `src/nextjs/src/components/common/content-card.tsx` - Title display

## Acceptance Criteria

- [ ] ICP memories display proper titles (filename without extension)
- [ ] No more "placeholder" titles in dashboard
- [ ] Title extraction works for various filename formats
- [ ] Fallback behavior works when no title is provided
- [ ] Title display is consistent between S3 and ICP memories
- [ ] Long filenames are handled appropriately (truncation)

## Priority Justification

**HIGH Priority** because:

- Directly impacts user experience
- Creates confusion about uploaded content
- Makes it difficult to identify memories
- Affects the core functionality of the memory management system
- Easy to reproduce and fix

## Summary

### Root Cause Identified ✅

The issue is **NOT** with title extraction (which is working correctly), but with **API architecture limitations**:

1. **Frontend correctly extracts title**: `file.name.split('.')[0] || 'Untitled'`
2. **Frontend correctly passes title**: In `MemoryMetadata.title` array
3. **Frontend calls wrong API**: `memories_create` (doesn't accept title) instead of `memories_create_with_internal_blobs` (accepts `MemoryMetadata` with title)
4. **Backend ignores title**: `memories_create` uses hardcoded "placeholder" instead of the provided title
5. **Architectural problem**: No single API handles mixed assets (3 blobs + 1 inline)

### Solution Options ✅

**Option A: Quick Fix (Recommended)**

- Convert placeholder to blob asset
- Use existing `memories_create_with_internal_blobs`
- **Impact**: Immediate fix, proper titles, 4 blob assets instead of 3+1 inline

**Option B: Proper Fix (Long-term)**

- Implement `memories_create_unified` in ICP backend
- **Impact**: Optimal architecture, single API call, handles all asset types

### Impact ✅

Both solutions will resolve the "placeholder" title issue and make ICP memories display proper filenames, matching the S3/Vercel Blob workflow behavior. Option A provides immediate relief, while Option B provides the optimal long-term architecture.
