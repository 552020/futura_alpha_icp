# Vercel Blob onUploadCompleted Callback Not Working in Local Development

## Problem Description

After implementing the Vercel Blob grant-based upload flow, files are successfully uploaded to blob storage but do not appear in the dashboard. The issue is that the `onUploadCompleted` callback is not being triggered in local development.

## Root Cause

The `onUploadCompleted` callback in Vercel Blob's **client-side upload flow** (`@vercel/blob/client`) **cannot call back to localhost** in local development. This is a known limitation documented in the Vercel Blob documentation.

### Client-Side vs Server-Side Upload Differences

#### **Server-Side Uploads** (No Problem)

- Uses `put()` from `@vercel/blob` package
- Upload happens from server → Vercel Blob
- No callback limitations
- Works perfectly in local development
- **Limitation**: 4.5MB serverless function request/response limit
- **What this limit applies to**:
  - **Request payload**: The entire HTTP request body sent to the serverless function
  - **Response payload**: The entire HTTP response body returned from the serverless function
  - **Includes**: File data + headers + any other request/response data
- **What this means**:
  - ✅ Files ≤4.5MB: Work perfectly (request fits within limit)
  - ❌ Files >4.5MB: **Cannot be uploaded through serverless functions** (request exceeds limit)
  - **Chunked uploads won't help**: Each chunk would still need to go through a serverless function
  - **Manual chunking required**: You'd need to implement your own chunking system
- **Example**:
  ```
  File size: 6MB
  Request to /api/memories: 6MB + headers = ~6MB
  Result: ❌ FAILS - exceeds 4.5MB limit
  ```

### Why Chunked Uploads Don't Solve the Serverless Function Limit

**The problem**: Each chunk would still need to go through a serverless function, and you'd need to:

1. **Split the file** into chunks ≤4.5MB each
2. **Upload each chunk** through a separate serverless function call
3. **Reassemble chunks** on the server side
4. **Handle partial failures** and retries manually

**Example of manual chunking**:

```typescript
// This is what you'd need to implement manually
const chunkSize = 4 * 1024 * 1024; // 4MB chunks
const chunks = splitFileIntoChunks(file, chunkSize);

for (const chunk of chunks) {
  // Each chunk still goes through serverless function
  await fetch('/api/upload-chunk', {
    method: 'POST',
    body: chunk // Still limited to 4.5MB
  });
}

// Then reassemble on server
await fetch('/api/reassemble-chunks', {
  method: 'POST',
  body: JSON.stringify({ chunkIds: [...] })
});
```

**Why this is problematic**:

- ❌ **Complex implementation**: You need to handle chunking, reassembly, error recovery
- ❌ **Multiple serverless function calls**: Each chunk = one function call
- ❌ **Higher costs**: More function invocations
- ❌ **Error handling**: What if one chunk fails?
- ❌ **No progress tracking**: You'd need to implement this yourself

### Alternative: Real Backend Server

**Yes!** The 4.5MB limit only applies to **serverless functions** (like Vercel Functions, AWS Lambda, etc.).

**With a real backend server**, you can:

- ✅ **Upload files of any size** (no 4.5MB limit)
- ✅ **Use server-side uploads** with `put()` from `@vercel/blob`
- ✅ **Handle large files** without chunking complexity
- ✅ **Work in local development** without callback issues

**Examples of real backends**:

- **Node.js/Express server** (not serverless)
- **Docker containers** on platforms like Railway, Render, DigitalOcean
- **Traditional VPS** (AWS EC2, DigitalOcean Droplets)
- **Container platforms** (Google Cloud Run, AWS ECS)

**Trade-offs**:

- ❌ **Higher costs**: Always-on server vs pay-per-request
- ❌ **More complexity**: Server management, scaling, monitoring
- ❌ **Less "serverless"**: You manage the infrastructure
- ✅ **No size limits**: Upload files of any size
- ✅ **Full control**: Custom logic, file processing, etc.
- Example:

```typescript
// Server-side - works for files ≤4.5MB
import { put } from "@vercel/blob";
const blob = await put(filename, file, { access: "public" });
// No callback needed - upload is synchronous
// BUT: Fails if file > 4.5MB due to serverless function limits
```

#### **Client-Side Uploads** (Localhost Problem)

- Uses `upload()` from `@vercel/blob/client` package
- Upload happens from browser → Vercel Blob
- Requires `onUploadCompleted` callback for post-processing
- **Callback cannot reach localhost** in local development
- Example:

```typescript
// Client-side - callback fails on localhost
import { upload } from "@vercel/blob/client";
const blob = await upload(filename, file, {
  handleUploadUrl: "/api/memories/grant",
  // This callback won't work on localhost:
  onUploadCompleted: async ({ blob }) => {
    // Create memory record - FAILS on localhost
  },
});
```

### Why We Use Client-Side Uploads

We chose client-side uploads because they provide:

1. **Large File Support**: Up to 5TB (vs 4.5MB server-side limit)
2. **Better Performance**: Direct browser → blob upload (bypasses serverless function limits)
3. **Progress Tracking**: Real-time upload progress for users
4. **Automatic Multipart**: Built-in chunking and retries for large files
5. **Better UX**: No serverless function request/response limits
6. **Scalability**: Reduces server load and bandwidth

**Trade-off**: Requires `onUploadCompleted` callback for database operations, which doesn't work on localhost.

### File Size Comparison

