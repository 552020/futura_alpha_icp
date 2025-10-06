# Backend API Documentation

## Overview

This document provides comprehensive technical documentation for all ICP Backend API endpoints, covering capsule management, memory operations, asset management, gallery operations, and administrative functions.

**Last Updated**: October 6, 2025  
**Last Commit Checked**: `4d9580e` - Complete upload service refactoring with multiple asset support  
**Location**: `src/backend/src/lib.rs` and related modules

## System Architecture & Concepts

### 🏠 **Capsules: Digital Identity Containers**

A **Capsule** represents a user's digital identity and serves as the primary container for organizing their digital life. Think of it as a personal digital vault that can contain memories, galleries, and connections.

**Key Concepts:**

- **Self-Capsule**: Each user has exactly **one self-capsule** where they are the subject/owner
- **Multiple Capsules**: Users can create and manage **unlimited additional capsules** for others (e.g., a capsule for their grandson, preparing a capsule for future family members)
- **Shared Canister**: Currently, all capsules live in a shared canister, but users will soon be able to create their own autonomous canisters
- **Access Control**: Capsules have sophisticated permission systems with owners, controllers, and connection groups

### 🧠 **Memories: Digital Life Events**

**Memories** are the core content units that represent life events, experiences, and digital artifacts. Each memory can contain multiple assets optimized for different use cases.

**Architecture:**

- **Metadata Storage**: Memory metadata is stored directly in the capsule structure for fast access
- **Asset Storage**: Assets are stored in a blob system within the central canister
- **Future Split**: Soon we'll have both central blob storage and capsule-specific blob storage
- **Multiple Assets**: A single memory can hold multiple assets (original + optimized versions)

**Asset Types:**

- **Original**: Full-resolution, unprocessed file
- **Display**: Optimized for viewing (~1600-2048px, WebP format)
- **Thumbnail**: Small version for grids (~320-512px, WebP format)
- **Placeholder**: Low-quality placeholder (blurhash, base64) for fast loading

**Future: Memory Clusters**
We will introduce **memory clusters** to group related memories of the same episode (e.g., multiple photos and videos from a wedding kiss moment).

### 🖼️ **Galleries: Curated Collections**

**Galleries** are collections of memories organized around themes, events, or personal preferences. They provide a way to curate and share specific sets of memories.

**Key Features:**

- **Multiple Collections**: The same memory can be part of multiple galleries
- **Flexible Organization**: Create galleries for any theme (vacations, family events, work projects)
- **Sharing**: Galleries can be shared with specific people or groups

### 📁 **Folders: Hierarchical Organization**

**Folders** provide a hierarchical way to organize memories within a capsule, similar to a file system.

**Constraints:**

- **Single Parent**: A memory can only belong to one folder at a time
- **Tree Structure**: Folders can be nested to create complex organizational structures
- **Capsule-Scoped**: Folders are specific to each capsule

### 🔄 **Storage Evolution**

**Current State:**

- All capsules live in a shared canister
- Assets stored in central blob system
- Metadata stored in capsule structures

**Future Vision:**

- **Autonomous Canisters**: Users can create their own canisters with complete control
- **Distributed Storage**: Split between central and capsule-specific blob storage
- **Enhanced Privacy**: Complete data ownership and control

## API Overview

The ICP Backend provides a comprehensive set of APIs organized into 6 main categories:

### 📦 **Capsule Management (7 endpoints)**

Core container management for organizing memories and galleries:

