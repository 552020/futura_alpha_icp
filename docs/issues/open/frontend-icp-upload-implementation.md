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

### **Current Database (Neon/PostgreSQL) vs ICP Backend**

| Aspect             | Current DB (Neon)    | ICP Backend (Capsule)            |
| ------------------ | -------------------- | -------------------------------- |
| **Structure**      | Centralized tables   | User-specific capsules           |
| **Memory Storage** | `memories` table     | `Memory` struct per capsule      |
| **User Data**      | `users` table        | User capsule with memories       |
| **File Metadata**  | Database records     | Memory struct fields             |
| **Blob Storage**   | External (S3/Vercel) | ICP blob storage (same canister) |
| **Access Pattern** | SQL queries          | Canister calls                   |

### **Memory Data Mapping:**

#### **Database Memory Record:**

**SQL Schema:**

```sql
-- Current database structure
CREATE TABLE memories (
  id UUID PRIMARY KEY,
  owner_id TEXT REFERENCES all_users(id),
  type memory_type_t NOT NULL, -- 'image', 'video', 'note', 'document', 'audio'
  title TEXT,
  description TEXT,
  is_public BOOLEAN DEFAULT false,
  owner_secure_code TEXT NOT NULL,
  parent_folder_id UUID,
  tags TEXT[] DEFAULT '{}',
  recipients TEXT[] DEFAULT '{}',
  file_created_at TIMESTAMP,
  unlock_date TIMESTAMP,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
  deleted_at TIMESTAMP,
  metadata JSON DEFAULT '{}',
  storage_duration INTEGER
);
```

**Drizzle Schema (lines 402-447 in schema.ts):**

```typescript
export const memories = pgTable(
  "memories",
  {
    id: uuid("id").primaryKey().defaultRandom(),
    ownerId: text("owner_id")
      .notNull()
      .references(() => allUsers.id, { onDelete: "cascade" }),
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
  },
  (table) => [
    index("memories_owner_created_idx").on(table.ownerId, table.createdAt.desc()),
    index("memories_type_idx").on(table.type),
    index("memories_public_idx").on(table.isPublic),
    index("memories_tags_idx").on(table.tags),
    index("memories_storage_duration_idx").on(table.storageDuration),
  ]
);
```

#### **Database Memory Assets Record:**

**SQL Schema:**

```sql
-- Memory assets table for multiple optimized versions per memory
CREATE TABLE memory_assets (
  id UUID PRIMARY KEY,
  memory_id UUID REFERENCES memories(id) ON DELETE CASCADE,
  asset_type asset_type_t NOT NULL, -- 'original', 'display', 'thumb', 'placeholder', 'poster', 'waveform'
  variant TEXT,
  url TEXT NOT NULL,
  asset_location blob_hosting_t NOT NULL, -- 's3', 'vercel_blob', 'icp', 'arweave', 'ipfs', 'neon'
  bucket TEXT,
  storage_key TEXT NOT NULL,
  bytes BIGINT NOT NULL,
  width INTEGER,
  height INTEGER,
  mime_type TEXT NOT NULL,
  sha256 TEXT,
  processing_status processing_status_t DEFAULT 'pending' NOT NULL,
  processing_error TEXT,
  deleted_at TIMESTAMP,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

  CONSTRAINT memory_assets_bytes_positive CHECK (bytes > 0),
  CONSTRAINT memory_assets_dimensions_positive CHECK (
    (width IS NULL OR width > 0) AND (height IS NULL OR height > 0)
  )
);

-- Unique constraint: one asset type per memory
CREATE UNIQUE INDEX memory_assets_unique ON memory_assets (memory_id, asset_type);
```

**Drizzle Schema (lines 488-527 in schema.ts):**

```typescript
export const memoryAssets = pgTable(
  "memory_assets",
  {
    id: uuid("id").primaryKey().defaultRandom(),
    memoryId: uuid("memory_id")
      .notNull()
      .references(() => memories.id, { onDelete: "cascade" }),
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
  },
  (table) => [
    uniqueIndex("memory_assets_unique").on(table.memoryId, table.assetType),
    index("memory_assets_memory_idx").on(table.memoryId),
    index("memory_assets_type_idx").on(table.assetType),
    index("memory_assets_url_idx").on(table.url),
    index("memory_assets_storage_idx").on(table.assetLocation, table.storageKey),
    check("memory_assets_bytes_positive", sql`${table.bytes} > 0`),
    check(
      "memory_assets_dimensions_positive",
      sql`(${table.width} IS NULL OR ${table.width} > 0) AND (${table.height} IS NULL OR ${table.height} > 0)`
    ),
  ]
);
```

#### **ICP Memory Struct:**

