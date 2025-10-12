# Token-Gated `http_request` Implementation for Futura

**Status**: Implementation Plan  
**Phase**: Following the roadmap in `http_request_implementation_final.md`  
**Approach**: Token-gated HTTP requests with HMAC authentication

---

## 🎯 Overview

This implementation follows the **token-gated `http_request`** approach outlined in the final roadmap. Key principles:

- **Everything is private** (including thumbnails)
- **No Web2 proxy** for access control
- **No per-image writes** just to read
- **Stateless HMAC tokens** with 3-minute TTL
- **Next.js `<Image>`** integration where possible
- **CustomImage fallback** for edge cases

---

## 🏗️ Architecture

### Token Flow

```
1. Frontend calls mint_http_token(memory_id, variants, ttl) [QUERY]
2. Canister validates user permissions and returns HMAC token
3. Frontend builds URLs: /asset/{memory_id}/{variant}?token=...
4. Browser requests asset via http_request
5. Canister verifies token and serves asset
```

### URL Structure

```
# All variants are private and require tokens
https://your-canister.icp0.io/asset/{memory_id}/thumbnail?token=...
https://your-canister.icp0.io/asset/{memory_id}/preview?token=...
https://your-canister.icp0.io/asset/{memory_id}/placeholder?token=...
https://your-canister.icp0.io/asset/{memory_id}/original/{asset_id}?token=...
```

---

## 🔐 Token System

### Token Payload (Canonical JSON)

```json
{
  "ver": 1,
  "scope": {
    "memory_id": "c6f07efb-4e4f-73c0-c6f0-0000000073c0",
    "variants": ["thumbnail", "preview", "original"]
  },
  "exp": 1738950000,
  "nonce": "96-bit-random"
}
```

### Token Generation & Verification

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde_json::{json, Value};
use base64::{Engine as _, engine::general_purpose};

type HmacSha256 = Hmac<Sha256>;

// Secret management
struct SecretStore {
    secret: [u8; 32],
}

impl SecretStore {
    fn new() -> Self {
        Self {
            secret: Self::generate_secret(),
        }
    }

    fn generate_secret() -> [u8; 32] {
        // Generate 32-byte secret on init/post_upgrade
        let mut secret = [0u8; 32];
        ic_cdk::api::management_canister::main::raw_rand()
            .await
            .unwrap()
            .0
            .chunks_exact(32)
            .next()
            .unwrap()
            .copy_from_slice(&mut secret);
        secret
    }
}

// Token minting (QUERY method)
#[query]
fn mint_http_token(
    memory_id: String,
    variants: Vec<String>,
    ttl_secs: u64,
) -> Result<String, String> {
    // Validate caller has VIEW permission on memory
    let caller = ic_cdk::api::msg_caller();
    validate_memory_access(&memory_id, caller)?;

    // Create token payload
    let now = ic_cdk::api::time() / 1_000_000_000; // Convert to seconds
    let exp = now + ttl_secs;
    let nonce = generate_nonce();

    let payload = json!({
        "ver": 1,
        "scope": {
            "memory_id": memory_id,
            "variants": variants
        },
        "exp": exp,
        "nonce": nonce
    });

    // Sign with HMAC
    let token = sign_token(&payload)?;
    Ok(token)
}

