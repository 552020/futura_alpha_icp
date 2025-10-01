# Frontend ICP Upload Implementation

## 📋 **Issue Summary**

🔄 **IN PROGRESS** - Frontend-to-ICP backend upload functionality has been implemented and enhanced with valuable features from the redundant implementation. **Testing required** to confirm full functionality.

## 🎯 **Current State**

- ✅ **Backend**: ICP upload API with chunked uploads and blob_read endpoint
- ✅ **Node.js Uploader**: Working uploader with mainnet authentication
- ✅ **Settings**: Users can select ICP as blob hosting preference
- ✅ **Frontend**: Complete ICP upload implementation in `upload/icp-upload.ts`
- ✅ **Migration**: Enhanced with features from redundant class-based implementation
- 🔄 **Testing**: Implementation needs to be tested to confirm functionality

## 🔄 **Upload Flow**

```
Hosting Preferences (ICP selected) → Upload Button (File/Folder) → Routing Logic → Authentication Check → Upload Original + Asset Creation → Upload Derivative Assets
```

### **Detailed Flow:**

1. **Hosting Preferences** (ICP selected by default for II users)

   - Users who register through Internet Identity should have ICP as default for Blob/Backend/DB
   - Users can change preferences in settings page or through other UI components
   - **Note**: Users can have preferences without touching the settings page (e.g., through onboarding)
   - **Relevant files:**
     - `src/nextjs/src/app/[lang]/user/settings/page.tsx` - Settings UI
     - `src/nextjs/src/hooks/use-hosting-preferences.ts` - Hosting preferences hook
     - `src/nextjs/src/app/api/me/hosting-preferences/route.ts` - Hosting preferences API
     - `src/nextjs/auth.ts` - Authentication configuration
     - `src/nextjs/src/app/[lang]/user/icp/page.tsx` - ICP main page (reference for ICP patterns)
     - `src/nextjs/src/components/auth/user-button-client-with-ii.tsx` - II authentication components

2. **Upload Button** (File/Folder Upload)

   - User selects files or folders to upload
   - Triggers upload process

3. **Routing Logic** (single-file-processor.ts / multiple-file-processor.ts)

   - Determines upload destination based on user preferences
   - Routes to appropriate upload service (ICP, S3, Vercel Blob, etc.)

4. **Authentication Check** (Before Upload)

   - Check if user is authenticated with Internet Identity
   - Users authenticated with Google still need II for ICP uploads
   - Verify Actor and Agent creation for ICP communication

5. **Upload Original + Asset Creation**

   - Upload original file to ICP blob storage
   - Create asset records in database
   - Generate derivative assets (thumbnails, etc.)

6. **Upload Derivative Assets**
   - Upload generated thumbnails and other derivatives
   - Complete the upload process

## 📁 **Key Files**

### **Authentication & Settings:**

- `src/nextjs/auth.ts` - Authentication configuration
- `src/nextjs/src/app/[lang]/user/settings/page.tsx` - Hosting preferences UI
- `src/nextjs/src/hooks/use-hosting-preferences.ts` - Hosting preferences hook
- `src/nextjs/src/app/api/me/hosting-preferences/route.ts` - Hosting preferences API
- `src/nextjs/src/app/[lang]/user/icp/page.tsx` - ICP main page (reference for ICP patterns)
- `src/nextjs/src/components/auth/user-button-client-with-ii.tsx` - II authentication components

### **Upload Processing:**

- `src/nextjs/src/services/upload/single-file-processor.ts` - Upload routing logic
- `src/nextjs/src/services/upload/multiple-file-processor.ts` - Multiple file routing logic
- `src/nextjs/src/services/upload/icp-upload.ts` - ✅ **Complete ICP upload implementation**

### **Reference Implementation:**

- `tests/backend/shared-capsule/upload/ic-upload.mjs` - Working Node.js uploader

## 🔀 **Routing Logic**

### **Upload Destination Decision:**

The routing logic determines where to upload files based on user preferences:

```typescript
// In single-file-processor.ts / multiple-file-processor.ts
if (preferences.blob_storage === "icp") {
  // Route to ICP upload service
  const { uploadToICP } = await import("./icp-upload");
  results = await uploadToICP(files, preferences, onProgress);
} else if (preferences.blob_storage === "s3") {
  // Route to S3 upload service
  const { uploadToS3 } = await import("./s3-upload");
  results = await uploadToS3(files, preferences, onProgress);
} else if (preferences.blob_storage === "vercel_blob") {
  // Route to Vercel Blob upload service
  const { uploadToVercelBlob } = await import("./vercel-blob-upload");
  results = await uploadToVercelBlob(files, preferences, onProgress);
}
```

### **Upload Architecture Options:**

**Note**: Upload to ICP Blob (which is in the same canister as the backend) could happen:

- **Frontend side** (current approach) - Direct upload from browser to ICP
- **Backend side** (Vercel) - Upload to Vercel first, then to ICP

**Current Implementation**: We are going with the **frontend side** approach for direct ICP uploads.

### **Current Scope - Blob Storage Only:**

**Note**: At the moment we will add only the **Blob functionality**. This means we aim first to solve the problem of having files saved also in ICP, and we want to keep track of the metadata so we want to have also a copy of the Memory DB.

**Backend Architecture Note**: The organization of the backend is in **capsules** - we don't have a central Memory table, but each capsule representing a user has its own memory struct.

## 🗄️ **Backend Data Structure Comparison**

For detailed database schema comparison and field mapping between the current database (Neon/PostgreSQL) and ICP backend, see:

**→ [Backend Data Structure Comparison](./frontend-icp-upload-implementation-types.md)**

This document includes:

- Complete SQL and Drizzle schema definitions
- ICP Memory struct definitions with field mappings
- Missing fields analysis and compatibility issues
- Access control system comparison
- Data synchronization strategy

## ✅ **Implementation Completed**

1. ✅ **Create ICP upload service** (`icp-upload.ts`) - **DONE**
2. ✅ **Implement chunked upload** using existing Node.js uploader logic - **DONE**
3. ✅ **Add authentication** with Internet Identity - **DONE**
4. ✅ **Handle file processing** and response normalization - **DONE**
5. ✅ **Add error handling** and progress tracking - **DONE**
6. ✅ **Enhanced features** migrated from redundant implementation - **DONE**

## 🔄 **Next Steps - Testing Required**

### **Authentication Testing:**

1. 🔄 **Test II authentication check** - Verify users are prompted for II auth when needed
2. 🔄 **Test Actor/Agent creation** - Confirm proper ICP communication setup
3. 🔄 **Test Google + II dual auth** - Users with Google auth still need II for ICP uploads

### **Upload Flow Testing:**

4. 🔄 **Test routing logic** - Verify correct service selection based on preferences
5. 🔄 **Test upload flow** - Verify files can be uploaded to ICP
6. 🔄 **Test chunked uploads** - Verify large file handling
7. 🔄 **Test asset creation** - Verify original + derivative asset uploads
8. 🔄 **Test error handling** - Confirm proper error responses
9. 🔄 **Test progress tracking** - Verify progress callbacks work

### **Integration Testing:**

10. 🔄 **Test with settings page** - Verify default ICP selection for II users
11. 🔄 **Test with upload components** - Test with actual frontend components
12. 🔄 **Test multiple file uploads** - Verify batch upload functionality

## 🎯 **Success Criteria - 🔄 PENDING TESTING**

### **Core Functionality:**

- 🔄 Users can upload files to ICP when selected in settings
- 🔄 II users have ICP as default blob/backend/DB preference
- 🔄 Chunked uploads work for large files (>2MB)
- 🔄 Original + derivative asset uploads complete successfully

### **Authentication & Communication:**

- 🔄 Proper authentication with Internet Identity
- 🔄 Actor and Agent creation for ICP communication
- 🔄 Google-authenticated users can still upload to ICP (with II auth)
- 🔄 Authentication prompts work correctly

### **Integration & UX:**

- 🔄 Routing logic correctly selects ICP upload service
- 🔄 Consistent response format with other upload providers
- 🔄 Error handling and user feedback
- 🔄 Multiple file uploads work correctly

### **Enhanced Features (Implemented):**

- ✅ Enhanced progress tracking with detailed file information
- ✅ Utility functions for authentication status checking
- ✅ Agent reuse for better performance

## 🔗 **Related**

- Backend blob_read API: `feat(backend): add blob_read API endpoint`
- Node.js uploader: `feat(upload): implement Node.js uploader`
- Settings UI: Already implemented
- **Migration completed**: `feat(frontend): migrate and enhance ICP upload implementation`

## 📊 **Implementation Summary**

### **Files Created/Enhanced:**

- ✅ `src/nextjs/src/services/upload/icp-upload.ts` - Complete implementation (584 lines)
- ✅ Enhanced with 225+ lines of valuable features from redundant implementation

### **Key Features Implemented:**

- ✅ **Function-based pattern** - Consistent with project standards
- ✅ **Chunked upload support** - For large files (>2MB)
- ✅ **Internet Identity authentication** - Full II integration
- ✅ **Enhanced progress tracking** - Detailed file information
- ✅ **Utility functions** - Authentication helpers
- ✅ **Error handling** - Comprehensive error management
- ✅ **Agent reuse** - Performance optimization

### **Migration Results:**

- ✅ **Redundancy eliminated** - Deleted unused class-based implementation
- ✅ **Pattern consistency** - Function-based approach maintained
- ✅ **Feature enhancement** - All valuable features preserved and improved
