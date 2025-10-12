# Phase 1 Implementation Guide

**Date:** December 2024  
**Status:** Final Implementation Plan  
**Based on:** Tech Lead's Final Review

## Overview

This document provides the specific implementation details for Phase 1 of the HTTP module refactoring, based on the tech lead's final review and refinements.

## 5 Precise Refinements (Tech Lead's Requirements)

### 1. **Keep CQRS Lightweight**

- Token mint is a **query** (no state mutation)
- Don't add `commands.rs/queries.rs` unless we later introduce real writes

### 2. **Tighten Trait Boundaries**

- Current `Clock`, `SecretStore`, `AssetStore` are the right size
- Add tiny `Acl` trait at adapter layer to avoid domain imports in routes

### 3. **Error Typing**

- Replace `String` errors across core with small enums
- Map to HTTP status in routes for clean error handling

### 4. **Token Defaults (Security Posture)**

- TTL default: **180s**
- Bind token to caller (`sub: Principal`) by default
- Scope must include **memory_id + variant + (optional) asset_id**
- Headers: `Cache-Control: private, no-store` (all assets private)

### 5. **Streaming Seam (Phase 2)**

- Keep route code unchanged
- Add streaming via `AssetStore::read_blob_chunk` with callback token re-verification

## Implementation Code

### **New ACL Adapter**

```rust
// src/http/adapters/acl.rs
use candid::Principal;

pub trait Acl {
    fn can_view(&self, memory_id: &str, who: Principal) -> bool;
}

// Implementation that wraps effective_perm_mask()
pub struct YourAclAdapter;

impl Acl for YourAclAdapter {
    fn can_view(&self, memory_id: &str, who: Principal) -> bool {
        // Bridge to existing domain logic
        // This wraps effective_perm_mask() without importing domain code into HTTP layer
        validate_memory_access(memory_id, who)
    }
}
```

### **Updated Token Minting**

```rust
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

### **Error Enums (Core)**

```rust
// src/http/core/types.rs
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

### **Error Mapping in Routes**

```rust
// src/http/routes/assets.rs
match verify_token_core(&clock, &secret, &token, &want) {
    Ok(()) => {}
    Err(AuthErr::Expired) => return status(401, "Expired"),
    Err(AuthErr::BadSig) => return status(403, "Forbidden"),
    Err(AuthErr::WrongMemory) => return status(403, "Forbidden"),
    Err(AuthErr::VariantNotAllowed) => return status(403, "Forbidden"),
    Err(AuthErr::AssetNotAllowed) => return status(403, "Forbidden"),
}

match serve_asset_core(&asset_store, memory_id, variant, req) {
    Ok(response) => response,
    Err(AssetErr::NotFound) => status(404, "Not Found"),
    Err(AssetErr::TooLargeForInline) => status(413, "Payload Too Large"),
    Err(AssetErr::Io) => status(500, "Internal Server Error"),
}
```

### **Helper Functions**

```rust
// Helper for 96-bit random nonce generation
fn rand_nonce_96() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    // Use existing random generation logic
    // This should be implemented using the same approach as in secret.rs
    nonce
}

// Helper for URL-safe token encoding
fn encode_token_url(token: &EncodedToken) -> String {
    // Base64 URL-safe encoding without padding
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(token).unwrap())
}
```

## Module Structure

```
src/http.rs                  // router + entry points
src/http/core/
  types.rs                   // TokenPayload, TokenScope, AuthErr, AssetErr
  auth_core.rs               // pure: sign/verify token
  secret_core.rs             // pure: key lifecycle API (trait)
  path_core.rs               // parse/validate path/variant/scope (pure)
src/http/adapters/
  canister_env.rs            // now(), caller(), time source
  secret_store.rs            // stable secret get/rotate (ICP deps)
  asset_store.rs             // reads images via existing memories/upload APIs
  acl.rs                     // NEW: ACL trait implementation
src/http/routes/
  assets.rs                  // thin: parse→verify→serve
  health.rs
```

## Security Headers

All successful responses must include:

```rust
let headers = vec![
    ("Content-Type".to_string(), content_type),
    ("Cache-Control".to_string(), "private, no-store".to_string()),
    ("Content-Length".to_string(), size.to_string()),
];
```

## Token Payload Structure

```rust
pub struct TokenPayload {
    pub ver: u8,                    // Version (1)
    pub exp_ns: u64,               // Expiry in nanoseconds
    pub nonce: [u8; 12],           // 96-bit random nonce
    pub scope: TokenScope,         // Memory ID + variants + optional asset IDs
    pub sub: Option<Principal>,    // Caller binding (default: Some(caller))
}

pub struct TokenScope {
    pub memory_id: String,
    pub variants: Vec<String>,             // e.g. ["thumbnail","preview","original"]
    pub asset_ids: Option<Vec<String>>,    // optional narrowing
}
```

## Checkpoint Checklist

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

## Green/Yellow/Red Status

- **Green:** Module layout, traits, token HMAC core, adapter isolation, unit-testability
- **Yellow:** Error enums + mapping (small change, high win)
- **Green:** "All assets private" & "no certification" captured explicitly
- **Green:** Short TTL and no caching
- **Yellow:** Add `Acl` trait now to avoid domain imports in HTTP

## Next Steps

1. **Implement the ACL adapter** to wrap `effective_perm_mask()`
2. **Add the `rand_nonce_96()` helper** for token generation
3. **Create structured error types** and mapping logic
4. **Update token minting** with caller binding
5. **Add comprehensive unit tests** for core logic

This implementation guide provides the exact code structure and requirements needed to complete Phase 1 according to the tech lead's specifications.

---

**Related Documents:**

- [HTTP Module Architecture Analysis](./http-module-architecture-analysis.md)
- [Implementation Blockers and Solutions](./implementation-blockers-and-solutions.md)
- [Tech Lead's 9-Point Feedback](./tech-lead-9-point-feedback.md)
- [Phase 1 Implementation TODOs](./phase1-implementation-todos.md)
