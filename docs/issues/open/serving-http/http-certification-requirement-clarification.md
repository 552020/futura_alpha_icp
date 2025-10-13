# HTTP Certification Requirement Clarification

**Status**: 🔴 **CRITICAL** - Need guidance on HTTP certification strategy  
**Priority**: **HIGH** - Blocking HTTP module functionality  
**Date**: 2025-01-12  
**Reporter**: Development Team

## 🚨 **Problem Summary**

We're getting `503 - response verification error` from the HTTP gateway, but we need to clarify whether we actually need HTTP certification for our use case. Our HTTP module is designed for **private asset serving** with token-based authentication, not public certified content.

## 🤔 **Core Question**

**Do we need HTTP certification for private assets with token authentication?**

## 📋 **Our Use Case**

### **Private Asset Serving**

- **Authentication**: Token-based HMAC authentication
- **Access Control**: ACL-based permission system
- **Content**: Private family/personal assets (photos, documents)
- **Caching**: We explicitly set `Cache-Control: private, no-store`
- **Security**: Assets are private and personalized per user

### **Current Implementation**

```rust
// Our HTTP responses for private assets
HttpResponse::ok(
    inline.bytes,
    vec![
        ("Content-Type".into(), inline.content_type),
        ("Cache-Control".into(), "private, no-store".into()),  // Private, no caching
        ("Content-Length".into(), content_length),
    ]
).build()
```

## 🔍 **HTTP Certification Options**

### **Option 1: Skip Certification** ⭐ **RECOMMENDED FOR PRIVATE ASSETS**

**When to use**: Private, personalized, or dynamic content
**Benefits**:

- Simpler implementation
- No certification tree management
- Appropriate for private assets
- Still secure via HTTPS

**Implementation**:

```rust
// Skip certification for private assets
set_certified_data(&skip_certification_certified_data());

// Add skip certification header to responses
add_skip_certification_header(data_certificate().unwrap(), &mut response);
```

### **Option 2: Response-Only Certification**

**When to use**: Public content that needs verification
**Benefits**:

- Cryptographic proof of response authenticity
- Can be cached by boundary nodes
- Good for public, immutable content

**Drawbacks**:

- More complex implementation
- Requires certification tree management
- Not suitable for private/personalized content

### **Option 3: Full Certification**

**When to use**: Public APIs with request/response verification
**Benefits**:

- Complete request/response verification
- Maximum security and caching

**Drawbacks**:

- Most complex implementation
- Not suitable for private assets
- Overkill for our use case

## 🎯 **Our Specific Questions**

### **1. Certification Strategy**

- Should we use **skip certification** for private assets with token authentication?
- Is HTTP certification required even for private content?
- Can we serve private assets without certification?

### **2. Security Model**

- Is token-based authentication sufficient for private assets?
- Does HTTPS + token auth provide adequate security without certification?
- Are there security implications of skipping certification for private content?

### **3. Implementation Approach**

- Should we implement skip certification as the primary approach?
- Do we need to support both certified and uncertified routes?
- How should we handle the certification headers?

### **4. ICP Best Practices**

- What's the recommended approach for private asset serving on ICP?
- Are there examples of successful private asset serving without certification?
- What are the performance implications of different approaches?

## 📊 **Current Evidence**

### **What's Working**

- ✅ Backend logic (memory creation, token minting, ACL)
- ✅ `dfx canister call` works perfectly
- ✅ Token authentication system is robust

### **What's Failing**

- ❌ HTTP gateway requests return 503
- ❌ Browser cannot access assets via HTTP URLs
- ❌ Next.js Image component blocked

### **Root Cause Hypothesis**

The HTTP gateway expects some form of certification response, but we're returning raw `HttpResponse` objects without any certification headers. The question is: **what type of certification response should we provide?**

## 🛠️ **Proposed Solution**

### **Immediate Fix: Skip Certification**

If skip certification is appropriate for private assets:

```rust
// Add to init()
#[ic_cdk::init]
async fn init() {
    http::secret_store::init().await;

    // Skip certification for private assets
    set_certified_data(&skip_certification_certified_data());
}

// Modify http_request to add skip certification header
#[ic_cdk::query]
fn http_request(req: HttpRequest) -> HttpResponse<'static> {
    let mut response = http::handle(req);

    // Add skip certification header
    add_skip_certification_header(
        data_certificate().expect("No data certificate available"),
        &mut response
    );

    response
}
```

## 📋 **Test Cases to Validate**

Once we implement the correct approach:

- [ ] Health check returns 200 OK via HTTP gateway
- [ ] Asset serving works with valid tokens via HTTP gateway
- [ ] Proper error responses (401, 403, 404) via HTTP gateway
- [ ] Browser can render images directly from HTTP URLs
- [ ] Next.js Image component works with HTTP URLs
- [ ] All HTTP tests pass without 503 errors

## 🎯 **Success Criteria**

The issue is resolved when:

1. **Clear Guidance**: Tech lead and ICP expert provide clear guidance on certification strategy
2. **Working Implementation**: HTTP gateway requests work without 503 errors
3. **Proper Security**: Private assets remain secure with appropriate authentication
4. **Performance**: Solution doesn't negatively impact performance
5. **Maintainability**: Implementation is simple and maintainable

## ✅ **Tech Lead Response - SOLUTION PROVIDED**

### **Short Answer**

For **private, token-gated** content you **don't need to certify the body**, but the HTTP gateway still expects a **certificate + expression header**. If you return a "raw" `HttpResponse` without those headers, the gateway can't validate anything and replies **503 response verification error**. The fix is to return a **"skip certification"** response: the body is _not_ certified, but the gateway can verify that _you explicitly chose to skip_ by checking your canister's certified data + expression.

### **Exact Implementation Steps**

#### **1. Set certified data on init / post_upgrade**

This tells replicas what root hash to sign for the _skip_ mode.

```rust
use ic_cdk::init;
use ic_cdk::post_upgrade;
use ic_http_certification::utils::skip_certification_certified_data;

#[init]
fn init() {
    ic_cdk::api::set_certified_data(&skip_certification_certified_data());
}

#[post_upgrade]
fn post_upgrade() {
    init();
}
```

#### **2. Add the "skip" certificate headers to every private HTTP response**

Use the helper that attaches both `IC-Certificate` (v2) and the expression `no_certification`.

```rust
use ic_cdk::query;
use ic_cdk::api::data_certificate;
use ic_http_certification::HttpResponse;
use ic_http_certification::utils::add_skip_certification_header;

#[query]
fn http_request(req: ic_http_certification::HttpRequest) -> HttpResponse<'static> {
    let mut resp = HttpResponse::builder()
        .with_status_code(200)
        .with_body(/* your bytes */)
        .with_headers(vec![
            ("content-type".into(), "image/webp".into()),
            ("cache-control".into(), "private, no-store".into()),
        ])
        .build();

    // <- critical line
    add_skip_certification_header(data_certificate().expect("no data cert"), &mut resp);

    resp
}
```

### **Answers to Our Questions**

- **Do we need HTTP certification for private assets with token auth?**
  You need **the certification headers**, but they can declare **`no_certification`** (skip). That's the supported way to serve private/dynamic content over the gateway.

- **Is HTTPS + token auth enough?**
  Yes—**for confidentiality and access control**. Certification is mainly integrity against replica misbehavior. For private, short-lived, per-user content, the official guidance allows **skip certification**; just be aware of the stated security trade-off.

- **Do we need both certified and uncertified routes?**
  Only if you later serve **public cacheable** assets. Private images can all use **skip** + `Cache-Control: private, no-store`.

### **Why We Saw 503**

The gateway's v2 verification flow **requires** `IC-Certificate` and `IC-CertificateExpression`. If they're missing—or the expression isn't in the certified tree—the gateway can't verify and returns **503**. Adding the **skip headers plus correct certified data** fixes this.

## ✅ **ICP Expert Response - CONFIRMATION & VALIDATION**

### **Expert Confirmation**

The ICP expert has confirmed the tech lead's solution and provided additional validation from official ICP documentation:

#### **1. Certification Strategy - CONFIRMED**

