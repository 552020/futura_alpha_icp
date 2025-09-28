
# Task

**Refactor file upload system from server-side to client-side S3 uploads using presigned URLs.**

---

## Current State

-   `src/app/api/memories/route.ts` handles all uploads server-side.
    
-   Frontend calls `/api/memories` with `FormData` for folder/file uploads.
    
-   Server uploads files directly to S3, then stores records in the database.
    

---

## Required Changes

### New Upload API Structure

| Endpoint | Purpose |
| --- | --- |
| `/api/upload/presign/route.ts` | Generate individual file presigned URL |
| `/api/upload/batch-presign/route.ts` | Generate batch presigned URLs |
| `/api/upload/commit/route.ts` | Commit individual file metadata to DB |
| `/api/upload/batch-commit/route.ts` | Commit batch file metadata to DB |
| `/api/upload/utils/presign-logic.ts` | Shared presigning utilities |

---

### New Flow

1.  Frontend requests presigned URLs from API with file metadata.
    
2.  Frontend uploads files **directly to S3** using presigned URLs.
    
3.  Frontend calls commit endpoints to create memory records in the database.
    
4.  Keep existing `/api/memories` POST endpoint as fallback for server-side uploads.
    

---

### Requirements

-   Extract presigning logic into **reusable utilities**.
    
-   Batch operations should **optimize database queries**.
    
-   Maintain **compatibility with existing onboarding flow**.
    
-   Handle **error states**: failed uploads, orphaned presigned URLs.
    
-   Include proper **TypeScript types** throughout.
    

---

### Context

The current `handleFolderUpload` function:

-   Processes files server-side.
    
-   Performs **parallel uploads to S3**.
    
-   Stores results in the database.
    

**Goal of the refactor:**

-   Move S3 uploads to the client.
    
-   Preserve **database schema** and **folder creation logic**.
    
-   Maintain fallback support for server-side uploads.
    





