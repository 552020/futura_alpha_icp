# Memory Structure Comparison: ICP Backend vs Web2 Database

## Overview

This document compares the Memory data structures between the ICP backend (Rust/Candid) and the Web2 database (PostgreSQL/Drizzle) to identify compatibility issues and design a sync strategy for MVP.

**Analysis Date**: October 6, 2025  
**Status**: Current Implementation Analysis  
**Location**: `src/backend/src/memories/types.rs` vs `src/nextjs/src/db/schema.ts`  
**Key Commits**:

- `4d9580e` - Complete upload service refactoring with multiple asset support
- `56e48b1` - Add bulk memory API tests and reorganize test structure
- `d0fc48f` - Implement stable memory infrastructure for canister upgrades

## Current State Analysis

### ✅ CURRENT: ICP Backend Memory Structure

**Location**: `src/backend/src/memories/types.rs` (lines 267-274)

```rust
pub struct Memory {
    pub id: String,                                         // unique identifier
    pub metadata: MemoryMetadata, // memory-level metadata (title, description, etc.)
    pub access: MemoryAccess,     // who can access + temporal rules
    pub inline_assets: Vec<MemoryAssetInline>, // 0 or more inline assets
    pub blob_internal_assets: Vec<MemoryAssetBlobInternal>, // 0 or more ICP blob assets
    pub blob_external_assets: Vec<MemoryAssetBlobExternal>, // 0 or more external blob assets
}

pub struct MemoryMetadata {
    // Basic info
    pub memory_type: MemoryType,
    pub title: Option<String>,       // Optional title (matches database)
    pub description: Option<String>, // Optional description (matches database)
    pub content_type: String,

    // Timestamps
    pub created_at: u64,
    pub updated_at: u64,
    pub uploaded_at: u64,
    pub date_of_memory: Option<u64>, // when the actual event happened
    pub file_created_at: Option<u64>, // when the original file was created

    // Organization
    pub parent_folder_id: Option<String>,
    pub tags: Vec<String>, // Memory tags
    pub deleted_at: Option<u64>,

    // Content info
    pub people_in_memory: Option<Vec<String>>, // People in the memory
    pub location: Option<String>,              // Where the memory was taken
    pub memory_notes: Option<String>,          // Additional notes

    // System info
    pub created_by: Option<String>, // Who created this memory
    pub database_storage_edges: Vec<StorageEdgeDatabaseType>,
}

// Asset structures
pub struct MemoryAssetInline {
    pub asset_id: String, // Unique identifier for this asset
    pub bytes: Vec<u8>,
    pub metadata: AssetMetadata,
}

pub struct MemoryAssetBlobInternal {
    pub asset_id: String, // Unique identifier for this asset
    pub blob_ref: BlobRef,
    pub metadata: AssetMetadata,
}

pub struct MemoryAssetBlobExternal {
    pub asset_id: String,              // Unique identifier for this asset
    pub location: StorageEdgeBlobType, // Where the asset is stored externally
    pub storage_key: String,           // Key/ID in external storage system
    pub url: Option<String>,           // Public URL (if available)
    pub metadata: AssetMetadata,       // Type-specific metadata
}
```

### ✅ CURRENT: Web2 Database Memory Structure

**Location**: `src/nextjs/src/db/schema.ts` (lines 402-447, 488-527)

