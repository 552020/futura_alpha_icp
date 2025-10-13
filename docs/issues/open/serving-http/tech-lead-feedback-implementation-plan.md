# Tech Lead Feedback Implementation Plan

**Date:** 2025-01-27  
**Status:** Implementation Ready  
**Priority:** High

## Executive Summary

The tech lead has provided excellent, actionable feedback on our HTTP serving architecture. This document outlines the implementation plan to address all recommendations and complete Phase 1 with production-ready quality.

---

## 👍 What's Strong (Confirmed)

- ✅ Single URL scheme works for both hub and future capsule autonomy
- ✅ Stateless HMAC tokens with short TTL and key version (`kid`)
- ✅ All assets private with `Cache-Control: private, no-store`
- ✅ Adapter pattern with feature flags for hub/capsule modes
- ✅ Domain integration targets actual APIs (`asset_get_by_id_core`, blob store)

---

## ⚠️ Critical Issues to Address

### **1. Token Scope Hardness**

#### **Current Implementation:**

```rust
pub struct TokenScope {
    pub memory_id: String,
    pub variants: Vec<String>,        // ["thumbnail", "preview", "original"]
    pub asset_ids: Option<Vec<String>>, // Optional specific asset IDs
}
```

#### **Required Changes:**

```rust
pub struct TokenScope {
    pub canister_id: String,          // NEW: Defense-in-depth
    pub memory_id: String,
    pub variants: Vec<String>,
    pub asset_ids: Option<Vec<String>>, // Mandatory for originals
}

// Token validation rules:
// - Originals: asset_ids MUST be specified
// - Previews/Thumbnails: asset_ids optional but encouraged
// - Limits blast radius if token leaks
```

#### **Implementation:**

```rust
// In mint_http_token function
let asset_ids = match variants.contains(&"original".to_string()) {
    true => asset_ids.ok_or("Asset IDs required for original variants")?,
    false => asset_ids.unwrap_or_default(),
};
```

### **2. ACL & Context Hydration**

#### **Current Problem:**

```rust
let ctx = PrincipalContext {
    principal: who,
    groups: vec![], // TODO: Get from user system
    link: None,     // TODO: Extract from HTTP request if needed
    now_ns: ic_cdk::api::time(),
};
```

#### **Required Changes:**

```rust
// Add parser hook in HTTP layer
pub fn extract_magic_link_from_request(req: &ParsedRequest) -> Option<String> {
    // Extract magic link token from headers or query params
    req.headers.get("X-Magic-Link")
        .or_else(|| req.query_params.get("magic_link"))
}

// Update PrincipalContext construction
let ctx = PrincipalContext {
    principal: who,
    groups: vec![], // TODO: Get from user system
    link: extract_magic_link_from_request(&parsed), // NEW: Extract from request
    now_ns: ic_cdk::api::time(),
};
```

### **3. Secret Storage (Persistence)**

#### **Current Problem:**

```rust
// Hub: In-memory secrets (not persistent)
static SECRET: Mutex<Option<[u8; 32]>> = Mutex::new(None);
```

#### **Required Changes:**

```rust
// Hub: StableCell for persistent secrets
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Secrets {
    pub current: [u8; 32],
    pub previous: Option<[u8; 32]>,
    pub kid: u32,
}

impl Storable for Secrets {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(serde_json::to_vec(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        serde_json::from_slice(&bytes).unwrap()
    }
}

impl BoundedStorable for Secrets {
    const MAX_SIZE: u32 = 1024; // Reasonable upper bound
    const IS_FIXED_SIZE: bool = false;
}

// Stable storage
thread_local! {
    static SECRETS: RefCell<StableCell<Secrets, MemoryId>> =
        RefCell::new(StableCell::new(MEM_SECRETS, Secrets::default()).unwrap());
}
```

### **4. Deprecated APIs & Lifetimes**

#### **Current Problems:**

