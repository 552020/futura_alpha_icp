# Metadata Refactoring Architecture Decision

## 🎯 **Problem Statement**

The current ICP backend metadata structure has evolved from a single-asset model to a multi-asset model, but the metadata architecture hasn't been updated accordingly. This creates confusion and architectural inconsistencies.

### **Current Issues:**

1. **`MemoryMetadataBase`** was designed for single-asset memories, but now memories have multiple assets
2. **`MemoryMeta`** is confusingly named (it's actually asset metadata, not memory metadata)
3. **Asset metadata is incomplete** - missing many fields from the database `memory_assets` table
4. **No type-specific asset metadata** - all assets use the same metadata structure regardless of type
5. **`MemoryMetadata`** contains asset-specific info that should be in `AssetMetadata`
6. **`name` field confusion** - unclear purpose when we have `id` and `title`

## 🏗️ **Proposed Architecture**

### **Memory Level Structure:**

```rust
pub struct Memory {
    pub id: String,                            // ✅ UNIQUE IDENTIFIER (UUID)
    pub metadata: MemoryMetadata,              // ✅ MEMORY-LEVEL METADATA (title, description, etc.)
    pub access: MemoryAccess,                  // ✅ ACCESS CONTROL
    pub inline_assets: Vec<MemoryAssetInline>,        // ✅ ASSETS WITH AssetMetadata
    pub blob_internal_assets: Vec<MemoryAssetBlobInternal>,  // ✅ ASSETS WITH AssetMetadata
    pub blob_external_assets: Vec<MemoryAssetBlobExternal>,  // ✅ ASSETS WITH AssetMetadata
}
```

### **Enhanced MemoryMetadata (Memory-Level Metadata):**

```rust
pub struct MemoryMetadata {
    // Basic info
    pub memory_type: MemoryType,
    pub title: Option<String>,        // ✅ OPTIONAL TITLE (matches database)
    pub description: Option<String>,  // ✅ OPTIONAL DESCRIPTION (matches database)
    pub content_type: String,

    // Timestamps
    pub created_at: u64,
    pub updated_at: u64,
    pub uploaded_at: u64,
    pub date_of_memory: Option<u64>,     // when the actual event happened
    pub file_created_at: Option<u64>,    // when the original file was created

    // Organization
    pub parent_folder_id: Option<String>,
    pub tags: Vec<String>,               // Memory tags
    pub deleted_at: Option<u64>,

    // Content info
    pub people_in_memory: Option<Vec<String>>, // People in the memory
    pub location: Option<String>,        // Where the memory was taken
    pub memory_notes: Option<String>,    // Additional notes

    // System info
    pub created_by: Option<String>,      // Who created this memory
    pub database_storage_edges: Vec<StorageEdgeDatabaseType>,
}
```

### **Asset Level Structure:**

#### **Base Asset Metadata (Shared):**

```rust
pub struct AssetMetadataBase {
    // Basic info
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,

    // Asset classification
    pub asset_type: AssetType,        // Moved from asset struct to metadata

    // File properties
    pub bytes: u64,                   // File size
    pub mime_type: String,            // MIME type
    pub sha256: Option<String>,       // File hash

    // Dimensions (for images/videos)
    pub width: Option<u32>,
    pub height: Option<u32>,

    // Storage info
    pub url: Option<String>,          // Public URL (if applicable)
    pub storage_key: Option<String>,  // Storage system key
    pub bucket: Option<String>,       // Storage bucket/container
    pub asset_location: Option<String>, // Where the asset is stored

    // Processing status
    pub processing_status: Option<String>,
    pub processing_error: Option<String>,

    // Timestamps
    pub created_at: u64,
    pub updated_at: u64,
    pub deleted_at: Option<u64>,      // Soft delete support
}
```

#### **Type-Specific Asset Metadata Extensions:**

```rust
pub struct ImageAssetMetadata {
    pub base: AssetMetadataBase,
    pub color_space: Option<String>,
    pub exif_data: Option<String>,
    pub compression_ratio: Option<f32>,
    pub dpi: Option<u32>,
    pub orientation: Option<u8>,
}

pub struct VideoAssetMetadata {
    pub base: AssetMetadataBase,
    pub duration: Option<u64>,        // Duration in milliseconds
    pub frame_rate: Option<f32>,      // Frames per second
    pub codec: Option<String>,        // Video codec (H.264, VP9, etc.)
    pub bitrate: Option<u64>,         // Bitrate in bits per second
    pub resolution: Option<String>,   // Resolution string (e.g., "1920x1080")
    pub aspect_ratio: Option<f32>,    // Aspect ratio
}

pub struct AudioAssetMetadata {
    pub base: AssetMetadataBase,
    pub duration: Option<u64>,        // Duration in milliseconds
    pub sample_rate: Option<u32>,     // Sample rate in Hz
    pub channels: Option<u8>,         // Number of audio channels
    pub bitrate: Option<u64>,         // Bitrate in bits per second
    pub codec: Option<String>,        // Audio codec (MP3, AAC, etc.)
    pub bit_depth: Option<u8>,        // Bit depth (16, 24, 32)
}

pub struct DocumentAssetMetadata {
    pub base: AssetMetadataBase,
    pub page_count: Option<u32>,      // Number of pages
    pub document_type: Option<String>, // PDF, DOCX, etc.
    pub language: Option<String>,     // Document language
    pub word_count: Option<u32>,      // Word count
}

pub struct NoteAssetMetadata {
    pub base: AssetMetadataBase,
    pub word_count: Option<u32>,      // Word count
    pub language: Option<String>,     // Note language
    pub format: Option<String>,       // Markdown, plain text, etc.
}
```

#### **Unified Asset Metadata Enum:**

```rust
pub enum AssetMetadata {
    Image(ImageAssetMetadata),
    Video(VideoAssetMetadata),
    Audio(AudioAssetMetadata),
    Document(DocumentAssetMetadata),
    Note(NoteAssetMetadata),
}
```

#### **Updated Asset Structs:**

```rust
pub struct MemoryAssetInline {
    pub bytes: Vec<u8>,              // File data stored directly in canister
    pub metadata: AssetMetadata,     // Type-specific metadata
}

pub struct MemoryAssetBlobInternal {
    pub blob_ref: BlobRef,           // Reference to ICP blob storage (same canister)
    pub metadata: AssetMetadata,     // Type-specific metadata
}

pub struct MemoryAssetBlobExternal {
    pub location: StorageEdgeBlobType,  // Where the asset is stored externally
    pub storage_key: String,            // Key/ID in external storage system
    pub url: Option<String>,            // Public URL (if available)
    pub metadata: AssetMetadata,        // Type-specific metadata
}

// ✅ EXISTING: Already defined in types.rs
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum StorageEdgeBlobType {
    Icp,        // ICP canister storage
    VercelBlob, // Vercel Blob storage
    S3,         // AWS S3 storage
    Arweave,    // Arweave storage
    Ipfs,       // IPFS storage
    Neon,       // Neon database - for small assets
}
```

### **Benefits of Separate Asset Arrays:**

1. **Type Safety**: Compile-time guarantees about storage type and access patterns
2. **Performance**: Direct access without enum matching overhead
3. **Memory Efficiency**: No wasted space from unused enum variants
4. **Clear Separation**: Each storage type has optimized data structures
5. **Database Compatibility**: Direct mapping to `asset_location` field
6. **Scalability**: Can have multiple assets of each type without empty arrays

### **Storage Decision Matrix:**

| Asset Size | Access Pattern | Recommended Storage           | Reason                             |
| ---------- | -------------- | ----------------------------- | ---------------------------------- |
| < 32KB     | Frequent       | `Inline`                      | Fastest access, no external calls  |
| 32KB - 2MB | Frequent       | `BlobInternal` (ICP)          | Good balance of speed and capacity |
| > 2MB      | Occasional     | `BlobExternal` (S3/Vercel)    | Cost-effective for large files     |
| Any size   | Archive        | `BlobExternal` (Arweave/IPFS) | Permanent decentralized storage    |

## 🔄 **Migration Strategy**

### **Phase 1: Create New Structures** ✅ **COMPLETED**

1. ✅ Create `AssetMetadataBase` struct
2. ✅ Create type-specific metadata structs (`ImageAssetMetadata`, etc.)
3. ✅ Create `AssetMetadata` enum
4. ✅ Update asset structs to use new metadata

### **Phase 2: Update Constructors** ✅ **COMPLETED**

1. ✅ Update `Memory::inline` constructor
2. ✅ Update `Memory::from_blob` constructor
3. ✅ Update `create_memory_object` function
4. ✅ Update all test memory creation functions

### **Phase 3: Update All References** ✅ **COMPLETED**

1. ✅ Update `memories.rs` functions
2. ✅ Update `upload/service.rs` functions
3. ✅ Update `upload/types.rs` functions
4. ✅ Update all canister factory functions
5. ✅ Update all tests

### **Phase 4: Remove Old Structures** ✅ **COMPLETED**

1. ✅ Remove `MemoryMeta` struct
2. ✅ Remove `MemoryMetadataBase` struct (or repurpose it)
3. ✅ Remove `name` field from `MemoryMetadata` (replaced by `title`)
4. ✅ Clean up unused imports
5. ✅ Update documentation

**Note**: `MemoryMetadata` struct will stay (enhanced version replaces the old enum)

## 📊 **Impact Analysis**

### **Files That Were Updated:** ✅ **ALL COMPLETED**

- ✅ `src/backend/src/types.rs` (major changes)
- ✅ `src/backend/src/memories.rs`
- ✅ `src/backend/src/upload/service.rs`
- ✅ `src/backend/src/upload/types.rs`
- ✅ `src/backend/src/canister_factory/export.rs`
- ✅ `src/backend/src/canister_factory/import.rs`
- ✅ `src/backend/src/canister_factory/verify.rs`
- ✅ `src/backend/src/capsule.rs`
- ✅ `src/backend/src/lib.rs`
- ✅ All test files and capsule store files

### **Benefits:**

1. **Type Safety**: Compile-time guarantees about asset-specific metadata
2. **Completeness**: All database fields are now represented
3. **Clarity**: Clear separation between memory and asset metadata
4. **Extensibility**: Easy to add new asset types and metadata fields
5. **Database Compatibility**: Direct mapping to database schema

### **Risks:** ✅ **ALL MITIGATED**

1. ✅ **Heavy Refactoring**: Successfully completed across all files
2. ✅ **Compilation Errors**: All resolved - project compiles successfully
3. ✅ **Testing**: All 174 unit tests passing
4. ✅ **Time**: Completed efficiently with systematic approach

## 🤔 **Alternative Approaches**

### **Option 1: Heavy Refactoring (Proposed)** ✅ **CHOSEN & COMPLETED**

- ✅ Complete architectural overhaul
- ✅ Type-specific metadata for each asset type
- ✅ Maximum type safety and completeness
- ✅ High development effort - successfully completed

### **Option 2: Light Refactoring**

- Just rename `MemoryMeta` → `AssetMetadata`
- Add missing fields to single metadata struct
- Keep existing architecture
- Lower development effort

### **Option 3: Hybrid Approach**

- Create `AssetMetadataBase` with common fields
- Keep single `AssetMetadata` struct but with all fields
- Add type-specific fields as optional fields
- Medium development effort

## 📋 **Decision Points**

1. **Should we proceed with the heavy refactoring?** ✅ **YES - DECIDED**
2. **Which asset types need type-specific metadata?** ✅ **ALL TYPES - Image, Video, Audio, Document, Note**
3. **Should we keep `MemoryMetadataBase` or remove it?** ✅ **REMOVE - Replace with enhanced MemoryMetadata**
4. **Should we keep `MemoryMetadata` enum?** ✅ **REMOVE - Replace with enhanced MemoryMetadata struct**
5. **Should we keep `name` field in MemoryMetadata?** ✅ **REMOVE - Use `title` instead (matches database)**
6. **How should we handle the migration timeline?** ✅ **INCREMENTAL - Phase by phase**
7. **Should we implement this incrementally or all at once?** ✅ **INCREMENTAL - Safer approach**

## 🎯 **Next Steps**

1. **✅ Decision**: Heavy refactoring approach chosen
2. **✅ Planning**: Detailed migration plan created
3. **✅ Implementation**: All phases completed successfully
4. **✅ Testing**: All 174 unit tests passing
5. **✅ Documentation**: Architecture documented and implemented

## 🎯 **Final Architecture Summary**

### **Key Changes:**

- **Remove `MemoryMetadata` enum** → Replace with enhanced `MemoryMetadata` struct
- **Remove `MemoryMetadataBase`** → Replace with enhanced `MemoryMetadata` struct
- **Remove `name` field** → Use `title: Option<String>` (matches database)
- **Enhance `AssetMetadata`** → Type-specific metadata for each asset type
- **Separate asset arrays** → `inline_assets`, `blob_internal_assets`, `blob_external_assets`

### **Final Structure:**

```rust
pub struct Memory {
    pub id: String,                            // UUID identifier
    pub metadata: MemoryMetadata,              // Memory-level metadata
    pub access: MemoryAccess,                  // Access control
    pub inline_assets: Vec<MemoryAssetInline>,        // Inline assets
    pub blob_internal_assets: Vec<MemoryAssetBlobInternal>,  // ICP blob assets
    pub blob_external_assets: Vec<MemoryAssetBlobExternal>,  // External blob assets
}
```

### **Benefits:**

- **Database Alignment**: Perfect match with web2 database schema
- **Type Safety**: Compile-time guarantees for asset-specific metadata
- **Clear Separation**: Memory-level vs asset-level metadata
- **Performance**: Direct access without enum matching
- **Scalability**: Multiple assets per memory with optimized storage

## 📝 **Notes**

- This refactoring aligns with the database schema in `schema.ts`
- The new architecture provides better type safety and completeness
- Consider the development timeline and team capacity
- Ensure backward compatibility during migration if possible

## 🎉 **Implementation Results**

### **✅ Successfully Completed:**

- **All 4 phases** of the migration strategy completed
- **174 unit tests** passing with 0 failures
- **Complete compilation** success with only minor warnings
- **Full type safety** achieved with new metadata architecture
- **Database alignment** - perfect match with web2 database schema

### **📊 Test Results:**

```
running 175 tests
test result: ok. 174 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

### **🔧 Key Achievements:**

1. **Architecture Overhaul**: Complete transformation from single-asset to multi-asset model
2. **Type Safety**: Compile-time guarantees for asset-specific metadata
3. **Database Compatibility**: Direct mapping to `schema.ts` database structure
4. **Performance**: Optimized storage with separate arrays for different asset types
5. **Maintainability**: Clear separation between memory-level and asset-level metadata

### **🚀 Ready for Next Phase:**

The backend is now ready for:

- Frontend integration testing
- ICP upload implementation
- End-to-end testing
- Production deployment preparation

**Status**: ✅ **COMPLETE** - All objectives achieved successfully!
