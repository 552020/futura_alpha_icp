# Authorization Tests TODO

## Overview

This document outlines the comprehensive authorization testing strategy for the Futura ICP canister. Currently, authorization tests only cover 2 endpoints (`capsules_create`, `capsules_list`) but we need to test authorization for ALL endpoints to ensure system security.

## Current Status

### ✅ What's Working

- **Capsule endpoints**: `capsules_create`, `capsules_list` have basic auth tests
- **Auth logic unit tests**: Comprehensive tests for `ensure_owner()`, `ensure_admin()` functions
- **Test infrastructure**: Unit vs E2E test separation, mainnet/local support

### ❌ Critical Gaps

- **Memory management**: 5 endpoints with NO authorization tests
- **Gallery management**: 7 endpoints with NO authorization tests
- **Admin functions**: 4 endpoints with NO authorization tests
- **User/auth functions**: 2 endpoints with NO authorization tests
- **Upload functions**: Multiple endpoints with NO authorization tests

## Priority 1: Memory Management Authorization Tests

### Endpoints to Test

- `memories_create(capsule_id, bytes, blob_ref, external_location, external_storage_key, external_url, external_size, external_hash, asset_metadata, idem)`
- `memories_read(memory_id)`
- `memories_update(memory_id, updates)`
- `memories_delete(memory_id)`
- `memories_list(capsule_id)`
- `memories_ping(memory_ids)`

### Test Scenarios for Each Endpoint

```bash
# For memories_create
test_memories_create_unauthorized() {
    # Try to create memory without authentication
    # Should return Unauthorized error
}

test_memories_create_cross_capsule() {
    # User A creates memory in User B's capsule
    # Should return Unauthorized or NotFound error
}

test_memories_create_owner() {
    # User creates memory in their own capsule
    # Should succeed
}

# For memories_read
test_memories_read_unauthorized() {
    # Try to read memory without authentication
    # Should return Unauthorized error
}

test_memories_read_cross_user() {
    # User A creates memory, User B tries to read it
    # Should return NotFound or Unauthorized
}

test_memories_read_owner() {
    # User reads their own memory
    # Should return memory data
}

# For memories_update
test_memories_update_unauthorized() {
    # Try to update memory without authentication
    # Should return Unauthorized error
}

test_memories_update_cross_user() {
    # User A creates memory, User B tries to update it
    # Should return NotFound or Unauthorized
}

test_memories_update_owner() {
    # User updates their own memory
    # Should succeed
}

# For memories_delete
test_memories_delete_unauthorized() {
    # Try to delete memory without authentication
    # Should return Unauthorized error
}

test_memories_delete_cross_user() {
    # User A creates memory, User B tries to delete it
    # Should return NotFound or Unauthorized
}

test_memories_delete_owner() {
    # User deletes their own memory
    # Should succeed
}

# For memories_list
test_memories_list_unauthorized() {
    # Try to list memories without authentication
    # Should return Unauthorized error
}

test_memories_list_cross_capsule() {
    # User A tries to list memories in User B's capsule
    # Should return empty list or Unauthorized
}

test_memories_list_owner() {
    # User lists memories in their own capsule
    # Should return their memories
}

# For memories_ping
test_memories_ping_unauthorized() {
    # Try to ping memories without authentication
    # Should return Unauthorized error
}

test_memories_ping_cross_user() {
    # User A pings User B's memories
    # Should return false for all or Unauthorized
}

test_memories_ping_owner() {
    # User pings their own memories
    # Should return actual presence data
}
```

## Priority 2: Gallery Management Authorization Tests

### Endpoints to Test

- `galleries_create(gallery_data)`
- `galleries_create_with_memories(gallery_data, sync_memories)`
- `galleries_read(gallery_id)`
- `galleries_update(gallery_id, update_data)`
- `galleries_delete(gallery_id)`
- `galleries_list()`
- `update_gallery_storage_location(gallery_id, new_location)`

### Test Scenarios for Each Endpoint

