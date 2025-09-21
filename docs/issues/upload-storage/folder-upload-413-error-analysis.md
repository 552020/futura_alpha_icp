# 🚨 Folder Upload 413 Error - Root Cause Analysis

**Labels**: `type:bug`, `area:uploads`, `priority:critical`, `platform:vercel`

## **Problem Summary**

Folder uploads are failing with HTTP 413 "Content Too Large" errors, preventing users from uploading multiple files simultaneously. The error occurs at the Vercel platform level before reaching application code.

## **Error Details**

### **Primary Error**

```
POST https://www.futura.now/api/memories
Status: 413 (Content Too Large)
```

### **Secondary Error**

```
Folder upload error: SyntaxError: Unexpected token 'R', "Request En"... is not valid JSON
```

### **Error Flow**

1. User selects folder with multiple files
2. Frontend sends FormData with all files to `/api/memories`
3. **Vercel platform** rejects request due to 4.5MB body size limit
4. Server returns HTML error page (413)
5. Client tries to parse HTML as JSON → SyntaxError

## **Root Cause Analysis**

### **Platform Limitation**

- **Vercel Serverless Functions**: Hard limit of **4.5MB** for request bodies
- **Application Target**: Designed to handle **12GB** uploads (500 files × 24MB each)
- **Mismatch**: 4.5MB limit blocks all folder uploads before reaching application code

### **Architecture Mismatch**

The application has **two separate upload systems**:

#### **System 1: Grant-Based Upload (Working)**

```
Large Files (>4MB) → /api/memories/grant → Presigned URL → Direct Upload
```

- ✅ Bypasses Vercel 4.5MB limit
- ✅ Supports files up to 5TB
- ✅ Used for single large files

#### **System 2: FormData Upload (Broken)**

```
Folder Uploads → FormData → /api/memories → ❌ 413 Error
```

- ❌ Hits Vercel 4.5MB limit
- ❌ All files sent in single request
- ❌ Used for folder uploads

## **Code Evidence**

### **Frontend: Folder Upload Implementation**

**File**: `src/nextjs/src/hooks/user-file-upload.ts`

```typescript
// Lines 354-363: Folder upload uses FormData
const formData = new FormData();
Array.from(files).forEach((file) => {
  formData.append("file", file);
});
// ❌ NO size check, NO grant system used

const response = await fetch("/api/memories", {
  method: "POST",
  body: formData, // ❌ Sends all files in single request
});
```

### **Backend: FormData Processing**

**File**: `src/nextjs/src/app/api/memories/post.ts`

```typescript
// Lines 67-70: Parses all files at once
const formData = await request.formData();
const files = formData.getAll("file") as File[];
// ❌ All files loaded into memory simultaneously
// ❌ Request body size = sum of all file sizes
```

### **Size Validation Mismatch**

**File**: `src/nextjs/src/config/upload-limits.ts`

```typescript
// Application expects to handle:
MAX_TOTAL_UPLOAD_SIZE_MB: 12000,  // 12GB
MAX_FILES_PER_UPLOAD: 600,        // 500 files

// But Vercel limits to:
// 4.5MB total request body
```

## **Impact Assessment**

### **User Experience**

- ❌ **Complete failure** of folder upload feature
- ❌ **No error recovery** - users get cryptic JSON parsing errors
- ❌ **No progress indication** - uploads fail silently
- ❌ **No partial success** - all-or-nothing approach

### **Business Impact**

- ❌ **Core feature broken** - folder uploads are primary use case
- ❌ **User frustration** - 413 errors are not user-friendly
- ❌ **Support burden** - users can't upload their content

### **Technical Debt**

- ❌ **Inconsistent architecture** - two different upload systems
- ❌ **Platform dependency** - tied to Vercel's 4.5MB limit
- ❌ **No scalability** - can't handle target 12GB uploads

## **Current Workarounds**

### **Single File Uploads**

