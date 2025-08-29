# Requirements Document

## Introduction

This feature implements capsule migration functionality that allows users to create their own personal canister containing all their capsule data. Users will be able to migrate from the shared backend canister to their own dedicated canister instance, taking all their memories, connections, and capsule information with them. This provides users with full ownership and control over their data while maintaining the same capsule functionality in their personal canister.

## Requirements

### Requirement 1

**User Story:** As a capsule owner, I want to migrate my capsule data to a new personal canister, so that I can have full ownership and control over my memories and connections.

#### Acceptance Criteria

1. WHEN migration starts THEN controllers SHALL be set to {factory, user} and after verification switch to {user} only
2. WHEN new canister is created THEN the system SHALL use a single personal-canister WASM with no templates or variants
3. WHEN cycles are needed THEN the system SHALL use admin-funded cycles from factory reserve and fail early with clear error if insufficient
4. WHEN data is transferred THEN the system SHALL use internal export/import for capsule metadata, memory metadata + blobs, and connections only
5. WHEN migration progresses THEN state SHALL follow: NotStarted → Exporting → Creating → Installing → Importing → Verifying → Completed/Failed
6. WHEN migrate_capsule is called repeatedly THEN the system SHALL return existing status/result idempotently
7. WHEN migration fails THEN original data SHALL remain unchanged and new canister SHALL be left for retry/cleanup

### Requirement 2

**User Story:** As an administrator, I want to control capsule migration with admin-funded cycles, so that migrations can proceed without complex payment processing.

#### Acceptance Criteria

1. WHEN migration is requested THEN factory SHALL check internal cycles reserve against threshold
2. WHEN reserve is sufficient THEN factory SHALL consume cycles from reserve to fund new canister
3. WHEN reserve is insufficient THEN migration SHALL fail early with clear error message
4. WHEN admin manages reserve THEN they SHALL be able to monitor and top up factory cycles
5. WHEN migration succeeds THEN consumed cycles SHALL be deducted from factory reserve
6. WHEN multiple migrations occur THEN factory SHALL track total cycles consumed
7. WHEN reserve runs low THEN admin SHALL receive notifications to replenish cycles

### Requirement 3

**User Story:** As a capsule owner, I want minimal frontend integration for migration, so that I can trigger migration and monitor its progress.

#### Acceptance Criteria

1. WHEN frontend calls migrate_capsule THEN the system SHALL initiate migration and return status
2. WHEN frontend polls get_migration_status THEN the system SHALL return current migration state
3. WHEN migration succeeds THEN frontend SHALL use returned personal canister ID for user's reads/writes
4. WHEN no personal canister exists THEN frontend SHALL fallback to shared canister
5. WHEN migration is in progress THEN frontend SHALL display appropriate status to user
6. WHEN migration fails THEN frontend SHALL show error message and allow retry
7. WHEN user has already migrated THEN repeated calls SHALL return existing canister information

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

**User Story:** As a system administrator, I want essential migration controls, so that I can manage the system without over-engineering.

#### Acceptance Criteria

1. WHEN admin needs control THEN migration enable/disable toggle SHALL be available
2. WHEN tracking is needed THEN basic success/failure counters SHALL be maintained
3. WHEN migration is disabled THEN all migration requests SHALL be rejected with clear message
4. WHEN statistics are requested THEN system SHALL provide total migrations attempted and succeeded
5. WHEN admin monitors system THEN they SHALL see current migration states and any failures
6. WHEN integration is required THEN shared backend behavior SHALL remain completely intact
7. WHEN new functions are added THEN existing capsule functionality SHALL be unaffected

### Requirement 6

**User Story:** As a platform engineer, I want consistent factory conventions for migration, so that the MVP is reliable, secure, and future-proof.

#### Acceptance Criteria

1. WHEN a personal canister is created THEN the system SHALL persist a registry entry containing: canister_id, created_by, created_at, status, cycles_consumed
2. WHEN migration endpoints are invoked THEN the caller principal SHALL be the capsule owner; admin-only functions SHALL guard reserve management and toggles
3. WHEN creating and installing a personal canister THEN the factory cycles reserve SHALL be checked against a threshold before proceeding and aggregate cycles consumed SHALL be tracked
4. WHEN receiving creation input THEN the system SHALL accept a minimal CreatePersonalCanisterConfig (optional name, optional subnet) and ignore non-MVP options without error
5. WHEN controller handoff occurs THEN it SHALL follow the sequence defined in Requirement 1

### Security Note

**Encryption Compatibility:** If encryption is in use, personal canister must be able to decrypt the same content. VetKeys note: migration should carry/derive the needed key material or allow key re-wrapping for the user. Full VetKeys flow is out of scope for MVP.
