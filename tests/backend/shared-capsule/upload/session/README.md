# Session Management Tests

This directory contains tests specifically for session management functionality in the upload system.

## Test Files

### Core Session Tests

- **`test_session_collision.mjs`** - Tests parallel upload session isolation

  - Validates that multiple concurrent uploads don't interfere with each other
  - Tests unique session IDs prevent collisions
  - Status: Part of concurrency testing suite

- **`test_session_debug.mjs`** - Debug test for session lifecycle

  - Tests session creation, chunk upload, and finalization
  - Useful for debugging session-related issues
  - Status: Debugging/development tool

- **`test_session_isolation.mjs`** - Tests session isolation between callers

  - Validates that sessions from different callers are isolated
  - Tests that unique session IDs work correctly
  - Status: Part of session management validation

- **`test_session_persistence.mjs`** - Tests session persistence across calls
  - Validates that session state persists between canister calls
  - Tests session cleanup and expiration
  - Status: Session lifecycle testing

### Asset Tests

- **`test_asset_retrieval_debug.mjs`** - Debug test for asset retrieval after upload
  - Tests that uploaded assets can be retrieved immediately
  - Validates index update atomicity
  - Status: Part of E2E validation (currently failing in parallel mode)

## Related Documentation

### Architecture Documents

- `/docs/issues/open/backend-session-architecture-reorganization.md` - Session architecture redesign
- `/docs/issues/open/backend-session-concurrency-mvp.md` - Concurrency issues and solutions
- `/docs/issues/open/backend-session-file-organization.md` - File structure proposals

### Implementation Files

- `/src/backend/src/session/service.rs` - Generic SessionService (17/17 tests passing)
- `/src/backend/src/session/compat.rs` - SessionCompat compatibility layer
- `/src/backend/src/session/adapter.rs` - IC-specific adapter
- `/src/backend/src/upload/service.rs` - Upload service using sessions

### Current Issues

- `/docs/issues/open/compatibility-layer-e2e-test-failures.md` - **ACTIVE** E2E test failures
- `/docs/issues/open/compatibility-layer-test-status.md` - Test status tracking

## Running Tests

### Individual Tests

```bash
cd /Users/stefano/Documents/Code/Futura/futura_alpha_icp/tests/backend/shared-capsule/upload/session

# Get backend canister ID
BACKEND_CANISTER_ID=$(dfx canister id backend)

# Run specific test
node test_session_isolation.mjs $BACKEND_CANISTER_ID
node test_session_collision.mjs $BACKEND_CANISTER_ID
node test_session_debug.mjs $BACKEND_CANISTER_ID
```

### Prerequisites

1. **dfx running**: `dfx start --background`
2. **Backend deployed**: `./scripts/deploy-local.sh`
3. **Node.js installed**: Tests use ES modules

## Test Status

| Test                             | Status     | Notes                               |
| -------------------------------- | ---------- | ----------------------------------- |
| `test_session_collision.mjs`     | ⚠️ Unknown | Needs re-run with new architecture  |
| `test_session_debug.mjs`         | ⚠️ Unknown | Debugging tool, not automated       |
| `test_session_isolation.mjs`     | ⚠️ Unknown | Needs validation with SessionCompat |
| `test_session_persistence.mjs`   | ⚠️ Unknown | Needs validation with SessionCompat |
| `test_asset_retrieval_debug.mjs` | ❌ Failing | Part of E2E failures (see issue)    |

## Known Issues

### 1. Parallel Upload Failures (Active)

- **Issue**: Parallel uploads fail while sequential uploads work
- **Impact**: 60% E2E test failure rate
- **Status**: Blocked, awaiting tech lead guidance
- **Details**: See `/docs/issues/open/compatibility-layer-e2e-test-failures.md`

### 2. SessionCompat Unit Tests (IC-Dependent)

- **Issue**: 12 SessionCompat unit tests require IC runtime
- **Impact**: Can't run in `cargo test --lib`
- **Status**: Normal, will pass in canister environment
- **Details**: Tests use `ic_cdk::api::time()` which requires IC runtime

## Test Helpers

Session tests use shared helpers from parent directory:

- `../helpers.mjs` - Upload utilities, progress tracking, formatting
- `../ic-identity.js` - Identity management for local/mainnet testing

## Debugging Session Issues

### Enable Backend Logging

```bash
# View canister logs
dfx canister logs backend

# Filter for session-related logs
dfx canister logs backend | grep -E "(SESSION|UPLOAD|FINISH)"
```

### Add Debug Logging

In backend code, add:

```rust
ic_cdk::println!("SESSION_DEBUG: sid={}, status={:?}", session_id, status);
```

### Monitor Session State

Use admin functions:

```bash
# List all sessions
dfx canister call backend sessions_list

# Get session stats
dfx canister call backend sessions_stats
```

## Contributing

When adding new session tests:

1. **Naming**: Use `test_session_*.mjs` pattern
2. **Location**: Place in this `session/` directory
3. **Documentation**: Update this README with test description
4. **Helpers**: Reuse helpers from `../helpers.mjs`
5. **Error Handling**: Include detailed error messages for debugging

## Architecture Notes

### Current Implementation

The session system uses a **hybrid architecture**:

- **Generic Layer**: `session::service::SessionService` - Pure Rust, no upload semantics
- **Compatibility Layer**: `session::compat::SessionCompat` - Bridges old/new APIs
- **Upload Layer**: `upload::service::UploadService` - Upload-specific business logic

### Key Principles

1. **No Heap Buffering**: Chunks write directly to storage (ByteSink)
2. **Session Isolation**: Sessions don't interfere with each other
3. **Idempotency**: Duplicate `begin()` calls return same session
4. **TTL Cleanup**: Expired sessions cleaned up automatically

---

**Last Updated**: 2025-10-01  
**Status**: Active development - compatibility layer E2E testing phase
