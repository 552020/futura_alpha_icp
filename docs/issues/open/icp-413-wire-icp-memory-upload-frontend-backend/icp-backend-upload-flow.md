# ICP Backend Upload Flow Issue

## Problem

When a user has `storagePreference: "icp"` (ICP-only preference), the current blob-first upload flow doesn't work because:

1. **Current Flow**: `uploadFile` → blob storage → `/api/memories` (Neon DB)
2. **ICP-Only Flow Needed**: `uploadFile` → ICP canister directly (no Neon DB calls)

## Current State

**ICP Upload Path (Working):**

- Hook detects `storage.chosen_storage === "icp-canister"`
- Calls `icpUploadService.uploadFile()` directly
- Uploads to ICP canister, no API calls to `/api/memories`

**Neon/Blob Upload Path (New):**

- Hook calls `uploadFile()` from `services/upload.ts`
- Uploads to blob storage, then calls `/api/memories`

## Issue

**The new blob-first `uploadFile` function always calls `/api/memories` (Neon DB), but ICP-only users should bypass this entirely.**

## ✅ Solution Implemented (Stub)

Updated `uploadFile` function to:

1. ✅ Check user's storage preference parameter
2. ✅ If `userStoragePreference === "icp"` → route to ICP upload service
3. ✅ If `userStoragePreference === "neon"` → use current blob-first flow
4. ✅ If `userStoragePreference === "dual"` → use blob-first flow (default)

## Files Updated

- ✅ `src/services/upload.ts` - Added ICP preference routing with stub implementation
- ⏳ `src/hooks/user-file-upload.ts` - Needs to pass user preference to uploadFile

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
