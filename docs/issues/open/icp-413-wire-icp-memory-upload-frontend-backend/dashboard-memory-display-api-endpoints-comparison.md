# Dashboard Memory Display API Endpoints Comparison

**Priority**: High  
**Type**: Technical Analysis  
**Status**: In Progress - Implementation Complete  
**Created**: 2025-01-16  
**Updated**: 2025-01-16  
**Related**: Database switching functionality, ICP memory integration

## 📋 **Issue Summary**

This document compares the API endpoints and data structures used to fetch memories from the Neon database (Web2) versus the ICP backend (Web3) for dashboard display. It analyzes the current implementation and identifies the required transformations to enable database switching functionality.

## ✅ **Implementation Status - COMPLETED**

**Date**: 2025-01-16  
**Status**: Database switching functionality has been successfully implemented!

### **What Was Implemented:**

1. **✅ Enhanced ICP Backend**: Added pre-computed dashboard fields to `MemoryHeader` in `memories_list`
2. **✅ Database Switching Service**: Updated `fetchMemories()` to accept `dataSource` parameter
3. **✅ ICP Memory Fetching**: Implemented `fetchMemoriesFromICP()` with proper data transformation
4. **✅ Frontend Integration**: Connected database toggle in dashboard to actual switching logic
5. **✅ Data Transformation**: Created `transformICPMemoryHeaderToNeon()` for format compatibility

### **Current Implementation:**

```typescript
// Updated fetchMemories function in src/nextjs/src/services/memories.ts
export const fetchMemories = async (
  page: number,
  dataSource: "neon" | "icp" = "neon"
): Promise<FetchMemoriesResult> => {
  if (dataSource === "icp") {
    return await fetchMemoriesFromICP(page);
  } else {
    return await fetchMemoriesFromNeon(page);
  }
};
```

### **Dashboard Integration:**

```typescript
// Dashboard component with database switching
const [dataSource, setDataSource] = useState<"neon" | "icp">("neon");

const { data } = useInfiniteQuery({
  queryKey: qk.memories.dashboard(userId, params.lang as string, dataSource),
  queryFn: ({ pageParam = 1 }) => fetchMemories(pageParam as number, dataSource),
  // ... other options
});
```

### **UI Toggle:**

The database toggle switch in the dashboard top bar now controls the actual data source:

- **Neon**: Fetches from `/api/memories` (existing Web2 API)
- **ICP**: Fetches directly from ICP canister using `memories_list`

## 🎯 **Current State Analysis**

### **Neon Database API Endpoint**

#### **Endpoint**: `GET /api/memories`

**File**: `src/nextjs/src/app/api/memories/get.ts` (lines 30-248)

**Request Parameters**:

```typescript
interface RequestParams {
  page?: number; // Page number for pagination (default: 1)
  limit?: number; // Items per page (default: 12)
  type?: string; // Filter by memory type ('image', 'video', 'note', 'document', 'audio')
  includeAssets?: boolean; // Include full asset information (default: false)
  optimized?: boolean; // Use optimized query with galleries (default: false)
}
```

**Response Structure**:

```typescript
interface NeonApiResponse {
  success: boolean;
  data: MemoryWithFolder[]; // Array of memory objects
  hasMore: boolean; // Pagination flag (currently always false)
  total: number; // Total count of memories
}
```

**Memory Object Structure (Neon)**:

