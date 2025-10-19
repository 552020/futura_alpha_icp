# 🔍 S3 Upload Flow Analysis

**Date:** 2024-12-19  
**Status:** Analysis Complete  
**Priority:** High  
**Labels:** `analysis`, `s3-upload`, `image-processing`, `vercel-blob`, `onboarding`  
**Purpose:** Understand S3 upload flow to reproduce with Vercel Blob

## 📋 **Summary**

Comprehensive analysis of the S3 upload flow that creates multiple assets (original, display, thumbnail, placeholder) and how to reproduce this exact functionality using Vercel Blob for onboarding uploads.

## 🏗️ **S3 Upload Architecture**

### **Two-Lane Parallel Processing**

The S3 upload system uses a sophisticated **two-lane parallel processing** approach:

```
┌─────────────────────────────────────────────────────────────┐
│                    UPLOAD PIPELINE                          │
├─────────────────────────────────────────────────────────────┤
│  Lane A: Original Upload          Lane B: Derivative Proc. │
│  ┌─────────────────────────┐      ┌─────────────────────────┐│
│  │ 1. Get S3 Grants        │      │ 1. Process Image       ││
│  │ 2. Upload Original      │      │ 2. Create Display      ││
│  │ 3. Create Memory        │      │ 3. Create Thumbnail    ││
│  │ 4. Create Asset Record   │      │ 4. Create Placeholder  ││
│  └─────────────────────────┘      └─────────────────────────┘│
│           │                              │                │
│           └──────────────┬───────────────────┘                │
│                        │                                    │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              FINALIZE ALL ASSETS                      │ │
│  │  • Create derivative asset records                     │ │
│  │  • Update processing status                            │ │
│  │  │  • Handle failures gracefully                      │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## 🎯 **Asset Types Created**

### **1. Original Asset**

- **Purpose**: Full resolution file for archival
- **Storage**: S3 bucket
- **Processing**: None (uploaded as-is)
- **Database**: `assetType: 'original'`

### **2. Display Asset**

- **Purpose**: Optimized for viewing in galleries
- **Specifications**:
  - **Size**: ~2048px long edge
  - **Format**: WebP
  - **Quality**: 85%
  - **File Size**: 150-400KB
- **Database**: `assetType: 'display'`

### **3. Thumbnail Asset**

- **Purpose**: Grid thumbnails, previews
- **Specifications**:
  - **Size**: ~512px long edge
  - **Format**: WebP
  - **Quality**: 80%
  - **File Size**: 20-60KB
- **Database**: `assetType: 'thumb'`

### **4. Placeholder Asset**

- **Purpose**: Loading states, blur effects
- **Specifications**:
  - **Size**: ~32px long edge
  - **Format**: Data URL (base64)
  - **File Size**: ~1KB
- **Database**: `assetType: 'placeholder'`

## 🔧 **Key Functions Analysis**

### **1. Main Upload Function**

```typescript
// src/services/upload/s3-with-processing.ts
export async function uploadToS3WithProcessing(
  file: File,
  onProgress?: (progress: number) => void
): Promise<UploadServiceResult>;
```

**Process:**

1. **Get S3 Grants** - Request presigned URLs
2. **Start Lane A** - Upload original to S3
3. **Start Lane B** - Process image derivatives
4. **Wait for both lanes** - Parallel execution
5. **Finalize assets** - Create database records

### **2. Pure Image Processing**

```typescript
// src/services/upload/image-derivatives.ts
export async function processImageDerivativesPure(file: File): Promise<ProcessedBlobs>;
```

**Pure Functions Used:**

- `processImageDerivativesWithWorkerPure()` - Web Worker processing
- `calculateResizeDimensions()` - Dimension calculations
- `createAssetDataFromProcessed()` - Asset metadata creation

### **3. Asset Creation**

```typescript
// src/app/api/memories/utils/memory-database.ts
export async function processMultipleFilesBatch(params: {
  files: File[];
  urls: string[];
  ownerId: string;
  assetLocation?: "s3" | "vercel_blob";
});
```

**Database Records Created:**

- **Memory record** - Main memory entry
- **Original asset** - Full resolution file
- **Display asset** - Optimized viewing version
- **Thumbnail asset** - Grid preview version
- **Placeholder asset** - Loading state version

## 📊 **Processing Specifications**

### **Image Processing Constants**

```typescript
// src/app/api/memories/utils/image-processing.ts
export const DISPLAY_MAX_SIZE = 2048; // Maximum long edge for display
export const THUMB_MAX_SIZE = 512; // Maximum long edge for thumbnail
```

### **Quality Settings**

```typescript
// Backend processing (Sharp)
original: sharp(buffer).webp({ quality: 90 }).toBuffer();
display: sharp(buffer).resize().webp({ quality: 85 }).toBuffer();
thumb: sharp(buffer).resize().webp({ quality: 80 }).toBuffer();
```

### **Resize Logic**

```typescript
// Calculate dimensions maintaining aspect ratio
const displaySize = calculateResizeDimensions(originalWidth, originalHeight, 2048);
const thumbSize = calculateResizeDimensions(originalWidth, originalHeight, 512);
```

## 🎯 **Vercel Blob Implementation Plan**

### **Phase 1: Pure Function Extraction**

**Extract these pure functions:**

1. `processImageDerivativesPure()` - Core image processing
2. `calculateResizeDimensions()` - Dimension calculations
3. `createAssetDataFromProcessed()` - Asset metadata creation
4. `detectMemoryTypeFromFile()` - File type detection

### **Phase 2: Vercel Blob Upload Function**

```typescript
// New function: uploadToVercelBlobWithProcessing
export async function uploadToVercelBlobWithProcessing(
  file: File,
  isOnboarding: boolean,
  onProgress?: (progress: number) => void
): Promise<UploadServiceResult> {
  // 1. Upload original to Vercel Blob
  const originalResult = await uploadOriginalToVercelBlob(file, isOnboarding);

  // 2. Process image derivatives (pure function)
  const processedBlobs = await processImageDerivativesPure(file);

  // 3. Upload derivatives to Vercel Blob
  const derivativeResults = await uploadProcessedAssetsToVercelBlob(processedBlobs);

  // 4. Create memory and asset records
  const memoryResult = await createMemoryWithAssets(originalResult, derivativeResults);

  return memoryResult;
}
```

### **Phase 3: Asset Upload Functions**

```typescript
// Upload processed assets to Vercel Blob
async function uploadProcessedAssetsToVercelBlob(processedBlobs: ProcessedBlobs): Promise<ProcessedAssets> {
  const uploadPromises = [];

  if (processedBlobs.display) {
    uploadPromises.push(uploadDerivativeToVercelBlob(processedBlobs.display, "display"));
  }

  if (processedBlobs.thumb) {
    uploadPromises.push(uploadDerivativeToVercelBlob(processedBlobs.thumb, "thumb"));
  }

  if (processedBlobs.placeholder) {
    uploadPromises.push(uploadPlaceholderToVercelBlob(processedBlobs.placeholder));
  }

  return await Promise.all(uploadPromises);
}
```

## 🔄 **Database Schema Compatibility**

### **Asset Location Field**

```typescript
// Support both S3 and Vercel Blob
assetLocation: "s3" | "vercel_blob";
```

### **Storage Key Handling**

```typescript
// S3: Extract key from full URL
// Vercel Blob: Use pathname from blob result
storageKey: assetLocation === "s3" ? extractS3Key(url) : extractVercelBlobPathname(url);
```

## 🧪 **Testing Strategy**

### **1. Pure Function Tests**

```typescript
// Test image processing without uploads
describe("processImageDerivativesPure", () => {
  it("should create display, thumb, and placeholder assets", async () => {
    const result = await processImageDerivativesPure(testImage);
    expect(result.display).toBeDefined();
    expect(result.thumb).toBeDefined();
    expect(result.placeholder).toBeDefined();
  });
});
```

### **2. Integration Tests**

```typescript
// Test full upload flow with Vercel Blob
describe("uploadToVercelBlobWithProcessing", () => {
  it("should upload original and create all derivative assets", async () => {
    const result = await uploadToVercelBlobWithProcessing(testImage, true);
    expect(result.memoryId).toBeDefined();
    expect(result.assets).toHaveLength(4); // original, display, thumb, placeholder
  });
});
```

## 📁 **File Structure for Implementation**

```
src/services/upload/
├── vercel-blob-with-processing.ts    # Main upload function
├── vercel-blob-derivatives.ts        # Asset upload functions
├── image-derivatives.ts              # Pure processing functions (existing)
└── shared/
    ├── asset-creation.ts             # Database record creation
    ├── dimension-calculations.ts      # Resize logic
    └── file-type-detection.ts        # File type utilities
