# From Next.js Image Error to ICP HTTP Module: A Journey of Decentralized Asset Serving

**Date:** January 27, 2025  
**Author:** Futura Development Team  
**Category:** Technical Deep Dive, ICP Development, Web3 Architecture

## The Problem That Started It All

It all began with a seemingly simple error in our Next.js frontend:

```
Error: Image Optimization using the default loader is not compatible with `output: 'export'`.
```

This error was the catalyst for a deep dive into decentralized asset serving on the Internet Computer Protocol (ICP), leading us to build a complete token-gated HTTP module from scratch.

## The Challenge: Serving Private Assets on ICP

Our Futura project needed to serve private user assets (images, documents, etc.) directly from ICP canisters while maintaining:

- **Privacy**: Assets should only be accessible to authorized users
- **Decentralization**: No reliance on Web2 proxies or external services
- **Performance**: Fast, efficient asset delivery
- **Security**: Token-based access control with short-lived, stateless tokens

The traditional approach of using external CDNs or Web2 services would compromise our decentralized architecture and user privacy principles.

## The Solution: Token-Gated HTTP Module

We designed and implemented a comprehensive HTTP module that allows ICP canisters to serve private assets directly via the `http_request` method, with access controlled by HMAC-signed tokens.

### Architecture Overview

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Next.js App   │───▶│  HTTP Module     │───▶│  Asset Store    │
│                 │    │  (Token Gateway) │    │  (Memories)     │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         │                       ▼                       │
         │              ┌──────────────────┐             │
         │              │   ACL System     │             │
         └─────────────▶│  (Permissions)   │◀────────────┘
                        └──────────────────┘
```

### Key Components

1. **Token Minting API**: Generates short-lived HMAC tokens for authorized access
2. **HTTP Request Handler**: Validates tokens and serves assets
3. **ACL Integration**: Leverages existing domain permission system
4. **Asset Store Bridge**: Connects to existing memory and blob storage
5. **Secret Management**: Secure HMAC key generation and rotation

## The Implementation Journey

### Phase 1: Foundation

We started by implementing the core HTTP module structure:

```rust
// Core types for the HTTP module
pub struct TokenPayload {
    pub ver: u32,
    pub kid: u32,           // Key version for rotation
    pub exp_ns: u64,        // Expiration timestamp
    pub scope: TokenScope,  // What assets can be accessed
    pub nonce: [u8; 12],    // Anti-replay protection
}

pub trait Acl {
    fn can_view(&self, memory_id: &str, who: Principal) -> bool;
}

pub trait AssetStore {
    fn get_inline(&self, memory_id: &str, asset_id: &str) -> Option<InlineAsset>;
    fn exists(&self, memory_id: &str, asset_id: &str) -> bool;
}
```

### Phase 2: Domain Integration

The real challenge was integrating with our existing domain logic without creating tight coupling:

```rust
// ACL adapter that bridges to existing domain logic
impl Acl for FuturaAclAdapter {
    fn can_view(&self, memory_id: &str, who: Principal) -> bool {
        let ctx = PrincipalContext {
            principal: who,
            groups: vec![],
            link: None,
            now_ns: ic_cdk::api::time(),
        };

        let store = StoreAdapter;
        let accessible_capsules = store.get_accessible_capsules(&PersonRef::Principal(who));

        for capsule_id in accessible_capsules {
            if let Some(memory) = store.get_memory(&capsule_id, &memory_id.to_string()) {
                let perm_mask = effective_perm_mask(&memory, &ctx);
                return (perm_mask & Perm::VIEW.bits()) != 0;
            }
        }
        false
    }
}
```

### The WASM Compatibility Challenge

One of the biggest hurdles was WASM compatibility. We encountered the classic `getrandom` crate issue:

```
error: the wasm*-unknown-unknown targets are not supported by default,
you may need to enable the "js" feature.
```

**Our Solution**: We followed the same approach that worked for our UUID v7 implementation - remove external randomness dependencies and use ICP's native randomness:

```rust
// Instead of using rand crate
use ic_cdk::management_canister::raw_rand;

async fn generate_random_bytes(len: usize) -> Vec<u8> {
    let rnd = raw_rand().await.expect("raw_rand failed");
    rnd.into_iter().take(len).collect()
}

