# Senior Developer Context - Vercel Blob Grant Implementation

## Overview

We need help implementing the corrected Vercel Blob grant flow using `handleUpload()` instead of our current failing `put()` approach. Here's the complete context of our current implementation.

## Current Problem

**Error**: `Vercel Blob: Missing [x]-content-length header`  
**Location**: `src/nextjs/src/app/api/memories/grant/route.ts` line 69  
**Root Cause**: Using `put()` with `Buffer.alloc(0)` confuses the multipart flow

## Files to Update (As Requested by Senior)

### 1. `src/nextjs/src/app/api/memories/grant/route.ts` (Current Implementation)

**Current Code**:

```typescript
import { NextRequest, NextResponse } from "next/server";
import { put } from "@vercel/blob";
import { generateBlobFilename } from "@/lib/storage/blob-config";

interface GrantRequest {
  filename: string;
  size: number;
  mimeType: string;
  checksum?: string;
}

interface GrantResponse {
  success: boolean;
  uploadUrl: string;
  token: string;
  expiresAt: string;
  maxSize: number;
  allowedMimeTypes: string[];
}

export async function POST(request: NextRequest) {
  try {
    const body: GrantRequest = await request.json();
    const { filename, size, mimeType, checksum } = body;

    // Validate request
    if (!filename || !size || !mimeType) {
      return NextResponse.json({ error: "Missing required fields: filename, size, mimeType" }, { status: 400 });
    }

    // Check file size limits (4MB for small files, 100MB for large files)
    const maxSize = size > 4 * 1024 * 1024 ? 100 * 1024 * 1024 : 4 * 1024 * 1024;
    if (size > maxSize) {
      return NextResponse.json({ error: `File too large. Max size: ${maxSize / (1024 * 1024)}MB` }, { status: 400 });
    }

    // Validate MIME type
    const allowedMimeTypes = [
      "image/jpeg",
      "image/png",
      "image/gif",
      "image/webp",
      "image/svg+xml",
      "video/mp4",
      "video/webm",
      "video/quicktime",
      "application/pdf",
      "text/plain",
      "text/markdown",
      "application/msword",
      "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    ];

    if (!allowedMimeTypes.includes(mimeType)) {
      return NextResponse.json({ error: `Unsupported file type: ${mimeType}` }, { status: 400 });
    }

    // Generate unique filename with timestamp
    const uniqueFilename = generateBlobFilename(filename);

    // ❌ PROBLEMATIC CODE - This is what's failing
    const blob = await put(uniqueFilename, Buffer.alloc(0), {
      access: "public",
      contentType: mimeType,
      addRandomSuffix: false,
      headers: {
        "content-length": size.toString(), // This confuses the multipart flow
      },
    });

    // Create token payload (in a real implementation, this would be signed)
    const tokenPayload = {
      filename: uniqueFilename,
      originalFilename: filename,
      mimeType,
      size,
      checksum,
      expiresAt: Date.now() + 60 * 60 * 1000, // 1 hour expiry
      userId: "temp", // TODO: Get from session/auth
    };

    const response: GrantResponse = {
      success: true,
      uploadUrl: blob.url,
      token: JSON.stringify(tokenPayload), // TODO: Sign this token
      expiresAt: new Date(tokenPayload.expiresAt).toISOString(),
      maxSize,
      allowedMimeTypes,
    };

    console.log(`✅ Upload grant issued for ${filename} (${size} bytes, ${mimeType})`);
    return NextResponse.json(response);
  } catch (error) {
    console.error("❌ Upload grant failed:", error);
    return NextResponse.json({ error: "Failed to generate upload grant" }, { status: 500 });
  }
}
```

**Issues**:

- Using `put()` instead of `handleUpload()`
- `Buffer.alloc(0)` with fake `content-length` header
- Custom token generation instead of using Vercel's built-in flow

### 2. Client Upload Hook/Component

**File**: `src/nextjs/src/services/upload.ts`

**Relevant Function**:

```typescript
/**
 * Upload files using multi-asset approach (for images) or direct-to-blob (for large files)
 */
async function uploadLargeFile(
  file: File,
  isOnboarding: boolean,
  existingUserId?: string,
  mode: UploadMode = "files"
): Promise<UploadResponse> {
  console.log(`☁️ Using multi-asset upload approach for: ${file.name}`);

  // This function currently calls our grant endpoint
  // We need to update it to use the new client-side upload flow

  // Current flow:
  // 1. Process image for multiple assets (if image)
  // 2. Upload each asset to blob storage
  // 3. Call /api/memories with asset URLs

  // New flow should be:
  // 1. Use @vercel/blob/client upload() with handleUploadUrl: '/api/memories/grant'
  // 2. Handle multipart uploads and progress
  // 3. Create memory with blob URL
}
```

**Current Upload Decision Logic**:

```typescript
// From uploadFile function
const fileSizeMB = file.size / (1024 * 1024);
const isLargeFile = fileSizeMB > 4; // 4MB threshold
const isImage = file.type.startsWith("image/");

if (isImage) {
  // Images always use multi-asset approach (original, display, thumb)
  return await uploadLargeFile(file, isOnboarding, existingUserId, mode);
} else if (isLargeFile) {
  // Large files use direct-to-blob with server tokens
  return await uploadLargeFile(file, isOnboarding, existingUserId, mode);
} else {
  // Small files use server-side upload
  return await uploadSmallFile(file);
}
```

### 3. Auth Util Used by Grant Route

**File**: `src/nextjs/src/app/api/memories/utils/user-utils.ts`

**Relevant Functions**:

```typescript
/**
 * Get user ID for upload operations (authenticated or temporary)
 */
export async function getUserIdForUpload({ providedUserId }: { providedUserId?: string }) {
  // Returns { allUserId: string, error?: NextResponse }
  // Handles both authenticated users and temporary users for onboarding
}

/**
 * Get user ID from request (for authenticated users)
 */
export async function getAllUserId(request: NextRequest) {
  // Returns { allUserId: string, error?: NextResponse }
  // Extracts user ID from session/auth
}
```

**Current Usage in Grant Route**:

```typescript
// TODO: We need to wire this authentication properly
// Currently hardcoded as 'temp'
const tokenPayload = {
  // ...
  userId: "temp", // TODO: Get from session/auth
};
```

### 4. DB Write Path for Storing blob.url

**File**: `src/nextjs/src/app/api/memories/post.ts`

**Relevant Function**:

```typescript
/**
 * Handle memory creation from JSON (after blob upload)
 */
export async function createMemoryFromJson(request: NextRequest, allUserId: string): Promise<NextResponse> {
  // This function creates memory records in the database
  // It's called after blob upload to persist the memory metadata
  // We need to integrate this with the onUploadCompleted callback
  // in the new grant flow
}
```

**Current Memory Creation Flow**:

```typescript
// From uploadLargeFile function
const response = await fetch("/api/memories", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({
    type: memoryType,
    title: file.name.split(".")[0] || "Untitled",
    description: "",
    fileCreatedAt: new Date().toISOString(),
    isPublic: false,
    isOnboarding,
    mode,
    existingUserId,
    assets, // Array of asset objects with blob URLs
  }),
});
```

## Current Architecture Overview

### Upload Flow Decision Matrix

```
File Upload Request
├── Is Image? → Multi-asset approach (original, display, thumb)
├── Is Large (>4MB)? → Grant-based upload flow
└── Is Small (<4MB)? → Server-side upload
```

### Current Grant Flow (Broken)

```
Client → POST /api/memories/grant → put() with Buffer.alloc(0) → ❌ Error
```

### Desired Grant Flow (Fixed)

```
Client → upload() with handleUploadUrl → handleUpload() → ✅ Success
```

## Integration Points

### 1. Image Processing

**File**: `src/nextjs/src/app/api/memories/utils/image-processing.ts`

- Currently processes images for multiple assets (original, display, thumb)
- We need to decide how to integrate this with the new client-side upload flow

### 2. Storage Manager

**File**: `src/nextjs/src/lib/storage.ts`

- Currently handles blob storage operations
- May need updates to work with new client-side flow

### 3. Memory Creation

**File**: `src/nextjs/src/app/api/memories/post.ts`

- Handles memory creation after blob upload
- Needs integration with `onUploadCompleted` callback

## Questions for Senior Developer

1. **How do we handle multi-asset image processing** with the new client-side flow?

   - Currently we process images on the frontend and upload 3 versions
   - Should we still do this, or move to backend processing?

2. **How do we integrate with existing memory creation flow**?

   - Currently we call `/api/memories` after blob upload
   - Should we use the `onUploadCompleted` callback instead?

3. **How do we handle authentication** in the new flow?

   - Currently we have `getUserIdForUpload` and `getAllUserId` functions
   - How do we wire these into `onBeforeGenerateToken`?

4. **How do we maintain our current file size decision matrix**?
   - We have different paths for small vs large files
   - Should we keep this, or use client-side uploads for everything?

## Environment Variables

```bash
BLOB_READ_WRITE_TOKEN="vercel_blob_rw_..."
BLOB_FOLDER_NAME="futura"
```

## Dependencies

```json
{
  "@vercel/blob": "^0.19.0"
}
```

## Expected Outcome

We need exact patches to:

1. Replace the failing grant endpoint with `handleUpload()`
2. Update the client upload service to use the new flow
3. Wire authentication properly
4. Integrate with existing memory creation flow

The goal is to fix the "Missing [x]-content-length header" error while maintaining our current architecture and adding multipart upload support with progress tracking.
