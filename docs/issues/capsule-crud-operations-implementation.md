# Capsule CRUD Operations Implementation

## Status: 🔴 **CRITICAL MISSING FUNCTIONALITY**

**Priority:** High  
**Effort:** Medium  
**Impact:** High - Incomplete CRUD system affects user experience

## Problem Statement

The current capsule management system is **incomplete** - we have CREATE and READ operations, but are missing UPDATE and DELETE operations. This creates an incomplete CRUD system that limits user functionality.

### Current CRUD Coverage:

- ✅ **CREATE**: `capsules_create` - Working
- ✅ **READ**: `capsules_read_full`, `capsules_read_basic`, `capsules_list` - Working
- ⚠️ **UPDATE**: `capsules_bind_neon` - Limited (only binding to Neon)
- ❌ **DELETE**: `capsules_delete` - **MISSING**

## Missing Functions

### 1. `capsules_update`

**Function Signature:**

```rust
fn capsules_update(capsule_id: String, updates: CapsuleUpdateData) -> Result<()>
```

**Purpose:** Update capsule properties like ownership, connections, and social graph

### 2. `capsules_delete`

**Function Signature:**

```rust
fn capsules_delete(capsule_id: String) -> Result<()>
```

**Purpose:** Delete a capsule (with proper authorization and cleanup)

## Proposed Implementation

### 1. CapsuleUpdateData Structure

```rust
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct CapsuleUpdateData {
    // Ownership changes
    pub add_owners: Option<Vec<PersonRef>>,
    pub remove_owners: Option<Vec<PersonRef>>,
    pub add_controllers: Option<Vec<PersonRef>>,
    pub remove_controllers: Option<Vec<PersonRef>>,

    // Social graph changes
    pub add_connections: Option<Vec<(PersonRef, Connection)>>,
    pub remove_connections: Option<Vec<PersonRef>>,
    pub update_connection_groups: Option<Vec<(String, ConnectionGroup)>>,
    pub remove_connection_groups: Option<Vec<String>>,
}
```

### 2. Capsule Properties Analysis

#### 🔒 **IMMUTABLE Properties (Cannot be updated):**

- `id` - Unique identifier (never changes)
- `created_at` - Creation timestamp (historical record)
- `subject` - Who the capsule is about (fundamental identity)

#### ✅ **UPDATABLE Properties:**

- **`owners`** - Add/remove owners (with proper authorization)
- **`controllers`** - Add/remove controllers (delegated admins)
- **`connections`** - Add/remove connections to other people
- **`connection_groups`** - Create/update/delete connection groups
- **`bound_to_neon`** - Already handled by `capsules_bind_neon`
- **`updated_at`** - Automatically updated on any change
- **`inline_bytes_used`** - Automatically calculated

#### 📝 **Content Properties (Handled by separate functions):**

- **`memories`** - Handled by `memories_create`, `memories_delete`, `memories_update`
- **`galleries`** - Handled by `galleries_create`, `galleries_delete`, `galleries_update`

## Use Cases

### 1. Ownership Transfer

```rust
// Add a co-owner to a capsule
capsules_update("capsule_123", CapsuleUpdateData {
    add_owners: Some(vec![PersonRef::Principal(new_owner)]),
    ..Default::default()
})
```

### 2. Social Graph Management

```rust
// Add a connection
capsules_update("capsule_123", CapsuleUpdateData {
    add_connections: Some(vec![(PersonRef::Principal(friend), Connection::new())]),
    ..Default::default()
})
```

### 3. Controller Management

```rust
// Grant controller access to a family member
capsules_update("capsule_123", CapsuleUpdateData {
    add_controllers: Some(vec![PersonRef::Principal(family_member)]),
    ..Default::default()
})
```

### 4. Connection Groups

```rust
// Create a family group
capsules_update("capsule_123", CapsuleUpdateData {
    update_connection_groups: Some(vec![("family".to_string(), family_group)]),
    ..Default::default()
})
```

### 5. Capsule Deletion

```rust
// Delete a capsule (with proper cleanup)
capsules_delete("capsule_123")
```

## Authorization Rules

### For `capsules_update`:

1. **Only owners/controllers can update** - Must have write access
2. **Ownership changes require special permissions** - Maybe only current owners can add/remove owners
3. **Self-capsule restrictions** - Some updates might be restricted for self-capsules
4. **Audit trail** - Track who made what changes and when

### For `capsules_delete`:

1. **Only owners can delete** - Controllers cannot delete
2. **Self-capsule deletion** - Special handling for self-capsules
3. **Cascade cleanup** - Remove from all indexes and related data
4. **Confirmation required** - Maybe require explicit confirmation

## Implementation Plan

### Phase 1: Data Structures

- [ ] Add `CapsuleUpdateData` to `types.rs`
- [ ] Add `CapsuleUpdateResponse` to `types.rs`
- [ ] Update Candid interface (`backend.did`)

### Phase 2: Core Functions

- [ ] Implement `capsules_update` in `capsule.rs`
- [ ] Implement `capsules_delete` in `capsule.rs`
- [ ] Add thin facades in `lib.rs`

### Phase 3: Authorization & Validation

- [ ] Implement authorization checks
- [ ] Add validation for update operations
- [ ] Handle edge cases (self-capsules, ownership changes)

### Phase 4: Testing

- [ ] Create `test_capsules_update.sh`
- [ ] Create `test_capsules_delete.sh`
- [ ] Update existing tests in `general/` folder
- [ ] Integration tests for complex scenarios

### Phase 5: Documentation

- [ ] Update API documentation
- [ ] Add usage examples
- [ ] Update Candid interface documentation

## Files to Modify

### Backend Code:

- `src/backend/src/types.rs` - Add new data structures
- `src/backend/src/capsule.rs` - Implement core functions
- `src/backend/src/lib.rs` - Add thin facades
- `src/backend/backend.did` - Update Candid interface

### Tests:

- `scripts/tests/backend/capsule/test_capsules_update.sh` - Create
- `scripts/tests/backend/capsule/test_capsules_delete.sh` - Create
- `scripts/tests/backend/general/` - Update existing tests

## Testing Strategy

### Unit Tests:

- Test authorization rules
- Test validation logic
- Test edge cases

### Integration Tests:

- Test with real canister
- Test complex update scenarios
- Test deletion with cleanup

### Test Scenarios:

1. **Basic Updates:**

   - Add/remove owners
   - Add/remove controllers
   - Add/remove connections

2. **Complex Updates:**

   - Multiple operations in one call
   - Ownership transfer
   - Connection group management

3. **Authorization:**

   - Unauthorized access attempts
   - Self-capsule restrictions
   - Controller vs owner permissions

4. **Deletion:**
   - Basic deletion
   - Self-capsule deletion
   - Cleanup verification

## Risks & Considerations

### 1. Data Integrity

- Ensure indexes are updated correctly
- Handle concurrent updates
- Maintain referential integrity

### 2. Authorization Complexity

- Complex ownership rules
- Self-capsule special cases
- Controller vs owner permissions

### 3. Backward Compatibility

- Ensure existing functions still work
- Don't break existing API contracts
- Maintain Candid interface compatibility

### 4. Performance

- Update operations should be efficient
- Index maintenance overhead
- Large capsule updates

## Success Criteria

- [ ] All CRUD operations implemented and working
- [ ] Comprehensive test coverage
- [ ] Proper authorization and validation
- [ ] No breaking changes to existing API
- [ ] Documentation updated
- [ ] Integration tests passing

## Related Issues

- [ ] Update existing test suite in `general/` folder
- [ ] Consider capsule versioning for audit trails
- [ ] Implement capsule sharing mechanisms
- [ ] Add capsule export/import functionality

## Notes

- The existing `capsules_bind_neon` function is a limited form of update
- Memory and gallery operations are handled separately (good separation of concerns)
- The `CapsuleStore` trait already has `update` and `remove` methods - we just need to expose them
- Consider implementing batch operations for efficiency

---

**Created:** 2024-01-XX  
**Last Updated:** 2024-01-XX  
**Assignee:** TBD  
**Labels:** `enhancement`, `crud`, `capsules`, `high-priority`