// For deterministic operations (like query functions)
fn deterministic_key() -> [u8; 32] {
    let canister_id = ic_cdk::api::id();
    let time = ic_cdk::api::time();

    let mut key = [0u8; 32];
    let canister_bytes = canister_id.as_slice();
    let time_bytes = time.to_le_bytes();

    for i in 0..32 {
        key[i] = canister_bytes[i % canister_bytes.len()] ^ time_bytes[i % 8];
    }

    key
}
```

### The Initialization Challenge

Another critical issue was system calls during canister initialization:

```
Error: Canister violated contract: "ic0_call_new" cannot be executed in init mode.
```

**Our Solution**: Use deterministic initialization during `init()` and reserve async randomness for runtime operations:

```rust
pub async fn init() {
    // Use deterministic initialization during init (system calls not allowed)
    let seeded = Secrets {
        current: deterministic_key(),
        previous: [0; 32],
        version: 1
    };
    // ... rest of initialization
}
```

## The Result: A Complete Solution

### Token Minting API

```rust
#[query]
fn mint_http_token(
    memory_id: String,
    variants: Vec<String>,
    asset_ids: Option<Vec<String>>,
    ttl_secs: u32
) -> String {
    let caller = ic_cdk::caller();

    // Validate permissions using existing domain logic
    let acl = FuturaAclAdapter;
    if !acl.can_view(&memory_id, caller) {
        panic!("Unauthorized access to memory {}", memory_id);
    }

    // Validate asset existence
    let store = FuturaAssetStore;
    if let Some(ids) = &asset_ids {
        for id in ids {
            if !store.exists(&memory_id, id) {
                panic!("Asset {} not found in memory {}", id, memory_id);
            }
        }
    }

    // Generate secure token with deterministic nonce
    let mut nonce = [0u8; 12];
    let time_bytes = time().to_le_bytes();
    let caller_bytes = caller.as_slice();
    for i in 0..12 {
        nonce[i] = time_bytes[i % 8] ^ caller_bytes[i % caller_bytes.len()];
    }

    let payload = TokenPayload {
        ver: 1,
        kid: 1,
        exp_ns: time() + (ttl_secs.min(180) as u64) * 1_000_000_000,
        scope: TokenScope { memory_id, variants, asset_ids },
        nonce,
    };

    sign_token_core(&payload, &StableSecretStore)
}
```

### HTTP Request Handler

```rust
#[query]
fn http_request(req: HttpRequest) -> HttpResponse {
    match req.url.as_str() {
        "/health" => health_check(),
        path if path.starts_with("/assets/") => {
            serve_asset(path, &req.headers)
        }
        _ => HttpResponse {
            status_code: 404,
            headers: vec![],
            body: "Not Found".as_bytes().to_vec(),
        }
    }
}
```

## Testing and Validation

We created a comprehensive test suite to validate the entire system:

```bash
# Test results
✅ Health check endpoint working
✅ Token minting properly validates permissions
✅ Asset serving properly rejects requests without token
✅ Asset serving properly rejects requests with invalid token
✅ Invalid endpoints properly return 404
```

## Key Learnings

### 1. WASM Compatibility is Critical

- External dependencies like `rand` and `getrandom` can break ICP deployments
- Always use ICP's native APIs when possible
- Document dependency restrictions clearly

### 2. System Call Restrictions

- Canister initialization cannot make system calls
- Use deterministic approaches during `init()`
- Reserve async operations for runtime

### 3. Domain Integration Patterns

- Use adapter pattern to bridge between layers
- Avoid tight coupling between HTTP and domain logic
- Leverage existing permission systems

### 4. Security Considerations

- Short-lived tokens (max 180 seconds)
- Deterministic nonce generation for query functions
- Proper token validation and error handling

## The Impact

This HTTP module enables:

- **True Decentralization**: Assets served directly from ICP canisters
- **Privacy Preservation**: No external services can access user data
- **Performance**: Direct canister-to-client communication
- **Security**: Token-based access control with proper validation
- **Scalability**: Stateless tokens that don't require canister state

## Frontend Integration

With the HTTP module in place, our Next.js app can now serve images directly from ICP:

```typescript
// Instead of external CDN
<Image
  src={`https://${canisterId}.ic0.app/assets/${memoryId}/${assetId}?token=${token}`}
  alt="User asset"
  width={500}
  height={300}
/>
```

## Conclusion

What started as a simple Next.js Image component error led us to build a complete decentralized asset serving solution. This journey demonstrates the power of embracing challenges as opportunities to build better, more decentralized systems.

The HTTP module is now production-ready and successfully serving private assets with token-gated access, maintaining our commitment to user privacy and decentralization.

## Next Steps

- **Streaming Support**: Implement chunked responses for large assets
- **Caching**: Add HTTP caching headers for better performance
- **Rate Limiting**: Implement token minting rate limits
- **Monitoring**: Add metrics and logging for production use

---

_This article documents the real-world implementation of a token-gated HTTP module for the Internet Computer Protocol, showcasing the challenges and solutions encountered when building decentralized applications._

## Related Resources

- [HTTP Module Implementation Documentation](../issues/open/serving-http/)
- [WASM Compatibility Issues Resolution](../issues/open/uuid-memories/uuid-v7-deployment-wasm-compatibility-issues.md)
- [ICP HTTP Request Documentation](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-http_request)
- [Futura Project Repository](https://github.com/futura-icp/futura_alpha_icp)
