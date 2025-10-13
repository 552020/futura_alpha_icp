# HTTP Certification 503 Error Analysis

**Status**: 🔴 **CRITICAL** - Blocking HTTP module functionality  
**Priority**: **HIGH** - Prevents asset serving via HTTP gateway  
**Date**: 2025-01-12  
**Reporter**: Development Team

## 🚨 **Problem Summary**

All HTTP gateway requests are returning `503 - response verification error`, preventing the HTTP module from serving assets over the Internet Computer's HTTP gateway. This blocks the core functionality of serving images and other assets via HTTP.

**✅ SOLUTION PROVIDED**: Tech lead has provided the exact fix - we need to implement "skip certification" for private assets. See [HTTP Certification Requirement Clarification](./http-certification-requirement-clarification.md) for the complete solution and implementation steps.

## 🔍 **Detailed Analysis**

### **Error Details**

- **Error Code**: `503 - response verification error`
- **Error Message**: "The response from the canister failed verification and cannot be trusted"
- **Affected Endpoints**: ALL HTTP gateway requests
- **Working Endpoints**: dfx canister calls work fine

### **Test Results**

#### ✅ **Working Components**

- **Memory Creation**: Successfully creates memories with assets using proper utilities
- **Token Minting**: Properly validates permissions and returns "forbidden" as expected
- **dfx Canister Calls**: All backend functionality accessible via `dfx canister call`
- **ACL Integration**: Access control is working correctly

#### ❌ **Failing Components**

- **HTTP Gateway Requests**: All return 503 response verification error
- **Asset Serving**: Cannot serve assets via HTTP
- **Health Check**: Even basic `/health` endpoint fails via HTTP gateway
- **Browser Tests**: Blocked by HTTP certification issue
- **Next.js Integration**: Blocked by HTTP certification issue

### **Test Evidence**

```bash
# Working: dfx canister call (direct canister method call)
$ dfx canister call backend http_request '(record { method = "GET"; url = "/health"; headers = vec {}; body = blob "" })'
# Returns: (record { body = blob "OK"; headers = vec { record { "Content-Type"; "text/plain" } }; status_code = 200 : nat16; upgrade = null })

# Working: Token minting (backend logic works)
$ dfx canister call backend mint_http_token '("memory_123", vec {"thumbnail"}, null, 180)'
# Returns: "forbidden" (expected - ACL correctly blocks unauthorized access)

# Failing: HTTP gateway request (503 response verification error)
$ curl -i "http://uxrrr-q7777-77774-qaaaq-cai.localhost:4943/health"
# Returns: 503 - response verification error

# Failing: Asset serving via HTTP gateway
$ curl -i "http://uxrrr-q7777-77774-qaaaq-cai.localhost:4943/asset/memory_123/thumbnail?token=..."
# Returns: 503 - response verification error
```

## 🔧 **Technical Investigation**

### **HTTP Module Implementation**

The HTTP module is implemented using `ic-http-certification` v3.0.3 with a 3-layer architecture:

```rust
// Main HTTP entrypoint router (src/backend/src/http.rs)
pub fn handle(req: HttpRequest) -> HttpResponse<'static> {
    let parsed = match parse(req) {
        Ok(p) => p,
        Err(r) => return r,
    };

    match (parsed.method.as_str(), parsed.path_segments.as_slice()) {
        ("GET", [health]) if health == "health" => health_route::get(&parsed),
        ("GET", [asset, mem, var]) if asset == "asset" => assets_route::get(mem, var, &parsed),
        _ => HttpResponse::builder()
            .with_status_code(StatusCode::NOT_FOUND)
            .with_headers(vec![("Content-Type".into(), "text/plain".into())])
            .with_body(b"Not Found")
            .build(),
    }
}

// Health route implementation (src/backend/src/http/routes/health.rs)
pub fn get(_: &ParsedRequest) -> HttpResponse<'static> {
    HttpResponse::ok(
        b"OK",
        vec![("Content-Type".into(), "text/plain".into())]
    ).build()
}
```

### **Response Structure**

