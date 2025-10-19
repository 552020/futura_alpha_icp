# 🔍 Vercel Blob Integration Analysis

**Date:** 2024-12-20  
**Status:** Analysis Complete  
**Priority:** High  
**Labels:** `vercel-blob`, `integration`, `onboarding`, `architecture`, `existing-code`  
**Assigned:** Development Team

## 📋 **Summary**

This issue analyzes the current Vercel Blob implementation in the codebase and provides a strategic plan for integrating it with the new onboarding flow. The analysis reveals that Vercel Blob is **partially implemented** but **not actively used** in the main application flow.

## 🎯 **Current State Analysis**

### **✅ What We Have (Vercel Blob Infrastructure)**

#### **1. Backend Endpoints**

```
/api/upload/vercel-blob/
├── route.ts                    # ✅ Upload endpoint (no auth)
└── grant/route.ts             # ✅ Upload + Memory creation (auth required)
```

#### **2. Service Layer**

```
src/services/upload/vercel-blob-upload.ts
├── uploadFileToVercelBlob()           # ✅ Basic upload
├── uploadToVercelBlob()               # ✅ Multiple files
├── uploadToVercelBlobWithProcessing() # ✅ With image processing
└── uploadOriginalToVercelBlob()       # ✅ Lane A processing
```

#### **3. Storage Providers**

```
src/lib/storage/providers/
├── vercel-blob.ts             # ✅ VercelBlobProvider class
└── vercel-blob-grant.ts       # ✅ VercelBlobGrantProvider class
```

#### **4. Utility Functions**

```
src/lib/blob.ts
└── uploadFromPath()           # ✅ Buffer upload utility
```

### **❌ What We Don't Have (Active Usage)**

#### **1. Frontend Integration**

- **No active usage** in main upload flows
- **No integration** with `useFileUpload` hook
- **No UI components** using Vercel Blob

#### **2. Production Usage**

- **S3 is primary** storage backend
- **Vercel Blob is unused** in main application
- **No user-facing features** using Vercel Blob

#### **3. Onboarding Integration**

- **No onboarding-specific** Vercel Blob flow
- **No unauthenticated** upload support
- **No staging/cleanup** system

## 🎯 **Integration Strategy**

### **Phase 1: Leverage Existing Infrastructure**

#### **✅ Use Existing Endpoints**

```typescript
// Existing endpoints we can use:
/api/upload/vercel-blob/route.ts        # For uploads (no auth)
/api/upload/complete/route.ts            # For database operations (auth required)
```

#### **✅ Use Existing Services**

```typescript
// Existing services we can leverage:
import { uploadToVercelBlobWithProcessing } from "@/services/upload/vercel-blob-upload";
import { VercelBlobProvider } from "@/lib/storage/providers/vercel-blob";
```

### **Phase 2: Add Onboarding Support**

#### **🆕 New Onboarding Endpoints**

```typescript
// New endpoints for onboarding:
/api/onboarding/upload-url/route.ts      # Get upload URL (no auth)
/api/onboarding/commit/route.ts          # Create memory (no auth)
/api/onboarding/cleanup/route.ts         # Cleanup job
```

#### **🆕 Environment Variables**

```bash
# New environment variables:
OPEN_ONBOARDING_UPLOADS=true
OPEN_ONBOARDING_COMMIT=true
OPEN_ONBOARDING_PREFIX=onboarding/
OPEN_ONBOARDING_MAXSIZE=200MB
OPEN_ONBOARDING_TTL_HOURS=48
```

### **Phase 3: Integrate with Frontend**

#### **🆕 Update useFileUpload Hook**

```typescript
// src/hooks/use-file-upload.ts
const handleFileUploadOnboarding = async (files: File[]) => {
  // Use existing Vercel Blob services
  const result = await uploadToVercelBlobWithProcessing(
    files[0],
    true, // isOnboarding
    undefined, // existingUserId
    "multiple-files"
  );
  return result;
};
```

## 🚀 **Implementation Plan**

### **Step 1: Create Onboarding Endpoints (New)**

```typescript
// /api/onboarding/upload-url/route.ts
export async function POST() {
  if (process.env.OPEN_ONBOARDING_UPLOADS !== "true") return NextResponse.json({ error: "closed" }, { status: 403 });

  const { url, id } = await blobs.createUploadURL({
    access: "public",
    prefix: "onboarding/",
    maxSize: parseSize(process.env.OPEN_ONBOARDING_MAXSIZE ?? "200MB"),
  });

  return NextResponse.json({ uploadUrl: url, blobId: id });
}
```

### **Step 2: Create Commit Endpoint (New)**

```typescript
// /api/onboarding/commit/route.ts
export async function POST(req: Request) {
  if (process.env.OPEN_ONBOARDING_COMMIT !== "true") return NextResponse.json({ error: "closed" }, { status: 403 });

  const { blobUrl, metadata } = await req.json();

  // Use existing memory creation logic
  const memoryId = await createMemoryAndAsset(blobUrl, metadata, { isOnboarding: true });

  return NextResponse.json({ ok: true, memoryId });
}
```

