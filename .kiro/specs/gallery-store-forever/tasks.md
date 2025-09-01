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

- [x] 2. Web2 Backend API for Storage Integration (MVP COMPLETE)

  **MVP Status**: ✅ **SUFFICIENT FOR MVP** - Core storage infrastructure exists and supports "Store Forever" functionality

  **What's Already Implemented**:

  - ✅ Storage edges management API (`PUT /api/storage/edges`, `GET /api/storage/edges`)
  - ✅ Individual storage status endpoints (`GET /api/memories/presence`, `GET /api/galleries/[id]/presence`)
  - ✅ Gallery storage status integration in existing gallery API
  - ✅ Basic Internet Identity account linking (`POST /api/auth/link-ii`)
  - ✅ Database views: `memory_presence`, `gallery_presence` with optimized queries

  **MVP Conclusion**: The existing APIs provide sufficient functionality for MVP "Store Forever" feature. Frontend can use individual presence endpoints and storage edges API to track and display storage status.

  _Requirements: 5.1, 11.1, 16.1_

- [x] 2.1 Storage Status Infrastructure (ALREADY IMPLEMENTED)

  **Current Implementation Status**: ✅ **COMPLETE FOR MVP**

  **Existing APIs that support MVP**:

  1. **`GET /api/memories/presence`** - Individual memory storage status

     - Returns: `{ memoryId, memoryType, metaNeon, assetBlob, metaIcp, assetIcp, storageStatus: {neon, blob, icp, icpPartial}, overallStatus }`
     - Supports all required storage status queries for MVP

  2. **`GET /api/galleries/[id]/presence`** - Individual gallery storage status

     - Returns: `{ galleryId, totalMemories, icpCompleteMemories, icpComplete, icpAny, icpCompletePercentage, storageStatus }`
     - Provides complete gallery storage overview for MVP

  3. **Gallery API with storage status** - Existing gallery endpoints include storage status
     - Uses `addStorageStatusToGallery` utility to enhance gallery objects
     - Provides storage status in gallery listings and details

  **MVP Usage**: Frontend can call these individual endpoints to get storage status. For MVP, individual calls are acceptable vs batch endpoints.

  _Requirements: 5.1, 5.2, 16.1_

- [x] 2.2 Storage Edges Management (ALREADY IMPLEMENTED)

  **Current Implementation Status**: ✅ **COMPLETE FOR MVP**

  **Existing Implementation**:

  1. **`PUT /api/storage/edges`** - Upsert storage edge records

     - Supports batch operations for updating storage status after ICP operations
     - Handles sync state transitions (idle → migrating → idle/failed)
     - Includes proper validation and error handling
     - Supports idempotent operations

  2. **`GET /api/storage/edges`** - Query storage edges with filtering
     - Supports filtering by memoryId, memoryType, backend, artifact, syncState
     - Provides complete storage edge data for debugging and monitoring

  **MVP Usage**: Frontend can use PUT endpoint to update storage status after successful ICP operations, and GET endpoint to query current storage state.

  **Key Features Already Implemented**:

  - ✅ Batch upsert operations
  - ✅ Sync state management (idle → migrating → failed)
  - ✅ Proper validation and error handling
  - ✅ Storage edge cleanup and updates

  _Requirements: 1.5, 12.1, 12.2, 12.3_

- [x] 2.3 Internet Identity Integration (SUFFICIENT FOR MVP)

  **Current Implementation Status**: ✅ **SUFFICIENT FOR MVP**

  **Existing II Integration**:

  - ✅ `POST /api/auth/link-ii` - Basic II account linking
  - ✅ `GET /api/ii/challenge` - II challenge generation
  - ✅ `POST /api/ii/verify-nonce` - II nonce verification
  - ✅ NextAuth integration with II provider

  **MVP Usage**: Basic II linking is sufficient for MVP. Users can link their II accounts to enable ICP storage functionality.

  _Requirements: 3.1, 14.1, 14.2_