```typescript
interface MemoryWithFolder {
  // Core identification
  id: string;
  ownerId: string;

  // Memory metadata
  type: "image" | "video" | "note" | "document" | "audio";
  title: string | null;
  description: string | null;
  isPublic: boolean;

  // Organization
  parentFolderId: string | null;
  tags: string[];
  recipients: string[];

  // Timestamps
  createdAt: string; // ISO string
  updatedAt: string; // ISO string
  fileCreatedAt: string | null;
  unlockDate: string | null;
  deletedAt: string | null;

  // Storage information
  storageDuration: number | null;
  metadata: {
    originalPath?: string;
    custom?: Record<string, unknown>;
  };

  // Sharing information (computed)
  status: "public" | "shared" | "private";
  sharedWithCount: number;

  // Folder information (if in folder)
  folder?: {
    id: string;
    name: string;
  };

  // Assets (if includeAssets=true)
  assets?: Array<{
    id: string;
    assetType: "original" | "display" | "thumb" | "placeholder" | "poster" | "waveform";
    variant: string | null;
    url: string;
    assetLocation: "s3" | "vercel_blob" | "icp" | "arweave" | "ipfs" | "neon";
    bucket: string | null;
    storageKey: string;
    bytes: number;
    width: number | null;
    height: number | null;
    mimeType: string;
    sha256: string | null;
    processingStatus: "pending" | "processing" | "completed" | "failed";
    processingError: string | null;
    deletedAt: string | null;
    createdAt: string;
    updatedAt: string;
  }>;

  // Legacy fields for UI compatibility
  thumbnail?: string; // Pre-computed thumbnail URL
  url?: string; // Primary asset URL
}
```

### **ICP Backend API Endpoints**

#### **Endpoint 1**: `memories_list` (Query)

**File**: `src/backend/src/lib.rs` (lines 440-487)

**Request Parameters**:

```rust
fn memories_list(
    capsule_id: String,        // User's capsule ID
    cursor: Option<String>,    // Pagination cursor
    limit: Option<u32>,        // Items per page (default: 50, max: 100)
) -> Result<Page<MemoryHeader>, Error>
```

**Response Structure**:

```rust
struct Page<MemoryHeader> {
    items: Vec<MemoryHeader>,      // Array of memory headers
    next_cursor: Option<String>,   // Pagination cursor for next page
}

struct MemoryHeader {
    id: String,
    name: String,                  // Memory name/title
    memory_type: MemoryType,       // 'Note' | 'Image' | 'Document' | 'Audio' | 'Video'
    size: u64,                     // Total size in bytes
    created_at: u64,               // Nanoseconds since Unix epoch
    updated_at: u64,               // Nanoseconds since Unix epoch
    access: MemoryAccess,          // Access control information
}
```

#### **Endpoint 2**: `memories_read` (Query)

**File**: `src/backend/src/lib.rs` (lines 344-354)

**Request Parameters**:

```rust
fn memories_read(
    memory_id: String,             // Specific memory ID
) -> Result<Memory, Error>
```

**Response Structure**:

```rust
struct Memory {
    id: String,
    metadata: MemoryMetadata,      // Rich metadata
    access: MemoryAccess,          // Access control
    inline_assets: Vec<MemoryAssetInline>,           // Inline assets
    blob_internal_assets: Vec<MemoryAssetBlobInternal>, // ICP blob assets
    blob_external_assets: Vec<MemoryAssetBlobExternal>, // External blob assets
}

struct MemoryMetadata {
    // Basic info
    memory_type: MemoryType,
    title: Option<String>,
    description: Option<String>,
    content_type: String,

    // Timestamps (nanoseconds since Unix epoch)
    created_at: u64,
    updated_at: u64,
    uploaded_at: u64,
    date_of_memory: Option<u64>,
    file_created_at: Option<u64>,

    // Organization
    parent_folder_id: Option<String>,
    tags: Vec<String>,
    deleted_at: Option<u64>,

    // Content info
    people_in_memory: Option<Vec<String>>,
    location: Option<String>,
    memory_notes: Option<String>,

    // System info
    created_by: Option<String>,
    database_storage_edges: Vec<StorageEdgeDatabaseType>,
}
```

## 🔄 **Data Structure Comparison**

### **Key Differences**

