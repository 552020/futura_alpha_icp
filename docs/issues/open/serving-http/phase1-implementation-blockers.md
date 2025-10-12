# Phase 1 Implementation Blockers - ICP Expert Consultation

**Date:** December 2024  
**Context:** Implementing token-gated `http_request` system for private asset serving on ICP  
**Status:** Blocked on 4 technical issues requiring expert guidance

## 🎯 **Objective**

We're implementing Phase 1 of a token-gated `http_request` system based on the tech lead's detailed proposal. The implementation includes:

- HMAC secret management with stable memory storage
- Stateless token generation and verification
- HTTP request routing and response handling
- Integration with existing canister lifecycle

## 🚫 **Current Blockers**

### **Blocker 1: StatusCodeWrapper Type Mismatch**

**Issue:** The `ic-http-certification` crate v3.0.3 expects `StatusCodeWrapper` but we can't find the correct way to create it.

**Error:**

```rust
error[E0308]: mismatched types
 --> src/backend/src/http/response.rs:5:22
  |
5 |         status_code: 200,
  |                      ^^^ expected `StatusCodeWrapper`, found integer
```

**Current Code:**

```rust
use ic_http_certification::HttpResponse;

pub fn ok(bytes: Vec<u8>, ct: &str) -> HttpResponse<'static> {
    HttpResponse {
        status_code: 200,  // ❌ Type mismatch
        headers: vec![
            ("Content-Type".into(), ct.into()),
            ("Cache-Control".into(), "private, no-store".into()),
        ],
        body: bytes.into(),
        upgrade: None,
    }
}
```

**What we tried:**

1. ✅ `use ic_http_certification::{HttpResponse, StatusCodeWrapper};` - `StatusCodeWrapper` not found in root
2. ✅ `use http::StatusCode;` - `http` crate not available in dependencies
3. ✅ Direct integer assignment - Type mismatch persists

**Question for ICP Expert:**

- How do we create a `StatusCodeWrapper` in `ic-http-certification` v3.0.3?
- Is there a constructor function or different import path?
- Should we use a different approach for HTTP status codes?

---

### **Blocker 2: Stable Memory Integration Issues**

**Issue:** The tech lead's proposal uses `StableCell` with `OnceCell` but encounters thread safety issues.

**Error:**

```rust
error[E0277]: `Rc<RefCell<MemoryManagerInner<Rc<...>>>>` cannot be shared between threads safely
static CELL: OnceCell<StableCell<HttpHmacSecret, Mem>> = OnceCell::new();
```

**Tech Lead's Proposed Code:**

```rust
use ic_stable_structures::{StableCell, memory_manager::{MemoryManager, VirtualMemory}, DefaultMemoryImpl};
use once_cell::sync::OnceCell;

type Mem = VirtualMemory<DefaultMemoryImpl>;

static CELL: OnceCell<StableCell<HttpHmacSecret, Mem>> = OnceCell::new();
static MM: OnceCell<MemoryManager<DefaultMemoryImpl>> = OnceCell::new();

pub fn init_secret() {
    let mm = MemoryManager::init(DefaultMemoryImpl::default());
    let mem = mm.get(0);  // ❌ Also fails: expected `MemoryId`, found integer
    let cell = StableCell::init(mem, random_secret()).expect("init cell");
    MM.set(mm).ok();
    CELL.set(cell).ok();
}
```

**Additional Errors:**

```rust
error[E0308]: mismatched types
let mem = mm.get(0);  // expected `MemoryId`, found integer

error[E0277]: the trait bound `HttpHmacSecret: Storable` is not satisfied
let cell = StableCell::init(mem, random_secret()).expect("init cell");
```

**What we tried:**

1. ✅ Exact implementation from tech lead's proposal
2. ✅ Simplified to in-memory storage (works but not persistent across upgrades)

**Question for ICP Expert:**

- What's the correct way to store secrets in stable memory with `ic-stable-structures` v0.6?
- Should we use a different approach than `StableCell` + `OnceCell`?
- How do we properly implement the `Storable` trait for custom structs?
- What's the correct way to get memory segments from `MemoryManager`?

---

### **Blocker 3: Async Random Generation**

**Issue:** The tech lead's proposal uses `ic_cdk::block_on()` which doesn't exist.

**Error:**

```rust
error[E0425]: cannot find function `block_on` in crate `ic_cdk`
let rnd = ic_cdk::block_on(async { raw_rand().await.unwrap().0 });
```

**Tech Lead's Proposed Code:**

```rust
fn random_secret() -> HttpHmacSecret {
    let mut key = [0u8; 32];
    let rnd = ic_cdk::block_on(async { raw_rand().await.unwrap().0 });  // ❌ block_on doesn't exist
    for (i,b) in rnd.iter().enumerate().take(32) { key[i] = *b; }
    HttpHmacSecret { ver: 1, created_ns: time(), key }
}
```

**What we tried:**

1. ✅ Direct `block_on` call - Function doesn't exist in `ic_cdk`
2. ✅ Simplified deterministic approach (works but not cryptographically secure)

**Question for ICP Expert:**

- What's the correct way to generate random bytes in a synchronous context within an ICP canister?
- Should we use a different approach than `raw_rand()`?
- Is there a synchronous random generation API available?
- How do we properly handle async operations in canister initialization?

---

### **Blocker 4: Deprecated API Usage**

**Issue:** The tech lead's proposal uses deprecated APIs that generate warnings.

**Warnings:**

```rust
warning: use of deprecated function `ic_cdk::caller`: Use `msg_caller` instead
warning: use of deprecated function `ic_cdk::api::management_canister::main::raw_rand`
```

**Current Code:**

```rust
// In lib.rs
let caller: Principal = ic_cdk::caller();  // ❌ Deprecated

// In secret.rs
use ic_cdk::api::{management_canister::main::raw_rand, time};  // ❌ Deprecated
```

**Question for ICP Expert:**

- Should we update the tech lead's proposal to use the new APIs (`msg_caller`, `management_canister::raw_rand`)?
- Is there a reason to stick with the deprecated ones for compatibility?
- What's the migration path for these deprecated functions?

---

## 📋 **Current Implementation Status**

### ✅ **What's Working:**

- Modern Rust module structure (no `mod.rs`)
- HTTP request parsing with `ParsedRequest`
- Basic response helpers (except status codes)
- Module integration in `lib.rs`
- Dependencies added (`hmac`, `once_cell`, `rand`)

### ❌ **What's Blocked:**

- HTTP response creation (StatusCodeWrapper issue)
- Stable memory secret storage
- Cryptographically secure random generation
- Clean compilation (deprecated API warnings)

### 🔄 **Workaround in Place:**

We've implemented a simplified version that compiles but uses:

- In-memory secret storage (not persistent)
- Deterministic "random" generation (not secure)
- Placeholder status codes

## 🎯 **Expected Outcome**

Once these blockers are resolved, we'll have a complete Phase 1 implementation that includes:

1. **Secret Management:** Stable memory storage with proper initialization and rotation
2. **Token System:** HMAC-based stateless tokens with proper signing/verification
3. **HTTP Routing:** Working `http_request` handler with proper status codes
4. **Mint API:** Query method for token generation
5. **Clean Compilation:** No warnings or errors

## 📞 **Next Steps**

1. **Expert Consultation:** Get guidance on the 4 blockers above
2. **Implementation:** Apply expert recommendations
3. **Testing:** Verify Phase 1 functionality
4. **Phase 2 Planning:** Move to route integration and asset serving

---

**Contact:** Ready for expert consultation on these specific technical blockers.