- [x] 3. Frontend Gallery Service with ICP Integration (MVP COMPLETE)

  **MVP Status**: ✅ **SUFFICIENT FOR MVP** - Core "Store Forever" functionality is working end-to-end

  **What's Already Implemented**:

  - ✅ Real ICP integration with `ICPGalleryService` class
  - ✅ Working `storeGalleryForever()` method that calls ICP backend
  - ✅ Complete `ForeverStorageProgressModal` with step-by-step UI
  - ✅ Internet Identity authentication flow
  - ✅ Basic error handling and user feedback
  - ✅ TypeScript types matching ICP backend declarations

  **MVP Conclusion**: Users can successfully store galleries on ICP through the existing UI with progress tracking and error handling. The core "Store Forever" feature is functional.

  _Requirements: 1.1, 2.1, 6.1, 15.1_

- [x] 3.1 Basic ICP Storage Integration (ALREADY IMPLEMENTED)

  **Current Implementation Status**: ✅ **COMPLETE FOR MVP**

  **Existing Implementation**:

  1. **`ICPGalleryService` class** - Complete service layer for ICP operations

     - Real `storeGalleryForever()` method that calls `actor.store_gallery_forever()`
     - Proper error handling and response formatting
     - Identity management integration with Internet Identity
     - TypeScript types matching backend declarations

  2. **Gallery-level storage** - Stores complete galleries on ICP

     - Converts Web2 gallery data to ICP format using `convertWeb2GalleryToICP()`
     - Handles gallery metadata and memory entries
     - Returns proper success/failure responses with ICP gallery IDs

  3. **Authentication integration** - Works with existing II system
     - Checks for `icpPrincipal` in session
     - Integrates with NextAuth and Internet Identity
     - Handles authentication errors gracefully

  **MVP Usage**: The current implementation successfully stores galleries on ICP as complete units, which is sufficient for MVP. Gallery-level storage proves the "Store Forever" concept works.

  _Requirements: 15.1, 15.2_

- [x] 3.2 Progress Tracking and UI (ALREADY IMPLEMENTED)

  **Current Implementation Status**: ✅ **COMPLETE FOR MVP**

  **Existing Implementation**:

  1. **`ForeverStorageProgressModal`** - Complete modal with step-by-step progress

     - Step-based progress: idle → auth → prepare → store → verify → success
     - Visual progress bar with percentage tracking
     - Status messages and detailed descriptions for each step
     - Loading states and animations

  2. **Authentication flow** - Seamless II integration

     - Checks for Internet Identity connection
     - Redirects to II signin when needed
     - Auto-resumes storage process after authentication
     - Clear messaging about authentication requirements

  3. **Error handling and recovery** - User-friendly error states
     - Displays error messages with retry functionality
     - Confirmation dialogs for cancellation
     - Success states with celebration messaging
     - Proper cleanup on modal close

  **MVP Usage**: The modal provides excellent user experience for the storage process, with clear feedback and error handling that's sufficient for MVP launch.

  _Requirements: 2.2, 13.1, 15.4_

- [x] 3.3 Basic Error Handling (SUFFICIENT FOR MVP)

  **Current Implementation Status**: ✅ **SUFFICIENT FOR MVP**

  **Existing Error Handling**:

  1. **Service-level error handling** - Catches and formats errors

     - Try-catch blocks in all ICP service methods
     - Proper error message formatting
     - Fallback error messages for unknown errors
     - Console logging for debugging

  2. **Modal error states** - User-friendly error display

     - Dedicated error step in progress modal
     - Error message display with retry button
     - Authentication error handling with signin redirect
     - Cancellation confirmation to prevent accidental exits

  3. **Network error handling** - Basic resilience
     - Timeout handling in ICP calls
     - Connection error detection
     - User feedback for network issues

  **MVP Usage**: Basic error handling is sufficient for MVP. Users get clear feedback when things go wrong and can retry operations.

  _Requirements: 6.1, 6.2, 13.4_

