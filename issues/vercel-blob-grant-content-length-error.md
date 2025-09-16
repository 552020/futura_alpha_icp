# Vercel Blob Grant Upload: Missing [x]-content-length Header Error

## Problem Description

When uploading large files through the grant endpoint (`/api/memories/grant`), we're encountering the following error:

```
Upload grant failed: Error: Vercel Blob: Missing [x]-content-length header.
```

## Location

**File**: `src/nextjs/src/app/api/memories/grant/route.ts`  
**Function**: `POST` handler (line ~69)  
**Error occurs at**: `put()` call from `@vercel/blob`

## Current Implementation

```typescript
// Generate presigned URL for Vercel Blob
const blob = await put(uniqueFilename, Buffer.alloc(0), {
  access: "public",
  contentType: mimeType,
  addRandomSuffix: false,
  // Add content-length header explicitly
  headers: {
    "content-length": size.toString(),
  },
});
```

## Context

### Upload Flow Architecture

Our application has two distinct upload paths based on file size:

1. **Small Files (<4MB)**: Direct server-side upload via `/api/memories` endpoint
2. **Large Files (≥4MB)**: Grant-based upload flow via `/api/memories/grant` endpoint

### Grant-Based Upload Flow (3-step process)

1. **Step 1**: Client requests upload grant from `/api/memories/grant` with file metadata
2. **Step 2**: Server generates presigned URL and returns it to client
3. **Step 3**: Client uploads file directly to Vercel Blob using presigned URL

### Current Implementation Details

- The grant endpoint is supposed to generate a presigned URL for direct client-side upload
- We're using `Buffer.alloc(0)` as a placeholder since we're not actually uploading the file content at this stage
- The `size` parameter comes from the client request and represents the actual file size
- This approach avoids exposing server credentials to the client while enabling direct uploads

### Why Grant Flow is Used

- **Security**: Server tokens never leave the server
- **Performance**: Direct upload to blob storage, bypassing server
- **Scalability**: Reduces server load for large file uploads
- **User Experience**: Faster uploads for large files

## Attempted Fix

I tried adding an explicit `content-length` header to the `put()` options, but this may not be the correct approach for the Vercel Blob grant flow.

## Questions for Senior Developer

1. **Is `Buffer.alloc(0)` the correct approach** for generating a presigned URL in the grant flow?
2. **Should we be using a different Vercel Blob method** for grant-based uploads? (e.g., `createUploadUrl()` instead of `put()`)
3. **What's the proper way to handle content-length** in the grant flow?
4. **Are we missing any required headers or options** for the grant endpoint?
5. **Is there a specific Vercel Blob API** designed for grant-based uploads that we should be using instead?

## Code Flow Context

### Frontend Upload Decision Logic

```typescript
// From src/services/upload.ts
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

### Grant Endpoint Request Format

The client sends a POST request to `/api/memories/grant` with:

```typescript
{
  filename: string,
  contentType: string,
  size: number,
  userId?: string,
  isOnboarding?: boolean
}
```

### Expected Response Format

The grant endpoint should return:

```typescript
{
  url: string,        // Presigned URL for direct upload
  downloadUrl: string // URL for accessing the uploaded file
}
```

## Expected Behavior

The grant endpoint should successfully generate a presigned URL that allows the client to upload the file directly to Vercel Blob without exposing server credentials.

## Error Details

- **Error Type**: `Error: Vercel Blob: Missing [x]-content-length header`
- **Trigger**: Large file uploads (>4MB) that go through the grant flow
- **Impact**: Prevents large file uploads from working

## Related Documentation

- [Vercel Blob Documentation](https://vercel.com/docs/storage/vercel-blob)
- [Grant-based Upload Flow](https://vercel.com/docs/storage/vercel-blob/using-blob-sdk#grant-based-uploads)

## Environment

- **Framework**: Next.js 14
- **Blob SDK**: `@vercel/blob`
- **Deployment**: Vercel
- **File Size Threshold**: 4MB (configurable)