| Aspect             | Neon Database                         | ICP Backend                           |
| ------------------ | ------------------------------------- | ------------------------------------- |
| **Pagination**     | `page`/`limit` with `hasMore`         | `cursor`/`limit` with `next_cursor`   |
| **Timestamps**     | ISO strings                           | Nanoseconds since Unix epoch          |
| **Memory Types**   | String literals                       | Rust enums with variants              |
| **Assets**         | Single `assets` array                 | Three separate arrays by storage type |
| **Access Control** | Boolean `isPublic` + `recipients`     | Complex `MemoryAccess` enum           |
| **Folder Info**    | Embedded `folder` object              | `parent_folder_id` only               |
| **Storage Info**   | `assetLocation` per asset             | `database_storage_edges` array        |
| **Sharing**        | Computed `status` + `sharedWithCount` | Embedded in `MemoryAccess`            |

### **Memory Type Mapping**

| Neon Type    | ICP Type             | Notes          |
| ------------ | -------------------- | -------------- |
| `'image'`    | `{ Image: null }`    | Image memories |
| `'video'`    | `{ Video: null }`    | Video memories |
| `'note'`     | `{ Note: null }`     | Text notes     |
| `'document'` | `{ Document: null }` | PDFs, docs     |
| `'audio'`    | `{ Audio: null }`    | Audio files    |

### **Asset Structure Mapping**

| Neon Asset                 | ICP Asset                                             | Notes           |
| -------------------------- | ----------------------------------------------------- | --------------- |
| `assetType: 'original'`    | `inline_assets[0]` or `blob_internal_assets[0]`       | Primary asset   |
| `assetType: 'thumb'`       | `blob_internal_assets` with `asset_type: 'Thumbnail'` | Thumbnail       |
| `assetType: 'display'`     | `blob_internal_assets` with `asset_type: 'Preview'`   | Display version |
| `assetType: 'placeholder'` | `blob_internal_assets` with `asset_type: 'Metadata'`  | Placeholder     |

## 🎯 **Required Transformations**

### **1. ICP to Neon Format Transformation**

