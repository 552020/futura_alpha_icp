# Frontend S3 Chunk Sizes & Derivative Assets Analysis

## 📊 Overview

Analysis of chunk sizes and derivative asset configurations used in the frontend S3 upload system to inform ICP implementation.

## 🔧 Upload Configuration

### **S3 Upload Limits** (`src/nextjs/src/config/upload-limits.ts`)

```typescript
export const UPLOAD_LIMITS_S3 = {
  // File size limits
  MAX_FILE_SIZE_MB: 24, // 20MB + 20% safety
  MAX_FILES_PER_UPLOAD: 600, // 500 + 20% safety
  MAX_TOTAL_UPLOAD_SIZE_MB: 12000, // 10GB + 20% safety
  
  // Inline storage limit (for database storage)
  INLINE_MAX_BYTES: 32 * 1024, // 32KB (database storage limit)
};
```

### **ICP Upload Limits** (`src/nextjs/src/config/upload-limits.ts`)

```typescript
export const UPLOAD_LIMITS_ICP = {
  // Chunking configuration
  CHUNK_SIZE_BYTES: 1.5 * 1024 * 1024, // 1.5MB chunks (frontend optimized)
  MAX_CHUNKS: 512, // Maximum number of chunks allowed
  MAX_FILE_SIZE_MB: 768, // 512 chunks × 1.5MB
  
  // Inline storage limit (for database storage - same as S3)
  INLINE_MAX_BYTES: 32 * 1024, // 32KB (database storage limit)
};
```

### **Backend ICP Limits** (`src/backend/src/upload/types.rs`)

```rust
pub const INLINE_MAX: u64 = 32 * 1024; // 32KB (database storage)
pub const CHUNK_SIZE: usize = 1_800_000; // 1.8MB (backend optimized)
```

## 🖼️ Image Derivative Sizes

### **Image Processing Configuration** (`src/nextjs/src/workers/image-processor.worker.ts`)

```typescript
// Default sizes (can be overridden)
const maxDisplaySize = 2048; // Display version: max 2048px
const maxThumbSize = 512; // Thumbnail version: max 512px
const maxPlaceholderSize = 32; // Placeholder version: max 32px
```

### **Image Processing Pipeline**

1. **Original** → **Display** (2048px max, WebP, quality 0.82)
2. **Display** → **Thumb** (512px max, WebP, quality 0.82)
3. **Thumb** → **Placeholder** (32px max, WebP, quality 0.6, data URL)

### **Processing Chain**

```
Original Image
    ↓
Display (2048px max) → Thumb (512px max) → Placeholder (32px max)
```

## 📏 Expected Asset Sizes

### **Typical Size Ranges** (based on processing pipeline)

| Asset Type      | Max Dimensions | Format   | Quality  | Typical Size Range |
| --------------- | -------------- | -------- | -------- | ------------------ |
| **Original**    | Original       | Original | Original | 1-20MB             |
| **Display**     | 2048px         | WebP     | 0.82     | 100KB-2MB          |
| **Thumb**       | 512px          | WebP     | 0.82     | 10KB-200KB         |
| **Placeholder** | 32px           | WebP     | 0.6      | 1KB-10KB           |

### **Size Reduction Factors**

- **Display**: ~10-20% of original size
- **Thumb**: ~1-5% of original size
- **Placeholder**: ~0.1-0.5% of original size

## 🔄 Chunking Strategy

### **Frontend ICP Chunking**

```typescript
// From icp-upload.ts
const limits = {
  inline_max: 1.5 * 1024 * 1024, // 1.5MB inline limit
  chunk_size: 1.5 * 1024 * 1024, // 1.5MB chunks
  max_chunks: 512, // Max 512 chunks
};

// Upload decision logic
const isInline = fileSize <= limits.inline_max;
const expectedChunks = Math.ceil(fileSize / limits.chunk_size);
```

### **Chunk Size Comparison**

| System           | Chunk Size | Inline Limit | Max Chunks | Max File Size |
| ---------------- | ---------- | ------------ | ---------- | ------------- |
| **S3**           | N/A        | 32KB         | N/A        | 24MB          |
| **Frontend ICP** | 1.5MB      | 32KB         | 512        | 768MB         |
| **Backend ICP**  | 1.8MB      | 32KB         | N/A        | N/A           |

## 🎯 Key Insights for ICP Implementation

### **1. Chunk Size Limits & Constraints**

**Frontend ICP Configuration:**

- **Chunk Size**: 1.5MB (`CHUNK_SIZE_BYTES`)
- **Inline Limit**: 1.5MB (`INLINE_MAX_BYTES`)
- **Max Chunks**: 512
- **Max File Size**: 768MB (512 × 1.5MB)

**Backend ICP Configuration:**

- **Chunk Size**: 1.8MB (`CHUNK_SIZE`)
- **Inline Limit**: 32KB (`INLINE_MAX`)
- **Max File Size**: No explicit limit (limited by canister memory)

