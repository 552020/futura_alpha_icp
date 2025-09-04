# Stable Memory Migration: API Compatibility Issues

## ⚠️ ARCHIVED DOCUMENT - See capsule-storage-foundation-plan.md

**Status:** 🏛️ **ARCHIVED** - Historical Technical Reference
**Migration Completed:** ✅ **Stable Memory Implementation Complete**
**Current Status:** See [capsule-storage-foundation-plan.md](../capsule-storage-foundation-plan.md) for current project status

---

## Original Issue Description (Historical)

This document captured the challenges of migrating from volatile thread-local storage (`HashMap`) to persistent stable memory (`StableBTreeMap`) for capsule storage during the initial implementation phase.

## Current Status

### ✅ What We've Accomplished

- **Removed volatile storage**: Eliminated `CAPSULES` thread-local `HashMap` declaration
- **Renamed functions**: Changed `with_stable_capsules` → `with_capsules` (now default)
- **Updated imports**: Fixed `capsule.rs` to use new function names
- **Removed legacy functions**: Eliminated duplicate volatile access functions

### ❌ What's Blocking Us

- **API incompatibility**: `StableBTreeMap` has different methods than `HashMap`
- **Method differences**: Need to rewrite all capsule operations to work with stable storage
- **Complex refactoring**: This affects 65+ functions across the codebase

## Technical Details

### The Core Problem

```rust
// OLD (HashMap) - Works
capsules.values().find(|c| ...).cloned()

// NEW (StableBTreeMap) - Doesn't work
capsules.values().find(|c| ...).cloned()  // ❌ No `values()` method
```

### Method Mapping Required

| HashMap Method | StableBTreeMap Equivalent  | Status            |
| -------------- | -------------------------- | ----------------- | ----------- | ----------------- |
| `values()`     | `iter().map(               | (\_, v)           | v)`         | ❌ Need to update |
| `cloned()`     | `map(                      | (\_, v)           | v.clone())` | ❌ Need to update |
| `get_mut()`    | Different mutation pattern | ❌ Need to update |
| `values_mut()` | `iter_mut().map(           | (\_, v)           | v)`         | ❌ Need to update |

## Impact Assessment

### Scope

- **Files affected**: `capsule.rs` (primary), potentially others
- **Functions affected**: 65+ backend endpoints
- **Test impact**: All capsule-related tests will need updates
- **Deployment risk**: High - this affects core data storage

### Current Linter Errors

```
Line 260: no method named `cloned` found for enum `std::option::Option<types::Capsule>`
Line 298: no method named `cloned` found for enum `std::option::Option<types::Capsule>`
Line 354: no method named `cloned` found for enum `std::option::Option<types::Capsules>`
Line 390: no method named `get_mut` found for mutable reference
Line 408: no method named `values_mut` found for mutable reference
Line 428: no method named `values_mut` found for mutable reference
Line 469: cannot be built from an iterator over elements
Line 489: no method named `cloned` found for enum `std::option::Option<types::Capsule>`
Line 503: no method named `cloned` found for enum `std::option::Option<types::Capsule>`
Line 596: no method named `cloned` found for enum `std::option::Option<types::Capsule>`
```

## Questions for Senior Developer

### 1. **Architecture Decision**

- **Should we continue** with this stable memory migration approach?
- **Alternative approaches** we should consider?
- **Is this the right time** for this refactoring?

### 2. **Implementation Strategy**

- **Best practices** for migrating from `HashMap` to `StableBTreeMap`?
- **Patterns** for handling the different API methods?
- **Should we create wrapper functions** to maintain similar API?

### 3. **Scope and Priority**

- **Is this migration critical** for the current sprint?
- **Should we defer** this until after other refactoring is complete?
- **Risk assessment** of continuing vs. rolling back?

### 4. **Technical Approach**

- **Recommended patterns** for stable storage operations?
- **Error handling** strategies for stable memory operations?
- **Performance implications** of stable vs. volatile storage?

## Proposed Solutions

### Option 1: Complete Migration (Current Approach)

- **Pros**: Persistent storage, no data loss on restarts
- **Cons**: Massive refactoring, high risk, blocks other work
- **Timeline**: 2-3 days minimum

### Option 2: Hybrid Approach

- **Pros**: Gradual migration, lower risk
- **Cons**: More complex, temporary duplication
- **Timeline**: 1-2 weeks

### Option 3: Defer Migration

- **Pros**: Focus on other refactoring, lower risk
- **Cons**: Data loss on restarts, technical debt
- **Timeline**: Move to future sprint

### Option 4: Create Compatibility Layer

- **Pros**: Minimal code changes, familiar API
- **Cons**: Performance overhead, complexity
- **Timeline**: 1-2 days

## Code Examples

### Current Problematic Code

