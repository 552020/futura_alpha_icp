# 🚨 File Upload Errors - Multiple Issues Blocking Folder Uploads

## **Summary**

File upload functionality is failing with multiple errors, primarily due to server-side size limits and misaligned validation between client and server. Users cannot upload folders or large files.

## **Error Details**

### 1. **HTTP 413 - Payload Too Large** (Critical)

```
Failed to load resource: the server responded with a status of 413 ()
Resource: /api/memories/upload/folder:1
```

- **Impact**: Blocks all folder uploads
- **Frequency**: Recurring for any upload attempt
- **Root Cause**: Next.js default body size limit (~1MB) not configured

### 2. **Invalid File Type Error** (High)

```
Folder upload error: Error: Invalid file type
Location: handleFileChange in useFileUpload hook
```

- **Impact**: Prevents file processing before upload
- **Root Cause**: Client-side validation rejecting file types

### 3. **JSON Parsing Error** (High)

```
Folder upload error: SyntaxError: Unexpected token 'R', "Request En"... is not valid JSON
```

- **Impact**: Breaks error handling and user feedback
- **Root Cause**: Server returns HTML error page (413) but client expects JSON

### 4. **HTTP 400 - Bad Request** (Medium)

```
Failed to load resource: the server responded with a status of 400 ()
Resource: /api/memories/upload/folder:1
```

- **Impact**: Additional upload failures
- **Root Cause**: Malformed requests due to size limit issues

## **Technical Analysis**

### **Size Limit Misalignment**

| Component                            | Current Limit               | Expected          |
| ------------------------------------ | --------------------------- | ----------------- |
| Client-side (`useFileUpload.ts:112`) | 50MB                        | ✅                |
| Server-side (`utils.ts:10-11`)       | 10MB (files), 25MB (videos) | ❌ Too low        |
| Next.js body parser                  | ~1MB (default)              | ❌ Not configured |

### **File Type Validation Issues**

- **Accepted MIME types** are restrictive (see `ACCEPTED_MIME_TYPES` in `utils.ts`)
- **Client validation** happens before server validation
- **Error messages** are not user-friendly

### **Error Handling Problems**

- **Server errors** return HTML instead of JSON
- **Client code** assumes JSON responses
- **Error propagation** breaks user experience

## **Files Affected**

### **Core Upload Logic**

- `src/nextjs/src/hooks/user-file-upload.ts` - Client-side upload hook
- `src/nextjs/src/components/memory/item-upload-button.tsx` - Upload UI component
- `src/nextjs/src/app/api/memories/upload/folder/route.ts` - Folder upload API
- `src/nextjs/src/app/api/memories/upload/utils.ts` - Validation utilities

### **Configuration**

- `src/nextjs/next.config.ts` - Missing body size configuration

## **Proposed Solutions**

### **1. Fix Next.js Body Size Limit** (Critical)

```typescript
// next.config.ts
const nextConfig: NextConfig = {
  // ... existing config
  experimental: {
    serverComponentsExternalPackages: ["@vercel/blob"],
  },
  // Add body size configuration
  api: {
    bodyParser: {
      sizeLimit: "50mb",
    },
  },
};
```

### **2. Align Server-Side Size Limits** (High)

```typescript
// utils.ts
export const MAX_FILE_SIZE = 50 * 1024 * 1024; // 50MB (align with client)
export const MAX_VIDEO_SIZE = 50 * 1024 * 1024; // 50MB (align with client)
```

### **3. Improve Error Handling** (High)

- Return JSON error responses instead of HTML
- Add proper error parsing in client code
- Implement retry logic for transient failures

### **4. Enhance File Type Validation** (Medium)

- Expand accepted MIME types
- Add better error messages
- Implement progressive validation (client → server)

### **5. Add Upload Progress & Feedback** (Low)

- Show upload progress for large files
- Better user feedback during uploads
- Implement chunked uploads for very large files

## **Testing Scenarios**

### **Test Cases to Verify Fixes**

1. **Small folder upload** (< 1MB total) - Should work
2. **Medium folder upload** (1-10MB total) - Should work after fixes
3. **Large folder upload** (10-50MB total) - Should work after fixes
4. **Invalid file types** - Should show clear error messages
5. **Network failures** - Should handle gracefully

### **Files to Test With**

- Images: JPG, PNG, GIF, WebP
- Videos: MP4, WebM, MOV
- Documents: PDF, DOC, TXT, MD
- Mixed folders with various file types

## **Priority**

- **P0**: Fix 413 errors (blocking all uploads)
- **P1**: Align size limits and improve error handling
- **P2**: Enhance validation and user experience

## **Acceptance Criteria**

- [ ] Folder uploads work for files up to 50MB total
- [ ] Clear error messages for invalid file types
- [ ] Proper JSON error responses from server
- [ ] Upload progress feedback for large files
- [ ] Graceful handling of network failures

## **Related Issues**

- May be related to ICP upload service integration
- Could affect onboarding flow if users can't upload files
- May impact gallery creation and memory management features

---

**Environment**: Development
**Browser**: Chrome/Firefox (based on console logs)
**Reproduction**: 100% - happens on every folder upload attempt
