# Issue 9: Single Mutable Lock for CapsuleStore Updates ✅ COMPLETED

## Problem (RESOLVED)

Previously, capsule mutations used the `update` API on `CapsuleStore`:

```rust
store.update(&capsule_id, |cap| {
    if !cap.owners.contains_key(&caller) && cap.subject != caller {
        return; // early return
    }
    if cap.inline_bytes_used.saturating_add(len) > CAPSULE_INLINE_BUDGET {
        return; // another silent early return
    }
    // ...
});
```

**Problems with this pattern (NOW FIXED):**

1. **Silent early returns** ✅ FIXED
   Errors inside the closure don't propagate back to the caller. Returning from the closure just stops mutation, but the API always reports success.

2. **No return value** ✅ FIXED
   `update` has a void closure signature, so you can't directly return a `Result<MemoryId, Error>` from inside the closure. Instead you're forced to capture a mutable `out` variable outside, which is error-prone and unreadable.

3. **Inconsistent semantics** ✅ FIXED
   Some mutations succeed silently, others bail silently. Debugging and testing is harder because errors don't bubble up properly.

4. **Cross-backend compatibility** ✅ FIXED
   Adding a `get_mut` trait method isn't portable to stable structures (like `StableBTreeMap`), because you can't hand out a real `&mut` reference to data in stable memory. But we still want the "single mutable lock" semantics.

## Goal ✅ ACHIEVED

- Hold one mutable capsule handle (Hash backend: `&mut Capsule`; Stable backend: owned `Capsule` copy). ✅
- Perform all checks and mutations under that one lock. ✅
- Propagate errors and return values (`Result<R, Error>`) directly, without side-effects or silent returns. ✅
- Keep the API portable across backends. ✅

## Implemented Solution

### 1. ✅ Added `update_with` to `CapsuleStore`

Instead of `update` (void closure), introduced `update_with` that lets the closure return a `Result<R, Error>`.

**Actual Implementation:**

```rust
pub trait CapsuleStore {
    fn get(&self, id: &CapsuleId) -> Option<Capsule>;

    /// Update a capsule with a closure that returns a result
    ///
    /// This method allows the closure to return a `Result<R, Error>`, enabling
    /// proper error propagation from within the update operation. This eliminates
    /// the need for silent early returns and provides better error handling.
    ///
    /// Returns the result from the closure, or UpdateError if the capsule wasn't found.
    fn update_with<R, F>(&mut self, id: &CapsuleId, f: F) -> Result<R, UpdateError>
    where
        F: FnOnce(&mut crate::types::Capsule) -> Result<R, crate::types::Error>;
}
```

**Error Conversion Support:**

```rust
impl From<crate::types::Error> for UpdateError {
    fn from(err: crate::types::Error) -> Self {
        match err {
            crate::types::Error::NotFound => UpdateError::NotFound,
            crate::types::Error::Unauthorized => {
                UpdateError::Validation("unauthorized".to_string())
            }
            crate::types::Error::InvalidArgument(msg) => UpdateError::Validation(msg),
            crate::types::Error::Conflict(msg) => UpdateError::Validation(msg),
            crate::types::Error::ResourceExhausted => {
                UpdateError::Validation("resource exhausted".to_string())
            }
            crate::types::Error::Internal(msg) => UpdateError::Validation(msg),
        }
    }
}
```

### 2. ✅ Implemented for each backend

#### Hash backend ✅

**Actual Implementation:**

```rust
impl CapsuleStore for HashStore {
    fn update_with<R, F>(&mut self, id: &CapsuleId, f: F) -> Result<R, UpdateError>
    where
        F: FnOnce(&mut Capsule) -> Result<R, crate::types::Error>,
    {
        if let Some(capsule) = self.capsules.get_mut(id) {
            let old_subject = capsule.subject.principal().cloned();
            let old_owners: Vec<_> = capsule
                .owners
                .keys()
                .filter_map(Self::principal_from_person_ref)
                .cloned()
                .collect();

            let result = f(capsule)?;

            // Update indexes if subject or owners changed
            let new_subject = capsule.subject.principal().cloned();
            let new_owners: Vec<_> = capsule
                .owners
                .keys()
                .filter_map(Self::principal_from_person_ref)
                .cloned()
                .collect();

            // Update subject index if changed
            if old_subject != new_subject {
                if let Some(old_principal) = old_subject {
                    self.subject_index.remove(&old_principal);
                }
                if let Some(new_principal) = &new_subject {
                    self.subject_index.insert(*new_principal, id.clone());
                }
            }

            // Update owner index for added/removed owners
            for owner in &old_owners {
                if !new_owners.contains(owner) {
                    if let Some(owner_capsules) = self.owner_index.get_mut(owner) {
                        owner_capsules.retain(|capsule_id| capsule_id != id);
                        if owner_capsules.is_empty() {
                            self.owner_index.remove(owner);
                        }
                    }
                }
            }
            for owner in &new_owners {
                if !old_owners.contains(owner) {
                    self.owner_index.entry(*owner).or_default().push(id.clone());
                }
            }

            Ok(result)
        } else {
            Err(UpdateError::NotFound)
        }
    }
}
```

#### Stable backend ✅

**Actual Implementation:**