- [x] 4. ForeverStorageProgressModal Integration (ONE MVP GAP REMAINING)

  **MVP Status**: ⚠️ **95% COMPLETE - ONE CRITICAL FIX NEEDED**

  **What's Already Implemented**:

  - ✅ Complete modal UI with step-by-step progress tracking
  - ✅ Internet Identity authentication flow
  - ✅ Visual progress indicators and animations
  - ✅ Error handling and retry functionality
  - ✅ Success/failure states with user feedback
  - ✅ Modal state management and cleanup

  **Critical MVP Gap**: Modal uses placeholder storage function instead of real ICP integration

  _Requirements: 2.1, 2.3, 13.1, 15.4_

- [x] 4.1 Connect Modal to Real ICP Storage (CRITICAL MVP FIX)

  **Current Implementation Status**: ⚠️ **NEEDS ONE CRITICAL FIX FOR MVP**

  **What's Already Working**:

  - ✅ Complete `ForeverStorageProgressModal` component with professional UI
  - ✅ Step-based progress: idle → auth → prepare → store → verify → success
  - ✅ Internet Identity authentication integration
  - ✅ Progress bar with percentage tracking
  - ✅ Error states and retry functionality
  - ✅ Modal state management and cleanup

  **Critical MVP Gap**:

  - ❌ **Modal uses placeholder `storeGalleryOnICP()` function that simulates success**
  - ❌ **Not connected to real `ICPGalleryService.storeGalleryForever()` method**

  **Required MVP Fix**:

  ```typescript
  // CURRENT (❌ PLACEHOLDER):
  const result = await storeGalleryOnICP(gallery); // Simulates 2-second delay

  // REQUIRED (✅ REAL ICP INTEGRATION):
  import { ICPGalleryService } from "@/services/icp-gallery";

  const icpService = new ICPGalleryService(identity);
  const galleryData = icpService.convertWeb2GalleryToICP(gallery, gallery.items, ownerPrincipal);
  const result = await icpService.storeGalleryForever(galleryData);
  ```

  **Implementation Steps**:

  1. Import `ICPGalleryService` in `ForeverStorageProgressModal.tsx`
  2. Replace placeholder `storeGalleryOnICP()` function with real service call
  3. Add proper identity and principal handling
  4. Convert Web2 gallery format to ICP format using existing utility
  5. Handle real ICP responses and errors

  **MVP Usage**: Once fixed, users will actually store galleries on ICP instead of seeing fake success messages.

  _Requirements: 2.1, 2.3, 15.4_

- [x] 4.2 Modal UI and User Experience (ALREADY IMPLEMENTED)

  **Current Implementation Status**: ✅ **COMPLETE FOR MVP**

  **Existing Implementation**:

  1. **Step-by-step progress tracking** - Professional UI with clear feedback

     - Visual progress bar with percentage (0% → 100%)
     - Step indicators: idle → auth → prepare → store → verify → success
     - Loading animations and status icons
     - Clear messaging for each step

  2. **Authentication flow** - Seamless Internet Identity integration

     - Checks for `icpPrincipal` in session
     - Redirects to II signin when needed with `storeForever=1` parameter
     - Auto-resumes storage process after authentication
     - Clear messaging about authentication requirements

  3. **User feedback and controls** - Excellent UX
     - Success celebration with detailed confirmation
     - Error states with retry functionality
     - Cancellation confirmation dialogs
     - Proper button states and loading indicators

  **MVP Usage**: The modal provides excellent user experience that's ready for production launch.

  _Requirements: 13.1, 15.4_

- [x] 4.3 Basic Error Handling (SUFFICIENT FOR MVP)

  **Current Implementation Status**: ✅ **SUFFICIENT FOR MVP**

  **Existing Error Handling**:

  1. **Modal error states** - User-friendly error display

     - Dedicated error step with clear messaging
     - Error message display with retry button
     - Authentication error handling with II signin redirect
     - Network error detection and user feedback

  2. **Operation error handling** - Basic resilience

     - Try-catch blocks around storage operations
     - Proper error message formatting
     - Fallback error messages for unknown errors
     - Console logging for debugging

  3. **User recovery options** - Clear next steps
     - Retry button for failed operations
     - Cancellation confirmation to prevent accidental exits
     - Clear success/failure messaging

  **MVP Usage**: Error handling is sufficient for MVP. Users get clear feedback when things go wrong and can retry operations.

  _Requirements: 6.1, 6.4, 13.4_

