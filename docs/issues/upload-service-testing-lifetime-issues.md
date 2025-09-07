# Upload Service Testing Lifetime Issues

**Status**: OPEN  
**Priority**: MEDIUM  
**Category**: Testing Infrastructure  
**Assigned**: TBD  
**Created**: 2024-12-19

## Problem Description

The `UploadService` struct cannot be easily unit tested due to Rust lifetime constraints. The service holds a mutable reference to a `Store` instance, which creates borrowing conflicts in test scenarios.

## Root Cause Analysis

### The Lifetime Problem

The `UploadService` struct is defined as:

```rust
pub struct UploadService<'a> {
    store: &'a mut Store,
    sessions: SessionStore,
    blob_store: BlobStore,
}
```

This creates a lifetime parameter `'a` that ties the service to the lifetime of the `Store` reference. In test scenarios, this leads to several issues:

1. **Cannot return both service and store from test helpers**
2. **Cannot store service in test fixtures**
3. **Cannot use service across multiple test functions**
4. **Borrow checker prevents mutable access patterns needed for testing**

## Practical Examples of Failures

### Example 1: Test Helper Function Failure

```rust
// This FAILS to compile
fn create_test_upload_service() -> (UploadService, Store) {
    let mut store = create_test_store();
    let service = UploadService::new(&mut store); // ❌ Cannot return both
    (service, store) // ❌ Borrow checker error
}
```

**Error**: `cannot return value referencing local variable 'store'`

### Example 2: Test Fixture Failure

```rust
// This FAILS to compile
struct TestFixture {
    service: UploadService, // ❌ Cannot store without lifetime parameter
    store: Store,
}

impl TestFixture {
    fn new() -> Self {
        let mut store = create_test_store();
        let service = UploadService::new(&mut store); // ❌ Lifetime mismatch
        Self { service, store }
    }
}
```

**Error**: `missing lifetime specifier` and `lifetime mismatch`

### Example 3: Multiple Test Function Failure

```rust
// This FAILS to compile
#[test]
fn test_begin_upload() {
    let mut store = create_test_store();
    let mut service = UploadService::new(&mut store);
    // ... test logic
}

#[test]
fn test_put_chunk() {
    let mut store = create_test_store();
    let mut service = UploadService::new(&mut store);
    // ... test logic
}
```

**Error**: Each test needs its own service instance, but cannot share state between tests.

### Example 4: Integration Test Failure

```rust
// This FAILS to compile
#[test]
fn test_complete_upload_workflow() {
    let mut store = create_test_store();
    let mut service = UploadService::new(&mut store);

    // Step 1: Begin upload
    let session_id = service.begin_upload(/* ... */).unwrap();

    // Step 2: Put chunks
    service.put_chunk(&session_id, 0, chunk_data).unwrap();

    // Step 3: Finish upload
    let memory_id = service.commit(&session_id, /* ... */).unwrap();

    // Step 4: Verify result
    // ❌ Cannot access store to verify the memory was created
    // ❌ Cannot access store to verify session was cleaned up
}
```

**Error**: Cannot access `store` after service creation due to borrowing rules.

## Current Workaround

All integration tests and most unit tests are currently commented out in:

- `src/backend/src/upload/service.rs` (unit tests)
- `src/backend/src/upload/integration_tests.rs` (integration tests)

Only basic utility tests remain active that don't require `UploadService` instances.

## Proposed Solutions

### Option 1: Dependency Injection with Trait Objects

Create a trait for the store operations and inject it into `UploadService`:

```rust
pub trait StoreOps {
    fn get_capsule(&self, id: &str) -> Option<Capsule>;
    fn update_capsule(&mut self, id: &str, f: impl FnOnce(&mut Capsule));
    // ... other operations
}

pub struct UploadService<S: StoreOps> {
    store: S,
    sessions: SessionStore,
    blob_store: BlobStore,
}
```

**Pros**: Testable with mock implementations
**Cons**: Significant refactoring required, not standard pattern for IC canisters

### Option 2: Global Test Store (IC Best Practice)

Use a global, thread-local store for testing. This is **explicitly recommended** for IC Rust canisters:

```rust
thread_local! {
    static TEST_STORE: RefCell<Store> = RefCell::new(create_test_store());
}

pub struct UploadService {
    sessions: SessionStore,
    blob_store: BlobStore,
}

impl UploadService {
    fn new() -> Self {
        Self {
            sessions: SessionStore::new(),
            blob_store: BlobStore::new(),
        }
    }

    fn with_store<F, R>(&self, f: F) -> R
    where F: FnOnce(&mut Store) -> R {
        TEST_STORE.with(|store| f(&mut store.borrow_mut()))
    }
}
```

**Pros**:

- **IC Best Practice**: Standard pattern for IC canister state management
- **Minimal refactoring**: Existing `UploadService` code can remain largely unchanged
- **Quick implementation**: Can be implemented in a few hours
- **Test isolation**: Each test can reset the global store state
- **Maintains existing API**: No changes to public interface
- **Safe**: Avoids memory corruption and asynchrony issues

**Cons**: Global state (but this is the recommended pattern for IC)

