# Storage Provider Failure Issue

## Problem Description

Users are experiencing upload failures with the error message:

```
All storage providers failed. Available: 0, Tried: 0. Available providers:
```

This error occurs when trying to upload files through the dashboard, preventing users from storing their memories.

## Error Details

### Error Stack Trace

```
All storage providers failed. Available: 0, Tried: 0. Available providers:
    at StorageManager.uploadWithFallback (src/lib/storage/storage-manager.ts:191:11)
    at StorageManager.uploadToSingleProvider (src/lib/storage/storage-manager.ts:94:19)
    at StorageManager.upload (src/lib/storage/storage-manager.ts:75:19)
    at uploadFile (src/services/upload.ts:208:24)
    at async processSingleFile (src/hooks/user-file-upload.ts:177:28)
    at async handleFileUpload (src/hooks/user-file-upload.ts:384:7)
```

### Root Cause Analysis

1. **Storage Manager Initialization**: The `StorageManager` is not finding any available storage providers
2. **Environment Variables**: The `BLOB_READ_WRITE_TOKEN` may not be properly loaded in the client-side context
3. **Provider Registration**: Only Vercel Blob provider is registered, but it's not being detected as available

## Environment Configuration

### Current Environment Variables

```bash
BLOB_READ_WRITE_TOKEN="vercel_blob_rw_Mt0fstoMHElqwQJj_U6BIUrzN0sJVYOS4Ss7MY0hYMOQrm1"
```

### Expected Working Providers

- **Vercel Blob**: Primary storage provider (should work with `BLOB_READ_WRITE_TOKEN`)
- **Neon Database**: For metadata storage (not file storage)

## Debugging Steps Added

### 1. Storage Manager Initialization Logs

Added detailed logging in `StorageManager.initializeProviders()`:

```typescript
console.log("🔧 Initializing storage providers...");
console.log("🔍 BLOB_READ_WRITE_TOKEN exists:", !!process.env.BLOB_READ_WRITE_TOKEN);
console.log("🔍 Vercel Blob isAvailable:", vercelBlobProvider.isAvailable());
```

### 2. Upload Service Logs

Added parameter logging in `uploadFile()`:

```typescript
console.log(`🔍 Upload parameters:`, {
  fileName: file.name,
  fileSize: file.size,
  fileType: file.type,
  isOnboarding,
  existingUserId,
  mode,
  storageBackend,
  userStoragePreference,
});
```

### 3. File Upload Hook Logs

Added processing logs in `processSingleFile()`:

```typescript
console.log(`📁 Processing single file: ${file.name} (${file.size} bytes)`);
console.log(`🔍 User storage preference: ${userStoragePreference}`);
```

## Potential Issues

### 1. Client-Side Environment Variables

The `BLOB_READ_WRITE_TOKEN` might not be available in the client-side context. Vercel Blob operations should typically happen server-side.

### 2. Provider Availability Check

The `isAvailable()` method in `VercelBlobProvider` checks for `process.env.BLOB_READ_WRITE_TOKEN`, but this might not be available in the browser context.

### 3. Storage Architecture Mismatch

The current implementation tries to upload files directly from the client to blob storage, but this should typically be done through a server-side API endpoint.

## Recommended Solutions

### 1. Move Blob Upload to Server-Side

- Create a server-side API endpoint for file uploads
- Handle blob storage operations on the server
- Pass file data from client to server

### 2. Fix Environment Variable Access

- Ensure `BLOB_READ_WRITE_TOKEN` is available where needed
- Use `NEXT_PUBLIC_` prefix for client-side environment variables if needed

### 3. Implement Proper Error Handling

- Add fallback mechanisms for storage failures
- Provide user-friendly error messages
- Implement retry logic

## Files Modified for Debugging

1. `src/lib/storage/storage-manager.ts` - Added initialization and availability logging
2. `src/services/upload.ts` - Added parameter and flow logging
3. `src/hooks/user-file-upload.ts` - Added file processing logging

## Next Steps

