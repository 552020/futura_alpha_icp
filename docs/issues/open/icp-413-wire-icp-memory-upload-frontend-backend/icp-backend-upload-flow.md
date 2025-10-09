# ICP Backend Upload Flow Issue

**Status**: ✅ **IMPLEMENTED** - Complete ICP Backend Upload Flow

## Problem - ✅ **RESOLVED**

When a user has `storagePreference: "icp"` (ICP-only preference), the current blob-first upload flow doesn't work because:

1. **Current Flow**: `uploadFile` → blob storage → `/api/memories` (Neon DB)
2. **ICP-Only Flow Needed**: `uploadFile` → ICP canister directly (no Neon DB calls)

**✅ SOLUTION IMPLEMENTED**: Complete 2-lane + 4-asset system with direct ICP canister uploads and dual storage integration.

## Current State - ✅ **IMPLEMENTED**

**ICP Upload Path (Working):**

- ✅ Hook detects hosting preferences with `blobHosting` includes `'icp'`
- ✅ Calls `uploadToICPWithProcessing()` directly via `single-file-processor.ts`
- ✅ Uploads to ICP canister via chunked uploads with 2-lane parallel processing
- ✅ Creates Neon database records via `finalizeAllAssets()` for dual storage
- ✅ Creates ICP memory edges via `createICPMemoryEdge()` for bidirectional linking

**Neon/Blob Upload Path (Working):**

- ✅ Hook calls appropriate upload services based on hosting preferences
- ✅ Uploads to blob storage (S3/Vercel Blob), then calls `/api/upload/complete`
- ✅ Creates database records with all 4 assets

## Issue - ✅ **RESOLVED**

**The new blob-first `uploadFile` function always calls `/api/memories` (Neon DB), but ICP-only users should bypass this entirely.**

**✅ SOLUTION IMPLEMENTED**: Complete routing system with hosting preferences.

## ✅ Solution Implemented - **COMPLETE**

Updated upload routing system to:

1. ✅ Check user's hosting preferences via `useHostingPreferences()` hook
2. ✅ If `blobHosting` includes `'icp'` → route to ICP upload service via `uploadToICPWithProcessing()`
3. ✅ If `blobHosting` includes `'s3'` → route to S3 upload service
4. ✅ If `blobHosting` includes `'vercel_blob'` → route to Vercel Blob upload service
5. ✅ Dual storage support with ICP memory edges for bidirectional linking

## Files Updated - ✅ **COMPLETE**

- ✅ `src/services/upload/single-file-processor.ts` - Complete ICP routing with hosting preferences
- ✅ `src/services/upload/icp-with-processing.ts` - Complete 2-lane + 4-asset ICP upload system
- ✅ `src/services/upload/icp-upload.ts` - Enhanced ICP upload with chunked uploads
- ✅ `src/hooks/use-file-upload.ts` - Complete hosting preference integration
- ✅ `src/hooks/use-hosting-preferences.ts` - Complete hosting preferences management
- ✅ `src/app/[lang]/user/settings/page.tsx` - Complete hosting preferences UI

## Implementation Details

**New Function Signature:**

```typescript
uploadFile(
  file: File,
  isOnboarding: boolean,
  existingUserId?: string,
  mode: UploadMode = "files",
  storageBackend: StorageBackend | StorageBackend[] = "vercel_blob",
  userStoragePreference?: "neon" | "icp" | "dual"  // ← NEW PARAMETER
)
```

**ICP Upload Flow:**

- ✅ Calls `/api/upload/intent` to get real `UploadStorage` with ICP canister config
- ✅ Calls `icpUploadService.uploadFile()` with real storage configuration
- ✅ Calls `/api/upload/verify` for upload verification (best-effort)
- ✅ Returns formatted response compatible with existing interface
- ✅ Bypasses blob storage and `/api/memories` entirely

**✅ Completed:**

- ✅ Hook passes user's actual storage preference from `useStoragePreferences()`
- ✅ Real upload storage obtained from `/api/upload/intent` API
- ✅ ICP authentication checked before upload
- ✅ Upload verification integrated
- ✅ Single file uploads working with unified flow

## Key Code References

- **ICP Upload**: `src/hooks/user-file-upload.ts:172-195`
- **Storage Preference**: `src/db/schema.ts:138` (`storagePreference: storage_pref_t`)
- **ICP Service**: `src/services/icp-upload.ts`
