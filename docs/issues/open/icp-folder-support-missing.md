# ICP Upload Missing Folder Support

## Issue

The ICP upload flow does not support folder organization for directory mode uploads, unlike the S3 upload flow. When users upload a folder, the folder structure is lost and memories are created without proper parent folder relationships.

## Current State

### S3 Flow ✅ (Working)

**File**: [src/nextjs/src/services/upload/s3-with-processing.ts:176-194](../../../src/nextjs/src/services/upload/s3-with-processing.ts#L176-L194)

```typescript
// 1. Creates folder for directory mode
const parentFolderId = await createFolderIfNeeded(mode, files);

// 2. Passes folder ID to finalizeAllAssets
await finalizeAllAssets(laneAResultForFile, laneBResultForFile, parentFolderId);

// 3. finalizeAllAssets passes it to memory creation
await finalizeAssets({ memoryId, assets, parentFolderId });
```

**Result**: ✅ Memories are created with proper `parent_folder_id` and folder structure is preserved

### ICP Flow ❌ (Broken)

**File**: [src/nextjs/src/services/upload/icp-with-processing.ts:193-194](../../../src/nextjs/src/services/upload/icp-with-processing.ts#L193-L194)

```typescript
// 1. Folder creation is COMMENTED OUT
// const _parentFolderId = await createFolderIfNeeded(mode, files);

// 2. Memory creation has EMPTY parent_folder_id
parent_folder_id: [], // Always empty!
```

**Result**: ❌ Memories are created without folder context, folder structure is lost

## Backend Support

### ✅ Backend Supports Folder IDs

**File**: [src/backend/backend.did:347](../../../src/backend/backend.did#L347)

```candid
type MemoryHeader = record {
  // ... other fields
  parent_folder_id : opt text;  // ✅ Supported
  // ... other fields
};
```

**File**: [src/backend/backend.did:574-579](../../../src/backend/backend.did#L574-L579)

```candid
memories_create_with_internal_blobs : (
    text,                    // capsule_id
    MemoryMetadata,          // ✅ Contains parent_folder_id
    vec InternalBlobAssetInput,
    text,                    // idempotency_key
) -> (Result6);
```

**File**: [src/backend/src/memories/types.rs:158](../../../src/backend/src/memories/types.rs#L158)

```rust
pub struct MemoryMetadata {
    // ... other fields
    pub parent_folder_id: Option<String>, // ✅ Supported in Rust
    // ... other fields
}
```

## Missing Implementation

### 1. Folder Creation

**Location**: [src/nextjs/src/services/upload/icp-with-processing.ts:193-194](../../../src/nextjs/src/services/upload/icp-with-processing.ts#L193-L194)

**Current State**: Commented out

```typescript
// 4. Create folder if needed (for directory mode)
// const _parentFolderId = await createFolderIfNeeded(mode, files);
```

**Required**: Uncomment and implement folder creation

### 2. Folder ID Usage in Memory Creation

**Location**: [src/nextjs/src/services/upload/icp-with-processing.ts:659](../../../src/nextjs/src/services/upload/icp-with-processing.ts#L659)

**Current State**: Always empty

```typescript
parent_folder_id: [], // Always empty!
```

**Required**: Use actual folder ID from folder creation

### 3. Folder Context in Asset Addition

**Location**: [src/nextjs/src/services/upload/icp-with-processing.ts:196-209](../../../src/nextjs/src/services/upload/icp-with-processing.ts#L196-L209)

**Current State**: No folder context passed to `addDerivativeAssetsToMemory`

**Required**: Pass folder context to asset addition functions

## Required Implementation

### 1. Enable Folder Creation

```typescript
// 4. Create folder if needed (for directory mode)
const parentFolderId = await createFolderIfNeeded(mode, files);
```

### 2. Update Memory Creation

```typescript
// In createICPMemoryWithOriginalBlob function
const memoryMetadata: MemoryMetadata = {
  // ... other fields
  parent_folder_id: parentFolderId ? [parentFolderId] : [], // Use actual folder ID
  // ... other fields
};
```

### 3. Pass Folder Context Through Flow

```typescript
// Pass folder ID through the entire flow
const assetAdditionPromises = files.map(async (file, index) => {
  const memoryResult = laneAResult.value[index];
  const derivativeResult = laneBResult.value[index];

  if (derivativeResult) {
    await addDerivativeAssetsToMemory(
      memoryResult.data.id,
      derivativeResult,
      file,
      parentFolderId // Pass folder context
    );
  }
});
```

### 4. Update Asset Addition Function

```typescript
// Update addDerivativeAssetsToMemory signature
async function addDerivativeAssetsToMemory(
  icpMemoryId: string,
  derivativeAssets: ProcessedAssets,
  file: File,
  parentFolderId?: string // Add folder context
): Promise<void> {
  // Use folder context in asset metadata if needed
}
```

## Impact

### Current Behavior

- **Directory uploads lose folder structure**
- **Memories created without parent folder relationships**
- **Inconsistent behavior between S3 and ICP flows**
- **Users cannot organize memories in folders when using ICP**

### Expected Behavior

- **Directory uploads preserve folder structure**
- **Memories created with proper parent folder relationships**
- **Consistent behavior between S3 and ICP flows**
- **Users can organize memories in folders with ICP**

## Testing

### Test Scenarios

1. **Directory mode upload** - Should create folder and set `parent_folder_id` on memories
2. **Multiple files mode upload** - Should not create folder, `parent_folder_id` should be empty
3. **Mixed upload types** - Should handle both modes correctly
4. **Folder hierarchy** - Should support nested folder structures

### Test Files

- [test_upload_2lane_4asset_system.mjs](../../../tests/backend/shared-capsule/upload/test_upload_2lane_4asset_system.mjs) - Should test folder creation
- [test_memories_add_asset.mjs](../../../tests/backend/shared-capsule/upload/test_memories_add_asset.mjs) - Should test folder context in asset addition

## Dependencies

### Required Functions

- `createFolderIfNeeded()` - Already exists in S3 flow, needs to be imported
- `extractFolderName()` - Already exists, needs to be imported

### Import Statements Needed

```typescript
import { extractFolderName } from "./shared-utils";
```

## Priority

**High** - This affects the core functionality of folder organization and creates inconsistent behavior between storage backends. Users expect folder uploads to preserve folder structure regardless of the storage backend used.

## Related Issues

- [ICP Multiple Files Incomplete Flow](./icp-multiple-files-incomplete-flow.md) - Related to the overall multiple files implementation
- [ICP Upload Flow Documentation](./icp-upload-flow-documentation.md) - Documents the current flow structure

