# 🧪 Vercel Blob Upload Testing Proposal

**Date:** 2024-12-19  
**Status:** Proposal  
**Priority:** High  
**Labels:** `testing`, `vercel-blob`, `onboarding`, `upload`, `integration-test`  
**Assigned:** Tech Lead

## 📋 **Summary**

Proposal for comprehensive testing of the Vercel Blob upload flow in the onboarding process. This includes both frontend direct uploads and backend serverless function uploads, with considerations for the 5MB serverless function limit.

## 🎯 **Testing Objectives**

### **Primary Goals**

- Verify unauthenticated users can upload files to Vercel Blob during onboarding
- Ensure upload flow works without authentication requirements
- Test both small files (<5MB) and large files (>5MB) scenarios
- Validate integration between frontend and backend upload systems

### **Secondary Goals**

- Performance testing for different file sizes
- Error handling for failed uploads
- Progress tracking and user feedback
- Mobile device compatibility

## 🔧 **Technical Implementation Options**

### **Option 1: Frontend Direct Upload (Recommended for Large Files)**

**Pros:**

- No serverless function size limits (up to 5TB)
- Better performance for large files
- Direct client-to-Vercel Blob communication
- No backend processing overhead

**Cons:**

- Requires client-side token management
- More complex error handling
- Potential CORS issues

**Implementation:**

```typescript
// Frontend direct upload using @vercel/blob
import { put } from "@vercel/blob";

const uploadFile = async (file: File) => {
  const blob = await put(file.name, file, {
    access: "public",
    token: process.env.NEXT_PUBLIC_BLOB_READ_WRITE_TOKEN,
  });
  return blob.url;
};
```

### **Option 2: Backend Serverless Upload (Recommended for Small Files)**

**Pros:**

- Simpler implementation
- Better error handling
- Server-side processing capabilities
- No client-side token exposure

**Cons:**

- 5MB serverless function limit
- Additional server processing time
- Higher serverless function costs

**Implementation:**

```typescript
// Backend upload via /api/upload/vercel-blob
const formData = new FormData();
formData.append("file", file);

const response = await fetch("/api/upload/vercel-blob", {
  method: "POST",
  body: formData,
});
```

### **Option 3: Hybrid Approach (Recommended)**

**Implementation:**

- Files < 5MB: Backend serverless upload
- Files > 5MB: Frontend direct upload
- Automatic routing based on file size

## 📁 **Key Files Involved**

### **Frontend Components**

- `src/nextjs/src/hooks/use-file-upload.ts` - Main upload hook
- `src/nextjs/src/components/memory/item-upload-button.tsx` - Upload UI components
- `src/nextjs/src/app/[lang]/onboarding/items-upload/items-upload-client.tsx` - Onboarding upload page

### **Backend API Endpoints**

- `src/nextjs/src/app/api/upload/vercel-blob/route.ts` - Serverless upload endpoint
- `src/nextjs/src/services/upload/vercel-blob-upload.ts` - Upload service layer

### **Configuration Files**

- `src/nextjs/src/lib/storage/storage-manager.ts` - Storage provider configuration
- `src/nextjs/src/hooks/use-hosting-preferences.ts` - Hosting preferences

### **Test Infrastructure**

- `src/nextjs/tests-integration/` - Integration test folder
- `src/nextjs/vitest.config.ts` - Test configuration

## 🧪 **Testing Framework vs Script Approach**

### **Framework Testing (Vitest) - Recommended**

**Pros:**

- Integrated with existing test infrastructure
- Mocking capabilities for external services
- CI/CD pipeline integration
- Better error reporting and debugging
- Can test both frontend and backend components

**Cons:**

- Requires test environment setup
- May not test real browser interactions
- Mocking might miss real-world issues

**Implementation:**

```typescript
// tests-integration/api/upload/onboarding-upload.test.ts
import { describe, it, expect, vi } from "vitest";
import { NextRequest } from "next/server";
import { handleVercelBlobUpload } from "../../../src/app/api/upload/vercel-blob/route";

describe("Onboarding Upload Integration", () => {
  it("should upload file to Vercel Blob without auth", async () => {
    // Test implementation
  });
});
```

