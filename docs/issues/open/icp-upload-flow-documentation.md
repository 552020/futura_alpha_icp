# ICP Upload Flow Documentation

## Issue

The ICP upload flow is not clearly documented, making it difficult to understand how files are processed from the routing decision point through to completion. This document provides a comprehensive overview of the ICP upload flow for both single files and multiple files/folders.

## ICP Upload Flow Overview

The ICP upload system uses a parallel two-lane approach:

- **Lane A**: Upload original file and create memory with original blob
- **Lane B**: Process image derivatives and upload to ICP
- **Post-processing**: Add derivatives to existing memory and create storage edges

## Single vs Multiple Files Decision

The decision between single and multiple file processing happens at the **upload hook level**, before any routing to specific processors:

### Flow Diagram

```
User Action
    ↓
┌─────────────────┐
│   Upload Hook   │
│(use-file-upload)│
│ handleFileUpload│
└─────────────────┘
    ↓
┌─────────────────┐
│ File Count?     │
│ • Single File   │
│ • Multiple Files│
└─────────────────┘
    ↓
┌─────────────────┐    ┌─────────────────┐
│ Single File     │    │ Multiple Files  │
│ Processor       │    │ Processor       │
│                 │    │                 │
│ ↓               │    │ ↓               │
│ Check ICP?      │    │ Check ICP?      │
│ • Yes → ICP     │    │ • Yes → ICP     │
│ • No → Other    │    │ • No → Other    │
└─────────────────┘    └─────────────────┘
```

### Upload Hook Entry Point

