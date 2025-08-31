# Implementation Plan

- [x] 1. Set up migration module structure and basic types

  - Create `src/backend/src/canister_factory.rs` module file
  - Define core types: `MigrationResponse`, `MigrationStatus`, `MigrationStatusResponse`, `ExportData`, `ExportMetadata`
  - Add module import to `src/backend/src/lib.rs`
  - _Requirements: 1.5, 5.1_

- [x] 2. Implement migration state management and registry

  - [x] 2.1 Create migration state storage structures

    - Define `MigrationState`, `MigrationConfig`, and `PersonalCanisterRecord` structs
    - Extend existing `State` struct with migration fields and personal canisters registry
    - Implement default values and initialization
    - _Requirements: 1.5, 5.4, 6.1_

  - [x] 2.2 Add migration state persistence to upgrade hooks

    - Update `pre_upgrade` function to include migration state and registry
    - Update `post_upgrade` function to restore migration state and registry
    - Test state preservation across canister upgrades
    - _Requirements: 5.6_

  - [x] 2.3 Implement personal canister registry management
    - Create functions to persist registry entries with canister_id, created_by, created_at, status, cycles_consumed
    - Add registry update functions for status transitions
    - Implement registry query functions for admin monitoring
    - Add admin query to fetch registry entries by user principal and by status for ops
    - _Requirements: 6.1_

- [x] 3. Implement cycles reserve management

  - [x] 3.1 Create cycles reserve checking functions

    - Implement `preflight_cycles_reserve` function for threshold checking
    - Add `consume_cycles_from_reserve` function
    - Create admin functions for reserve management and monitoring
    - _Requirements: 2.1, 2.2, 2.3, 6.3_

  - [x] 3.2 Add cycles reserve monitoring and alerts
    - Implement reserve threshold checking
    - Add logging for cycles consumption
    - Create admin notification system for low reserves
    - _Requirements: 2.4, 2.7_

- [x] 4. Create data export functionality

  - [x] 4.1 Implement capsule data serialization

    - Create `export_user_capsule_data` function
    - Serialize capsule metadata, memories, and connections
    - Generate export metadata with timestamps and checksums
    - _Requirements: 1.2, 1.4_

  - [x] 4.2 Add data validation and integrity checks
    - Implement data completeness validation
    - Add checksum generation for exported data
    - Create manifest generation for verification
    - _Requirements: 1.4, 4.7_

- [x] 5. Implement access control and guards

  - [x] 5.1 Create access control functions

    - Implement `ensure_owner` function to verify caller owns capsule
    - Add `ensure_admin` function for admin-only operations
    - Create caller validation for migration endpoints
    - _Requirements: 6.2_

- [x] 6. Implement canister creation and WASM installation

  - [x] 6.1 Create personal canister with dual controllers

    - Implement canister creation with {factory, user} controllers
    - Add cycles funding from factory reserve with preflight check using with_cycles() on management calls
    - Handle creation failures and cleanup
    - Persist registry entry with Creating status
    - _Requirements: 1.1, 2.5, 6.1, 6.3_

  - [x] 6.2 Install personal canister WASM module

    - Load single personal-canister WASM binary
    - Install WASM with proper initialization
    - Handle installation failures and error reporting
    - Add API_VERSION compatibility check pre-import and fail fast if incompatible
    - _Requirements: 1.2, 4.1, 4.7_

  - [x] 6.3 Add minimal creation configuration support
    - Implement `CreatePersonalCanisterConfig` with optional name and subnet_id
    - Accept minimal config input and ignore non-MVP options without error
    - Add configuration validation and defaults
    - _Requirements: 6.4_

- [x] 7. Create internal data transfer system

  - [x] 7.1 Implement chunked data import API

    - Create `begin_import`, `put_memory_chunk`, `commit_memory`, `finalize_import` functions
    - Add session management for import operations
    - Implement chunk validation and assembly with max chunk size and total import size guards via config
    - Reject oversize chunks with clear error messages
    - _Requirements: 1.4, 4.2_

  - [x] 7.2 Add data transfer verification
    - Implement hash-based verification of transferred data
    - Add count reconciliation between source and target
    - Create verification failure handling and cleanup
    - _Requirements: 1.5, 4.7_