```typescript
// Transform ICP Memory to Neon MemoryWithFolder format
function transformICPMemoryToNeonFormat(icpMemory: ICPMemory): MemoryWithFolder {
  return {
    // Core identification
    id: icpMemory.id,
    ownerId: "icp-user", // ICP users don't have ownerId in same way

    // Memory metadata
    type: mapICPMemoryTypeToNeon(icpMemory.metadata.memory_type),
    title: icpMemory.metadata.title || "Untitled",
    description: icpMemory.metadata.description || null,
    isPublic: isMemoryPublic(icpMemory.access),

    // Organization
    parentFolderId: icpMemory.metadata.parent_folder_id || null,
    tags: icpMemory.metadata.tags || [],
    recipients: extractRecipientsFromAccess(icpMemory.access),

    // Timestamps (convert nanoseconds to ISO strings)
    createdAt: new Date(Number(icpMemory.metadata.created_at / 1000000n)).toISOString(),
    updatedAt: new Date(Number(icpMemory.metadata.updated_at / 1000000n)).toISOString(),
    fileCreatedAt: icpMemory.metadata.file_created_at
      ? new Date(Number(icpMemory.metadata.file_created_at / 1000000n)).toISOString()
      : null,
    unlockDate: null, // ICP doesn't have unlock dates
    deletedAt: icpMemory.metadata.deleted_at
      ? new Date(Number(icpMemory.metadata.deleted_at / 1000000n)).toISOString()
      : null,

    // Storage information
    storageDuration: null, // ICP doesn't have TTL
    metadata: {
      originalPath: null,
      custom: {},
    },

    // Sharing information (computed from access)
    status: computeSharingStatus(icpMemory.access),
    sharedWithCount: countSharedRecipients(icpMemory.access),

    // Folder information
    folder: icpMemory.metadata.parent_folder_id
      ? {
          id: icpMemory.metadata.parent_folder_id,
          name: "ICP Folder", // TODO: Get actual folder name
        }
      : undefined,

    // Assets (combine all asset types)
    assets: combineICPAssets(icpMemory),

    // Legacy fields
    thumbnail: getThumbnailUrl(icpMemory),
    url: getPrimaryAssetUrl(icpMemory),
  };
}

// Helper functions
function mapICPMemoryTypeToNeon(icpType: ICPMemoryType): NeonMemoryType {
  switch (icpType) {
    case { Image: null }:
      return "image";
    case { Video: null }:
      return "video";
    case { Note: null }:
      return "note";
    case { Document: null }:
      return "document";
    case { Audio: null }:
      return "audio";
    default:
      return "document";
  }
}

function combineICPAssets(icpMemory: ICPMemory): NeonAsset[] {
  const assets: NeonAsset[] = [];

  // Add inline assets
  icpMemory.inline_assets.forEach((asset, index) => {
    assets.push({
      id: asset.asset_id,
      assetType: mapICPAssetTypeToNeon(asset.metadata.asset_type),
      variant: null,
      url: `icp://memory/${icpMemory.id}/inline/${index}`,
      assetLocation: "icp",
      bucket: null,
      storageKey: asset.asset_id,
      bytes: asset.bytes.length,
      width: asset.metadata.width || null,
      height: asset.metadata.height || null,
      mimeType: asset.metadata.mime_type,
      sha256: asset.metadata.sha256 || null,
      processingStatus: "completed",
      processingError: null,
      deletedAt: null,
      createdAt: new Date(Number(icpMemory.metadata.created_at / 1000000n)).toISOString(),
      updatedAt: new Date(Number(icpMemory.metadata.updated_at / 1000000n)).toISOString(),
    });
  });

  // Add blob internal assets
  icpMemory.blob_internal_assets.forEach((asset) => {
    assets.push({
      id: asset.asset_id,
      assetType: mapICPAssetTypeToNeon(asset.metadata.asset_type),
      variant: null,
      url: `icp://memory/${icpMemory.id}/blob/${asset.asset_id}`,
      assetLocation: "icp",
      bucket: null,
      storageKey: asset.blob_ref.locator,
      bytes: asset.blob_ref.len,
      width: asset.metadata.width || null,
      height: asset.metadata.height || null,
      mimeType: asset.metadata.mime_type,
      sha256: asset.blob_ref.hash ? Buffer.from(asset.blob_ref.hash).toString("hex") : null,
      processingStatus: "completed",
      processingError: null,
      deletedAt: null,
      createdAt: new Date(Number(icpMemory.metadata.created_at / 1000000n)).toISOString(),
      updatedAt: new Date(Number(icpMemory.metadata.updated_at / 1000000n)).toISOString(),
    });
  });

  // Add blob external assets
  icpMemory.blob_external_assets.forEach((asset) => {
    assets.push({
      id: asset.asset_id,
      assetType: mapICPAssetTypeToNeon(asset.metadata.asset_type),
      variant: null,
      url: asset.url || `external://${asset.location}/${asset.storage_key}`,
      assetLocation: mapICPStorageTypeToNeon(asset.location),
      bucket: null,
      storageKey: asset.storage_key,
      bytes: asset.metadata.bytes,
      width: asset.metadata.width || null,
      height: asset.metadata.height || null,
      mimeType: asset.metadata.mime_type,
      sha256: asset.metadata.sha256 || null,
      processingStatus: "completed",
      processingError: null,
      deletedAt: null,
      createdAt: new Date(Number(icpMemory.metadata.created_at / 1000000n)).toISOString(),
      updatedAt: new Date(Number(icpMemory.metadata.updated_at / 1000000n)).toISOString(),
    });
  });

  return assets;
}
```

### **2. Pagination Handling**

```typescript
// ICP uses cursor-based pagination, Neon uses page-based
function handleICPPagination(
  icpPage: Page<MemoryHeader>,
  currentPage: number
): { memories: MemoryWithFolder[]; hasMore: boolean } {
  const memories = icpPage.items.map((header) => transformICPMemoryHeaderToNeon(header));

  return {
    memories,
    hasMore: icpPage.next_cursor !== null,
  };
}
```

### **3. Authentication Differences**

```typescript
// Neon: Uses NextAuth session
const session = await auth();
if (!session?.user?.id) {
  return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
}

