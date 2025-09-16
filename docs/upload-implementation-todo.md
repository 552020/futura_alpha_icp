# Upload Implementation Todo List

## Overview

Implementation plan for direct-to-storage uploads with server-issued tokens, based on tech lead decision.

## Current API Status

- **Active Endpoint**: `/api/memories` POST (handles both single files and folder uploads)
- **Deprecated Endpoints**:
  - `/api/memories/upload/onboarding/file` (commented out)
  - `/api/memories/upload/onboarding/folder` (commented out)
  - `/api/memories/upload/file` (empty directory)
  - `/api/memories/upload/folder` (empty directory)
- **Current Implementation**: Uses blob-first approach with `uploadFiles` service from `@/services/upload`

## Current System Analysis

- **Working Parts**:
  - ICP upload flow (uses `/api/upload/intent` and `/api/upload/verify`)
  - Image processing for multiple assets (original, display, thumb)
  - Unified `/api/memories` POST endpoint
- **Broken Parts**:
  - Client-side `StorageManager` requires `BLOB_READ_WRITE_TOKEN` (not available client-side)
  - Vercel Blob uploads fail because of missing environment variables
  - Only ICP uploads work (when user is authenticated)
- **Key Issue**: The system tries to do client-side blob uploads but `BLOB_READ_WRITE_TOKEN` is server-side only
- **Critical Insight**: The system already has working ICP uploads and server-side infrastructure - we just need to fix the Vercel Blob client-side issue

## Phase 1: Re-enable Current Server Route (Small Files)

### 1. Fix Current Upload System