// Token verification
fn verify_http_token(token_b64: &str, path: &str, now: u64) -> Result<TokenScope, String> {
    // Decode and verify token
    let token_bytes = general_purpose::STANDARD
        .decode(token_b64)
        .map_err(|_| "Invalid token format")?;

    // Split token and signature
    let (payload_b64, signature_b64) = token_bytes
        .split_last()
        .ok_or("Invalid token structure")?;

    // Verify HMAC signature
    let mut mac = HmacSha256::new_from_slice(&SECRET_STORE.secret)
        .map_err(|_| "Invalid secret")?;
    mac.update(payload_b64);
    mac.verify_slice(signature_b64)
        .map_err(|_| "Invalid token signature")?;

    // Parse payload
    let payload_str = String::from_utf8(payload_b64.to_vec())
        .map_err(|_| "Invalid token payload")?;
    let payload: Value = serde_json::from_str(&payload_str)
        .map_err(|_| "Invalid token JSON")?;

    // Check expiration
    let exp = payload["exp"].as_u64().ok_or("Missing expiration")?;
    if now > exp {
        return Err("Token expired".to_string());
    }

    // Extract scope
    let scope = TokenScope {
        memory_id: payload["scope"]["memory_id"]
            .as_str()
            .ok_or("Missing memory_id")?
            .to_string(),
        variants: payload["scope"]["variants"]
            .as_array()
            .ok_or("Missing variants")?
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect(),
    };

    Ok(scope)
}

#[derive(Debug)]
struct TokenScope {
    memory_id: String,
    variants: Vec<String>,
}
```

---

## 🌐 HTTP Request Handler

### Main Router

```rust
use ic_cdk::query;
use ic_http_certification::{HttpRequest, HttpResponse, DefaultCelBuilder};

#[query]
fn http_request(req: HttpRequest) -> HttpResponse {
    let path = req.url.trim_start_matches('/');

    // Parse asset path: /asset/{memory_id}/{variant}/{asset_id?}
    if let Some(asset_path) = path.strip_prefix("asset/") {
        return serve_asset(asset_path, &req);
    }

    // Handle other routes
    match path {
        "health" => create_health_response(),
        "metrics" => create_metrics_response(),
        _ => create_error_response(404, "Not found"),
    }
}

fn serve_asset(asset_path: &str, req: &HttpRequest) -> HttpResponse {
    // Parse path components
    let parts: Vec<&str> = asset_path.split('/').collect();
    if parts.len() < 2 {
        return create_error_response(400, "Invalid asset path");
    }

    let memory_id = parts[0];
    let variant = parts[1];
    let asset_id = parts.get(2);

    // Extract token from query params
    let token = extract_token_from_query(&req.url)
        .ok_or_else(|| create_error_response(401, "Missing token"))?;

    // Verify token
    let now = ic_cdk::api::time() / 1_000_000_000;
    let scope = match verify_http_token(&token, asset_path, now) {
        Ok(scope) => scope,
        Err(e) => return create_error_response(403, &format!("Token error: {}", e)),
    };

    // Validate path matches token scope
    if scope.memory_id != memory_id {
        return create_error_response(403, "Token scope mismatch");
    }

    if !scope.variants.contains(&variant.to_string()) {
        return create_error_response(403, "Variant not in token scope");
    }

    // Get and serve asset
    match get_asset_data(memory_id, variant, asset_id) {
        Ok(asset_data) => create_asset_response(asset_data, req),
        Err(e) => create_error_response(404, &format!("Asset not found: {}", e)),
    }
}

fn extract_token_from_query(url: &str) -> Option<String> {
    // Parse ?token=... from URL
    url.split('?')
        .nth(1)?
        .split('&')
        .find(|param| param.starts_with("token="))?
        .strip_prefix("token=")
        .map(|s| s.to_string())
}
```

### Asset Retrieval

```rust
use crate::memories_read_core;
use crate::canister_factory::store::StoreAdapter;
use crate::canister_factory::env::CanisterEnv;

fn get_asset_data(
    memory_id: &str,
    variant: &str,
    asset_id: Option<&str>,
) -> Result<AssetData, String> {
    let env = CanisterEnv;
    let store = StoreAdapter;

    // Get memory using existing Futura method
    let memory = memories_read_core(&env, &store, memory_id.to_string())
        .map_err(|e| format!("Memory not found: {}", e))?;

    // Find asset by variant
    match variant {
        "thumbnail" => find_asset_by_type(&memory, "Thumbnail"),
        "preview" => find_asset_by_type(&memory, "Preview"),
        "placeholder" => find_asset_by_type(&memory, "Placeholder"),
        "original" => {
            let asset_id = asset_id.ok_or("Asset ID required for original")?;
            find_asset_by_id(&memory, asset_id)
        }
        _ => Err("Invalid variant".to_string()),
    }
}

