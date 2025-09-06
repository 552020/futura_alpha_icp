# Hash Store vs Stable Store Analysis

## Problem Statement

We have two capsule storage implementations:

1. **HashMap Store** (`capsule_store/hash.rs`) - In-memory storage
2. **StableBTreeMap Store** (`capsule_store/stable.rs`) - Persistent storage

**Question**: Is the HashMap store legacy code that should be removed, or does it serve a legitimate purpose?

## Current Architecture

### **HashMap Store (`capsule_store/hash.rs`)**

```rust
pub struct HashMapStore {
    capsules: RefCell<HashMap<String, Capsule>>,
    subject_index: RefCell<HashMap<Vec<u8>, String>>,
    owner_index: RefCell<HashMap<Vec<u8>, HashSet<String>>>,
}
```

**Characteristics:**

- **Storage**: In-memory HashMap
- **Persistence**: Data lost on canister restart
- **Performance**: O(1) operations
- **Memory**: Uses heap memory
- **Use Case**: Testing, development, temporary storage

### **StableBTreeMap Store (`capsule_store/stable.rs`)**

```rust
pub struct StableStore {
    capsules: StableBTreeMap<CapsuleId, Capsule, VirtualMemory<DefaultMemoryImpl>>,
    subject_index: StableBTreeMap<Vec<u8>, CapsuleId, VirtualMemory<DefaultMemoryImpl>>,
    owner_index: StableBTreeMap<OwnerIndexKey, (), VirtualMemory<DefaultMemoryImpl>>,
}
```

**Characteristics:**

- **Storage**: Persistent stable memory
- **Persistence**: Survives canister upgrades
- **Performance**: O(log n) operations
- **Memory**: Uses stable memory (limited)
- **Use Case**: Production, permanent storage

## Arguments for Keeping HashMap Store

### **1. Testing & Development**

- **Fast iteration**: No persistence overhead during development
- **Predictable state**: Clean slate on each restart
- **Easy debugging**: Simple in-memory data structures
- **Unit testing**: Isolated test environments

### **2. Runtime Flexibility**

- **Feature flags**: Switch between storage backends
- **Performance testing**: Compare HashMap vs StableBTreeMap performance
- **Migration testing**: Test data migration between backends
- **Fallback option**: Emergency fallback if stable storage fails

### **3. Code Architecture**

- **Trait-based design**: Both implement the same `CapsuleStore` trait
- **Clean separation**: Different concerns (speed vs persistence)
- **Future-proofing**: Easy to add new storage backends

## Arguments for Removing HashMap Store

### **1. Code Duplication**

- **Maintenance burden**: Two implementations to maintain
- **Bug risk**: Bugs can exist in one but not the other
- **Testing complexity**: Need to test both implementations
- **Code bloat**: Unnecessary code in production

### **2. Production Reality**

- **IC requirement**: Production must use stable storage
- **No real use case**: HashMap store can't be used in production
- **Confusion**: Developers might accidentally use wrong store
- **Memory waste**: HashMap store uses precious heap memory

### **3. Current Usage Analysis**

- **Production code**: Only uses StableBTreeMap store
- **Test code**: Uses legacy HashMap functions (not the new store)
- **Migration**: Already migrated to stable storage
- **Dead code**: HashMap store might be unused

## Current Usage Investigation

### **HashMap Store Usage**

```bash
# Search for HashMapStore usage
grep -r "HashMapStore" src/backend/src/
grep -r "Store::HashMap" src/backend/src/
```

### **Legacy HashMap Usage**

```bash
# Search for legacy HashMap capsule storage
grep -r "with_capsules" src/backend/src/
grep -r "CAPSULES.*HashMap" src/backend/src/
```

### **Stable Store Usage**

```bash
# Search for StableStore usage
grep -r "StableStore" src/backend/src/
grep -r "Store::Stable" src/backend/src/
```

## Recommendation Matrix

| Scenario               | Keep HashMap Store      | Remove HashMap Store       |
| ---------------------- | ----------------------- | -------------------------- |
| **Active development** | ✅ Fast iteration       | ❌ Slower testing          |
| **Production ready**   | ❌ Unnecessary code     | ✅ Clean codebase          |
| **Team size: 1-2**     | ❌ Maintenance burden   | ✅ Simpler codebase        |
| **Team size: 5+**      | ✅ Parallel development | ❌ Slower iteration        |
| **Complex testing**    | ✅ Multiple backends    | ❌ Limited testing options |
| **Simple testing**     | ❌ Overkill             | ✅ Sufficient              |

## Decision Framework

### **Keep HashMap Store If:**

- [ ] Active development phase
- [ ] Need fast iteration cycles
- [ ] Complex testing requirements
- [ ] Large development team
- [ ] Performance comparison needed
- [ ] Migration testing required

### **Remove HashMap Store If:**

- [ ] Production-ready codebase
- [ ] Small development team
- [ ] Simple testing needs
- [ ] Code maintenance burden
- [ ] Memory optimization needed
- [ ] Clean architecture priority

## Proposed Action Plan

### **Option A: Keep Both (Conservative)**

1. **Audit usage**: Verify HashMap store is actually used
2. **Add feature flags**: Control which store to use
3. **Document purpose**: Clear documentation of when to use each
4. **Monitor maintenance**: Track time spent maintaining both

### **Option B: Remove HashMap Store (Aggressive)**

1. **Verify unused**: Confirm HashMap store is not used in production
2. **Update tests**: Migrate any tests using HashMap store
3. **Remove code**: Delete `capsule_store/hash.rs`
4. **Simplify architecture**: Remove trait complexity if not needed

### **Option C: Hybrid Approach (Pragmatic)**

1. **Keep for now**: Don't remove during active development
2. **Add deprecation warning**: Mark HashMap store as deprecated
3. **Plan removal**: Schedule removal for next major version
4. **Document decision**: Record why and when to remove

## Questions for Senior Review

1. **What is the current development phase?** (MVP, Beta, Production)
2. **How often do you need fast iteration?** (Daily, Weekly, Monthly)
3. **What is the team size?** (1-2, 3-5, 5+)
4. **How complex are your testing requirements?**
5. **What is the maintenance burden tolerance?**
6. **Is memory optimization critical?**
7. **What is the timeline for production deployment?**

## Conclusion

The decision depends on your current development phase and team needs:

- **Keep HashMap Store**: If you're in active development and need fast iteration
- **Remove HashMap Store**: If you're production-ready and want clean code
- **Hybrid Approach**: If you're unsure and want to defer the decision

**Recommendation**: Start with Option C (Hybrid) - keep it for now but plan for removal, unless you have specific needs for fast iteration during development.

---

**Next Steps**:

1. Audit actual usage of HashMap store
2. Get senior review on development phase and team needs
3. Make decision based on findings
4. Implement chosen approach