```rust
// This pattern doesn't work with StableBTreeMap
let existing_self_capsule = with_capsules(|capsules| {
    capsules
        .values()  // ❌ No such method
        .find(|capsule| capsule.subject == caller)
        .cloned()  // ❌ No such method
});
```

### What We Need

```rust
// This pattern works with StableBTreeMap
let existing_self_capsule = with_capsules(|capsules| {
    capsules
        .iter()  // ✅ Available
        .find(|(_, capsule)| capsule.subject == caller)
        .map(|(_, capsule)| capsule.clone())  // ✅ Available
});
```

## Senior Developer Responses

### 🎯 **Senior Developer #1: Continue Migration with Compatibility Layer**

**Short answer**: Yes, continue the migration—but do it behind a thin compatibility layer and switch call sites incrementally.

**Recommended Approach**: Option 4 (Compatibility Layer) + Option 2 (Hybrid)

- **Phase 1**: Create adapter that exposes HashMap-like helpers over StableBTreeMap
- **Phase 2**: Switch most critical paths (reads/writes that must persist)
- **Phase 3**: Defer long-tail refactors to reduce churn and unblock sprint

### 🎯 **Senior Developer #2: Repository Pattern with Trait + Adapters**

**Short answer**: You're on the right track. Don't fight `StableBTreeMap` directly across 65+ call sites—put a thin compatibility layer in front of it and migrate the app code to that layer.

**Recommended Approach**: Create `CapsuleStore` trait with two adapters:

- **`HashMapStore`** implements over `HashMap` (fast, for tests)
- **`StableStore`** implements over `StableBTreeMap` (persistent, for production)
- **App code only calls the trait** - removes all map API surface from 65+ endpoints

### 🏗️ **Architecture Decision**

- **Prefer StableBTreeMap** over heap HashMap to avoid pre/post-upgrade serialization limits
- **Avoid failed upgrades** on large state
- **Version stable memory** and test upgrades
- **Scale to GBs** without hooks

### 🔧 **Implementation Strategy (Senior #1)**

Create a small adapter that exposes HashMap-like helpers over StableBTreeMap idioms:

- `values()`: `btree.iter().map(|(_, v)| v)`
- `values_mut()`: `btree.iter_mut().map(|(_, v)| v)`
- `cloned()`: `iterator.map(|x| x.clone())`
- `get_mut()`: Consider take/put or explicit update paths

### 🔧 **Implementation Strategy (Senior #2)**

Create a `CapsuleStore` trait and two adapters:

```rust
pub trait CapsuleStore {
    type Iter<'a>: Iterator<Item=(types::CapsuleId, types::Capsule)> where Self: 'a;

    fn get(&self, id: &types::CapsuleId) -> Option<types::Capsule>;
    fn put(&mut self, id: types::CapsuleId, c: types::Capsule) -> Option<types::Capsule>;
    fn remove(&mut self, id: &types::CapsuleId) -> Option<types::Capsule>;
    fn iter(&self) -> Self::Iter<'_>;

    // Convenience: common queries you use everywhere
    fn find_by_subject(&self, subj: &Principal) -> Option<types::Capsule> {
        self.iter().find_map(|(_, c)| (c.subject == *subj).then_some(c))
    }
}
```

- **`HashMapStore`** implements this over `HashMap` (fast, for tests)
- **`StableStore`** implements this over `StableBTreeMap` (persistent, for production)
- **App code only calls the trait** - removes all map API surface from 65+ endpoints

### 📊 **Scope and Priority**

- **Hybrid approach**: Introduce adapter now, switch critical paths, defer long-tail refactors
- **Reduce churn** and unblock sprint while moving toward target architecture
- **Acceptable for primary storage** at scale with Wasm-native stable memory

### 🚀 **Technical Approach Patterns (Senior #1)**

- Use `iter/find/map/clone` in place of `values()/cloned()`
- Design explicit update functions that read, modify, then insert back
- Version stable memory (e.g., header byte) for future migrations
- Keep all globals in `thread_local` with `RefCell` as recommended

### 🚀 **Technical Approach Patterns (Senior #2)**

**Patterns you'll need (mapping cheatsheet):**

- **values()**: `map.iter().map(|(_, v)| v)` - expose `iter()` returning `(K, V)` then `.map(|(_, v)| v)` as needed
- **cloned()**: `Option<T>::cloned()` exists on `Option<&T>`. With `StableBTreeMap::iter()` you already have owned `V`, so use `.map(|(_, v)| v)` (owned) or `.as_ref().cloned()` only when you truly have `Option<&T>`
- **get_mut()/values_mut()**: Not supported. Use read-modify-write:
  ```rust
  if let Some(mut c) = store.get(&id) {
      /* mutate c */
      store.put(id, c);
  }
  ```

**Error handling & perf:**

