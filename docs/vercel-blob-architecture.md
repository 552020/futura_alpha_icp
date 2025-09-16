# Vercel Blob Architecture & Implementation Guide

## Overview

This document outlines the architecture and implementation of Vercel Blob storage in the Futura application, incorporating best practices and recommendations from senior developers.

## Current Architecture

### Upload Flow Decision Matrix

Our application uses a size-based decision matrix to determine the optimal upload path:

```typescript
// From src/services/upload.ts
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

### Current Upload Paths

1. **Small Files (<4MB)**: Direct server-side upload via `/api/memories` endpoint
2. **Large Files (≥4MB)**: Grant-based upload flow via `/api/memories/grant` endpoint
3. **Images (Any Size)**: Multi-asset approach (original, display, thumb)

## Recommended Architecture (Senior Developer Input)

### Why Client-Side Uploads?

**Problem**: Serverless functions on Vercel have a 4.5MB request/response limit, which we're hitting with large files.

**Solution**: Use Vercel's official client-side upload flow that bypasses this limit and supports files up to 5TB.

### Benefits of Client-Side Upload Flow

- **No Size Limits**: Supports files up to 5TB (vs our current 100MB limit)
- **Better Performance**: Direct upload to blob storage, bypassing server
- **Multipart Support**: Automatic chunking and retry for large files
- **Progress Tracking**: Built-in upload progress for better UX
- **Security**: Server tokens never leave the server
- **Scalability**: Reduces server load for large file uploads

## Implementation Guide

### 1. Install Dependencies

```bash
pnpm i @vercel/blob
```

### 2. Client-Side Upload Component

```tsx
"use client";

import { upload } from "@vercel/blob/client";
import { useRef, useState } from "react";

export default function UploadPage() {
  const inputRef = useRef<HTMLInputElement>(null);
  const [pct, setPct] = useState(0);
  const [url, setUrl] = useState<string | null>(null);

  return (
    <form
      onSubmit={async (e) => {
        e.preventDefault();
        const f = inputRef.current?.files?.[0];
        if (!f) return;

        const blob = await upload(f.name, f, {
          access: "public", // or 'private'
          handleUploadUrl: "/api/blob/upload", // our route handler
          multipart: true, // big files: chunked + retries
          onUploadProgress: (ev) => setPct(ev.percentage ?? 0),
        });

        setUrl(blob.url);
      }}
    >
      <input type="file" ref={inputRef} required />
      <button type="submit">Upload</button>
      {pct > 0 && <div>{pct.toFixed(0)}%</div>}
      {url && (
        <p>
          <a href={url}>{url}</a>
        </p>
      )}
    </form>
  );
}
```

### 3. Upload Route Handler (Corrected Grant Flow)

**Important**: This replaces our current failing grant endpoint. The key insight is that `put()` is NOT the grant generator - we need to use `handleUpload()` instead.

```typescript
// src/app/api/memories/grant/route.ts (REPLACE current implementation)
import { NextResponse } from "next/server";
import { handleUpload, type HandleUploadBody } from "@vercel/blob/client";

