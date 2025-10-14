# Asset Type Tagging Mismatch: Display vs Derivative

## Issue Summary

**Critical Bug**: Display assets were being incorrectly tagged as `Derivative` instead of `Display` during ICP upload, causing the backend to return placeholder-sized images (32x21 pixels) instead of proper display images (2048px+).

## Root Cause

The frontend was incorrectly tagging display assets with `asset_type: { Derivative: null }` instead of `asset_type: { Display: null }`, causing a mismatch between frontend asset creation and backend asset lookup.

## Code Locations

### Frontend (Fixed)

**File**: `src/nextjs/src/services/upload/icp-with-processing.ts`

**Before (Buggy)**:

```typescript
// Line 736 - WRONG asset type
asset_type: { Derivative: null },  // ❌ Should be Display
```

**After (Fixed)**:

```typescript
// Line 736 - CORRECT asset type
asset_type: { Display: null },     // ✅ Now matches backend expectations
```

**Thumbnail (Was Already Correct)**:

```typescript
// Line 808 - This was already correct
asset_type: { Thumbnail: null },   // ✅ Correct
```

### Backend Asset Type Definitions

**File**: `src/backend/src/memories/types.rs`

```rust
// Lines 19-25
pub enum AssetType {
    Original,
    Thumbnail,
    Display,      // ← Backend looks for this
    Derivative,   // ← Frontend was incorrectly using this
    Metadata,
}
```

### Backend Asset Lookup

**File**: `src/backend/src/memories/utils.rs`

```rust
// Line 234 - Looking for Thumbnail assets
find_asset_by_type(memory, crate::memories::types::AssetType::Thumbnail)

// Line 256 - Looking for Display assets
find_asset_by_type(memory, crate::memories::types::AssetType::Display)
```

## Impact

### Before Fix

- **Display assets**: Tagged as `Derivative` → Backend couldn't find them → Returned placeholder images (32x21px)
- **Thumbnail assets**: Tagged as `Thumbnail` → Backend found them → Returned proper thumbnails (512px)
- **Result**: Dashboard showed low-quality placeholder images instead of high-quality display images

### After Fix

- **Display assets**: Tagged as `Display` → Backend finds them → Returns proper display images (2048px+)
- **Thumbnail assets**: Tagged as `Thumbnail` → Backend finds them → Returns proper thumbnails (512px)
- **Result**: Dashboard shows high-quality images as intended

## Technical Details

### Asset Type Mapping

| Frontend Tag           | Backend Enum            | Purpose              | Dimensions        |
| ---------------------- | ----------------------- | -------------------- | ----------------- |
| `{ Display: null }`    | `AssetType::Display`    | High-quality viewing | ~2048px long edge |
| `{ Thumbnail: null }`  | `AssetType::Thumbnail`  | Grid thumbnails      | ~512px long edge  |
| `{ Derivative: null }` | `AssetType::Derivative` | Generic derivatives  | Variable          |

### Image Processing Pipeline

1. **Frontend**: Processes original image → Creates display (2048px) and thumbnail (512px) blobs
2. **Frontend**: Uploads blobs to ICP with asset metadata
3. **Frontend**: Tags assets with `asset_type` field
4. **Backend**: Searches for assets by `AssetType` enum
5. **Backend**: Returns asset links for found assets
6. **Frontend**: Displays images using returned links

### The Bug

- **Step 3**: Frontend incorrectly tagged display assets as `Derivative`
- **Step 4**: Backend searched for `AssetType::Display` but found `AssetType::Derivative`
- **Step 5**: Backend couldn't find display assets, returned placeholder dimensions
- **Step 6**: Frontend displayed 32x21px placeholder images instead of 2048px+ display images

## Symptoms Observed

1. **Upload Success**: All assets uploaded successfully to ICP
2. **Memory Creation**: Memory records created with all asset references
3. **Asset Lookup Failure**: Backend couldn't find display assets during memory listing
4. **Placeholder Fallback**: Dashboard displayed 32x21px images instead of proper display images
5. **Console Logs**: Backend logs showed "No display asset found" messages

## Fix Applied

**Single Line Change**:

```diff
- asset_type: { Derivative: null },
+ asset_type: { Display: null },
```

**File**: `src/nextjs/src/services/upload/icp-with-processing.ts:736`

## Testing

### Before Fix

- Upload image → Dashboard shows 32x21px placeholder
- Backend logs: "❌ No display asset found"
- Asset dimensions: All assets 32x21px

### After Fix

- Upload image → Dashboard shows proper 2048px+ display image
- Backend logs: "✅ Found display asset: [asset_id]"
- Asset dimensions: Display ~2048px, Thumbnail ~512px, Placeholder 32px

## Prevention

1. **Type Safety**: Consider using TypeScript enums for asset types to prevent mismatches
2. **Unit Tests**: Add tests to verify asset type tagging matches backend expectations
3. **Integration Tests**: Test end-to-end asset creation and retrieval flow
4. **Code Review**: Verify asset type constants match between frontend and backend

## Related Files

- `src/nextjs/src/services/upload/icp-with-processing.ts` - Frontend asset tagging
- `src/backend/src/memories/types.rs` - Backend asset type definitions
- `src/backend/src/memories/utils.rs` - Backend asset lookup logic
- `src/nextjs/src/services/memories.ts` - Frontend asset URL generation

## Status

✅ **RESOLVED** - Fixed in commit [pending]

**Date**: 2025-01-14  
**Severity**: Critical (affects user experience)  
**Impact**: High (dashboard image quality)


