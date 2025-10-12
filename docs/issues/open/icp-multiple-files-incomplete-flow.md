# ICP Multiple Files Upload Flow Incomplete

## Issue

The multiple files upload flow in ICP is incomplete compared to the single file flow. While single files properly complete the full 4-step process, multiple files are missing crucial steps for asset addition and storage edge creation.

## Current State

### Single File Flow (Complete) ✅

**File**: [src/nextjs/src/services/upload/icp-with-processing.ts:104-162](../../../src/nextjs/src/services/upload/icp-with-processing.ts#L104-L162)

**Function**: `uploadFileAndCreateMemoryWithDerivatives`

**Steps**:

1. ✅ **Lane A**: Upload original + create memory with original blob
2. ✅ **Lane B**: Process derivatives + upload to ICP
3. ✅ **Asset Addition**: `addDerivativeAssetsToMemory()` - adds derivatives to existing memory
4. ✅ **Storage Edges**: `createStorageEdgesForAllAssets()` - creates storage tracking records

### Multiple Files Flow (Incomplete) ❌

**File**: [src/nextjs/src/services/upload/icp-with-processing.ts:172-352](../../../src/nextjs/src/services/upload/icp-with-processing.ts#L172-L352)

**Function**: `uploadMultipleToICPWithProcessing`

**Steps**:

1. ✅ **Lane A**: Upload all originals + create memories with original blobs
2. ✅ **Lane B**: Process derivatives for all image files + upload to ICP
3. ✅ **Asset Addition**: COMPLETED - derivatives added to existing memories
4. ✅ **Storage Edges**: COMPLETED - storage tracking records created

## Missing Implementation

### 1. Asset Addition Step ✅ COMPLETED

**Location**: [src/nextjs/src/services/upload/icp-with-processing.ts:196-209](../../../src/nextjs/src/services/upload/icp-with-processing.ts#L196-L209)

**Implementation**: ✅ Implemented

```typescript
// 5. Add derivative assets to existing memories
if (laneAResult.status === "fulfilled" && laneBResult?.status === "fulfilled") {
  // Add derivatives to each memory
  const assetAdditionPromises = files.map(async (file, index) => {
    const memoryResult = laneAResult.value[index];
    const derivativeResult = laneBResult.value[index];

    if (derivativeResult) {
      await addDerivativeAssetsToMemory(memoryResult.data.id, derivativeResult, file);
    }
  });

  await Promise.all(assetAdditionPromises);
}
```

**Status**: ✅ **COMPLETED** - Asset addition now properly adds derivatives to existing memories

### 2. Storage Edges Creation ✅ COMPLETED

**Location**: [src/nextjs/src/services/upload/icp-with-processing.ts:211-249](../../../src/nextjs/src/services/upload/icp-with-processing.ts#L211-L249)

**Implementation**: ✅ Implemented

```typescript
// 6. Create storage edges for all assets
if (laneAResult.status === "fulfilled") {
  const storageEdgePromises = files.map(async (file, index) => {
    const memoryResult = laneAResult.value[index];
    const derivativeResult = laneBResult?.status === "fulfilled" ? laneBResult.value[index] : null;

    const derivativeAssets = derivativeResult
      ? {
          display: derivativeResult.display
            ? {
                blobId: derivativeResult.display.storageKey || "",
                size: derivativeResult.display.bytes || 0,
              }
            : undefined,
          thumb: derivativeResult.thumb
            ? {
                blobId: derivativeResult.thumb.storageKey || "",
                size: derivativeResult.thumb.bytes || 0,
              }
            : undefined,
          placeholder: derivativeResult.placeholder
            ? {
                blobId: "inline",
                size: derivativeResult.placeholder.bytes || 0,
              }
            : undefined,
        }
      : {};

    await createStorageEdgesForAllAssets(memoryResult.data.id, file, memoryResult.results[0].blobId, derivativeAssets);
  });

  await Promise.all(storageEdgePromises);
}
```

**Status**: ✅ **COMPLETED** - Storage edges now properly created for all assets

## Additional Issues

### 3. Folder Name and Title Handling

**Location**: [src/nextjs/src/services/upload/icp-with-processing.ts:200-201](../../../src/nextjs/src/services/upload/icp-with-processing.ts#L200-L201)

**Current State**: Commented out folder creation

```typescript
// 4. Create folder if needed (for directory mode)
// const _parentFolderId = await createFolderIfNeeded(mode, files);
```

**Issues**:

- Folder creation is commented out
- No folder name extraction for directory mode
- Memory titles may not properly reflect folder structure

**Required**: Implement proper folder handling for directory mode uploads

### 4. Memory Metadata Consistency

**Location**: [src/nextjs/src/services/upload/icp-with-processing.ts:290-317](../../../src/nextjs/src/services/upload/icp-with-processing.ts#L290-L317)

**Current State**: Memory metadata is created in the commented out section

**Issues**:

- Memory titles use `file.name.split('.')[0]` which may not be appropriate for folder uploads
- No folder context in memory metadata
- Inconsistent with single file memory creation

## Required Implementation

### 1. Complete Asset Addition

```typescript
// After Lane A and Lane B complete
if (laneAResult.status === "fulfilled" && laneBResult?.status === "fulfilled") {
  // Add derivatives to each memory
  const assetAdditionPromises = files.map(async (file, index) => {
    const memoryResult = laneAResult.value[index];
    const derivativeResult = laneBResult.value[index];

    if (derivativeResult) {
      await addDerivativeAssetsToMemory(memoryResult.data.id, derivativeResult, file);
    }
  });

  await Promise.all(assetAdditionPromises);
}
```

### 2. Complete Storage Edges Creation

```typescript
// After asset addition
const storageEdgePromises = files.map(async (file, index) => {
  const memoryResult = laneAResult.value[index];
  const derivativeResult = laneBResult.value[index];

  const derivativeAssets = derivativeResult
    ? {
        display: derivativeResult.display
          ? {
              blobId: derivativeResult.display.storageKey || "",
              size: derivativeResult.display.bytes || 0,
            }
          : undefined,
        thumb: derivativeResult.thumb
          ? {
              blobId: derivativeResult.thumb.storageKey || "",
              size: derivativeResult.thumb.bytes || 0,
            }
          : undefined,
        placeholder: derivativeResult.placeholder
          ? {
              blobId: "inline",
              size: derivativeResult.placeholder.bytes || 0,
            }
          : undefined,
      }
    : {};

  await createStorageEdgesForAllAssets(memoryResult.data.id, file, memoryResult.results[0].blobId, derivativeAssets);
});

await Promise.all(storageEdgePromises);
```

### 3. Implement Folder Handling

```typescript
// Extract folder name for directory mode
const folderName = mode === "directory" ? extractFolderName(files[0]) : null;

// Create folder if needed
const parentFolderId = folderName ? await createFolder(folderName) : null;

// Use folder context in memory metadata
const memoryTitle = folderName ? `${folderName}/${file.name.split(".")[0]}` : file.name.split(".")[0];
```

## Impact

### Current Behavior

- Multiple files upload to ICP but derivatives are not linked to memories
- No storage tracking for multiple file uploads
- Inconsistent behavior between single and multiple file uploads
- Folder structure not preserved

### Expected Behavior

- All files complete the full 4-step process
- Derivatives properly linked to their respective memories
- Storage edges created for all assets
- Folder structure preserved in memory metadata
- Consistent behavior with single file uploads

## Testing

### Test Scenarios

1. **Multiple image files** - Should create memories with all 4 assets (original + 3 derivatives)
2. **Mixed file types** - Images get derivatives, non-images get only original
3. **Directory mode** - Should preserve folder structure in memory titles
4. **Multiple files mode** - Should create individual memories without folder context

### Test Files

- [test_upload_2lane_4asset_system.mjs](../../../tests/backend/shared-capsule/upload/test_upload_2lane_4asset_system.mjs) - Should be updated to test complete flow
- [test_memories_add_asset.mjs](../../../tests/backend/shared-capsule/upload/test_memories_add_asset.mjs) - Should test asset addition for multiple files

## Priority

**High** - This affects the core functionality of multiple file uploads and creates inconsistent behavior between single and multiple file uploads.