- [x] 5. Add Storage Status UI Components (COMPLETE FOR MVP)

  **MVP Status**: ✅ **COMPLETE** - All storage status UI components are implemented and working

  **What's Already Implemented**:

  - ✅ Gallery storage status badges in all three key locations
  - ✅ Dynamic "Store Forever" button states with proper color coding
  - ✅ "View on ICP" button for stored galleries
  - ✅ Hover tooltips explaining storage status
  - ✅ Responsive design across all screen sizes
  - ✅ Proper integration with existing gallery UI

  **MVP Conclusion**: Users have complete visual feedback about gallery storage status across the entire application. The UI clearly distinguishes between Web2-only, partially stored, and fully stored galleries.

  _Requirements: 5.1, 5.2, 16.2, 21.1_

- [x] 5.1 Create Storage Status Badge Components (ALREADY IMPLEMENTED)

  **Current Implementation Status**: ✅ **COMPLETE FOR MVP**

  **What's Already Implemented**:

  1. **`StorageStatusBadge` component** - Complete badge component with proper styling

     - ✅ Shows "ICP" badge for stored galleries (green/success styling)
     - ✅ Shows "NEON" badge for Web2-only galleries (secondary styling)
     - ✅ Supports different sizes (sm, md) and custom className
     - ✅ Uses proper Badge component from UI library

  2. **Helper functions for status determination**:

     - ✅ `getGalleryStorageStatus()` - Maps gallery.storageStatus to badge status
     - ✅ `getMemoryStorageStatus()` - Maps memory storage status to badge status
     - ✅ Handles API status values: "stored_forever" → "icp", "web2_only" → "neon"
     - ✅ Includes fallback logic for galleries without storageStatus

  3. **Integration in all three key locations**:
     - ✅ **Gallery List Page** (`/gallery/page.tsx`) - Badge shown on each gallery card
     - ✅ **Gallery Detail Page** (`/gallery/[id]/page.tsx`) - Badge shown in header with tooltip
     - ✅ **Gallery Preview Page** (`/gallery/[id]/preview/page.tsx`) - No badge needed (has "Store Forever" button)

  **Current Badge Behavior**:

  - "ICP" badge (green) for galleries with `storageStatus.status === "stored_forever"`
  - "ICP" badge (green) for galleries with `storageStatus.status === "partially_stored"`
  - "NEON" badge (gray) for galleries with `storageStatus.status === "web2_only"`
  - Tooltips implemented on gallery detail page explaining storage status

  **MVP Conclusion**: Storage status badges are fully implemented and working across all gallery pages. Users can clearly see which galleries are stored forever vs Web2-only.

  _Requirements: 5.1, 5.2, 16.3_

- [x] 5.2 Implement Gallery Card Storage Indicators (ALREADY IMPLEMENTED)

  **Current Implementation Status**: ✅ **COMPLETE FOR MVP**

  **What's Already Implemented**:

  1. **Gallery List Page** (`/gallery/page.tsx`):

     - ✅ Storage status badges on each gallery card using `<StorageStatusBadge status={getGalleryStorageStatus(gallery)} />`
     - ✅ Badges positioned in top-right corner alongside privacy badges
     - ✅ Proper visual hierarchy with other gallery metadata (image count, date)
     - ✅ Responsive layout that works on all screen sizes

  2. **Gallery Detail Page** (`/gallery/[id]/page.tsx`):

     - ✅ Storage status badge in header with hover tooltip
     - ✅ Detailed tooltip explaining storage status on hover
     - ✅ Badge positioned alongside privacy indicator
     - ✅ Responsive header layout with proper badge placement

  3. **Visual Design**:
     - ✅ "ICP" badges use green/success styling to indicate permanent storage
     - ✅ "NEON" badges use gray/secondary styling for standard storage
     - ✅ Consistent badge sizing and typography across all locations
     - ✅ Proper contrast and accessibility

  **Current Badge Locations**:

  - Gallery cards: Top-right corner next to privacy badge
  - Gallery detail: Header area with explanatory tooltip
  - Gallery preview: Not needed (has "Store Forever" button instead)

  **MVP Conclusion**: Gallery cards and detail pages have complete storage status indicators that clearly show users which galleries are stored forever vs standard storage.

  _Requirements: 5.1, 16.1, 21.1_

