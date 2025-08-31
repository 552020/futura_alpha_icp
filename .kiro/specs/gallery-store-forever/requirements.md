# Requirements Document

## Introduction

The "Store Forever" feature enables users to permanently store their photo galleries on the Internet Computer (ICP) blockchain, providing immutable, decentralized storage that ensures their memories are preserved forever. This feature builds upon the existing Web2 gallery system and storage_edges architecture to replicate gallery data to ICP's blockchain infrastructure.

**MVP Development Principles:**

- **MVP over Clean Code**: Since the existing code is working with tests in place, we will change only what is necessary to implement the feature
- **Minimal Changes**: Preserve existing patterns and structures where possible
- **Incremental Enhancement**: Build upon current error handling rather than replacing it entirely
- **Working Software First**: Prioritize functional implementation over architectural perfection

**Current Implementation Status:**

- ✅ "Store Forever" buttons are implemented in gallery detail and preview pages
- ✅ ForeverStorageProgressModal UI component is implemented with step-by-step progress
- ✅ storage_edges table and memory_presence/gallery_presence views are implemented
- ✅ ICP gallery service structure is implemented with placeholder functions
- ✅ Internet Identity account linking flow is implemented
- 🔄 Backend ICP canister integration needs to be connected to real endpoints
- 🔄 Storage edge updates need to be integrated with the storage flow
- 🔄 Gallery presence views need to be used for storage status display

## Requirements

### Requirement 1

**User Story:** As a user, I want to store my gallery permanently on the Internet Computer using the existing "Store Forever" button, so that my memories are preserved forever and cannot be lost due to server failures or service shutdowns.

#### Acceptance Criteria

1. WHEN a user clicks "Store Forever" on a gallery detail or preview page THEN the existing ForeverStorageProgressModal SHALL open and initiate the ICP storage process
2. WHEN the storage process begins THEN the system SHALL check for linked Internet Identity and prompt for linking if needed
3. WHEN authentication is complete THEN the system SHALL convert the gallery data to ICP-compatible format using existing convertWeb2GalleryToICP function
4. WHEN data conversion is complete THEN the system SHALL call the real backend canister store_gallery_forever endpoint (not placeholder)
5. WHEN storage is successful THEN after ICP commit and hash verification, the system SHALL upsert storage_edges(..., backend='icp-canister') with present=true, sync_state='idle', last_synced_at; gallery status is derived from views
6. WHEN storage fails THEN the existing error handling in ForeverStorageProgressModal SHALL provide clear error messages and allow retry

### Requirement 2

**User Story:** As a user, I want to see clear progress during the storage process using the existing progress modal, so that I understand what's happening and feel confident the process is working.

#### Acceptance Criteria

1. WHEN the storage process starts THEN the existing ForeverStorageProgressModal SHALL display with step-by-step indicators (auth, prepare, store, verify, success)
2. WHEN each step begins THEN the existing progress indicator SHALL update to show current step with appropriate icons and messages
3. WHEN a step is in progress THEN the existing modal SHALL show loading animations and progress messages with real progress data
4. WHEN a step completes THEN the existing modal SHALL show success indicators and move to next step
5. WHEN all steps complete THEN the existing success state SHALL show celebration message with storage confirmation
6. WHEN an error occurs THEN the existing error handling SHALL show specific error messages with retry functionality

### Requirement 3

**User Story:** As a user, I want my ICP identity to be managed automatically, so that I can use the "Store Forever" feature without complex blockchain setup.

#### Acceptance Criteria

1. WHEN "Store Forever" starts AND no internet-identity account is linked THEN the system SHALL link II via account row (provider='internet-identity', type='oidc') if missing; never replace the current session
2. WHEN a user has an existing linked II account THEN the system SHALL use that account for ICP operations
3. WHEN II authentication completes THEN the system SHALL create an account row (provider='internet-identity', type='oidc') linked to the same user
4. WHEN the linked principal is ready THEN the system SHALL use it for all ICP operations
5. WHEN II authentication fails THEN the system SHALL provide clear error messages and retry options

### Requirement 4

**User Story:** As a user, I want my gallery data to be properly converted for ICP storage, so that all my memories and metadata are preserved accurately.

#### Acceptance Criteria

