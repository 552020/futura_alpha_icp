# Vercel Blob Usage in Futura Alpha ICP

## Overview

This document provides a comprehensive overview of how Vercel Blob is used throughout the Futura Alpha ICP codebase for file storage and management.

## Architecture

### Storage Backend Types

- **Primary Backend**: `vercel_blob` - Used for most file uploads
- **Alternative Backends**: `s3`, `icp`, `arweave`, `ipfs`, `cloudinary`, `neon`

### Provider Implementation

- **`VercelBlobProvider`**: Direct uploads for small/medium files
- **`VercelBlobGrantProvider`**: Grant-based uploads for large files (>4MB)

## Core Files and Functions

### 1. Main Provider Implementation

**File**: `src/lib/storage/providers/vercel-blob.ts`

- **Class**: `VercelBlobProvider`
- **Methods**:
  - `upload(file, options)` - Direct file upload
  - `delete(url)` - File deletion
  - `isAvailable()` - Check if BLOB_READ_WRITE_TOKEN is set
  - `reconstructUrl(key)` - URL reconstruction (limited)

### 2. Grant-Based Provider

**File**: `src/lib/storage/providers/vercel-blob-grant.ts`

- **Class**: `VercelBlobGrantProvider`
- **Methods**:
  - `upload(file, options)` - Grant-based upload for large files
  - `delete(url)` - File deletion
  - `isAvailable()` - Check availability

### 3. Storage Manager

**File**: `src/lib/storage/storage-manager.ts`

- **Class**: `StorageManager`
- **Default Backend**: `vercel_blob`
- **Provider Registration**: Automatically registers both Vercel Blob providers

## Upload Flow Patterns

### Small Files (< 4MB)

```
File → VercelBlobProvider → Direct Upload → Vercel Blob Storage
```

### Large Files (> 4MB)

```
File → VercelBlobGrantProvider → Grant Request → Presigned URL → Direct Upload
```

### Image Processing

```
Image → processImageForMultipleAssets() → 3 Assets → uploadProcessedAssetsToBlob() → Vercel Blob
```

## Key Upload Locations

### 1. Image Processing

**File**: `src/app/api/memories/utils/image-processing.ts`

```typescript
// Upload processed image assets (original, display, thumb)
const { put } = await import("@vercel/blob");
const blobResults = await uploadProcessedAssetsToBlob(processedAssets, baseFileName);
```

### 2. Grant-Based Uploads

**File**: `src/app/api/memories/grant/route.ts`

```typescript
import { put } from "@vercel/blob";
// Generate presigned URL for Vercel Blob
```

### 3. Memory Creation

**File**: `src/app/api/memories/post.ts`

```typescript
// Default storage backend for memory creation
storageBackend: "vercel_blob";
```

### 4. Upload Service

**File**: `src/services/upload.ts`

```typescript
// Upload processed assets to Vercel Blob
const [originalResult, displayResult, thumbResult] = await Promise.all([
  storageManager.upload(processedAssets.original.blob as File, "vercel_blob"),
  storageManager.upload(processedAssets.display.blob as File, "vercel_blob"),
  storageManager.upload(processedAssets.thumb.blob as File, "vercel_blob"),
]);
```

## Configuration

### Environment Variables

- **`BLOB_READ_WRITE_TOKEN`**: Required for Vercel Blob access (handles both direct and grant-based uploads)
- **`BLOB_FOLDER_NAME`**: Folder name for storing application files (default: "futura")

### Package Dependencies

```json
{
  "@vercel/blob": "^0.27.3"
}
```

## Database Schema

### Storage Backend Enum

```sql
CREATE TYPE storage_backend_t AS ENUM (
  's3',
  'vercel_blob',
  'icp',
  'arweave',
  'ipfs',
  'neon'
);
```

### Asset Storage

```sql
CREATE TABLE assets (
  id TEXT PRIMARY KEY,
  memory_id TEXT NOT NULL,
  asset_type TEXT NOT NULL, -- 'original' | 'display' | 'thumb'
  url TEXT NOT NULL,
  storage_key TEXT NOT NULL,
  storage_backend storage_backend_t NOT NULL DEFAULT 'vercel_blob',
  bytes INTEGER NOT NULL,
  width INTEGER,
  height INTEGER,
  mime_type TEXT NOT NULL
);
```

## Usage Patterns by File Type

### Images

