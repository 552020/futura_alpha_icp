# HTTP Module Testing Guide

## 🎯 **Testing Overview**

This directory contains tests for the HTTP module that enables direct asset serving from ICP canisters via the `http_request` method. The module provides token-gated access to private assets with HMAC authentication.

## 🏗️ **Module Architecture**

```
HTTP Module
├── Core Types (TokenPayload, AssetStore, etc.)
├── Adapters (ACL, AssetStore, SecretStore)
├── Routes (Health, Assets)
└── Main Handler (http_request)
```

## 🧪 **Test Categories**

### **1. Basic Functionality Tests** ✅

**File**: `test_http_basic.sh`

**What it tests**:

- Health check endpoint (`/health`)
- Token minting (`mint_http_token`)
- Asset serving rejections (no token)
- 404 responses for non-existent assets

**Expected Results**:

- Health check returns 200 OK
- Token minting succeeds with valid payload
- Asset requests without tokens return 401/403
- Non-existent assets return 404

### **2. PocketIC Integration Tests** 🧪

**Directory**: `tests/pocket-ic/`

**What it tests**:

- Complete HTTP flow: Create memory → Mint token → Serve asset
- Token validation: Proper authentication and authorization
- Asset serving: Correct content type, headers, and body
- Error handling: 401, 403, 404 responses for various error cases
- Security: Private cache control, no-store headers
- Negative cases: Missing token, invalid token, wrong variant, non-existent memory

**Test Cases**:

- ✅ **Happy path**: Valid token → 200 OK with correct headers
- ✅ **Missing token**: No token → 401/403
- ✅ **Invalid token**: Bad token → 401
- ✅ **Wrong variant**: Token for thumbnail used on original → 403
- ✅ **Non-existent memory**: Invalid memory ID → 404

### **3. Local HTTP Gateway Tests** 🌐

**Files**: `test_http_core_functionality.mjs`, `test_local_http_gateway.mjs`

**What it tests**:

- **Core functionality**: Health checks, token minting, basic HTTP operations
- **HTTP Gateway**: Real HTTP requests via local replica gateway
- **Asset serving**: Complete flow with memory creation and token-based access
- **Error handling**: 401, 403, 404 responses for various error cases
- **Security**: Private cache control, no-store headers
- **Integration**: Uses existing test utilities for better error handling

**Test Cases**:

- ✅ **Health check**: HTTP gateway and dfx canister call
- ✅ **Token minting**: Permission validation (expects forbidden for unauthorized)
- ✅ **Asset serving**: With and without tokens
- ✅ **Invalid endpoints**: 404 responses for non-existent paths
- ✅ **Response headers**: Content-Type, Cache-Control validation
- ✅ **Complete flow**: Create memory → Mint token → Serve asset via HTTP

### **4. Authentication & Authorization Tests** 🔐

**File**: `test_auth.sh` (to be created)

**What it tests**:

- Valid token acceptance
- Invalid token rejection
- Expired token handling
- Token scope validation
- Principal-based access control

**Test Cases**:

```bash
# Valid token
curl -H "Authorization: Bearer <valid_token>" \
  https://<canister>.icp0.io/assets/memory123/asset456

# Invalid token
curl -H "Authorization: Bearer invalid_token" \
  https://<canister>.icp0.io/assets/memory123/asset456

# Expired token
curl -H "Authorization: Bearer <expired_token>" \
  https://<canister>.icp0.io/assets/memory123/asset456

# Wrong scope
curl -H "Authorization: Bearer <wrong_scope_token>" \
  https://<canister>.icp0.io/assets/memory123/asset456
```

### **3. Asset Serving Tests** 📁

**File**: `test_assets.sh` (to be created)

**What it tests**:

- Inline asset serving (small files)
- Blob asset serving (large files)
- Asset existence checks
- Content-Type headers
- Content-Length headers

**Test Cases**:

```bash
# Inline asset
curl -H "Authorization: Bearer <token>" \
  https://<canister>.icp0.io/assets/memory123/inline_asset

# Blob asset
curl -H "Authorization: Bearer <token>" \
  https://<canister>.icp0.io/assets/memory123/blob_asset

# Non-existent asset
curl -H "Authorization: Bearer <token>" \
  https://<canister>.icp0.io/assets/memory123/nonexistent
```

### **4. Performance Tests** ⚡

**File**: `test_performance.sh` (to be created)

**What it tests**:

- Response times for different asset sizes
- Concurrent request handling
- Memory usage during asset serving
- Token validation performance

**Metrics to Track**:

- Response time (target: <100ms for small assets)
- Throughput (requests per second)
- Memory consumption
- Token validation latency

### **5. Error Handling Tests** 🚨

**File**: `test_errors.sh` (to be created)

**What it tests**:

- Malformed requests
- Invalid memory IDs
- Invalid asset IDs
- Network timeouts
- Canister errors

**Test Cases**:

```bash
# Malformed URL
curl https://<canister>.icp0.io/assets/invalid

# Invalid memory ID
curl -H "Authorization: Bearer <token>" \
  https://<canister>.icp0.io/assets/invalid_memory/asset123

# Invalid asset ID
curl -H "Authorization: Bearer <token>" \
  https://<canister>.icp0.io/assets/memory123/invalid_asset
```

### **6. Integration Tests** 🔗

**File**: `test_integration.sh` (to be created)

**What it tests**:

- End-to-end asset upload and serving
- ACL integration
- Blob store integration
- Secret store integration

**Test Flow**:

1. Upload asset to canister
2. Mint HTTP token for asset
3. Serve asset via HTTP module
4. Verify content integrity

## 🚀 **Running Tests**

### **Prerequisites**

1. **Deployed canister** with HTTP module
2. **Test assets** uploaded to canister
3. **Valid Internet Identity** for authentication
4. **Test environment** with curl or similar tools

### **Basic Test Execution**

```bash
# Run basic functionality tests
./test_http_basic.sh

# Run all tests
./run_http_tests.sh

# Run specific test category
./test_auth.sh
./test_assets.sh
./test_performance.sh
```

### **Test Environment Setup**

```bash
# Set canister URL
export CANISTER_URL="https://<canister-id>.icp0.io"

# Set test credentials
export TEST_PRINCIPAL="<test-principal>"
export TEST_MEMORY_ID="<test-memory-id>"
export TEST_ASSET_ID="<test-asset-id>"
```

## 📊 **Test Results Format**

### **Success Criteria**

- ✅ All basic functionality tests pass
- ✅ Authentication works correctly
- ✅ Assets serve with proper headers
- ✅ Error handling is robust
- ✅ Performance meets targets

### **Test Report Structure**

```markdown
# HTTP Module Test Results

## Test Summary

- Total Tests: X
- Passed: Y
- Failed: Z
- Duration: Xms

## Test Categories

- [ ] Basic Functionality
- [ ] Authentication & Authorization
- [ ] Asset Serving
- [ ] Performance
- [ ] Error Handling
- [ ] Integration

## Issues Found

- Issue 1: Description
- Issue 2: Description

## Recommendations

- Recommendation 1
- Recommendation 2
```

## 🔧 **Test Development Guidelines**

### **Test Data Management**

- Use dedicated test memories and assets
- Clean up test data after runs
- Use consistent naming conventions
- Document test data requirements

### **Error Testing**

- Test both expected and unexpected errors
- Verify error messages are helpful
- Check HTTP status codes are correct
- Ensure errors don't leak sensitive information

### **Performance Testing**

- Test with realistic asset sizes
- Measure both cold and warm performance
- Test concurrent access patterns
- Monitor resource usage

## 📝 **Test Documentation**

Each test file should include:

1. **Purpose**: What the test validates
2. **Prerequisites**: Required setup and data
3. **Test Cases**: Specific scenarios tested
4. **Expected Results**: Success criteria
5. **Troubleshooting**: Common issues and solutions

## ⚠️ **Current Status: HTTP Certification Issue**

**CRITICAL ISSUE**: All HTTP gateway requests are returning `503 - response verification error`. This indicates that the HTTP certification is not working properly.

**Working Components:**

- ✅ Memory creation with assets (using proper utilities)
- ✅ Token minting (properly rejecting with "forbidden" as expected)
- ✅ dfx canister calls work fine

**Blocked Components:**

- ❌ HTTP gateway requests (all return 503)
- ❌ Asset serving via HTTP
- ❌ Browser tests
- ❌ Next.js integration tests

**Next Steps:**

1. **Fix HTTP Certification**: Investigate HTTP certification setup and configuration
2. **Check Response Headers**: Verify certification tree and response headers
3. **Test Raw Domain**: Try bypassing certification if needed
4. **Browser Tests**: Direct image rendering in browser (after certification fix)
5. **Next.js Integration**: Image component integration tests (after certification fix)
6. **Edge Cases**: Inline vs blob, wrong variant, bad token, rotation
7. **Success Criteria**: Verify minimal success criteria (curl 200, headers, error codes, browser render)

## 🎯 **Success Metrics**

- **Functionality**: 100% of basic tests pass
- **Performance**: <100ms response time for small assets
- **Reliability**: 99.9% success rate under normal load
- **Security**: All authentication tests pass
- **Error Handling**: Graceful handling of all error conditions

---

**Last Updated**: 2025-01-12  
**Test Coverage**: Basic functionality, authentication, asset serving  
**Next Phase**: Fix HTTP certification, then performance and integration testing