1. WHEN converting gallery data THEN the system SHALL preserve all gallery metadata (title, description, settings)
2. WHEN converting memory references THEN the system SHALL create GalleryMemoryEntry objects with position and captions
3. WHEN validating memory references THEN the system SHALL validate every gallery item resolves to an existing memory (id + type) and abort with a list of missing/invalid references; Web2 provides memory list to ICP for presence operations
4. WHEN conversion is complete THEN the system SHALL verify data integrity before storage
5. IF memory references are invalid THEN the system SHALL report specific validation errors with the list of missing memories

### Requirement 5

**User Story:** As a user, I want to view and manage my ICP-stored galleries with visual indicators, so that I can access my permanently stored memories and understand their storage status.

#### Acceptance Criteria

1. WHEN viewing gallery lists THEN the system SHALL display storage status indicators derived from existing gallery_presence views
2. WHEN gallery_presence.icp_complete = true THEN the system SHALL show "Stored Forever" badges on gallery cards and headers
3. WHEN viewing gallery details THEN the system SHALL read from ICP iff icp_complete=true; if partial, prefer Web2, but allow per-memory 'view on ICP' where asset_icp=true
4. WHEN galleries are partially on ICP THEN the system SHALL show "Partially on ICP" badges with unified view
5. WHEN ICP storage fails to load THEN the system SHALL fallback to Web2 data with appropriate warnings using existing error handling

### Requirement 6

**User Story:** As a system administrator, I want comprehensive error handling and rollback capabilities, so that failed storage attempts don't leave the system in inconsistent states.

#### Acceptance Criteria

1. WHEN authentication fails THEN the system SHALL provide specific error messages and retry options
2. WHEN data conversion fails THEN the system SHALL set sync_state='failed' with sync_error and report errors
3. WHEN ICP storage fails THEN the system SHALL set sync_state='failed' and retain diagnostic sync_error; UI shows retry
4. WHEN verification fails THEN the system SHALL mark storage as failed and provide diagnostic information
5. WHEN operations are idempotent THEN all ICP writes must be idempotent by (memoryId, memoryType, artifact, contentHash) using idempotency_key parameter; repeat calls return AlreadyExists

### Requirement 7

**User Story:** As a user, I want the storage process to handle different gallery types and sizes, so that all my galleries can be stored regardless of content.

#### Acceptance Criteria

1. WHEN storing galleries with images THEN the system SHALL handle image memory references correctly
2. WHEN storing galleries with videos THEN the system SHALL handle video memory references correctly
3. WHEN storing galleries with documents THEN the system SHALL handle document memory references correctly
4. WHEN storing large galleries THEN the system SHALL process memories in configurable N-sized batches with back-pressure handling (rate limiting/retry with jitter); after each batch, refresh memory_presence to drive progress
5. WHEN storing empty galleries THEN the system SHALL handle them gracefully with appropriate messaging

### Requirement 8

**User Story:** As a user, I want my stored galleries to be accessible across different devices and sessions, so that my permanently stored memories are always available.

#### Acceptance Criteria