- [x] 8. Implement controller handoff mechanism

  - [x] 8.1 Create controller transition logic

    - Implement `handoff_controllers` function
    - Switch controllers from {factory, user} to {user} only
    - Add verification before handoff
    - _Requirements: 1.1, 4.7, 6.5_

  - [x] 8.2 Add handoff failure handling and registry finalization
    - Implement rollback for failed handoffs
    - Add retry logic for controller updates
    - Create cleanup procedures for failed migrations
    - Update registry status to Completed and record cycles consumed
    - _Requirements: 1.7, 5.6, 6.1_

- [x] 9. Create main migration orchestration

  - [x] 9.1 Implement `migrate_capsule` function

    - Create state machine progression: NotStarted → Exporting → Creating → Installing → Importing → Verifying → Completed/Failed
    - Add idempotency for repeated calls
    - Implement comprehensive error handling
    - Add access control validation using ensure_owner
    - _Requirements: 1.5, 1.6, 6.2_

  - [x] 9.2 Add migration status tracking
    - Implement `get_migration_status` function
    - Add progress reporting and error messages
    - Create status persistence across canister restarts
    - Add `get_personal_canister_id(user)` query to simplify frontend fallback logic
    - _Requirements: 3.2, 5.5_

- [x] 10. Implement admin controls and monitoring

  - [x] 10.1 Create migration enable/disable functionality

    - Add `set_migration_enabled` admin function with ensure_admin guard
    - Implement migration request rejection when disabled
    - Add admin authentication checks
    - _Requirements: 5.1, 5.3, 6.2_

  - [x] 10.2 Add basic migration statistics
    - Implement success/failure counters
    - Create `get_migration_stats` function
    - Add migration attempt tracking
    - _Requirements: 5.2, 5.4_

- [x] 11. Add Candid interface integration

  - [x] 11.1 Export migration functions in Candid interface

    - Add `migrate_capsule` and `get_migration_status` to service definition
    - Update `backend.did` file with new types and functions
    - Test Candid interface generation
    - _Requirements: 3.1, 5.7_

  - [x] 11.2 Ensure API compatibility
    - Add API_VERSION constant to personal canister
    - Implement compatibility checking during migration
    - Add version mismatch error handling
    - _Requirements: 4.7_

- [x] 12. Create comprehensive error handling

  - [x] 12.1 Define migration-specific error types

    - Create typed MigrationError enum (ReserveInsufficient, CreateFailed, InstallFailed, ImportFailed, VerifyFailed, HandoffFailed, Disabled, Unauthorized)
    - Add error context and debugging information
    - Implement error recovery strategies
    - Use error enum consistently across all migration functions
    - _Requirements: 1.6, 1.7_

  - [x] 12.2 Add error logging and monitoring
    - Implement error logging for debugging
    - Add error rate tracking
    - Create error notification system for admins
    - _Requirements: 2.4, 5.5_

- [x] 13. Refactor canister_factory into maintainable modules

  - [x] 13.1 Create module structure and extract types

    - Create `src/backend/src/canister_factory/` directory
    - Extract all types, enums, configs to `types.rs`
    - Create `mod.rs` with public facade
    - Update imports and ensure compilation
    - _Requirements: 6.2 (maintainability)_

  - [x] 13.2 Extract state management modules

    - Extract registry functions to `registry.rs` (PersonalCanisterRecord CRUD)
    - Extract cycles management to `cycles.rs` (reserve, preflight, consume)
    - Extract auth functions to `auth.rs` (ensure_owner, ensure_admin)
    - Update imports and ensure compilation
    - _Requirements: 1.6, 4.7_

  - [x] 13.3 Extract core functionality modules

    - Extract factory functions to `factory.rs` (create canister, install WASM, handoff)
    - Extract export functions to `export.rs` (export data, validation, manifest)
    - Extract import functions to `import.rs` (sessions, chunks, assembly)
    - Update imports and ensure compilation
    - _Requirements: 2.1, 2.2, 5.6_

  - [x] 13.4 Extract verification and orchestration
    - Extract verification functions to `verify.rs` (data verification, health checks)
    - Extract orchestration to `orchestrator.rs` (migrate_capsule state machine)
    - Update imports and ensure compilation
    - _Requirements: 6.1, 6.3_