export async function POST(req: Request) {
  const body = (await req.json()) as HandleUploadBody;

  // TODO: Implement authentication check here
  // const user = await getAuthenticatedUser(req);
  // if (!user) return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });

  const res = await handleUpload({
    request: req,
    body,
    onBeforeGenerateToken: async ({ pathname, multipart, clientPayload }) => ({
      // Control what's allowed:
      allowedContentTypes: [
        "image/*",
        "video/*",
        "application/pdf",
        "text/plain",
        "text/markdown",
        "application/msword",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
      ],
      maximumSizeInBytes: 5 * 1024 ** 4, // up to 5 TB
      addRandomSuffix: true,
      // Optionally carry context you'll receive on completion:
      tokenPayload: JSON.stringify({ userId: "..." }), // TODO: Use real user ID
    }),
    onUploadCompleted: async ({ blob, tokenPayload }) => {
      // Persist blob.url / metadata in your DB
      // tokenPayload is whatever you set above
      console.log("Upload completed:", blob.url);

      // TODO: Create memory record in database
      // await createMemoryFromBlob(blob, tokenPayload);
    },
  });

  return NextResponse.json(res);
}
```

**Why this works**: The server route returns a short-lived client token; the browser then uploads directly to Blob (bypassing the 4.5 MB function limit). The SDK takes care of `x-content-length`, part sizes, retries, and completion.

### Key Insights from Senior Developer

#### Root Cause of Our Error

- `put()` actually tries to upload something right now
- `Buffer.alloc(0)` with fake `content-length` confuses the multipart flow
- Blob expects the browser to send the real `x-content-length` during actual upload
- **`put()` is NOT the grant generator**

#### Answers to Original Questions

1. **Is `Buffer.alloc(0)` correct for grants?**

   - **No.** It triggers a real upload with zero bytes and mismatched headers. Use `handleUpload()` to issue tokens.

2. **Different method for grant-based uploads?**

   - **Yes:** `handleUpload()` on the server and `upload()` on the client (with `handleUploadUrl`).

3. **How to handle content-length?**

   - **Don't set it yourself.** The SDK/browser sets `x-content-length` during the real upload. Your server just validates and issues the token.

4. **Missing required headers/options?**
   - **Remove the fake `headers` on `put()`.** In `onBeforeGenerateToken`, specify size/content-type limits and (optionally) expiry. That's it.

#### Alternative: Manual Multipart Flow

If you need more control, use the multipart helpers instead of a dummy `put()`:

- `createMultipartUpload(pathname, { access, contentType })`
- Client uploads parts with `uploadPart(...)`
- Finish with `completeMultipartUpload(...)`

But this is lower-level; the tokenized client-upload flow above is the recommended path for browsers.

## Integration with Current System

### Replacing the Grant Endpoint

**Current Problem**: Our custom grant endpoint (`/api/memories/grant`) is failing with "Missing [x]-content-length header" error.

**Root Cause**: We're using `put()` with `Buffer.alloc(0)` which confuses the multipart flow. The `put()` function is NOT the grant generator.

**Solution**: Replace with `handleUpload()` which properly generates tokens for client-side uploads.

### Migration Strategy

1. **Phase 1**: Implement new client-side upload flow alongside existing system
2. **Phase 2**: Update frontend to use new upload flow for large files
3. **Phase 3**: Remove old grant endpoint
4. **Phase 4**: Integrate with existing memory creation flow

### Files to Update

Based on the senior developer's analysis, we need to update these specific files:

1. **`src/nextjs/src/app/api/memories/grant/route.ts`** (current implementation)

   - Replace `put()` with `handleUpload()`
   - Remove fake `content-length` headers
   - Add proper token generation

2. **Client upload hook/component** (where we currently call the grant)

   - Update to use `upload()` with `handleUploadUrl: '/api/memories/grant'`
   - Add multipart support and progress tracking

3. **Auth util used by the grant route**

   - Wire authentication correctly in `onBeforeGenerateToken`
   - Ensure only authenticated users can upload

4. **DB write path for storing `blob.url`**
   - Integrate with existing memory creation flow
   - Handle the `onUploadCompleted` callback properly

### Integration Points

#### Frontend Upload Service

```typescript
// Update src/services/upload.ts
async function uploadLargeFile(file: File, isOnboarding: boolean, existingUserId?: string) {
  // Use new client-side upload flow instead of grant endpoint
  const blob = await upload(file.name, file, {
    access: "public",
    handleUploadUrl: "/api/memories/grant", // Use corrected grant endpoint
    multipart: true, // chunked + parallel + retries
    onUploadProgress: (ev) => {
      // Update progress in UI
      console.log(`Upload progress: ${ev.percentage}%`);
    },
  });

  // Create memory with blob URL
  return await createMemoryFromBlob(blob, { isOnboarding, existingUserId });
}
```

#### Memory Creation Integration

```typescript
// New function to create memory from blob
async function createMemoryFromBlob(blob: any, metadata: any) {
  const response = await fetch("/api/memories", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      type: getMemoryTypeFromFile(blob),
      title: blob.pathname.split("/").pop() || "Untitled",
      description: "",
      fileCreatedAt: new Date().toISOString(),
      isPublic: false,
      isOnboarding: metadata.isOnboarding,
      existingUserId: metadata.existingUserId,
      assets: [
        {
          assetType: "original",
          url: blob.url,
          bytes: blob.size,
          mimeType: blob.contentType,
          storageBackend: "vercel_blob",
          storageKey: blob.pathname,
        },
      ],
    }),
  });

  return await response.json();
}
```

## Configuration

### Environment Variables

```bash
# Required
BLOB_READ_WRITE_TOKEN="vercel_blob_rw_..."