- **`capsules_create()`** - Create new capsules (self or for others)

  - `capsules_create(subject: Option<PersonRef>) -> Result<Capsule, Error>` → [Details](#create-capsule)

- **`capsules_read_full()`** - Get complete capsule data with all contents

  - `capsules_read_full(capsule_id: Option<String>) -> Result<Capsule, Error>` → [Details](#read-capsule-full)

- **`capsules_read_basic()`** - Get lightweight capsule information

  - `capsules_read_basic(capsule_id: Option<String>) -> Result<CapsuleInfo, Error>` → [Details](#read-capsule-basic-info)

- **`capsules_update()`** - Update capsule properties and settings

  - `capsules_update(capsule_id: String, updates: CapsuleUpdateData) -> Result<Capsule, Error>` → [Details](#update-capsule)

- **`capsules_delete()`** - Permanently delete capsules

  - `capsules_delete(capsule_id: String) -> Result<(), Error>` → [Details](#delete-capsule)

- **`capsules_list()`** - List all accessible capsules

  - `capsules_list() -> Vec<CapsuleHeader>` → [Details](#list-capsules)

- **`capsules_bind_neon()`** - Bind/unbind to external database storage
  - `capsules_bind_neon(resource_type: ResourceType, resource_id: String, bind: bool) -> Result<(), Error>` → [Details](#bind-to-neon-database)

### 🧠 **Memory Management (8 endpoints)**

Digital memory creation, organization, and lifecycle management:

- **`memories_create()`** - Create memories with inline, blob, or external assets

  - `memories_create(capsule_id: CapsuleId, bytes: Option<Vec<u8>>, blob_ref: Option<BlobRef>, external_location: Option<StorageEdgeBlobType>, external_storage_key: Option<String>, external_url: Option<String>, external_size: Option<u64>, external_hash: Option<Vec<u8>>, asset_metadata: AssetMetadata, idem: String) -> Result20` → [Details](#create-memory)

- **`memories_create_with_internal_blobs()`** - Create memories with multiple internal blobs

  - `memories_create_with_internal_blobs(capsule_id: CapsuleId, memory_metadata: MemoryMetadata, internal_blob_assets: Vec<InternalBlobAssetInput>, idem: String) -> Result20` → [Details](#create-memory-with-internal-blobs)

- **`memories_delete()`** - Delete single memory with optional asset cleanup

  - `memories_delete(memory_id: String, delete_assets: bool) -> Result<(), Error>` → [Details](#delete-memory)

- **`memories_delete_bulk()`** - Delete multiple memories efficiently

  - `memories_delete_bulk(capsule_id: String, memory_ids: Vec<String>, delete_assets: bool) -> Result<BulkDeleteResult, Error>` → [Details](#bulk-delete-memories)

- **`memories_delete_all()`** - Clear entire capsule (high-risk operation)

  - `memories_delete_all(capsule_id: String, delete_assets: bool) -> Result<BulkDeleteResult, Error>` → [Details](#delete-all-memories)

- **`memories_update()`** - Update memory metadata and properties

  - `memories_update(memory_id: String, updates: MemoryUpdateData) -> Result<Memory, Error>` → [Details](#update-memory)

- **`memories_list()`** - List memories with pagination support

  - `memories_list(capsule_id: String, cursor: Option<String>, limit: Option<u32>) -> Result<Page<MemoryHeader>, Error>` → [Details](#list-memories)

- **`memories_cleanup_assets_all()`** - Remove all assets while preserving memory
  - `memories_cleanup_assets_all(memory_id: String) -> Result<AssetCleanupResult, Error>` → [Details](#cleanup-all-assets)

### 🎯 **Asset Management (8 endpoints)**

Granular control over individual assets within memories:

- **`asset_remove()`** - Remove specific asset by reference

  - `asset_remove(memory_id: String, asset_ref: String) -> Result<AssetRemovalResult, Error>` → [Details](#remove-asset)

- **`asset_remove_inline()`** - Remove inline asset by index

  - `asset_remove_inline(memory_id: String, asset_index: u32) -> Result<AssetRemovalResult, Error>` → [Details](#remove-inline-asset)

- **`asset_remove_internal()`** - Remove ICP blob asset by reference

  - `asset_remove_internal(memory_id: String, blob_ref: String) -> Result<AssetRemovalResult, Error>` → [Details](#remove-internal-asset)

- **`asset_remove_external()`** - Remove external storage asset by key

  - `asset_remove_external(memory_id: String, storage_key: String) -> Result<AssetRemovalResult, Error>` → [Details](#remove-external-asset)

- **`asset_remove_by_id()`** - Remove asset by unique ID

  - `asset_remove_by_id(memory_id: String, asset_id: String) -> Result<AssetRemovalResult, Error>` → [Details](#remove-asset-by-id)

- **`asset_get_by_id()`** - Retrieve specific asset data

  - `asset_get_by_id(memory_id: String, asset_id: String) -> Result<MemoryAsset, Error>` → [Details](#get-asset-by-id)

- **`memories_list_assets()`** - List all assets in a memory

  - `memories_list_assets(memory_id: String) -> Result<MemoryAssetsList, Error>` → [Details](#list-memory-assets)

- **`memories_cleanup_assets_bulk()`** - Bulk asset cleanup across memories
  - `memories_cleanup_assets_bulk(memory_ids: Vec<String>) -> Result<BulkResult<String>, Error>` → [Details](#bulk-cleanup-assets)

### 🖼️ **Gallery Management (7 endpoints)**

Collection and curation of memories into organized galleries:

- **`galleries_create()`** - Create new galleries

  - `galleries_create(gallery_data: GalleryData) -> Result<Gallery, Error>` → [Details](#create-gallery)

- **`galleries_create_with_memories()`** - Create galleries with memory synchronization

  - `galleries_create_with_memories(gallery_data: GalleryData, sync_memories: bool) -> Result<Gallery, Error>` → [Details](#create-gallery-with-memories)

- **`galleries_read()`** - Get complete gallery data

  - `galleries_read(gallery_id: String) -> Result<Gallery, Error>` → [Details](#read-gallery)

- **`galleries_update()`** - Update gallery properties and contents

  - `galleries_update(gallery_id: String, updates: GalleryUpdateData) -> Result<Gallery, Error>` → [Details](#update-gallery)

- **`galleries_delete()`** - Delete galleries

  - `galleries_delete(gallery_id: String) -> Result<(), Error>` → [Details](#delete-gallery)

- **`galleries_list()`** - List all accessible galleries

  - `galleries_list() -> Vec<GalleryHeader>` → [Details](#list-galleries)

- **`update_gallery_storage_location()`** - Change gallery storage backend
  - `update_gallery_storage_location(gallery_id: String, new_location: GalleryStorageLocation) -> Result<(), Error>` → [Details](#update-gallery-storage-location)

### 📤 **Upload & Blob Management (8 endpoints)**

File upload, storage, and retrieval infrastructure:

- **`uploads_begin()`** - Start new upload session

  - `uploads_begin(capsule_id: String, total_size: u64, chunk_count: u32, content_type: String, idem: String) -> Result<UploadBeginResult, Error>` → [Details](#begin-upload)

- **`uploads_chunk()`** - Upload file chunks

  - `uploads_chunk(pmid: String, chunk_index: u32, chunk_data: Vec<u8>) -> Result<(), Error>` → [Details](#upload-chunk)

- **`uploads_finish()`** - Complete upload and get blob ID

  - `uploads_finish(pmid: String) -> Result<BlobId, Error>` → [Details](#finish-upload)

- **`uploads_abort()`** - Cancel upload session

  - `uploads_abort(pmid: String) -> Result<(), Error>` → [Details](#abort-upload)

- **`blob_read()`** - Read complete blob data

  - `blob_read(locator: String) -> Result<Vec<u8>, Error>` → [Details](#read-blob)

- **`blob_read_chunk()`** - Read blob data in chunks

  - `blob_read_chunk(locator: String, chunk_index: u32) -> Result<Vec<u8>, Error>` → [Details](#read-blob-chunk)

- **`blob_get_meta()`** - Get blob metadata and statistics

  - `blob_get_meta(locator: String) -> Result<BlobMeta, Error>` → [Details](#get-blob-metadata)

- **`blob_delete()`** - Delete blob from storage
  - `blob_delete(locator: String) -> Result<(), Error>` → [Details](#delete-blob)

### ⚙️ **Administrative (5 endpoints)**

System administration and maintenance functions:

- **`add_admin()`** - Grant admin privileges

  - `add_admin(principal: Principal) -> Result<(), Error>` → [Details](#add-admin)

- **`remove_admin()`** - Revoke admin privileges

  - `remove_admin(principal: Principal) -> Result<(), Error>` → [Details](#remove-admin)

- **`list_admins()`** - List all administrators

  - `list_admins() -> Vec<Principal>` → [Details](#list-admins)

- **`list_superadmins()`** - List super administrators

  - `list_superadmins() -> Vec<Principal>` → [Details](#list-super-admins)

- **`clear_all_stable_memory()`** - Emergency data reset
  - `clear_all_stable_memory() -> Result<(), Error>` → [Details](#clear-all-stable-memory)

### 🔑 **Key Features**

- **Access Control**: Comprehensive ACL system with owner/controller permissions
- **Bulk Operations**: Efficient batch processing for large datasets
- **Asset Flexibility**: Support for inline, internal blob, and external storage
- **Idempotency**: Safe retry mechanisms with deterministic IDs
- **Error Handling**: Detailed error reporting with recovery guidance
- **Pagination**: Efficient data retrieval for large collections

## Table of Contents

1. [Capsule Management APIs](#capsule-management-apis)
2. [Memory Management APIs](#memory-management-apis)
3. [Asset Management APIs](#asset-management-apis)
4. [Gallery Management APIs](#gallery-management-apis)
5. [Upload & Blob APIs](#upload--blob-apis)
6. [Administrative APIs](#administrative-apis)
7. [Data Structures](#data-structures)
8. [Error Handling](#error-handling)

---

## Capsule Management APIs

### Create Capsule

```rust
capsules_create(subject: Option<PersonRef>) -> Result<Capsule, Error>
```

**Parameters:**

- `subject: Option<PersonRef>` - Optional subject for the capsule. If `None`, uses caller as subject.

**Returns:**

- `Result<Capsule, Error>` - Created capsule or error

**Behavior:**

- If `subject` is `None`, creates a self-capsule for the caller
- If `subject` is provided, creates a capsule for that subject
- Only one self-capsule per principal is allowed
- Returns existing self-capsule if attempting to create duplicate

**Location**: `src/backend/src/lib.rs:105-110`

### Read Capsule (Full)

```rust
capsules_read_full(capsule_id: Option<String>) -> Result<Capsule, Error>
```

**Parameters:**

- `capsule_id: Option<String>` - Optional capsule ID. If `None`, returns caller's self-capsule.

**Returns:**

- `Result<Capsule, Error>` - Full capsule data or error

**Behavior:**

- Returns complete capsule with all memories, galleries, and connections
- Requires ownership or controller access
- Returns `NotFound` if capsule doesn't exist
- Returns `Unauthorized` if caller lacks access

**Location**: `src/backend/src/lib.rs:124-130`

### Read Capsule (Basic Info)

```rust
capsules_read_basic(capsule_id: Option<String>) -> Result<CapsuleInfo, Error>
```

**Parameters:**

- `capsule_id: Option<String>` - Optional capsule ID. If `None`, returns caller's self-capsule info.

**Returns:**

- `Result<CapsuleInfo, Error>` - Basic capsule information or error

**Behavior:**

- Returns lightweight capsule information without full data
- If `capsule_id` is `None`, returns caller's self-capsule
- More efficient than `capsules_read_full()` for list views

**Location**: `src/backend/src/lib.rs:113-121`

### Update Capsule

```rust
capsules_update(capsule_id: String, updates: CapsuleUpdateData) -> Result<Capsule, Error>
```

**Parameters:**

- `capsule_id: String` - Unique identifier of the capsule
- `updates: CapsuleUpdateData` - Fields to update

**Returns:**

- `Result<Capsule, Error>` - Updated capsule or error

**Behavior:**

- Updates specified fields in the capsule
- Requires ownership or controller access
- Most fields are immutable (subject, owners, etc.)
- Only certain fields can be updated

**Location**: `src/backend/src/lib.rs:133-139`

### Delete Capsule

```rust
capsules_delete(capsule_id: String) -> Result<(), Error>
```

**Parameters:**

- `capsule_id: String` - Unique identifier of the capsule

**Returns:**

- `Result<(), Error>` - Success or error

**Behavior:**

- Permanently deletes the capsule and all associated data
- Requires ownership or controller access
- Cannot be undone
- Cascades to memories, galleries, and connections

**Location**: `src/backend/src/lib.rs:142-145`

### List Capsules

```rust
capsules_list() -> Vec<CapsuleHeader>
```

**Returns:**

- `Vec<CapsuleHeader>` - List of capsule headers

**Behavior:**

- Returns all capsules accessible to the caller
- Includes ownership and control status
- Returns basic information for each capsule
- Efficient for displaying capsule lists

**Location**: `src/backend/src/lib.rs:148-151`

### Bind to Neon Database

```rust
capsules_bind_neon(resource_type: ResourceType, resource_id: String, bind: bool) -> Result<(), Error>
```

**Parameters:**

- `resource_type: ResourceType` - Type of resource to bind
- `resource_id: String` - ID of the resource
- `bind: bool` - Whether to bind or unbind

**Returns:**

- `Result<(), Error>` - Success or error

**Behavior:**

- Binds or unbinds capsule to Neon database
- Enables external storage and retrieval
- Affects data persistence and access patterns
- Requires ownership or controller access

**Location**: `src/backend/src/lib.rs:154-161`

---

## Memory Management APIs

### Create Memory

```rust
memories_create(
    capsule_id: CapsuleId,
    bytes: Option<Vec<u8>>,
    blob_ref: Option<BlobRef>,
    external_location: Option<StorageEdgeBlobType>,
    external_storage_key: Option<String>,
    external_url: Option<String>,
    external_size: Option<u64>,
    external_hash: Option<Vec<u8>>,
    asset_metadata: AssetMetadata,
    idem: String,
) -> Result20
```

**Purpose**: Single unified endpoint handling all asset types

**Location**: `src/backend/src/lib.rs:267-302`

### Create Memory with Internal Blobs

```rust
memories_create_with_internal_blobs(
    capsule_id: CapsuleId,
    memory_metadata: MemoryMetadata,
    internal_blob_assets: Vec<InternalBlobAssetInput>,
    idem: String,
) -> Result20
```

**Purpose**: Specialized endpoint for multiple internal blob assets

**Location**: `src/backend/src/lib.rs:304-320`

### Delete Memory

```rust
memories_delete(memory_id: String, delete_assets: bool) -> Result<(), Error>
```

**Parameters:**

- `memory_id: String` - Unique identifier of the memory
- `delete_assets: bool` - Whether to delete associated assets

**Returns:**

- `Result<(), Error>` - Success or error

**Location**: `src/backend/src/lib.rs:415-423`

### Bulk Delete Memories

```rust
memories_delete_bulk(
    capsule_id: String,
    memory_ids: Vec<String>,
    delete_assets: bool,
) -> Result<BulkDeleteResult, Error>
```

**Parameters:**

- `capsule_id: String` - Capsule containing the memories
- `memory_ids: Vec<String>` - List of memory IDs to delete
- `delete_assets: bool` - Whether to delete associated assets

**Returns:**

- `Result<BulkDeleteResult, Error>` - Deletion results with counts

**Location**: `src/backend/src/lib.rs:1123-1135`

### Delete All Memories

```rust
memories_delete_all(capsule_id: String, delete_assets: bool) -> Result<BulkDeleteResult, Error>
```

**Parameters:**

- `capsule_id: String` - Capsule to clear
- `delete_assets: bool` - Whether to delete associated assets

**Returns:**

- `Result<BulkDeleteResult, Error>` - Deletion results with counts

**Location**: `src/backend/src/lib.rs:1139-1150`

### Update Memory

```rust
memories_update(memory_id: String, updates: MemoryUpdateData) -> Result<Memory, Error>
```

**Parameters:**

- `memory_id: String` - Unique identifier of the memory
- `updates: MemoryUpdateData` - Fields to update

**Returns:**

- `Result<Memory, Error>` - Updated memory or error

**Location**: `src/backend/src/lib.rs:405-412`

### List Memories

```rust
memories_list(
    capsule_id: String,
    cursor: Option<String>,
    limit: Option<u32>,
) -> Result<Page<MemoryHeader>, Error>
```

**Parameters:**

- `capsule_id: String` - Capsule to list memories from
- `cursor: Option<String>` - Pagination cursor
- `limit: Option<u32>` - Maximum number of memories to return

**Returns:**

- `Result<Page<MemoryHeader>, Error>` - Paginated list of memory headers

**Location**: `src/backend/src/lib.rs:426-460`

---

## Asset Management APIs

### Cleanup All Assets

```rust
memories_cleanup_assets_all(memory_id: String) -> Result<AssetCleanupResult, Error>
```

**Parameters:**

- `memory_id: String` - Memory to clean up assets from

**Returns:**

- `Result<AssetCleanupResult, Error>` - Cleanup results

**Purpose**: Remove ALL assets while preserving memory metadata

**Location**: `src/backend/src/lib.rs:1154-1164`

### Bulk Cleanup Assets

```rust
memories_cleanup_assets_bulk(memory_ids: Vec<String>) -> Result<BulkResult<String>, Error>
```

**Parameters:**

- `memory_ids: Vec<String>` - List of memory IDs to clean up

**Returns:**

- `Result<BulkResult<String>, Error>` - Bulk cleanup results

**Location**: `src/backend/src/lib.rs:1168-1196`

### Remove Asset

```rust
asset_remove(memory_id: String, asset_ref: String) -> Result<AssetRemovalResult, Error>
```

**Parameters:**

- `memory_id: String` - Memory containing the asset
- `asset_ref: String` - Asset reference to remove

**Returns:**

- `Result<AssetRemovalResult, Error>` - Removal results

**Location**: `src/backend/src/lib.rs:1204-1215`

### Remove Inline Asset

```rust
asset_remove_inline(memory_id: String, asset_index: u32) -> Result<AssetRemovalResult, Error>
```

**Parameters:**

- `memory_id: String` - Memory containing the asset
- `asset_index: u32` - Index of inline asset to remove

**Returns:**

- `Result<AssetRemovalResult, Error>` - Removal results

**Location**: `src/backend/src/lib.rs:1219-1229`

### Remove Internal Asset

```rust
asset_remove_internal(memory_id: String, blob_ref: String) -> Result<AssetRemovalResult, Error>
```

**Parameters:**

- `memory_id: String` - Memory containing the asset
- `blob_ref: String` - Blob reference to remove

**Returns:**

- `Result<AssetRemovalResult, Error>` - Removal results

**Location**: `src/backend/src/lib.rs:1234-1244`

### Remove External Asset

```rust
asset_remove_external(memory_id: String, storage_key: String) -> Result<AssetRemovalResult, Error>
```

**Parameters:**

- `memory_id: String` - Memory containing the asset
- `storage_key: String` - Storage key to remove

**Returns:**

- `Result<AssetRemovalResult, Error>` - Removal results

**Location**: `src/backend/src/lib.rs:1249-1259`

### Remove Asset by ID

```rust
asset_remove_by_id(memory_id: String, asset_id: String) -> Result<AssetRemovalResult, Error>
```

**Parameters:**

- `memory_id: String` - Memory containing the asset
- `asset_id: String` - Asset ID to remove

**Returns:**

- `Result<AssetRemovalResult, Error>` - Removal results

**Location**: `src/backend/src/lib.rs:1264-1274`

### List Memory Assets

```rust
memories_list_assets(memory_id: String) -> Result<MemoryAssetsList, Error>
```

**Parameters:**

- `memory_id: String` - Memory to list assets from

**Returns:**

- `Result<MemoryAssetsList, Error>` - List of assets in the memory

**Location**: `src/backend/src/lib.rs:1291-1300`

### Get Asset by ID

```rust
asset_get_by_id(memory_id: String, asset_id: String) -> Result<MemoryAsset, Error>
```

**Parameters:**

- `memory_id: String` - Memory containing the asset
- `asset_id: String` - Asset ID to retrieve

**Returns:**

- `Result<MemoryAsset, Error>` - Asset data

**Location**: `src/backend/src/lib.rs:1302-1312`

---

## Gallery Management APIs

### Create Gallery

```rust
galleries_create(gallery_data: GalleryData) -> Result<Gallery, Error>
```

**Parameters:**

- `gallery_data: GalleryData` - Gallery creation data

**Returns:**

- `Result<Gallery, Error>` - Created gallery or error

**Location**: `src/backend/src/lib.rs:167-172`

### Create Gallery with Memories

```rust
galleries_create_with_memories(
    gallery_data: GalleryData,
    sync_memories: bool,
) -> Result<Gallery, Error>
```

**Parameters:**

- `gallery_data: GalleryData` - Gallery creation data
- `sync_memories: bool` - Whether to sync with existing memories

**Returns:**

- `Result<Gallery, Error>` - Created gallery or error

**Location**: `src/backend/src/lib.rs:175-181`

### Update Gallery Storage Location

```rust
update_gallery_storage_location(
    gallery_id: String,
    new_location: GalleryStorageLocation,
) -> Result<(), Error>
```

**Parameters:**

- `gallery_id: String` - Gallery to update
- `new_location: GalleryStorageLocation` - New storage location

**Returns:**

- `Result<(), Error>` - Success or error

**Location**: `src/backend/src/lib.rs:184-190`

### List Galleries

```rust
galleries_list() -> Vec<GalleryHeader>
```

**Returns:**

- `Vec<GalleryHeader>` - List of gallery headers

**Location**: `src/backend/src/lib.rs:193-196`

### Read Gallery

```rust
galleries_read(gallery_id: String) -> Result<Gallery, Error>
```

**Parameters:**

- `gallery_id: String` - Gallery to read

**Returns:**

- `Result<Gallery, Error>` - Gallery data or error

**Location**: `src/backend/src/lib.rs:199-203`

### Update Gallery

```rust
galleries_update(gallery_id: String, updates: GalleryUpdateData) -> Result<Gallery, Error>
```

**Parameters:**

- `gallery_id: String` - Gallery to update
- `updates: GalleryUpdateData` - Fields to update

**Returns:**

- `Result<Gallery, Error>` - Updated gallery or error

**Location**: `src/backend/src/lib.rs:205-211`

### Delete Gallery

```rust
galleries_delete(gallery_id: String) -> Result<(), Error>
```

**Parameters:**

- `gallery_id: String` - Gallery to delete

**Returns:**

- `Result<(), Error>` - Success or error

**Location**: `src/backend/src/lib.rs:213-219`

---

## Upload & Blob APIs

### Begin Upload

```rust
uploads_begin(
    capsule_id: String,
    total_size: u64,
    chunk_count: u32,
    content_type: String,
    idem: String,
) -> Result<UploadBeginResult, Error>
```

**Parameters:**

- `capsule_id: String` - Target capsule
- `total_size: u64` - Total file size
- `chunk_count: u32` - Number of chunks
- `content_type: String` - MIME type
- `idem: String` - Idempotency key

**Returns:**

- `Result<UploadBeginResult, Error>` - Upload session info

**Location**: `src/backend/src/lib.rs:222-230`

### Upload Chunk

```rust
uploads_chunk(
    pmid: String,
    chunk_index: u32,
    chunk_data: Vec<u8>,
) -> Result<(), Error>
```

**Parameters:**

- `pmid: String` - Upload session ID
- `chunk_index: u32` - Chunk index
- `chunk_data: Vec<u8>` - Chunk data

**Returns:**

- `Result<(), Error>` - Success or error

**Location**: `src/backend/src/lib.rs:232-240`

### Finish Upload

```rust
uploads_finish(pmid: String) -> Result<BlobId, Error>
```

**Parameters:**

- `pmid: String` - Upload session ID

**Returns:**

- `Result<BlobId, Error>` - Blob ID for created blob

**Location**: `src/backend/src/lib.rs:242-250`

### Abort Upload

```rust
uploads_abort(pmid: String) -> Result<(), Error>
```

**Parameters:**

- `pmid: String` - Upload session ID

**Returns:**

- `Result<(), Error>` - Success or error

**Location**: `src/backend/src/lib.rs:252-260`

### Read Blob

```rust
blob_read(locator: String) -> Result<Vec<u8>, Error>
```

**Parameters:**

- `locator: String` - Blob locator

**Returns:**

- `Result<Vec<u8>, Error>` - Blob data

**Location**: `src/backend/src/lib.rs:262-270`

### Read Blob Chunk

```rust
blob_read_chunk(locator: String, chunk_index: u32) -> Result<Vec<u8>, Error>
```

**Parameters:**

- `locator: String` - Blob locator
- `chunk_index: u32` - Chunk index

**Returns:**

- `Result<Vec<u8>, Error>` - Chunk data

**Location**: `src/backend/src/lib.rs:272-280`

### Get Blob Metadata

```rust
blob_get_meta(locator: String) -> Result<BlobMeta, Error>
```

**Parameters:**

- `locator: String` - Blob locator

**Returns:**

- `Result<BlobMeta, Error>` - Blob metadata

**Location**: `src/backend/src/lib.rs:282-290`

### Delete Blob

```rust
blob_delete(locator: String) -> Result<(), Error>
```

**Parameters:**

- `locator: String` - Blob locator

**Returns:**

- `Result<(), Error>` - Success or error

**Location**: `src/backend/src/lib.rs:292-300`

---

## Administrative APIs

### Add Admin

```rust
add_admin(principal: Principal) -> Result<(), Error>
```

**Parameters:**

- `principal: Principal` - Principal to add as admin

**Returns:**

- `Result<(), Error>` - Success or error

**Location**: `src/backend/src/lib.rs:80-82`

### Remove Admin

```rust
remove_admin(principal: Principal) -> Result<(), Error>
```

**Parameters:**

- `principal: Principal` - Principal to remove from admins

**Returns:**

- `Result<(), Error>` - Success or error

**Location**: `src/backend/src/lib.rs:85-87`

### List Admins

```rust
list_admins() -> Vec<Principal>
```

**Returns:**

- `Vec<Principal>` - List of admin principals

**Location**: `src/backend/src/lib.rs:90-92`

### List Super Admins

```rust
list_superadmins() -> Vec<Principal>
```

**Returns:**

- `Vec<Principal>` - List of super admin principals

**Location**: `src/backend/src/lib.rs:95-97`

### Clear All Stable Memory

```rust
clear_all_stable_memory() -> Result<(), Error>
```

**Returns:**

- `Result<(), Error>` - Success or error

**Purpose**: Emergency function to clear all stored data

**Location**: `src/backend/src/lib.rs:1315-1325`

---

## Data Structures

### Core Types

#### Capsule

```rust
pub struct Capsule {
    pub id: String,
    pub subject: PersonRef,
    pub owners: HashMap<PersonRef, OwnerState>,
    pub controllers: HashMap<PersonRef, ControllerState>,
    pub connections: HashMap<PersonRef, Connection>,
    pub connection_groups: HashMap<String, ConnectionGroup>,
    pub memories: HashMap<String, Memory>,
    pub galleries: HashMap<String, Gallery>,
    pub created_at: u64,
    pub updated_at: u64,
    pub bound_to_neon: bool,
    pub inline_bytes_used: u64,
}
```

#### Memory

```rust
pub struct Memory {
    pub id: String,
    pub metadata: MemoryMetadata,
    pub access: MemoryAccess,
    pub inline_assets: Vec<MemoryAssetInline>,
    pub blob_internal_assets: Vec<MemoryAssetBlobInternal>,
    pub blob_external_assets: Vec<MemoryAssetBlobExternal>,
}
```

#### Gallery

```rust
pub struct Gallery {
    pub id: String,
    pub name: String,
    pub description: String,
    pub items: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub created_by: PersonRef,
}
```

### Result Types

#### BulkDeleteResult

```rust
pub struct BulkDeleteResult {
    pub deleted_count: u32,
    pub failed_count: u32,
    pub message: String,
}
```

#### AssetCleanupResult

```rust
pub struct AssetCleanupResult {
    pub memory_id: String,
    pub assets_cleaned: u32,
    pub message: String,
}
```

#### AssetRemovalResult

```rust
pub struct AssetRemovalResult {
    pub memory_id: String,
    pub asset_id: String,
    pub removed: bool,
    pub message: String,
}
```

---

## Error Handling

### Error Types

```rust
pub enum Error {
    Internal(String),           // System errors
    NotFound,                   // Resource not found
    Unauthorized,              // Access denied
    InvalidArgument(String),   // Invalid input
    ResourceExhausted,         // Rate limit exceeded
    NotImplemented(String),    // Feature not available
    Conflict(String),          // Conflicting operation
}
```

### Error Handling Patterns

- **Internal Errors**: System failures, database errors, network issues
- **NotFound Errors**: Resource doesn't exist, invalid ID
- **Unauthorized Errors**: Insufficient permissions, expired session
- **InvalidArgument Errors**: Invalid input data, validation failures
- **ResourceExhausted Errors**: Rate limits exceeded, storage limits reached
- **NotImplemented Errors**: Feature not available, deprecated functionality
- **Conflict Errors**: Conflicting operations, concurrent modifications

---

## Summary

This documentation provides complete coverage of the ICP Backend API for frontend implementation. The system supports:

- **Capsule Management**: Full CRUD operations with proper access control
- **Memory Management**: Creation, deletion, updates, and bulk operations
- **Asset Management**: Granular asset operations and cleanup
- **Gallery Management**: Collection management and organization
- **Upload & Blob Management**: File upload and storage operations
- **Administrative Functions**: System administration and maintenance
- **Error Handling**: Comprehensive error handling patterns and recovery strategies

The API is designed to be intuitive for frontend developers while providing powerful capabilities for managing digital capsules and their associated data.