- [ ] 14. Write comprehensive unit tests for refactored modules

  - [x] 14.1 Test auth and access control

    - Test `ensure_owner` and `ensure_admin` functions
    - Test caller validation and authorization roles
    - Test access control guards with various scenarios
    - _Requirements: 1.6, 4.7_

  - [x] 14.2 Test cycles and registry management

    - Test cycles reserve preflight and consumption
    - Test registry CRUD operations
    - Test registry queries by user and status
    - Test cycles threshold monitoring and alerts
    - _Requirements: 4.7, 6.2_

  - [x] 14.3 Test data export and validation

    - Test capsule data export functionality
    - Test export data validation and integrity checks
    - Test manifest generation and verification
    - _Requirements: 2.1, 2.2_

  - [x] 14.4 Test import session management

    - Test import session creation and lifecycle
    - Test chunk upload and assembly
    - Test memory commit and finalization
    - Test session cleanup and error handling
    - _Requirements: 2.1, 2.2_

  - [x] 14.5 Test factory operations

    - Test personal canister creation
    - Test WASM installation and configuration
    - Test controller handoff logic
    - Test cleanup on failure scenarios
    - _Requirements: 5.6, 6.1_

  - [x] 14.6 Test verification and health checks
    - Test data verification against manifests
    - Test API compatibility checks
    - Test canister health verification
    - Test comprehensive verification flow
    - _Requirements: 6.1, 6.3_

- [x] 15. Write integration tests for complete migration flow

  - [x] 15.1 Test end-to-end migration scenarios

    - Test complete successful migration flow
    - Test idempotent `migrate_capsule` behavior
    - Test migration status tracking and updates
    - _Requirements: 2.1, 2.2, 5.6, 6.1, 6.3_

  - [x] 15.2 Test failure scenarios and recovery

    - Test failure at each migration stage
    - Test cleanup and rollback procedures
    - Test error logging and monitoring
    - Test retry mechanisms and recovery strategies
    - _Requirements: 6.1, 6.3_

  - [x] 15.3 Test upgrade resilience
    - Test restart-resume functionality (simulate mid-state)
    - Test pre/post-upgrade state persistence
    - Test idempotency across canister upgrades
    - Test migration state recovery after restart
    - _Requirements: 6.2, 6.3_

- [x] 16. Update dependencies and build configuration

  - [x] 16.1 Add required dependencies to Cargo.toml

    - Add `sha2` for hash generation
    - Add `hex` for hash encoding
    - Update existing dependencies if needed
    - _Requirements: 4.2, 4.7_

  - [x] 16.2 Update build and deployment scripts
    - Ensure migration module compiles correctly
    - Add feature flag support for migration functionality
    - Test deployment with migration features
    - _Requirements: 5.1, 5.7_