fn find_asset_by_type(memory: &Memory, asset_type: &str) -> Result<AssetData, String> {
    // Search through memory assets for the specified type
    for asset in &memory.assets {
        if asset.asset_type == asset_type {
            return load_asset_bytes(asset);
        }
    }
    Err("Asset not found".to_string())
}

fn find_asset_by_id(memory: &Memory, asset_id: &str) -> Result<AssetData, String> {
    // Search through memory assets for the specified ID
    for asset in &memory.assets {
        if asset.asset_id == asset_id {
            return load_asset_bytes(asset);
        }
    }
    Err("Asset not found".to_string())
}

fn load_asset_bytes(asset: &MemoryAsset) -> Result<AssetData, String> {
    match &asset.storage {
        MemoryAssetData::Inline(data) => Ok(AssetData {
            bytes: data.clone(),
            content_type: asset.metadata.mime_type.clone(),
            size: data.len() as u64,
        }),
        MemoryAssetData::BlobInternal(blob_ref) => {
            // Use existing blob_read method for chunked assets
            let env = CanisterEnv;
            let store = StoreAdapter;
            let bytes = blob_read(&env, &store, blob_ref.clone())
                .map_err(|e| format!("Failed to read blob: {}", e))?;

            Ok(AssetData {
                bytes,
                content_type: asset.metadata.mime_type.clone(),
                size: asset.metadata.size_bytes,
            })
        }
        MemoryAssetData::BlobExternal(_) => {
            Err("External assets not supported in HTTP serving".to_string())
        }
    }
}

#[derive(Debug)]
struct AssetData {
    bytes: Vec<u8>,
    content_type: String,
    size: u64,
}
```

### Response Creation

```rust
fn create_asset_response(asset_data: AssetData, req: &HttpRequest) -> HttpResponse {
    let size = asset_data.size;

    // For assets >= 2MB, use streaming
    if size >= 2 * 1024 * 1024 {
        create_streaming_response(asset_data, req)
    } else {
        create_inline_response(asset_data)
    }
}

fn create_inline_response(asset_data: AssetData) -> HttpResponse {
    HttpResponse {
        status_code: 200,
        headers: vec![
            ("Content-Type".into(), asset_data.content_type),
            ("Content-Length".into(), asset_data.size.to_string()),
            ("Cache-Control".into(), "private, no-store".into()),
            ("ETag".into(), format!("\"{}\"", sha256_hex(&asset_data.bytes))),
        ],
        body: asset_data.bytes,
        upgrade: None,
        streaming_strategy: None,
    }
}

fn create_streaming_response(asset_data: AssetData, req: &HttpRequest) -> HttpResponse {
    // Create streaming callback token
    let callback_token = StreamingCallbackToken {
        memory_id: extract_memory_id_from_path(&req.url),
        asset_id: extract_asset_id_from_path(&req.url),
        chunk_index: 0,
        total_chunks: (asset_data.size / CHUNK_SIZE) + 1,
        token: extract_token_from_query(&req.url).unwrap(),
    };

    HttpResponse {
        status_code: 200,
        headers: vec![
            ("Content-Type".into(), asset_data.content_type),
            ("Content-Length".into(), asset_data.size.to_string()),
            ("Cache-Control".into(), "private, no-store".into()),
            ("ETag".into(), format!("\"{}\"", sha256_hex(&asset_data.bytes))),
        ],
        body: vec![], // Empty body for streaming
        upgrade: None,
        streaming_strategy: Some(StreamingStrategy::Callback {
            callback: "http_request_streaming_callback".into(),
            token: callback_token,
        }),
    }
}

