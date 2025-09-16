# FormData Double Parsing Bug - Small File Uploads Failing

## Files Involved

- **Primary**: `src/nextjs/src/app/api/memories/post.ts`
- **Secondary**: `src/nextjs/src/app/api/memories/utils/form-parsing.ts`
- **Frontend**: `src/nextjs/src/services/upload.ts`

## Functions Involved

- `handleApiMemoryPost()` - Main POST handler
  - **Location**: `src/nextjs/src/app/api/memories/post.ts:57-102`
- `handleFileUpload()` - Single file upload handler
  - **Location**: `src/nextjs/src/app/api/memories/post.ts:195-327`
- `parseMultipleFiles()` - FormData parsing utility
  - **Location**: `src/nextjs/src/app/api/memories/utils/form-parsing.ts:52-79`
- `uploadSmallFile()` - Frontend upload function
  - **Location**: `src/nextjs/src/services/upload.ts:210-228`

## Problem Description

Small file uploads (< 4MB) are failing with the error **"Invalid form data"** due to a critical bug in the request parsing logic.

### Error Details

```
Error Type: Console Error
Error Message: Invalid form data
Location: src/services/upload.ts:224:11
Stack Trace:
  at uploadSmallFile (src/services/upload.ts:224:11)
  at async uploadFile (src/services/upload.ts:199:14)
  at async processSingleFile (src/hooks/user-file-upload.ts:191:28)
  at async handleFileUpload (src/hooks/user-file-upload.ts:398:7)
```

### Root Cause

The issue occurs because **`request.formData()` is being called twice on the same request**, which is not allowed in Next.js/Web APIs. Once a request body is consumed by `request.formData()`, subsequent calls will fail.

#### The Problematic Flow:

1. **Frontend** (`uploadSmallFile`): Correctly creates FormData and sends to `/api/memories`

   ```typescript
   const formData = new FormData();
   formData.append("file", file);
   const response = await fetch("/api/memories", {
     method: "POST",
     body: formData,
   });
   ```

2. **Backend** (`handleApiMemoryPost`): **FIRST CALL** to `request.formData()`

   - **Location**: `src/nextjs/src/app/api/memories/post.ts:66`

   ```typescript
   if (contentType.includes("multipart/form-data")) {
     const formData = await request.formData(); // ← CONSUMES REQUEST BODY
     const files = formData.getAll("file") as File[];

     if (files.length > 1) {
       return await handleFolderUpload(request);
     } else {
       return await handleFileUpload(request, allUserId); // ← CALLS SECOND PARSING
     }
   }
   ```

3. **Backend** (`handleFileUpload`): **SECOND CALL** to `request.formData()`

   - **Location**: `src/nextjs/src/app/api/memories/post.ts:199`

   ```typescript
   const { files, error: parseError } = await parseMultipleFiles(request);
   ```

4. **Backend** (`parseMultipleFiles`): **FAILS** because body was already consumed

   - **Location**: `src/nextjs/src/app/api/memories/utils/form-parsing.ts:58`

   ```typescript
   const formData = await request.formData(); // ← THROWS ERROR
   ```

### Why This Happens

- **Single file uploads** go through the `handleFileUpload` path
- **Folder uploads** go through the `handleFolderUpload` path
- Both paths call `parseMultipleFiles()` which tries to parse FormData again
- The first parsing in `handleApiMemoryPost` consumes the request body
- Subsequent parsing attempts fail with "Invalid form data"

## Impact

- **Small file uploads** (< 4MB) are completely broken
- **Large file uploads** (> 4MB) work because they use a different code path (blob-first approach)
- **Folder uploads** may also be affected depending on the implementation

## Proposed Solution

### Option 1: Pass Parsed FormData (Recommended)

Modify `handleApiMemoryPost` to parse FormData once and pass it to handlers:

```typescript
export async function handleApiMemoryPost(request: NextRequest): Promise<NextResponse> {
  try {
    const contentType = request.headers.get("content-type") || "";

    if (contentType.includes("multipart/form-data")) {
      // Parse FormData ONCE
      const formData = await request.formData();
      const files = formData.getAll("file") as File[];

      if (files.length > 1) {
        return await handleFolderUpload(request, formData); // Pass parsed data
      } else {
        const { allUserId, error } = await getAllUserId(request);
        if (error) return error;
        return await handleFileUpload(request, allUserId, formData); // Pass parsed data
      }
    } else {
      // JSON handling remains the same
    }
  } catch (error) {
    console.error("Error in memory creation:", error);
    return NextResponse.json({ error: "Failed to create memory" }, { status: 500 });
  }
}
```

### Option 2: Restructure Parsing Logic

Create a single parsing function that handles both single and multiple files:

```typescript
async function parseFormData(request: NextRequest): Promise<{
  files: File[];
  userId?: string;
  error: string | null;
}> {
  try {
    const formData = await request.formData();
    const files = formData.getAll("file") as File[];
    const userId = formData.get("userId") as string | null;

    if (!files || files.length === 0) {
      return { files: [], error: "No files provided" };
    }

    return { files, userId: userId || undefined, error: null };
  } catch (error) {
    console.error("❌ Error parsing form data:", error);
    return { files: [], error: "Invalid form data" };
  }
}
```

## Status

**FIXED** - The double parsing issue has been resolved by modifying the functions to parse FormData once and pass it to handlers.

## Priority

**HIGH** - This was a critical bug that broke core functionality for small file uploads.

## Testing

After implementing the fix, test:

1. Single small file upload (< 4MB)
2. Single large file upload (> 4MB)
3. Folder upload with multiple files
4. Mixed file sizes in folder upload

## Related Issues

- This may be related to the broader upload refactoring mentioned in the current branch `slombard/icp-430-refactor-itemuploadbutton`
- Consider reviewing the entire upload flow to ensure consistency between small/large file handling