### **Script Testing (Browser-based) - Alternative**

**Pros:**

- Tests real browser environment
- Can test actual file uploads from browser
- Tests real user interactions
- Can access localStorage/sessionStorage
- Tests real CORS and security policies

**Cons:**

- Requires browser automation (Playwright/Selenium)
- More complex setup
- Slower execution
- Harder to integrate with CI/CD

**Implementation:**

```javascript
// tests-integration/scripts/onboarding-upload-test.js
// Browser automation script
const { chromium } = require("playwright");

async function testOnboardingUpload() {
  const browser = await chromium.launch();
  const page = await browser.newPage();

  // Navigate to onboarding page
  await page.goto("http://localhost:3000/en/onboarding/items-upload");

  // Upload file from public folder
  const fileInput = await page.locator('input[type="file"]');
  await fileInput.setInputFiles("./public/test-files/sample.jpg");

  // Wait for upload completion
  await page.waitForSelector('[data-testid="upload-success"]');

  await browser.close();
}
```

### **Hybrid Approach - Best of Both Worlds**

**Framework Tests for:**

- API endpoint testing
- Authentication bypass
- Error handling
- Unit testing of components

**Script Tests for:**

- End-to-end user flows
- Real browser file uploads
- Mobile device testing
- Cross-browser compatibility

## 📁 **File Handling for Frontend Testing**

### **Option 1: Public Test Files**

```javascript
// Store test files in public folder
public/
  test-files/
    small-image.jpg (2MB)
    large-image.jpg (10MB)
    test-video.mp4 (50MB)
    sample-document.pdf (1MB)
```

**Pros:**

- Easy to access from browser
- No authentication required
- Can test different file types
- Real file sizes for testing

**Cons:**

- Files committed to repository
- Repository size increase
- Security considerations

### **Option 2: Generated Test Files**

```javascript
// Generate test files programmatically
const generateTestFile = (size, type) => {
  const content = new Array(size).fill("A").join("");
  return new File([content], `test-${size}.${type}`, { type: "image/jpeg" });
};

// Small file (2MB)
const smallFile = generateTestFile(2 * 1024 * 1024, "jpg");

// Large file (10MB)
const largeFile = generateTestFile(10 * 1024 * 1024, "jpg");
```

**Pros:**

- No repository bloat
- Dynamic file generation
- Can test various sizes
- No security concerns

**Cons:**

- Generated files not "real"
- May not catch real file issues
- Memory usage for large files

### **Option 3: localStorage/sessionStorage Files**

```javascript
// Store file data in browser storage
const storeTestFile = (file) => {
  const reader = new FileReader();
  reader.onload = (e) => {
    localStorage.setItem("test-file", e.target.result);
  };
  reader.readAsDataURL(file);
};

// Retrieve and use stored file
const getStoredTestFile = () => {
  const dataUrl = localStorage.getItem("test-file");
  return dataUrl ? new File([dataUrl], "test.jpg") : null;
};
```

**Pros:**

- Persistent across sessions
- Can store multiple test files
- No network requests needed
- Real file data

**Cons:**

- Browser storage limits
- Complex implementation
- May not work in all browsers

## 🔄 **Vercel Blob Callback Considerations**

### **The Callback Problem**

Vercel Blob requires a callback URL for upload completion notifications, which creates challenges for local development and testing:

**Issue:**

- Vercel Blob needs a publicly accessible callback URL
- `localhost:3000` is not accessible from Vercel's servers
- Callbacks fail in local development environment

**Historical Solution (ngrok):**

```bash
# Install ngrok
npm install -g ngrok

# Start local development server
npm run dev

# In another terminal, expose localhost
ngrok http 3000

# Use ngrok URL for callbacks
# https://abc123.ngrok.io/api/upload/vercel-blob/callback
```

### **Testing Implications**

**For Script Testing:**

- ✅ **Works**: Scripts can use ngrok URLs for callbacks
- ✅ **Real Environment**: Tests actual callback flow
- ❌ **Complexity**: Requires ngrok setup and management
- ❌ **Reliability**: ngrok URLs can change between sessions