The HTTP responses are constructed using the builder pattern with proper error handling:

```rust
// Asset serving response (src/backend/src/http/routes/assets.rs)
HttpResponse::ok(
    inline.bytes,
    vec![
        ("Content-Type".into(), inline.content_type),
        ("Cache-Control".into(), "private, no-store".into()),
        ("Content-Length".into(), content_length),
    ]
).build()

// Error response helper
fn status(code: u16, msg: &str) -> HttpResponse<'static> {
    let status_code = StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let body = msg.as_bytes().to_vec();
    HttpResponse::builder()
        .with_status_code(status_code)
        .with_headers(vec![("Content-Type".into(), "text/plain".into())])
        .with_body(body)
        .build()
}
```

### **Architecture Overview**

The HTTP module follows a clean 3-layer architecture:

1. **Core Layer** (`src/backend/src/http/core/`):

   - `types.rs`: Pure business logic types (TokenPayload, TokenScope, traits)
   - `auth_core.rs`: Token signing/verification logic
   - `path_core.rs`: URL parsing and scope validation

2. **Adapter Layer** (`src/backend/src/http/adapters/`):

   - `canister_env.rs`: ICP environment integration (time, caller)
   - `secret_store.rs`: Secret management with StableCell
   - `asset_store.rs`: Asset retrieval from memories/blob store
   - `acl.rs`: Access control integration with existing domain

3. **Route Layer** (`src/backend/src/http/routes/`):
   - `health.rs`: Health check endpoint
   - `assets.rs`: Asset serving with token authentication

### **Domain Integration**

The HTTP module is fully integrated with the existing domain:

- **ACL**: Uses `effective_perm_mask()` for permission validation
- **Asset Storage**: Connects to existing memory/blob store APIs
- **Token System**: HMAC-based stateless authentication with key rotation

### **Route Structure**

The HTTP module supports the following routes:

1. **Health Check**: `GET /health`

   - Returns: `200 OK` with `"OK"` body
   - Purpose: Basic canister health verification

2. **Asset Serving**: `GET /asset/{memory_id}/{variant}?token=...&id={asset_id}`
   - Returns: Asset bytes with proper Content-Type and Cache-Control headers
   - Authentication: Required token parameter with HMAC verification
   - Variants: `thumbnail`, `preview`, `original`
   - Error Responses: `401` (missing/expired token), `403` (invalid token), `404` (asset not found)

### **Token Authentication System**

The module implements a sophisticated token-based authentication system:

```rust
// Token payload structure
pub struct TokenPayload {
    pub ver: u8,           // Version
    pub kid: u32,          // Key ID for rotation
    pub exp_ns: u64,       // Expiration timestamp
    pub nonce: [u8; 12],   // Nonce for uniqueness
    pub scope: TokenScope, // Access scope
    pub sub: Option<Principal>, // Subject (caller)
}

// Token scope defines what the token can access
pub struct TokenScope {
    pub memory_id: String,           // Which memory
    pub variants: Vec<String>,       // Which variants (thumbnail, preview, original)
    pub asset_ids: Option<Vec<String>>, // Optional specific asset IDs
}
```

**Token Minting**: Via `mint_http_token` query method with ACL validation
**Token Verification**: HMAC signature verification with expiration and scope checks
**Key Rotation**: Support for multiple key versions with `kid` field

## 🤔 **Hypotheses**

### **1. Missing HTTP Certification Response** ⭐ **MOST LIKELY**

**Evidence**: Our code analysis shows:

- ❌ **No `set_certified_data()` calls** found in the codebase
- ❌ **No `HttpCertificationTree` usage** found in the codebase
- ❌ **No `IC-Certificate` headers** found in our HTTP responses
- ❌ **No certification tree initialization** in `init()` or `post_upgrade()`

**Root Cause**: We're returning raw `HttpResponse` objects without any HTTP certification response. The HTTP gateway expects some form of certification response (even if it's "skip certification") but we're not providing any certification infrastructure.