```rust
// Deprecated API usage
use ic_cdk::api::msg_caller; // Should use ic_cdk::caller
use ic_cdk::api::raw_rand;   // Should use ic_cdk::raw_rand

// Lifetime issues
pub fn get(memory_id: &str, variant: &str, req: &ParsedRequest) -> HttpResponse<'static>
```

#### **Required Changes:**

```rust
// Fix deprecated APIs
use ic_cdk::caller;        // Instead of ic_cdk::api::msg_caller
use ic_cdk::raw_rand;      // Instead of ic_cdk::api::raw_rand

// Fix lifetimes
pub fn get(memory_id: &str, variant: &str, req: &ParsedRequest) -> HttpResponse<'static> {
    // Use owned data or static references
    let response_data = get_asset_data(memory_id, variant, req);
    HttpResponse::ok(
        response_data.bytes,
        vec![
            ("Content-Type".into(), response_data.content_type),
            ("Cache-Control".into(), "private, no-store".into()),
            ("Content-Length".into(), response_data.bytes.len().to_string()),
        ]
    ).build()
}
```

### **5. Blob Path Consistency**

#### **Current Implementation:**

```
/asset/{memory_id}/{variant}?id={asset_id}&token=...
```

#### **Recommended Change:**

```
/asset/{memory_id}/{variant}/{asset_id}?token=...
```

#### **Benefits:**

- Cache-friendly for future CDN layers
- Better logging and debugging
- Cleaner URL structure

#### **Implementation:**

```rust
// Update route pattern
match (parsed.method.as_str(), parsed.path_segments.as_slice()) {
    ("GET", [asset, mem, var, asset_id]) if asset == "asset" =>
        assets_route::get(mem, var, asset_id, &parsed),
    _ => HttpResponse::builder().with_status_code(StatusCode::NOT_FOUND).build(),
}
```

### **6. Large Object Fallback**

#### **Current Problem:**

No size limits, streaming deferred to Phase 2

#### **Required Changes:**

```rust
// Add size guard
const MAX_SINGLE_RESPONSE_SIZE: u64 = 2 * 1024 * 1024; // 2MB

pub fn get(memory_id: &str, variant: &str, asset_id: &str, req: &ParsedRequest) -> HttpResponse<'static> {
    // ... existing logic ...

    if let Some(inline) = store.get_inline(memory_id, asset_id) {
        if inline.bytes.len() as u64 > MAX_SINGLE_RESPONSE_SIZE {
            return HttpResponse::builder()
                .with_status_code(StatusCode::PAYLOAD_TOO_LARGE)
                .with_body("Asset too large for single response. Streaming support coming in Phase 2.")
                .build();
        }
        // ... serve inline asset ...
    }

    if let Some((blob_size, _)) = store.get_blob_len(memory_id, asset_id) {
        if blob_size > MAX_SINGLE_RESPONSE_SIZE {
            return HttpResponse::builder()
                .with_status_code(StatusCode::PAYLOAD_TOO_LARGE)
                .with_body("Asset too large for single response. Streaming support coming in Phase 2.")
                .build();
        }
        // ... serve blob asset ...
    }
}
```

### **7. Observability**

#### **Current Problem:**

Minimal logging, no metrics

#### **Required Changes:**