**For Framework Testing:**

- ✅ **Mocking**: Can mock callback responses
- ✅ **Reliability**: No external dependencies
- ❌ **Realism**: Doesn't test actual callback flow
- ❌ **Coverage**: May miss callback-related issues

### **Alternative Solutions**

**Option 1: Mock Callbacks (Recommended for Testing)**

```typescript
// Mock the callback in tests
vi.mock("@vercel/blob", () => ({
  put: vi.fn().mockResolvedValue({
    url: "https://mocked-vercel-blob-url.com/file.jpg",
    downloadUrl: "https://mocked-download-url.com/file.jpg",
  }),
}));
```

**Option 2: Use ngrok for Integration Tests**

```javascript
// tests-integration/scripts/onboarding-upload-with-ngrok.js
const { chromium } = require("playwright");
const { spawn } = require("child_process");

async function testWithNgrok() {
  // Start ngrok
  const ngrok = spawn("ngrok", ["http", "3000"]);

  // Wait for ngrok to start
  await new Promise((resolve) => setTimeout(resolve, 2000));

  // Get ngrok URL (you'd need to parse ngrok output)
  const ngrokUrl = "https://abc123.ngrok.io";

  // Run tests with ngrok URL
  const browser = await chromium.launch();
  const page = await browser.newPage();

  // Set callback URL environment variable
  process.env.VERCEL_BLOB_CALLBACK_URL = `${ngrokUrl}/api/upload/vercel-blob/callback`;

  // Run upload test
  await page.goto(`${ngrokUrl}/en/onboarding/items-upload`);
  // ... rest of test

  await browser.close();
  ngrok.kill();
}
```

**Option 3: Use Vercel Preview Deployments**

```bash
# Deploy to Vercel preview
vercel --prod=false

# Use preview URL for testing
# https://your-app-git-branch.vercel.app
```

### **Recommended Testing Strategy**

**For Development:**

- Use **mocked callbacks** in unit tests
- Use **ngrok** for manual testing
- Use **Vercel preview** for integration testing

**For CI/CD:**

- Use **mocked callbacks** for speed and reliability
- Use **Vercel preview** for end-to-end testing
- Avoid **ngrok** in CI (unreliable and slow)

## 🧪 **Proposed Test Scenarios**

### **Test 1: Small File Upload (< 5MB)**

```typescript
// tests-integration/api/upload/onboarding-small-file.test.ts
describe("Onboarding Small File Upload", () => {
  it("should upload small file via backend serverless", async () => {
    const file = new File(["test content"], "test.jpg", { type: "image/jpeg" });
    // Test backend upload
  });
});
```

### **Test 2: Large File Upload (> 5MB)**

```typescript
// tests-integration/api/upload/onboarding-large-file.test.ts
describe("Onboarding Large File Upload", () => {
  it("should upload large file via frontend direct", async () => {
    const file = new File(["large content"], "large.jpg", { type: "image/jpeg" });
    // Test frontend direct upload
  });
});
```

### **Test 3: Hybrid Upload Routing**

```typescript
// tests-integration/api/upload/onboarding-hybrid-routing.test.ts
describe("Onboarding Hybrid Upload Routing", () => {
  it("should route small files to backend", async () => {
    // Test automatic routing logic
  });

  it("should route large files to frontend", async () => {
    // Test automatic routing logic
  });
});
```

### **Test 4: Authentication Bypass**

```typescript
// tests-integration/api/upload/onboarding-auth-bypass.test.ts
describe("Onboarding Authentication Bypass", () => {
  it("should allow uploads without authentication", async () => {
    // Test unauthenticated upload flow
  });
});
```

## 🔍 **Testing Questions for Tech Lead**

### **1. Architecture Decisions**

- **Q:** Should we implement the hybrid approach (small files via backend, large files via frontend)?
- **Q:** Do we need to expose `BLOB_READ_WRITE_TOKEN` client-side for direct uploads?
- **Q:** Should we create a new endpoint for client-side token generation?