1. WHEN accessing ICP galleries from different devices THEN the system SHALL re-use the same linked II account
2. WHEN the user's session expires THEN the system SHALL use NextAuth re-auth + session.update() to retain the linked principal
3. WHEN switching between Web2 and ICP storage THEN the system SHALL provide seamless user experience
4. WHEN ICP services are temporarily unavailable THEN the system SHALL provide appropriate fallback messaging
5. WHEN galleries are stored on ICP THEN the system SHALL return canonical locator format (e.g., icp://<canister>/<key> + optional gateway URL)

### Requirement 9

**User Story:** As a developer, I want the storage system to integrate seamlessly with existing gallery functionality, so that users have a consistent experience.

#### Acceptance Criteria

1. WHEN integrating with existing gallery service THEN the system SHALL maintain all current gallery operations
2. WHEN adding ICP storage THEN the system SHALL not break existing Web2 gallery functionality
3. WHEN displaying galleries THEN the system SHALL show unified lists regardless of storage location
4. WHEN performing gallery operations THEN the system SHALL read from ICP iff icp_complete=true; if partial, prefer Web2, but allow per-memory 'view on ICP' where asset_icp=true; writes continue to Web2; ICP is a replication target
5. WHEN migrating data THEN the system SHALL maintain referential integrity between memories and galleries

### Requirement 10

**User Story:** As a user, I want clear feedback about storage costs and limitations, so that I can make informed decisions about storing my galleries.

#### Acceptance Criteria

1. WHEN initiating storage THEN the system SHALL display information about ICP storage benefits
2. WHEN storage is in progress THEN the system SHALL show (a) current step, (b) artifacts completed / total, and (c) bytes uploaded sourced from ICP chunk callbacks; no ETA is required
3. WHEN storage completes THEN the system SHALL display storage confirmation with gallery ID
4. WHEN storage fails THEN the system SHALL explain the failure and suggest next steps
5. WHEN viewing stored galleries THEN the system SHALL show storage timestamp and verification status

### Requirement 11

**User Story:** As a system, I want to use database views as the contract for storage status, so that the application has a consistent and reliable way to determine storage state.

#### Acceptance Criteria

1. WHEN determining storage status THEN the system SHALL compute memory_presence = VIEW; gallery_presence = MATERIALIZED VIEW with CONCURRENT refresh after batches or on a schedule
2. WHEN the application needs storage status THEN the system SHALL read from views only for status information
3. WHEN storage edges are updated THEN the system SHALL ensure views reflect current state
4. WHEN gallery_presence.icp_complete = true THEN the gallery "Stored Forever" badge SHALL appear
5. WHEN after successful ICP write of an artifact THEN the system SHALL verify presence via get_memory_presence_icp before UPSERT storage_edges(memory_id,memory_type,artifact,backend='icp-canister') with present=true, sync_state='idle', location, content_hash, and last_synced_at

### Requirement 12

**User Story:** As a system, I want a clear sync state machine for storage operations, so that the system can handle failures and retries reliably.

#### Acceptance Criteria

1. WHEN starting storage THEN edges SHALL use sync_state transition from idle → migrating
2. WHEN storage succeeds THEN sync_state SHALL transition from migrating → idle with present=true and last_synced_at=now()
3. WHEN storage fails THEN sync_state SHALL transition from migrating → failed with sync_error set
4. WHEN retrying failed storage THEN sync_state SHALL transition from failed → migrating; never delete failed edges; only transition failed → migrating on user retry or job resume
5. WHEN on failure THEN the system SHALL set sync_state='failed' and retain diagnostic sync_error; UI shows retry

### Requirement 13

**User Story:** As a user, I want the storage UI to be reliable and not get stuck in infinite loops, so that I can complete the storage process successfully.

#### Acceptance Criteria

1. WHEN displaying the storage modal THEN modal effects MUST not depend on changing callbacks
2. WHEN the modal is open THEN the system SHALL prevent infinite re-renders due to callback dependencies
3. WHEN storage process completes THEN the modal SHALL close cleanly without state inconsistencies
4. WHEN errors occur THEN the modal SHALL handle them without getting stuck in error loops
5. WHEN implementing modal effects THEN the system SHALL include tests ensuring modal effects don't depend on changing function identities (stable deps)

### Requirement 14

**User Story:** As a user logged in with Google, I want to link my Internet Identity without losing my current session, so that I can use "Store Forever" seamlessly.

#### Acceptance Criteria

1. WHEN a user is logged in with Google and starts "Store Forever" THEN the II flow SHALL link an account row (provider='internet-identity', type='oidc') to the same user
2. WHEN linking II account THEN the system SHALL never switch sessions mid-flow
3. WHEN II linking completes THEN the user SHALL remain logged in with their original session
4. WHEN accessing ICP features THEN the system SHALL use the linked II principal while maintaining the original session
5. WHEN the linking process fails THEN the user SHALL remain authenticated; show retry without leaving the flow

### Requirement 15

**User Story:** As a developer, I want to connect the existing UI components to real backend functionality, so that the "Store Forever" feature works end-to-end with actual ICP storage.

#### Acceptance Criteria

1. WHEN storeGalleryForever is called THEN the system SHALL replace the placeholder storeGalleryOnICP function with real backend canister calls
2. WHEN gallery storage succeeds THEN the system SHALL update storage_edges table with present=true, sync_state='idle', and appropriate metadata
3. WHEN displaying gallery storage status THEN the system SHALL query gallery_presence views instead of using mock data
4. WHEN the ForeverStorageProgressModal shows progress THEN the system SHALL display real progress based on actual memory processing and storage operations
5. WHEN checking if galleries are already stored THEN the system SHALL use existing storage_edges and gallery_presence views to determine current storage status

### Requirement 16

**User Story:** As a user, I want the system to detect if my gallery is already stored on ICP, so that I don't accidentally duplicate storage or lose track of what's already preserved.

#### Acceptance Criteria

1. WHEN opening a gallery detail page THEN the system SHALL check gallery_presence.icp_complete to determine if already stored
2. WHEN a gallery is already stored on ICP THEN the "Store Forever" button SHALL show "Already Stored" or "View on ICP" instead
3. WHEN viewing gallery lists THEN galleries already stored SHALL show "Stored Forever" badges based on gallery_presence views
4. WHEN a gallery is partially stored THEN the system SHALL show partial storage status and allow completion
5. WHEN storage is in progress THEN the system SHALL show "Storing..." status based on sync_state in storage_edges; UI hides 'Store Forever' when icp_complete=true and shows 'View on ICP'; for partial, show 'Continue storing'

### Requirement 17

**User Story:** As a system administrator, I want comprehensive security and authorization for ICP operations, so that only authorized users can store data and all operations are auditable.

#### Acceptance Criteria

1. WHEN any ICP write occurs THEN every ICP write must verify caller is authorized (linked II principal or delegated server principal); unauthorized attempts are rejected
2. WHEN ICP operations complete THEN the system SHALL log (userId, principal, galleryId, memoryId, artifact, bytes, outcome, duration) for each ICP write
3. WHEN users attempt unauthorized operations THEN the system SHALL reject with clear authorization error messages
4. WHEN audit logs are needed THEN the system SHALL provide queryable audit trail for all ICP operations
5. WHEN security incidents occur THEN the system SHALL have sufficient logging to investigate and respond

### Requirement 18

**User Story:** As a user, I want data integrity guarantees for my stored memories, so that I can trust my data is accurately preserved on ICP.

#### Acceptance Criteria

1. WHEN uploading assets to ICP THEN the system SHALL compute SHA-256 for assets and verify post-upload on ICP before marking present=true
2. WHEN storing gallery data THEN the system SHALL use lowercase, hyphenated UUID v4 across Web2 and ICP; reject non-canonical forms
3. WHEN ICP storage completes THEN the system SHALL verify content hash equals the recorded hash before setting present=true
4. WHEN data corruption is detected THEN the system SHALL mark storage as failed and require re-upload
5. WHEN calling storeMemoryArtifact THEN the system SHALL ensure idempotency: same (memoryId, memoryType, artifact, contentHash) returns success without duplicate writes

### Requirement 19

**User Story:** As a user, I want the system to remain available even when ICP services have issues, so that I can continue using the application reliably.

#### Acceptance Criteria

1. WHEN ICP endpoints fail N times in a rolling window THEN the system SHALL pause new migrations and surface 'temporarily unavailable' UI; provide resume
2. WHEN ICP services are degraded THEN the system SHALL continue serving from Web2 storage with appropriate status indicators
3. WHEN ICP connectivity is restored THEN the system SHALL automatically resume paused operations
4. WHEN system maintenance occurs THEN the system SHALL provide clear messaging about temporary unavailability
5. WHEN gallery_presence needs updates THEN the system SHALL refresh gallery_presence after each batch of K memories or every T seconds during long runs; manual refresh on job completion

### Requirement 20

**User Story:** As a user, I want efficient and reliable upload performance, so that storing large galleries doesn't overwhelm the system or fail due to resource constraints.

#### Acceptance Criteria

1. WHEN uploading large files THEN the system SHALL use max chunk size S (e.g., 1–2 MB), max concurrent uploads C per user, with exponential backoff on 429/5xx
2. WHEN querying storage status THEN presence queries that list multiple memories/galleries SHALL accept (cursor, limit) for pagination
3. WHEN uploads are rate-limited THEN the system SHALL implement proper backoff and retry mechanisms
4. WHEN system resources are constrained THEN the system SHALL throttle uploads to maintain system stability
5. WHEN batch operations run THEN the system SHALL provide progress feedback and allow cancellation

### Requirement 21

**User Story:** As a user, I want clear visual indicators for partial storage states, so that I understand exactly what's stored where and can take appropriate actions.

#### Acceptance Criteria

1. WHEN galleries have partial ICP presence THEN the system SHALL display per-item badges ('✓ ICP' / 'Web2 only') and a 'Complete storage' CTA
2. WHEN showing storage benefits THEN the system SHALL show benefits and bytes stored; avoid price quotes/ETAs
3. WHEN displaying storage costs THEN the system SHALL provide clear messaging about benefits without specific pricing
4. WHEN users need storage details THEN the system SHALL show which specific memories are stored on ICP vs Web2
5. WHEN partial storage exists THEN the system SHALL provide clear paths to complete the storage process

### Requirement 22

**User Story:** As a developer, I want structured error handling and operational constraints, so that the system is reliable and provides clear feedback for all failure modes.

#### Acceptance Criteria

1. WHEN ICP operations fail THEN the system SHALL use Result<T, ErrorCode> with structured errors (Unauthorized | AlreadyExists | NotFound | InvalidHash | UploadExpired | InsufficientCycles | QuotaExceeded | InvalidChunkSize | Internal)
2. WHEN upload operations occur THEN the system SHALL enforce max chunk size (2MB), max upload size (100MB), max concurrent uploads (3 per user), and upload session timeout (1 hour)
3. WHEN users exceed quotas THEN the system SHALL enforce max uploads per day (1000) and max total bytes per user (10GB) with clear QuotaExceeded errors
4. WHEN querying presence THEN the system SHALL support pagination with (cursor, limit) parameters and enforce max limit (100) with default (20)
5. WHEN operations use idempotency THEN all artifact writes SHALL use idempotency_key parameter format: <memoryId>:<artifact>:<contentHash>

### Requirement 23

**User Story:** As a system administrator, I want canister state to survive upgrades and maintain data integrity, so that user data is never lost during system maintenance.

#### Acceptance Criteria

1. WHEN canister upgrades occur THEN the system SHALL use ic-stable-structures for persistent storage that survives upgrades
2. WHEN pre-upgrade hooks run THEN the system SHALL ensure all state is persisted to stable memory automatically
3. WHEN post-upgrade hooks run THEN the system SHALL restore all state from stable memory without data loss
4. WHEN testing upgrades THEN the system SHALL include tests that verify state persistence across upgrade cycles
5. WHEN managing memory THEN the system SHALL use StableBTreeMap for capsules, upload sessions, and other critical state

### Requirement 24

**User Story:** As a developer, I want standardized time and data formats, so that the system maintains consistency across all operations and integrations.

#### Acceptance Criteria

1. WHEN handling timestamps THEN the system SHALL use nanoseconds since Unix epoch for all time fields (created_at_ns, updated_at_ns)
2. WHEN processing UUIDs THEN the system SHALL use lowercase, hyphenated UUID v4 format and reject non-canonical forms
3. WHEN naming API endpoints THEN the system SHALL use consistent naming: upsert_metadata, begin_asset_upload, put_chunk, commit_asset, get_memory_presence_icp, get_memory_list_presence_icp
4. WHEN handling content hashes THEN the system SHALL use SHA-256 format and verify integrity before marking artifacts as present
5. WHEN processing memory types THEN the system SHALL support standardized types: Image, Video, Audio, Document, Note

### Requirement 25

**User Story:** As a user, I want the system to track ICP-only presence without duplicating Web2 membership data, so that the architecture remains clean and scalable.

#### Acceptance Criteria

1. WHEN determining gallery membership THEN Web2 SHALL remain the single source of truth for which memories belong to which galleries
2. WHEN calling ICP presence operations THEN Web2 SHALL provide the memory list to ICP; ICP does not store gallery membership
3. WHEN querying storage status THEN ICP SHALL only report presence of individual memories, not gallery-level aggregations
4. WHEN aggregating gallery status THEN Web2 SHALL compute gallery_presence via views that combine ICP presence with Web2 membership
5. WHEN storing memories THEN ICP SHALL track only artifact presence (metadata, asset) per memory, not gallery relationships

### Requirement 26

**User Story:** As a system architect, I want Web2 and ICP to share canonical UUIDs for galleries and memories, so that data can be consistently referenced across both systems without ID mapping complexity.

#### Acceptance Criteria

1. WHEN Web2 creates galleries or memories THEN the system SHALL generate canonical lowercase hyphenated UUID v4 identifiers that will be used across both Web2 and ICP
2. WHEN ICP receives gallery or memory data THEN the system SHALL accept and preserve the UUID provided by Web2 without generating new IDs
3. WHEN ICP stores galleries THEN the system SHALL use the gallery.id provided in GalleryData without overwriting it with generated IDs
4. WHEN ICP stores memories THEN the system SHALL accept memory_id as a parameter and use it as the canonical identifier without generating timestamp-based IDs
5. WHEN the same UUID is used across systems THEN operations SHALL be idempotent, returning success for existing entities rather than creating duplicates