1. **Test with Debug Logs**: Run the application and check console logs to identify where the failure occurs
2. **Verify Environment Variables**: Ensure `BLOB_READ_WRITE_TOKEN` is properly loaded
3. **Consider Architecture**: Evaluate if client-side blob uploads are the right approach
4. **Implement Server-Side Upload**: Move blob operations to API endpoints

## Related Files

- `src/lib/storage/storage-manager.ts` - Storage manager implementation
- `src/lib/storage/providers/vercel-blob.ts` - Vercel Blob provider
- `src/services/upload.ts` - Upload service
- `src/hooks/user-file-upload.ts` - File upload hook
- `src/components/memory/item-upload-button.tsx` - Upload UI component

## Current Implementation Flow

### What We Changed (MAJOR ARCHITECTURE CHANGE)

We completely replaced the working upload system with a new "blob-first" approach:

**ORIGINAL WORKING FLOW (main branch):**

```typescript
// Simple FormData upload to server endpoints
export const uploadFile = async (
  file: File,
  isOnboarding: boolean,
  existingUserId?: string,
  mode: UploadMode = "files"
) => {
  const endpoint = isOnboarding
    ? mode === "folder"
      ? "/api/memories/upload/onboarding/folder"
      : "/api/memories/upload/onboarding/file"
    : mode === "folder"
    ? "/api/memories/upload/folder"
    : "/api/memories/upload/file";

  const formData = new FormData();
  formData.append("file", file);

  const response = await fetch(endpoint, { method: "POST", body: formData });
  return response.json();
};
```

**NEW BROKEN FLOW (current branch):**

```typescript
// Complex blob-first approach with client-side storage manager
export const uploadFile = async (
  file: File,
  isOnboarding: boolean,
  existingUserId?: string,
  mode: UploadMode = "files",
  storageBackend: StorageBackend | StorageBackend[] = "vercel_blob",
  userStoragePreference?: "neon" | "icp" | "dual"
) => {
  const storageManager = new StorageManager(); // ❌ This fails - no providers available
  // ... complex blob upload logic
};
```

### Key Changes Made

1. **Replaced Simple FormData Upload**: Changed from direct server upload to client-side blob storage
2. **Added StorageManager**: Introduced complex storage provider system that requires client-side environment variables
3. **Changed Upload Endpoints**: No longer using the working `/api/memories/upload/onboarding/file` endpoints
4. **Added Blob-First Logic**: Files now go through Vercel Blob before reaching the database

### Original Intent

**The goal was to move blob upload to the frontend** to:

- Reduce server load by handling file uploads client-side
- Provide better user experience with direct blob storage
- Implement a "blob-first" architecture where files go to storage first, then metadata to database

**However, the implementation failed because:**

- `BLOB_READ_WRITE_TOKEN` is not available in client-side context
- Vercel Blob operations should typically happen server-side for security
- The StorageManager approach requires client-side environment variables that aren't accessible

### Current Upload Flow

```
1. User selects file in dashboard
2. processSingleFile() called in user-file-upload.ts
3. uploadFile() called in services/upload.ts
4. StorageManager created and initialized
5. VercelBlobProvider.isAvailable() returns false
6. uploadWithFallback() called but no fallback providers available
7. Error: "All storage providers failed"
```

### Debug Output Analysis

From the console logs, we can see:

- `BLOB_READ_WRITE_TOKEN exists: false` (in client-side context)
- `Vercel Blob isAvailable: false`
- `NODE_ENV: development`
- `BLOB_READ_WRITE_TOKEN: NOT SET` (in client-side context)
- Image processing completes successfully (server-side)
- But storage upload fails (client-side)

## AI Analysis Request

**Please analyze the main branch code and describe how the working upload system actually functions:**

### 1. Upload Flow Analysis

**Describe the complete upload flow in the main branch:**

- How does `uploadFile()` work in the main branch?
- What endpoints does it call?
- How are files processed and stored?
- What role does the server play vs the client?

### 2. Server-Side Implementation

**Describe the server-side upload endpoints:**

- How does `/api/memories/upload/onboarding/file` work?
- How does `/api/memories/upload/onboarding/folder` work?
- What storage mechanisms are used on the server?
- How are files stored and where?