// ICP: Uses Internet Identity principal
const caller = PersonRef::from_caller();
if !capsule.has_read_access(&caller) {
  return Err(Error::Unauthorized);
}
```

## 🚀 **Implementation Strategy**

### **Phase 1: Create ICP Memory Fetching Service**

```typescript
// File: src/nextjs/src/services/memories-icp.ts
export const fetchMemoriesFromICP = async (page: number, limit: number = 12): Promise<FetchMemoriesResult> => {
  const { getActor } = await import("@/ic/backend");
  const actor = await getActor();

  // Get user's capsule ID
  const capsuleId = await getUserCapsuleId(actor);

  // Calculate cursor from page
  const cursor = page > 1 ? ((page - 1) * limit).toString() : undefined;

  // Call ICP canister
  const result = await actor.memories_list(capsuleId, cursor, limit);

  if ("Ok" in result) {
    const icpPage = result.Ok;

    // Transform ICP memories to Neon format
    const memories = await Promise.all(
      icpPage.items.map(async (header) => {
        // Get full memory details
        const memoryResult = await actor.memories_read(header.id);
        if ("Ok" in memoryResult) {
          return transformICPMemoryToNeonFormat(memoryResult.Ok);
        } else {
          // Fallback to header-only data
          return transformICPMemoryHeaderToNeon(header);
        }
      })
    );

    return {
      memories,
      hasMore: icpPage.next_cursor !== null,
    };
  } else {
    throw new Error(`ICP canister error: ${result.Err}`);
  }
};
```

### **Phase 2: Update Frontend Service**

```typescript
// File: src/nextjs/src/services/memories.ts
export const fetchMemories = async (
  page: number,
  dataSource: "neon" | "icp" = "neon"
): Promise<FetchMemoriesResult> => {
  if (dataSource === "icp") {
    return await fetchMemoriesFromICP(page);
  } else {
    // Existing Neon implementation
    const response = await fetch(`/api/memories?page=${page}`);
    // ... existing logic
  }
};
```

### **Phase 3: Handle Missing ICP Endpoints**

**Current Gap**: ICP backend doesn't have a direct equivalent to Neon's `/api/memories` endpoint.

**Required ICP Endpoints**:

```rust
// Add to src/backend/src/lib.rs
#[ic_cdk::query]
fn get_user_memories_dashboard(
    page: u32,
    limit: u32,
) -> Result<DashboardMemoriesResponse, Error> {
    // Implementation that returns memories in dashboard-friendly format
    // Similar to Neon API response structure
}

struct DashboardMemoriesResponse {
    success: bool,
    data: Vec<MemoryWithFolderInfo>,
    has_more: bool,
    total: u32,
}

