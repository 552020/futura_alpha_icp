# Vercel Blob Grant Implementation Todo List

## Overview

Implementation plan for fixing the "Missing [x]-content-length header" error by replacing our custom grant endpoint with the official Vercel Blob client-side upload flow using `handleUpload()`.

## Progress Summary

✅ **Sections 1-6 Complete** (Major Implementation Done)

- ✅ **Section 1**: Replace the Grant Route
- ✅ **Section 2**: Update Client Upload Service
- ✅ **Section 3**: Add Server Helper for Memory Creation
- ✅ **Section 4**: Handle Authentication Integration
- ✅ **Section 5**: Image Multi-Asset Handling (Server-Side)
- ✅ **Section 6**: Simplify Upload Decision Matrix

🔄 **Remaining Sections** (Testing & Cleanup)

- ⏳ **Section 7**: Testing and Validation
- ⏳ **Section 8**: Cleanup and Documentation
- ⏳ **Section 9**: Performance and Monitoring
- ⏳ **Section 10**: Deployment and Rollout

**Current Status**: All core implementation complete, ready for testing and cleanup phases.

## 1. Replace the Grant Route

### 1.1 Update Grant Route Implementation

- [x] **1.1.1** Replace `src/nextjs/src/app/api/memories/grant/route.ts` with senior's implementation
- [x] **1.1.2** Remove `put()` with `Buffer.alloc(0)` code
- [x] **1.1.3** Remove fake `content-length` headers
- [x] **1.1.4** Add `handleUpload()` from `@vercel/blob/client`
- [x] **1.1.5** Add proper imports for `HandleUploadBody` type

### 1.2 Wire Authentication

- [x] **1.2.1** Verify `getAllUserId` function returns correct format
- [x] **1.2.2** Add authentication check in grant route
- [x] **1.2.3** Handle unauthorized requests with proper error response
- [x] **1.2.4** Test authentication flow with both authenticated and onboarding users

### 1.3 Configure Token Generation

- [x] **1.3.1** Set up `onBeforeGenerateToken` callback
- [x] **1.3.2** Configure allowed content types
- [x] **1.3.3** Set maximum file size to 5TB
- [x] **1.3.4** Add `addRandomSuffix: true` for collision safety
- [x] **1.3.5** Configure `tokenPayload` with user context

### 1.4 Handle Upload Completion

- [x] **1.4.1** Implement `onUploadCompleted` callback
- [x] **1.4.2** Add error handling for DB creation failures
- [x] **1.4.3** Ensure non-blocking DB operations
- [x] **1.4.4** Add proper logging for upload completion

## 2. Update Client Upload Service

### 2.1 Install Required Dependencies

- [x] **2.1.1** Verify `@vercel/blob` is installed
- [x] **2.1.2** Add `@vercel/blob/client` import to upload service
- [x] **2.1.3** Update package.json if needed

### 2.2 Update Upload Large File Function

- [x] **2.2.1** Replace current `uploadLargeFile` implementation
- [x] **2.2.2** Use `upload()` from `@vercel/blob/client`
- [x] **2.2.3** Set `handleUploadUrl: "/api/memories/grant"`
- [x] **2.2.4** Add `multipart: true` for large files
- [x] **2.2.5** Configure `clientPayload` with context

### 2.3 Add Progress Tracking

- [x] **2.3.1** Implement `onUploadProgress` callback
- [x] **2.3.2** Add progress logging
- [x] **2.3.3** Consider UI progress integration
- [x] **2.3.4** Test progress tracking with large files

### 2.4 Update Return Format

- [x] **2.4.1** Verify return format matches `UploadResponse` interface
- [x] **2.4.2** Map blob response to expected asset format
- [x] **2.4.3** Handle single asset creation (original only)
- [x] **2.4.4** Test return format compatibility

## 3. Add Server Helper for Memory Creation

### 3.1 Create Memory Creation Function