### 3. Environment Variables Usage

**Describe how environment variables are used:**

- Where is `BLOB_READ_WRITE_TOKEN` used in the main branch?
- Is it used client-side or server-side?
- What other environment variables are involved in file uploads?
- How are they accessed and configured?

### 4. Storage Architecture

**Describe the storage architecture:**

- How are files actually stored in the working system?
- What storage providers are used?
- How does the database integration work?
- What's the relationship between file storage and memory records?

### 5. Client-Side vs Server-Side Responsibilities

**Describe the separation of concerns:**

- What happens on the client side?
- What happens on the server side?
- How do they communicate?
- What data flows between them?

**Please provide a detailed description of the working system so we can understand:**

- The actual architecture that works
- How it differs from our current broken implementation
- What we need to understand to fix our approach

## AI Analysis Results

### **Complete Upload Flow in Main Branch (WORKING):**

The working system follows a **server-side first** architecture:

```
1. User selects file(s) in dashboard
2. Client calls uploadFile() in services/upload.ts
3. uploadFile() creates FormData and sends to server endpoint
4. Server endpoint processes file completely:
   - Validates file (size, type, content)
   - Uploads to Vercel Blob storage
   - Stores metadata in Neon database
   - Creates storage edges
   - Returns success response
5. Client receives response and updates UI
```

### **Key Characteristics of Working System:**

- **Simple FormData upload**: No complex client-side storage management
- **Server-side processing**: All file handling happens on the server
- **Direct endpoint calls**: Uses standard REST API endpoints
- **No client-side environment variables**: BLOB_READ_WRITE_TOKEN is only used server-side

### **Server-Side Implementation (WORKING):**

**Single File Upload (`/api/memories/upload/onboarding/file`):**

```typescript
1. Parse form data and extract file
2. Validate file (size, type, content)
3. Upload file to Vercel Blob storage using @vercel/blob
4. Create temporary user
5. Store metadata in database (images/videos/documents table)
6. Create storage edges for tracking
7. Return success response with memory ID and owner ID
```

**Folder Upload (`/api/memories/upload/onboarding/folder`):**

```typescript
1. Parse form data and extract multiple files
2. Validate all files
3. Create ONE temporary user for all files
4. Process files in parallel (max 5 concurrent)
5. Upload each file to Vercel Blob
6. Batch insert all metadata into database
7. Create storage edges for all files
8. Return comprehensive results
```

### **Storage Architecture (WORKING):**

**Vercel Blob Storage:**

- Uses `@vercel/blob` package with `put()` function
- Files stored with public access
- URLs generated: `uploads/{timestamp}-{filename}`
- **Server-side only**: No client-side blob operations

**Neon Database:**

- Stores metadata in `images`, `videos`, `documents` tables
- Each record includes: `ownerId`, `url`, `title`, `metadata`, `ownerSecureCode`
- **Storage Edges**: Tracks where each memory is stored (`storageEdges` table)

### **Environment Variables Usage (WORKING):**

**BLOB_READ_WRITE_TOKEN:**

- **Server-side only**: Used in API routes, not client-side
- **Automatic detection**: `@vercel/blob` automatically reads from `process.env.BLOB_READ_WRITE_TOKEN`
- **No client-side access**: Never exposed to browser
- **Vercel deployment**: Automatically available in Vercel environment

### **Client-Side vs Server-Side Responsibilities (WORKING):**

**Client-Side (Browser):**

- File selection and validation (basic size check)
- FormData creation and HTTP request
- UI updates and error handling
- Progress indication

**Server-Side (API Routes):**

- File validation (comprehensive)
- Blob storage operations
- Database operations
- User management
- Storage edge creation
- Error handling and logging

### **Root Cause Analysis:**

**What Works (Main Branch):**

