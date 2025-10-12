# HTTP Module Architecture Analysis

**Date:** December 2024  
**To:** Tech Lead  
**From:** Development Team  
**Subject:** HTTP Module Implementation vs CQRS/Decoupled Architecture Principles

## Executive Summary

Our newly implemented HTTP module for token-gated asset serving **partially follows** the established CQRS and decoupled architecture principles, but requires refactoring to fully align with our architectural standards. The current implementation mixes concerns and lacks proper separation between canister, core, and adapter layers.

## Current Implementation Analysis

### ✅ **What We Did Right**

1. **Modern Rust Structure**

   - No `mod.rs` files (modern Rust approach)
   - Clean module organization with logical separation
   - Proper dependency management

2. **Basic Separation of Concerns**

   - `request.rs` - HTTP request parsing
   - `response.rs` - HTTP response helpers
   - `auth.rs` - Token authentication logic
   - `secret.rs` - Secret management
   - `routes/` - Route handlers

3. **Integration with Canister Lifecycle**
   - Proper `init` and `post_upgrade` integration
   - HTTP request handler exposed as canister function

### ❌ **Architectural Violations**

#### 1. **Missing CQRS Separation**

**Current Structure:**

```
src/http/
├── auth.rs          # Mixed: token generation + verification
├── secret.rs        # Mixed: storage + business logic
├── routes/
│   ├── assets.rs    # Mixed: auth + business logic + ICP
│   └── health.rs    # Simple (OK)
└── response.rs      # Pure helpers (OK)
```

**CQRS Violations:**

- `auth.rs` contains both **Commands** (token generation) and **Queries** (token verification)
- `routes/assets.rs` mixes read operations with business logic
- No clear separation between write operations (token minting) and read operations (token verification)

#### 2. **Missing Decoupled Architecture**

**Current Implementation:**

```rust
// ❌ VIOLATION: Direct ICP dependencies in business logic
// In auth.rs
use ic_cdk::api::time;
use crate::http::secret::get_current_secret;

pub fn sign_token(p: &TokenPayload) -> EncodedToken {
    let sec = get_current_secret().expect("Secret not initialized"); // ❌ ICP dependency
    // ... business logic mixed with ICP concerns
}

// In secret.rs
use ic_cdk::api::management_canister::main::raw_rand; // ❌ ICP dependency
use std::sync::Mutex;

pub async fn init_secret() -> Result<(), String> {
    let secret = generate_hmac_secret().await?; // ❌ ICP dependency
    // ... business logic mixed with ICP concerns
}
```

**Problems:**