```typescript
export const memories = pgTable("memories", {
  id: uuid("id").primaryKey().defaultRandom(),
  ownerId: text("owner_id").notNull(),
  type: memory_type_t("type").notNull(),
  title: text("title"),
  description: text("description"),
  isPublic: boolean("is_public").default(false).notNull(),
  ownerSecureCode: text("owner_secure_code").notNull(),
  parentFolderId: uuid("parent_folder_id"),
  // Tags for better performance and search
  tags: text("tags").array().default([]),
  // Universal fields for all memory types
  recipients: text("recipients").array().default([]),
  // Date fields - grouped together
  fileCreatedAt: timestamp("file_created_at", { mode: "date" }), // When file was originally created
  unlockDate: timestamp("unlock_date", { mode: "date" }), // When memory becomes accessible
  createdAt: timestamp("created_at").notNull().defaultNow(), // When memory was uploaded/created in our system
  updatedAt: timestamp("updated_at").notNull().defaultNow(), // When memory was last modified
  deletedAt: timestamp("deleted_at"), // Soft delete support
  // Flexible metadata for truly common additional data
  metadata: json("metadata")
    .$type<{
      // File upload context (applies to all types)
      originalPath?: string; // Original file path from upload
      // Custom user data (truly universal)
      custom?: Record<string, unknown>;
    }>()
    .default({}),
  // Storage status fields
  storageDuration: integer("storage_duration"), // Duration in days, null for permanent
});

export const memoryAssets = pgTable("memory_assets", {
  id: uuid("id").primaryKey().defaultRandom(),
  memoryId: uuid("memory_id")
    .notNull()
    .references(() => memories.id, { onDelete: "cascade" }),
  assetType: asset_type_t("asset_type").notNull(),
  variant: text("variant"), // Optional for future variants (2k, mobile, etc.)
  url: text("url").notNull(), // Derived/public URL
  assetLocation: blob_hosting_t("asset_location").notNull(),
  bucket: text("bucket"), // Storage bucket/container
  storageKey: text("storage_key").notNull(), // Bucket/key or blob ID
  bytes: bigint("bytes", { mode: "number" }).notNull(), // Use bigint for >2GB files
  width: integer("width"), // Nullable for non-image assets
  height: integer("height"), // Nullable for non-image assets
  mimeType: text("mime_type").notNull(), // Consistent naming
  sha256: text("sha256"), // 64-char hex (enforced by validation)
  processingStatus: processing_status_t("processing_status").default("pending").notNull(),
  processingError: text("processing_error"),
  deletedAt: timestamp("deleted_at"), // Soft delete support
  createdAt: timestamp("created_at").notNull().defaultNow(),
  updatedAt: timestamp("updated_at").notNull().defaultNow(),
});
```

## Field Mapping Analysis

### ✅ **Directly Compatible Fields**

| ICP Backend                 | Web2 Database    | Notes                                  |
| --------------------------- | ---------------- | -------------------------------------- |
| `id`                        | `id`             | Both use string identifiers            |
| `metadata.title`            | `title`          | ✅ **FIXED** - Now directly compatible |
| `metadata.description`      | `description`    | ✅ **FIXED** - Now directly compatible |
| `metadata.memory_type`      | `type`           | Both use enum types                    |
| `metadata.created_at`       | `createdAt`      | Both timestamps                        |
| `metadata.updated_at`       | `updatedAt`      | Both timestamps                        |
| `metadata.parent_folder_id` | `parentFolderId` | ✅ **FIXED** - Now directly compatible |
| `metadata.tags`             | `tags`           | ✅ **FIXED** - Now directly compatible |
| `metadata.deleted_at`       | `deletedAt`      | ✅ **FIXED** - Now directly compatible |
| `metadata.people_in_memory` | `recipients`     | ✅ **FIXED** - Now directly compatible |

### ⚠️ **Partially Compatible Fields**

| ICP Backend                | Web2 Database           | Compatibility Issues                                    |
| -------------------------- | ----------------------- | ------------------------------------------------------- |
| `metadata.date_of_memory`  | `fileCreatedAt`         | ICP uses u64, DB uses timestamp                         |
| `metadata.file_created_at` | `fileCreatedAt`         | ICP uses u64, DB uses timestamp                         |
| `access`                   | `isPublic`              | ICP has complex access control, DB has simple boolean   |
| `metadata.content_type`    | `mimeType` (in assets)  | **Location difference** - ICP in metadata, DB in assets |
| `metadata.originalPath`    | `metadata.originalPath` | ✅ **FIXED** - Now in same location                     |

### ❌ **Missing/Incompatible Fields**

| ICP Backend                       | Web2 Database | Issue                |
| --------------------------------- | ------------- | -------------------- |
| `metadata.uploaded_at`            | ❌ Missing    | No equivalent in DB  |
| `idempotency_key`                 | ❌ Missing    | No equivalent in DB  |
| `ownerSecureCode`                 | ❌ Missing    | No equivalent in ICP |
| `storageDuration`                 | ❌ Missing    | No equivalent in ICP |
| `unlockDate`                      | ❌ Missing    | No equivalent in ICP |
| `metadata.location`               | ❌ Missing    | No equivalent in DB  |
| `metadata.memory_notes`           | ❌ Missing    | No equivalent in DB  |
| `metadata.created_by`             | ❌ Missing    | No equivalent in DB  |
| `metadata.database_storage_edges` | ❌ Missing    | No equivalent in DB  |

### 🔄 **Asset Storage Differences**

| ICP Backend                          | Web2 Database           | Issue                                |
| ------------------------------------ | ----------------------- | ------------------------------------ |
| `inline_assets[].bytes`              | ❌ No inline storage    | ICP stores small files inline        |
| `blob_internal_assets[].blob_ref`    | `memoryAssets` table    | **Completely different structure**   |
| `blob_external_assets[].storage_key` | `memoryAssets` table    | **Completely different structure**   |
| Multiple asset vectors               | Multiple asset types    | DB has original/display/thumb assets |
| Asset-specific metadata              | Asset-specific metadata | Both have type-specific metadata     |