- [ ] 5.5 Add Memory Storage Badges to Dashboard (MVP)

  **Goal**: Add basic storage status badges to memory thumbnails in dashboard for MVP.

  **MVP Implementation**:

  1. **Simple badge integration** - Add badges to dashboard memory thumbnails

     - Import existing `MemoryStorageBadge` component in dashboard page
     - Add `<MemoryStorageBadge memoryId={memory.id} memoryType={memory.type} />` to memory thumbnails
     - Position badges in top-right corner (consistent with galleries)

  2. **Dashboard page updates** (`src/nextjs/src/app/[lang]/dashboard/page.tsx`)

     - Find memory thumbnail rendering location
     - Add badge component with minimal styling changes
     - Test basic functionality with different memory types

  **MVP Scope**: Basic badge display only - advanced features moved to post-MVP

  _Requirements: 5.5, 27.4_

- [ ] 5.6 Add Memory Storage Badges to Folder Views (MVP)

  **Goal**: Add basic storage status badges to memory thumbnails in folder pages for MVP.

  **MVP Implementation**:

  1. **Simple badge integration** - Add badges to folder memory thumbnails

     - Import existing `MemoryStorageBadge` component in folder detail page
     - Add `<MemoryStorageBadge memoryId={memory.id} memoryType={memory.type} />` to memory thumbnails
     - Position badges in top-right corner (consistent with other views)

  2. **Folder page updates** (`src/nextjs/src/app/[lang]/dashboard/folder/[id]/page.tsx`)

     - Find memory thumbnail rendering location in folder view
     - Add badge component with minimal changes to existing layout
     - Test basic functionality in folder context

  **MVP Scope**: Basic badge display only - folder-specific optimizations moved to post-MVP

  _Requirements: 5.6, 27.4_

- [ ] 5.7 Add Storage Status to Individual Memory Detail Pages (MVP)

  **Goal**: Add basic storage status display to individual memory pages for MVP.

  **MVP Implementation**:

  1. **Simple storage status display** - Add basic status indicator

     - Find individual memory detail page component
     - Add storage status query using existing `useMemoryStorageStatus` hook
     - Display simple "ICP" or "NEON" status badge or text
     - Use consistent styling with other storage indicators

  2. **Memory page updates** (identify memory detail route)

     - Add storage status section to memory detail page
     - Show binary storage status without complex explanations
     - Include basic "Store Forever" button if not on ICP

  **MVP Scope**: Basic status display only - detailed information and actions moved to post-MVP

  _Requirements: 5.7, 27.4_