### **2. File Size Limits**

- **Q:** What's the maximum file size we want to support for onboarding?
- **Q:** Should we implement file size validation before upload?
- **Q:** Do we need progress tracking for large file uploads?

### **3. Error Handling**

- **Q:** How should we handle upload failures in the onboarding flow?
- **Q:** Should we implement retry logic for failed uploads?
- **Q:** What user feedback should we provide during uploads?

### **4. Security Considerations**

- **Q:** Is it safe to expose Vercel Blob tokens client-side?
- **Q:** Should we implement rate limiting for uploads?
- **Q:** Do we need file type validation for security?

### **5. Performance Requirements**

- **Q:** What's the expected upload time for different file sizes?
- **Q:** Should we implement upload progress indicators?
- **Q:** Do we need to optimize for mobile uploads?

### **6. Testing Approach**

- **Q:** Should we use Vitest framework tests or browser automation scripts?
- **Q:** For frontend testing, should we use public test files or generate them programmatically?
- **Q:** Do we need both unit tests and end-to-end tests?
- **Q:** Should we test on real devices or is browser automation sufficient?

### **7. Vercel Blob Callback Handling**

- **Q:** How should we handle Vercel Blob callbacks in local testing?
- **Q:** Should we use ngrok for callback testing or mock the callbacks?
- **Q:** Do we need to test the actual callback flow or is mocking sufficient?
- **Q:** Should we use Vercel preview deployments for integration testing?

## 📊 **Test Implementation Plan**

### **Phase 1: Backend Upload Testing**

1. Test existing `/api/upload/vercel-blob` endpoint
2. Verify authentication bypass for onboarding
3. Test with files < 5MB
4. Validate error handling

### **Phase 2: Frontend Direct Upload Testing**

1. Implement client-side token management
2. Test direct uploads to Vercel Blob
3. Test with files > 5MB
4. Validate CORS and security

### **Phase 3: Hybrid Implementation Testing**

1. Implement automatic routing logic
2. Test seamless switching between methods
3. Validate user experience consistency
4. Performance testing

### **Phase 4: Integration Testing**

1. End-to-end onboarding flow testing
2. Mobile device testing
3. Error scenario testing
4. Performance optimization

## 🎯 **Success Criteria**

### **Functional Requirements**

- [ ] Unauthenticated users can upload files during onboarding
- [ ] Small files (<5MB) upload via backend serverless
- [ ] Large files (>5MB) upload via frontend direct
- [ ] Automatic routing based on file size
- [ ] Proper error handling and user feedback

### **Performance Requirements**

- [ ] Small files upload in < 5 seconds
- [ ] Large files show progress indicators
- [ ] Mobile uploads work reliably
- [ ] No memory leaks during uploads

### **Security Requirements**

- [ ] No sensitive tokens exposed client-side
- [ ] File type validation implemented
- [ ] Rate limiting in place
- [ ] CORS properly configured

## 🔗 **Related Documentation**