**File**: [src/nextjs/src/hooks/use-file-upload.ts:51](../../../src/nextjs/src/hooks/use-file-upload.ts#L51)

**Function**: `handleFileUpload`

The upload hook receives either:

- **Single File**: `File` object from file input
- **Multiple Files**: `FileList` or `File[]` from folder/multiple file input

```typescript
// File: [src/nextjs/src/hooks/use-file-upload.ts:51](../../../src/nextjs/src/hooks/use-file-upload.ts#L51)
const handleFileUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
  const fileList = event.target.files;

  // Convert FileList to static Array
  const files = Array.from(fileList);

  // Decision logic: single vs multiple files
  if (mode === 'single' || files.length === 1) {
    // Routes to single-file-processor.ts
    await processSingleFile({ file: files[0], ... });
  } else {
    // Routes to multiple-files-processor.ts
    await processMultipleFiles({ files, ... });
  }
};
```

### Routing Logic

The routing happens in the respective processors based on the file count:

**File**: [src/nextjs/src/services/upload/single-file-processor.ts:74](../../../src/nextjs/src/services/upload/single-file-processor.ts#L74)

```typescript
// In single-file-processor.ts (called for single files)
if (userBlobHostingPreferences.includes("icp")) {
  // Single file ICP flow
}
```

**File**: [src/nextjs/src/services/upload/multiple-files-processor.ts:XX](../../../src/nextjs/src/services/upload/multiple-files-processor.ts#LXX)

```typescript
// In multiple-files-processor.ts (called for multiple files)
if (userBlobHostingPreferences.includes("icp")) {
  // Multiple files ICP flow
}
```

### Entry Point: Routing Decision

The ICP upload flow begins when the system determines that the user's blob hosting preferences include 'icp':

**File**: [src/services/upload/single-file-processor.ts:74](../../../src/nextjs/src/services/upload/single-file-processor.ts#L74)

```typescript
if (userBlobHostingPreferences.includes("icp")) {
  // ICP upload flow begins here
}
```

## Single File Upload Flow

### 1. Authentication Check

**File**: [src/services/upload/single-file-processor.ts:76-85](../../../src/nextjs/src/services/upload/single-file-processor.ts#L76-L85)

```typescript
// Double-check ICP authentication (safety net for multiple upload flows)
try {
  await checkICPAuthentication();
} catch (_error) {
  showToast({
    variant: "destructive",
    title: "Authentication Required",
    description: "Please connect your Internet Identity to upload to ICP",
  });
  return;
}
```

### 2. Dynamic Import and Function Call

**File**: [src/services/upload/single-file-processor.ts:90-91](../../../src/nextjs/src/services/upload/single-file-processor.ts#L90-L91)

```typescript
// ICP upload with parallel processing (Lane A + Lane B + finalizeAllAssets)
const { uploadFileAndCreateMemoryWithDerivatives } = await import("./icp-with-processing");
const uploadResult = await uploadFileAndCreateMemoryWithDerivatives(file, onProgress);
```

### 3. Result Processing

**File**: [src/services/upload/single-file-processor.ts:92-104](../../../src/nextjs/src/services/upload/single-file-processor.ts#L92-L104)

```typescript
data = {
  data: uploadResult.data,
  results: uploadResult.results.map((result) => ({
    memoryId: result.memoryId,
    size: Number(result.size),
    checksum_sha256: result.checksumSha256
      ? Array.from(result.checksumSha256)
          .map((b) => b.toString(16).padStart(2, "0"))
          .join("")
      : null,
  })),
  userId: uploadResult.userId,
};
```

## Multiple Files/Folder Upload Flow

### 1. Authentication Check

```typescript
// File: [src/services/upload/multiple-files-processor.ts:XX-XX](../../../src/nextjs/src/services/upload/multiple-files-processor.ts#LXX-LXX)
// Check ICP authentication for multiple files
try {
  await checkICPAuthentication();
} catch (_error) {
  showToast({
    variant: "destructive",
    title: "Authentication Required",
    description: "Please connect your Internet Identity to upload to ICP",
  });
  return;
}
```

### 2. Dynamic Import and Function Call

```typescript
// ICP batch upload with parallel processing
const { uploadMultipleToICPWithProcessing } = await import("./icp-with-processing");
const uploadResults = await uploadMultipleToICPWithProcessing(files, mode, onProgress);
```

### 3. Result Processing

```typescript
data = {
  data: { id: "batch-upload" }, // Batch upload identifier
  results: uploadResults.flatMap((result) =>
    result.results.map((asset) => ({
      memoryId: asset.memoryId,
      size: Number(asset.size),
      checksum_sha256: asset.checksumSha256
        ? Array.from(asset.checksumSha256)
            .map((b) => b.toString(16).padStart(2, "0"))
            .join("")
        : null,
    }))
  ),
  userId: uploadResults[0]?.userId || "",
};
```

## Core ICP Upload Functions

### Single File: `uploadFileAndCreateMemoryWithDerivatives`

**Location**: [src/services/upload/icp-with-processing.ts](../../../src/nextjs/src/services/upload/icp-with-processing.ts)

**Function Signature**:

```typescript
// File: [src/services/upload/icp-with-processing.ts:96](../../../src/nextjs/src/services/upload/icp-with-processing.ts#L96)
export async function uploadFileAndCreateMemoryWithDerivatives(
  file: File,
  onProgress?: (progress: number) => void
): Promise<UploadServiceResult>;
```

**Process**:

1. **Lane A**: Upload original file and create memory with original blob
2. **Lane B**: Process image derivatives and upload to ICP
3. **Asset Addition**: Add derivatives to existing memory using `memories_add_asset` endpoints
4. **Storage Edges**: Create storage edge records for tracking

### Multiple Files: `uploadMultipleToICPWithProcessing`

**Location**: [src/services/upload/icp-with-processing.ts](../../../src/nextjs/src/services/upload/icp-with-processing.ts)

**Function Signature**:

```typescript
// File: [src/services/upload/icp-with-processing.ts:XXX](../../../src/nextjs/src/services/upload/icp-with-processing.ts#LXXX)
export async function uploadMultipleToICPWithProcessing(
  files: File[],
  mode: "directory" | "multiple-files",
  onProgress?: (progress: number) => void
): Promise<UploadServiceResult[]>;
```

**Process**:

1. **Parallel Processing**: Process all files simultaneously
2. **Lane A**: Upload all originals and create memories
3. **Lane B**: Process derivatives for each image file
4. **Asset Addition**: Add derivatives to each memory
5. **Storage Edges**: Create storage edges for all memories

## Detailed Flow Breakdown

### Lane A: Original Upload + Memory Creation

**File**: [src/services/upload/icp-with-processing.ts:XXX](../../../src/nextjs/src/services/upload/icp-with-processing.ts#LXXX)

```typescript
// Upload original file using ICP chunked upload
const uploadResult = await uploadFileToICPWithProgress(file, (progress) => {
  onProgress?.(progress);
});

// Create ICP memory record with the original blob
const icpMemoryId = await createICPMemoryWithOriginalBlob(file, uploadResult.uploadResult.blob_id);
```

### Lane B: Derivative Processing + Upload

**File**: [src/services/upload/icp-with-processing.ts:XXX](../../../src/nextjs/src/services/upload/icp-with-processing.ts#LXXX)

```typescript
// Process image derivatives (display, thumb, placeholder)
const processedBlobs = await processImageDerivativesPure(file);

// Upload derivatives to ICP
const derivativeAssets = await uploadProcessedAssetsToICP(processedBlobs, file.name);
```

### Asset Addition to Existing Memory

**File**: [src/services/upload/icp-with-processing.ts:982-1150](../../../src/nextjs/src/services/upload/icp-with-processing.ts#L982-L1150)

```typescript
// Add derivatives to existing memory using new endpoints
await addDerivativeAssetsToMemory(icpMemoryId, derivativeAssets, file);

// Functions used:
// - backend.memories_add_asset() for display and thumb derivatives
// - backend.memories_add_inline_asset() for placeholder inline asset
```

### Storage Edges Creation

**File**: [src/services/upload/icp-with-processing.ts:1178-1287](../../../src/nextjs/src/services/upload/icp-with-processing.ts#L1178-L1287)

```typescript
// Create storage edges for all artifacts
await createStorageEdgesForAllAssets(icpMemoryId, file, originalBlobId, derivativeAssets);

// Creates storage edge records for:
// - Metadata edge: icp://memory/{memoryId}
// - Original asset edge: icp://blob/{originalBlobId}
// - Display asset edge: icp://blob/{displayBlobId}
// - Thumb asset edge: icp://blob/{thumbBlobId}
// - Placeholder asset edge: icp://blob/{placeholderBlobId}
```

## Key Backend Endpoints Used

### Memory Creation

**File**: [src/backend/backend.did](../../../src/backend/backend.did)

- `backend.memories_create_with_internal_blobs()` - Create memory with original blob

### Asset Addition

**File**: [src/backend/backend.did](../../../src/backend/backend.did)

- `backend.memories_add_asset()` - Add blob assets to existing memory
- `backend.memories_add_inline_asset()` - Add inline assets to existing memory

### Upload Management

**File**: [src/backend/backend.did](../../../src/backend/backend.did)

- `backend.uploads_begin()` - Start upload session
- `backend.uploads_put_chunk()` - Upload file chunks
- `backend.uploads_finish()` - Complete upload

### Storage Management

**File**: [src/nextjs/src/app/api/storage/edges/route.ts](../../../src/nextjs/src/app/api/storage/edges/route.ts)

- `PUT /api/storage/edges` - Create storage edge records

## File Structure

```
src/services/upload/
├── [icp-with-processing.ts](../../../src/nextjs/src/services/upload/icp-with-processing.ts)          # Main ICP upload functions
├── [single-file-processor.ts](../../../src/nextjs/src/services/upload/single-file-processor.ts)        # Single file routing and processing
├── [multiple-files-processor.ts](../../../src/nextjs/src/services/upload/multiple-files-processor.ts)     # Multiple files routing and processing
├── [image-derivatives.ts](../../../src/nextjs/src/services/upload/image-derivatives.ts)            # Image processing utilities
└── [types.ts](../../../src/nextjs/src/services/upload/types.ts)                        # Type definitions
```

## Quick Reference Links

### Entry Points

- **Single File**: [src/services/upload/single-file-processor.ts:74](../../../src/nextjs/src/services/upload/single-file-processor.ts#L74) - `if (userBlobHostingPreferences.includes("icp"))`
- **Multiple Files**: [src/services/upload/multiple-files-processor.ts:XX](../../../src/nextjs/src/services/upload/multiple-files-processor.ts#LXX) - ICP routing logic

### Core Functions

- **Single Upload**: [src/services/upload/icp-with-processing.ts:96](../../../src/nextjs/src/services/upload/icp-with-processing.ts#L96) - `uploadFileAndCreateMemoryWithDerivatives`
- **Multiple Upload**: [src/services/upload/icp-with-processing.ts:XXX](../../../src/nextjs/src/services/upload/icp-with-processing.ts#LXXX) - `uploadMultipleToICPWithProcessing`
- **Asset Addition**: [src/services/upload/icp-with-processing.ts:982-1150](../../../src/nextjs/src/services/upload/icp-with-processing.ts#L982-L1150) - `addDerivativeAssetsToMemory`
- **Storage Edges**: [src/services/upload/icp-with-processing.ts:1178-1287](../../../src/nextjs/src/services/upload/icp-with-processing.ts#L1178-L1287) - `createStorageEdgesForAllAssets`

### Backend Definitions

- **API Definitions**: [src/backend/backend.did](../../../src/backend/backend.did) - All backend endpoint definitions
- **Storage API**: [src/nextjs/src/app/api/storage/edges/route.ts](../../../src/nextjs/src/app/api/storage/edges/route.ts) - Storage edge management

## Error Handling

### Authentication Errors

- Internet Identity not connected
- Authentication expired
- Invalid principal

### Upload Errors

- File size limits exceeded
- Chunk upload failures
- Memory creation failures

### Processing Errors

- Image processing failures
- Asset addition failures
- Storage edge creation failures

## Progress Tracking

### Single File Progress

- Overall upload progress (0-100%)
- Lane A progress (original upload)
- Lane B progress (derivative processing)

### Multiple Files Progress

- Per-file progress tracking
- Overall batch progress
- Individual file status (pending, uploading, completed, failed)

## Success Criteria

### Single File Upload Success

- ✅ Original file uploaded to ICP
- ✅ Memory created with original blob
- ✅ Derivatives processed and uploaded (if image)
- ✅ Assets added to memory using new endpoints
- ✅ Storage edges created for all artifacts

### Multiple Files Upload Success

- ✅ All files uploaded to ICP
- ✅ All memories created with original blobs
- ✅ All derivatives processed and uploaded (for images)
- ✅ All assets added to respective memories
- ✅ All storage edges created

## Testing

### Test Files

- [test_memories_add_asset.mjs](../../../tests/backend/shared-capsule/upload/test_memories_add_asset.mjs) - Tests asset addition endpoints
- [test_upload_2lane_4asset_system.mjs](../../../tests/backend/shared-capsule/upload/test_upload_2lane_4asset_system.mjs) - Tests complete upload flow
- [test_deletion_workflows.mjs](../../../tests/backend/shared-capsule/upload/test_deletion_workflows.mjs) - Tests cleanup workflows

### Test Scenarios

- Single image file upload
- Single non-image file upload
- Multiple image files upload
- Mixed file types upload
- Large file uploads
- Error scenarios

## Future Improvements

1. **Retry Logic**: Implement retry mechanisms for failed uploads
2. **Resume Uploads**: Support resuming interrupted uploads
3. **Batch Optimization**: Optimize batch uploads for better performance
4. **Progress Granularity**: More detailed progress reporting
5. **Error Recovery**: Better error recovery and cleanup mechanisms
