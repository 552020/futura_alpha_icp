# Decoupled Architecture for Memory Management

## Overview

This document explains the decoupled architecture implemented for memory management in our ICP application. The refactoring separates business logic from ICP-specific concerns, making the code more testable, maintainable, and reusable.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Before vs After: Function Refactoring](#before-vs-after-function-refactoring)
3. [Key Components](#key-components)
4. [Benefits](#benefits)
5. [Testing Strategy](#testing-strategy)
6. [Best Practices](#best-practices)

## Architecture Overview

### The Problem

The original architecture mixed business logic with ICP-specific code, making it difficult to:

- Write unit tests (ICP dependencies everywhere)
- Reuse business logic in different contexts
- Maintain and reason about the code
- Mock dependencies for testing

### The Solution

We implemented a **decoupled architecture** with three distinct layers:

```
┌─────────────────────────────────────────┐
│           Canister Layer                │
│  (memories.rs - Thin Wrappers)         │
│  • Candid serialization/deserialization │
│  • ICP environment access               │
│  • Delegates to core logic              │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│            Core Layer                   │
│  (memories_core.rs - Pure Logic)       │
│  • Business logic                       │
│  • No ICP dependencies                  │
│  • Uses traits for dependencies         │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│          Adapter Layer                  │
│  (CanisterEnv, StoreAdapter)           │
│  • Bridges core to ICP                  │
│  • Implements trait interfaces          │
│  • Handles ICP-specific operations      │
└─────────────────────────────────────────┘
```

## Before vs After: Function Refactoring

### Example: `memories_create` Function

#### ❌ **Before: Monolithic Approach**

```rust
// In memories.rs (1,224 lines - mixed concerns)
// Imports for the old monolithic approach:
use crate::memory::{with_capsule_store, with_capsule_store_mut};
use crate::capsule_store::CapsuleStore;
// This import makes 'store' available in the closure:
// with_capsule_store_mut(|store| { ... })

pub fn create_memory(
    capsule_id: CapsuleId,
    bytes: Option<Vec<u8>>,
    blob_ref: Option<BlobRef>,
    external_location: Option<StorageEdgeBlobType>,
    external_storage_key: Option<String>,
    external_url: Option<String>,
    external_size: Option<u64>,
    external_hash: Option<Vec<u8>>,
    asset_metadata: AssetMetadata,
    idem: String,
) -> Result<MemoryId> {
    let caller = PersonRef::from_caller(); // ❌ ICP dependency
    let now = ic_cdk::api::time();        // ❌ ICP dependency

    // ✅ Pure business logic validation (but ❌ mixed in same function with ICP concerns)
    let asset_count = bytes.is_some() as u8 + blob_ref.is_some() as u8 + external_location.is_some() as u8;
    if asset_count != 1 {
        return Err(Error::InvalidArgument(
            "Exactly one asset type must be provided".to_string(),
        ));
    }

    // ❌ Direct ICP storage access
    // store: &mut CapsuleStore (from with_capsule_store_mut)
    with_capsule_store_mut(|store| {
        // cap: &mut Capsule (from store.update_with)
        store.update_with(&capsule_id, |cap| {
            // ❌ Authorization logic mixed with business logic
            if !cap.owners.contains_key(&caller) && cap.subject != caller {
                return Err(Error::Unauthorized);
            }

            // ❌ Complex business logic inline
            let memory_id = generate_memory_id();
            let memory = create_memory_object(&memory_id, blob.clone(), asset_metadata.clone(), now);
            cap.memories.insert(memory_id.clone(), memory);
            cap.updated_at = now;

            Ok(memory_id)
        })
    })
}
```

**Problems:**

- ❌ **1,224 lines** of mixed concerns
- ❌ **ICP dependencies** scattered throughout
- ❌ **Hard to test** (can't mock `ic_cdk::api::time()`)
- ❌ **Business logic** tied to ICP environment
- ❌ **No reusability** outside canister context

#### ✅ **After: Decoupled Approach**

```rust
// In memories.rs (232 lines - thin wrappers only)
// Imports for the new decoupled approach:
use crate::memory::{with_capsule_store, with_capsule_store_mut};  // Still needed for StoreAdapter
use crate::capsule_store::CapsuleStore;                           // Still needed for StoreAdapter
use crate::types::{Error, MemoryId, /* ... */};
// The 'store' parameter comes from the StoreAdapter implementation

pub fn memories_create(
    capsule_id: String,
    bytes: Option<Vec<u8>>,
    blob_ref: Option<BlobRef>,
    external_location: Option<StorageEdgeBlobType>,
    external_storage_key: Option<String>,
    external_url: Option<String>,
    external_size: Option<u64>,
    external_hash: Option<Vec<u8>>,
    asset_metadata: AssetMetadata,
    idem: String,
) -> std::result::Result<MemoryId, Error> {
    let env = CanisterEnv;           // ✅ ICP environment adapter
    let mut store = StoreAdapter;    // ✅ Storage adapter (implements Store trait)

    // ✅ Delegate to pure core logic
    crate::memories_core::memories_create_core(
        &env,                        // env: &dyn Env
        &mut store,                  // store: &mut dyn Store
        capsule_id,
        bytes,
        blob_ref,
        external_location,
        external_storage_key,
        external_url,
        external_size,
        external_hash,
        asset_metadata,
        idem,
    )
}
```

```rust
// In memories_core.rs (pure business logic)
// Imports for the core layer:
use crate::types::{Error, MemoryId, /* ... */};

pub fn memories_create_core(
    env: &dyn Env,                    // ✅ Trait for environment (from memories_core::Env)
    store: &mut dyn Store,            // ✅ Trait for storage (from memories_core::Store)
    capsule_id: String,
    bytes: Option<Vec<u8>>,
    blob_ref: Option<BlobRef>,
    external_location: Option<StorageEdgeBlobType>,
    external_storage_key: Option<String>,
    external_url: Option<String>,
    external_size: Option<u64>,
    external_hash: Option<Vec<u8>>,
    asset_metadata: AssetMetadata,
    idem: String,
) -> std::result::Result<MemoryId, Error> {
    let caller = env.caller();        // ✅ Trait method
    let now = env.now();              // ✅ Trait method

    // ✅ Pure business logic validation
    let asset_count = bytes.is_some() as u8 + blob_ref.is_some() as u8 + external_location.is_some() as u8;
    if asset_count != 1 {
        return Err(Error::InvalidArgument(
            "Exactly one asset type must be provided".to_string(),
        ));
    }

    // ✅ Asset consistency validation
    validate_asset_consistency(&bytes, &blob_ref, &external_location, &external_size, &external_hash, &asset_metadata)?;

    // ✅ Get accessible capsules through trait
    // store.get_accessible_capsules() calls StoreAdapter which calls with_capsule_store()
    let accessible_capsules = store.get_accessible_capsules(&caller);
    if !accessible_capsules.contains(&capsule_id) {
        return Err(Error::Unauthorized);
    }

    // ✅ Create memory through trait
    let memory = create_memory_from_assets(
        &asset_metadata,
        bytes,
        blob_ref,
        external_location,
        external_storage_key,
        external_url,
        external_size,
        external_hash,
        now,
        &caller,
    )?;

    // ✅ Store through trait
    // store.insert_memory() calls StoreAdapter which calls with_capsule_store_mut()
    store.insert_memory(&capsule_id, memory.clone())?;

    Ok(memory.id)
}
```

**Benefits:**

- ✅ **232 lines** (81% reduction)
- ✅ **Pure business logic** in core
- ✅ **Easy to test** (mock traits)
- ✅ **Reusable** in different contexts
- ✅ **Clear separation** of concerns

## Key Components

### 1. **Core Layer** (`memories_core.rs`)

**Purpose**: Contains pure business logic with no external dependencies.

**Key Features**:

- Uses trait interfaces for dependencies
- No `ic_cdk` imports
- Fully testable with mocks
- Reusable across different contexts

**Example Traits**:

```rust
pub trait Env {
    fn caller(&self) -> PersonRef;
    fn now(&self) -> u64;
}

pub trait Store {
    fn insert_memory(&mut self, capsule: &CapsuleId, memory: Memory) -> std::result::Result<(), Error>;
    fn get_memory(&self, capsule: &CapsuleId, id: &MemoryId) -> Option<Memory>;
    fn delete_memory(&mut self, capsule: &CapsuleId, id: &MemoryId) -> std::result::Result<(), Error>;
    fn get_accessible_capsules(&self, caller: &PersonRef) -> Vec<CapsuleId>;
}
```

### 2. **Canister Layer** (`memories.rs`)

**Purpose**: Thin wrappers that handle Candid serialization and delegate to core.

**Key Features**:

- Minimal code (just delegation)
- Handles Candid encoding/decoding
- Creates adapter instances
- No business logic

### 3. **Adapter Layer** (`CanisterEnv`, `StoreAdapter`)

**Purpose**: Bridges between core logic and ICP-specific implementations.

**Key Features**:

- Implements trait interfaces
- Handles ICP-specific operations
- Isolates ICP dependencies

**Example Adapter**:

```rust
pub struct CanisterEnv;

impl crate::memories_core::Env for CanisterEnv {
    fn caller(&self) -> PersonRef {
        PersonRef::Principal(ic_cdk::api::msg_caller()) // ✅ ICP dependency isolated
    }

    fn now(&self) -> u64 {
        ic_cdk::api::time() // ✅ ICP dependency isolated
    }
}
```

## Benefits

### 1. **Testability** 🧪

**Before**: Impossible to unit test

```rust
// ❌ Can't test - depends on ic_cdk::api::time()
fn test_create_memory() {
    // This will fail in unit tests
    let result = create_memory(/* ... */);
}
```

**After**: Easy to unit test

```rust
// ✅ Easy to test with mocks
fn test_create_memory_core() {
    let mut mock_env = MockEnv::new();
    let mut mock_store = MockStore::new();

    mock_env.expect_caller().returning(|| test_caller());
    mock_store.expect_insert_memory().returning(|_, _| Ok(()));

    let result = memories_create_core(&mock_env, &mut mock_store, /* ... */);
    assert!(result.is_ok());
}
```

### 2. **Maintainability** 🔧

- **Single Responsibility**: Each layer has one clear purpose
- **Easier Debugging**: Issues are isolated to specific layers
- **Clear Dependencies**: Explicit trait interfaces show what's needed

### 3. **Reusability** ♻️

- **Core logic** can be used in different contexts (CLI tools, web services, etc.)
- **Adapters** can be swapped for different implementations
- **Traits** enable dependency injection

### 4. **Code Size Reduction** 📦

- **81% reduction** in `memories.rs` (1,224 → 232 lines)
- **Cleaner code** with focused responsibilities
- **Better readability** and understanding

## Testing Strategy

### Unit Tests (Core Layer)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::memories_core::{Env, Store};

    struct MockEnv {
        caller: PersonRef,
        now: u64,
    }

    impl Env for MockEnv {
        fn caller(&self) -> PersonRef { self.caller.clone() }
        fn now(&self) -> u64 { self.now }
    }

    #[test]
    fn test_memories_create_core_success() {
        let env = MockEnv { caller: test_caller(), now: 1000 };
        let mut store = InMemoryStore::new();

        let result = memories_create_core(&env, &mut store, /* ... */);
        assert!(result.is_ok());
    }
}
```

### Integration Tests (PocketIC)

```rust
#[test]
fn test_memories_create_integration() {
    let (mut pic, canister_id, _wasm) = create_test_canister();
    let controller = Principal::from_slice(&[1; 29]);

    // Test the full canister function
    let result = pic.update_call(canister_id, controller, "memories_create", /* ... */);
    assert!(result.is_ok());
}
```

## Best Practices

### 1. **Keep Core Pure**

- No `ic_cdk` imports in core
- Use traits for all external dependencies
- Make functions deterministic and testable

### 2. **Thin Wrappers**

- Canister functions should only handle serialization
- Delegate immediately to core functions
- Keep wrapper code minimal

### 3. **Clear Interfaces**

- Use descriptive trait names
- Keep trait methods focused
- Document trait contracts

### 4. **Consistent Error Handling**

- Use `std::result::Result<T, Error>` consistently
- Avoid custom type aliases that cause conflicts
- Map errors appropriately between layers

## Conclusion

The decoupled architecture provides significant benefits:

- ✅ **Better Testability**: Core logic can be unit tested
- ✅ **Improved Maintainability**: Clear separation of concerns
- ✅ **Enhanced Reusability**: Business logic can be used in different contexts
- ✅ **Reduced Complexity**: 81% code reduction in canister layer
- ✅ **Type Safety**: Consistent error handling with standard Rust types

This architecture makes the codebase more robust, maintainable, and easier to reason about while maintaining full backward compatibility for canister function signatures.