- ❌ **ICP dependencies** scattered throughout business logic
- ❌ **Hard to test** (can't mock `ic_cdk::api::time()`)
- ❌ **No reusability** outside canister context
- ❌ **Mixed concerns** in single functions

## Recommended Refactoring

### 1. **Tech Lead's Recommended Approach (Simplified)**

**Target Structure:**

```
src/http.rs                  // router + entry points
src/http/core/
  auth_core.rs               // pure: sign/verify token
  secret_core.rs             // pure: key lifecycle API (trait)
  path_core.rs               // parse/validate path/variant/scope (pure)
  types.rs                   // TokenPayload/Scope/etc (pure)
src/http/adapters/
  canister_env.rs            // now(), caller(), time source
  secret_store.rs            // stable secret get/rotate (ICP deps)
  asset_store.rs             // reads images via existing memories/upload APIs
src/http/routes/
  assets.rs                  // thin: parse→verify→serve
  health.rs
```

**Trait Design (Dependency Inversion):**

```rust
// core/types.rs
pub trait Clock {
    fn now_ns(&self) -> u64;
}

pub trait SecretStore {
    fn get_key(&self) -> [u8;32];
    fn rotate_key(&mut self) -> Result<(), String>;
}

pub trait AssetStore {
    fn get_inline(&self, mem: &str, id: &str) -> Option<(Vec<u8>, String)>;
    fn get_blob_len(&self, mem: &str, id: &str) -> Option<(u64, String)>;
    fn stream_blob(&self, mem: &str, id: &str, offset: u64, len: u64) -> Option<Vec<u8>>;
}

// NEW: ACL trait to avoid domain imports in HTTP layer
pub trait Acl {
    fn can_view(&self, memory_id: &str, who: Principal) -> bool;
}
```

**Error Types (Structured Error Handling):**

```rust
// core/types.rs
#[derive(Debug, PartialEq, Eq)]
pub enum AuthErr {
    Expired,
    BadSig,
    WrongMemory,
    VariantNotAllowed,
    AssetNotAllowed
}

#[derive(Debug, PartialEq, Eq)]
pub enum AssetErr {
    NotFound,
    TooLargeForInline,
    Io
}
```

**Token Minting (Query Operation):**

```rust
// Keep as query - no state mutation
#[query]
fn mint_http_token(
    memory_id: String,
    variants: Vec<String>,
    asset_ids: Option<Vec<String>>,
    ttl_secs: u32
) -> String {
    use crate::http::{
        core_types::{TokenPayload, TokenScope},
        auth_core::{sign_token_core, encode_token_url},
        adapters::{canister_env::CanisterClock, secret_store::StableSecretStore, acl::YourAclAdapter},
    };

    let env = CanisterClock;
    let acl = YourAclAdapter; // wraps effective_perm_mask()
    let caller = ic_cdk::caller();

    // ACL check via adapter (no domain imports in HTTP layer)
    assert!(acl.can_view(&memory_id, caller), "forbidden");

    let payload = TokenPayload {
        ver: 1,
        exp_ns: env.now_ns() + (ttl_secs as u64 * 1_000_000_000),
        nonce: rand_nonce_96(), // helper function
        scope: TokenScope { memory_id, variants, asset_ids },
        sub: Some(caller), // Bind token to caller by default
    };

    let token = sign_token_core(&StableSecretStore, &payload);
    encode_token_url(&token)
}
```

### 2. **Alternative: Full CQRS Pattern**

**Target Structure:**

```
src/http/
├── commands.rs      # Write operations (token generation)
├── queries.rs       # Read operations (token verification, asset serving)
├── domain.rs        # Shared business logic and types
├── core/            # Pure business logic (no ICP dependencies)
│   ├── auth_core.rs
│   ├── secret_core.rs
│   └── routes_core.rs
├── adapters/        # ICP-specific implementations
│   ├── canister_env.rs
│   └── storage_adapter.rs
└── routes/          # Thin canister wrappers
    ├── assets.rs
    └── health.rs
```

**CQRS Separation:**

```rust
// commands.rs - Write operations
#[update]
pub fn mint_http_token(scope: TokenScope, ttl_secs: u32) -> String {
    let env = CanisterEnv;
    let mut store = StorageAdapter;
    http_core::mint_token_command(&env, &mut store, scope, ttl_secs)
}

// queries.rs - Read operations
#[query]
pub fn verify_http_token(token: String, path_scope: TokenScope) -> bool {
    let env = CanisterEnv;
    let store = StorageAdapter;
    http_core::verify_token_query(&env, &store, token, path_scope)
}

#[query]
fn http_request(req: HttpRequest) -> HttpResponse {
    let env = CanisterEnv;
    let store = StorageAdapter;
    http_core::handle_http_request(&env, &store, req)
}
```

### 3. **Apply Decoupled Architecture**

**Core Layer (Pure Business Logic):**

```rust
// http/core/auth_core.rs
pub fn sign_token_core(
    clock: &dyn Clock,
    store: &dyn SecretStore,
    payload: &TokenPayload
) -> Result<EncodedToken, AuthError> {
    let secret = store.get_key();
    let now = clock.now_ns();

    // Pure business logic - no ICP dependencies
    let mut mac = HmacSha256::new_from_slice(&secret).unwrap();
    let bytes = canonical_bytes(payload);
    mac.update(&bytes);
    let sig = mac.finalize().into_bytes();

    Ok(EncodedToken { p: payload.clone(), s: sig })
}

pub fn verify_token_core(
    clock: &dyn Clock,
    store: &dyn SecretStore,
    token: &EncodedToken,
    path_scope: &TokenScope
) -> Result<(), VerifyErr> {
    // Pure verification logic - no ICP dependencies
    if clock.now_ns() > token.p.exp_ns { return Err(VerifyErr::Expired); }
    if token.p.scope.memory_id != path_scope.memory_id { return Err(VerifyErr::WrongMemory); }

    // Verify HMAC signature
    let secret = store.get_key();
    let mut mac = HmacSha256::new_from_slice(&secret).unwrap();
    let bytes = canonical_bytes(&token.p);
    mac.update(&bytes);
    mac.verify_slice(&token.s).map_err(|_| VerifyErr::BadSig)
}
```

**Adapter Layer (ICP-Specific):**

```rust
// adapters/canister_env.rs
pub struct CanisterEnv;

impl Clock for CanisterEnv {
    fn now_ns(&self) -> u64 {
        ic_cdk::api::time() // ✅ ICP dependency isolated
    }
}

// adapters/secret_store.rs
pub struct SecretStoreAdapter;

impl SecretStore for SecretStoreAdapter {
    fn get_key(&self) -> [u8; 32] {
        // Bridge to ICP storage
        get_current_secret_from_storage()
    }

    fn rotate_key(&mut self) -> Result<(), String> {
        // Bridge to ICP storage
        rotate_secret_in_storage()
    }
}

// adapters/asset_store.rs
pub struct AssetStoreAdapter;

impl AssetStore for AssetStoreAdapter {
    fn get_inline(&self, mem: &str, id: &str) -> Option<(Vec<u8>, String)> {
        // Bridge to existing memories/upload APIs
        get_inline_asset_from_memory(mem, id)
    }

    fn get_blob_len(&self, mem: &str, id: &str) -> Option<(u64, String)> {
        // Bridge to existing blob storage
        get_blob_length_from_storage(mem, id)
    }

    fn stream_blob(&self, mem: &str, id: &str, offset: u64, len: u64) -> Option<Vec<u8>> {
        // Bridge to existing streaming APIs
        stream_blob_chunk_from_storage(mem, id, offset, len)
    }
}
```

**Canister Layer (Thin Wrappers):**

```rust
// routes/assets.rs
pub fn get(memory_id: &str, variant: &str, req: &ParsedRequest) -> HttpResponse {
    let clock = CanisterEnv;
    let secret_store = SecretStoreAdapter;
    let asset_store = AssetStoreAdapter;

    // Parse path and verify token
    let path_scope = path_core::parse_asset_path(memory_id, variant)?;
    let token = extract_token_from_query(&req.url)?;
    let encoded_token = auth_core::decode_token(&token)?;

    // Verify token with structured error handling
    match auth_core::verify_token_core(&clock, &secret_store, &encoded_token, &path_scope) {
        Ok(()) => {},
        Err(AuthErr::Expired) => return status(401, "Expired"),
        Err(AuthErr::BadSig) => return status(403, "Forbidden"),
        Err(AuthErr::WrongMemory) => return status(403, "Forbidden"),
        Err(AuthErr::VariantNotAllowed) => return status(403, "Forbidden"),
        Err(AuthErr::AssetNotAllowed) => return status(403, "Forbidden"),
    }

    // Serve asset
    match serve_asset_core(&asset_store, memory_id, variant, req) {
        Ok(response) => response,
        Err(AssetErr::NotFound) => status(404, "Not Found"),
        Err(AssetErr::TooLargeForInline) => status(413, "Payload Too Large"),
        Err(AssetErr::Io) => status(500, "Internal Server Error"),
    }
}
```

## Benefits of Refactoring

### 1. **Testability** 🧪

```rust
#[test]
fn test_sign_token_core() {
    let mut mock_env = MockEnv::new();
    let mut mock_store = MockStore::new();

    mock_env.expect_now().returning(|| 1000);
    mock_store.expect_get_current_secret().returning(|| Ok([1u8; 32]));

    let result = sign_token_core(&mock_env, &mock_store, &payload);
    assert!(result.is_ok());
}
```

### 2. **Maintainability** 🔧

- Clear separation between business logic and ICP concerns
- Easy to modify business rules without touching ICP code
- Consistent error handling across layers

### 3. **Reusability** ♻️

- Core logic can be used in CLI tools, web services, etc.
- Adapters can be swapped for different implementations
- Business logic is platform-agnostic

## Migration Strategy

### **Recommended Approach (Tech Lead's)**

**Day 1: Core Modules + Adapters**

1. Create `http/core/{types,auth_core,path_core}.rs` and move:
   - Token structs + scope checks → `types.rs`
   - HMAC sign/verify (pure) → `auth_core.rs` (takes `SecretStore + Clock`)
   - URL→path parsing + scope mapping (pure) → `path_core.rs`
2. Create `http/adapters/{canister_env,secret_store,asset_store}.rs`:
   - Implement traits for ICP env/secret/stores (all ICP deps live here)
3. Update `mint_http_token(query)` to assemble deps and call **core**

**Day 2: Thin Routes + Testing**

1. Thin `routes/assets.rs`:
   - Parse path with `path_core`
   - Verify token with `auth_core`
   - Serve bytes via `AssetStore` (inline vs streaming)
2. Add unit tests for `auth_core` (pure) and `path_core`
3. One integration test that verifies `mint → GET` happy path

### **Alternative: Full CQRS Approach**

**Phase 1: Extract Core Logic**

1. Create `http_core/` module with trait definitions
2. Move business logic from `auth.rs` to `auth_core.rs`
3. Move business logic from `secret.rs` to `secret_core.rs`

**Phase 2: Create Adapters**

1. Implement `CanisterEnv` adapter
2. Implement `StorageAdapter` for secret management
3. Update canister functions to use adapters

**Phase 3: Apply CQRS**

1. Separate commands (token generation) from queries (token verification)
2. Create dedicated `commands.rs` and `queries.rs` modules
3. Update function annotations (`#[update]` vs `#[query]`)

**Phase 4: Testing**

1. Add unit tests for core logic
2. Add integration tests with PocketIC
3. Verify no regressions in functionality

## Current vs Target Architecture

### **Current (Mixed Concerns)**

```
Canister Functions
    ↓
Business Logic + ICP Dependencies
    ↓
Direct Storage Access
```

### **Target (Decoupled + CQRS)**

```
Commands (Updates)     Queries (Reads)
    ↓                      ↓
Core Logic (Pure)     Core Logic (Pure)
    ↓                      ↓
Adapters (ICP)        Adapters (ICP)
    ↓                      ↓
Storage Layer         Storage Layer
```

## Recommendation

**Priority: HIGH** - The HTTP module should be refactored to align with our established architectural principles before proceeding to Phase 2 (asset serving implementation).

**Recommended Approach:** Tech Lead's simplified three-layer architecture (core + adapters + routes)

**Benefits:**

- Consistent architecture across all modules
- Improved testability and maintainability
- Better separation of concerns
- Easier future development and debugging
- Clean services without CQRS theater

## Security Model

- **All assets private** (including thumbnails)
- **No HTTP certification** (private content)
- **Cache-Control: private, no-store** (no caching)
- **Token-based access control** (HMAC verified)
- **Short TTL** (180 seconds default)
- **Token binding** (sub: Principal) by default
- **Scope validation** (memory_id + variant + optional asset_id)

## Conclusion

While our HTTP module implementation is functionally correct and compiles successfully, it violates our established architectural principles. The tech lead's recommended refactoring approach provides a clean, testable solution that focuses on the real problem: ICP dependencies in business logic.

**Next Steps:**

1. Approve tech lead's refactoring approach
2. Schedule refactoring sprint
3. Document lessons learned for future modules

## Phase 1 Implementation Checklist

### **Core Requirements**

- [ ] `core/*` has **no** `ic_cdk` imports
- [ ] `mint_http_token` is a **query** calling `Acl` adapter for authorization
- [ ] Unit tests for `auth_core` (roundtrip, expiry, scope, tamper) pass
- [ ] Error enums in core; routes map to 401/403/404 cleanly
- [ ] Headers: `Cache-Control: private, no-store` everywhere

### **Implementation Tasks**

- [ ] Create `http/core/{types,auth_core,path_core}.rs`
- [ ] Create `http/adapters/{canister_env,secret_store,asset_store,acl}.rs`
- [ ] Implement structured error types (`AuthErr`, `AssetErr`)
- [ ] Add `Acl` trait to avoid domain imports in HTTP layer
- [ ] Update token minting with caller binding (`sub: Principal`)
- [ ] Set TTL default to 180 seconds
- [ ] Add `rand_nonce_96()` helper function
- [ ] Implement error mapping in routes (401/403/404/413/500)

### **Testing Requirements**

- [ ] Unit tests for `auth_core` (sign/verify/expiry/scope/tamper)
- [ ] Unit tests for `path_core` (URL parsing and validation)
- [ ] Integration test: valid token → 200; expired → 403
- [ ] Mock implementations for all traits in tests

## Further Discussion Points

### **1. Timeline Considerations**

**Tech Lead's Estimate:** 2 days  
**Our Experience:** 3-4 days (accounting for blockers)

**Discussion:** Should we plan for the optimistic 2-day timeline or be more conservative based on our blocker experience?

### **2. CQRS vs Clean Services**

**Tech Lead's Position:** "Keep CQRS lightweight here: don't force 'commands' just to be purist"  
**Our Analysis:** Full CQRS provides more structure but may be over-engineering

**Discussion:** Is the simplified approach sufficient, or do we need the full CQRS structure for future extensibility?

### **3. Trait Granularity**

**Current Design:** `Clock`, `SecretStore`, `AssetStore`  
**Potential Enhancement:** More granular traits for different asset types

**Discussion:** Should we keep traits minimal or add more specificity for different use cases?

### **4. Error Handling Strategy**

**Current Approach:** Simple `Result<T, String>`  
**Potential Enhancement:** Structured error types with proper HTTP status mapping

**Discussion:** How should we handle error propagation between layers?

### **5. Testing Strategy**

**Unit Tests:** Core logic with mocked traits  
**Integration Tests:** Full HTTP flow with PocketIC

**Discussion:** What level of test coverage do we need before considering the refactoring complete?

---

**Related Documents:**

- [CQRS Pattern](./CQRS.md)
- [Decoupled Architecture](./decoupled-architecture.md)
- [HTTP Module Implementation](../issues/open/serving-http/)
