# Tech Lead's 9-Point Feedback

**Date:** December 2024  
**Status:** Implementation Ready  
**Priority:** High

## Overview

The tech lead provided specific, actionable feedback to resolve our Phase 1 blockers and move from our current working implementation to a production-ready solution.

## 🔧 **The 9 Points**

### **1. Status codes (`ic-http-certification`)**

**Issue:** Using `HttpResponse::ok()` for all responses (always sets 200)

**Solution:**

```rust
use ic_http_certification::HttpResponse;

fn resp(code: u16, body: impl Into<Vec<u8>>, headers: &[(&str, &str)]) -> HttpResponse {
    HttpResponse {
        status_code: code,
        headers: headers.iter().map(|(k,v)| (k.to_string(), v.to_string())).collect(),
        body: body.into(),
        upgrade: None,
        streaming_strategy: None,
    }
}

pub fn bad_request(msg: &str) -> HttpResponse {
    resp(400, msg.as_bytes(), &[("Content-Type", "text/plain")])
}
```

**Why:** `ok()` always sets 200. Need accurate status semantics for 401/403/404 and 206 later.

**Time:** 2 minutes  
**Status:** [ ] TODO

---

### **2. Stable secret storage (persistence & rotation)**

**Issue:** Using `std::sync::Mutex` which doesn't persist across upgrades

**Solution:** Use `ic-stable-structures::StableCell` with proper `Storable`/`BoundedStorable` traits

```rust
// Cargo.toml
// ic-stable-structures = "0.6"

use candid::{CandidType, Deserialize, Serialize};
use ic_stable_structures::{
  memory_manager::{MemoryId, MemoryManager, VirtualMemory},
  DefaultMemoryImpl, StableCell, Storable, BoundedStorable
};

type Mem = VirtualMemory<DefaultMemoryImpl>;

#[derive(Clone, Copy, CandidType, Serialize, Deserialize)]
struct Secrets {
    current: [u8; 32],
    previous: [u8; 32], // zeroed means "none"
    version: u32,
}

impl Storable for Secrets {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::Encode!(&self).unwrap())
    }
    fn from_bytes(b: std::borrow::Cow<[u8]>) -> Self {
        candid::Decode!(&b, Self).unwrap()
    }
}

impl BoundedStorable for Secrets {
    const MAX_SIZE: u32 = 256;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static SECRET_CELL: std::cell::RefCell<Option<StableCell<Secrets, Mem>>> = Default::default();
}

#[ic_cdk::init]
async fn init() {
    let mm = MemoryManager::init(DefaultMemoryImpl::default());
    let mem = mm.get(MemoryId::new(42)); // pick a stable ID for this cell
    let seeded = Secrets {
        current: random_32().await,
        previous: [0;32],
        version: 1
    };
    let cell = StableCell::init(mem, seeded).expect("init stable cell");
    SECRET_CELL.with(|c| *c.borrow_mut() = Some(cell));
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    // Re-open existing cell
    let mm = MemoryManager::init(DefaultMemoryImpl::default());
    let mem = mm.get(MemoryId::new(42));
    let cell = StableCell::init(mem, Secrets {
        current: [0;32],
        previous: [0;32],
        version: 0
    }).expect("open cell");
    SECRET_CELL.with(|c| *c.borrow_mut() = Some(cell));
}

pub async fn rotate_secret() {
    SECRET_CELL.with(|c| {
        let cell = c.borrow_mut();
        let cell = cell.as_ref().expect("cell");
        let mut secrets = cell.get().clone();
        secrets.previous = secrets.current;
        secrets.current = ic_cdk::spawn_blocking(random_32_sync);
        secrets.version = secrets.version.wrapping_add(1);
        cell.set(secrets).expect("persist");
    });
}

pub fn get_current_key() -> [u8; 32] {
    SECRET_CELL.with(|c| c.borrow().as_ref().unwrap().get().current )
}

async fn random_32() -> [u8;32] {
    use ic_cdk::api::management_canister::main::raw_rand;
    let r = raw_rand().await.expect("raw_rand").0;
    let mut out = [0u8;32];
    out.copy_from_slice(&r[..32]);
    out
}
```

**Why:** Survives upgrades, no Mutex, no race conditions. Gives previous key slot for graceful rotation.

**Time:** 30-45 minutes  
**Status:** [ ] TODO

---

### **3. Async randomness**