- Expect O(n) scans for "find by subject". If that's hot, add a secondary index
- Avoid "mutable views" in public API. Only expose read/put operations
- Encapsulate secondary-index maintenance inside the adapter

## Next Steps

### Immediate (Following Senior Guidance)

1. **Create compatibility adapter** for StableBTreeMap
2. **Apply to hottest call sites** for immediate unblocking
3. **Proceed with gradual migration** to limit risk this sprint

### Implementation Plan (Senior #1)

1. **Create CapsulesAdapter** with HashMap-like API
2. **Update critical functions** to use stable storage
3. **Test core functionality** (create, read, update)
4. **Gradually migrate** remaining functions
5. **Remove volatile storage** completely

### Implementation Plan (Senior #2)

**Rollout plan (tight):**

1. **Introduce `CapsuleStore` trait + `HashMapStore`**
2. **Migrate endpoints to the trait** (no behavior change)
3. **Add `StableStore` + optional secondary index**
4. **Run test suite against both impls** (feature flag)
5. **Default to `StableStore` when green**

**This keeps the blast radius tiny, unblocks the sprint, and gives you a clean switch to stable memory without touching 65+ call sites twice.**

### Success Metrics

- ✅ **No linter errors** on critical paths
- ✅ **Core functionality working** with stable storage
- ✅ **Sprint unblocked** for other refactoring work
- ✅ **Architecture aligned** with ICP best practices

---

## 📚 Technical Patterns Extracted

**Important:** The valuable technical implementation patterns from this document have been extracted and integrated into the current usage guide:

- **Method Mapping Cheatsheet** → [capsule_store_usage_guide.md](../capsule_store_usage_guide.md#hashmap-vs-stablebtreemap-method-mapping)
- **Read-Modify-Write Patterns** → [capsule_store_usage_guide.md](../capsule_store_usage_guide.md#read-modify-write-pattern-for-mutations)
- **Performance Optimization Notes** → [capsule_store_usage_guide.md](../capsule_store_usage_guide.md#performance-optimization-notes)
- **Error Handling Best Practices** → [capsule_store_usage_guide.md](../capsule_store_usage_guide.md#error-handling-best-practices)

**For current development, refer to the [Capsule Store Usage Guide](../capsule_store_usage_guide.md) for up-to-date patterns and best practices.**

## Senior Developer's Adapter Pattern

### Minimal Adapter Sketch

```rust
// Pattern based on stable-structures examples
// Use thread_local! and MemoryManager per docs.
pub struct Capsules<'a, M>(pub &'a mut ic_stable_structures::StableBTreeMap<Key, Val, M>);

impl<'a, M: ic_stable_structures::Memory> Capsules<'a, M> {
    pub fn values<'b>(&'b self) -> impl Iterator<Item = &'b Val> {
        self.0.iter().map(|(_, v)| v)
    }
    pub fn values_mut<'b>(&'b mut self) -> impl Iterator<Item = &'b mut Val> {
        self.0.iter_mut().map(|(_, v)| v)
    }
    pub fn get(&self, k: &Key) -> Option<&Val> {
        self.0.get(k)
    }
    pub fn get_owned(&self, k: &Key) -> Option<Val> where Val: Clone {
        self.0.get(k).cloned()
    }
    pub fn update_with<F>(&mut self, k: &Key, f: F) -> Option<Val>
    where
        Val: Clone,
        F: FnOnce(&mut Val),
    {
        if let Some(mut v) = self.0.get(k).cloned() {
            f(&mut v);
            self.0.insert(k.clone(), v)
        } else {
            None
        }
    }
}
```

### Key Benefits

- **HashMap-like API** for familiar development experience
- **Stable storage** for persistence across upgrades
- **Incremental migration** without breaking existing code
- **Performance optimized** with Wasm-native stable memory

## Risk Assessment

| Risk                        | Probability | Impact | Mitigation                    |
| --------------------------- | ----------- | ------ | ----------------------------- |
| **Data Loss**               | Low         | High   | Rollback to volatile storage  |
| **API Breaking Changes**    | High        | Medium | Comprehensive testing         |
| **Development Block**       | High        | Medium | Senior developer guidance     |
| **Performance Degradation** | Medium      | Low    | Benchmark stable vs. volatile |

## Related Issues

- **Phase 0 Refactoring**: This migration is part of our larger lib.rs reorganization
- **Backend Endpoint Renaming**: Related refactoring work
- **Test Consolidation**: Will need updates after migration

## Notes

- **Greenfield Project**: No legacy compatibility concerns
- **Current Sprint**: This was not planned for current iteration
- **Team Impact**: This affects all capsule-related functionality
- **Deployment**: Cannot deploy with current linter errors

---

**Status**: 🟡 IN PROGRESS - Following Senior Developer Guidance  
**Priority**: HIGH - Core functionality affected  
**Assignee**: Current Team  
**Created**: Current Session  
**Updated**: Current Session
