# Capsule Store Migration Guide

## Overview

This guide shows how to migrate from the old `with_capsules` pattern to the new trait-based `with_capsule_store` pattern.

## Migration Status

✅ **Completed:**

- `capsules_read()` - Simple read operation
- `capsules_read_basic()` - Read with transformation

🔄 **In Progress:**

- `capsules_create()` - Complex read/write operation
- `capsule_read_self()` - Subject-based queries

## Migration Patterns

### Pattern 1: Simple Read Operations

#### Before:

```rust
with_capsules(|capsules: &HashMap<String, Capsule>| {
    capsules.get(&capsule_id)
})
```

#### After:

```rust
with_capsule_store(|store: &dyn CapsuleStore| {
    store.get(&capsule_id)
})
```

### Pattern 2: Read with Filtering/Transformation

#### Before:

```rust
with_capsules(|capsules| {
    capsules
        .get(&capsule_id)
        .filter(|capsule| capsule.has_access(&caller))
        .map(|capsule| transform_capsule(capsule))
})
```

#### After:

```rust
with_capsule_store(|store| {
    store.get(&capsule_id)
        .filter(|capsule| capsule.has_access(&caller))
        .map(|capsule| transform_capsule(capsule))
})
```

### Pattern 3: Iteration Operations

#### Before:

```rust
with_capsules(|capsules| {
    capsules
        .values()
        .find(|capsule| capsule.subject == subject)
})
```

#### After:

```rust
with_capsule_store(|store| {
    store.find_by_subject(&subject)
})
```

### Pattern 4: Write Operations

#### Before:

```rust
with_capsules_mut(|capsules| {
    capsules.insert(capsule_id, capsule)
})
```

#### After:

```rust
with_capsule_store_mut(|store| {
    store.put(capsule_id, capsule)
})
```

## Benefits of Migration

1. **Object-Safe**: Works with `dyn CapsuleStore`
2. **Runtime Polymorphism**: Switch backends at runtime
3. **Clean API**: Built-in convenience methods
4. **Test-Friendly**: Easy to mock different backends
5. **Type Safety**: Compile-time guarantees

## Migration Checklist

- [x] Add imports: `use crate::memory::{with_capsule_store, with_capsule_store_mut};`
- [x] Replace `with_capsules` with `with_capsule_store`
- [x] Replace direct HashMap operations with trait methods
- [x] Use convenience methods like `find_by_subject()` where appropriate
- [x] Test the migrated function
- [x] Update documentation

## Remaining Functions to Migrate

Based on grep analysis, these functions still use old patterns:

1. `capsules_create()` - Complex create/update logic
2. `capsule_read_self()` - Subject-based query
3. Various gallery and memory functions
4. Export/import functions in canister_factory

## Next Steps

1. **Migrate complex functions** - Handle read/write combinations
2. **Add convenience methods** - Extend trait for common patterns
3. **Update tests** - Use trait-based mocking
4. **Performance optimization** - Add secondary indexes if needed

## Testing Migration

After migrating a function, verify:

- ✅ Function compiles successfully
- ✅ All tests pass
- ✅ Functionality remains identical
- ✅ Performance is acceptable

## Rollback Plan

If issues arise:

1. The old `with_capsules` functions remain available
2. Can revert individual functions
3. Backward compatibility maintained via aliases

---

**Migration Progress**: 2/65+ functions completed
**Status**: 🟢 ACTIVE - Gradual migration in progress