# Optional - for custom folder structure
BLOB_FOLDER_NAME="futura"
```

### Blob Storage Structure

```
futura/
├── images/
│   ├── original/
│   ├── display/
│   └── thumb/
├── videos/
├── documents/
└── audio/
```

## Development Considerations

### Local Development

**Important**: The `onUploadCompleted` callback can't call back to localhost in local development. Use a tunnel (e.g., ngrok) to test the full round-trip.

```bash
# Install ngrok
npm install -g ngrok

# Start your Next.js app
npm run dev

# In another terminal, create tunnel
ngrok http 3000

# Use the ngrok URL for testing uploads
```

### File Size Limits

- **Client-side uploads**: Up to 5TB with multipart
- **Server-side uploads**: 4.5MB limit (Vercel serverless functions)
- **Recommended threshold**: 4MB (current implementation)

### Security Considerations

1. **Authentication**: Always verify user authentication in `onBeforeGenerateToken`
2. **Content Types**: Restrict allowed file types
3. **File Size**: Set appropriate size limits
4. **Token Payload**: Include user context for audit trails

## Management Scripts

### List Files in Blob Storage

```bash
# List all files
node scripts/blob/list-all-files.js

# List files in specific folder
node scripts/blob/list-folder-files.js futura
```

### Delete Files

```bash
# Delete all files in a folder (with confirmation)
node scripts/blob/delete-folder-files.js futura
```

## Troubleshooting

### Common Issues

1. **"Missing [x]-content-length header"**: Use official client-side upload flow instead of custom grant endpoint
2. **Local development callbacks**: Use ngrok tunnel for testing
3. **File size limits**: Ensure using client-side uploads for files >4MB
4. **Authentication errors**: Verify user authentication in upload handler

### Debugging

```typescript
// Enable detailed logging
console.log("Upload request:", { filename, size, contentType });
console.log("Blob response:", blob);
console.log("Upload progress:", ev.percentage);
```

## Future Enhancements

### Potential Improvements

1. **Resumable Uploads**: Consider TUS protocol for pause/resume functionality
2. **Image Processing**: Integrate with existing multi-asset processing
3. **CDN Integration**: Add CDN for faster file delivery
4. **Analytics**: Track upload metrics and performance
5. **Backup Strategy**: Implement cross-region replication

### When to Consider TUS/Uppy

- Need true pause/resume across page reloads
- Long offline periods
- Very large files (>1GB) with unreliable connections
- Advanced upload management features

For most use cases, Vercel Blob's built-in multipart upload is sufficient.

## References

- [Vercel Blob Documentation](https://vercel.com/docs/storage/vercel-blob)
- [Client Uploads with Vercel Blob](https://vercel.com/docs/vercel-blob/client-upload)
- [Vercel Functions Limitations](https://vercel.com/docs/functions/limitations)
- [File Upload Progress](https://vercel.com/changelog/vercel-blob-now-supports-file-upload-progress)
- [5TB File Transfers](https://vercel.com/changelog/5tb-file-transfers-with-vercel-blob-multipart-uploads)
