# Memory Structure Comparison: ICP Backend vs Web2 Database

## Overview

This document compares the Memory data structures between the ICP backend (Rust/Candid) and the Web2 database (PostgreSQL/Drizzle) to identify compatibility issues and design a sync strategy for MVP.

## Current State Analysis

### ICP Backend Memory Structure

```rust
pub struct Memory {
    pub id: String,                      // unique identifier
    pub info: MemoryInfo,                // basic info (name, type, timestamps)
    pub metadata: MemoryMetadata,        // rich metadata (size, dimensions, etc.)
    pub access: MemoryAccess,            // who can access + temporal rules
    pub data: MemoryData,                // actual data + storage location
    pub idempotency_key: Option<String>, // idempotency key for deduplication
}

pub struct MemoryInfo {
    pub name: String,
    pub memory_type: MemoryType,
    pub content_type: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub uploaded_at: u64,
    pub date_of_memory: Option<u64>,
}

pub enum MemoryMetadata {
    Image(ImageMetadata),      // + dimensions
    Video(VideoMetadata),      // + duration, width, height, thumbnail
    Audio(AudioMetadata),      // + duration, format, bitrate, sample_rate, channels
    Document(DocumentMetadata), // base only
    Note(NoteMetadata),        // + tags
}

pub enum MemoryData {
    Inline { bytes: Vec<u8>, meta: MemoryMeta },
    BlobRef { blob: BlobRef, meta: MemoryMeta },
}
```

### Web2 Database Memory Structure

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
  tags: text("tags").array().default([]),
  recipients: text("recipients").array().default([]),
  fileCreatedAt: timestamp("file_created_at", { mode: "date" }),
  unlockDate: timestamp("unlock_date", { mode: "date" }),
  createdAt: timestamp("created_at").notNull().defaultNow(),
  updatedAt: timestamp("updated_at").notNull().defaultNow(),
  deletedAt: timestamp("deleted_at"),
  metadata: json("metadata")
    .$type<{
      originalPath?: string;
      custom?: Record<string, unknown>;
    }>()
    .default({}),
  storageDuration: integer("storage_duration"),
});

export const memoryAssets = pgTable("memory_assets", {
  id: uuid("id").primaryKey().defaultRandom(),
  memoryId: uuid("memory_id").notNull(),
  assetType: asset_type_t("asset_type").notNull(),
  variant: text("variant"),
  url: text("url").notNull(),
  assetLocation: blob_hosting_t("asset_location").notNull(),
  bucket: text("bucket"),
  storageKey: text("storage_key").notNull(),
  bytes: bigint("bytes", { mode: "number" }).notNull(),
  width: integer("width"),
  height: integer("height"),
  mimeType: text("mime_type").notNull(),
  sha256: text("sha256"),
  processingStatus: processing_status_t("processing_status").default("pending").notNull(),
  processingError: text("processing_error"),
  deletedAt: timestamp("deleted_at"),
  createdAt: timestamp("created_at").notNull().defaultNow(),
  updatedAt: timestamp("updated_at").notNull().defaultNow(),
});
```

## Field Mapping Analysis

### ✅ **Directly Compatible Fields**

| ICP Backend          | Web2 Database          | Notes                                |
| -------------------- | ---------------------- | ------------------------------------ |
| `id`                 | `id`                   | Both use string identifiers          |
| `info.name`          | `title`                | **Naming difference** - need mapping |
| `info.memory_type`   | `type`                 | Both use enum types                  |
| `info.created_at`    | `createdAt`            | Both timestamps                      |
| `info.updated_at`    | `updatedAt`            | Both timestamps                      |
| `info.content_type`  | `mimeType` (in assets) | **Location difference**              |
| `metadata.base.size` | `bytes` (in assets)    | **Location difference**              |

### ⚠️ **Partially Compatible Fields**

| ICP Backend                      | Web2 Database           | Compatibility Issues                                  |
| -------------------------------- | ----------------------- | ----------------------------------------------------- |
| `info.date_of_memory`            | `fileCreatedAt`         | ICP uses u64, DB uses timestamp                       |
| `metadata.base.original_name`    | `metadata.originalPath` | **Location difference**                               |
| `metadata.base.people_in_memory` | `recipients`            | **Location difference**                               |
| `access`                         | `isPublic`              | ICP has complex access control, DB has simple boolean |
| `metadata.base.tags`             | `tags`                  | **Location difference**                               |

### ❌ **Missing/Incompatible Fields**

| ICP Backend        | Web2 Database | Issue                   |
| ------------------ | ------------- | ----------------------- |
| `info.uploaded_at` | ❌ Missing    | No equivalent in DB     |
| `info.description` | `description` | **Location difference** |
| `idempotency_key`  | ❌ Missing    | No equivalent in DB     |
| `ownerSecureCode`  | ❌ Missing    | No equivalent in ICP    |
| `parentFolderId`   | ❌ Missing    | No equivalent in ICP    |
| `storageDuration`  | ❌ Missing    | No equivalent in ICP    |
| `deletedAt`        | ❌ Missing    | No equivalent in ICP    |

### 🔄 **Asset Storage Differences**

| ICP Backend         | Web2 Database        | Issue                                |
| ------------------- | -------------------- | ------------------------------------ |
| `data.Inline.bytes` | ❌ No inline storage | ICP stores small files inline        |
| `data.BlobRef.blob` | `memoryAssets` table | **Completely different structure**   |
| Single data field   | Multiple asset types | DB has original/display/thumb assets |

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