```typescript
// Simple, working approach
export const uploadFile = async (
  file: File,
  isOnboarding: boolean,
  existingUserId?: string,
  mode: UploadMode = "files"
) => {
  const endpoint = isOnboarding ? "/api/memories/upload/onboarding/file" : "/api/memories/upload/file";
  const formData = new FormData();
  formData.append("file", file);

  const response = await fetch(endpoint, { method: "POST", body: formData });
  return response.json();
};
```

**What's Broken (Current Branch):**

```typescript
// Complex, broken approach
export const uploadFile = async (
  file: File,
  isOnboarding: boolean,
  existingUserId?: string,
  mode: UploadMode = "files",
  storageBackend: StorageBackend | StorageBackend[] = "vercel_blob",
  userStoragePreference?: "neon" | "icp" | "dual"
) => {
  const storageManager = new StorageManager(); // ❌ Fails - no providers available
  // ... complex blob upload logic
};
```

### **Architecture Mismatch Issues:**

- **Trying to do client-side blob uploads** (not recommended)
- **BLOB_READ_WRITE_TOKEN not available client-side** (security feature)
- **Complex storage manager** (unnecessary for simple use case)
- **Over-engineering** (simple FormData upload works perfectly)

## Tech Lead Decision

**DECISION: Implement direct-to-storage from browser with server-issued tokens (not "always" - use decision matrix)**

### **Upload Policy (Tech Lead Decision):**

#### **1. Default Path (Most Cases, Files >4-5 MB)**

- **Direct-to-blob from browser** with short-lived server-issued token/presigned URL
- Client streams to storage; server writes metadata after success (via webhook or client callback)
- **Why**: Avoids double-hop, bypasses serverless body limits, cheaper, faster, more reliable

#### **2. When Server-Side Proxy Uploads Are OK**

- Tiny files (<4 MB) where simplicity > complexity
- Back-office/admin tools doing server-initiated ingest (no browser)
- Internal migrations/copies between providers

#### **3. Very Large Files / Unstable Networks**

- Use resumable/multipart flow (S3 multipart or Vercel's chunked client flow)
- Parallel parts, retry by part, integrity check, then "complete" call
- Keep uploads idempotent with upload session ID

### **Concrete Implementation Plan (Vercel Blob First):**

#### **Client Side:**

- Ask API for upload grant (filename, size, type, checksum)
- Upload directly to Blob using returned URL/fields; show progress
- On success, call API with resulting URL + client-side metadata (dimensions, EXIF hash)
- If upload aborted, allow resume (session ID)

#### **Server Side:**

- API route issues short-lived, scope-limited token or presigned URL (size/type constrained; ties to user and quota)
- After client success: validate payload (size, content-type, checksum), write DB record, create storage edges, enqueue post-processing (thumbnails, AV scan, OCR)
- Webhook listener (optional) to reconcile late completions; make metadata write idempotent on object key

#### **Security/Quotas:**

- Enforce per-user quota before minting grants
- Limit max size, allowed MIME, and put user ID and expiry inside grant
- **Don't expose server secrets; never require `BLOB_READ_WRITE_TOKEN` in browser**

### **Migration Steps (1 Week Effort, Parallelizable):**

1. **Re-enable current server route** for small files; gate with `NEXT_PUBLIC_UPLOAD_V2`
2. **Implement "issue upload grant" API** and client uploader (progress + cancel + resume)
3. **Add DB write endpoint** + optional blob completion webhook; make writes idempotent
4. **Add session table** + quotas + MIME/size policy
5. **Wire post-processing queue** (thumbs/webp/EXIF/AV)
6. **Remove client-side StorageManager** that reads env; keep thin provider registry on server

### **Edge Cases to Cover:**

- **Folder uploads**: Create one session per file; show aggregate progress; commit per-file metadata
- **Duplicate detection**: Optional content hash to soft-dedupe
- **Offline/retry**: Persist pending sessions in IndexedDB; resume on reconnect
- **ICP/dual storage (later)**: Write-through from Blob completion to ICP; make it async

## Status

✅ **AI Analysis Complete** - Comprehensive understanding of working system
✅ **Tech Lead Decision Made** - Direct-to-storage with server-issued tokens
🚀 **Implementation Plan Ready** - 1 week migration plan with clear steps
