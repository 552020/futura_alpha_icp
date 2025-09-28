# S3 Upload Grant Inconsistency Issue

## Problem Description

The S3 upload system has inconsistent grant handling between single and multiple file uploads, leading to code duplication and architectural confusion.

## Current Inconsistent Behavior

### Single File Upload (`uploadToS3WithProcessing`)

- **Uses**: `getSingleGrant(file)` function
- **Endpoint**: `/api/upload/s3/request`
- **Request body**:
  ```json
  {
    "fileName": "file.jpg",
    "fileType": "image/jpeg",
    "fileSize": 12345,
    "derivatives": ["display", "thumb"]
  }
  ```
- **Response**: `GrantResponse` with `original`, `display?`, `thumb?` presigned URLs

### Multiple Files Upload (`uploadMultipleToS3WithProcessing`)

- **Uses**: Inline fetch call (no shared function)
- **Endpoint**: `/api/upload/batch-presign`
- **Request body**:
  ```json
  {
    "files": [
      {
        "fileName": "file1.jpg",
        "fileType": "image/jpeg",
        "fileSize": 12345
      },
      {
        "fileName": "file2.jpg",
        "fileType": "image/jpeg",
        "fileSize": 67890
      }
    ]
  }
  ```
- **Response**: Array of `GrantResponse` objects

## Key Differences Between Endpoints

### 1. Authentication Methods

- **Single file**: `auth()` session-based authentication
- **Multiple files**: `getUserIdForUpload()` utility function

### 2. Derivatives Handling

- **Single file**: **Conditional** - Only creates derivatives if `derivatives: ["display", "thumb"]` is provided
- **Multiple files**: **Automatic** - Always creates derivatives for `image/*` files, ignores non-image files

### 3. Request Format

- **Single file**: Direct file properties in request body
- **Multiple files**: File properties wrapped in `files` array

### 4. Response Format

- **Single file**: Direct `GrantResponse` object
- **Multiple files**: `{ grants: GrantResponse[] }` - Array wrapped in object

### 5. User ID Source

- **Single file**: `session.user.id` from authenticated session
- **Multiple files**: `allUserId` from `getUserIdForUpload()` utility

## Issues Identified

1. **Code Duplication**: Multiple files upload reimplements grant logic instead of reusing `getSingleGrant`
2. **Different Endpoints**: Single files use `/api/upload/s3/request`, multiple files use `/api/upload/batch-presign`
3. **Inconsistent Request Format**: Single file includes `derivatives` array, multiple files doesn't
4. **No Shared Abstraction**: No unified grant function for both single and multiple files
5. **Architectural Inconsistency**: Violates DRY principle and makes maintenance harder

## Impact

- **Maintenance Burden**: Changes to grant logic need to be made in multiple places
- **Bug Risk**: Inconsistencies between single and multiple file behavior
- **Code Quality**: Violates single responsibility and DRY principles
- **Developer Confusion**: Unclear which endpoint/function to use for different scenarios

## Proposed Solution

### Option 1: Unified Grant Function

Create a single `getGrants(files: File[])` function that:

- Detects single vs multiple files automatically
- Uses appropriate endpoint based on file count
- Returns consistent `GrantResponse[]` format
- Handles derivatives consistently

### Option 2: Separate Functions with Consistent Interface

- Keep `getSingleGrant(file)` for single files
- Create `getBatchGrants(files)` for multiple files
- Ensure both use same request/response format
- Standardize on single endpoint or clearly document when to use each

### Option 3: Endpoint Consolidation

- Consolidate to single endpoint that handles both single and multiple files
- Update both functions to use same endpoint
- Simplify API surface area

## Files Affected

- `src/services/upload/s3-grant.ts` - Grant abstraction layer
- `src/services/upload/s3-with-processing.ts` - Upload implementations
- `src/app/api/upload/s3/request/route.ts` - Single file endpoint
- `src/app/api/upload/batch-presign/route.ts` - Multiple files endpoint

## Priority

**High** - This affects code maintainability and architectural consistency. Should be addressed before further S3 upload unification work.

## Related Issues

- S3 Upload Unification (pending)
- API Route Cleanup (pending)