```bash
# For galleries_create
test_galleries_create_unauthorized() {
    # Try to create gallery without authentication
    # Should return Unauthorized error
}

test_galleries_create_owner() {
    # User creates gallery in their own capsule
    # Should succeed
}

# For galleries_read
test_galleries_read_unauthorized() {
    # Try to read gallery without authentication
    # Should return Unauthorized error
}

test_galleries_read_cross_user() {
    # User A creates gallery, User B tries to read it
    # Should return NotFound or Unauthorized
}

test_galleries_read_owner() {
    # User reads their own gallery
    # Should return gallery data
}

# For galleries_update
test_galleries_update_unauthorized() {
    # Try to update gallery without authentication
    # Should return Unauthorized error
}

test_galleries_update_cross_user() {
    # User A creates gallery, User B tries to update it
    # Should return NotFound or Unauthorized
}

test_galleries_update_owner() {
    # User updates their own gallery
    # Should succeed
}

# For galleries_delete
test_galleries_delete_unauthorized() {
    # Try to delete gallery without authentication
    # Should return Unauthorized error
}

test_galleries_delete_cross_user() {
    # User A creates gallery, User B tries to delete it
    # Should return NotFound or Unauthorized
}

test_galleries_delete_owner() {
    # User deletes their own gallery
    # Should succeed
}

# For galleries_list
test_galleries_list_unauthorized() {
    # Try to list galleries without authentication
    # Should return Unauthorized error
}

test_galleries_list_owner() {
    # User lists their own galleries
    # Should return their galleries only
}
```

## Priority 3: Admin Functions Authorization Tests

### Endpoints to Test

- `add_admin(principal)`
- `remove_admin(principal)`
- `list_admins()`
- `list_superadmins()`

### Test Scenarios for Each Endpoint

```bash
# For add_admin
test_add_admin_unauthorized() {
    # Regular user tries to add admin
    # Should return Unauthorized error
}

test_add_admin_superadmin() {
    # Superadmin adds admin
    # Should succeed
}

test_add_admin_regular_admin() {
    # Regular admin tries to add admin
    # Should return Unauthorized error
}

# For remove_admin
test_remove_admin_unauthorized() {
    # Regular user tries to remove admin
    # Should return Unauthorized error
}

test_remove_admin_superadmin() {
    # Superadmin removes admin
    # Should succeed
}

# For list_admins
test_list_admins_unauthorized() {
    # Regular user tries to list admins
    # Should return empty list or Unauthorized
}

test_list_admins_admin() {
    # Admin lists admins
    # Should return admin list
}

test_list_admins_superadmin() {
    # Superadmin lists admins
    # Should return admin list
}

# For list_superadmins
test_list_superadmins_unauthorized() {
    # Regular user tries to list superadmins
    # Should return empty list or Unauthorized
}

test_list_superadmins_admin() {
    # Regular admin tries to list superadmins
    # Should return empty list or Unauthorized
}

test_list_superadmins_superadmin() {
    # Superadmin lists superadmins
    # Should return superadmin list
}
```

## Priority 4: User/Auth Functions Authorization Tests

### Endpoints to Test

- `register_with_nonce(nonce)`
- `verify_nonce(nonce)`

### Test Scenarios

```bash
# For register_with_nonce
test_register_with_nonce_unauthorized() {
    # Try to register without valid nonce
    # Should return Unauthorized error
}

test_register_with_nonce_valid() {
    # Register with valid nonce
    # Should succeed
}

# For verify_nonce
test_verify_nonce_unauthorized() {
    # Try to verify invalid nonce
    # Should return NotFound error
}

test_verify_nonce_valid() {
    # Verify valid nonce
    # Should return principal
}
```

## Priority 5: Upload Functions Authorization Tests

### Endpoints to Test

- Upload configuration endpoints
- Chunked upload endpoints
- Upload session management

### Test Scenarios

```bash
# For upload functions
test_upload_unauthorized() {
    # Try to upload without authentication
    # Should return Unauthorized error
}

test_upload_owner() {
    # User uploads to their own capsule
    # Should succeed
}

test_upload_cross_user() {
    # User tries to upload to another user's capsule
    # Should return Unauthorized error
}
```

## Implementation Plan

### Phase 1: Fix Broken Unit Tests

