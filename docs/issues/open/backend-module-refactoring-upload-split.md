# Backend Module Refactoring: Split Upload Module

## Problem Statement

The current `upload/` module contains two distinct domains that should be separated for better architecture and maintainability.

## Current State

### **`src/backend/src/upload/` Module Contains:**

- **Blob storage operations** (ICP blob store management)
- **Asset reference management** (memory asset metadata)
- **Storage backend abstraction** (S3, ICP, Vercel, etc.)
- **Upload session management** (chunked uploads)

### **Issues with Current Structure:**

1. **Mixed responsibilities** - Blob storage + asset management in one module
2. **Unclear naming** - "Upload" doesn't reflect asset lifecycle management
3. **Scope confusion** - Blob operations vs asset reference operations
4. **Future API conflicts** - New asset APIs don't fit "upload" concept

## Proposed Solution

### **Split `upload/` into two focused modules:**

#### **1. `blob/` Module (ICP Blob Storage)**

```
src/backend/src/blob/
├── store.rs          # ICP blob storage operations
├── service.rs        # Blob management service
├── types.rs          # BlobId, BlobMeta, BlobStore
└── tests/
```

**Scope**: Binary data storage in ICP blob store

- `BlobId(u64)` - ICP blob identifier
- `BlobStore` - ICP blob storage operations
- `blob_store()` - Store binary data in ICP
- `blob_retrieve()` - Get binary data from ICP
- `blob_delete()` - Delete binary data from ICP

#### **2. `assets/` Module (Memory Asset References)**

```
src/backend/src/assets/
├── service.rs        # Asset reference management
├── types.rs          # AssetReference, AssetMetadata
└── tests/
```

**Scope**: Asset references and metadata in memories

- `AssetReference` - Asset link metadata
- `AssetMetadata` - Asset description and tags
- `memories_remove_asset()` - Remove asset reference (no blob deletion)
- `memories_cleanup_assets()` - Clean up asset references
- `memories_list_assets()` - List asset references

## Domain Separation Analysis

### **`blob/` Domain (Storage Layer)**

- **Purpose**: Manage binary data storage in ICP
- **Operations**: Store, retrieve, delete binary data
- **Scope**: ICP blob store integration
- **Examples**: `blob_store()`, `blob_delete()`, `BlobId`

### **`assets/` Domain (Memory Layer)**

- **Purpose**: Manage asset references in memory metadata
- **Operations**: Add, remove, list asset references
- **Scope**: Memory asset metadata management
- **Examples**: `memories_remove_asset()`, `memories_cleanup_assets()`

## Key Differences

### **Asset Operations ≠ Blob Operations**

#### **Asset Operations (Memory-Scoped)**

- Remove asset reference from memory metadata
- Clean up asset links without touching blob storage
- List asset references in memory
- Update asset metadata in memory

#### **Blob Operations (Storage-Scoped)**

- Delete actual binary data from ICP blob store
- Store binary data in ICP blob store
- Retrieve binary data from ICP blob store

## Current Backend Architecture

### **Memory Structure:**

```rust
pub struct Memory {
    pub inline_assets: Vec<MemoryAssetInline>,           // Inline binary data
    pub blob_internal_assets: Vec<MemoryAssetBlobInternal>, // ICP blob references
    pub blob_external_assets: Vec<MemoryAssetBlobExternal>, // External storage references
}
```

### **Asset Types:**

- **`MemoryAssetInline`** - Inline binary data (stored in memory)
- **`MemoryAssetBlobInternal`** - Reference to ICP blob (BlobId)
- **`MemoryAssetBlobExternal`** - Reference to external storage (S3, Vercel, etc.)

## Benefits of Separation

### **1. Clear Domain Boundaries**

- **`blob/`** - Binary data storage operations
- **`assets/`** - Asset reference management operations

### **2. Better API Design**

- **Asset APIs**: `memories_*_asset*` functions
- **Blob APIs**: `blob_*` functions
- **Clear separation** of concerns

### **3. Future-Proof Architecture**

- **Asset management** can grow independently
- **Blob storage** can be optimized separately
- **New APIs** fit naturally into correct domains

### **4. Improved Maintainability**

- **Focused modules** with single responsibilities
- **Easier testing** of individual domains
- **Clearer code organization**

## Implementation Plan

### **Phase 1: Create New Modules**

1. **Create `src/backend/src/blob/`** module
2. **Create `src/backend/src/assets/`** module
3. **Move blob-related code** from `upload/` to `blob/`
4. **Move asset-related code** from `upload/` to `assets/`

### **Phase 2: Update Imports**

1. **Update `src/backend/src/lib.rs`** imports
2. **Update `src/backend/src/types.rs`** re-exports
3. **Update all references** throughout codebase

### **Phase 3: Add New APIs**

1. **Add asset management APIs** to `assets/` module
2. **Add blob management APIs** to `blob/` module
3. **Update Candid interface** with new endpoints

## Files to Modify

### **New Files:**

- `src/backend/src/blob/store.rs`
- `src/backend/src/blob/service.rs`
- `src/backend/src/blob/types.rs`
- `src/backend/src/assets/service.rs`
- `src/backend/src/assets/types.rs`

### **Modified Files:**

- `src/backend/src/lib.rs` - Update imports
- `src/backend/src/types.rs` - Update re-exports
- `src/backend/backend.did` - Update Candid interface

### **Removed Files:**

- `src/backend/src/upload/` - Split into blob/ and assets/

## Success Criteria

- [ ] **`blob/` module** contains only blob storage operations
- [ ] **`assets/` module** contains only asset reference management
- [ ] **All imports updated** throughout codebase
- [ ] **No breaking changes** to existing functionality
- [ ] **New APIs fit naturally** into correct domains
- [ ] **Tests pass** for both modules
- [ ] **Documentation updated** for new structure

## Future Enhancements

### **Asset Module Extensions**

- Asset metadata management
- Asset versioning
- Asset permissions
- Asset lifecycle tracking

### **Blob Module Extensions**

- Blob compression
- Blob encryption
- Blob replication
- Blob performance optimization

## Conclusion

Splitting the `upload/` module into `blob/` and `assets/` modules will:

1. **Clarify domain boundaries** between storage and metadata
2. **Improve API design** with focused, single-purpose modules
3. **Enable future growth** in both domains independently
4. **Simplify maintenance** with clear separation of concerns

This refactoring is essential for implementing the new bulk memory APIs and maintaining a clean, scalable architecture.