```

## 🎯 **Success Criteria**

### **Functional Requirements**

- [ ] Upload original file to Vercel Blob
- [ ] Create display asset (2048px, WebP, 85% quality)
- [ ] Create thumbnail asset (512px, WebP, 80% quality)
- [ ] Create placeholder asset (32px, base64)
- [ ] Create database records for all assets
- [ ] Support onboarding flow (no authentication)

### **Performance Requirements**

- [ ] Parallel processing (original + derivatives)
- [ ] Progress tracking for uploads
- [ ] Error handling for failed derivatives
- [ ] Cleanup on failure

### **Compatibility Requirements**

- [ ] Same database schema as S3
- [ ] Same asset types and specifications
- [ ] Same pure functions (reusable)
- [ ] Same database records

## 📝 **Next Steps**

1. **Extract Pure Functions** - Move processing logic to shared modules
2. **Create Vercel Blob Upload Function** - Implement main upload logic
3. **Implement Asset Upload** - Handle derivative uploads
4. **Database Integration** - Create asset records
5. **Testing** - Verify functionality matches S3 flow
6. **Onboarding Integration** - Route onboarding users to Vercel Blob

---

**Conclusion**: The S3 upload flow is well-architected with pure functions that can be reused for Vercel Blob implementation. The key is maintaining the same asset specifications and database schema while changing only the storage backend.
