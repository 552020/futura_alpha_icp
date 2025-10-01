# Implement 2-Lane + 4-Asset Upload System for ICP

**Priority**: High  
**Type**: Feature Implementation  
**Assigned To**: Development Team  
**Created**: 2025-01-01  
**Status**: In Progress

## 🎯 Objective

Implement the 2-lane + 4-asset upload system for ICP backend, reproducing the frontend S3 upload architecture as a backend test before implementing it in the frontend for ICP.

## 📋 Background

### **Frontend S3 System (Reference)**

The frontend currently uses a sophisticated 2-lane + 4-asset upload system:

- **Lane A**: Upload original file directly to S3
- **Lane B**: Process image derivatives (display, thumb, placeholder) and upload them
- **4 Asset Types**: Original, Display, Thumb, Placeholder

**Reference Files:**

- **Main S3 Service**: `src/nextjs/src/lib/s3.ts`
- **2-Lane + 4-Asset System**: `src/nextjs/src/services/upload/s3-with-processing.ts`
- **Image Processing**: `src/nextjs/src/services/upload/image-derivatives.ts`
- **Finalization**: `src/nextjs/src/services/upload/finalize.ts`
- **S3 Grants**: `src/nextjs/src/services/upload/s3-grant.ts`
- **Shared Utils**: `src/nextjs/src/services/upload/shared-utils.ts`

### **ICP Implementation Goal**

Reproduce this system using ICP backend canisters instead of S3, with:

- **Lane A**: Upload original file to ICP blob storage
- **Lane B**: Process image derivatives and upload to ICP blob storage
- **Finalization**: Create memory with all 4 assets

### **Function Reuse Strategy**

**Goal**: Reuse frontend functions whenever possible to maintain consistency and reduce duplication.

**Functions to Reuse (Non-S3 Related):**

- **Image Processing**: `processImageDerivativesPure()` from `image-derivatives.ts`
  - Pure image processing logic (resizing, format conversion)
  - No S3 dependencies, can be used directly
- **Asset Metadata**: Asset type definitions and metadata structures
- **Utility Functions**: File handling, validation, error handling

**Functions to Adapt (S3-Specific):**

- **Upload Logic**: Replace S3 upload with ICP blob upload
- **Storage References**: Replace S3 URLs with ICP blob locators
- **Grant System**: Replace S3 presigned URLs with ICP upload sessions

**Implementation Plan:**

1. **Phase 1**: Import and reuse pure image processing functions
2. **Phase 2**: Adapt upload logic for ICP backend
3. **Phase 3**: Create ICP-specific finalization logic
4. **Phase 4**: Integrate with frontend for production use

### **Chunk Size Analysis**

**Frontend Configuration:**

- **Chunk Size**: 1.5MB (`UPLOAD_LIMITS_ICP.CHUNK_SIZE_BYTES`)
- **Inline Limit**: 1.5MB (`UPLOAD_LIMITS_ICP.INLINE_MAX_BYTES`)
- **Max Chunks**: 512
- **Max File Size**: 768MB (512 × 1.5MB)

**Backend Configuration:**

- **Chunk Size**: 1.8MB (`CHUNK_SIZE` in `types.rs`)
- **Inline Limit**: 32KB (`INLINE_MAX` in `types.rs`)

**Derivative Asset Sizes:**

- **Display**: ~100KB-2MB (max 2048px, WebP quality 0.82)
- **Thumb**: ~10KB-200KB (max 512px, WebP quality 0.82)
- **Placeholder**: ~1KB-10KB (max 32px, WebP quality 0.6, data URL)

**Recommendation**: Align frontend and backend chunk sizes to 1.5MB for consistency.

**Analysis Document**: `docs/analysis/frontend-s3-chunk-sizes-analysis.md`

## 🏗️ Architecture

### **2-Lane System**