struct MemoryWithFolderInfo {
    // Combined Memory + folder information
    // Optimized for dashboard display
}
```

## 📊 **Performance Considerations**

### **Neon Database**

- ✅ **Fast queries** - Direct SQL queries with indexes
- ✅ **Efficient pagination** - LIMIT/OFFSET with proper indexing
- ✅ **Asset URLs** - Pre-computed presigned URLs
- ❌ **Network latency** - HTTP requests to external database

### **ICP Backend**

- ✅ **Low latency** - Direct canister calls
- ✅ **Decentralized** - No single point of failure
- ❌ **Slower queries** - Canister query limitations
- ❌ **Complex pagination** - Cursor-based with manual implementation
- ❌ **Asset access** - Requires separate calls for each asset

### **Optimization Strategies**

1. **Batch Asset Loading**: Load multiple memories in parallel
2. **Caching**: Cache transformed data to avoid repeated transformations
3. **Lazy Loading**: Load full memory details only when needed
4. **Asset Optimization**: Pre-compute thumbnail URLs where possible

## 🔧 **Error Handling**

### **Neon Database Errors**

```typescript
// Standard HTTP error responses
{
  error: string;
  status: number;
  details?: Record<string, unknown>;
}
```

### **ICP Backend Errors**

```rust
// Candid Result types
enum Error {
  NotFound,
  Unauthorized,
  InvalidInput(String),
  InternalError(String),
}
```

### **Unified Error Handling**

```typescript
function handleMemoryFetchError(error: unknown): never {
  if (error instanceof Error) {
    // HTTP errors from Neon
    throw error;
  } else if (typeof error === "object" && error !== null && "Err" in error) {
    // ICP canister errors
    throw new Error(`ICP Error: ${error.Err}`);
  } else {
    throw new Error("Unknown error occurred");
  }
}
```

## 📋 **Implementation Checklist**

### **Phase 1: Core Transformation (High Priority) - ✅ COMPLETED**

- [x] Create `transformICPMemoryToNeonFormat()` function
- [x] Create `mapICPMemoryTypeToNeon()` helper
- [x] Create `combineICPAssets()` function (simplified for MemoryHeader)
- [x] Create `computeSharingStatus()` function (using pre-computed fields)
- [x] Test transformation with sample ICP data

### **Phase 2: ICP Service Integration (High Priority) - ✅ COMPLETED**

- [x] Create `fetchMemoriesFromICP()` service function
- [x] Implement cursor-based pagination handling
- [x] Add error handling for ICP canister calls
- [x] Test with real ICP canister

### **Phase 3: Frontend Integration (Medium Priority) - ✅ COMPLETED**

- [x] Update `fetchMemories()` to accept `dataSource` parameter
- [x] Connect database toggle to service function
- [x] Add loading states for ICP fetching (via React Query)
- [x] Test database switching functionality

### **Phase 4: Optimization (Low Priority) - 🔄 IN PROGRESS**

- [x] Implement caching for transformed data (React Query handles this)
- [ ] Add batch loading for multiple memories
- [ ] Optimize asset URL generation
- [ ] Add performance monitoring

## 🎯 **Success Criteria**

### **Functional Requirements - ✅ ACHIEVED**

- [x] Users can switch between Neon and ICP database views
- [x] ICP memories display correctly in dashboard format
- [x] Folder grouping works for ICP memories (via `parent_folder_id`)
- [x] Asset URLs are accessible and functional (pre-computed in MemoryHeader)
- [x] Pagination works correctly for both data sources

### **Performance Requirements - ✅ ACHIEVED**

- [x] ICP memory loading completes within 5 seconds (using pre-computed fields)
- [x] Database switching is responsive (< 2 seconds) (React Query caching)
- [x] Memory transformation doesn't block UI (async transformation)
- [x] Error handling provides clear user feedback (try/catch with user-friendly messages)

### **Compatibility Requirements - ✅ ACHIEVED**

- [x] Existing Neon functionality remains unchanged
- [x] Dashboard components work with both data sources (unified MemoryWithFolder format)
- [x] Asset loading works for both storage types (pre-computed URLs in MemoryHeader)
- [x] Folder navigation works for both systems (parent_folder_id mapping)

---

**Last Updated**: 2025-01-16  
**Status**: ✅ COMPLETED - Database switching functionality implemented  
**Priority**: High - Core functionality for database switching feature

## 🎉 **Implementation Summary**

The database switching functionality has been successfully implemented! Users can now:

1. **Toggle between databases** using the switch in the dashboard top bar
2. **View ICP memories** in the same dashboard format as Neon memories
3. **Experience seamless switching** with React Query caching for performance
4. **Access all memory features** including folders, tags, and sharing status

The implementation leverages the pre-computed dashboard fields we added to the ICP backend, ensuring fast query performance and compatibility with the existing dashboard UI.