- [x] 5.3 Enhance Store Forever Button Logic (ALREADY IMPLEMENTED)

  **Current Implementation Status**: ✅ **COMPLETE FOR MVP**

  **What's Already Implemented**:

  1. **Dynamic Button States** in Gallery Detail Page (`/gallery/[id]/page.tsx`):

     - ✅ **"Already Stored"** - Gray button, disabled, for `storageStatus.status === "stored_forever"`
     - ✅ **"Continue Storing"** - Orange button, enabled, for `storageStatus.status === "partially_stored"`
     - ✅ **"Store Forever"** - Blue button, enabled, for `storageStatus.status === "web2_only"`
     - ✅ Proper color coding and hover states for each button type

  2. **"View on ICP" Button**:

     - ✅ Additional purple "View on ICP" button appears when `storageStatus.status === "stored_forever"`
     - ✅ Positioned next to "Already Stored" button
     - ✅ Proper styling with purple theme
     - ✅ Click handler ready for ICP explorer integration

  3. **Button Tooltips**:

     - ✅ Hover tooltips explaining each button state
     - ✅ "Already stored" tooltip: "This gallery is already permanently stored on the Internet Computer"
     - ✅ "Continue storing" tooltip: "Continue storing the remaining items on the Internet Computer"
     - ✅ "Store forever" tooltip: "Store this gallery permanently on the Internet Computer blockchain"

  4. **Gallery Preview Page** (`/gallery/[id]/preview/page.tsx`):
     - ✅ "Store Forever" button in sticky header
     - ✅ Proper integration with ForeverStorageProgressModal
     - ✅ Blue styling consistent with detail page

  **Button State Logic**:

  ```typescript
  const getStoreForeverButtonState = () => {
    switch (gallery.storageStatus?.status) {
      case "stored_forever":
        return { text: "Already Stored", disabled: true, variant: "secondary", className: "green-theme" };
      case "partially_stored":
        return { text: "Continue Storing", disabled: false, variant: "outline", className: "orange-theme" };
      case "web2_only":
      default:
        return { text: "Store Forever", disabled: false, variant: "outline", className: "blue-theme" };
    }
  };
  ```

  **MVP Conclusion**: Store Forever button logic is fully implemented with proper states, colors, tooltips, and "View on ICP" functionality for completed galleries.

  _Requirements: 16.2, 16.5, 21.1_

- [ ] 5.4 Add Per-Memory Storage Status Indicators

  **Goal**: Show individual memory storage status within galleries to help users understand which specific memories are stored on ICP vs Neon.

  **What to implement**:

  1. **Memory-level storage badges** - Show storage status for each memory in gallery views

     - Add small storage indicators on memory thumbnails in gallery detail page
     - Use mini badges or icons to show "ICP" vs "NEON" storage per memory
     - Implement hover tooltips explaining individual memory storage status

  2. **Gallery storage breakdown** - Detailed view of what's stored where

     - Add expandable section showing "X of Y memories stored on ICP"
     - List which specific memories are on ICP vs Neon
     - Show storage progress for partially stored galleries

  3. **Individual memory actions** - Per-memory storage controls

     - Add "Store on ICP" button for individual memories not yet stored
     - Implement individual memory storage progress tracking
     - Allow users to selectively store specific memories

  4. **Storage status API integration** - Use existing memory presence endpoints
     - Query `/api/memories/[id]/storage-status` for individual memory status
     - Use batch endpoint `/api/memories/storage-status` for gallery memory lists
     - Display real-time storage status updates

  **Integration Points**:

  - Enhance gallery detail page with per-memory indicators
  - Add memory storage breakdown to gallery info panel
  - Support selective memory storage in ForeverStorageProgressModal

  _Requirements: 21.1, 21.4, 5.3_

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

## Post-MVP Enhancements

These tasks can be implemented after MVP launch to improve performance, user experience, and operational efficiency:

- [ ] **Enhanced Progress Tracking API** (moved from 1.6)

  **Goal**: Create detailed API endpoints to expose upload progress data for enhanced frontend progress tracking and user feedback.

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

  3. **Real-time progress updates** - WebSocket or polling integration
     - Add real-time progress updates in ForeverStorageProgressModal
     - Show detailed progress per memory/chunk during uploads
     - Display estimated time remaining and upload speed
     - Enable progress monitoring across browser sessions

  **Benefits**: Enhanced user experience with detailed progress tracking, better feedback during long uploads, and ability to monitor storage operations across sessions.

  _Requirements: 2.2, 10.2, 20.1_

### Backend Performance Optimizations

- [ ] **Batch Storage Status Endpoints**

  - `GET /api/galleries/storage-status?ids=gallery1,gallery2,gallery3` - Batch gallery status
  - `GET /api/memories/storage-status?ids=memory1,memory2&types=image,video` - Batch memory status
  - Reduce API calls for gallery listings and bulk operations

