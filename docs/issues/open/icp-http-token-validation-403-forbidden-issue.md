# ICP HTTP Token Validation 403 Forbidden Issue

## 🚨 **Problem Description**

After successfully resolving the 400 Bad Request issue (canister ID resolution), we now have a **403 Forbidden** error when the Next.js Image component tries to load images from the ICP HTTP gateway. The backend is rejecting the tokens even though they appear to be correctly formatted.

## 🔍 **Evidence from Logs**

### **✅ URL Generation Working (Fixed)**

```
🔍 [Transform] Generated display URL for memory: f75e5ad8-1fcf-2ee7-f75e-000000002ee7
URL: http://uxrrr-q7777-77774-qaaaq-cai.localhost:4943/asset/f75e5ad8-1fcf-2ee7-...token...

🔍 [ContentCard] Image src for memory: e7ccac87-acc5-7745-e7cc-000000007745
URL: http://uxrrr-q7777-77774-qaaaq-cai.localhost:4943/asset/e7ccac87-acc5-7745-...token...
```

### **❌ Token Validation Failing**

```
GET http://localhost:3000/_next/image?url=http%3A%2F%2Fuxrrr-q7777-77774-qaaaq-...&w=1920&q=75 403 (Forbidden)
🔍 [ContentCard] Image error for memory: e7ccac87-acc5-7745-e7cc-000000007745
URL: http://uxrrr-q7777-77774-qaaaq-cai.localhost:4943/asset/e7ccac87-acc5-7745-...token...
```

## 🎯 **Root Cause Analysis**

The issue has progressed from:

1. ✅ **400 Bad Request** → **RESOLVED** (canister ID resolution fixed)
2. ✅ **Canister ID resolved** → Working (requests reach backend)
3. ✅ **Route found** → Working (no more 404 errors)
4. ❌ **403 Forbidden** → **NEW ISSUE** (token validation failing)

## 🔍 **Root Cause Analysis - CRITICAL DISCOVERY**

### **✅ Backend HTTP Handler Works Perfectly**

Our test script `test_working_http_flow.mjs` shows:

```
HTTP/1.1 200 OK
content-type: image/png
✅ ✅ HTTP access successful! Complete flow works!
```

This proves:

- ✅ **Backend HTTP handler is working**
- ✅ **Token validation logic is working**
- ✅ **Asset serving is working**
- ✅ **Fresh tokens work perfectly**

### **🎯 ROOT CAUSE FOUND: Token Expiration!**

**Manual curl test reveals the issue:**

```bash
curl "http://uxrrr-q7777-77774-qaaaq-cai.localhost:4943/asset/.../thumbnail?token=..."
# Returns: HTTP/1.1 403 Forbidden
# Response body: "Token expired%"
```

**The Problem:**

- ✅ **Tokens are generated correctly** (30-minute TTL)
- ✅ **URLs are formatted correctly**
- ❌ **Tokens are expiring** before Next.js Image component can use them
- ❌ **403 Forbidden** is returned because tokens are expired

### **🔍 Token TTL Analysis**

**Current TTL Setting:**

```rust
const MEMORY_LISTING_TTL: u32 = 1800; // 30 minutes
```

**The Issue:**
Even with 30-minute TTL, tokens are expiring. This suggests either:

1. **Time synchronization issue** between token generation and validation
2. **Token generation timing** - tokens might be generated much earlier than expected
3. **Clock drift** between frontend and backend

### **❌ The Real Problem: Token Expiration**

The issue is with **token expiration**, not the backend. Possible causes:

### **1. Token Expiration (Most Likely)**

- Frontend tokens are expiring before Next.js Image component can use them
- TTL might be too short for the image loading process
- Tokens generated during memory listing might expire before images are displayed

### **2. Token Scope Issues**

- Tokens might not have the correct scope for the requested assets
- Memory ID or asset ID mismatch between token generation and usage

### **3. Token Format Issues**

- Tokens might be getting corrupted during URL encoding/decoding
- Special characters in tokens might not be handled correctly by Next.js

### **4. Next.js Image Optimization Interference**

- Next.js might be modifying the request in a way that breaks token validation
- Headers might be getting stripped or modified

## 🧪 **Investigation Steps**

### **Step 1: ✅ Backend Confirmed Working**

- ✅ **Backend HTTP handler works**: Test script returns HTTP 200 OK
- ✅ **Token validation works**: Fresh tokens are accepted
- ✅ **Asset serving works**: Images are returned correctly

### **Step 2: Compare Frontend vs Test Tokens**

Compare the tokens being used by the frontend vs. the working test tokens:

- **Frontend tokens** (from logs): `eyJwIjp7InZlciI6MSwia2lkIjoxLCJleHBfbnMiOjE3NjAzOTAzMzg5ODgyNzYwMDAs...`
- **Test tokens** (working): `eyJwIjp7InZlciI6MSwia2lkIjoxLCJleHBfbnMiOjE3NjAzOTAzMjY4NjQ4MzAwMDAs...`

### **Step 3: Check Token Expiration Times**

Compare the `exp_ns` field in the tokens:

- **Frontend token**: `1760390389827600000` (expires at)
- **Test token**: `1760390326483000000` (expires at)

### **Step 4: Test Frontend Tokens Manually**

Test the exact same tokens that are failing in the frontend with curl:

```bash
curl -v "http://uxrrr-q7777-77774-qaaaq-cai.localhost:4943/asset/{memory_id}/thumbnail?token={frontend_token}"
```

### **Step 5: Test Without Next.js Optimization**

Temporarily disable Next.js image optimization:

```typescript
<Image
  src={imageSrc}
  unoptimized={true} // Add this
  // ... other props
/>
```

## 🔧 **Potential Solutions**

### **Solution 1: Increase Token TTL**

If tokens are expiring too quickly:

```rust
// In TokenService::mint_token
let ttl_seconds = if is_memory_listing_context() { 1800 } else { 180 }; // 30 min vs 3 min
```

### **Solution 2: Fix Token Encoding**

If tokens are getting corrupted:

- Check URL encoding/decoding
- Ensure special characters are handled correctly

### **Solution 3: Bypass Next.js Optimization**

If Next.js is interfering:

```typescript
<Image
  src={imageSrc}
  unoptimized={true}
  // ... other props
/>
```

### **Solution 4: Add Request Logging**

Add detailed logging to the backend HTTP handler to see exactly what's being rejected.

### **Solution 5: Use Regular img Tag**

Test with a regular `<img>` tag instead of Next.js `<Image>` component to isolate the issue.

## 📋 **Next Steps**

1. ⏳ **Check Backend Logs**: Look at canister logs during image loading
2. ⏳ **Test Token Manually**: Use curl to test the exact failing tokens
3. ⏳ **Compare Token Generation**: Check if tokens are being generated correctly
4. ⏳ **Test Without Optimization**: Try disabling Next.js image optimization
5. ⏳ **Check Token TTL**: Verify token expiration times

## 🎯 **Expected Outcome**

The Next.js Image component should successfully load images from the ICP HTTP gateway without 403 Forbidden errors, with proper token validation working.

---

**Created**: 2025-01-13  
**Status**: 🔍 **INVESTIGATING** - 403 Forbidden errors from token validation  
**Priority**: 🟡 **MEDIUM** - 400 Bad Request resolved, token validation issue remains  
**Affected Components**: ContentCard, Dashboard Detail Page, Next.js Image optimization  
**Related Issues**: [nextjs-image-400-bad-request-issue.md](./nextjs-image-400-bad-request-issue.md) (RESOLVED)