```
┌─────────────────┐    ┌─────────────────┐
│     Lane A      │    │     Lane B      │
│                 │    │                 │
│ Original Upload │    │ Image Processing│
│ (Direct to ICP) │    │ (Derivatives)   │
│                 │    │                 │
└─────────────────┘    └─────────────────┘
         │                       │
         └───────────┬───────────┘
                     │
            ┌─────────────────┐
            │   Finalization  │
            │ (Create Memory) │
            │  with 4 Assets  │
            └─────────────────┘
```

### **4 Asset Types**

1. **Original**: Full-resolution original file
2. **Display**: Optimized for display (e.g., 1920px max width)
3. **Thumb**: Thumbnail version (e.g., 300px max width)
4. **Placeholder**: Low-quality placeholder (e.g., 50px max width)

## 📁 Current Implementation

### **Test File**: `test_upload_2lane_4asset_system.mjs`

- **Location**: `tests/backend/shared-capsule/upload/`
- **Status**: Scaffolded, needs completion
- **Chunk Size**: 1.8MB (optimized)

### **Key Components (Functional Approach - Matches Frontend)**

1. **Lane A**: `uploadOriginalToICP()` - Original file upload (matches `uploadOriginalToS3`)
2. **Lane B**: `processImageDerivativesToICP()` - Image processing (matches `processImageDerivativesPure`)
3. **Finalization**: `finalizeAllAssets()` - Memory creation (matches frontend `finalizeAllAssets`)
4. **Main Function**: `uploadToICPWithProcessing()` - Orchestrates both lanes (matches `uploadToS3WithProcessing`)

## 🚧 Current Status

### **✅ Completed**

- [x] Test file scaffolded
- [x] Refactored to functional approach (matches frontend S3 system)
- [x] Lane A implementation (original upload)
- [x] Chunk size optimization (1.8MB)
- [x] Backend CHUNK_SIZE updated
- [x] Memory allocation decision (no forced allocation)
- [x] Fixed chunk size calculation (exact bytes vs floating point)
- [x] Fixed uploads_finish parameters (hash, totalLen)
- [x] Fixed blob ID formatting (blob\_{id})
- [x] Lane B image processing simulation
- [x] Asset metadata structure for 4 asset types

### **🔄 In Progress**

- [ ] Memory creation with multiple assets (placeholder issue)
- [ ] Error handling and validation
- [ ] Integration testing

### **⏳ Pending**

- [ ] Import and reuse frontend `processImageDerivativesPure()` function
- [ ] Real image processing logic (currently simulated)
- [ ] Memory finalization with all assets
- [ ] Performance validation
- [ ] Frontend integration
- [ ] Align chunk sizes between frontend (1.5MB) and backend (1.8MB)

### **🐛 Current Issues**

1. **Placeholder Memory Creation**: "Invalid opt vec nat8 argument" error
2. **Blob Meta Retrieval**: Some tests still failing blob_get_meta calls
3. **Test Interruption**: Tests sometimes hang or get interrupted
4. **Error Handling**: Need better error messages and recovery

### **📊 Test Results Summary**

**Latest Test Run:**

- **Total Tests**: 5
- **Passed**: 2 (Lane B Image Processing, Parallel Lanes Execution)
- **Failed**: 3 (Lane A Original Upload, Complete System, Asset Retrieval)

**Key Achievements:**

- ✅ **Chunk Size Optimization**: 1.8MB chunks working (3 chunks for 3.6MB file)
- ✅ **Upload Process**: Chunk upload and finish working
- ✅ **Lane B Simulation**: Image processing simulation working
- ✅ **Parallel Execution**: Both lanes can run simultaneously

**Remaining Challenges:**

- ❌ **Blob Meta Retrieval**: `blob_get_meta` calls failing with "Unsupported locator format"
- ❌ **Placeholder Creation**: Memory creation with inline data failing
- ❌ **Asset Retrieval**: Final asset validation failing

## 🧪 Test Requirements

### **Test Scenarios**