### **Step 3: Create Cleanup Endpoint (New)**

```typescript
// /api/onboarding/cleanup/route.ts
export async function POST() {
  const prefix = process.env.OPEN_ONBOARDING_PREFIX ?? "onboarding/";
  const ttlHours = Number(process.env.OPEN_ONBOARDING_TTL_HOURS ?? 48);

  // Use existing Vercel Blob utilities
  const { blobs } = await list();
  const oldBlobs = blobs.filter(
    (blob) => blob.pathname.startsWith(prefix) && new Date(blob.uploadedAt).getTime() < cutoffTime
  );

  for (const blob of oldBlobs) {
    await del(blob.url);
  }

  return NextResponse.json({ ok: true, deleted: oldBlobs.length });
}
```

### **Step 4: Update Frontend Integration**

```typescript
// src/hooks/use-file-upload.ts
const handleFileUploadOnboarding = async (files: File[]) => {
  try {
    // 1. Get upload URL
    const { uploadUrl } = await fetch("/api/onboarding/upload-url", {
      method: "POST",
    }).then((r) => r.json());

    // 2. Upload file
    const uploadResponse = await fetch(uploadUrl, {
      method: "PUT",
      body: files[0],
    });
    const { url: blobUrl } = await uploadResponse.json();

    // 3. Commit to database
    const commitResponse = await fetch("/api/onboarding/commit", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        blobUrl,
        metadata: { title: files[0].name },
      }),
    });

    const { memoryId } = await commitResponse.json();
    return { success: true, memoryId };
  } catch (error) {
    return { success: false, error: error.message };
  }
};
```

## 🎯 **Benefits of This Approach**

### **✅ Leverage Existing Code**

- **Reuse Vercel Blob services** - No need to rewrite
- **Reuse memory creation logic** - Existing database operations
- **Reuse image processing** - Existing asset creation

### **✅ Minimal New Code**

- **Only 3 new endpoints** - Upload URL, commit, cleanup
- **No existing code changes** - Everything working stays working
- **Easy to test** - Isolated new functionality

### **✅ Strategic Architecture**

- **Onboarding-specific endpoints** - Clean separation
- **Environment controlled** - Easy to enable/disable
- **Future-proof** - Can extend for normal users later

## 🤔 **Questions for Implementation**

### **1. Endpoint Strategy**

- Should we create new `/api/onboarding/*` endpoints?
- Or modify existing `/api/upload/vercel-blob/*` endpoints?
- How do we handle authentication differences?

### **2. Service Integration**

- Should we use existing `uploadToVercelBlobWithProcessing()`?
- Or create new onboarding-specific functions?
- How do we handle the `isOnboarding` parameter?

### **3. Frontend Integration**

- Should we modify existing `useFileUpload` hook?
- Or create new `useOnboardingUpload` hook?
- How do we handle the different upload flows?

### **4. Database Integration**

- Should we use existing memory creation functions?
- Or create new onboarding-specific functions?
- How do we handle user authentication?

## 🚀 **Recommended Implementation**

### **Phase 1: Create New Endpoints (Safe)**

```typescript
// Create new onboarding-specific endpoints
/api/abdginnoor / upload -
  url / route.ts / api / onboarding / commit / route.ts / api / onboarding / cleanup / route.ts;
```

### **Phase 2: Integrate with Frontend (Strategic)**

```typescript
// Update useFileUpload hook to support onboarding
const handleFileUploadOnboarding = async (files: File[]) => {
  // Use new onboarding endpoints
  // Leverage existing Vercel Blob services
  // Create memory using existing database functions
};
```

### **Phase 3: Test and Deploy (Gradual)**

```typescript
// Test onboarding flow
// Deploy with environment variables
// Monitor usage and performance
// Add security measures as needed
```

## 📊 **Success Criteria**

### **✅ Functional Requirements**

- [ ] Onboarding users can upload files
- [ ] Files are stored in Vercel Blob
- [ ] Memory records are created in database
- [ ] Image processing works (display, thumb, placeholder)
- [ ] Cleanup job removes expired files

### **✅ Technical Requirements**

- [ ] No existing code broken
- [ ] New endpoints isolated
- [ ] Environment controlled
- [ ] Easy to test and debug

### **✅ Security Requirements**

- [ ] Rate limiting implemented
- [ ] Input validation working
- [ ] No database spam possible
- [ ] Monitoring in place

## 🔗 **Related Issues**

- [Vercel Blob Security Architecture Decision](./vercel-blob-security-architecture-decision.md)
- [S3 Upload Flow Analysis](./s3-upload-flow-analysis.md)
- [Vercel Blob Pure Functions Test Results](./vercel-blob-pure-functions-test-results.md)

---

**Next Steps**: Implement the new onboarding endpoints using existing Vercel Blob infrastructure, then integrate with the frontend upload flow.
