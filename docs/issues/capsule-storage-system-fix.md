# Capsule Storage System Fix

## Problem Analysis

The current codebase has **three different capsule storage systems** running in parallel, creating confusion and potential data inconsistency:

### 1. Legacy HashMap Storage (Currently Active)

```rust
// In memory.rs line 62
static CAPSULES: std::cell::RefCell<HashMap<String, Capsule>> = RefCell::new(HashMap::new());
```

**Access functions:**

- `with_capsules()` - Read access
- `with_capsules_mut()` - Write access

**Used by:** Test code in `export.rs` and some legacy code

**Functions Using Legacy HashMap (8 total):**

#### **Production Functions (2):**

1. **`get_stable_memory_stats()`** (memory.rs:304) - Gets capsule count from HashMap
2. **`migrate_capsules_to_stable()`** (memory.rs:302) - Migration helper (has bug, not used)

#### **Test Functions (6):**

3. **`test_export_user_capsule_data_success()`** (export.rs:683, 712) - 2 calls
4. **`test_export_user_capsule_data_no_self_capsule()`** (export.rs:734, 747) - 2 calls
5. **`test_export_user_capsule_data_not_owner()`** (export.rs:769, 781) - 2 calls

#### **Functions to Delete (4):**

6. **`with_capsules()`** (memory.rs:212) - Backward compatibility alias
7. **`with_capsules_mut()`** (memory.rs:220) - Backward compatibility alias
8. **`with_hashmap_capsules()`** (memory.rs:192) - Direct HashMap access
9. **`with_hashmap_capsules_mut()`** (memory.rs:200) - Direct HashMap access

**Issues:**

- Data is lost on canister upgrade (not persistent)
- Not suitable for production
- Still actively used in tests and some code paths

### 2. StableBTreeMap Storage (Partially Implemented)

```rust
// In memory.rs line 31
static STABLE_CAPSULES: RefCell<StableBTreeMap<String, Capsule, Memory>> = RefCell::new(
    StableBTreeMap::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(CAPSULES_MEMORY_ID))
    )
);
```

**Access functions:**

- `with_stable_capsules()` - Read access (marked as ` `)
- `with_stable_capsules_mut()` - Write access (marked as ` `)

**Issues:**

- Implemented but not used (dead code)
- Uses `CAPSULES_MEMORY_ID` instead of `MEM_CAPSULES`
- Redundant with the modern CapsuleStore system

### 3. Modern CapsuleStore System (Latest Architecture)

```rust
// In capsule_store/stable.rs
const MEM_CAPSULES: MemoryId = MemoryId::new(0);  // Uses the same ID!
```

**Features:**

- Actually uses `MEM_CAPSULES` from `memory_manager.rs`
- Modern trait-based architecture
- Supports both HashMap (testing) and StableBTreeMap (production) backends
- Runtime switching between backends
- Proper error handling and validation

## Root Cause

The `MEM_CAPSULES` constant in `memory_manager.rs` is **not being used** because:

1. **Legacy code** still uses the old HashMap system
2. **StableBTreeMap system** in `memory.rs` is unused (dead code)
3. **Modern CapsuleStore system** is implemented but not integrated with existing code

## Impact

- **Data Loss Risk**: Legacy HashMap storage is not persistent across canister upgrades
- **Code Confusion**: Multiple storage systems make the codebase hard to understand
- **Maintenance Burden**: Three different systems need to be maintained
- **Testing Issues**: Tests use legacy system, not the production system

## Solution

### Phase 1: Migration Planning

1. **Audit current usage** of `with_capsules()` and `with_capsules_mut()`
2. **Identify all code paths** that need migration
3. **Create migration strategy** for existing data

### Phase 2: Code Migration

1. **Replace legacy calls** with CapsuleStore system
2. **Update test code** to use CapsuleStore
3. **Remove dead code** (unused StableBTreeMap system in memory.rs)
4. **Ensure MEM_CAPSULES is properly used**

### Phase 3: Validation

1. **Test migration** with existing data
2. **Verify persistence** across canister upgrades
3. **Performance testing** of new system

## Files to Update

### High Priority

- `src/backend/src/canister_factory/export.rs` - Test code uses legacy system
- `src/backend/src/memory.rs` - Remove dead StableBTreeMap code
- `src/backend/src/capsule.rs` - Migrate to CapsuleStore

### Medium Priority

- Any other files using `with_capsules()` or `with_capsules_mut()`
- Test files that need to use the new system

## Implementation Steps

1. **Audit current usage**:

   ```bash
   grep -r "with_capsules" src/backend/src/
   ```

2. **Update test code** in `export.rs`:

   ```rust
   // Replace this:
   crate::memory::with_capsules_mut(|capsules| {
       capsules.insert("test_capsule".to_string(), capsule.clone());
   });

   // With this:
   crate::memory::with_capsule_store(|store| {
       store.upsert("test_capsule".to_string(), capsule.clone());
   });
   ```

3. **Remove dead code** in `memory.rs`:

   - Remove `STABLE_CAPSULES` and related functions
   - Remove `CAPSULES_MEMORY_ID` constant

4. **Ensure proper integration**:
   - Verify `MEM_CAPSULES` is used by CapsuleStore
   - Test persistence across canister upgrades

## Success Criteria

- [ ] Only one capsule storage system exists (CapsuleStore)
- [ ] `MEM_CAPSULES` constant is actively used
- [ ] All tests pass with new system
- [ ] Data persists across canister upgrades
- [ ] No dead code remains
- [ ] Performance is acceptable

## Risks

- **Data Loss**: If migration is not done carefully
- **Breaking Changes**: Existing code may break during migration
- **Performance Impact**: New system may have different performance characteristics

## Timeline

- **Phase 1**: 1-2 days (audit and planning)
- **Phase 2**: 3-5 days (code migration)
- **Phase 3**: 1-2 days (validation)

**Total**: 5-9 days

## Notes

- The CapsuleStore system is already implemented and tested
- The main work is migrating existing code to use it
- This is a critical fix for production readiness
- Consider doing this before any major feature development
