# Backend Upload Intent Validation Security Issue

## Problem

The upload system has a **dual-backend architecture conflict** that creates both security vulnerabilities and architectural inconsistencies:

### **Current Architecture**

- **Vercel Backend**: Acts as the "brain" - holds user state, preferences, and coordinates all upload decisions
- **ICP Backend**: Intended for web3-native users who want to bypass Vercel completely

### **The Conflict**

1. **Security Principle**: All upload routing decisions should be made by the backend (not frontend) to prevent malicious intervention
2. **ICP User Requirement**: Pure web3 users want direct frontend → ICP canister → ICP storage flow without touching Vercel
3. **Current Reality**: Frontend makes storage routing decisions, and intent verification is incomplete

### **Security Vulnerability**

Currently, the upload flow lacks proper backend validation of upload intent against stored user preferences, creating a potential security vulnerability where compromised frontend code could redirect uploads to unauthorized storage.

### **Intent Verification Attempt**

The system attempted to solve this with "intent verification" - checking against the backend just before uploading to validate that the preferences expressed by the frontend match the user's actual stored preferences. However, this implementation is incomplete and doesn't actually validate against stored preferences.

## Current State

**Frontend-controlled routing:**

```typescript
// Frontend decides storage routing
const userBlobHostingPreferences = preferences?.blobHosting || ["s3"];
if (userBlobHostingPreferences.includes("icp")) {
  data = await uploadToICP(file, preferences, onProgress);
}
```

**Intent endpoint returns static config:**

```typescript
// /api/upload/intent just returns what frontend asks for
const blob_storage = blobHosting || 's3'; // No validation against stored preferences
return { uploadStorage: { blob_storage, database: 'neon', ... } };
```

## Security Risk

1. **Compromised frontend**: Malicious code could redirect uploads to unauthorized storage
2. **No backend validation**: Backend doesn't verify upload intent against user's actual stored preferences
3. **Trust assumption**: System trusts frontend to make correct storage decisions

## Required Solution

**Backend should validate upload intent:**

1. **Frontend sends upload request** with intended storage
2. **Backend retrieves user's stored preferences** from database
3. **Backend validates**: Does the upload intent match stored preferences?
4. **Backend decides storage routing** based on validated preferences
5. **Backend returns storage config** for approved upload

## Implementation Requirements

### For Vercel Backend

- Retrieve user hosting preferences from Neon database
- Validate upload intent against stored preferences
- Return storage config only for approved uploads

### For ICP Backend (Future)

- ICP canister should act as "brain" for ICP-only flows
- Validate upload intent against user's ICP-stored preferences
- Enable direct frontend → ICP → ICP storage flow

## Priority

**Medium Priority** - Security enhancement for production deployment

## Related Files

### Archived Implementation (Removed for MVP)

The following files have been moved to `docs/issues/open/backend-upload-intent-validation-security-issue/` for documentation:

- `api-upload-intent-route.ts.md` - Original API route for intent verification
- `api-upload-verify-route.ts.md` - Original API route for upload verification
- `intent.ts.md` - Original intent verification service
- `verification.ts.md` - Original upload verification service
- `api-memories-post-legacy.md` - Original `/api/memories` POST handler (602 lines of multipart/JSON handling)
- `api-memories-post-analysis.md` - Analysis of the legacy POST handler and its dead code
- `legacy-upload-handler.md` - Legacy file upload handler (123 lines of FormData processing)
- `api-upload-commit-legacy.md` - Legacy single-file commit endpoint (42 lines, replaced by `/api/upload/complete`)
- `api-upload-batch-commit-legacy.md` - Legacy multiple files commit endpoint (62 lines, replaced by `/api/upload/complete`)
- `api-upload-presign-legacy.md` - Legacy single file presign endpoint (31 lines, replaced by `/api/upload/s3/presign`)
- `api-storage-status-legacy.md` - Legacy storage status endpoint (127 lines, unused)
- `api-memories-batch-legacy.md` - Legacy batch memory creation endpoint (228 lines, unused)
- `api-memories-commit-legacy.md` - Legacy memory asset commit endpoint (159 lines, unused)
- `api-memories-complete-legacy.md` - Legacy upload completion endpoint (107 lines, unused)
- `api-memories-complete-route-legacy.md` - Legacy upload completion route documentation
- `api-memories-complete-s3-legacy.md` - Legacy S3 upload completion endpoint
- `api-memories-grant-s3-legacy.md` - Legacy S3 grant endpoint (85 lines, unused)
- `api-memories-grant-s3-analysis.md` - Analysis of S3 grant endpoint functionality
- `api-memories-presign-legacy.md` - Legacy memory-specific presign endpoint (107 lines, unused)

### Current Implementation

- `src/nextjs/src/services/upload/single-file-processor.ts` - Upload routing logic
- `src/nextjs/src/services/upload/multiple-files-processor.ts` - Multiple files routing logic

## Notes

This issue addresses the core principle that **frontend should never decide storage routing alone** - the backend must validate and coordinate all storage decisions to prevent malicious intervention.