- [ ] Remove references to non-existent `metadata::tests`
- [ ] Remove references to non-existent `upload::tests`
- [ ] Remove references to non-existent `verify_caller_authorized` function
- [ ] Update unit tests to only test existing modules

### Phase 2: Memory Management Auth Tests

- [ ] Create `test_memories_authorization_e2e.sh`
- [ ] Implement all memory endpoint authorization tests
- [ ] Test cross-user access control
- [ ] Test rate limiting for memory operations

### Phase 3: Gallery Management Auth Tests

- [ ] Create `test_galleries_authorization_e2e.sh`
- [ ] Implement all gallery endpoint authorization tests
- [ ] Test cross-user access control
- [ ] Test gallery-memory relationship authorization

### Phase 4: Admin Functions Auth Tests

- [ ] Create `test_admin_authorization_e2e.sh`
- [ ] Implement all admin endpoint authorization tests
- [ ] Test superadmin vs regular admin privileges
- [ ] Test admin privilege escalation prevention

### Phase 5: User/Auth Functions Auth Tests

- [ ] Create `test_user_authorization_e2e.sh`
- [ ] Implement user registration authorization tests
- [ ] Test nonce verification security
- [ ] Test user isolation

### Phase 6: Upload Functions Auth Tests

- [ ] Create `test_upload_authorization_e2e.sh`
- [ ] Implement upload endpoint authorization tests
- [ ] Test upload session security
- [ ] Test cross-user upload prevention

### Phase 7: Comprehensive Integration Tests

- [ ] Create `test_authorization_comprehensive.sh`
- [ ] Test complex multi-user scenarios
- [ ] Test edge cases and boundary conditions
- [ ] Test performance under authorization load

## Test Infrastructure Improvements

### Enhanced Test Utilities

- [ ] Add `test_unauthorized_access()` helper function
- [ ] Add `test_cross_user_access()` helper function
- [ ] Add `test_admin_only_access()` helper function
- [ ] Add `test_rate_limiting()` helper function

### Multi-User Test Support

- [ ] Add support for multiple test identities
- [ ] Add identity switching utilities
- [ ] Add cross-user data setup helpers
- [ ] Add cleanup utilities for multi-user tests

### Reporting and Monitoring

- [ ] Add authorization test coverage reporting
- [ ] Add security test result aggregation
- [ ] Add performance metrics for auth tests
- [ ] Add automated security regression detection

## Security Considerations

### Critical Security Tests

- [ ] Test privilege escalation prevention
- [ ] Test data isolation between users
- [ ] Test admin function protection
- [ ] Test rate limiting effectiveness
- [ ] Test session management security

### Edge Cases

- [ ] Test anonymous caller handling
- [ ] Test invalid principal handling
- [ ] Test malformed request handling
- [ ] Test concurrent access scenarios
- [ ] Test resource exhaustion scenarios

## Success Criteria

### Coverage Goals

- [ ] 100% endpoint authorization coverage
- [ ] 100% user role authorization coverage
- [ ] 100% cross-user access prevention
- [ ] 100% admin function protection

### Performance Goals

- [ ] Authorization tests complete in < 5 minutes
- [ ] No false positives in authorization tests
- [ ] No false negatives in authorization tests
- [ ] Tests run reliably in CI/CD pipeline

### Security Goals

- [ ] Zero unauthorized access scenarios pass
- [ ] Zero privilege escalation scenarios pass
- [ ] Zero data leakage scenarios pass
- [ ] Zero admin function bypass scenarios pass

## Notes

- All authorization tests should be deterministic and repeatable
- Tests should clean up after themselves to avoid state pollution
- Tests should work in both local and mainnet environments
- Tests should provide clear error messages for debugging
- Tests should be maintainable and easy to extend

## Related Issues

- [Admin Functions Unit Testing - Decoupling for Testability](../../../docs/issues/open/admin-functions-unit-testing-decoupling.md)
- [Admin System Decoupled Architecture](../../../docs/issues/open/admin-decoupled-architecture.md)
- [Testing Strategy for ICP](../../../docs/testing-strategy-icp.md)
