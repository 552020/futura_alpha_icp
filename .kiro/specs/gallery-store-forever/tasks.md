# Implementation Plan

Convert the "Store Forever" feature design into a series of prompts for code-generation that will implement each step in a test-driven manner. Prioritize best practices, incremental progress, and early testing, ensuring no big jumps in complexity at any stage. Each task builds on previous tasks and ends with integration. Focus ONLY on tasks that involve writing, modifying, or testing code.

## Task Overview

This implementation plan transforms the existing "Store Forever" UI components and placeholder backend functions into a fully functional system that stores galleries permanently on the Internet Computer using an artifact-level protocol with storage_edges tracking.

## Implementation Tasks

- [ ] 1. Implement ICP Canister Artifact-Level API

  - Create structured error handling and response types
  - Implement chunked upload protocol with stable memory
  - Add memory presence verification endpoints
  - _Requirements: 17.1, 18.1, 22.1, 23.1, 24.3_

- [x] 1.0 Fix UUID Mapping in ICP Backend (CRITICAL)

  **Goal**: Fix the critical issue where ICP backend generates its own IDs instead of accepting canonical UUIDs from Web2, which breaks the core "Store Forever" feature.

  **Current Problem**:

  - ICP backend generates IDs: `format!("gallery_{}", ic_cdk::api::time())`
  - This breaks UUID mapping between Web2 and ICP systems
  - Same galleries/memories have different IDs in each system

  **Required Changes**:

  1. **Update Gallery Creation (capsule.rs)**:

     - REMOVE: `let gallery_id = format!("gallery_{}", ic_cdk::api::time());`
     - CHANGE: `gallery.id = gallery_id.clone();` → Don't overwrite gallery.id
     - USE: `let gallery_id = gallery_data.gallery.id.clone();` (accept Web2 UUID)

  2. **Update Memory Creation (capsule.rs)**:

     - REMOVE: `memory_id = format!("memory_{}", ic_cdk::api::time());`
     - ADD: `memory_id: String` parameter to function signature
     - USE: Accept memory_id from Web2 instead of generating

  3. **Add Idempotent Operations**:

     - Check if gallery/memory already exists with UUID
     - Return success for existing entities (don't create duplicates)
     - Example: `if let Some(existing) = get_gallery_by_id(gallery_id) { return success; }`

  4. **Update Function Signatures**:
     - `store_gallery_forever(gallery_data: GalleryData)` - no change, just don't overwrite gallery.id
     - `add_memory_to_capsule(memory_id: String, memory_data: MemoryData)` - add memory_id parameter

  **Files to Modify**:

  - `src/backend/src/capsule.rs` - Remove ID generation, accept external UUIDs
  - `src/backend/src/types.rs` - No changes needed (already uses String)
  - Update all functions that create galleries/memories

  **Testing**:

  - Verify same UUID works in both Web2 and ICP
  - Test idempotent operations (same UUID twice should succeed)
  - Ensure no ID generation occurs in ICP backend

  **UUID Strategy**:

  - PostgreSQL: `uuid` type (16-byte binary)
  - ICP Canister: `String` type (canonical string form)
  - Frontend: String throughout
  - Conversion: Use `uuid::text` for Postgres → ICP

  **Code Examples**:

  ```rust
  // CURRENT (❌ WRONG):
  pub fn store_gallery_forever(gallery_data: GalleryData) -> StoreGalleryResponse {
      let gallery_id = format!("gallery_{}", ic_cdk::api::time());  // ❌ Generates new ID
      let mut gallery = gallery_data.gallery;
      gallery.id = gallery_id.clone();  // ❌ Overwrites external UUID
      // ...
  }

  // REQUIRED (✅ CORRECT):
  pub fn store_gallery_forever(gallery_data: GalleryData) -> StoreGalleryResponse {
      let gallery_id = gallery_data.gallery.id.clone();  // ✅ Use Web2 UUID
      let mut gallery = gallery_data.gallery;
      // gallery.id is already set by Web2 - don't overwrite it!

      // Check if gallery already exists with this UUID (idempotency)
      if let Some(existing_gallery) = get_gallery_by_id(gallery_id.clone()) {
          return StoreGalleryResponse {
              success: true,
              gallery_id: Some(gallery_id),
              message: "Gallery already exists with this UUID".to_string(),
              // Return success for idempotent operation
          };
      }
      // Continue with creation...
  }

  // Memory function signature update:
  // CURRENT: pub fn add_memory_to_capsule(memory_data: MemoryData) -> MemoryOperationResponse
  // REQUIRED: pub fn add_memory_to_capsule(memory_id: String, memory_data: MemoryData) -> MemoryOperationResponse
  ```

  **Why This is Critical**:
  This UUID mapping issue is **blocking the entire "Store Forever" feature** because:

  - Web2 creates galleries with UUID `550e8400-e29b-41d4-a716-446655440000`
  - ICP generates its own ID like `gallery_1703123456789`
  - Systems can't communicate about the same entity
  - Storage status tracking breaks completely

  _Requirements: 26.1, 26.2, 26.3, 26.4, 26.5_

- [x] 1.1 Create Minimal ICP Error Model for MVP

  - Define basic ErrorCode enum with essential variants (Unauthorized, AlreadyExists, NotFound, InvalidHash, Internal)
  - Extend existing response types (MemoryResponse, etc.) to include ErrorCode where needed
  - Create minimal Result<T, ErrorCode> wrapper for new ICP endpoints only
  - Add basic error message formatting compatible with existing patterns
  - Preserve existing error handling patterns where they work
  - _Requirements: 22.1, MVP principles_

- [x] 1.2 Implement Stable Memory Infrastructure

  - Replace thread_local storage with ic-stable-structures
  - Create StableBTreeMap for capsules, upload sessions, and memory artifacts
  - Implement pre_upgrade and post_upgrade hooks
  - Add memory manager for multiple stable memory regions
  - _Requirements: 23.1, 23.2, 23.3_

- [x] 1.3 Implement Memory Metadata Operations

  **Goal**: Create ICP canister endpoints to store and query memory metadata (not the actual files yet, just the metadata about memories like titles, descriptions, etc.)

  **What to implement**:

  1. **upsert_metadata endpoint** - Store memory metadata on ICP

     - Function signature: `pub fn upsert_metadata(memory_id: String, memory_type: MemoryType, metadata: MemoryMetadata, idempotency_key: String) -> ICPResult<MetadataResponse>`
     - Store metadata in stable memory using `with_stable_memory_artifacts_mut`
     - Use idempotency_key to prevent duplicate writes (same key = return success without re-writing)
     - Validate memory_type is one of: Image, Video, Audio, Document, Note
     - Return MetadataResponse with success/error status

  2. **get_memory_presence_icp endpoint** - Check if a single memory's metadata exists on ICP

     - Function signature: `pub fn get_memory_presence_icp(memory_id: String) -> ICPResult<MemoryPresenceResponse>`
     - Query stable memory to check if metadata exists for this memory_id
     - Return MemoryPresenceResponse with metadata_present: bool, asset_present: bool (asset will be false for now)

  3. **get_memory_list_presence_icp endpoint** - Check presence for multiple memories (with pagination)

     - Function signature: `pub fn get_memory_list_presence_icp(memory_ids: Vec<String>, cursor: Option<String>, limit: u32) -> ICPResult<MemoryListPresenceResponse>`
     - Limit max 100 items per request, default 20
     - Return which memories have metadata stored on ICP
     - Support pagination with cursor for large lists

  4. **Add these endpoints to lib.rs** as public canister functions with #[ic_cdk::update] and #[ic_cdk::query] attributes

  **Key concepts**:

  - This is about METADATA only (titles, descriptions, etc.) - not the actual image/video files
  - Use the stable memory infrastructure from task 1.2
  - Idempotency means calling the same operation twice should be safe
  - Web2 will call these endpoints to store memory metadata on ICP before storing the actual files

  _Requirements: 22.5, 24.3, 25.3_

- [x] 1.4 Implement Chunked Asset Upload Protocol

  **Goal**: Create a robust chunked upload system for large files (images, videos) with progress tracking, error recovery, and operational constraints.

  **What to implement**:

  1. **begin_asset_upload endpoint** - Start chunked upload session

     - Function signature: `pub fn begin_asset_upload(memory_id: String, expected_hash: String, chunk_count: u32, total_size: u64) -> ICPResult<UploadSessionResponse>`
     - Create UploadSession in stable memory with session_id, expected_hash, chunk tracking
     - Validate file size limits (max 100MB), chunk count reasonable
     - Check if asset with same hash already exists (return AlreadyExists for idempotency)
     - Return session_id for subsequent chunk uploads

  2. **put_chunk endpoint** - Upload individual file chunks

     - Function signature: `pub fn put_chunk(session_id: String, chunk_index: u32, chunk_data: Vec<u8>) -> ICPResult<ChunkResponse>`
     - Validate session exists and not expired (30-minute timeout)
     - Validate chunk_index is within expected range and not already received
     - Validate chunk size (max 1MB per chunk)
     - Store chunk data in stable memory linked to session
     - Update session progress tracking (chunks_received, bytes_received)
     - Return chunk confirmation with progress info

  3. **commit_asset endpoint** - Finalize upload after all chunks received

     - Function signature: `pub fn commit_asset(session_id: String, final_hash: String) -> ICPResult<CommitResponse>`
     - Validate all chunks received and session complete
     - Reconstruct file from chunks and verify SHA-256 hash matches expected_hash
     - Create MemoryArtifact with ArtifactType::Asset in stable memory
     - Clean up upload session and temporary chunk data
     - Return success with final_hash and total_bytes

  4. **cancel_upload endpoint** - Cancel upload and cleanup resources

     - Function signature: `pub fn cancel_upload(session_id: String) -> ICPResult<()>`
     - Remove upload session and all associated chunk data
     - Safe to call multiple times (idempotent)
     - Return success even if session doesn't exist

  5. **Add session timeout and cleanup**

     - Implement session expiration (30 minutes from creation)
     - Add periodic cleanup of expired sessions
     - Handle timeout gracefully in all endpoints

  6. **Add these endpoints to lib.rs** as public canister functions with proper attributes

  **Key Features**:

  - **File Size Limits**: Max 100MB per file, max 1MB per chunk
  - **Hash Verification**: SHA-256 validation at begin and commit
  - **Session Management**: 30-minute timeout, automatic cleanup
  - **Progress Tracking**: Track chunks received, bytes uploaded
  - **Rate Limiting**: Max 3 concurrent uploads per user
  - **Idempotency**: Safe to retry operations, duplicate hash detection
  - **Error Recovery**: Graceful handling of network failures, timeouts

  **Operational Constraints**:

  - Use stable memory for persistence across canister upgrades
  - Implement proper error handling with ICPErrorCode enum
  - Add comprehensive logging for debugging and monitoring
  - Ensure memory efficiency (cleanup temporary data promptly)

  **Testing Requirements**:

  - Create bash scripts in `scripts/tests/backend/icp-upload/` for E2E testing
  - Test basic single-chunk upload flow
  - Test multi-chunk upload (3MB file = 3 chunks)
  - Test error scenarios (invalid hash, timeout, oversized files)
  - Test idempotency (duplicate uploads with same hash)
  - Test concurrent upload limits and rate limiting
  - Test session cleanup and timeout handling

  _Requirements: 18.1, 22.2, 22.3_

- [x] 1.5 Add Basic Authorization (MVP)

  - Implement caller principal verification for write operations only
  - Add basic concurrent upload limiting (max 3 per user)
  - Create simple unauthorized error responses
  - Skip detailed audit logging and quotas for MVP (can be added post-MVP)
  - _Requirements: 17.1 (partial), 22.3 (basic)_

- [ ] 1.6 Add Progress Tracking API for Large File Uploads

  **Goal**: Create API endpoints to expose upload progress data for frontend progress tracking and user feedback.

  **What to implement**:

  1. **get_upload_progress endpoint** - Query upload session progress

     - Function signature: `pub fn get_upload_progress(session_id: String) -> ICPResult<UploadProgressResponse>`
     - Return: `{ session_id, memory_id, total_chunks, chunks_received, bytes_received, total_size, progress_percentage, estimated_time_remaining, status: "active"|"completed"|"expired"|"failed" }`
     - Calculate progress percentage: `(chunks_received / total_chunks) * 100`
     - Estimate time remaining based on upload rate (bytes_received / elapsed_time)
     - Handle expired/failed sessions gracefully

  2. **get_user_upload_sessions endpoint** - List user's active upload sessions

     - Function signature: `pub fn get_user_upload_sessions() -> ICPResult<UserUploadSessionsResponse>`
     - Return: Array of active upload sessions for the caller
     - Include progress data for each session
     - Filter out expired/completed sessions
     - Limit to max 10 sessions per user for performance

  3. **Add these endpoints to lib.rs** as public canister functions with #[ic_cdk::query] attributes

  **Integration Points**:

  - Support Task 4.2 (Detailed Progress Reporting) in frontend modal
  - Enable real-time progress updates in UI components
  - Provide data for progress bars and time estimates
  - Support session monitoring and cleanup

  **Performance Considerations**:

  - Use efficient queries to stable memory
  - Cache progress calculations for active sessions
  - Implement rate limiting for progress queries
  - Clean up expired session data periodically

  _Requirements: 2.2, 10.2, 20.1_

- [ ] 1.7 Implement Backend Retry Logic for Failed ICP Operations

  **Goal**: Add automatic retry mechanisms at the backend level for transient failures and network issues.

  **What to implement**:

  1. **Retry wrapper for ICP operations** - Automatic retry for transient failures

     - Create `retry_icp_operation<T>(operation: Fn, max_retries: u32, backoff_ms: u32) -> ICPResult<T>`
     - Implement exponential backoff with jitter: `delay = min(base_delay * 2^attempt + random_jitter, max_delay)`
     - Retry only on specific error types: `ICPErrorCode::Internal` (network issues), timeout errors
     - Skip retry for permanent errors: `Unauthorized`, `NotFound`, `InvalidHash`, `AlreadyExists`

  2. **Enhanced error categorization** - Distinguish retryable vs non-retryable errors

     - Add `is_retryable_error(error: &ICPErrorCode) -> bool` helper function
     - Categorize errors: Network (retryable), Validation (not retryable), Auth (not retryable)
     - Implement circuit breaker pattern for repeated failures
     - Track failure rates per operation type

  3. **Retry configuration** - Configurable retry parameters

     - Default: 3 retries, 1000ms base delay, 10000ms max delay
     - Allow per-operation override of retry settings
     - Implement retry budget per user to prevent abuse
     - Add retry attempt tracking in session data

  4. **Apply retry logic to critical operations**:

     - `put_chunk` - Retry failed chunk uploads
     - `commit_asset` - Retry finalization failures
     - `upsert_metadata` - Retry metadata storage failures
     - Skip retry for `begin_asset_upload` (session creation)

  **Error Handling**:

  - Preserve original error context in retry attempts
  - Log retry attempts and success rates for monitoring
  - Implement graceful degradation when retries exhausted
  - Provide detailed error messages for debugging

  **Integration Points**:

  - Enhance existing error handling in Tasks 1.3 and 1.4
  - Support Task 3.3 (Frontend Error Handling) with better error categorization
  - Enable Task 4.3 (Modal Error Recovery) with retry information
  - Provide data for Task 7.4 (Production Monitoring)

  _Requirements: 6.1, 6.2, 19.1, 22.1_

- [ ] 2. Enhance Web2 Backend API for Storage Integration

  - Create storage status endpoints using existing views
  - Implement storage_edges update operations
  - Add gallery presence aggregation endpoints
  - _Requirements: 5.1, 11.1, 16.1_

- [ ] 2.1 Create Storage Status API Endpoints

  **Goal**: Create dedicated, optimized endpoints for querying storage status that the frontend can use to display storage indicators, manage "Store Forever" button states, and show progress during storage operations.

  **Current State Analysis**:

  - ✅ Database infrastructure exists: `storageEdges` table, `memory_presence` and `gallery_presence` views
  - ✅ Existing APIs: `GET /api/memories/presence`, `GET /api/storage/edges`, gallery API with storage status
  - ❌ Missing: Dedicated storage status endpoints, batch endpoints for multiple items

  **What to implement**:

  1. **GET /api/galleries/[id]/storage-status** - Dedicated gallery storage status endpoint

     - Use `gallery_presence` view for optimized queries
     - Return: `{ galleryId, totalMemories, icpCompleteMemories, icpComplete, icpAny, icpCompletePercentage, status: "stored_forever"|"partially_stored"|"web2_only" }`
     - Authentication: Check user owns gallery OR gallery is shared with user
     - Handle gallery access control (public galleries, shared galleries)

  2. **GET /api/memories/[id]/storage-status** - Dedicated memory storage status endpoint

     - Use `memory_presence` view for optimized queries
     - Return: `{ memoryId, memoryType, metaNeon, assetBlob, metaIcp, assetIcp, storageStatus: {neon, blob, icp, icpPartial}, overallStatus }`
     - Authentication: Check user owns memory OR memory is accessible via gallery sharing
     - Support query param `?type=image|video|note|document|audio`

  3. **GET /api/galleries/storage-status** - Batch gallery storage status (query param: `?ids=gallery1,gallery2,gallery3`)

     - Limit max 100 galleries per request for performance
     - Return array of gallery storage status objects
     - Filter results to only galleries user has access to

  4. **GET /api/memories/storage-status** - Batch memory storage status (query params: `?ids=memory1,memory2&types=image,video`)
     - Limit max 100 memories per request for performance
     - Return array of memory storage status objects
     - Validate memory IDs and types match (same array length)

  **Error Handling & Performance**:

  - 401 Unauthorized, 404 Not Found, 403 Forbidden, 400 Bad Request, 500 Internal Server Error
  - Use existing optimized views for performance
  - Implement proper pagination for large result sets
  - Reuse existing utilities: `addStorageStatusToGallery`, `getMemoryPresenceById`, `getGalleryPresenceById`
  - Follow same response format patterns as existing APIs

  **Integration Points**:

  - Maintain consistency with existing `/api/memories/presence` endpoint
  - Support frontend storage indicator components
  - Enable "Store Forever" button state management
  - Provide data for storage progress tracking

  _Requirements: 5.1, 5.2, 16.1_

- [ ] 2.2 Implement Storage Edges Management API

  - Create POST /api/storage/edges for batch upsert operations
  - Add PUT /api/storage/edges/[id] for individual edge updates
  - Implement sync_state transition management (idle → migrating → idle/failed)
  - Add storage_edges cleanup for failed operations
  - _Requirements: 1.5, 12.1, 12.2, 12.3_

- [ ] 2.3 Add Gallery Presence View Integration

  - Create endpoint to refresh gallery_presence materialized view
  - Implement batch refresh after storage operations
  - Add scheduled refresh mechanism for long-running operations
  - Create presence verification before marking edges as present
  - _Requirements: 11.1, 11.5, 19.5_

- [ ] 2.4 Implement Internet Identity Account Linking

  - Enhance existing II linking to support "Store Forever" flow
  - Add session preservation during II authentication
  - Create account linking verification and error handling
  - Implement retry mechanisms for failed linking attempts
  - _Requirements: 3.1, 14.1, 14.2, 14.5_

- [ ] 3. Update Frontend Gallery Service with Real ICP Integration

  - Replace placeholder storeGalleryOnICP with artifact-level protocol
  - Implement chunked upload with progress tracking
  - Add comprehensive error handling and retry logic
  - _Requirements: 1.1, 2.1, 6.1, 15.1_

- [ ] 3.1 Implement Artifact-Level Storage Protocol

  - Replace storeGalleryOnICP placeholder with real implementation
  - Create memory-by-memory processing with metadata and asset storage
  - Implement idempotency key generation and usage
  - Add presence verification before updating storage_edges
  - _Requirements: 15.1, 15.2, 22.5, 25.1_

- [ ] 3.2 Create Chunked Upload Implementation

  - Implement createChunks utility for splitting large files
  - Add ChunkedUploader class with rate limiting and concurrency control
  - Create upload progress tracking with real byte counts
  - Implement exponential backoff for failed chunk uploads
  - _Requirements: 2.2, 7.4, 20.1, 20.3_

- [ ] 3.3 Add Comprehensive Error Handling

  - Create error categorization for different failure types (auth, network, validation)
  - Implement circuit breaker pattern for ICP endpoint failures
  - Add retry logic with jitter for transient failures
  - Create user-friendly error messages for each error category
  - _Requirements: 6.1, 6.2, 19.1, 22.1_

- [ ] 3.4 Implement Content Hash Verification

  - Add SHA-256 hash computation for all uploaded assets
  - Create hash verification before and after upload
  - Implement content integrity checks with storage_edges updates
  - Add hash mismatch detection and re-upload logic
  - _Requirements: 18.1, 18.3, 24.4_

- [ ] 4. Enhance ForeverStorageProgressModal with Real Progress Tracking

  - Connect modal to real ICP operations with live progress
  - Implement step-by-step progress with actual data
  - Add detailed error states and recovery options
  - _Requirements: 2.1, 2.3, 13.1, 15.4_

- [ ] 4.1 Connect Modal to Real Storage Operations

  - Replace placeholder storeGalleryOnICP calls with real service integration
  - Add real progress tracking based on memory processing and chunk uploads
  - Implement step transitions based on actual operation completion
  - Create proper cleanup on modal close or cancellation
  - _Requirements: 2.1, 2.3, 15.4_

- [ ] 4.2 Implement Detailed Progress Reporting

  - Add memory-level progress tracking (X of Y memories processed)
  - Create chunk-level progress for large file uploads
  - Implement bytes uploaded tracking with real ICP callback data
  - Add estimated time remaining based on actual upload speeds
  - _Requirements: 2.2, 10.2, 20.1_

- [ ] 4.3 Enhance Error Handling and Recovery

  - Create specific error states for each failure type
  - Add retry functionality for failed operations
  - Implement partial success handling (some memories stored, others failed)
  - Create detailed error reporting with actionable next steps
  - _Requirements: 6.1, 6.4, 13.4_

- [ ] 4.4 Fix Modal Effect Dependencies

  - Audit all useEffect and useCallback dependencies for stability
  - Replace changing function references with stable alternatives
  - Add comprehensive tests for modal state management
  - Implement proper cleanup to prevent memory leaks
  - _Requirements: 13.1, 13.2, 13.5_

- [ ] 5. Add Storage Status UI Components

  - Create gallery storage status badges and indicators
  - Implement partial storage visualization
  - Add "Store Forever" button state management
  - _Requirements: 5.1, 5.2, 16.2, 21.1_

- [ ] 5.1 Create Storage Status Badge Components

  - Implement "Stored Forever" badge for fully stored galleries
  - Create "Partially on ICP" badge with progress indicators
  - Add "Storing..." badge for galleries in progress
  - Create hover tooltips with detailed storage information
  - _Requirements: 5.1, 5.2, 16.3_

- [ ] 5.2 Implement Gallery Card Storage Indicators

  - Add storage status queries to existing gallery cards
  - Create visual indicators for storage state (icons, colors, badges)
  - Implement real-time status updates during storage operations
  - Add click handlers for storage status details
  - _Requirements: 5.1, 16.1, 21.1_

- [ ] 5.3 Enhance Store Forever Button Logic

  - Implement dynamic button text based on storage status
  - Add "Already Stored" state for completed galleries
  - Create "Continue Storing" option for partial galleries
  - Add "View on ICP" functionality for stored galleries
  - _Requirements: 16.2, 16.5, 21.1_

- [ ] 5.4 Create Partial Storage Detail View

  - Implement per-memory storage status indicators
  - Add "Complete Storage" call-to-action for partial galleries
  - Create detailed breakdown of what's stored where
  - Add individual memory "Store on ICP" functionality
  - _Requirements: 21.1, 21.4_

- [ ] 6. Implement Comprehensive Testing Suite

  - Create unit tests for all new components and functions
  - Add integration tests for end-to-end storage flow
  - Implement performance tests for large gallery storage
  - _Requirements: 13.5, 23.4_

- [ ] 6.1 Create Frontend Component Tests

  - Test ForeverStorageProgressModal state management and transitions
  - Add tests for storage status components and badge rendering
  - Create tests for gallery service error handling and retry logic
  - Implement tests for chunked upload functionality
  - _Requirements: 13.5, 4.1_

- [ ] 6.2 Implement Backend API Tests

  - Test storage_edges CRUD operations and idempotency
  - Add tests for storage status endpoints and view queries
  - Create tests for II account linking and session management
  - Implement tests for gallery presence view refresh
  - _Requirements: 11.5, 14.1_

- [ ] 6.3 Create ICP Canister Tests

  - Test all artifact-level API endpoints with various scenarios
  - Add tests for stable memory persistence across upgrades
  - Create tests for authorization, quotas, and rate limiting
  - Implement tests for chunked upload protocol and error handling
  - _Requirements: 17.1, 22.1, 23.4_

- [ ] 6.4 Add Integration and Performance Tests

  - Test complete end-to-end storage flow from UI to ICP
  - Add performance tests for large galleries (1000+ memories)
  - Create tests for concurrent user operations and rate limiting
  - Implement tests for network failure scenarios and recovery
  - _Requirements: 19.1, 20.1, 20.4_

- [ ] 7. Deploy and Monitor Production Integration

  - Deploy enhanced ICP canister with stable memory
  - Update Web2 backend with new storage endpoints
  - Deploy frontend changes with feature flags
  - _Requirements: Deployment and monitoring_

- [ ] 7.1 Deploy ICP Canister Updates

  - Deploy canister with new artifact-level API and stable memory
  - Verify canister upgrade preserves existing data
  - Test all new endpoints in production environment
  - Monitor canister performance and resource usage
  - _Requirements: 23.1, 23.3_

- [ ] 7.2 Deploy Web2 Backend Changes

  - Deploy new storage status and edges management endpoints
  - Update existing gallery API to include storage status
  - Deploy gallery_presence view refresh mechanisms
  - Monitor API performance and error rates
  - _Requirements: 11.1, 19.5_

- [ ] 7.3 Deploy Frontend Updates with Feature Flags

  - Deploy updated gallery service with feature flag protection
  - Enable "Store Forever" functionality for beta users
  - Monitor modal performance and user completion rates
  - Collect user feedback and error reports
  - _Requirements: Feature deployment_

- [ ] 7.4 Implement Production Monitoring
  - Add performance monitoring for storage operations
  - Create alerts for high error rates and quota violations
  - Implement audit log analysis and reporting
  - Monitor ICP canister cycles and resource usage
  - _Requirements: 17.2, 19.1_

## Task Dependencies

### Critical Path

1. Tasks 1.1-1.5 (ICP Canister API) must be completed first as they provide the foundation
2. Tasks 2.1-2.4 (Web2 Backend) depend on ICP API completion
3. Tasks 3.1-3.4 (Frontend Service) depend on both ICP and Web2 backend completion
4. Tasks 4.1-4.4 (Modal Enhancement) depend on frontend service completion
5. Tasks 5.1-5.4 (UI Components) can be developed in parallel with modal work
6. Tasks 6.1-6.4 (Testing) should be developed alongside each component
7. Tasks 7.1-7.4 (Deployment) are the final integration phase

### Parallel Development Opportunities

- ICP canister development (1.x) can proceed independently
- Frontend service (3.x) and modal (4.x) can be developed in parallel once backend is ready
- UI components (5.x) can be developed alongside modal work
- Testing (6.x) should be developed incrementally with each component

### Integration Points

- Task 3.1 integrates ICP canister API with frontend service
- Task 4.1 integrates frontend service with existing modal UI
- Task 5.2 integrates storage status with existing gallery components
- Task 7.x integrates all components in production environment

## Success Criteria

Each task is considered complete when:

1. **Code Implementation**: All specified functionality is implemented and working
2. **Unit Tests**: Comprehensive tests cover the new functionality
3. **Integration**: The component integrates properly with existing systems
4. **Error Handling**: Robust error handling covers all failure scenarios
5. **Documentation**: Code is properly documented and commented
6. **Performance**: Implementation meets performance requirements
7. **Security**: All security requirements are properly implemented

The overall feature is complete when users can successfully store galleries on ICP through the existing UI with full progress tracking, error handling, and storage status visibility.