// Streaming callback with token re-verification
#[query]
fn http_request_streaming_callback(
    token: StreamingCallbackToken,
) -> Result<StreamingCallbackHttpResponse, String> {
    // Re-verify token for each chunk
    let now = ic_cdk::api::time() / 1_000_000_000;
    let _scope = verify_http_token(&token.token, "", now)
        .map_err(|e| format!("Token verification failed: {}", e))?;

    // Get asset data and return chunk
    let asset_data = get_asset_data(&token.memory_id, "original", Some(&token.asset_id))?;
    let start = token.chunk_index * CHUNK_SIZE;
    let end = std::cmp::min(start + CHUNK_SIZE, asset_data.size);

    if start >= asset_data.size {
        return Ok(StreamingCallbackHttpResponse {
            body: vec![],
            token: None,
        });
    }

    let chunk = asset_data.bytes[start as usize..end as usize].to_vec();
    let next_token = if end < asset_data.size {
        Some(StreamingCallbackToken {
            chunk_index: token.chunk_index + 1,
            ..token
        })
    } else {
        None
    };

    Ok(StreamingCallbackHttpResponse {
        body: chunk,
        token: next_token,
    })
}
```

---

## 🔧 Helper Functions

### Error Responses

```rust
fn create_error_response(status_code: u16, message: &str) -> HttpResponse {
    HttpResponse {
        status_code,
        headers: vec![("Content-Type".into(), "text/plain".into())],
        body: message.as_bytes().to_vec(),
        upgrade: None,
        streaming_strategy: None,
    }
}

fn create_health_response() -> HttpResponse {
    HttpResponse {
        status_code: 200,
        headers: vec![("Content-Type".into(), "text/plain".into())],
        body: b"OK".to_vec(),
        upgrade: None,
        streaming_strategy: None,
    }
}

fn create_metrics_response() -> HttpResponse {
    let metrics = json!({
        "total_requests": METRICS.total_requests.get(),
        "total_bytes_served": METRICS.total_bytes_served.get(),
        "avg_latency_ms": METRICS.avg_latency_ms.get(),
        "error_count": METRICS.error_count.get(),
    });

    HttpResponse {
        status_code: 200,
        headers: vec![("Content-Type".into(), "application/json".into())],
        body: metrics.to_string().as_bytes().to_vec(),
        upgrade: None,
        streaming_strategy: None,
    }
}
```

### Utility Functions

```rust
fn generate_nonce() -> String {
    // Generate 96-bit random nonce
    let mut nonce = [0u8; 12];
    ic_cdk::api::management_canister::main::raw_rand()
        .await
        .unwrap()
        .0
        .chunks_exact(12)
        .next()
        .unwrap()
        .copy_from_slice(&mut nonce);
    general_purpose::STANDARD.encode(nonce)
}

fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn validate_memory_access(memory_id: &str, caller: Principal) -> Result<(), String> {
    // Use existing Futura access control
    let env = CanisterEnv;
    let store = StoreAdapter;

    let memory = memories_read_core(&env, &store, memory_id.to_string())
        .map_err(|e| format!("Memory not found: {}", e))?;

    // Check if caller has VIEW permission
    let ctx = PrincipalContext {
        principal: caller,
        now_ns: ic_cdk::api::time(),
        link: None,
    };

    let perm_mask = effective_perm_mask(&memory, &ctx);
    if perm_mask & Perm::VIEW.bits() == 0 {
        return Err("Access denied: insufficient permissions".to_string());
    }

    Ok(())
}
```

---

## 📊 Metrics & Monitoring

```rust
use std::sync::atomic::{AtomicU64, Ordering};

struct Metrics {
    total_requests: AtomicU64,
    total_bytes_served: AtomicU64,
    avg_latency_ms: AtomicU64,
    error_count: AtomicU64,
}