**⚠️ CLARIFICATION NEEDED**: We need to determine if we should use skip certification for private assets or implement full certification. See [HTTP Certification Requirement Clarification](./http-certification-requirement-clarification.md).

### **2. Missing Certification Headers** ⭐ **HIGHLY LIKELY**

**Evidence**: Our HTTP responses only include:

```rust
// Current response headers
vec![
    ("Content-Type".into(), inline.content_type),
    ("Cache-Control".into(), "private, no-store".into()),
    ("Content-Length".into(), content_length),
]
```

**Missing**: Required HTTP certification headers:

- `IC-Certificate`: The actual certification proof
- `IC-CertificateExpression`: The CEL expression defining what's certified

### **3. No Certification Tree Management**

**Evidence**: Our implementation is missing:

- HTTP certification tree initialization
- CEL expression definitions
- Certification tree updates for responses
- Proper certification lifecycle management

### **4. Using Wrong HTTP Certification Approach**

**Evidence**: We're using `ic-http-certification` v3.0.3 but may be:

- Using the wrong certification pattern (full vs response-only vs skip)
- Not following the proper certification workflow
- Missing the certification tree setup entirely

### **5. Local Replica HTTP Gateway Issue**

- Local replica may not be properly configured for HTTP certification
- Gateway configuration issue with the local development environment
- Version mismatch between local replica and HTTP certification

## 🔍 **Technical Analysis for Tech Lead**

### **Code Review Findings**

After reviewing our HTTP module implementation, here are the specific issues:

#### **1. Missing HTTP Certification Infrastructure** ⭐ **CRITICAL**

Our current implementation is **completely missing** HTTP certification setup:

```rust
// Current init() - ONLY initializes secret store
#[ic_cdk::init]
async fn init() {
    http::secret_store::init().await;  // ✅ Secret store works
    // ❌ MISSING: HTTP certification tree setup
    // ❌ MISSING: set_certified_data() call
    // ❌ MISSING: CEL expression definitions
}

// Current http_request - returns raw HttpResponse
#[ic_cdk::query]
fn http_request(req: HttpRequest) -> HttpResponse<'static> {
    http::handle(req)  // ❌ Returns uncertified response
}
```

#### **2. Missing Required Certification Headers**

Our HTTP responses are missing the essential certification headers:

```rust
// Current response (src/backend/src/http/routes/assets.rs)
HttpResponse::ok(
    inline.bytes,
    vec![
        ("Content-Type".into(), inline.content_type),        // ✅ Present
        ("Cache-Control".into(), "private, no-store".into()), // ✅ Present
        ("Content-Length".into(), content_length),           // ✅ Present
        // ❌ MISSING: "IC-Certificate" header
        // ❌ MISSING: "IC-CertificateExpression" header
    ]
).build()
```

#### **3. No Certification Tree Management**

Our codebase has **zero** HTTP certification tree management:

- No `HttpCertificationTree` usage
- No CEL expression definitions
- No certification tree initialization
- No `set_certified_data()` calls

### **Specific Technical Questions for Tech Lead**

1. **Certification Strategy**: Should we use:

   - **Full certification** (request + response certified)
   - **Response-only certification** (only response certified)
   - **Skip certification** (no certification, just HTTPS)

2. **Implementation Approach**: Should we:

   - Add certification tree to existing HTTP module
   - Implement skip certification for private assets
   - Use a hybrid approach (certified public, uncertified private)

3. **CEL Expression**: What CEL expression should we use for:

   - Health check endpoint (`/health`)
   - Asset serving endpoint (`/asset/{memory_id}/{variant}`)

4. **Certification Lifecycle**: How should we handle:
   - Certification tree initialization in `init()`
   - Certification tree updates in `post_upgrade()`
   - Dynamic certification for changing responses

## 🛠️ **Recommended Solutions**

### **Immediate Actions**

1. **Implement HTTP Certification Tree**: Add proper certification tree setup in canister init
2. **Add Certification Headers**: Include required `IC-Certificate` and `IC-CertificateExpression` headers
3. **Choose Certification Strategy**: Decide between full, response-only, or skip certification
4. **Test with Skip Certification**: Implement skip certification as a quick fix to isolate the issue