```rust
impl CapsuleStore for StableStore {
    fn update_with<R, F>(&mut self, id: &CapsuleId, f: F) -> Result<R, UpdateError>
    where
        F: FnOnce(&mut Capsule) -> Result<R, crate::types::Error>,
    {
        if let Some(mut capsule) = self.capsules.get(id) {
            let old_subject = capsule.subject.principal().cloned();
            let old_owners: Vec<_> = capsule
                .owners
                .keys()
                .filter_map(|person_ref| match person_ref {
                    crate::types::PersonRef::Principal(p) => Some(*p),
                    crate::types::PersonRef::Opaque(_) => None,
                })
                .collect();

            let result = f(&mut capsule)?;

            // Update indexes if subject or owners changed
            let new_subject = capsule.subject.principal().cloned();
            let new_owners: Vec<_> = capsule
                .owners
                .keys()
                .filter_map(|person_ref| match person_ref {
                    crate::types::PersonRef::Principal(p) => Some(*p),
                    crate::types::PersonRef::Opaque(_) => None,
                })
                .collect();

            // Update subject index if changed
            if old_subject != new_subject {
                if let Some(old_principal) = old_subject {
                    let old_key = old_principal.as_slice().to_vec();
                    self.subject_index.remove(&old_key);
                }
                if let Some(new_principal) = &new_subject {
                    let new_key = new_principal.as_slice().to_vec();
                    self.subject_index.insert(new_key, id.clone());
                }
            }

            // Update owner index for added/removed owners (sparse multimap)
            // Remove old owner relationships
            for owner in &old_owners {
                if !new_owners.contains(owner) {
                    let owner_key = owner.as_slice().to_vec();
                    let key = OwnerIndexKey::new(owner_key, id.clone());
                    self.owner_index.remove(&key);
                }
            }
            // Add new owner relationships
            for owner in &new_owners {
                if !old_owners.contains(owner) {
                    let owner_key = owner.as_slice().to_vec();
                    let key = OwnerIndexKey::new(owner_key, id.clone());
                    self.owner_index.insert(key, ());
                }
            }

            // Save the updated capsule
            self.capsules.insert(id.clone(), capsule);
            Ok(result)
        } else {
            Err(UpdateError::NotFound)
        }
    }
}
```

#### Store enum ✅

**Actual Implementation:**

```rust
impl CapsuleStore for Store {
    fn update_with<R, F>(&mut self, id: &CapsuleId, f: F) -> Result<R, UpdateError>
    where
        F: FnOnce(&mut Capsule) -> Result<R, crate::types::Error>,
    {
        match self {
            Store::Hash(store) => store.update_with(id, f),
            Store::Stable(store) => store.update_with(id, f),
        }
    }
}
```

### 3. ✅ Using `update_with` in domain logic

**Old pattern (silent early returns) - REPLACED:**

```rust
store.update(&capsule_id, |cap| {
    if !cap.owners.contains_key(&caller) && cap.subject != caller {
        return; // silent
    }
    if cap.inline_bytes_used.saturating_add(len) > CAPSULE_INLINE_BUDGET {
        return; // silent
    }
    // ...
});
```

**New pattern (proper error propagation) - IMPLEMENTED:**

```rust
// Use update_with for proper error propagation
store.update_with(&capsule_id, |cap| {
    // Check authorization
    if !cap.owners.contains_key(&caller) && cap.subject != caller {
        return Err(crate::types::Error::Unauthorized);
    }

    // Check budget (use maintained counter)
    let used = cap.inline_bytes_used;
    if used.saturating_add(len_u64) > CAPSULE_INLINE_BUDGET as u64 {
        return Err(crate::types::Error::ResourceExhausted);
    }

    // Pre-generate the ID
    let memory_id = generate_memory_id();
    let now = ic_cdk::api::time();

    // Use the BlobRef directly (no conversion needed)
    let types_blob = blob.clone();

    // Insert the memory
    cap.insert_memory(
        &memory_id,
        types_blob.clone(),
        meta.clone(),
        now,
        Some(idem.clone()),
    )
    .map_err(|e| crate::types::Error::Internal(format!("insert_memory: {:?}", e)))?;

    // Maintain inline budget counter when the blob originated as inline
    if blob.locator.starts_with("inline_") {
        cap.inline_bytes_used = cap.inline_bytes_used.saturating_add(blob.len);
    }

    cap.updated_at = now;

    // Return the generated ID
    Ok(memory_id)
})
.map_err(|e| match e {
    crate::capsule_store::UpdateError::NotFound => crate::types::Error::NotFound,
    crate::capsule_store::UpdateError::Validation(msg) => crate::types::Error::InvalidArgument(msg),
    crate::capsule_store::UpdateError::Concurrency => crate::types::Error::Internal("concurrency error".into()),
})
```

**This implementation:**

✅ Holds one mutable capsule handle  
✅ Checks all invariants  
✅ Returns early with proper errors  
✅ Produces the new or existing `MemoryId` cleanly  
✅ Proper error propagation throughout the call chain

## Benefits

- **Correct error propagation** – no more silent failures
- **Cleaner code** – no `out` captures, no `return;` hacks
- **Single lock** – ensures atomic capsule updates
- **Backend portability** – works on HashMap and Stable backends
- **Standard Rust feel** – closure returning a `Result` is natural

## Open Questions

1. Should we **deprecate the old `update`** method to force migration to `update_with`?
2. Should we add a **convenience `get_mut` only on HashStore** (non-trait) for testing/dev ergonomics?
3. Do we need to add **transaction semantics** later (rollbacks for partial failures)?

## Implementation Locations

The method would need to be implemented in:

1. **Trait Definition**: `src/backend/src/capsule_store/mod.rs` (line ~60-100)
2. **Store Enum**: `src/backend/src/capsule_store/store.rs` (line ~50-150)
3. **Hash Backend**: `src/backend/src/capsule_store/hash.rs`
4. **Stable Backend**: `src/backend/src/capsule_store/stable.rs`

## Related Context

This is part of implementing Fix #9 from the memory creation implementation:

- **Fix #9**: Do everything under one mutable lock - use `update_with` and return `Result` directly instead of silent early returns in update closure

The goal is to eliminate silent early returns that don't propagate errors and make the code more maintainable and debuggable.