1. **Basic 2-Lane Upload**: Original + 3 derivatives
2. **Large File Handling**: 3.6MB+ files with 1.8MB chunks
3. **Error Handling**: Failed processing, network issues
4. **Performance**: Upload time, memory usage
5. **Validation**: Asset integrity, metadata correctness

### **Success Criteria**

- [ ] All 4 assets uploaded successfully
- [ ] Memory created with proper asset references
- [ ] Image processing produces correct derivatives
- [ ] No ResourceExhausted errors
- [ ] Performance meets expectations

## 🔧 Technical Details

### **Backend Functions Used**

- `uploads_begin()`: Start upload session
- `uploads_put_chunk()`: Upload file chunks
- `uploads_finish()`: Complete upload
- `memories_create()`: Create memory with assets
- `blob_read_chunk()`: Read uploaded blobs

### **Asset Metadata Structure**

```javascript
const assetMetadata = {
  Image: {
    dpi: [],
    color_space: [],
    base: {
      // Asset-specific metadata
      asset_type: { Original: null }, // or Display, Thumb, Placeholder
      // ... other fields
    },
    exif_data: [],
    compression_ratio: [],
    orientation: [],
  },
};
```

### **Chunk Configuration**

- **Chunk Size**: 1.8MB (optimal for ICP)
- **Memory Allocation**: Default best-effort (no forced allocation)
- **Expected Performance**: 91% efficiency improvement

## 🐛 Known Issues

### **Current Problems**

1. **Image Processing**: Lane B implementation incomplete
2. **Asset Types**: Need to define Display, Thumb, Placeholder variants
3. **Memory Creation**: Multiple asset handling needs completion
4. **Error Handling**: Comprehensive error scenarios not covered

### **Previous Issues Resolved**

- ✅ **ResourceExhausted**: Fixed with chunk size optimization
- ✅ **Memory Allocation**: Decided on default best-effort
- ✅ **Chunk Size**: Optimized to 1.8MB

## 📊 Performance Targets

### **Expected Results**

- **Upload Time**: < 10 seconds for 3.6MB file
- **Memory Usage**: Efficient chunk processing
- **Success Rate**: 100% for test scenarios
- **Asset Quality**: Proper image derivatives

### **Benchmarks**

- **64KB chunks**: 83 seconds (baseline)
- **1.8MB chunks**: 8 seconds (target)
- **Efficiency**: 91% improvement

## 🎯 Next Steps

### **Immediate (This Session)**

1. **Complete Lane B**: Implement image processing logic
2. **Asset Types**: Define Display, Thumb, Placeholder variants
3. **Memory Creation**: Handle multiple assets in finalization
4. **Test Execution**: Run complete 2-lane + 4-asset test

### **Short Term**

1. **Error Handling**: Add comprehensive error scenarios
2. **Performance Testing**: Validate with larger files
3. **Integration**: Test with frontend integration points

### **Long Term**

1. **Frontend Integration**: Implement in frontend for ICP
2. **Production Deployment**: Deploy optimized system
3. **Monitoring**: Add performance monitoring

## 📚 Related Issues

- [Upload Chunk Size Optimization](./upload-chunk-size-optimization-issue.md) - ✅ Resolved
- [Blob Lookup Performance Issue](./blob-lookup-performance-issue.md) - ✅ Resolved
- [Memory Storage Critical Bug](./memory_storage_critical_bug.md) - ✅ Resolved

## 🔗 References

- [Frontend S3 Upload System](../../../src/nextjs/src/lib/s3.ts)
- [Image Processing Utils](../../../src/nextjs/src/app/api/memories/utils/image-processing.ts)
- [S3 with Processing Service](../../../src/nextjs/src/services/upload/s3-with-processing.ts)
- [Image Derivatives Service](../../../src/nextjs/src/services/upload/image-derivatives.ts)

---

**Last Updated**: 2025-01-01  
**Next Review**: After Lane B implementation completion