- [x] **3.1.1** Add `createMemoryFromBlob` function to `post.ts`
- [x] **3.1.2** Define input types for blob data and metadata
- [x] **3.1.3** Implement memory type detection from content type
- [x] **3.1.4** Add title extraction from pathname

### 3.2 Implement Database Insertion

- [x] **3.2.1** Map blob data to memory record
- [x] **3.2.2** Insert memory record using existing schema
- [x] **3.2.3** Insert memory asset record
- [x] **3.2.4** Handle all required fields (parentFolderId, tags, etc.)
- [x] **3.2.5** Add proper error handling

### 3.3 Handle User Context

- [x] **3.3.1** Use `allUserId` from metadata
- [x] **3.3.2** Handle `isOnboarding` flag
- [x] **3.3.3** Handle `mode` parameter
- [x] **3.3.4** Set appropriate default values

### 3.4 Export Function

- [x] **3.4.1** Export `createMemoryFromBlob` function
- [x] **3.4.2** Import in grant route
- [x] **3.4.3** Test function integration

## 4. Handle Authentication Integration

### 4.1 Verify Auth Functions

- [x] **4.1.1** Check `getAllUserId` return format
- [x] **4.1.2** Verify `getUserIdForUpload` compatibility
- [x] **4.1.3** Test with authenticated users
- [x] **4.1.4** Test with onboarding users

### 4.2 Handle Onboarding Users

- [x] **4.2.1** Accept `existingUserId` in `clientPayload`
- [x] **4.2.2** Use `getUserIdForUpload` when needed
- [x] **4.2.3** Handle temporary user creation
- [x] **4.2.4** Test onboarding flow

### 4.3 Error Handling

- [x] **4.3.1** Handle authentication failures
- [x] **4.3.2** Return proper error responses
- [x] **4.3.3** Add logging for auth issues
- [x] **4.3.4** Test error scenarios

## 5. Image Multi-Asset Handling (Server-Side)

### 5.1 Choose Implementation Approach

- [x] **5.1.1** Decide on server-side processing (Option B)
- [x] **5.1.2** Plan image processing workflow
- [x] **5.1.3** Design asset grouping mechanism

### 5.2 Implement Server-Side Image Processing

- [x] **5.2.1** Create image processing job queue
- [x] **5.2.2** Implement `processImageForMultipleAssetsBackend`
- [x] **5.2.3** Add sharp-based image processing
- [x] **5.2.4** Handle original, display, and thumb generation

### 5.3 Handle Asset Upload

- [x] **5.3.1** Upload derivatives to blob storage
- [x] **5.3.2** Use server-side `put()` for derivatives
- [x] **5.3.3** Generate proper asset records
- [x] **5.3.4** Link assets to original memory

### 5.4 Implement Async Processing

- [x] **5.4.1** Use `process.nextTick` for fire-and-forget
- [x] **5.4.2** Add error handling for processing failures
- [x] **5.4.3** Add logging for processing status
- [x] **5.4.4** Test async processing flow

## 6. Simplify Upload Decision Matrix

### 6.1 Update Decision Logic

- [x] **6.1.1** Modify file size decision matrix
- [x] **6.1.2** Use client upload for all file sizes
- [x] **6.1.3** Remove 4MB threshold logic
- [x] **6.1.4** Simplify upload paths

### 6.2 Handle Small Files

- [x] **6.2.1** Route small files through client upload
- [x] **6.2.2** Test small file uploads
- [x] **6.2.3** Verify performance for small files
- [x] **6.2.4** Update documentation

### 6.3 Remove Old Upload Paths

- [x] **6.3.1** Remove `uploadSmallFile` function
- [x] **6.3.2** Remove server-side FormData upload
- [x] **6.3.3** Clean up unused code
- [x] **6.3.4** Update comments and documentation

## 7. Testing and Validation

### 7.1 Test Grant Endpoint

- [ ] **7.1.1** Test with small files (<4MB)
- [ ] **7.1.2** Test with large files (>4MB)
- [ ] **7.1.3** Test with images
- [ ] **7.1.4** Test with different file types
- [ ] **7.1.5** Test authentication scenarios

