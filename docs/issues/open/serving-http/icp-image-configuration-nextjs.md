# ICP Image Configuration Issue - Next.js Image Component

## 🐛 **Problem Description**

The Next.js `Image` component was throwing errors when trying to display images stored on ICP (Internet Computer Protocol) due to unconfigured hostnames.

### **Error Message:**

```
Invalid src prop (icp://memory/c6f07efb-4e4f-73c0-c6f0-0000000073c0/blob/b01288cc-fc1c-07a3-95f9-7cb46ae2b89bd0b4) on `next/image`, hostname "memory" is not configured under images in your `next.config.js`
```

### **Root Cause:**

- Next.js Image component requires explicit configuration for external image sources
- ICP URLs use custom protocol (`icp://`) which Next.js doesn't recognize by default
- The `icp://memory/` and `icp://blob/` URLs need custom handling to serve actual image data

## 🎯 **Impact**

- **User Experience**: Images from ICP uploads were not displaying in the UI
- **Functionality**: Complete ICP upload flow was working, but visual feedback was broken
- **Development**: Console errors cluttering development environment

## ✅ **Current Implementation (To Be Replaced)**

**Note**: The current implementation uses a Backend Proxy approach that violates our privacy model. See the Architectural Options section below for the recommended solutions.

### **Current Architecture Issues:**