```rust
// Add metrics counters
thread_local! {
    static METRICS: RefCell<HttpMetrics> = RefCell::new(HttpMetrics::default());
}

#[derive(Default)]
struct HttpMetrics {
    requests_total: u64,
    auth_fail_total: u64,
    acl_deny_total: u64,
    bytes_served_total: u64,
    latency_ms_bucket: Vec<u64>, // 0-100ms, 100-500ms, 500ms+
}

// Logging function
fn log_request(method: &str, path: &str, memory_id: &str, variant: &str,
               status: u16, latency_ms: u64, bytes_served: u64) {
    ic_cdk::println!(
        "HTTP_REQUEST: method={} path={} memory_id={} variant={} status={} latency_ms={} bytes_served={}",
        method, path, memory_id, variant, status, latency_ms, bytes_served
    );

    // Update metrics
    METRICS.with(|m| {
        let mut metrics = m.borrow_mut();
        metrics.requests_total += 1;
        metrics.bytes_served_total += bytes_served;

        // Update latency bucket
        let bucket = match latency_ms {
            0..=100 => 0,
            101..=500 => 1,
            _ => 2,
        };
        if bucket >= metrics.latency_ms_bucket.len() {
            metrics.latency_ms_bucket.resize(bucket + 1, 0);
        }
        metrics.latency_ms_bucket[bucket] += 1;
    });
}
```

---

## Implementation Checklist

### **Phase 1 Completion Tasks**

#### **Critical Fixes (Must Complete)**

- [ ] **Fix `HttpResponse<'static>` lifetimes** on route functions
- [ ] **Replace deprecated API calls** (`ic_cdk::caller`, correct `raw_rand` import)
- [ ] **Implement `StableCell<Secrets>`** with `Storable/BoundedStorable` for hub
- [ ] **Enforce token scope rules** (originals must specify `asset_id`, add `canister_id`)
- [ ] **Add 2MB guard** (return 413) with clear error message
- [ ] **Map errors to proper HTTP status codes** (400, 401, 403, 404, 413, 500)
- [ ] **Add minimal logging** (method, path, memory_id, variant, status, latency, bytes)

#### **Enhancements (Should Complete)**

- [ ] **Token TTL improvements** (min TTL 15s, clock skew tolerance ±5s)
- [ ] **Content headers** (`Content-Disposition: inline; filename="<sanitized>"`)
- [ ] **MIME type detection** (magic sniff for jpeg/png/webp/gif)
- [ ] **Magic link parser hook** (extract from request, populate `ctx.link`)
- [ ] **URL path consistency** (move `asset_id` to path: `/asset/{memory_id}/{variant}/{asset_id}`)

#### **Testing (Must Complete)**

- [ ] **Unit tests for `auth_core`** (sign/verify, rotation)
- [ ] **Unit tests for `path_core`** parsing
- [ ] **One PocketIC test** (mint → GET happy path for inline + blob)

---

## Adapter Implementation Details

### **Hub Adapters (Current)**

#### **Acl Adapter:**

```rust
impl Acl for FuturaAclAdapter {
    fn can_view(&self, memory_id: &str, who: Principal) -> bool {
        // Search caller's accessible capsules
        let accessible_capsules = self.store.get_accessible_capsules(&who);
        for capsule_id in accessible_capsules {
            if let Some(memory) = self.store.get_memory(&capsule_id, memory_id) {
                let ctx = PrincipalContext {
                    principal: who,
                    groups: vec![], // TODO: Get from user system
                    link: None,     // TODO: Extract from request
                    now_ns: ic_cdk::api::time(),
                };
                let perm_mask = effective_perm_mask(&memory, &ctx);
                return (perm_mask & Perm::VIEW.bits()) != 0;
            }
        }
        false
    }
}
```

#### **AssetStore Adapter:**

```rust
impl AssetStore for FuturaAssetStore {
    fn get_inline(&self, memory_id: &str, asset_id: &str) -> Option<InlineAsset> {
        // Use asset_get_by_id_core
        match asset_get_by_id_core(&self.env, &self.store, memory_id.to_string(), asset_id.to_string()) {
            Ok(MemoryAssetData::Inline { bytes, content_type, size, sha256 }) => {
                Some(InlineAsset { bytes, content_type })
            }
            _ => None,
        }
    }

    fn get_blob_len(&self, memory_id: &str, asset_id: &str) -> Option<(u64, String)> {
        // Use blob store API
        match asset_get_by_id_core(&self.env, &self.store, memory_id.to_string(), asset_id.to_string()) {
            Ok(MemoryAssetData::BlobInternal { blob_ref, .. }) => {
                match blob_get_meta(blob_ref) {
                    Ok(meta) => Some((meta.size, blob_ref)),
                    Err(_) => None,
                }
            }
            _ => None,
        }
    }
}
```

