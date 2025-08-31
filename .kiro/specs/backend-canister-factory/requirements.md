# Requirements Document

## Introduction

This feature implements personal canister creation functionality that allows users to create their own dedicated canister containing all their capsule data. Users will be able to create a personal canister instance with all their memories, connections, and capsule information transferred from the shared backend canister. This provides users with full ownership and control over their data while maintaining the same capsule functionality in their personal canister.

## Requirements

### Requirement 1

**User Story:** As a capsule owner, I want to create my own personal canister with all my capsule data, so that I can have full ownership and control over my memories and connections.

#### Acceptance Criteria

1. WHEN personal canister creation starts THEN controllers SHALL be set to {factory, user} and after verification switch to {user} only
2. WHEN new canister is created THEN the system SHALL use a single personal-canister WASM with no templates or variants
3. WHEN cycles are needed THEN the system SHALL use admin-funded cycles from factory reserve and fail early with clear error if insufficient
4. WHEN data is transferred THEN the system SHALL use internal export/import for capsule metadata, memory metadata + blobs, and connections only
5. WHEN creation progresses THEN state SHALL follow: NotStarted → Exporting → Creating → Installing → Importing → Verifying → Completed/Failed
6. WHEN create_personal_canister is called repeatedly THEN the system SHALL return existing status/result idempotently
7. WHEN creation fails THEN original data SHALL remain unchanged and new canister SHALL be left for retry/cleanup

### Requirement 2

**User Story:** As an administrator, I want to control personal canister creation with admin-funded cycles, so that canister creation can proceed without complex payment processing.

#### Acceptance Criteria

1. WHEN personal canister creation is requested THEN factory SHALL check internal cycles reserve against threshold
2. WHEN reserve is sufficient THEN factory SHALL consume cycles from reserve to fund new canister
3. WHEN reserve is insufficient THEN creation SHALL fail early with clear error message
4. WHEN admin manages reserve THEN they SHALL be able to monitor and top up factory cycles
5. WHEN creation succeeds THEN consumed cycles SHALL be deducted from factory reserve
6. WHEN multiple creations occur THEN factory SHALL track total cycles consumed
7. WHEN reserve runs low THEN admin SHALL receive notifications to replenish cycles

### Requirement 3

**User Story:** As a capsule owner, I want minimal frontend integration for personal canister creation, so that I can trigger creation and monitor its progress.

#### Acceptance Criteria

1. WHEN frontend calls create_personal_canister THEN the system SHALL initiate creation and return status
2. WHEN frontend polls get_creation_status THEN the system SHALL return current creation state
3. WHEN creation succeeds THEN frontend SHALL use returned personal canister ID for user's reads/writes
4. WHEN no personal canister exists THEN frontend SHALL fallback to shared canister
5. WHEN creation is in progress THEN frontend SHALL display appropriate status to user
6. WHEN creation fails THEN frontend SHALL show error message and allow retry
7. WHEN user already has a personal canister THEN repeated calls SHALL return existing canister information

### Requirement 4

**User Story:** As a capsule owner, I want my personal canister to provide the same core functionality as the shared backend, so that I can access my memories and connections seamlessly.

#### Acceptance Criteria

1. WHEN personal canister is created THEN it SHALL provide the same API endpoints for memory and connection management
2. WHEN user accesses personal canister THEN all their memories SHALL be available and functional
3. WHEN user queries data THEN the personal canister SHALL respond with the same data structures as the shared backend
4. WHEN user adds new memories THEN the personal canister SHALL store them with the same functionality
5. WHEN user manages connections THEN the personal canister SHALL handle relationships identically
6. WHEN frontend connects to personal canister THEN it SHALL work without modification
7. WHEN personal canister is deployed THEN it SHALL expose API_VERSION for compatibility check and migration SHALL fail if incompatible

### Requirement 5

**User Story:** As a system administrator, I want essential personal canister creation controls, so that I can manage the system without over-engineering.

#### Acceptance Criteria

1. WHEN admin needs control THEN personal canister creation enable/disable toggle SHALL be available
2. WHEN tracking is needed THEN basic success/failure counters SHALL be maintained
3. WHEN creation is disabled THEN all creation requests SHALL be rejected with clear message
4. WHEN statistics are requested THEN system SHALL provide total creations attempted and succeeded
5. WHEN admin monitors system THEN they SHALL see current creation states and any failures
6. WHEN integration is required THEN shared backend behavior SHALL remain completely intact
7. WHEN new functions are added THEN existing capsule functionality SHALL be unaffected

### Requirement 6

**User Story:** As a platform engineer, I want consistent factory conventions for personal canister creation, so that the MVP is reliable, secure, and future-proof.

#### Acceptance Criteria

1. WHEN a personal canister is created THEN the system SHALL persist a registry entry containing: canister_id, created_by, created_at, status, cycles_consumed
2. WHEN creation endpoints are invoked THEN the caller principal SHALL be the capsule owner; admin-only functions SHALL guard reserve management and toggles
3. WHEN creating and installing a personal canister THEN the factory cycles reserve SHALL be checked against a threshold before proceeding and aggregate cycles consumed SHALL be tracked
4. WHEN receiving creation input THEN the system SHALL accept a minimal CreatePersonalCanisterConfig (optional name, optional subnet) and ignore non-MVP options without error
5. WHEN controller handoff occurs THEN it SHALL follow the sequence defined in Requirement 1

### Requirement 7

**User Story:** As a developer and system administrator, I want comprehensive integration tests for all personal canister creation functionality, so that I can verify each component works correctly and catch regressions early.

#### Acceptance Criteria

1. WHEN testing creation flow THEN bash scripts SHALL test each creation state transition individually
2. WHEN testing API endpoints THEN scripts SHALL verify all creation endpoints return correct responses and status codes
3. WHEN testing error conditions THEN scripts SHALL verify proper error handling for insufficient cycles, invalid states, and network failures
4. WHEN testing admin functions THEN scripts SHALL verify enable/disable toggles, statistics reporting, and cycles management
5. WHEN testing data integrity THEN scripts SHALL verify exported data matches imported data after creation
6. WHEN testing idempotency THEN scripts SHALL verify repeated calls return consistent results
7. WHEN testing edge cases THEN scripts SHALL cover partial failures, timeouts, and recovery scenarios
8. WHEN running tests THEN each test SHALL be independent and able to run in isolation
9. WHEN tests complete THEN they SHALL provide clear pass/fail status and detailed error messages
10. WHEN testing feature flags THEN scripts SHALL verify personal canister creation functionality can be enabled/disabled at build time

### Security Note

**Encryption Compatibility:** If encryption is in use, personal canister must be able to decrypt the same content. VetKeys note: migration should carry/derive the needed key material or allow key re-wrapping for the user. Full VetKeys flow is out of scope for MVP.