**Reference**: `src/backend/src/types.rs` (lines 734-741, 481-493)

**Important**: Memories in ICP are always part of a **Capsule** structure. The capsule acts as the "owner" container:

```rust
// ICP backend structure (per capsule)
pub struct Memory {
    pub id: String,                      // unique identifier
    pub info: MemoryInfo,                // basic info (name, type, timestamps)
    pub metadata: MemoryMetadata,        // rich metadata (size, dimensions, etc.)
    pub access: MemoryAccess,            // who can access + temporal rules
    pub data: MemoryData,                // actual data + storage location
    pub parent_folder_id: Option<String>, // folder organization (matches database schema)
    pub idempotency_key: Option<String>, // idempotency key for deduplication
}

// Supporting structs (lines 678-686, 567-576, 617-622, 635-654, 719-731)
pub struct MemoryInfo {
    pub memory_type: MemoryType,    // 'image', 'video', 'note', 'document', 'audio'
    pub name: String,
    pub content_type: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub uploaded_at: u64,
    pub date_of_memory: Option<u64>, // when the actual event happened
}

pub struct MemoryMetadataBase {
    pub size: u64,
    pub mime_type: String,
    pub original_name: String,
    pub uploaded_at: String,
    pub date_of_memory: Option<String>,
    pub people_in_memory: Option<Vec<String>>,
    pub format: Option<String>,
    pub bound_to_neon: bool, // whether linked to Neon database
}

pub enum MemoryMetadata {
    Image(ImageMetadata),
    Video(VideoMetadata),
    Audio(AudioMetadata),
    Document(DocumentMetadata),
    Note(NoteMetadata),
}

pub enum MemoryAccess {
    Public,
    Private,
    Custom { individuals: Vec<PersonRef>, groups: Vec<String> },
    Scheduled { accessible_after: u64, access: Box<MemoryAccess> },
    EventTriggered { trigger_event: AccessEvent, access: Box<MemoryAccess> },
}

pub enum MemoryData {
    Inline { bytes: Vec<u8>, meta: MemoryMeta },
    BlobRef { blob: BlobRef, meta: MemoryMeta },
}

// Capsule structure that contains memories (lines 481-493)
pub struct Capsule {
    pub id: String,                                          // unique capsule identifier
    pub subject: PersonRef,                                  // who this capsule is about
    pub owners: HashMap<PersonRef, OwnerState>,              // 1..n owners (usually 1)
    pub controllers: HashMap<PersonRef, ControllerState>,    // delegated admins (full control)
    pub connections: HashMap<PersonRef, Connection>,         // social graph
    pub connection_groups: HashMap<String, ConnectionGroup>, // organized connection groups
    pub memories: HashMap<String, Memory>,                   // content - THIS IS WHERE MEMORIES LIVE
    pub galleries: HashMap<String, Gallery>,                 // galleries (collections of memories)
    pub created_at: u64,
    pub updated_at: u64,
    pub bound_to_neon: bool,    // Neon database binding status
    pub inline_bytes_used: u64, // Track inline storage consumption
}
```

### **Database Sharing System:**

The database has a comprehensive sharing system with the `memoryShares` table:

```sql
-- Memory sharing table (lines 868-895 in schema.ts)
CREATE TABLE memory_share (
  id TEXT PRIMARY KEY,
  memory_id UUID NOT NULL,           -- References memories.id
  memory_type TEXT NOT NULL,         -- 'image', 'video', 'note', 'document', 'audio'
  owner_id TEXT NOT NULL,            -- References all_users.id (who owns the memory)

  shared_with_type TEXT NOT NULL,    -- 'user', 'group', 'relationship'
  shared_with_id TEXT,               -- For direct user sharing
  group_id TEXT,                     -- For group sharing
  shared_relationship_type TEXT,     -- For relationship-based sharing

  access_level TEXT DEFAULT 'read',  -- 'read', 'write'
  invitee_secure_code TEXT NOT NULL, -- For invitee to access the memory
  invitee_secure_code_created_at TIMESTAMP DEFAULT NOW(),
  created_at TIMESTAMP DEFAULT NOW()
);
```

**Sharing Types:**

1. **Direct User Sharing** - Share with specific users
2. **Group Sharing** - Share with groups of users
3. **Relationship-based Sharing** - Share based on relationships (family, friends, etc.)

**Access Levels:**

- `read` - Can view the memory
- `write` - Can view and modify the memory

### **ICP Access Control System:**

The ICP backend has a sophisticated access control system with the `MemoryAccess` enum:

```rust
// ICP access control (lines 635-661 in types.rs)
pub enum MemoryAccess {
    Public {
        owner_secure_code: String, // secure code for owner access control
    },
    Private {
        owner_secure_code: String, // secure code for owner access control
    },
    Custom {
        individuals: Vec<PersonRef>, // Direct individual access
        groups: Vec<String>,         // Group access (group IDs)
        owner_secure_code: String,   // secure code for owner access control
    },
    Scheduled {
        accessible_after: u64,     // Time-based access
        access: Box<MemoryAccess>, // What it becomes after time
        owner_secure_code: String, // secure code for owner access control
    },
    EventTriggered {
        trigger_event: AccessEvent, // Event-based access
        access: Box<MemoryAccess>,  // What it becomes after event
        owner_secure_code: String, // secure code for owner access control
    },
}
```

**Access Types:**

1. **Public** - Everyone can access
2. **Private** - Only capsule owners
3. **Custom** - Specific individuals and groups
4. **Scheduled** - Time-based access (e.g., reveal after 5 years)
5. **EventTriggered** - Event-based access (e.g., after death, birthday, etc.)

**Key Differences:**

- **ICP**: More sophisticated with time/event-based access, embedded in memory struct
- **Database**: Separate sharing table with audit trails and secure codes
- **ICP**: Access control is part of the memory itself
- **Database**: Sharing is tracked separately with detailed metadata

**Unified Access Control in ICP:**

The ICP `MemoryAccess` system elegantly unifies what the database splits into two systems:

- **Database `recipients`** → **ICP `MemoryAccess::Custom.individuals`**
- **Database `memoryShares`** → **ICP `MemoryAccess` enum variants**

This provides a cleaner, more unified approach to access control.

### **Missing Fields Analysis - ICP vs Database Compatibility:**

#### **Fields Missing in ICP Memory Struct:**

| Database Field      | ICP Equivalent                        | Status        | Notes                         |
| ------------------- | ------------------------------------- | ------------- | ----------------------------- |
| `owner_id`          | ✅ **CAPSULE-BASED**                  | **MAPPED**    | Memories belong to capsules   |
| `parent_folder_id`  | ✅ **ADDED**                          | **MAPPED**    | Folder organization support   |
| `is_public`         | ✅ `MemoryAccess::Public`             | **MAPPED**    | Different access model        |
| `owner_secure_code` | ✅ **ADDED**                          | **MAPPED**    | Secure codes in MemoryAccess  |
| `tags`              | ✅ `NoteMetadata.tags`                | **PARTIAL**   | Only for notes, not all types |
| `recipients`        | ✅ `MemoryAccess::Custom.individuals` | **MAPPED**    | Unified in access control     |
| `sharing`           | ✅ `MemoryAccess::Custom`             | **DIFFERENT** | Different sharing model       |
| `deleted_at`        | ❌ **MISSING**                        | **IMPORTANT** | No soft delete support        |
| `storage_duration`  | ❌ **MISSING**                        | **IMPORTANT** | No TTL support                |

#### **Fields Missing in Database (ICP has but DB doesn't):**

| ICP Field          | Database Equivalent  | Status        | Notes                    |
| ------------------ | -------------------- | ------------- | ------------------------ |
| `memory_type`      | ✅ `type`            | **MAPPED**    | Same concept             |
| `content_type`     | ✅ `mime_type`       | **MAPPED**    | Same concept             |
| `date_of_memory`   | ✅ `file_created_at` | **MAPPED**    | Same concept             |
| `people_in_memory` | ❌ **MISSING**       | **IMPORTANT** | No people tracking in DB |
| `bound_to_neon`    | ❌ **MISSING**       | **IMPORTANT** | No sync status tracking  |
| `idempotency_key`  | ❌ **MISSING**       | **IMPORTANT** | No deduplication support |

#### **Critical Compatibility Issues:**

1. **Owner Management**: ICP uses capsule-based ownership - memories belong to capsules, capsules have owners
2. **Access Control**: Different models - DB uses boolean flags, ICP uses enum-based access
3. **Folder Organization**: ✅ **ADDED** - `parent_folder_id` field added to Memory struct
4. **Soft Deletes**: ICP has no soft delete mechanism
5. **People Tracking**: ICP tracks people in memories, DB doesn't
6. **Sync Status**: ICP tracks Neon binding, DB doesn't track ICP sync
7. **Access Control Architecture**: DB has separate recipients + sharing systems, ICP has unified MemoryAccess system

### **Data Synchronization Strategy:**

1. **Upload to ICP**: Store file in ICP blob storage + create Memory struct
2. **Metadata Sync**: Keep database record for search/indexing
3. **Dual Storage**: File in ICP, metadata in both ICP and database
4. **Fallback**: Database remains source of truth for queries

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