#### **SecretStore Adapter:**

```rust
impl SecretStore for FuturaSecretStore {
    fn get_secret(&self, kid: u32) -> Option<[u8; 32]> {
        SECRETS.with(|secrets| {
            let secrets_cell = secrets.borrow();
            let secrets_data = secrets_cell.get();

            match kid {
                k if k == secrets_data.kid => Some(secrets_data.current),
                k if k == secrets_data.kid - 1 => secrets_data.previous,
                _ => None,
            }
        })
    }

    fn rotate_secret(&mut self) -> Result<u32, String> {
        SECRETS.with(|secrets| {
            let mut secrets_cell = secrets.borrow_mut();
            let mut secrets_data = secrets_cell.get();

            // Generate new secret
            let new_secret = generate_random_secret()?;

            // Rotate secrets
            secrets_data.previous = Some(secrets_data.current);
            secrets_data.current = new_secret;
            secrets_data.kid += 1;

            secrets_cell.set(secrets_data);
            Ok(secrets_data.kid)
        })
    }
}
```

### **Capsule Adapters (Future)**

#### **Acl Adapter:**

```rust
impl Acl for CapsuleAclAdapter {
    fn can_view(&self, memory_id: &str, who: Principal) -> bool {
        // Resolve memory_id directly from capsule-local maps
        if let Some(memory) = self.capsule.memories.get(memory_id) {
            let ctx = PrincipalContext {
                principal: who,
                groups: vec![], // TODO: Get from user system
                link: None,     // TODO: Extract from request
                now_ns: ic_cdk::api::time(),
            };
            let perm_mask = effective_perm_mask(memory, &ctx);
            return (perm_mask & Perm::VIEW.bits()) != 0;
        }
        false
    }
}
```

---

## Error Mapping

### **HTTP Status Code Mapping:**

```rust
pub fn map_error_to_status(error: &HttpError) -> u16 {
    match error {
        HttpError::MalformedToken => 400,
        HttpError::MissingToken => 401,
        HttpError::TokenExpired => 401,
        HttpError::InvalidSignature => 401,
        HttpError::AclDenied => 403,
        HttpError::AssetNotFound => 404,
        HttpError::MemoryNotFound => 404,
        HttpError::AssetTooLarge => 413,
        HttpError::InternalError => 500,
    }
}
```

---

## Production Readiness Checklist

### **Security:**

- [ ] Token scope hardening (originals require asset_ids)
- [ ] Canister ID in token payload
- [ ] Stable secret storage with rotation
- [ ] Proper error mapping (no information leakage)

### **Performance:**

- [ ] 2MB size guard with clear error messages
- [ ] Efficient asset lookup (no unnecessary cross-capsule searches)
- [ ] Proper HTTP headers for browser compatibility

### **Observability:**

- [ ] Request logging with key metrics
- [ ] Error tracking and categorization
- [ ] Performance monitoring (latency buckets)

### **Reliability:**

- [ ] Comprehensive error handling
- [ ] Graceful degradation for large assets
- [ ] Proper HTTP status codes

---

## Conclusion

The tech lead's feedback provides a clear path to production-ready HTTP serving. The key improvements focus on:

1. **Security hardening** (token scope, secret persistence)
2. **Production reliability** (error handling, size limits)
3. **Observability** (logging, metrics)
4. **Code quality** (fix deprecated APIs, lifetimes)

With these changes, the HTTP module will be production-ready for hub mode and provide a solid foundation for future capsule autonomy.

---

**Status:** ✅ **Ready for Implementation**  
**Priority:** **High** - Complete Phase 1 with production quality