- ✅ **Skip certification is appropriate** for private, personalized, or dynamic content
- ✅ **No requirement** to use HTTP certification for private, user-specific content
- ✅ **Safer than raw.icp0.io** because the decision goes through consensus, not a single node

#### **2. Security Model - VALIDATED**

- ✅ **Token-based authentication + HTTPS are sufficient** for private assets
- ✅ **Strong authentication and access control** mitigate the risks of skipping certification
- ✅ **Appropriate for private, user-specific content** with proper access controls

#### **3. Implementation Approach - CONFIRMED**

- ✅ **Skip certification is the recommended approach** for private asset serving
- ✅ **Must add skip certification header** or HTTP gateway returns 503 error
- ✅ **Simpler and more performant** than full certification tree management

#### **4. ICP Best Practices - DOCUMENTED**

- ✅ **Official ICP documentation supports** this pattern
- ✅ **Examples exist** (e.g., serving metrics) where certification is skipped
- ✅ **Performance benefits** of skipping certification for private content

### **Expert Implementation Example**

```rust
use ic_cdk::{api::data_certificate, *};
use ic_http_certification::utils::{set_certified_data, skip_certification_certified_data, add_skip_certification_header};

#[init]
fn init() {
    set_certified_data(&skip_certification_certified_data());
}

#[query]
fn http_request(req: HttpRequest) -> HttpResponse<'static> {
    let mut response = handle_request(req);
    add_skip_certification_header(data_certificate().unwrap(), &mut response);
    response
}
```

### **Summary Table from Expert**

| Use Case               | Certification Required? | Skip Certification Supported? | Security Implications         | Implementation Note     |
| ---------------------- | ----------------------- | ----------------------------- | ----------------------------- | ----------------------- |
| Private assets (yours) | ❌ No                   | ✅ Yes                        | Trust subnet, use strong auth | Add skip cert header    |
| Public, static assets  | ✅ Yes                  | ❌ No                         | Cryptographic proof           | Use asset certification |
| Mixed (public/private) | Both                    | Yes (private), No (public)    | As above                      | Route accordingly       |

### **Expert Conclusion**

- For private, authenticated assets, use **skip certification** and add the skip certification header to your responses.
- There is **no requirement** to use HTTP certification for private, user-specific content.
- Your proposed implementation is correct and aligns with ICP best practices.
- This will resolve your 503 errors and is the recommended, secure, and performant approach for your use case.

## 📝 **Implementation Todo List**

### **Immediate Implementation Steps**

1. **Update `init()` function** - Add `set_certified_data(&skip_certification_certified_data())`
2. **Update `post_upgrade()` function** - Add the same certified data setup
3. **Modify `http_request()` function** - Add `add_skip_certification_header()` call
4. **Update HTTP response handlers** - Ensure all responses include skip certification headers
5. **Test the fix** - Verify 503 errors are resolved

### **Sanity Checklist**

- [ ] In `init` (and `post_upgrade`) set: `set_certified_data(&skip_certification_certified_data());`
- [ ] For **every** private route, call `add_skip_certification_header(data_certificate().unwrap(), &mut resp);` before returning.
- [ ] Include `Cache-Control: private, no-store` on private assets.
- [ ] Keep bodies **≤ 2 MB** per response for now (streaming later).
- [ ] If you later add certified/public routes, use the asset/tree pattern and `add_v2_certificate_header` with a witness.

---

**Status**: ✅ **SOLUTION CONFIRMED BY BOTH TECH LEAD & ICP EXPERT**  
**Priority**: 🔴 **CRITICAL** - Ready for implementation  
**Estimated Effort**: 1-2 hours implementation + testing  
**Dependencies**: None - clear implementation steps provided by both experts  
**Blocking**: HTTP module functionality, browser tests, Next.js integration

## 🔗 **Related Documentation**

- [HTTP Certification 503 Error Analysis](./http-certification-503-error-analysis.md)
- [Serving Assets Over HTTP on ICP](./README.md)
- [HTTP Module Architecture Analysis](./http-module-architecture-analysis.md)
- [Tech Lead 9-Point Feedback](./tech-lead-9-point-feedback.md)