### Option 3: Test-Specific Service Builder

Create a test-specific service that doesn't hold store references:

```rust
pub struct TestUploadService {
    sessions: SessionStore,
    blob_store: BlobStore,
}

impl TestUploadService {
    pub fn new() -> Self {
        Self {
            sessions: SessionStore::new(),
            blob_store: BlobStore::new(),
        }
    }

    pub fn begin_upload(&mut self, store: &mut Store, /* ... */) -> Result<SessionId, Error> {
        // Implementation that takes store as parameter
    }

    pub fn put_chunk(&mut self, store: &mut Store, /* ... */) -> Result<(), Error> {
        // Implementation that takes store as parameter
    }

    pub fn commit(&mut self, store: &mut Store, /* ... */) -> Result<MemoryId, Error> {
        // Implementation that takes store as parameter
    }
}
```

**Pros**: Testable, clear separation of concerns
**Cons**: Duplicate implementation, maintenance overhead

## IC Expert Assessment

Based on IC Rust canister best practices:

### Why `thread_local!` is Recommended for Canister State

The use of `thread_local!` with `RefCell` for global state is the **standard and safest way** to manage mutable canister state on the Internet Computer. This pattern is not only used in production canisters but is also considered best practice for testability and safety:

> Use `thread_local!` with `Cell/RefCell` for state variables... This option is the safest. It will help you avoid memory corruption and issues with asynchrony.  
> — [Effective Rust canisters: Canister state](https://mmapped.blog/posts/01-effective-rust-canisters#canister-state)

### Test Isolation

Test isolation is critical and straightforward with this approach:

> For the parts that still interact with the system API, create a thin abstraction of the system API that is faked in unit tests...  
> — [Security best practices: Miscellaneous](https://internetcomputer.org/docs/building-apps/security/misc#test-your-canister-code-even-in-the-presence-of-system-api-calls)

### Why Option 1 is Not Standard for IC

While trait-based dependency injection is a classic Rust approach for testability, it is not the standard pattern for IC canisters, where global state is the norm due to the single-threaded, event-driven execution model.

## Senior Developer Assessment

### The Stateless Service Approach (Recommended)

**Short version**: The IC expert's advice is fine for **canister state**, but for **testing `UploadService`** the senior developer prefers making the service **stateless** (no `&'a mut Store` field; pass `&mut Store` into methods). That gives you clean tests without globals, while staying fully idiomatic for IC.

### Why the Stateless Service is Nicer for Tests

- Change `UploadService<'a> { store: &'a mut Store, … }` → `UploadService { … }` and accept `&mut impl CapsuleStore` (or `&mut Store`) **per method**.
- In tests you own both `store` and `service`, call methods in sequence, then assert on `store`. No lifetime wrangling, no globals, no resets.
- In production endpoints you still do:

  ```rust
  with_capsule_store_mut(|store| {
      let mut svc = UploadService::new();
      svc.begin_upload(&mut *store, …)
  })
  ```

  It composes perfectly with your existing `thread_local!` canister state.

### Why Not Fully Sold on "Global Test Store"

- `thread_local! + RefCell` is the **right pattern for production canister state** on IC (you already use it for `MM`, sessions, blobs, etc.).
- Using another `thread_local! TEST_STORE` to make tests compile works, but you now need reliable **reset hooks** between tests and you must remember that Rust runs tests in parallel (thread-local means each test thread gets its own copy—good for isolation, but surprising if you expected sharing).

### If You Really Want Option 2 (Global Test Store)

Totally viable—just:

- Add a `test_reset()` helper that clears sessions, chunks, blobs, and the store.
- Call it at the start of every test.
- Be aware that parallel test threads each get their own thread-local state (usually fine).
- Keep an eye on any usage of `ic_cdk::api::msg_caller()` in unit tests—wrap it or feature-gate so tests can set a fake caller.

## Recommendation

**The Stateless Service Approach is strongly recommended** for the following reasons:

1. **Clean Tests**: No lifetime wrangling, no globals, no resets needed
2. **IC Idiomatic**: Composes perfectly with existing `thread_local!` canister state
3. **Minimal Refactoring**: ~15–30 lines of signature/field changes
4. **Simple and Robust**: You own both `store` and `service` in tests
5. **Production Ready**: Works seamlessly with existing endpoint patterns
6. **No Global State**: Avoids the complexity of test isolation and reset hooks

## Practical Implementation Pattern

### Recommended: Stateless Service Approach

Below is the recommended pattern showing how to make `UploadService` stateless by passing `&mut Store` into methods:

```rust
// Your real store
#[derive(Default)]
pub struct Store {
    pub counter: u64,
    // ... other fields
}

// Stateless service - no lifetime parameters, no store field
pub struct UploadService {
    sessions: SessionStore,
    blob_store: BlobStore,
    // other fields that don't require borrowing the store
}

impl UploadService {
    pub fn new() -> Self {
        Self {
            sessions: SessionStore::new(),
            blob_store: BlobStore::new(),
        }
    }

    // Service methods accept &mut Store as parameter
    pub fn begin_upload(&mut self, store: &mut Store, /* ... */) -> Result<SessionId, Error> {
        // ... do work with store
        store.counter += 1;
        Ok(SessionId::new())
    }

    pub fn put_chunk(&mut self, store: &mut Store, /* ... */) -> Result<(), Error> {
        // ... do work with store
        store.counter += 1;
        Ok(())
    }

    pub fn commit(&mut self, store: &mut Store, /* ... */) -> Result<MemoryId, Error> {
        // ... do work with store
        store.counter += 1;
        Ok(MemoryId::new())
    }
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_begin_upload_increments_counter() {
        let mut store = Store::default();
        let mut svc = UploadService::new();

        let _session = svc.begin_upload(&mut store, /* ... */).unwrap();
        assert_eq!(store.counter, 1);

        let _session2 = svc.begin_upload(&mut store, /* ... */).unwrap();
        assert_eq!(store.counter, 2);
    }

    #[test]
    fn test_full_workflow() {
        let mut store = Store::default();
        let mut svc = UploadService::new();

        let session = svc.begin_upload(&mut store, /* ... */).unwrap();
        svc.put_chunk(&mut store, &session, /* ... */).unwrap();
        let memory_id = svc.commit(&mut store, &session, /* ... */).unwrap();

        // Verify post-conditions by reading store
        assert_eq!(store.counter, 3);
        // ... other assertions
    }
}

// ---- Production Usage ----

// In your canister endpoints, you still use thread_local! for canister state
thread_local! {
    static CANISTER_STORE: RefCell<Store> = RefCell::new(Store::default());
}

fn with_capsule_store_mut<F, R>(f: F) -> R
where F: FnOnce(&mut Store) -> R {
    CANISTER_STORE.with(|cell| f(&mut cell.borrow_mut()))
}

// Endpoint implementation
pub fn uploads_begin_upload(/* ... */) -> Result<SessionId, Error> {
    with_capsule_store_mut(|store| {
        let mut svc = UploadService::new();
        svc.begin_upload(store, /* ... */)
    })
}
```

### Alternative: Global Test Store Approach

If you prefer the global test store approach, here's how it would work:

```rust
use std::cell::RefCell;

// Thread-local global store used in tests
thread_local! {
    static TEST_STORE: RefCell<Store> = RefCell::new(Store::default());
}

// Helper used by service methods to get &mut Store without storing a reference
fn with_test_store<R>(f: impl FnOnce(&mut Store) -> R) -> R {
    TEST_STORE.with(|cell| f(&mut cell.borrow_mut()))
}

// Reset the global store before each test to ensure isolation
fn reset_store() {
    TEST_STORE.with(|cell| *cell.borrow_mut() = Store::default());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_begin_upload_increments_counter() {
        reset_store();
        let mut svc = UploadService::new();

        let c1 = svc.begin_upload(with_test_store);
        assert_eq!(c1, 1);

        let c2 = svc.begin_upload(with_test_store);
        assert_eq!(c2, 2);
    }
}
```

### Why the Stateless Approach Works Better

- **Clean Tests**: No lifetime wrangling, no globals, no resets needed
- **Simple Ownership**: You own both `store` and `service` in tests
- **IC Idiomatic**: Composes perfectly with existing `thread_local!` canister state
- **Minimal Refactoring**: ~15–30 lines of signature/field changes
- **Production Ready**: Works seamlessly with existing endpoint patterns

## Implementation Plan

### Recommended Approach: Stateless Service

1. **Phase 1**: Refactor `UploadService` to be stateless (~15-30 lines of changes)
   - Remove `store: &'a mut Store` field from struct
   - Add `store: &mut Store` parameter to all service methods
   - Update method signatures to accept store as parameter
2. **Phase 2**: Uncomment and fix existing tests
3. **Phase 3**: Add comprehensive test coverage
4. **Phase 4**: Verify production endpoints work with new stateless service

### Alternative Approach: Global Test Store

1. **Phase 1**: Implement global test store approach using `thread_local!`
2. **Phase 2**: Add `test_reset()` helper and call it at start of each test
3. **Phase 3**: Uncomment and fix existing tests
4. **Phase 4**: Add comprehensive test coverage
5. **Phase 5**: Handle `ic_cdk::api::msg_caller()` in tests (wrap or feature-gate)

## Acceptance Criteria

- [ ] All existing commented tests compile and run
- [ ] Integration tests can verify complete upload workflows
- [ ] Unit tests can test individual service methods
- [ ] Tests can verify store state changes
- [ ] Test isolation is maintained between test runs
- [ ] No performance impact on production code

## Related Files

- `src/backend/src/upload/service.rs` - Main service implementation
- `src/backend/src/upload/integration_tests.rs` - Integration tests (commented out)
- `src/backend/src/upload/types.rs` - Service types and constants
- `src/backend/src/upload/sessions.rs` - Session management
- `src/backend/src/upload/blob_store.rs` - Blob storage

## Notes

This issue blocks comprehensive testing of the upload service, which is critical for ensuring reliability of the file upload functionality. The current workaround of commenting out tests is not sustainable for long-term development.