- [Vercel Blob Archaeological Research](./vercel-blob-archaeological-research.md)
- [Onboarding Debug Issue](./onboarding-debug-issue.md)
- [Vercel Blob API Documentation](https://vercel.com/docs/storage/vercel-blob)
- [Next.js Serverless Functions Limits](https://vercel.com/docs/functions/serverless-functions)

## 📝 **Next Steps**

1. **Tech Lead Review**: Review this proposal and provide feedback
2. **Architecture Decision**: Choose between frontend, backend, or hybrid approach
3. **Implementation**: Implement chosen approach with tests
4. **Validation**: Test with real onboarding flow
5. **Documentation**: Document the working solution

## 🤔 **Questions for Tech Lead**

1. **Which upload approach do you prefer for onboarding?**
2. **What's the maximum file size we should support?**
3. **Should we implement the hybrid approach or stick to one method?**
4. **Are there any security concerns with client-side uploads?**
5. **What's the timeline for implementing this testing strategy?**

---

**Note**: This proposal is based on the existing Vercel Blob infrastructure found in the codebase. All necessary components are already implemented and ready for testing.

## 📚 **Vercel Blob Testing Cheatsheet Reference**

### **Quick Testing Approaches (No UI Required)**

**1. Node/TS Scripts (Real Uploads)**

```typescript
// Token mode (dev only) - fastest
import { put } from "@vercel/blob";
const res = await put("filename.jpg", fileBuffer, {
  access: "public",
  token: process.env.BLOB_READ_WRITE_TOKEN!,
  addRandomSuffix: true,
});
```

**2. Pre-signed URL Mode (Prod-safe)**

```typescript
// Get upload URL from backend
const { uploadUrl } = await fetch("/api/blob/upload-url", { method: "POST" }).then((r) => r.json());
// Upload directly to Vercel Blob
const res = await fetch(uploadUrl, { method: "PUT", body: fileBuffer });
```

**3. Bash/curl Testing**

```bash
# Token mode
curl -X PUT \
  -H "Authorization: Bearer $BLOB_READ_WRITE_TOKEN" \
  --data-binary @./file.jpg \
  "https://api.vercel.com/v2/blobs?filename=file.jpg&access=public"

# Pre-signed URL mode
UPLOAD_URL=$(curl -s -X POST $BASE_URL/api/blob/upload-url | jq -r .uploadUrl)
curl -X PUT --data-binary @./file.jpg "$UPLOAD_URL"
```

**4. Vitest Integration Tests**

```typescript
// Backend route testing
it("uploads small file via serverless", async () => {
  const res = await request(app).post("/api/upload/vercel-blob").attach("file", Buffer.from("hello"), "hello.txt");
  expect(res.status).toBe(200);
  expect(res.body.url).toBeTruthy();
});

// Direct upload testing
it("uploads via direct URL", async () => {
  const { uploadUrl } = await fetch("/api/blob/upload-url", { method: "POST" }).then((r) => r.json());
  const file = Buffer.alloc(256 * 1024); // 256KB
  const put = await fetch(uploadUrl, { method: "PUT", body: file });
  expect(put.ok).toBe(true);
});
```

**5. Playwright Browser Testing (No UI Clicks)**

```typescript
test("direct client upload via one-time URL", async ({ page, request }) => {
  const { uploadUrl } = await (await request.post("/api/blob/upload-url")).json();

  const meta = await page.evaluate(async (uploadUrl: string) => {
    const file = new File([new Uint8Array(256 * 1024)], "sample.jpg", { type: "image/jpeg" });
    const res = await fetch(uploadUrl, { method: "PUT", body: file });
    return res.json();
  }, uploadUrl);

  const head = await request.fetch(meta.url, { method: "HEAD" });
  expect(head.ok()).toBeTruthy();
});
```

### **Large File Generation for Tests**

```typescript
// Node buffer
const big = Buffer.alloc(10 * 1024 * 1024, 0xab); // 10MB

// Browser
const bigBlob = new Blob([new Uint8Array(10 * 1024 * 1024)], { type: "application/octet-stream" });
```

### **CI Strategy**

- **Unit/Integration (Vitest)**: Mock most, keep **one** small live canary upload
- **Playwright**: **One** E2E for direct upload (workers=1)
- **No ngrok in CI**: Use **Vercel Preview** for real webhook testing
- **Cleanup**: Delete blobs created by CI runs via `/api/blob/delete`

### **Safety Notes**

- Never expose `BLOB_READ_WRITE_TOKEN` to client
- Prefer **pre-signed upload URLs** for frontend uploads
- Rate-limit `/api/blob/upload-url`
- Validate file type/size on server and client

### **Package.json Scripts**

```json
{
  "scripts": {
    "blob:put": "ts-node scripts/blob-upload-token.ts",
    "blob:put:url": "ts-node scripts/blob-upload-presigned.ts",
    "blob:head": "ts-node scripts/blob-head.ts",
    "blob:del": "ts-node scripts/blob-delete.ts"
  }
}
```

---

**Reference**: This cheatsheet provides end-to-end ways to verify Vercel Blob uploads **without UI**: scripts, curl, Vitest, and Playwright in browser context.