- **Small Images**: Direct upload via `VercelBlobProvider`
- **Large Images**: Grant-based upload via `VercelBlobGrantProvider`
- **Processed Images**: Always creates 3 assets (original, display, thumb)

### Documents

- **Small Documents**: Direct upload
- **Large Documents**: Grant-based upload

### General Files

- **Default Backend**: `vercel_blob`
- **Fallback**: Other backends if Vercel Blob unavailable

## API Endpoints

### Upload Endpoints

- **`/api/memories`** - Main upload endpoint (multipart/form-data)
- **`/api/memories/grant`** - Grant-based upload for large files
- **`/api/memories/complete`** - Complete upload process

### Asset Management

- **`/api/memories/[id]/assets`** - Asset retrieval and management

## Error Handling

### Common Issues

1. **Missing Token**: `BLOB_READ_WRITE_TOKEN` not set
2. **Upload Failure**: Network or permission issues
3. **URL Reconstruction**: Cannot reconstruct URLs from keys alone

### Error Messages

```typescript
"Vercel Blob is not available. BLOB_READ_WRITE_TOKEN is required.";
"Failed to upload to Vercel Blob: [error details]";
"Cannot reconstruct Vercel Blob URL from key alone. Store the full URL.";
```

## Performance Considerations

### File Size Limits

- **Direct Upload**: < 4MB recommended
- **Grant Upload**: > 4MB files
- **CDN**: Fast global delivery via Vercel's CDN

### Optimization

- **WebP Conversion**: All images converted to WebP for better compression
- **Multiple Assets**: Separate optimized versions for different use cases
- **Parallel Uploads**: Multiple assets uploaded simultaneously

## Testing

### Test Files

- **`src/lib/storage/test-blob-upload.ts`** - Vercel Blob upload tests
- **`tests/storage-edge-creation.test.ts`** - Storage edge creation tests

### Test Scenarios

- Direct upload with Vercel Blob
- Multiple backend uploads
- Asset creation and retrieval

## Migration and Deployment

### Local Development

- Requires `BLOB_READ_WRITE_TOKEN` environment variable
- Uses Vercel Blob for all uploads by default

### Production

- Automatic CDN distribution
- Global edge locations
- High availability and reliability

## Future Considerations

### Potential Improvements

1. **Hybrid Storage**: Combine Vercel Blob with other backends
2. **Cost Optimization**: Implement storage tiering
3. **Performance**: Add caching layers
4. **Monitoring**: Add upload metrics and monitoring

### Alternative Backends

- **AWS S3**: For large files and cost optimization
- **ICP**: For decentralized storage
- **Arweave**: For permanent storage
- **IPFS**: For distributed storage

## Troubleshooting

### Common Problems

1. **Upload Failures**: Check token permissions and network connectivity
2. **URL Issues**: Store full URLs, not just keys
3. **Size Limits**: Use grant-based uploads for large files
4. **Performance**: Monitor upload times and optimize file sizes

### Debug Commands

```bash
# Check Vercel Blob CLI
vercel --version

# Test upload
curl -X POST /api/memories/grant -F "file=@test.jpg"
```

## Folder Structure and Organization

### Current Blob Storage Structure

Based on analysis of the current blob storage, files are organized as follows:

```
blob-vercel (your blob store)
├── futura/ (438 files, 534.88 MB)
│   ├── User-uploaded files from Futura app
│   ├── Processed image assets (original, display, thumb)
│   └── All files uploaded through the application
├── fotokotti/ (519 files, 187.09 MB)
│   ├── Assets from Fotokotti project
│   ├── Product images, logos, service images
│   └── Static assets for the Fotokotti website
└── root/ (1 file, 0.02 MB)
    └── Miscellaneous files
```

### File Organization by Source

#### `futura/` Folder (configurable via `BLOB_FOLDER_NAME`)

- **Source**: Futura Alpha ICP application
- **Content**: All user-uploaded files, processed images, memory assets
- **Naming Pattern**: `{BLOB_FOLDER_NAME}/{timestamp}-{original-filename}` (default: `futura/`)
- **File Types**: Images, documents, videos, processed assets
- **Size**: 534.88 MB (438 files)

#### `fotokotti/` Folder

- **Source**: Fotokotti project (separate application)
- **Content**: Product images, service images, logos, static assets
- **Naming Pattern**: `fotokotti/{category}/{subcategory}/{filename}`
- **File Types**: Product photos, service images, logos, UI assets
- **Size**: 187.09 MB (519 files)