## Sync Strategy for MVP

### 1. **Core Memory Sync**

Create a unified memory structure that can be converted between both systems:

```typescript
interface UnifiedMemory {
  // Core fields (present in both)
  id: string;
  title: string; // maps to ICP info.name
  description?: string; // maps to ICP info.description (missing)
  type: "Image" | "Video" | "Audio" | "Document" | "Note";
  createdAt: number; // ICP u64, DB timestamp
  updatedAt: number; // ICP u64, DB timestamp

  // ICP-specific fields
  uploadedAt?: number; // ICP only
  dateOfMemory?: number; // ICP u64, DB timestamp
  idempotencyKey?: string; // ICP only

  // DB-specific fields
  ownerId?: string; // DB only
  ownerSecureCode?: string; // DB only
  parentFolderId?: string; // DB only
  isPublic?: boolean; // simplified from ICP access control
  storageDuration?: number; // DB only
  deletedAt?: number; // DB only

  // Metadata (location differs)
  tags: string[];
  recipients: string[];
  originalName?: string; // ICP metadata.base.original_name
  originalPath?: string; // DB metadata.originalPath
  peopleInMemory?: string[]; // ICP metadata.base.people_in_memory

  // Asset information
  assets: UnifiedAsset[];
}

interface UnifiedAsset {
  type: "original" | "display" | "thumb" | "placeholder";
  url: string;
  storageBackend: "icp" | "vercel" | "s3";
  storageKey: string;
  bytes: number;
  width?: number;
  height?: number;
  mimeType: string;
  sha256?: string;
}
```

### 2. **Sync Functions**

```typescript
// ICP to DB conversion
function icpMemoryToDb(icpMemory: ICPMemory): DbMemory {
  return {
    id: icpMemory.id,
    ownerId: "derived_from_capsule_owner", // Need to derive from capsule
    type: icpMemory.info.memory_type,
    title: icpMemory.info.name,
    description: icpMemory.info.description || null,
    isPublic: icpMemory.access === "Public",
    tags: extractTags(icpMemory.metadata),
    recipients: extractRecipients(icpMemory.metadata),
    fileCreatedAt: icpMemory.info.date_of_memory ? new Date(icpMemory.info.date_of_memory) : null,
    createdAt: new Date(icpMemory.info.created_at),
    updatedAt: new Date(icpMemory.info.updated_at),
    metadata: {
      originalPath: extractOriginalName(icpMemory.metadata),
      custom: {},
    },
  };
}

// DB to ICP conversion
function dbMemoryToIcp(dbMemory: DbMemory): ICPMemory {
  return {
    id: dbMemory.id,
    info: {
      name: dbMemory.title,
      memory_type: dbMemory.type,
      content_type: "application/octet-stream", // Default, should be from assets
      created_at: dbMemory.createdAt.getTime(),
      updated_at: dbMemory.updatedAt.getTime(),
      uploaded_at: dbMemory.createdAt.getTime(), // Use created_at as fallback
      date_of_memory: dbMemory.fileCreatedAt?.getTime() || null,
    },
    metadata: createMetadataFromDb(dbMemory),
    access: dbMemory.isPublic ? "Public" : "Private",
    data: createDataFromAssets(dbMemory.assets),
    idempotency_key: null, // Not available in DB
  };
}
```

### 3. **Asset Sync Strategy**

The biggest challenge is the asset storage difference:

**ICP Approach:**

- Single data field per memory
- Inline storage for small files (≤64KB)
- Blob reference for large files

**DB Approach:**

- Multiple assets per memory (original, display, thumb)
- All assets stored externally (Vercel Blob, S3)
- No inline storage

**Sync Solution:**