### 7.2 Test Client Upload

- [ ] **7.2.1** Test progress tracking
- [ ] **7.2.2** Test multipart uploads
- [ ] **7.2.3** Test error handling
- [ ] **7.2.4** Test with various file sizes

### 7.3 Test Memory Creation

- [ ] **7.3.1** Test memory record creation
- [ ] **7.3.2** Test asset record creation
- [ ] **7.3.3** Test with different user types
- [ ] **7.3.4** Test error scenarios

### 7.4 Test Image Processing

- [ ] **7.4.1** Test server-side image processing
- [ ] **7.4.2** Test derivative generation
- [ ] **7.4.3** Test asset linking
- [ ] **7.4.4** Test async processing

## 8. Cleanup and Documentation

### 8.1 Remove Old Code

- [ ] **8.1.1** Delete failing `put()` implementation
- [ ] **8.1.2** Remove presigned URL client calls
- [ ] **8.1.3** Clean up unused imports
- [ ] **8.1.4** Remove old error handling

### 8.2 Update Documentation

- [ ] **8.2.1** Update Vercel Blob architecture docs
- [ ] **8.2.2** Update upload service documentation
- [ ] **8.2.3** Update API documentation
- [ ] **8.2.4** Add troubleshooting guide

### 8.3 Add Error Handling

- [ ] **8.3.1** Add comprehensive error logging
- [ ] **8.3.2** Add user-friendly error messages
- [ ] **8.3.3** Add retry mechanisms
- [ ] **8.3.4** Add monitoring and alerts

## 9. Performance and Monitoring

### 9.1 Add Monitoring

- [ ] **9.1.1** Add upload success/failure metrics
- [ ] **9.1.2** Add performance monitoring
- [ ] **9.1.3** Add error rate tracking
- [ ] **9.1.4** Add file size distribution tracking

### 9.2 Optimize Performance

- [ ] **9.2.1** Test upload performance
- [ ] **9.2.2** Optimize image processing
- [ ] **9.2.3** Add caching where appropriate
- [ ] **9.2.4** Monitor memory usage

## 10. Deployment and Rollout

### 10.1 Prepare for Deployment

- [ ] **10.1.1** Test in staging environment
- [ ] **10.1.2** Verify environment variables
- [ ] **10.1.3** Test with production-like data
- [ ] **10.1.4** Prepare rollback plan

### 10.2 Deploy Changes

- [ ] **10.2.1** Deploy grant route changes
- [ ] **10.2.2** Deploy client upload changes
- [ ] **10.2.3** Deploy memory creation changes
- [ ] **10.2.4** Monitor deployment

### 10.3 Post-Deployment

- [ ] **10.3.1** Monitor error rates
- [ ] **10.3.2** Monitor upload success rates
- [ ] **10.3.3** Monitor performance metrics
- [ ] **10.3.4** Gather user feedback

## Success Criteria

- [ ] **No more "Missing [x]-content-length header" errors**
- [ ] **Large file uploads work (>4MB)**
- [ ] **Small file uploads work (<4MB)**
- [ ] **Image uploads create multiple assets**
- [ ] **Progress tracking works**
- [ ] **Authentication works for all user types**
- [ ] **Memory creation works properly**
- [ ] **Performance is acceptable**
- [ ] **Error handling is robust**

## Dependencies

- [ ] **@vercel/blob** package installed
- [ ] **BLOB_READ_WRITE_TOKEN** environment variable
- [ ] **BLOB_FOLDER_NAME** environment variable (optional)
- [ ] **Database schema** for memories and memory_assets
- [ ] **Authentication system** working
- [ ] **Image processing libraries** (sharp) for server-side processing

## Notes

- **Priority**: High - This fixes a critical upload error
- **Complexity**: Medium - Requires changes to multiple files
- **Risk**: Low - Well-tested approach from senior developer
- **Timeline**: 2-3 days for full implementation
- **Testing**: Extensive testing required due to upload complexity