| Upload Method   | Max File Size | Chunking  | Local Dev         | Production |
| --------------- | ------------- | --------- | ----------------- | ---------- |
| **Server-Side** | 4.5MB         | Manual    | ✅ Works          | ✅ Works   |
| **Client-Side** | 5TB           | Automatic | ⚠️ Callback issue | ✅ Works   |

### Detailed Flow Comparison

#### **Server-Side Upload Flow (4.5MB limit)**

```
1. Browser → /api/memories (with file in request body)
   - Request size: File size + headers
   - If file > 4.5MB: ❌ REQUEST TOO LARGE

2. Serverless function → Vercel Blob
   - Uses put() to upload file
   - No size limits here

3. Serverless function → Browser
   - Response with blob URL
   - Response size: Small (just the URL)
```

#### **Client-Side Upload Flow (5TB limit)**

```
1. Browser → /api/memories/grant (request for upload token)
   - Request size: Small (just metadata)
   - No file data in this request

2. Browser → Vercel Blob (direct upload)
   - Bypasses serverless functions entirely
   - File size: Up to 5TB with automatic chunking

3. Vercel Blob → /api/memories/grant (onUploadCompleted callback)
   - Callback with blob URL
   - This is where localhost fails
```

## Current Flow (Broken in Local Dev)

1. ✅ **Client uploads file** → Vercel Blob storage
2. ✅ **Upload completes** → Blob URL generated
3. ✅ **Client receives success** → Dashboard refresh triggered
4. ❌ **`onUploadCompleted` callback** → **NOT TRIGGERED** (localhost limitation)
5. ❌ **Memory creation** → **NOT EXECUTED** (depends on callback)
6. ❌ **File appears in dashboard** → **NO** (no memory record in database)

## Evidence

### Frontend Console Logs (Working)

```
✅ Client-side upload successful: https://mt0fstomhelqwqj.j.public.blob.vercel-storage.com/white-room-LlGpZ028NI9i079...
► Fetch finished loading: GET "http://localhost:3000/api/memories?page=1"
```

### Backend Console Logs (Missing)

```
❌ Missing: 🎉 onUploadCompleted callback triggered!
❌ Missing: 📦 Parsed token payload: {...}
❌ Missing: 📝 Memory creation result: {...}
❌ Missing: ✅ Memory created from blob: {...}
```

### Grant Endpoint Logs (Partial)

```
📦 Client payload received: {}  // Empty - this is also an issue
POST /api/memories/grant 200 in 1283ms
```

## Files Affected

- **`src/nextjs/src/app/api/memories/grant/route.ts`** - Grant endpoint with `onUploadCompleted` callback
- **`src/nextjs/src/services/upload.ts`** - Client-side upload service
- **`src/nextjs/src/app/api/memories/utils/memory-creation.ts`** - Memory creation function

## Proposed Solutions

### Option 1: Client-Side Memory Creation (Recommended for Local Dev)

Modify the client-side upload to handle memory creation directly after upload completion:

```typescript
// In uploadFileToBlob function
const blob = await blobUpload(file.name, file, {
  // ... existing config
});

// After successful upload, create memory record
const memoryResponse = await fetch("/api/memories", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({
    blobUrl: blob.url,
    filename: file.name,
    contentType: file.type,
    // ... other metadata
  }),
});

return await memoryResponse.json();
```

### Option 2: Use ngrok for Local Development

Set up ngrok tunnel to make localhost accessible to Vercel Blob callbacks:

```bash
# Install ngrok
npm install -g ngrok

# Start tunnel
ngrok http 3000

# Update handleUploadUrl to use ngrok URL
handleUploadUrl: 'https://abc123.ngrok.io/api/memories/grant'
```

### Option 3: Deploy to Staging

Test the full flow on a deployed environment where callbacks work properly.

## Implementation Priority

1. **Immediate**: Implement Option 1 (client-side memory creation) for local development
2. **Short-term**: Set up ngrok for full callback testing
3. **Long-term**: Ensure production deployment works with server-side callbacks

## Testing Strategy

1. **Local Development**: Use client-side memory creation
2. **Staging/Production**: Use server-side `onUploadCompleted` callbacks
3. **Image Processing**: Test async image processing in deployed environment

## How UploadThing Solves This Problem

UploadThing faces the **exact same issue** and has documented solutions:

### **UploadThing's Approach:**

1. **Development**: Simulates callbacks locally (no external server needed)
2. **Production**: Uses external server to trigger callbacks
3. **Local Testing**: Recommends ngrok for full callback testing

### **UploadThing's FAQ on Localhost Callbacks:**

> "In order for UploadThing to work, our external server must be able to reach your application to trigger the callbacks you have set up in your file router. This is not possible if your application is running on localhost."

> "When you're running in development, UploadThing will simulate this callback for you so that you can test your application."

### **UploadThing's Solutions:**

1. **Development**: Built-in callback simulation
2. **Local Testing**: Use ngrok or Cloudflare Tunnels
3. **Environment Variable**: `UPLOADTHING_CALLBACK_URL` for custom URLs

## Related Documentation

- [Vercel Blob Client Uploads](https://vercel.com/docs/vercel-blob/client-upload)
- [Local Development Limitations](https://vercel.com/docs/vercel-blob/client-upload#local-development)
- [UploadThing FAQ - Localhost Callbacks](https://docs.uploadthing.com/faq#my-callback-runs-in-development-but-not-in-production)

## Status

- **Issue Identified**: ✅ Root cause confirmed
- **Workaround Available**: ✅ Client-side memory creation
- **Production Ready**: ⏳ Needs testing in deployed environment