### **Quick Fix Option: Skip Certification**

For private assets, we could implement skip certification as an immediate solution:

```rust
// Add to init()
#[ic_cdk::init]
async fn init() {
    http::secret_store::init().await;

    // Skip certification for private assets
    set_certified_data(&skip_certification_certified_data());
}

// Add to HTTP responses
use ic_http_certification::utils::add_skip_certification_header;

fn serve_asset_with_skip_cert(req: &HttpRequest) -> HttpResponse<'static> {
    let mut response = http::handle(req);

    // Add skip certification header
    add_skip_certification_header(
        data_certificate().expect("No data certificate available"),
        &mut response
    );

    response
}
```

This would allow HTTP gateway requests to work immediately while we implement proper certification.

### **Investigation Steps**

1. **Add Certification Tree Initialization**: Implement proper HTTP certification tree setup in canister init
2. **Enable Debug Logging**: Add detailed logging to HTTP responses and certification process
3. **Compare with Working Examples**: Check other ICP projects with working HTTP certification
4. **Review Gateway Logs**: Check local replica logs for certification errors
5. **Test with Skip Certification**: Try implementing skip certification to isolate the issue

### **Potential Fixes**

1. **Implement Certification Tree**: Add proper HTTP certification tree initialization and management
2. **Add Certification Headers**: Include required `IC-Certificate` and `IC-CertificateExpression` headers
3. **Review Response Format**: Ensure responses match HTTP certification requirements
4. **Update Local Replica**: Check if local replica needs updates for HTTP certification support

## 📋 **Test Cases to Validate Fix**

Once the issue is resolved, we need to verify:

- [ ] Health check returns 200 OK via HTTP gateway
- [ ] Asset serving works with valid tokens
- [ ] Proper error responses (401, 403, 404) via HTTP gateway
- [ ] Response headers are correctly set
- [ ] Browser can render images directly
- [ ] Next.js Image component works

## 🔗 **Related Documentation**

- [HTTP Module Implementation](../architecture/http-module-architecture.md)
- [ic-http-certification API Analysis](./ic-http-certification-api-analysis.md)
- [Response Verification Repo Analysis](./response-verification-repo-analysis.md)
- [Tech Lead 9-Point Feedback](./tech-lead-9-point-feedback.md)

## 📊 **Impact Assessment**

### **Current Impact**

- **High**: Core HTTP functionality is completely blocked
- **User Experience**: Cannot serve assets via HTTP
- **Development**: Blocks browser and Next.js integration tests
- **Deployment**: HTTP module is non-functional

### **Business Impact**

- **Critical**: Asset serving is a core feature
- **User Adoption**: Users cannot access images and files
- **Performance**: Forces fallback to slower dfx canister calls

## 🎯 **Success Criteria**

The issue is resolved when:

1. **Health Check**: `GET /health` returns `200 OK` via HTTP gateway
2. **Asset Serving**: `GET /asset/{memory_id}/{variant}?token=...` works with valid tokens
3. **Error Handling**: Proper error responses (401, 403, 404) via HTTP gateway
4. **Response Headers**: Correct Content-Type, Cache-Control, and certification headers
5. **Browser Integration**: Browser can directly render images from HTTP URLs
6. **Next.js Integration**: Next.js Image component works with HTTP URLs
7. **All Tests Pass**: HTTP module tests pass without 503 errors

## 📝 **Next Steps**

1. **Tech Lead Review**: Review this analysis and provide guidance
2. **Investigation**: Follow recommended investigation steps
3. **Implementation**: Apply the appropriate fix
4. **Testing**: Validate fix with comprehensive test suite
5. **Documentation**: Update implementation docs with lessons learned

---

**Priority**: 🔴 **CRITICAL**  
**Estimated Effort**: 2-4 hours investigation + implementation  
**Dependencies**: Tech lead guidance on HTTP certification best practices  
**Blocking**: HTTP module functionality, browser tests, Next.js integration