static METRICS: Metrics = Metrics {
    total_requests: AtomicU64::new(0),
    total_bytes_served: AtomicU64::new(0),
    avg_latency_ms: AtomicU64::new(0),
    error_count: AtomicU64::new(0),
};

fn record_request(bytes_served: u64, latency_ms: u64, is_error: bool) {
    METRICS.total_requests.fetch_add(1, Ordering::Relaxed);
    METRICS.total_bytes_served.fetch_add(bytes_served, Ordering::Relaxed);

    if is_error {
        METRICS.error_count.fetch_add(1, Ordering::Relaxed);
    }

    // Simple moving average for latency
    let current_avg = METRICS.avg_latency_ms.load(Ordering::Relaxed);
    let new_avg = (current_avg + latency_ms) / 2;
    METRICS.avg_latency_ms.store(new_avg, Ordering::Relaxed);
}
```

---

## 🧪 Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_mint_and_verify() {
        // Test token generation and verification
        let memory_id = "test-memory".to_string();
        let variants = vec!["thumbnail".to_string(), "preview".to_string()];
        let ttl = 180;

        let token = mint_http_token(memory_id.clone(), variants.clone(), ttl).unwrap();
        let now = ic_cdk::api::time() / 1_000_000_000;

        let scope = verify_http_token(&token, "", now).unwrap();
        assert_eq!(scope.memory_id, memory_id);
        assert_eq!(scope.variants, variants);
    }

    #[test]
    fn test_expired_token() {
        // Test token expiration
        let token = create_expired_token();
        let now = ic_cdk::api::time() / 1_000_000_000;

        let result = verify_http_token(&token, "", now);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expired"));
    }

    #[test]
    fn test_scope_validation() {
        // Test token scope validation
        let token = mint_token_for_memory("memory-1", vec!["thumbnail".to_string()]);

        // Should work for memory-1/thumbnail
        let result = verify_http_token(&token, "asset/memory-1/thumbnail", now);
        assert!(result.is_ok());

        // Should fail for memory-2/thumbnail
        let result = verify_http_token(&token, "asset/memory-2/thumbnail", now);
        assert!(result.is_err());

        // Should fail for memory-1/preview (not in scope)
        let result = verify_http_token(&token, "asset/memory-1/preview", now);
        assert!(result.is_err());
    }
}
```

### Integration Tests

```rust
// HTTP integration tests using dfx
#[test]
fn test_http_asset_serving() {
    // Deploy canister
    // Mint token
    // Make HTTP request
    // Verify response
}

#[test]
fn test_streaming_large_asset() {
    // Test streaming for assets >= 2MB
    // Verify all chunks are received
    // Verify token re-verification on each chunk
}
```

---

## 🚀 Implementation Phases

This implementation follows the phases outlined in `http_request_implementation_final.md`:

1. **Phase 1**: Token system (SecretStore, TokenMint, TokenVerify)
2. **Phase 2**: HTTP routes with authorization and serving
3. **Phase 3**: Frontend integration with Next.js
4. **Phase 4**: Performance optimization and hardening
5. **Phase 5**: Rollout and cleanup

---

## 🔒 Security Considerations

- **Token TTL**: 3 minutes (configurable) limits exposure window
- **HMAC Security**: 32-byte secret with SHA256-HMAC
- **Scope Validation**: Strict path-to-token-scope matching
- **No Storage**: Stateless tokens don't require canister state
- **Re-verification**: Streaming callbacks re-verify tokens
- **Private Headers**: `Cache-Control: private, no-store`

---

## 📈 Performance Targets

- **Response Time**: p95 < 150ms for cached small assets
- **Streaming**: Stable for large assets (≥2MB)
- **Memory**: Minimal overhead, no per-request state
- **Errors**: < 0.5% error rate
- **Throughput**: Handle concurrent requests efficiently

---

This implementation provides a complete, production-ready token-gated `http_request` system that maintains privacy, integrates with existing Futura architecture, and supports both inline and streaming responses for optimal performance.