**Key Constraint**: Frontend chunks (1.5MB) must fit within backend chunk limit (1.8MB)

- ✅ **Compatible**: 1.5MB ≤ 1.8MB
- **Recommendation**: **Keep different chunk sizes** - each system optimized for its constraints

### **2. Derivative Asset Storage**

- **Display**: Blob storage (~100KB-2MB)
- **Thumb**: Blob storage (~10KB-200KB)
- **Placeholder**: **Inline storage** (~1KB-10KB, data URL in database)

### **3. Processing Chain**

- Each derivative is processed from the previous one
- WebP format with quality settings
- Data URL for placeholder (base64 encoded)

### **4. Upload Strategy & Flow**

**Upload Decision Logic:**

```typescript
// Frontend decision
const isInline = fileSize <= 1.5MB; // Frontend inline limit
const expectedChunks = Math.ceil(fileSize / 1.5MB); // Frontend chunk size

// Backend validation
if (chunkSize > 1.8MB) { // Backend chunk limit
  throw Error("Chunk too large");
}
```

**Asset Storage Strategy:**

- **Original/Display/Thumb**: Always blob storage (chunked if >1.5MB)
- **Placeholder**: Always inline storage (data URL in database)
- **Max file size**: 768MB (512 chunks × 1.5MB)

**Upload Flow:**

1. **Frontend**: Chunks files at 1.5MB
2. **Backend**: Validates chunks ≤ 1.8MB
3. **Result**: Compatible and optimized for both systems

## 📋 Recommendations

### **1. Chunk Size Strategy**

```typescript
// Frontend: Optimized for ICP message limits
const FRONTEND_CHUNK_SIZE = 1.5 * 1024 * 1024; // 1.5MB
const FRONTEND_INLINE_MAX = 1.5 * 1024 * 1024; // 1.5MB
const FRONTEND_MAX_CHUNKS = 512;

// Backend: Optimized for canister processing
const BACKEND_CHUNK_SIZE = 1.8 * 1024 * 1024; // 1.8MB
const BACKEND_INLINE_MAX = 32 * 1024; // 32KB

// Compatibility: Frontend chunks fit within backend limits
// ✅ 1.5MB ≤ 1.8MB (chunk size)
// ✅ 1.5MB > 32KB (inline limit - frontend uses blob storage)
```

### **2. Derivative Upload Strategy**

- **Display**: Blob storage + chunked upload (100KB-2MB)
- **Thumb**: Blob storage + chunked upload (10KB-200KB)
- **Placeholder**: **Inline storage** (1KB-10KB, data URL in database)

### **3. Test Scenarios**

- **Small image**: Display/thumb chunked, placeholder inline
- **Medium image**: Display/thumb chunked, placeholder inline
- **Large image**: Display/thumb chunked, placeholder inline

### **4. Storage Strategy Clarification**

**Inline Storage (Database):**

- **Purpose**: Store small data directly in memory record
- **Limit**: 32KB (backend `INLINE_MAX`)
- **Use case**: Placeholder data URLs only

**Blob Storage (Separate):**

- **Purpose**: Store larger assets in dedicated blob storage
- **Limit**: No size limit (chunked upload)
- **Use case**: Original, display, thumb assets

### **5. Performance Considerations**

- **Parallel processing**: Lane A + Lane B simultaneously
- **Chunk optimization**: Use system-specific chunk sizes
- **Memory management**: Process derivatives sequentially to avoid memory spikes

## 🎯 Summary: ICP Chunked Upload Limits

### **The Key Point:**

ICP chunked upload has **TWO different chunk size limits**:

1. **Frontend Limit**: 1.5MB chunks (optimized for ICP message size)
2. **Backend Limit**: 1.8MB chunks (optimized for canister processing)

### **Why This Works:**

- **Frontend** chunks files at 1.5MB
- **Backend** accepts chunks up to 1.8MB
- **Result**: Frontend chunks fit within backend limits ✅

### **Upload Flow:**

```
File → Frontend (1.5MB chunks) → Backend (≤1.8MB validation) → Storage
```

### **Asset Storage:**

- **Original/Display/Thumb**: Blob storage (chunked if >1.5MB)
- **Placeholder**: Inline storage (data URL in database)

## 🔗 Related Files

- **Upload Limits**: `src/nextjs/src/config/upload-limits.ts`
- **Image Processing**: `src/nextjs/src/workers/image-processor.worker.ts`
- **ICP Upload**: `src/nextjs/src/services/upload/icp-upload.ts`
- **Image Derivatives**: `src/nextjs/src/services/upload/image-derivatives.ts`
- **S3 Processing**: `src/nextjs/src/services/upload/s3-with-processing.ts`
- **Backend Types**: `src/backend/src/upload/types.rs`