**Issue:** We had issues with `ic_cdk::block_on()` being removed

**Solution:** Keep using `raw_rand().await` directly in async functions

**Why:** No `block_on`; no deadlocks.

**Time:** Already implemented  
**Status:** [x] DONE

---

### **4. Deprecated API**

**Issue:** Using deprecated APIs

**Solution:** Replace calls where you touch them next:

- `ic_cdk::api::msg_caller()` → `ic_cdk::caller()`
- Already using correct `raw_rand` path

**Why:** Removes warnings; keeps code base modern.

**Time:** 2 minutes  
**Status:** [ ] TODO

---

### **5. Module structure**

**Issue:** Module organization

**Solution:** ✅ **Already implemented** - Modern `#[path]` attributes approved

**Why:** You switched to modern module wiring — ✅

**Time:** Already implemented  
**Status:** [x] DONE

---

### **6. Streaming deferred**

**Issue:** Streaming functionality commented out

**Solution:** ✅ **Totally fine to defer** - But with one guardrail:

**Guardrail:** Keep token verification pure and reusable so you can call the same `verify(&token, &path_scope)` from the streaming callback without duplicating logic. Plan to include the same `token` and exact `path` in the callback token to re-verify every chunk.

**Why:** Interface seams already make it easy to plug in later.

**Time:** Deferred to Phase 2  
**Status:** [x] DEFERRED

---

### **7. Security polish (quick wins)**

**Requirements:**

- Default TTL to **180s** ✅ (already implemented)
- Put **key version** in token payload (`kid`) so rotation is painless later
- Always set: `Cache-Control: private, no-store` ✅ (already implemented)

**Later additions:**

- Add **ETag** from asset hash and support `If-None-Match` → still private but saves bandwidth
- Add tiny `Acl` adapter (`can_view(memory_id, principal)`) so `mint_http_token` stays thin and testable

**Time:** 10 minutes (for `kid` field)  
**Status:** [ ] TODO (partially done)

---

### **8. Status code map & helpers (tiny UX fix)**

**Solution:**

```rust
fn ok_img(ct: &str, body: Vec<u8>) -> HttpResponse {
    resp(200, body, &[("Content-Type", ct), ("Cache-Control", "private, no-store")])
}
fn unauthorized() -> HttpResponse {
    resp(401, b"Unauthorized".as_slice(), &[("Content-Type","text/plain")])
}
fn forbidden() -> HttpResponse {
    resp(403, b"Forbidden".as_slice(), &[("Content-Type","text/plain")])
}
fn not_found() -> HttpResponse {
    resp(404, b"Not Found".as_slice(), &[("Content-Type","text/plain")])
}
```

**Why:** Use these in routes so behaviors are consistent.

**Time:** 5 minutes  
**Status:** [ ] TODO

---

### **9. Why current workarounds are okay (and how to exit them)**

**Assessment:**

- **Mutex secret**: okay dev-only, but rotate to `StableCell` asap (above)
- **`ok()` for all statuses**: fix now (struct literal)
- **Streaming**: defer—your interface seams already make it easy to plug in later

**Why:** Validates current approach while providing clear path forward.

**Time:** Already assessed  
**Status:** [x] ASSESSED

---

## 🎯 **Implementation Priority**

### **Immediate (Today)**

1. **Status codes** (2 mins) - Use struct literals instead of `ok()`
2. **Status code helpers** (5 mins) - Clean up route code
3. **Deprecated API check** (2 mins) - Verify current usage

### **This Week**

4. **Stable secret storage** (30-45 mins) - Big architectural fix
5. **Key version in token** (10 mins) - For rotation support

### **Already Done ✅**

- Async randomness
- Module structure
- Streaming deferred
- Workarounds assessed

## 📊 **Impact Assessment**

**High Impact:**

- Stable secret storage (solves persistence across upgrades)
- Status code fixes (proper HTTP semantics)

**Medium Impact:**

- Status code helpers (code quality)
- Key version in token (future rotation support)

**Low Impact:**

- Deprecated API check (cleanup)

## 🚀 **Next Steps**

1. **Start with immediate fixes** - Quick wins that improve code quality
2. **Tackle stable storage** - The big architectural improvement
3. **Add security polish** - Key version for future rotation
4. **Test thoroughly** - Ensure all changes work together

This feedback provides a clear, actionable path from our current working implementation to a production-ready Phase 1 solution.