### File Upload Paths in Code

#### Grant-Based Uploads (`/api/memories/grant`)

```typescript
const uniqueFilename = generateBlobFilename(filename);
```

#### Image Processing Uploads (`image-processing.ts`)

```typescript
put(originalFileName, processedAssets.original.blob, {
put(displayFileName, processedAssets.display.blob, {
put(thumbFileName, processedAssets.thumb.blob, {
```

#### Blob Configuration (`blob-config.ts`)

```typescript
// Centralized configuration for blob folder names
export function getBlobFolderName(): string {
  return process.env.BLOB_FOLDER_NAME || "futura";
}

export function generateBlobFilename(originalFilename: string, addRandomSuffix = false): string {
  const folderName = getBlobFolderName();
  const timestamp = Date.now();
  // ... generates: {folderName}/{timestamp}-{baseName}.{extension}
}
```

## Management Scripts

### Available Scripts

Located in `scripts/blob/` directory:

#### 1. File Listing Script (`list-files.js`)

**Purpose**: Comprehensive analysis of blob storage contents

**Usage**:

```bash
node scripts/blob/list-files.js
```

**Features**:

- Storage statistics (total files, size, average file size)
- Files grouped by type and folder
- Recent uploads analysis
- Large files identification (>1MB)
- File type breakdown
- JSON export of all data

**Output Example**:

```
📊 Storage Statistics:
   Total files: 958
   Total size: 721.99 MB
   Average file size: 771.73 KB

📁 Files by type:
   unknown: 958 files (721.99 MB)

📂 Files by folder:
   fotokotti: 519 files (187.09 MB)
   uploads: 438 files (534.88 MB)
   root: 1 files (0.02 MB)
```

#### 2. Folder Deletion Script (`delete-folder.js`)

**Purpose**: Safely delete all files in a specific folder

**Usage**:

```bash
# Preview what would be deleted (safe)
node scripts/blob/delete-folder.js futura --dry-run

# Interactive deletion with confirmation
node scripts/blob/delete-folder.js futura
```

**Features**:

- Interactive confirmation (no command line flags required)
- Two-step confirmation process:
  1. "Are you sure you want to proceed? (yes/no)"
  2. "Type 'DELETE' to confirm deletion"
- Batch processing (10 files at a time)
- Progress tracking and error reporting
- Dry run mode for safe preview

**Safety Features**:

- ✅ Interactive prompts instead of command line flags
- ✅ Two-step confirmation process
- ✅ Clear warnings with file count and size
- ✅ Easy cancellation at any step
- ✅ Dry run mode for safe preview

#### 3. Test Script (`test-blob.js`)

**Purpose**: Test basic blob operations (upload, list, delete)

**Usage**:

```bash
node scripts/blob/test-blob.js
```

**Features**:

- Upload test files (text and JSON)
- List all blobs
- Delete test files
- Verify blob connectivity

### Script Dependencies

All scripts require:

- `@vercel/blob` package
- `dotenv` package for environment variable loading
- `BLOB_READ_WRITE_TOKEN` environment variable

### Environment Setup

Scripts automatically load environment variables from `.env.local`:

```bash
BLOB_READ_WRITE_TOKEN="vercel_blob_rw_..."
```

## Storage Management Best Practices

### File Organization

- **Application files**: Always use `futura/` folder (configurable via `BLOB_FOLDER_NAME`)
- **Static assets**: Use project-specific folders (e.g., `fotokotti/`)
- **Naming convention**: Include timestamps to prevent conflicts
- **File types**: Use appropriate content types for better organization

### Cleanup Strategies

- **Regular cleanup**: Use `list-files.js` to monitor storage usage
- **Selective deletion**: Use `delete-folder.js` for targeted cleanup
- **Backup before deletion**: Export data using `list-files.js` before major cleanup
- **Test first**: Always use `--dry-run` before actual deletion

### Monitoring

- **Storage usage**: Monitor total size and file count
- **Large files**: Identify files >1MB for optimization
- **Recent uploads**: Track upload patterns and frequency
- **Error handling**: Monitor failed uploads and deletions

## Related Documentation

- [Multiple Assets Implementation](./multiple-assets-implementation.md)
- [Upload Flow Refactoring](./issues/refactor-upload-flow-to-frontend-blob.md)
- [Storage Provider Analysis](./issues/icp-backend-upload-flow.md)
