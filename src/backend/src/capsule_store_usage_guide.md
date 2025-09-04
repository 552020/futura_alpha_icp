# Capsule Store Usage Guide

## File Transformation Summary

**Originally**: `capsule_migration_guide.md` - Outdated migration tracking (2/65+ functions)
**Now**: `capsule_store_usage_guide.md` - Comprehensive usage patterns for greenfield project

**Transformation**: Removed migration mindset, added best practices, focused on correct patterns for greenfield development.

## Overview

This guide shows the correct patterns for using the `CapsuleStore` trait in the codebase. Since this is a **greenfield project**, we use the modern `with_capsule_store` patterns exclusively - no backward compatibility needed.

## Important Notes

- ✅ **Greenfield Project**: No legacy `with_capsules` patterns exist
- ✅ **Single Source of Truth**: All capsule operations use `with_capsule_store`
- ✅ **Runtime Backend Selection**: HashMap (tests) vs Stable (production) via enum
- ✅ **Performance Optimized**: Secondary indexes for O(log n) queries

## Correct Usage Patterns

### Pattern 1: Simple Read Operations

```rust
// ✅ CORRECT - Use with_capsule_store
with_capsule_store(|store| {
    store.get(&capsule_id)
})
```

**Benefits:**

- Works with both HashMap (tests) and Stable (production) backends
- Type-safe at compile time
- Optimized with secondary indexes

### Pattern 2: Read with Filtering/Transformation

```rust
// ✅ CORRECT - Use store methods with filtering
with_capsule_store(|store| {
    store.get(&capsule_id)
        .filter(|capsule| capsule.has_write_access(&caller))
        .map(|capsule| transform_to_capsule_info(capsule))
})
```

**Tip:** Combine store methods with standard Rust iterators for complex queries.

### Pattern 3: Subject-Based Queries

```rust
// ✅ CORRECT - Use convenience methods
with_capsule_store(|store| {
    store.find_by_subject(&caller_principal)
})

// Alternative: Manual iteration (less efficient)
with_capsule_store(|store| {
    store.paginate(None, u32::MAX, Order::Asc)
        .items
        .into_iter()
        .find(|(_, capsule)| capsule.subject == caller_principal)
})
```

**Performance Note:** Use `find_by_subject()` for O(log n) performance with indexes.

### Pattern 4: Write Operations

```rust
// ✅ CORRECT - Use store mutation methods
with_capsule_store_mut(|store| {
    store.upsert(capsule_id, capsule)  // Insert or update
})

// For conditional updates:
with_capsule_store_mut(|store| {
    store.update(&capsule_id, |capsule| {
        capsule.updated_at = current_time();
        capsule.some_field = new_value;
    })
})
```

### Pattern 5: Batch Operations

```rust
// ✅ CORRECT - Use batch methods for efficiency
with_capsule_store(|store| {
    store.get_many(&[id1, id2, id3])  // Batch read
})

// ✅ CORRECT - Use pagination for large result sets
with_capsule_store(|store| {
    store.paginate(Some(last_id), 50, Order::Asc)  // Keyset pagination
})
```

## Best Practices

### 1. **Use Convenience Methods**

```rust
// ✅ PREFERRED - Built-in optimized methods
store.find_by_subject(&subject)
store.list_by_owner(&owner)
store.paginate(after, limit, order)

// ❌ AVOID - Manual inefficient patterns
store.paginate(None, u32::MAX, Order::Asc)
    .items.into_iter()
    .find(|(_, c)| c.subject == subject)
```

### 2. **Proper Error Handling**

```rust
// ✅ CORRECT - Handle update errors
with_capsule_store_mut(|store| {
    store.update(&capsule_id, |capsule| {
        // mutate capsule
    }).map_err(|e| match e {
        UpdateError::NotFound => Error::CapsuleNotFound,
        UpdateError::Validation(msg) => Error::Validation(msg),
        UpdateError::Concurrency => Error::ConcurrentModification,
    })
})
```

### 3. **Backend Selection**

```rust
// Tests automatically use HashMap
#[cfg(test)]
with_capsule_store(|store| { /* fast tests */ });

// Production uses Stable (should be configured)
with_capsule_store(|store| { /* persistent data */ });
```

## Common Mistakes to Avoid

### ❌ **Don't Use Old Patterns**

```rust
// WRONG - These patterns don't exist in greenfield
with_capsules(|capsules| { /* old HashMap access */ });
with_capsules_mut(|capsules| { /* old mutation */ });
```

### ❌ **Don't Bypass the Store**

```rust
// WRONG - Direct memory access
with_hashmap_capsules(|capsules| {
    capsules.get(&id)  // Skip the abstraction layer
});
```

### ❌ **Don't Use Inefficient Patterns**

```rust
// WRONG - O(n) scan instead of O(log n) index lookup
with_capsule_store(|store| {
    store.paginate(None, u32::MAX, Order::Asc)
        .items.into_iter()
        .find(|(_, c)| c.subject == subject)  // Inefficient!
});
```

## Testing Guidelines

When writing tests:

- ✅ **Use the store abstraction** - tests work with both backends
- ✅ **Test both backends** when possible
- ✅ **Mock at the store level** for unit tests
- ✅ **Verify data persistence** across backend switches

---

**Status**: 🟢 ACTIVE - Usage patterns documented
**Purpose**: Guide for correct CapsuleStore usage patterns