- [x] 1.1. **IMMEDIATE FIX**: Fix client-side `StorageManager` to work with server-issued tokens (don't replace)
- [x] 1.2. **IMMEDIATE FIX**: Move Vercel Blob token generation to server-side API endpoints
- [x] 1.3. Keep existing `/api/memories` POST endpoint working (handles both single files and folders)
- [x] 1.4. Keep ICP upload flow working (already functional with `/api/upload/intent` and `/api/upload/verify`)
- [x] 1.5. Implement file size decision matrix (server-side for <4MB, direct-to-blob for >4MB)
- [x] 1.6. Add file size check in client-side upload logic
- [x] 1.7. Route files based on size: small files to server-side, large files to direct-to-blob
- [x] 1.8. Note: The old `/api/memories/upload/onboarding/file` and `/api/memories/upload/onboarding/folder` endpoints are commented out and deprecated

## Phase 2: Implement Upload Grant API (For Large Files)

### 2. Create Upload Grant API Endpoint

- [x] 2.1. Create new API route `/api/memories/grant` (follows existing pattern)
- [x] 2.2. Implement POST handler for upload grant requests
- [x] 2.3. Add request validation (filename, size, type, checksum)
- [ ] 2.4. Add user quota checking before issuing grants
- [x] 2.5. Generate short-lived, scoped tokens (max 1 hour expiry)
- [x] 2.6. Include user ID and expiry in grant payload
- [x] 2.7. Add MIME type and size constraints to grants
- [x] 2.8. Return presigned URL or upload fields for Vercel Blob
- [x] 2.9. **Note**: This is only needed for large files (>4MB) - small files use existing server-side flow

### 3. Add Upload Session Management

- [ ] 3.1. Create `upload_sessions` database table
- [ ] 3.2. Add session status tracking (requested → uploading → completed|failed)
- [ ] 3.3. Store session metadata (user_id, filename, size, type, checksum)
- [ ] 3.4. Add byte count tracking for progress monitoring
- [ ] 3.5. Add error code storage for failed uploads
- [ ] 3.6. Implement session cleanup for expired sessions

## Phase 3: Implement Client-Side Direct Upload

### 4. Create Client-Side Upload Service

- [x] 4.1. Create new `DirectUploadService` class (implemented as `VercelBlobGrantProvider`)
- [x] 4.2. Implement upload grant request method
- [x] 4.3. Implement direct-to-blob upload with progress tracking
- [ ] 4.4. Add upload cancellation support
- [ ] 4.5. Add upload resume functionality (using session ID)
- [ ] 4.6. Implement retry logic for network failures
- [x] 4.7. Add upload completion callback to server

### 5. Update Upload Hook

- [x] 5.1. Modify `user-file-upload.ts` to use decision matrix
- [x] 5.2. Add file size check for upload method selection
- [x] 5.3. Integrate `DirectUploadService` for large files
- [x] 5.4. Keep existing server-side upload for small files
- [ ] 5.5. Add progress tracking for both upload methods
- [ ] 5.6. Update error handling for both upload paths

## Phase 4: Add Database Write Endpoint

### 6. Create Upload Completion API

- [x] 6.1. Create new API route `/api/memories/complete` (created as `/api/memories/complete`)
- [x] 6.2. Implement POST handler for upload completion
- [x] 6.3. Validate upload payload (size, content-type, checksum)
- [x] 6.4. Write memory record to database
- [ ] 6.5. Create storage edges for tracking
- [ ] 6.6. Update upload session status to completed
- [ ] 6.7. Make metadata writes idempotent on object key

### 7. Add Post-Processing Queue

- [ ] 7.1. Create post-processing job queue system
- [ ] 7.2. Add thumbnail generation jobs
- [ ] 7.3. Add WebP conversion jobs
- [ ] 7.4. Add EXIF data extraction jobs
- [ ] 7.5. Add antivirus scanning jobs
- [ ] 7.6. Add OCR processing jobs
- [ ] 7.7. Implement job retry logic
- [ ] 7.8. Add job status tracking

## Phase 5: Add Webhook Support

### 8. Implement Blob Completion Webhook

- [ ] 8.1. Create webhook endpoint `/api/upload/webhook`
- [ ] 8.2. Add webhook signature verification
- [ ] 8.3. Implement late completion reconciliation
- [ ] 8.4. Handle webhook retry logic
- [ ] 8.5. Add webhook event logging
- [ ] 8.6. Implement idempotent webhook processing

## Phase 6: Add Security and Quotas

### 9. Implement User Quotas

- [ ] 9.1. Add user storage quota tracking
- [ ] 9.2. Check quota before issuing upload grants
- [ ] 9.3. Add quota enforcement in upload completion
- [ ] 9.4. Create quota exceeded error handling
- [ ] 9.5. Add quota usage reporting
- [ ] 9.6. Implement quota reset mechanisms

### 10. Add Security Measures

- [x] 10.1. Add file type validation (MIME type checking)
- [ ] 10.2. Add file size limits per user tier
- [ ] 10.3. Add content scanning for malicious files
- [ ] 10.4. Implement rate limiting for upload grants
- [ ] 10.5. Add audit logging for upload activities
- [ ] 10.6. Implement secure token generation

## Phase 7: Add Advanced Features

### 11. Implement Folder Upload Support

- [ ] 11.1. Create one session per file in folder uploads
- [ ] 11.2. Add aggregate progress tracking for folder uploads
- [ ] 11.3. Implement per-file metadata commit
- [ ] 11.4. Add folder upload cancellation
- [ ] 11.5. Handle partial folder upload failures
- [ ] 11.6. Add folder upload resume functionality

### 12. Add Duplicate Detection

- [ ] 12.1. Implement content hash calculation
- [ ] 12.2. Add duplicate detection before upload
- [ ] 12.3. Create soft-deduplication system
- [ ] 12.4. Add duplicate file handling options
- [ ] 12.5. Implement duplicate file UI indicators

### 13. Add Offline/Retry Support

- [ ] 13.1. Implement IndexedDB for pending sessions
- [ ] 13.2. Add offline upload queue
- [ ] 13.3. Implement resume on reconnect
- [ ] 13.4. Add offline status indicators
- [ ] 13.5. Handle network state changes
- [ ] 13.6. Add offline upload progress tracking

## Phase 8: Cleanup and Migration

### 14. Remove Old Storage Manager

- [ ] 14.1. Remove client-side `StorageManager` class
- [ ] 14.2. Remove client-side storage provider implementations
- [ ] 14.3. Keep thin provider registry on server only
- [ ] 14.4. Remove client-side environment variable dependencies
- [ ] 14.5. Clean up unused storage-related imports
- [ ] 14.6. Update documentation

### 15. Add Monitoring and Observability

- [ ] 15.1. Add structured logging for upload operations
- [ ] 15.2. Implement upload metrics collection
- [ ] 15.3. Add error rate monitoring
- [ ] 15.4. Create upload performance dashboards
- [ ] 15.5. Add alerting for upload failures
- [ ] 15.6. Implement upload analytics

## Phase 9: Testing and Validation

### 16. Add Comprehensive Testing

- [ ] 16.1. Add unit tests for upload grant API
- [ ] 16.2. Add unit tests for direct upload service
- [ ] 16.3. Add integration tests for upload flow
- [ ] 16.4. Add end-to-end tests for file uploads
- [ ] 16.5. Add performance tests for large files
- [ ] 16.6. Add error scenario testing

### 17. Add Error Handling

- [ ] 17.1. Implement typed error responses
- [ ] 17.2. Add user-friendly error messages
- [ ] 17.3. Add retry logic for transient failures
- [ ] 17.4. Add error recovery mechanisms
- [ ] 17.5. Add error reporting and analytics
- [ ] 17.6. Add error documentation

## Phase 10: Future Enhancements

### 18. Add ICP/Dual Storage Support

- [ ] 18.1. Implement write-through from Blob to ICP
- [ ] 18.2. Make ICP storage async
- [ ] 18.3. Add ICP storage status tracking
- [ ] 18.4. Implement ICP storage fallback
- [ ] 18.5. Add ICP storage monitoring
- [ ] 18.6. Create ICP storage migration tools

### 19. Add Advanced Upload Features

- [ ] 19.1. Implement multipart uploads for very large files
- [ ] 19.2. Add parallel chunk uploads
- [ ] 19.3. Implement upload integrity checking
- [ ] 19.4. Add upload acceleration features
- [ ] 19.5. Implement upload bandwidth throttling
- [ ] 19.6. Add upload scheduling capabilities

## Notes

- All tasks should be implemented with proper error handling
- Each phase can be worked on in parallel by different team members
- Testing should be added incrementally with each feature
- Documentation should be updated as features are implemented
- Security review should be conducted for each phase
- Performance testing should be done for each major feature