- ❌ Backend calls canisters using its own identity (not user's principal)
- ❌ Breaks user-data ownership rule
- ❌ Adds latency with double ICP calls (250-900ms)
- ❌ Not aligned with privacy architecture

### **Files to Remove:**

1. `src/nextjs/src/app/api/memories/[memoryId]/assets/[assetId]/route.ts` - **DELETE** (Backend proxy)
2. `src/nextjs/src/app/api/blobs/[blobId]/route.ts` - **DELETE** (Backend proxy)
3. `src/nextjs/src/lib/icp-image-loader.ts` - **DELETE** (Custom loader not needed for http_request)

### **Files to Update:**

1. `src/nextjs/next.config.ts` - **UPDATE** (Remove custom loader, add ICP hostnames to remotePatterns)
2. **Backend Canister** - **IMPLEMENT** (Add `http_request` method for direct asset serving)

## 🧪 **Testing Requirements for New Implementation**

### **Test Cases for http_request Implementation:**

#### **Backend Canister Implementation**

- [ ] `http_request` method implementation
- [ ] Asset serving from canister storage
- [ ] HTTP response headers (Content-Type, Cache-Control)
- [ ] Error handling (404, 403, 500)
- [ ] Authentication token validation (for private assets)

#### **Frontend Integration**

- [ ] Direct ICP URLs in `<Image>` components
- [ ] Next.js remotePatterns configuration
- [ ] Fallback behavior for non-ICP URLs
- [ ] Image optimization parameters handling

### **Verification Steps:**

- Upload an image to ICP
- Verify image displays via direct canister URL
- Check no backend proxy calls
- Confirm proper privacy model compliance
- Test image optimization parameters (width, quality)
- Verify caching headers and CDN behavior

## ⏱️ **Performance Analysis of Current vs Recommended Solutions**

### **Current Implementation Performance Issues:**

#### **Backend Proxy Call (Current - To Be Removed)**

- **Double ICP Calls**: 250-900ms total latency
- **Memory Read**: 50-200ms
- **Blob Read**: 100-800ms
- **Privacy Violation**: Backend uses its own identity

### **http_request Implementation Performance:**

#### **Direct Canister Asset Serving**

- **Direct HTTPS**: ~10-50ms (CDN cached)
- **Certified Assets**: Browser caching
- **No Backend**: Pure Web3 solution
- **Scalable**: Handles high concurrency
- **User Principal**: Maintains privacy model
- **Image Optimization**: Built-in Next.js optimization

### **Performance Comparison:**

| Solution                | First Load | Subsequent Loads | Privacy   | Scalability |
| ----------------------- | ---------- | ---------------- | --------- | ----------- |
| Current (Backend Proxy) | 250-900ms  | 5ms              | ❌ Broken | Limited     |
| http_request (Target)   | 10-50ms    | Instant          | ✅ Full   | Excellent   |

## 🎓 **Expert Validation & Recommendations**

### **ICP Expert Review**

Our solution has been validated by an ICP expert who confirmed:

> "Your analysis and solution for handling ICP image URLs in a Next.js application are well-aligned with best practices for integrating Internet Computer (ICP) assets into modern web frameworks... Your approach is correct, aligns with ICP best practices, and should provide a smooth user experience after the initial image load. Well done!"

### **Key Expert Insights:**

#### **1. Performance Optimization Priorities**

**Server-side caching** would provide the biggest immediate impact for our use case. The ICP documentation emphasizes caching to reduce repeated canister calls, which are the main source of latency for asset serving.

#### **2. Production Deployment Considerations**

- **Subnet load**: High-load subnets can increase latency. Monitor canister's subnet and consider moving to less loaded subnets if needed
- **Cycles**: Ensure canister is generously topped up with cycles to avoid service interruptions
- **Canister memory**: Each canister can now address up to 500 GB, but be mindful of storage and scaling limits

#### **3. Alternative Architecture Patterns**

Using an **asset canister** (such as `ic-certified-assets` or `ic-asset`) is recommended for serving static assets efficiently. These canisters are optimized for asset storage and delivery, support certification, and may provide better performance than our custom memory/blob approach.

#### **4. Scaling and Edge Cases**

- **Query calls** (for public, non-certified images) can handle thousands of requests per second per subnet
- **Update calls** are much slower and should be avoided for serving images
- For very high concurrency, consider sharding assets across multiple canisters or subnets

#### **5. Security and Access Control**

Serving public images without authentication is possible using query calls or certified assets, which do not require Internet Identity. This can simplify architecture and improve performance for non-sensitive content.

#### **6. Monitoring and Debugging**

- Use **canister logs** and **performance counters** for real-time monitoring
- The [Internet Computer Dashboard](https://dashboard.internetcomputer.org/) for subnet and canister-level metrics
- Consider enabling metrics endpoints (e.g., `/metrics`) for custom monitoring

#### **7. Future-Proofing**

Keep an eye on official release notes and NNS proposals for changes that might affect asset serving, such as deprecation of older HTTP interfaces or new asset canister features.

### **Recommended Next Steps:**

1. **Implement server-side caching** for immediate performance improvement
2. **Consider asset canister migration** for better long-term performance
3. **Set up monitoring** using ICP Dashboard and canister logs
4. **Evaluate public image serving** for non-sensitive content
5. **Monitor subnet performance** and consider canister placement optimization

## 🎯 **BREAKING: ICP Expert Latest Update - Direct Asset Serving**

### **Game-Changing Discovery**

The ICP expert just confirmed a **major architectural opportunity**:

> **"Yes, you can use the asset library (such as `ic-certified-assets` or `ic-asset-certification`) directly in a backend canister to serve images or other static assets."**

### **What This Means**

Instead of our current architecture:

```
Browser → Next.js API → ICP Backend (memories_read + blobs_read) → Response
```

We could potentially have:

```
Browser → ICP Backend (direct asset serving) → Response
```

### **Official ICP Pattern**

According to the expert and ICP documentation:

1. **Single Canister Architecture**: Build one canister that hosts both backend logic AND static assets
2. **Asset Certification Libraries**: Use `ic-certified-assets` or `ic-asset-certification`
3. **HTTP Request Handler**: Expose `http_request` method for direct asset serving
4. **No Proxy Needed**: Eliminate the Next.js API layer entirely

### **Architecture Comparison**

#### **Current Architecture (Our Implementation)**

```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐    ┌─────────────┐
│   Browser   │───▶│ Next.js API  │───▶│ ICP Backend │───▶│   Response  │
└─────────────┘    └──────────────┘    └─────────────┘    └─────────────┘
                          │
                          ▼
                   ┌──────────────┐
                   │ Custom Loader│
                   └──────────────┘
```

#### **Potential New Architecture (ICP Expert Suggestion)**

```
┌─────────────┐    ┌─────────────────────────────────────┐    ┌─────────────┐
│   Browser   │───▶│        ICP Backend Canister         │───▶│   Response  │
└─────────────┘    │  ┌─────────────┐  ┌─────────────┐  │    └─────────────┘
                   │  │ Backend     │  │ Asset       │  │
                   │  │ Logic       │  │ Serving     │  │
                   │  └─────────────┘  └─────────────┘  │
                   └─────────────────────────────────────┘
```

### **Benefits of Direct Asset Serving**

1. **Eliminate Double Round-Trip**: No more `memories_read` + `blobs_read`
2. **No Next.js API Layer**: Direct canister-to-browser communication
3. **Built-in Certification**: ICP asset libraries handle HTTP certification
4. **Better Performance**: Single canister call instead of multiple
5. **Simplified Architecture**: No custom loader or API routes needed

### **Implementation Requirements**

To implement this approach, we would need to:

1. **Integrate Asset Library**: Add `ic-certified-assets` to our backend canister
2. **HTTP Request Handler**: Implement `http_request` method for asset serving
3. **Asset Management**: Store and serve assets directly from the canister
4. **Update Frontend**: Remove custom loader, use direct ICP URLs

### **Migration Path**

#### **Option 1: Hybrid Approach (Recommended)**

- Keep current Next.js API for complex transformations
- Add direct asset serving for simple image requests
- Gradually migrate based on performance needs

#### **Option 2: Full Migration**

- Completely replace Next.js API with direct canister serving
- Implement all image transformations in Rust
- Update frontend to use direct ICP URLs

### **Questions for Implementation**

1. **Asset Storage**: How do we migrate existing blob storage to asset library format?
2. **Image Transformations**: Can we implement Sharp-like transformations in Rust?
3. **Caching**: How does ICP asset certification handle browser caching?
4. **Authentication**: How do we maintain Internet Identity auth for private assets?
5. **Performance**: What's the actual performance difference?

### **Next Steps**

1. **Research**: Study `ic-certified-assets` documentation and examples
2. **Prototype**: Create a simple proof-of-concept with direct asset serving
3. **Benchmark**: Compare performance between current and new approach
4. **Plan Migration**: Design migration strategy based on findings

## ⚖️ **Tech Lead Reality Check on Direct Asset Serving**

### **Tech Lead's Pragmatic Assessment**

Our tech lead provided a **realistic trade-off analysis** on the direct asset serving approach:

> _"Short answer: yes, you can embed `ic-certified-assets` in your backend canister and serve images via `http_request`. It's a supported pattern. Whether you should depends on your needs for auth, transforms, and ops."_

### **When Direct Asset Serving is a Win** ✅

- **Public assets** (wedding previews, thumbnails, marketing): certified, cacheable, CDN-friendly
- **Integrity guarantees**: clients can verify content (good for "forever" claims)
- **Simpler infra**: one canister hosts logic + assets

### **When It's Not Enough Alone** ❌

- **Private/protected assets**: certification ≠ authorization. Still need gated proxy for II auth
- **On-the-fly transforms**: asset libs serve stored bytes; they don't transform (no Sharp equivalent)
- **Complex range/stream semantics**: doable but you'll implement it yourself

### **Architecture Options (Pick One Per Asset Class)**

#### **Option 1: Public-Only Assets**

- Store originals + prebuilt sizes (256/1024/2048) and formats (JPEG/WEBP/AVIF)
- Drop custom loader, use direct HTTPS canister URLs
- **Best latency and caching; minimal backend hops**

#### **Option 2: Hybrid Approach (Recommended)**

- **Thumbnails/previews**: certified assets, direct HTTPS from canister
- **Originals/private**: keep Next.js proxy (II auth, ETag/Range/stream/transform)
- **UX is great (fast grids), privacy preserved for full-res**

#### **Option 3: Everything in Backend Canister**

- Integrate `ic-asset-certification` in same canister as business logic
- Implement auth in `http_request` for private paths
- Still need pre-generated variants or separate transform service

### **Concrete Changes for Hybrid Approach**

#### **Backend/ICP Changes:**

```rust
// Add asset library to canister
// Define routes:
// /pub/:assetId/:variant → public, certified
// /priv/:assetId → deny in asset canister; serve via Next proxy
```

#### **Next.js Changes:**

```typescript
// Remove custom loader for public thumbs
// Keep it only for private/originals
images: {
  remotePatterns: [
    { protocol: 'https', hostname: '*.ic0.app' },
    { protocol: 'https', hostname: '*.icp0.io' },
  ],
}
```

### **Operational Reality Check**

- **Pre-generation costs cycles once; saves user time forever**
- **Certified public assets play nicely with browser/CDN caching**
- **Keep current proxy route for cases certification can't solve**
- **If you later move to dedicated asset canister, URLs don't have to change**

### **Decision Rule of Thumb**

- **If an image can be public at rest** → asset library (direct)
- **If it must be access-controlled or transformed on demand** → keep proxy

### **Tech Lead's Bottom Line**

The tech lead's assessment is **much more measured** than the initial excitement:

1. **It's possible** but not a silver bullet
2. **Hybrid approach** is most pragmatic
3. **Keep current proxy** for private/transform use cases
4. **Use direct serving** only for public, pre-generated assets

### **Revised Recommendation**

Given our current use case (private memories with Internet Identity auth), the **hybrid approach** makes the most sense:

- **Thumbnails/previews**: Direct ICP serving (public)
- **Original images**: Keep Next.js proxy (private, authenticated)
- **Best of both worlds**: Fast grids + secure originals

## 🚀 **Tech Lead Recommendations & Improvements**

### **Current Solution Assessment**

Our tech lead confirmed: _"Looks solid. The custom loader + API router is the right pattern for `icp://…`"_ but provided targeted improvements for production readiness.

## 📋 **Architectural Options for ICP Image Delivery**

**Goal:** Make images stored on the Internet Computer (ICP) display correctly in the frontend (Next.js) while respecting our **privacy model** (only users can access their own data).

### **1. Backend Proxy Call (Current Implementation)**

**Flow**

```
<Image> → Next.js API → ICP canister → bytes → browser
```

**How it works**

- Next.js API route fetches image bytes from ICP and returns them as an HTTP response.
- `<Image>` works because it receives a normal HTTPS URL.

**Pros**

- Easy integration with `<Image>`
- Works out of the box with Vercel and Internet Identity sessions

**Cons**

- Backend calls canisters using its own identity (not the user's principal)
- Breaks user-data ownership rule
- Adds latency (extra network hop)

**Status:** Functional, but not aligned with our privacy architecture.

### **2. "Crazy" Temporary Endpoint (Ephemeral Bridge)**

**Flow**

```
Browser (user) → ICP (bytes)
Browser → POST /api/temp → store bytes
<Image src="/api/temp/..."> → backend → return bytes → browser
```

**How it works**

- User fetches bytes from ICP directly (authorized).
- Browser uploads bytes to a temporary Web2 endpoint.
- `<Image>` then loads from that temporary HTTPS URL.

**Pros**

- Preserves user principal
- `<Image>` compatibility
- Quick to implement for tests/prototypes

**Cons**

- Double data transfer (slow + expensive)
- Temporary backend storage (RAM/TTL cleanup needed)
- Still moves private bytes through Web2 (ephemeral but not ideal)

**Status:** Feasible workaround — not scalable or elegant.

### **3. Custom Wrapper with `<img>` (Direct Agent Fetch)**

**Flow**

```
Browser (user) → ICP canister (get_photo) → bytes → Blob → <img src="blob:...">
```

**How it works**

- Frontend calls canister directly using `@dfinity/agent` with the user's Internet Identity.
- Converts bytes to a Blob and displays using a `<img>` tag (not `<Image>`).

**Pros**

- Fastest path, full privacy (user principal)
- No backend, no extra hops

**Cons**

- `<Image>` cannot use `blob:` URLs (we must wrap `<Image>` in a custom component)
- No CDN caching or responsive optimization
- Requires memory management for many images

**Status:** Best privacy; needs a lightweight "CustomImage" wrapper for all ICP assets.

### **4. `http_request` Method on Canister (Direct HTTPS from ICP)**

**Flow**

```
<Image src="https://<canister>.icp0.io/..."> → boundary node → canister.http_request()
```

**How it works**

- Implement system-recognized `http_request` in the canister.
- The boundary node routes HTTPS requests to it.
- The canister returns image bytes inside an `HttpResponse`.
- Works natively with `<Image>` (standard HTTPS).

**Pros**

- Pure Web3: no backend, normal HTTPS URLs
- Works with `<Image>` out of the box
- Supports certified (public) or non-certified (private) responses

**Cons**

- Must implement and maintain HTTP logic in Rust
- Boundary node doesn't forward user principal (for private images need custom token/delegation)

**Status:** Ideal long-term solution; clean, scalable, zero Web2 dependency.

### **Summary Table**

| #   | Approach          | Works with `<Image>` | Uses User Principal | Needs Web2     | Performance | Long-Term Fit   |
| --- | ----------------- | -------------------- | ------------------- | -------------- | ----------- | --------------- |
| 1   | Backend Proxy     | ✅                   | ❌                  | ✅             | Medium      | 🚫 No           |
| 2   | Temp Endpoint     | ✅                   | ✅                  | ✅ (ephemeral) | Low         | ⚠️ Experimental |
| 3   | Wrapper + `<img>` | ❌ (wrapper)         | ✅                  | ❌             | ✅ Fast     | ✅ Good         |
| 4   | `http_request`    | ✅                   | ❌ (needs token)    | ❌             | ✅ Good     | ✅ Best         |

### **Decision Guideline**

- **Short term:** Keep #1 (works) or test #3 for user-owned flow.
- **Medium term:** Migrate to #4 (`http_request`) for clean architecture.
- **Never use #2** beyond quick proof-of-concepts.

### **Action Items**

- Update issue labels with these four options.
- Add this memo to `/docs/architecture/image-delivery-options.md`.
- Reference this memo in related tickets (`ICP Image Loader`, `Canister Asset Serving`, `Frontend Wrapper`).

### **Priority Improvements**

#### **1. Skip Double Canister Round-Trip** 🔥

**Current Issue**: We make two ICP calls (`memories_read` + `blobs_read`)
**Solution**: Persist fast lookup table

```typescript
// KV/SQL lookup: (memoryId, assetId) → blobLocator
{
  canister_id: string,
  blob_id: string,
  content_type: string,
  bytes: number,
  hash: string
}
```

**Impact**: Eliminate 50-200ms memory read call

#### **2. Strong Caching Semantics** 🔥

**Current Issue**: Basic caching headers only
**Solution**: Add ETag + 304 support

```typescript
const etag = `"${blob.hash}"`;
if (request.headers.get("If-None-Match") === etag) {
  return new NextResponse(null, { status: 304 });
}
headers.set("ETag", etag);
```

**Impact**: Eliminate unnecessary data transfer

#### **3. Streaming + Range Support** 🔥

**Current Issue**: `Buffer.from(...)` loads entire file into memory
**Solution**: Stream from canister to client

```typescript
const readable = new ReadableStream({
  start(controller) {
    for await (const chunk of icpBlobStream) controller.enqueue(chunk);
    controller.close();
  },
});
```

**Impact**: Handle large images/videos without memory pressure

#### **4. Real Image Transformations** 🔥

**Current Issue**: `?w=&q=` parameters ignored
**Solution**: Server-side resizing with Sharp

```typescript
import sharp from "sharp";
if (w) {
  const transformer = sharp()
    .rotate()
    .resize({ width: Number(w) })
    .jpeg({ quality: q ?? 75 });
  const stream = icpBlobStream.pipeThrough(nodeToWebTransform(transformer));
  headers.set("Content-Type", "image/jpeg");
  return new NextResponse(stream, { headers });
}
```

**Impact**: Actually reduce payload size

#### **5. Edge Runtime & CDN Cache** 🔥

**Current Issue**: Node.js runtime latency
**Solution**: Edge Runtime + Vercel CDN

```typescript
// Surrogate cache key
const cacheKey = `icp:mem:${memoryId}:asset:${assetId}:w${w}:q${q}:fmt${fmt}`;
```

**Impact**: Reduce latency to ~10-50ms

#### **6. Security & AuthZ Hardening** 🔥

**Current Issue**: Basic error handling
**Solution**: Strict validation + rate limiting

```typescript
// Validate IDs strictly, avoid timing attacks
// Return 404 for not found, 403 for no access
// Rate-limit image routes
// Check II session before canister calls
```

**Impact**: Production-ready security

#### **7. Accessibility & UX** 🔥

**Current Issue**: Basic image serving
**Solution**: Enhanced UX features

```typescript
// Proper sizes on <Image> for responsive srcset
// blurDataURL placeholders (20-40px LQIP)
// Normalize EXIF orientation server-side
```

**Impact**: Better user experience

#### **8. API Ergonomics** 🔥

**Current Issue**: Complex URLs
**Solution**: Simplified endpoints

```typescript
// Direct route: /api/assets/:assetId
// Batch manifest endpoint for multiple assets
// Signed, cacheable URLs
```

**Impact**: Better developer experience

### **Performance Expectations After Improvements**

| Scenario                  | Current             | After Improvements                |
| ------------------------- | ------------------- | --------------------------------- |
| **First Load**            | 250-900ms           | 50-200ms (one canister read)      |
| **Subsequent Loads**      | 5ms (browser cache) | Near-instant (CDN + 304)          |
| **Large Assets**          | Memory pressure     | Streaming + ranges                |
| **Image Transformations** | None                | Real resizing + format conversion |

### **Implementation Priority**

1. **🔥 High Priority**: Skip double round-trip, strong caching, streaming
2. **🔥 High Priority**: Real image transformations, edge runtime
3. **🔥 High Priority**: Security hardening, rate limiting
4. **📈 Medium Priority**: Accessibility features, API ergonomics

### **Code Quality Improvements**

#### **Error Handling**

```typescript
// Guard regex matches
if (!memoryMatch) {
  return new NextResponse("Invalid ICP URL format", { status: 400 });
}
```

#### **Structured Logging**

```typescript
console.log("Image request", {
  memoryId,
  assetId,
  w,
  q,
  fmt,
  userId: identity.getPrincipal().toString(),
});
```

#### **Testing Requirements**

- [ ] 200/206/304/403/404 response paths
- [ ] ETag behavior and 304 responses
- [ ] Range request handling
- [ ] Accept header permutations
- [ ] Image transformation accuracy

### **Migration Strategy**

1. **Phase 1**: Implement lookup table + ETag caching
2. **Phase 2**: Add streaming + range support
3. **Phase 3**: Implement image transformations
4. **Phase 4**: Move to Edge Runtime + CDN
5. **Phase 5**: Add security hardening + UX features

## 🚀 **Deployment Notes**

- **Development**: Restart dev server after configuration changes
- **Production**: No additional configuration needed
- **Caching**: Images are cached for 1 year with immutable headers
- **Authentication**: Requires Internet Identity authentication

## 🔗 **Related Issues**

- [ICP Upload Flow Documentation](../open/icp-upload-flow-documentation.md)
- [ICP Multiple Files Incomplete Flow](../open/icp-multiple-files-incomplete-flow.md)

## ✅ **Status**

**RESOLVED** - ICP images now display correctly in Next.js Image components.

---

**Created**: 2025-01-12  
**Resolved**: 2025-01-12  
**Priority**: High  
**Type**: Bug Fix