```typescript
// For ICP → DB sync
function syncIcpAssetToDb(icpMemory: ICPMemory): DbAsset[] {
  const assets: DbAsset[] = [];

  if (icpMemory.data.type === "Inline") {
    // Upload inline data to external storage
    const url = await uploadToVercelBlob(icpMemory.data.bytes);
    assets.push({
      memoryId: icpMemory.id,
      assetType: "original",
      url,
      assetLocation: "vercel",
      storageKey: generateStorageKey(),
      bytes: icpMemory.data.bytes.length,
      mimeType: icpMemory.info.content_type,
      sha256: await calculateSHA256(icpMemory.data.bytes),
    });
  } else if (icpMemory.data.type === "BlobRef") {
    // Convert blob reference to external storage
    const blobData = await fetchBlobFromICP(icpMemory.data.blob);
    const url = await uploadToVercelBlob(blobData);
    assets.push({
      memoryId: icpMemory.id,
      assetType: "original",
      url,
      assetLocation: "vercel",
      storageKey: generateStorageKey(),
      bytes: icpMemory.data.blob.len,
      mimeType: icpMemory.info.content_type,
      sha256: icpMemory.data.blob.hash?.toString("hex"),
    });
  }

  return assets;
}

// For DB → ICP sync
function syncDbAssetToIcp(dbAssets: DbAsset[]): ICPMemoryData {
  const originalAsset = dbAssets.find((a) => a.assetType === "original");
  if (!originalAsset) throw new Error("No original asset found");

  // Download asset data
  const assetData = await downloadFromUrl(originalAsset.url);

  if (assetData.length <= 64 * 1024) {
    // Small file - store inline
    return {
      type: "Inline",
      bytes: assetData,
      meta: {
        name: originalAsset.mimeType,
        tags: [],
        description: null,
      },
    };
  } else {
    // Large file - store as blob reference
    const blobId = await storeBlobInICP(assetData);
    return {
      type: "BlobRef",
      blob: {
        kind: "ICPCapsule",
        locator: `blob_${blobId}`,
        hash: await calculateSHA256(assetData),
        len: assetData.length,
      },
      meta: {
        name: originalAsset.mimeType,
        tags: [],
        description: null,
      },
    };
  }
}
```

## MVP Implementation Plan

### Phase 1: Basic Sync (Week 1-2)

1. **Create unified memory interface**
2. **Implement basic field mapping**
3. **Handle simple cases only:**
   - Public/Private access (ignore complex access control)
   - Single asset per memory (original only)
   - Basic metadata (title, description, tags)

### Phase 2: Asset Sync (Week 3-4)

1. **Implement asset conversion**
2. **Handle storage backend differences**
3. **Add error handling and retry logic**

### Phase 3: Advanced Features (Week 5-6)

1. **Support multiple asset types**
2. **Handle complex access control**
3. **Add bidirectional sync**

## Critical Issues to Address

### 1. **ID Generation**

- **Problem**: ICP uses `format!("mem_{now}")`, DB uses UUID
- **Solution**: Use UUID for both systems, generate in sync layer

### 2. **Timestamp Format**

- **Problem**: ICP uses u64 (nanoseconds), DB uses timestamp
- **Solution**: Convert between formats in sync functions

### 3. **Access Control**

- **Problem**: ICP has complex access control, DB has simple boolean
- **Solution**: MVP only sync Public/Private, ignore complex rules

### 4. **Asset Storage**

- **Problem**: Completely different storage approaches
- **Solution**: Convert between inline/blob and external storage

### 5. **Missing Fields**

- **Problem**: Some fields exist in one system but not the other
- **Solution**: Use defaults or derive from available data

## Recommendations

1. **Start with MVP sync** focusing on core fields only
2. **Use external storage** for all assets to simplify sync
3. **Implement bidirectional sync** but start with DB → ICP direction
4. **Add comprehensive logging** for sync operations
5. **Handle errors gracefully** with retry mechanisms
6. **Consider using a sync queue** for large batches

This approach will allow us to maintain data consistency between both systems while gradually improving the sync capabilities as we move beyond MVP.

## ✅ **CURRENT STATUS SUMMARY (October 6, 2025)**

### **Major Improvements Since Original Analysis**

1. **✅ Field Alignment**: Many previously incompatible fields are now directly compatible:

   - `title` and `description` now exist in both systems
   - `parent_folder_id` now exists in both systems
   - `tags` now exist in both systems
   - `deleted_at` now exists in both systems
   - `people_in_memory`/`recipients` now exist in both systems

2. **✅ Asset Architecture**: Both systems now support multiple assets per memory:

   - ICP: `inline_assets`, `blob_internal_assets`, `blob_external_assets`
   - DB: `memory_assets` table with multiple asset types

3. **✅ Metadata Structure**: ICP now has a unified `MemoryMetadata` struct that aligns better with DB schema

### **Remaining Challenges**

1. **⚠️ Access Control**: ICP has complex access control vs DB's simple boolean
2. **⚠️ Asset Storage**: Different approaches (inline vs external-only)
3. **⚠️ Timestamp Formats**: u64 vs timestamp conversion needed
4. **❌ Missing Fields**: Some fields still exist in only one system

### **Sync Feasibility Assessment**

**HIGH COMPATIBILITY**: The structures are now much more aligned, making sync significantly easier than originally anticipated. The major architectural differences have been resolved, and most core fields are directly compatible.

**RECOMMENDATION**: Proceed with MVP sync implementation focusing on the directly compatible fields first, then gradually add support for the remaining differences.