- [-] 17. Refactor terminology from "migration" to "personal canister creation"

  - [x] 17.1 Update API endpoint names and function signatures

    - Rename `migrate_capsule()` to `create_personal_canister()`
    - Rename `get_migration_status()` to `get_creation_status()`
    - Rename `get_detailed_migration_status()` to `get_detailed_creation_status()`
    - Rename `set_migration_enabled()` to `set_personal_canister_creation_enabled()`
    - Rename `get_migration_stats()` to `get_personal_canister_creation_stats()`
    - _Requirements: 3.1, 3.2, 5.1, 5.4_

  - [-] 17.2 Update type names and data structures

    - Rename `MigrationResponse` to `PersonalCanisterCreationResponse`
    - Rename `MigrationStatus` to `CreationStatus`
    - Rename `MigrationStatusResponse` to `CreationStatusResponse`
    - Rename `MigrationState` to `PersonalCanisterCreationState`
    - Rename `MigrationConfig` to `PersonalCanisterCreationConfig`
    - Rename `MigrationStats` to `PersonalCanisterCreationStats`
    - _Requirements: 1.5, 5.2_

  - [x] 17.3 Update internal function and variable names

    - Update all internal function names to use "creation" terminology
    - Update variable names and comments throughout codebase
    - Update error messages and user-facing strings
    - Update logging and debugging messages
    - _Requirements: 1.6, 5.3_

  - [x] 17.4 Update Candid interface and documentation

    - Update `backend.did` file with new endpoint names and types
    - Update function documentation and comments
    - Update API documentation and examples
    - Ensure backward compatibility during transition
    - _Requirements: 3.1, 4.7_

  - [x] 17.5 Update test names and descriptions

    - Rename all test functions to use "creation" terminology
    - Update test descriptions and comments
    - Update mock function names and test data
    - Ensure all tests pass after renaming
    - _Requirements: 7.1, 7.2_

  - [x] 17.6 Update configuration and feature flags

    - Update feature flag names from "migration" to "personal_canister_creation"
    - Update configuration variable names
    - Update deployment script terminology
    - Update environment variable names
    - _Requirements: 5.1, 7.10_

- [ ] 18. Create comprehensive bash integration test suite

  - [ ] 17.1 Set up test infrastructure and utilities

    - Create `scripts/test-migration/` directory structure
    - Implement test utilities for dfx calls, JSON parsing, and assertions
    - Create test data setup and cleanup functions
    - Add test configuration and environment management
    - _Requirements: 7.8, 7.9_

  - [ ] 18.2 Test personal canister creation API endpoints individually

    - Test `create_personal_canister` endpoint with various user scenarios
    - Test `get_creation_status` and `get_detailed_creation_status` queries
    - Test `get_personal_canister_id` and `get_my_personal_canister_id` functions
    - Test admin endpoints: `set_personal_canister_creation_enabled`, `get_personal_canister_creation_stats`
    - _Requirements: 7.2, 7.4_

  - [ ] 18.3 Test personal canister creation state transitions

    - Test each creation state individually: NotStarted → Exporting → Creating → Installing → Importing → Verifying → Completed
    - Test state persistence across canister restarts
    - Test idempotent behavior at each state
    - Test error state transitions and recovery
    - _Requirements: 7.1, 7.6_

  - [ ] 17.4 Test error conditions and edge cases

    - Test insufficient cycles scenarios
    - Test invalid user permissions and unauthorized access
    - Test network failures and timeout scenarios
    - Test partial data corruption and recovery
    - Test concurrent migration attempts
    - _Requirements: 7.3, 7.7_

  - [ ] 17.5 Test data integrity and verification

    - Test complete data export and import cycle
    - Verify exported data matches original capsule data
    - Test hash verification and checksum validation
    - Test memory content integrity after migration
    - Test connection data preservation
    - _Requirements: 7.5_

  - [ ] 18.6 Test admin functionality and monitoring

    - Test personal canister creation enable/disable toggle functionality
    - Test cycles reserve management and monitoring
    - Test creation statistics and reporting
    - Test admin-only access controls
    - Test registry queries and user lookup functions
    - _Requirements: 7.4_

  - [ ] 18.7 Test feature flag functionality

    - Test building and deploying with personal canister creation features enabled
    - Test building and deploying with personal canister creation features disabled
    - Test API availability based on feature flag configuration
    - Test upgrade scenarios with different feature flag states
    - _Requirements: 7.10_

  - [ ] 17.8 Create test orchestration and reporting

    - Create master test runner script that executes all test suites
    - Implement parallel test execution for independent tests
    - Add detailed test reporting with pass/fail status and timing
    - Create CI/CD integration scripts for automated testing
    - Add test coverage reporting and gap analysis
    - _Requirements: 7.8, 7.9_