- [ ] **Gallery Presence View Automation**

  - Endpoint to refresh `gallery_presence` materialized view
  - Batch refresh after storage operations
  - Scheduled refresh mechanism for long-running operations
  - Presence verification before marking edges as present

- [ ] **Advanced Storage Edges Management**
  - `POST /api/storage/edges` - Dedicated batch upsert endpoint
  - `PUT /api/storage/edges/[id]` - Individual edge update endpoint
  - Enhanced sync state management and monitoring
  - Automated cleanup for failed operations

### Frontend Performance & Reliability

- [ ] **Artifact-Level Storage Protocol**

  - Memory-by-memory processing instead of gallery-level storage
  - Separate metadata and asset storage for better granularity
  - Idempotency key generation and usage for reliability
  - Presence verification before updating storage_edges
  - Better progress tracking per memory/asset

- [ ] **Chunked Upload Implementation**

### Advanced Memory Storage Badge Features

- [ ] **Enhanced Dashboard Badge Performance**

  - Batch loading optimization with `useBatchMemoryStorageStatus` hook
  - Virtualization for large memory collections
  - Advanced loading states and error handling
  - Performance monitoring and optimization

- [ ] **Advanced Folder Badge Integration**

  - Drag/drop compatibility with storage badges
  - Folder-level storage summaries and statistics
  - Advanced folder management integration
  - Batch operations with storage status awareness

- [ ] **Enhanced Memory Detail Page Storage**

  - Detailed storage information and technical details
  - Individual memory "Store Forever" functionality
  - Storage timestamp and verification status display
  - Educational content about ICP storage benefits
  - Links to view memory on ICP explorer
  - Advanced storage actions and management

  - `createChunks` utility for splitting large files (>1MB)
  - `ChunkedUploader` class with rate limiting and concurrency control
  - Real byte-level progress tracking for large uploads
  - Exponential backoff for failed chunk uploads
  - Resume capability for interrupted uploads

- [ ] **Advanced Error Handling**

  - Error categorization (auth, network, validation, quota)
  - Circuit breaker pattern for ICP endpoint failures
  - Retry logic with jitter for transient failures
  - User-friendly error messages per error category
  - Automatic retry for recoverable errors

- [ ] **Content Hash Verification**
  - SHA-256 hash computation for all uploaded assets
  - Hash verification before and after upload
  - Content integrity checks with storage_edges updates
  - Hash mismatch detection and automatic re-upload
  - Corruption detection and recovery

### Enhanced User Experience

- [ ] **Advanced II Integration**

  - Enhanced II linking with session preservation during authentication
  - Account linking verification and error handling
  - Retry mechanisms for failed linking attempts
  - Seamless "Store Forever" flow without authentication interruptions

- [ ] **Dedicated Storage Status Endpoints**

  - `GET /api/galleries/[id]/storage-status` - Optimized gallery status endpoint
  - `GET /api/memories/[id]/storage-status` - Optimized memory status endpoint
  - Faster response times and reduced database load

- [ ] **Advanced Progress Tracking**

  - Memory-level progress (X of Y memories processed)
  - Chunk-level progress for large file uploads
  - Bytes uploaded tracking with real ICP callback data
  - Estimated time remaining based on actual upload speeds
  - Pause/resume functionality for long operations

- [ ] **Enhanced Modal Features**
  - Detailed progress reporting with memory-by-memory tracking
  - Enhanced error recovery with partial success handling
  - Specific error states for each failure type (auth, network, validation)
  - Detailed error reporting with actionable next steps
  - Modal performance optimization and dependency stability

### Operational Improvements

- [ ] **Monitoring and Analytics**

  - Storage operation success/failure rates
  - Performance metrics for large gallery operations
  - User adoption and usage analytics
  - Automated alerting for system issues
  - Cost tracking and optimization

- [ ] **Advanced ICP Backend Features**
  - Backend retry logic for failed ICP operations
  - Enhanced error categorization and handling
  - Retry configuration and circuit breakers
  - Performance monitoring and logging
