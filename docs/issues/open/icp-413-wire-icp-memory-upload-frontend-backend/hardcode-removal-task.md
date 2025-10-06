# Hardcode Removal Task

**Priority**: High  
**Type**: Bug Fix  
**Status**: Pending

## Issue

The frontend upload hook has hardcoded ICP preferences for testing purposes, which prevents users from choosing their preferred storage option.

## Current Code

**File**: `src/nextjs/src/hooks/use-file-upload.ts` (lines 57-74)

```typescript
// 🚨 TEMPORARY HARDCODE: Force ICP blob hosting for testing
// TODO: REMOVE THIS HARDCODE - This is for testing ICP upload flow only
// This should be replaced with proper user preference checking
const userBlobHostingPreferences = ["icp"]; // HARDCODED FOR ICP TESTING

console.log("🚨 HARDCODED ICP PREFERENCES ACTIVE - This should appear in console!");

// Original code (commented out for testing):
// const userBlobHostingPreferences = preferences?.blobHosting || ['s3'];
```

## Required Changes

1. **Remove hardcoded preferences**:

   ```typescript
   // Replace hardcoded line with:
   const userBlobHostingPreferences = preferences?.blobHosting || ["s3"];
   ```

2. **Remove hardcoded preferences object**:

   ```typescript
   // Remove this entire block:
   const hardcodedPreferences = {
     frontendHosting: preferences?.frontendHosting || "vercel",
     backendHosting: preferences?.backendHosting || "vercel",
     databaseHosting: preferences?.databaseHosting || ["neon"],
     blobHosting: ["icp"] as BlobHosting[],
     updatedAt: preferences?.updatedAt,
   };
   ```

3. **Use actual preferences**:
   ```typescript
   // Use the real preferences object instead of hardcodedPreferences
   ```

## Testing Required

After removing the hardcode:

1. **Test S3 upload** - User with S3 preference should upload to S3
2. **Test ICP upload** - User with ICP preference should upload to ICP
3. **Test default behavior** - User with no preference should default to S3
4. **Test settings page** - Changing preferences should affect upload routing

## Files to Update

- `src/nextjs/src/hooks/use-file-upload.ts` - Remove hardcode
- Update any tests that depend on hardcoded behavior

## Acceptance Criteria

- ✅ Users can choose storage preference in settings
- ✅ Upload routing respects user choice
- ✅ Default behavior works (S3 fallback)
- ✅ All existing functionality preserved
- ✅ No hardcoded preferences in production code