- ✅ **Working**: Uses grant-based system for files >4MB
- ✅ **Working**: Direct S3 uploads for smaller files

### **Folder Uploads**

- ❌ **Broken**: No working alternative
- ❌ **No fallback**: Users must upload files individually

## **Technical Constraints**

### **Vercel Platform Limits**

- **Request Body**: 4.5MB maximum
- **Function Timeout**: 10 seconds (Hobby), 60 seconds (Pro)
- **Memory**: 1024MB maximum
- **No Configuration**: Cannot increase body size limits

### **Application Requirements**

- **Target Upload Size**: 12GB (500 files × 24MB)
- **File Count**: Up to 600 files per upload
- **User Experience**: Single-click folder upload
- **Performance**: Parallel processing preferred

## **Proposed Solutions (Analysis Only)**

### **Option 1: Grant-Based Folder Upload**

- **Approach**: Use existing grant system for each file in folder
- **Pros**: Leverages existing working system
- **Cons**: Requires significant frontend refactoring

### **Option 2: Chunked Upload System**

- **Approach**: Split large uploads into smaller chunks
- **Pros**: Works within Vercel limits
- **Cons**: Complex implementation, requires new infrastructure

### **Option 3: Direct Client Upload**

- **Approach**: Upload files directly to storage, bypass server
- **Pros**: No server limits, better performance
- **Cons**: Security implications, requires presigned URLs

### **Option 4: Alternative Platform**

- **Approach**: Deploy to platform with higher limits
- **Pros**: Solves root cause
- **Cons**: Migration effort, potential other limitations

## **Files Requiring Changes**

### **Frontend**

- `src/nextjs/src/hooks/user-file-upload.ts` - Folder upload logic
- `src/nextjs/src/components/memory/item-upload-button.tsx` - Upload UI
- `src/nextjs/src/services/upload.ts` - Upload service layer

### **Backend**

- `src/nextjs/src/app/api/memories/post.ts` - Main upload handler
- `src/nextjs/src/app/api/memories/grant/route.ts` - Grant system
- `src/nextjs/src/app/api/memories/utils/` - Upload utilities

### **Configuration**

- `src/nextjs/vercel.json` - Platform configuration
- `src/nextjs/next.config.ts` - Next.js configuration

## **Testing Scenarios**

### **Current Broken Scenarios**

1. **Small folder** (5 files, 2MB total) → ❌ 413 Error
2. **Medium folder** (20 files, 8MB total) → ❌ 413 Error
3. **Large folder** (100 files, 50MB total) → ❌ 413 Error

### **Working Scenarios**

1. **Single file** (<4MB) → ✅ Works
2. **Single large file** (>4MB) → ✅ Works (grant system)

## **Monitoring & Metrics**

### **Error Tracking**

- **413 Errors**: Currently not tracked
- **Upload Success Rate**: Unknown
- **User Impact**: No metrics available

### **Performance Metrics**

- **Upload Speed**: Not measurable (uploads fail)
- **File Processing Time**: Not measurable (uploads fail)
- **User Abandonment**: Likely high

## **Priority Assessment**

### **Severity**: 🔴 **CRITICAL**

- Core feature completely broken
- No working alternative for folder uploads
- Affects primary user workflow

### **Urgency**: 🔴 **HIGH**

- Users cannot upload their content
- Business impact is immediate
- Support burden is growing

### **Complexity**: 🟡 **MEDIUM**

- Requires architectural changes
- Multiple systems need coordination
- Testing across different file sizes needed

## **Next Steps**

1. **Immediate**: Document workaround for users (single file uploads)
2. **Short-term**: Implement grant-based folder upload
3. **Medium-term**: Add proper error handling and user feedback
4. **Long-term**: Consider platform migration or chunked upload system

---

**Created**: 2024-12-19  
**Last Updated**: 2024-12-19  
**Status**: 🔴 **OPEN** - Critical issue requiring immediate attention
