# Gallery Type Refactor - Schema Normalization

**Status**: `OPEN` - Analysis Required  
**Priority**: `MEDIUM` - Architecture Improvement  
**Assigned**: Backend Developer + Frontend Developer  
**Created**: 2024-12-19  
**Related Issues**: [Name/Title Semantics Standardization](./name-title-semantics-standardization.md)

## Problem Description

The current Gallery type system lacks consistency with the Memory system's `name`/`title` semantics, has incomplete storage location tracking, and **CRITICALLY** uses an oversimplified `isPublic` boolean that doesn't match our sophisticated sharing system.

### Current State

Our gallery types have **inconsistent architecture** compared to the memory pattern, leading to:

#### 🚨 **CRITICAL: Oversimplified Access Control**

**Current Problem**: Galleries use a simple `isPublic: boolean` field, but our system has a **sophisticated multi-layered sharing architecture**:

**Database Sharing System** (`galleryShares` table):

- ✅ **Direct User Sharing**: Share with specific users
- ✅ **Group Sharing**: Share with user groups
- ✅ **Relationship-based Sharing**: Share with family/friends based on relationships
- ✅ **Access Levels**: `read` | `write` permissions
- ✅ **Secure Access Codes**: For invitee authentication
- ✅ **Audit Trail**: Creation timestamps, secure code tracking

**ICP Backend Sharing System** (`MemoryAccess` enum):

- ✅ **Public/Private**: Basic access control
- ✅ **Custom Access**: Specific individuals and groups
- ✅ **Scheduled Access**: Time-based revelation (e.g., reveal after 5 years)
- ✅ **Event-triggered Access**: After death, birthdays, anniversaries, etc.
- ✅ **Secure Codes**: Owner access control

**Gallery Access Modes** (from documentation):

- ✅ **Private**: Only explicitly shared users
- ✅ **Public-auth**: Any logged-in user can view
- ✅ **Public-link**: Anyone with URL, no login required
- ✅ **Customer Groups**: Wedding couples, family members, etc.
- ✅ **Role-based Permissions**: Owner, Customer Admin, Customer Member, Guest

**The Problem**: `isPublic: boolean` cannot represent this complexity!

#### 🏗️ **Architectural Pattern: Separate Sharing Tables**

**Memories Pattern** (what we should follow):

```typescript
// memories table - basic fields only
export const memories = pgTable("memories", {
  id: uuid("id").primaryKey().defaultRandom(),
  ownerId: text("owner_id").notNull(),
  title: text("title"),
  isPublic: boolean("is_public").default(false).notNull(), // ✅ Simple boolean for backward compatibility
  // ... other basic fields
});

// memoryShares table - complex sharing logic
export const memoryShares = pgTable("memory_share", {
  memoryId: uuid("memory_id").notNull(),
  sharedWithType: text("shared_with_type", { enum: ["user", "group", "relationship"] }),
  accessLevel: text("access_level", { enum: ["read", "write"] }),
  // ... complex sharing fields
});
```

**Galleries Pattern** (should be identical):

```typescript
// galleries table - basic fields only
export const galleries = pgTable("galleries", {
  id: text("id").primaryKey(),
  ownerId: text("owner_id").notNull(),
  title: text("title").notNull(),
  // ❌ REMOVE: isPublic: boolean('is_public').default(false).notNull(),
  // ✅ NO SHARING FIELDS IN MAIN TABLE - use galleryShares table instead
});

// galleryShares table - complex sharing logic (already exists!)
export const galleryShares = pgTable("gallery_share", {
  galleryId: text("gallery_id").notNull(),
  sharedWithType: text("shared_with_type", { enum: ["user", "group", "relationship"] }),
  accessLevel: text("access_level", { enum: ["read", "write"] }),
  // ... same complex sharing fields as memoryShares
});
```

**Key Insight**: The main table (`memories`/`galleries`) should **NOT** contain sharing logic - that belongs in the separate `*Shares` table!

#### 🚀 **Performance Consideration: Pre-computed vs Real-time Fields**

**Your Question**: For gallery tiles showing "public/private/shared" status, should we:

1. **Make 2 API calls** (gallery + galleryShares)?
2. **Pre-compute and store** sharing status in gallery table?
3. **Single JOIN query** to get both?

**Answer**: **Pre-compute and store** (Option 2) - Here's why:

**Current Memory Pattern** (what we should follow):

```rust
// MemoryMetadata has pre-computed dashboard fields
pub struct MemoryMetadata {
    // ... basic fields ...

    // ✅ PRE-COMPUTED: Dashboard-specific fields (no extra queries needed!)
    pub is_public: bool,                   // Computed from access rules
    pub shared_count: u32,                 // Count of shared recipients
    pub sharing_status: String,            // "public" | "shared" | "private"
    pub total_size: u64,                   // Sum of all asset sizes
    pub asset_count: u32,                  // Total number of assets
    pub thumbnail_url: Option<String>,     // Pre-computed thumbnail URL
    pub primary_asset_url: Option<String>, // Primary asset URL for display
    pub has_thumbnails: bool,              // Whether thumbnails exist
    pub has_previews: bool,                // Whether previews exist
}
```

**Why Pre-compute is Better**:

1. **Performance**: Single query instead of N+1 queries
2. **Cost**: No extra HTTP requests to database
3. **Consistency**: Same pattern as memories (already implemented)
4. **Scalability**: Gallery listings are fast regardless of sharing complexity
5. **User Experience**: Instant gallery tiles without loading states

**Gallery Should Follow Same Pattern**:

```typescript
// galleries table - add pre-computed fields
export const galleries = pgTable("galleries", {
  id: text("id").primaryKey(),
  ownerId: text("owner_id").notNull(),
  title: text("title").notNull(),
  name: text("name").notNull(), // ✅ ADD IT: URL-safe identifier

  // ✅ PRE-COMPUTED: Dashboard fields (like memories)
  // ❌ REMOVED: is_public: boolean("is_public").default(false).notNull(), // Redundant with sharing_status
  shared_count: integer("shared_count").default(0).notNull(), // Count of active shares
  sharing_status: text("sharing_status").default("private").notNull(), // "public" | "shared" | "private"
  total_memories: integer("total_memories").default(0).notNull(), // Count of memories
  storage_location: blob_hosting_t("storage_location").notNull(), // Where gallery is stored

  createdAt: timestamp("created_at").defaultNow().notNull(),
  updatedAt: timestamp("updated_at").defaultNow().notNull(),
});
```

**Update Strategy**: Recompute these fields when:

- Gallery sharing changes (add/remove shares)
- Memories added/removed from gallery
- Gallery metadata changes

#### 🏗️ **Database Architecture Context**

**Your HTTP Request Concern**: You're absolutely right! In our serverless architecture:

```
Frontend (Next.js) → API Route → Database (Neon PostgreSQL)
     ↓                    ↓              ↓
  Client-side        Serverless      External DB
  (Browser)          (Vercel)        (HTTP requests)
```

**Each database query = HTTP request** to Neon PostgreSQL, so:

❌ **Bad**: Gallery listing + separate sharing queries

```typescript
// This would be 1 + N HTTP requests (expensive!)
const galleries = await db.query.galleries.findMany(); // 1 request
for (const gallery of galleries) {
  const shares = await db.query.galleryShares.findMany({
    // N requests!
    where: eq(galleryShares.galleryId, gallery.id),
  });
}
```

✅ **Good**: Pre-computed fields in single query

```typescript
// This is just 1 HTTP request (fast!)
const galleries = await db.query.galleries.findMany(); // 1 request
// All sharing info already included in gallery.is_public, gallery.shared_count, etc.
```

**Best Practice**: Pre-compute expensive-to-calculate fields during writes, not reads!

1. **Mixed Concerns**: Metadata fields scattered in main struct instead of dedicated metadata struct
2. **Inconsistent Naming**: `name` field is just a copy of `title` (same redundancy as MemoryHeader)
3. **No URL-safe Identifier**: No proper name generation for URLs
4. **Schema Mismatch**: Backend types don't match the database schema from `schema.ts`

### Current Gallery Types (Backend)

```rust
// From src/backend/src/types.rs
pub struct Gallery {
    pub id: String,
    pub title: String,                    // ❌ PROBLEM: Should be in metadata
    pub description: Option<String>,      // ❌ PROBLEM: Should be in metadata
    pub is_public: bool,                  // ❌ PROBLEM: Should be in metadata
    pub storage_location: GalleryStorageLocation,
    pub memory_entries: Vec<GalleryMemoryEntry>,
    // ... other fields
}

pub struct GalleryHeader {
    pub id: String,
    pub name: String,                     // ❌ PROBLEM: Just a copy of title
    pub memory_count: u64,
    // ... other fields
}

// From src/backend/src/gallery.rs line 486
impl Gallery {
    pub fn to_header(&self) -> GalleryHeader {
        GalleryHeader {
            name: self.title.clone(),     // ❌ Same redundancy as MemoryHeader
            // ... other fields
        }
    }
}
```

### Database Schema (Frontend)

```typescript
// From src/nextjs/src/db/schema.ts
// ✅ UPDATED SCHEMA - FOLLOWS MEMORY PATTERN WITH PRE-COMPUTED FIELDS
export const galleries = pgTable("gallery", {
  id: text("id")
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID()),
  ownerId: text("owner_id")
    .notNull()
    .references(() => allUsers.id, { onDelete: "cascade" }),
  title: text("title").notNull(), // ✅ EXISTS: User-facing title
  name: text("name").notNull(), // ✅ ADDED: URL-safe identifier (auto-generated)
  description: text("description"),

  // ✅ PRE-COMPUTED: Dashboard fields (like memories)
  // ❌ REMOVED: is_public: boolean("is_public").default(false).notNull(), // Redundant with sharing_status
  shared_count: integer("shared_count").default(0).notNull(), // Count of active shares
  sharing_status: text("sharing_status").default("private").notNull(), // "public" | "shared" | "private"
  total_memories: integer("total_memories").default(0).notNull(), // Count of memories
  storage_location: jsonb("storage_location").$type<BlobHosting[]>().default(["s3"]).notNull(), // ✅ MULTIPLE VALUES: Where gallery is stored

  createdAt: timestamp("created_at").defaultNow().notNull(),
  updatedAt: timestamp("updated_at").defaultNow().notNull(),

  // ❌ REMOVED: averageStorageDuration: integer("average_storage_duration"), // Not needed
  // ❌ REMOVED: storageDistribution: json("storage_distribution").$type<Record<string, number>>().default({}), // Legacy field - not needed in greenfield
});

// ✅ TYPE INFERENCE - Drizzle handles this automatically
export type DBGallery = typeof galleries.$inferSelect;
export type NewDBGallery = typeof galleries.$inferInsert;
```

#### 🎯 **Managing Type Complexity**

**Your Concern**: "Is the type don't get messy this way?"

**Answer**: No, if we follow good patterns! Here's how to keep types clean:

**✅ Good Pattern - Layered Types**:

```typescript
// 1. Base database type (auto-generated by Drizzle)
export type DBGallery = typeof galleries.$inferSelect;

// 2. Frontend display type (subset for UI)
export interface GalleryTile {
  id: string;
  title: string;
  name: string;
  is_public: boolean;
  shared_count: number;
  sharing_status: "public" | "shared" | "private";
  total_memories: number;
  storage_location: string;
  createdAt: Date;
}

// 3. API response type (with computed fields)
export interface GalleryListResponse {
  galleries: GalleryTile[];
  hasMore: boolean;
  totalCount: number;
}

// 4. Utility function to convert DB → UI
export function dbGalleryToTile(dbGallery: DBGallery): GalleryTile {
  return {
    id: dbGallery.id,
    title: dbGallery.title,
    name: dbGallery.name,
    is_public: dbGallery.is_public,
    shared_count: dbGallery.shared_count,
    sharing_status: dbGallery.sharing_status as "public" | "shared" | "private",
    total_memories: dbGallery.total_memories,
    storage_location: dbGallery.storage_location,
    createdAt: dbGallery.createdAt,
  };
}
```

**✅ Benefits of This Pattern**:

1. **Type Safety**: Drizzle auto-generates DB types
2. **Separation of Concerns**: DB types vs UI types vs API types
3. **Flexibility**: Easy to add/remove fields without breaking everything
4. **Performance**: Only fetch what you need for each use case
5. **Maintainability**: Clear boundaries between layers

**❌ Bad Pattern - Monolithic Types**:

```typescript
// DON'T DO THIS - one giant type for everything
export interface GalleryEverything {
  // ... 50+ fields including internal DB fields, computed fields, UI fields, etc.
}
```

```typescript
// ✅ SIMPLIFIED: gallery_item is just a JOIN/relationship table
export const galleryItems = pgTable("gallery_item", {
  id: text("id")
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID()),
  galleryId: text("gallery_id")
    .notNull()
    .references(() => galleries.id, { onDelete: "cascade" }),
  memoryId: uuid("memory_id").notNull(),
  memoryType: text("memory_type", { enum: MEMORY_TYPES }).notNull(),
  position: integer("position").notNull(),
  caption: text("caption"),
  isFeatured: boolean("is_featured").default(false).notNull(),
  metadata: json("metadata").$type<Record<string, unknown>>().notNull().default({}),
  // ❌ REMOVE: createdAt: timestamp("created_at").defaultNow().notNull(), // Gallery items aren't "created" - memories are created
  // ❌ REMOVE: updatedAt: timestamp("updated_at").defaultNow().notNull(), // Gallery items aren't "updated" - they're just relationships
});
```

#### 🎯 **Understanding Gallery Items: Relationship Table, Not Entity**

**Your Insight**: "createdAt means added to the gallery, cause a gallery item is not 'created' a memory is created."

**You're absolutely right!** `gallery_item` is a **relationship/join table**, not a standalone entity:

**✅ What Gallery Items Actually Are**:

```typescript
// gallery_item = Many-to-Many relationship between galleries and memories
// It's like a "bookmark" or "pin" - not a thing that gets "created"
interface GalleryItemRelationship {
  galleryId: string; // Which gallery
  memoryId: string; // Which memory
  position: number; // Order in gallery
  caption?: string; // Gallery-specific caption (different from memory description)
  isFeatured: boolean; // Gallery-specific feature flag
  metadata: object; // Gallery-specific metadata
}
```

**❌ What Gallery Items Are NOT**:

- ❌ **Not a standalone entity** that gets "created"
- ❌ **Not a copy of the memory** - it's just a reference
- ❌ **Not something that needs timestamps** - the memory has its own timestamps

**✅ Correct Semantics**:

- **Memory**: Gets "created" when user uploads → has `createdAt`
- **Gallery**: Gets "created" when user creates collection → has `createdAt`
- **Gallery Item**: Gets "added" to gallery → no timestamps needed

**Real-world Analogy**:

- **Memory** = Photo (has creation date)
- **Gallery** = Photo album (has creation date)
- **Gallery Item** = Photo placed in album (no creation date - just "when was it added?")

**If we need "when added to gallery"**:

```typescript
// Better field name if we need this info
addedToGalleryAt: timestamp("added_to_gallery_at").defaultNow().notNull(),
// But even this is questionable - do we really need it?
```

```typescript
// ❌ CURRENT: galleryShares table is TOO SIMPLE for wedding gallery requirements
export const galleryShares = pgTable("gallery_share", {
  id: text("id")
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID()),
  galleryId: text("gallery_id")
    .notNull()
    .references(() => galleries.id, { onDelete: "cascade" }),
  ownerId: text("owner_id")
    .notNull()
    .references(() => allUsers.id, { onDelete: "cascade" }),
  sharedWithType: text("shared_with_type", {
    enum: ["user", "group", "relationship"],
  }).notNull(),
  sharedWithId: text("shared_with_id").references(() => allUsers.id, { onDelete: "cascade" }),
  groupId: text("group_id").references(() => group.id, { onDelete: "cascade" }),
  sharedRelationshipType: text("shared_relationship_type", {
    enum: SHARING_RELATIONSHIP_TYPES,
  }),
  accessLevel: text("access_level", { enum: ACCESS_LEVELS }).default("read").notNull(),
  inviteeSecureCode: text("invitee_secure_code").notNull(),
  inviteeSecureCodeCreatedAt: timestamp("secure_code_created_at", { mode: "date" }).notNull().defaultNow(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});
```

#### ✅ **Gallery Sharing System - COMPLETED**

**The gallery sharing system has been fully implemented** with a universal resource sharing architecture.

**See completed implementation**:

- [Universal Resource Sharing System](./done-resource-sharing-table-unification.md) - **COMPLETED**
- [Gallery Sharing Table Enhancement](./done-gallery-sharing-table-enhancement.md) - **COMPLETED**

**✅ All Issues Resolved**:

- ✅ Universal role-based permissions (owner, superadmin, admin, member, guest)
- ✅ Bitmask permissions with 5 core permission types (VIEW, DOWNLOAD, SHARE, MANAGE, OWN)
- ✅ Magic links with TTL, use limits, and consumption tracking
- ✅ Public access modes (private, public-auth, public-link) as first-class grants
- ✅ Universal sharing system works for galleries, memories, and folders

````

## Analysis

### Schema Mismatch Issues

1. **Missing Fields**: Backend types don't include all database fields
2. **Type Inconsistency**: Backend uses `String` for IDs, database uses `text` with UUID generation
3. **Missing Relations**: Backend doesn't have proper relations to `galleryItems`
4. **Storage Fields**: Database has storage status fields not reflected in backend types
5. **❌ MISSING NAME FIELD**: Database schema is missing the `name` field for URL-safe identifiers
6. **❌ MISSING STORAGE LOCATION FIELD**: Database schema is missing the `storageLocation` field
7. **❌ UNNECESSARY FIELD**: `averageStorageDuration` field exists but is not needed

### Architecture Inconsistency

**Memory Pattern (Good):**

```rust
pub struct Memory {
    pub id: String,
    pub metadata: MemoryMetadata,  // ✅ Dedicated metadata struct
    pub access: MemoryAccess,
    pub inline_assets: Vec<MemoryAssetInline>,
    pub blob_internal_assets: Vec<MemoryAssetBlobInternal>,
    pub blob_external_assets: Vec<MemoryAssetBlobExternal>,
}

pub struct MemoryMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub content_type: String,
    // ... other metadata fields
}
````

**Gallery Pattern (Bad):**

```rust
pub struct Gallery {
    pub id: String,
    pub title: String,                    // ❌ Direct field instead of metadata
    pub description: Option<String>,      // ❌ Direct field instead of metadata
    pub is_public: bool,                  // ❌ Direct field instead of metadata
    // ... other fields
}
```

## 🎯 **Proposed Solution**

### 1. **🚨 CRITICAL: Replace isPublic with Complex Access Control**

**Database Schema** (`src/nextjs/src/db/schema.ts`):

```typescript
// ❌ REMOVE: isPublic: boolean("is_public").default(false).notNull(),
// ✅ COMPLEX SHARING: Uses galleryShares table + access control system (SAME AS MEMORIES!)
// - Private: No entries in galleryShares table
// - Public-auth: Special entry with sharedWithType: 'public_auth'
// - Public-link: Special entry with sharedWithType: 'public_link'
// - Custom: Entries in galleryShares table with specific users/groups/relationships

// ✅ ARCHITECTURAL CONSISTENCY: Memories follow the SAME pattern:
// - memories table: has isPublic boolean (for backward compatibility)
// - memoryShares table: handles complex sharing logic
// - Galleries should follow identical pattern
```

**ICP Backend Types** (`src/backend/src/types.rs`):

```rust
// ✅ ADD IT: GalleryAccess enum (mirrors MemoryAccess)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum GalleryAccess {
    Public {
        owner_secure_code: String,
    },
    Private {
        owner_secure_code: String,
    },
    Custom {
        individuals: Vec<PersonRef>,
        groups: Vec<String>,
        owner_secure_code: String,
    },
    Scheduled {
        accessible_after: u64,
        access: Box<GalleryAccess>,
        owner_secure_code: String,
    },
    EventTriggered {
        trigger_event: AccessEvent,
        access: Box<GalleryAccess>,
        owner_secure_code: String,
    },
}

// ✅ UPDATE: Gallery struct (follows Memory pattern)
pub struct Gallery {
    pub id: String,
    pub capsule_id: String,
    pub metadata: GalleryMetadata,
    // ❌ REMOVE: pub access: GalleryAccess, // Complex access belongs in shares, not main struct
    pub memories: Vec<String>, // Memory IDs
    pub shares: Vec<GalleryShare>, // ✅ Complex sharing logic here (like Memory)
    pub created_at: u64,
    pub updated_at: u64,
}

// ✅ ARCHITECTURAL CONSISTENCY: Memory struct follows same pattern:
// - Memory struct: basic fields + shares vector
// - Complex access control: handled via shares, not embedded in main struct
// - Gallery should follow identical pattern
```

### 2. **Standardize Name/Title Semantics**

### **Normalized Gallery Types (Consistent with Memory Pattern)**

```rust
// New consistent structure
pub struct Gallery {
    pub id: String,
    pub owner_id: String,                 // ✅ From database schema
    pub metadata: GalleryMetadata,        // ✅ Consistent with Memory pattern
    pub storage_location: GalleryStorageLocation,
    pub memory_entries: Vec<GalleryMemoryEntry>,
    pub shares: Vec<GalleryShare>,        // ✅ Gallery sharing support
    pub created_at: u64,                  // ✅ From database schema
    pub updated_at: u64,                  // ✅ From database schema
}

pub struct GalleryMetadata {
    pub title: String,                    // ✅ User-facing title
    pub name: String,                     // ✅ ADD IT: URL-safe identifier (auto-generated)
    pub description: Option<String>,      // ✅ User-facing description
    // ❌ SIMPLIFIED: pub is_public: bool,                  // ✅ Access control
    // ✅ COMPLEX SHARING: Uses GalleryAccess enum (like MemoryAccess)
    pub total_memories: u32,              // ✅ From database schema
    pub storage_location: BlobHosting,    // ✅ ADD IT: SAME AS MEMORY ASSETS
    pub storage_distribution: std::collections::HashMap<String, u32>, // ✅ From database schema
}

pub struct GalleryHeader {
    pub id: String,
    pub title: String,                    // ✅ What user sees
    pub name: String,                     // ✅ ADD IT: URL-safe identifier (auto-generated)
    pub memory_count: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub storage_location: BlobHosting,    // ✅ ADD IT: SAME AS MEMORY ASSETS
    // ❌ SIMPLIFIED: pub is_public: bool,
    // ✅ COMPLEX SHARING: pub access: GalleryAccess, // Like MemoryAccess enum
    pub total_memories: u32,
    pub share_count: u32,                 // ✅ Number of active shares
}

// New implementation
impl Gallery {
    pub fn to_header(&self) -> GalleryHeader {
        let title = self.metadata.title.clone();
        let name = self.metadata.name.clone();  // ✅ Use stored name

        GalleryHeader {
            id: self.id.clone(),
            title,
            name,                          // ✅ URL-safe name from metadata
            memory_count: self.memory_entries.len() as u64,
            created_at: self.created_at,
            updated_at: self.updated_at,
            storage_location: self.metadata.storage_location.clone(),
            is_public: self.metadata.is_public,
            total_memories: self.metadata.total_memories,
            share_count: self.shares.len() as u32,  // ✅ Count active shares
        }
    }
}
```

### **Gallery Item Types (From Database Schema)**

```rust
pub struct GalleryItem {
    pub id: String,
    pub gallery_id: String,
    pub memory_id: String,                // ✅ UUID from database
    pub memory_type: MemoryType,          // ✅ Enum from database
    pub position: u32,
    pub caption: Option<String>,
    pub is_featured: bool,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
    pub created_at: u64,
    pub updated_at: u64,
}

pub struct GalleryItemHeader {
    pub id: String,
    pub memory_id: String,
    pub memory_type: MemoryType,
    pub position: u32,
    pub caption: Option<String>,
    pub is_featured: bool,
    pub created_at: u64,
    pub updated_at: u64,
}
```

### **Gallery Sharing Types (From Database Schema)**

```rust
pub struct GalleryShare {
    pub id: String,
    pub gallery_id: String,
    pub owner_id: String,
    pub shared_with_type: SharedWithType,
    pub shared_with_id: Option<String>,           // For direct user sharing
    pub group_id: Option<String>,                 // For group sharing
    pub shared_relationship_type: Option<SharingRelationshipType>, // For relationship-based sharing
    pub access_level: AccessLevel,
    pub invitee_secure_code: String,
    pub invitee_secure_code_created_at: u64,
    pub created_at: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum SharedWithType {
    User,
    Group,
    Relationship,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum AccessLevel {
    Read,
    Write,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum SharingRelationshipType {
    CloseFamily,
    Family,
    Partner,
    CloseFriend,
    Friend,
    Colleague,
    Acquaintance,
}
```

### **Storage Location Enum (Normalized)**

```rust
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum GalleryStorageLocation {
    Web2Only,    // Only in Neon database
    ICPOnly,     // Only in ICP canister
    Both,        // In both systems
    Migrating,   // Currently being migrated
    Failed,      // Migration failed
}
```

## Implementation Plan

### **Phase 1: Backend Type Refactoring**

#### 1. Update Gallery Types

**File**: `src/backend/src/types.rs`

```rust
// Replace existing Gallery struct with normalized version
pub struct Gallery {
    pub id: String,
    pub capsule_id: String,               // ✅ SAME AS MEMORY: Capsule context
    pub owner_principal: Principal,       // ✅ SAME AS CURRENT: ICP Principal (not String)
    pub metadata: GalleryMetadata,        // ✅ Consistent with Memory pattern
    pub storage_location: GalleryStorageLocation,
    pub memory_entries: Vec<GalleryMemoryEntry>,
    pub created_at: u64,                  // ✅ From database schema
    pub updated_at: u64,                  // ✅ From database schema
}

pub struct GalleryMetadata {
    pub title: Option<String>,            // ✅ User-editable title (optional - if None, use name)
    pub name: String,                     // ✅ URL-safe identifier (auto-generated, never empty)
    pub description: Option<String>,      // ✅ User-facing description (from schema.ts)

    // ✅ PRE-COMPUTED: Dashboard fields (from schema.ts)
    // ❌ REMOVED: pub is_public: bool,                  // Redundant with sharing_status
    pub shared_count: u32,                // Count of active shares
    pub sharing_status: SharingStatus,    // ✅ ENUM: "public" | "shared" | "private"
    pub total_memories: u32,              // Count of memories

    // ✅ COMPUTED: Storage location (computed from memory storage_edges)
    pub storage_location: Vec<BlobHosting>, // ✅ COMPUTED: Where gallery memories are stored
    // ❌ REMOVED: pub storage_distribution: std::collections::HashMap<String, u32>, // Legacy field - not needed in greenfield
}

pub struct GalleryHeader {
    pub id: String,
    pub title: Option<String>,            // ✅ User-editable title (optional - if None, use name)
    pub name: String,                     // ✅ URL-safe identifier (auto-generated, never empty)
    pub memory_count: u64,                // ✅ Count of memories in gallery
    pub created_at: u64,                  // ✅ From database schema
    pub updated_at: u64,                  // ✅ From database schema

    // ✅ PRE-COMPUTED: Dashboard fields (from schema.ts)
    // ❌ REMOVED: pub is_public: bool,                  // Redundant with sharing_status
    pub shared_count: u32,                // Count of active shares
    pub sharing_status: SharingStatus,    // ✅ ENUM: "public" | "shared" | "private"
    pub total_memories: u32,              // Count of memories

    // ✅ COMPUTED: Storage location (computed from memory storage_edges)
    pub storage_location: Vec<BlobHosting>, // ✅ COMPUTED: Where gallery memories are stored
}
```

### **How Storage Location is Computed:**

```rust
impl Gallery {
    pub fn compute_storage_location(&self) -> Vec<BlobHosting> {
        // 1. Get all memory IDs in this gallery
        let memory_ids: Vec<String> = self.memory_entries.iter()
            .map(|entry| entry.memory_id.clone())
            .collect();

        // 2. Query storage_edges table for each memory
        let mut storage_locations = std::collections::HashSet::new();
        for memory_id in memory_ids {
            // Query storage_edges where memoryId = memory_id
            // Collect unique locationAsset values (S3, ICP, Vercel, etc.)
            let locations = get_storage_locations_for_memory(memory_id);
            storage_locations.extend(locations);
        }

        // 3. Convert to Vec<BlobHosting>
        storage_locations.into_iter().collect()
    }

    pub fn to_header(&self) -> GalleryHeader {
        GalleryHeader {
            id: self.id.clone(),
            title: self.metadata.title.clone(), // ✅ Optional - if None, frontend uses name
            name: self.metadata.name.clone(),
            memory_count: self.memory_entries.len() as u64,
            created_at: self.created_at,
            updated_at: self.updated_at,
            shared_count: self.metadata.shared_count,
            sharing_status: self.metadata.sharing_status.clone(),
            total_memories: self.metadata.total_memories,
            storage_location: self.compute_storage_location(), // ✅ COMPUTED
        }
    }
}
```

### **Title/Name Logic:**

```rust
// Frontend display logic
fn get_display_title(gallery: &GalleryHeader) -> String {
    match &gallery.title {
        Some(title) if !title.is_empty() => title.clone(),
        _ => gallery.name.clone(), // ✅ Fallback to name if title is None or empty
    }
}

// Backend name generation
fn generate_gallery_name(title: Option<&String>) -> String {
    match title {
        Some(title) if !title.is_empty() => title_to_name(title), // Convert "My Gallery" → "my-gallery"
        _ => generate_default_name(), // Generate "gallery-123" or similar
    }
}
```

#### 2. Add Gallery Item Types

**File**: `src/backend/src/types.rs`

```rust
pub struct GalleryItem {
    pub id: String,                       // ✅ From database schema
    pub gallery_id: String,               // ✅ From database schema
    pub memory_id: String,                // ✅ UUID from database schema
    pub memory_type: MemoryType,          // ✅ Enum from database schema
    pub position: u32,                    // ✅ From database schema
    pub caption: Option<String>,          // ✅ From database schema
    pub is_featured: bool,                // ✅ From database schema
    pub metadata: std::collections::HashMap<String, serde_json::Value>, // ✅ From database schema
    // ❌ NO timestamps - it's a relationship table (from database schema)
}

pub struct GalleryItemHeader {
    pub id: String,                       // ✅ From database schema
    pub memory_id: String,                // ✅ From database schema
    pub memory_type: MemoryType,          // ✅ From database schema
    pub position: u32,                    // ✅ From database schema
    pub caption: Option<String>,          // ✅ From database schema
    pub is_featured: bool,                // ✅ From database schema
    // ❌ NO timestamps - it's a relationship table (from database schema)
}
```

#### 3. Add Gallery Sharing Types

**File**: `src/backend/src/types.rs`

```rust
pub struct GalleryShare {
    pub id: String,                       // ✅ From database schema
    pub gallery_id: String,               // ✅ From database schema
    pub owner_id: String,                 // ✅ From database schema
    pub shared_with_type: SharedWithType, // ✅ From database schema
    pub shared_with_id: Option<String>,   // ✅ From database schema
    pub group_id: Option<String>,         // ✅ From database schema
    pub shared_relationship_type: Option<SharingRelationshipType>, // ✅ From database schema
    pub access_level: AccessLevel,        // ✅ From database schema
    pub invitee_secure_code: String,      // ✅ From database schema
    pub invitee_secure_code_created_at: u64, // ✅ From database schema
    pub created_at: u64,                  // ✅ From database schema
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum SharedWithType {
    User,
    Group,
    Relationship,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum AccessLevel {
    Read,
    Write,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum SharingRelationshipType {
    CloseFamily,
    Family,
    Partner,
    CloseFriend,
    Friend,
    Colleague,
    Acquaintance,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum SharingStatus {
    Public,     // Publicly accessible
    Shared,     // Shared with specific users/groups
    Private,    // Only owner can access
}
```

#### 4. Update Gallery Implementation

**File**: `src/backend/src/gallery.rs`

```rust
impl Gallery {
    pub fn to_header(&self) -> GalleryHeader {
        GalleryHeader {
            id: self.id.clone(),
            title: self.metadata.title.clone(),
            name: self.metadata.name.clone(),
            memory_count: self.memory_entries.len() as u64,
            created_at: self.created_at,
            updated_at: self.updated_at,

            // ✅ PRE-COMPUTED: Dashboard fields (from schema.ts)
            // ❌ REMOVED: is_public: self.metadata.is_public,  // Redundant with sharing_status
            shared_count: self.metadata.shared_count,
            sharing_status: self.metadata.sharing_status.clone(),
            total_memories: self.metadata.total_memories,

            // ✅ STORAGE: Multiple storage locations (from schema.ts)
            storage_location: self.metadata.storage_location.clone(),
        }
    }

    pub fn add_memory(&mut self, memory_id: String, memory_type: MemoryType, position: u32) {
        let item = GalleryMemoryEntry {
            memory_id,
            memory_type,
            position,
            // ❌ NO added_at - it's a relationship table (from database schema)
        };
        self.memory_entries.push(item);
        self.metadata.total_memories += 1;
        self.updated_at = ic_cdk::api::time();
    }

    pub fn remove_memory(&mut self, memory_id: &str) {
        self.memory_entries.retain(|entry| entry.memory_id != memory_id);
        self.metadata.total_memories = self.memory_entries.len() as u32;
        self.updated_at = ic_cdk::api::time();
    }

    pub fn add_share(&mut self, share: GalleryShare) {
        self.shares.push(share);
        self.metadata.shared_count += 1;
        self.updated_at = ic_cdk::api::time();
    }

    pub fn remove_share(&mut self, share_id: &str) {
        self.shares.retain(|share| share.id != share_id);
        self.metadata.shared_count = self.shares.len() as u32;
        self.updated_at = ic_cdk::api::time();
    }

    pub fn get_shares_by_type(&self, share_type: SharedWithType) -> Vec<&GalleryShare> {
        self.shares.iter()
            .filter(|share| share.shared_with_type == share_type)
            .collect()
    }
}
```

## 🎯 **Gallery Type Mirroring Strategy**

The Gallery type should be a **mirror** of both:

### **🚨 CRITICAL: Remove isPublic from Both Gallery AND Memory**

**Both Gallery and Memory should remove `isPublic` boolean and use `sharing_status` enum instead:**

- ❌ **REMOVED**: `isPublic: boolean` (redundant with sophisticated sharing system)
- ✅ **ADDED**: `sharing_status: SharingStatus` enum ("public" | "shared" | "private")

**This applies to:**

1. **Gallery types** (this document) ✅ **UPDATED**
2. **Memory types** (backend and database schema) ✅ **UPDATED**
3. **Database schema** (both `galleries` and `memories` tables)

### **Why Remove `isPublic`?**

**`isPublic` is redundant because:**

1. **`sharing_status: SharingStatus`** already tells us if something is public:

   - `SharingStatus::Public` = is public
   - `SharingStatus::Shared` = shared with specific people
   - `SharingStatus::Private` = private

2. **No Need for Boolean**: We don't need both `isPublic: bool` AND `sharing_status: SharingStatus`

3. **Consistency**: All entities (Memory, Gallery, Folder, Capsule) use the same `sharing_status` enum

4. **Future-Proof**: `sharing_status` can be extended with more states if needed

### **1. Memory Pattern (Backend Architecture)**

- ✅ **Metadata Struct**: `GalleryMetadata` (like `MemoryMetadata`)
- ✅ **Capsule ID**: `capsule_id: String` (same as Memory's `capsule_id: String`)
- ✅ **Owner Principal**: `owner_principal: Principal` (ICP Principal for ownership)
- ✅ **Access Control**: Complex sharing via `GalleryShare` (like Memory's `MemoryAccess`)
- ✅ **Storage Location**: `Vec<BlobHosting>` (multiple storage locations, same as memory assets)

### **2. Database Schema (schema.ts)**

- ✅ **Pre-computed Fields**: `is_public`, `shared_count`, `sharing_status`, `total_memories`
- ✅ **Name/Title Semantics**: `title` (user-facing) + `name` (URL-safe)
- ✅ **Storage Fields**: `storage_location` (jsonb array for multiple values, same as memory assets)
- ✅ **Relationship Tables**: `galleryItems` (no timestamps - it's a join table)

### **Key Benefits of Mirroring**:

1. **Architectural Consistency**: Gallery follows same pattern as Memory
2. **Database Alignment**: Backend types match database schema exactly
3. **Type Safety**: Full type safety from database to frontend
4. **Performance**: Pre-computed fields for dashboard display
5. **Maintainability**: Single source of truth for each concern

### **Phase 2: Database Schema Alignment**

#### 1. Add Missing Name Field to Database Schema

**File**: `src/nextjs/src/db/schema.ts`

```typescript
// ❌ CURRENT: Missing name field
export const galleries = pgTable("gallery", {
  // ... existing fields
  title: text("title").notNull(),
  // ❌ MISSING: name field
});

// ✅ UPDATED: Add name field, add storageLocation field, remove unnecessary field
export const galleries = pgTable("gallery", {
  // ... existing fields
  title: text("title").notNull(), // ✅ User-facing title
  name: text("name").notNull(), // ✅ ADD IT: URL-safe identifier (auto-generated)
  // ... other fields
  storageLocation: blob_hosting_t("storage_location").notNull(), // ✅ ADD IT: Same as memory assets
  // ❌ REMOVE: averageStorageDuration: integer("average_storage_duration"),
});
```

**Migration Required**:

1. **ADD IT**: `name` column to existing `gallery` table
2. **ADD IT**: `capsule_id` column to existing `gallery` table (same as memory)
3. **ADD IT**: `storage_location` column to existing `gallery` table (jsonb array for multiple values)
4. **ADD IT**: Pre-computed dashboard fields:
   - ❌ **REMOVED**: `is_public: boolean` (redundant with sharing_status)
   - `shared_count: integer` (count of active shares)
   - `sharing_status: text` ("public" | "shared" | "private")
   - `total_memories: integer` (count of memories)
5. **REMOVE IT**: `averageStorageDuration` column from existing `gallery` table
6. **REMOVE IT**: `storageDistribution` column from existing `gallery` table (legacy field - not needed in greenfield)
7. **REMOVE IT**: `createdAt` and `updatedAt` from `gallery_item` table (relationship table doesn't need timestamps)

#### 2. Update Database Types

**File**: `src/nextjs/src/db/schema.ts`

```typescript
// Ensure gallery types match backend structure
export type DBGallery = typeof galleries.$inferSelect;
export type NewDBGallery = typeof galleries.$inferInsert;

export type DBGalleryItem = typeof galleryItems.$inferSelect;
export type NewDBGalleryItem = typeof galleryItems.$inferInsert;

// Add relations
export const galleriesRelations = relations(galleries, ({ one, many }) => ({
  owner: one(allUsers, {
    fields: [galleries.ownerId],
    references: [allUsers.id],
  }),
  items: many(galleryItems),
  shares: many(galleryShares), // ✅ Gallery sharing relations
}));

export const galleryItemsRelations = relations(galleryItems, ({ one }) => ({
  gallery: one(galleries, {
    fields: [galleryItems.galleryId],
    references: [galleries.id],
  }),
}));

export const gallerySharesRelations = relations(galleryShares, ({ one }) => ({
  gallery: one(galleries, {
    fields: [galleryShares.galleryId],
    references: [galleries.id],
  }),
  owner: one(allUsers, {
    fields: [galleryShares.ownerId],
    references: [allUsers.id],
  }),
  sharedWith: one(allUsers, {
    fields: [galleryShares.sharedWithId],
    references: [allUsers.id],
  }),
  group: one(group, {
    fields: [galleryShares.groupId],
    references: [group.id],
  }),
}));
```

### **Phase 3: Frontend Integration**

#### 1. Update Gallery Services

**File**: `src/nextjs/src/services/galleries.ts`

```typescript
// Transform database gallery to backend format
export function transformDBGalleryToBackend(dbGallery: DBGallery): Gallery {
  return {
    id: dbGallery.id,
    owner_id: dbGallery.ownerId,
    metadata: {
      title: dbGallery.title,
      description: dbGallery.description,
      is_public: dbGallery.isPublic,
      total_memories: dbGallery.totalMemories,
      average_storage_duration: dbGallery.averageStorageDuration,
      storage_distribution: dbGallery.storageDistribution,
    },
    storage_location: determineStorageLocation(dbGallery),
    memory_entries: [], // Will be populated separately
    created_at: dbGallery.createdAt.getTime(),
    updated_at: dbGallery.updatedAt.getTime(),
  };
}

// Transform backend gallery to frontend format
export function transformBackendGalleryToFrontend(backendGallery: Gallery): GalleryWithItems {
  return {
    id: backendGallery.id,
    title: backendGallery.metadata.title,
    name: title_to_name(backendGallery.metadata.title),
    description: backendGallery.metadata.description,
    is_public: backendGallery.metadata.is_public,
    total_memories: backendGallery.metadata.total_memories,
    average_storage_duration: backendGallery.metadata.average_storage_duration,
    storage_distribution: backendGallery.metadata.storage_distribution,
    storage_location: backendGallery.storage_location,
    memory_entries: backendGallery.memory_entries,
    shares: backendGallery.shares, // ✅ Include sharing information
    created_at: new Date(backendGallery.created_at),
    updated_at: new Date(backendGallery.updated_at),
  };
}
```

#### 2. Update Gallery Components

**File**: `src/nextjs/src/components/gallery/gallery-card.tsx`

```typescript
interface GalleryCardProps {
  gallery: GalleryWithItems;
}

export function GalleryCard({ gallery }: GalleryCardProps) {
  return (
    <div className="gallery-card">
      <h3>{gallery.title}</h3>
      <p>{gallery.description}</p>
      <div className="gallery-stats">
        <span>{gallery.total_memories} memories</span>
        <span>Storage: {gallery.storage_location}</span>
        <span>{gallery.shares.length} shares</span>
      </div>
    </div>
  );
}
```

## Benefits

### **Architecture Consistency**

1. **Unified Pattern**: Galleries now follow the same metadata pattern as memories
2. **Clear Separation**: Metadata concerns separated from core entity logic
3. **Scalability**: Easy to add new metadata fields without changing core structure

### **Schema Alignment**

1. **Database Sync**: Backend types match database schema exactly
2. **Type Safety**: Full type safety from database to frontend
3. **No Mismatches**: Eliminates schema drift between backend and database

### **URL Safety**

1. **Auto-generated Names**: URL-safe identifiers generated from titles
2. **Consistent URLs**: All entities use same naming convention
3. **SEO Friendly**: Clean, readable URLs for galleries

### **Maintainability**

1. **Single Source of Truth**: Metadata logic centralized
2. **Easy Updates**: Adding new fields only requires metadata struct changes
3. **Clear Relations**: Proper relations between galleries and items

## Migration Strategy

### **Existing Data**

1. **Gallery Records**: Update existing galleries to use new metadata structure
2. **Gallery Items**: Ensure all items have proper relations
3. **Storage Fields**: Populate storage status fields from existing data
4. **❌ CRITICAL: Generate Name Fields**: Auto-generate `name` fields from existing `title` fields using `title_to_name()` function
5. **❌ CRITICAL: Add Storage Location Field**: Add `storageLocation` column to database
6. **❌ CRITICAL: Remove Unnecessary Field**: Drop `averageStorageDuration` column from database

### **Backward Compatibility**

1. **API**: Keep existing field names, just restructure internally
2. **Frontend**: No breaking changes to display logic
3. **Database**: No schema changes needed (already matches)

## Testing Scenarios

### **Unit Tests**

```rust
#[test]
fn test_gallery_metadata_creation() {
    let metadata = GalleryMetadata {
        title: "Summer Photos".to_string(),
        description: Some("Photos from summer vacation".to_string()),
        is_public: false,
        total_memories: 0,
        average_storage_duration: None,
        storage_distribution: std::collections::HashMap::new(),
    };

    let gallery = Gallery {
        id: "gallery-123".to_string(),
        owner_id: "user-456".to_string(),
        metadata,
        storage_location: GalleryStorageLocation::Web2Only,
        memory_entries: Vec::new(),
        created_at: 1234567890,
        updated_at: 1234567890,
    };

    let header = gallery.to_header();
    assert_eq!(header.title, "Summer Photos");
    assert_eq!(header.name, "summer-photos");
    assert_eq!(header.is_public, false);
    assert_eq!(header.share_count, 0);
}

#[test]
fn test_gallery_memory_management() {
    let mut gallery = create_test_gallery();

    // Add memory
    gallery.add_memory("memory-1".to_string(), MemoryType::Image, 1);
    assert_eq!(gallery.metadata.total_memories, 1);
    assert_eq!(gallery.memory_entries.len(), 1);

    // Remove memory
    gallery.remove_memory("memory-1");
    assert_eq!(gallery.metadata.total_memories, 0);
    assert_eq!(gallery.memory_entries.len(), 0);
}

#[test]
fn test_gallery_sharing_management() {
    let mut gallery = create_test_gallery();

    // Add share
    let share = GalleryShare {
        id: "share-1".to_string(),
        gallery_id: gallery.id.clone(),
        owner_id: "owner-1".to_string(),
        shared_with_type: SharedWithType::User,
        shared_with_id: Some("user-2".to_string()),
        group_id: None,
        shared_relationship_type: None,
        access_level: AccessLevel::Read,
        invitee_secure_code: "secure-code".to_string(),
        invitee_secure_code_created_at: 1234567890,
        created_at: 1234567890,
    };

    gallery.add_share(share);
    assert_eq!(gallery.shares.len(), 1);

    // Get shares by type
    let user_shares = gallery.get_shares_by_type(SharedWithType::User);
    assert_eq!(user_shares.len(), 1);

    // Remove share
    gallery.remove_share("share-1");
    assert_eq!(gallery.shares.len(), 0);
}
```

### **Integration Tests**

1. **Gallery Creation**: Verify metadata is properly structured
2. **Memory Management**: Test adding/removing memories
3. **Sharing Management**: Test adding/removing shares
4. **Storage Status**: Verify storage location tracking
5. **Frontend Display**: Test gallery card rendering with sharing info

## Success Criteria

- [ ] Gallery types follow memory pattern (metadata struct)
- [ ] Backend types match database schema exactly
- [ ] **❌ CRITICAL: Add `name` field to database schema**
- [ ] **❌ CRITICAL: Add `capsule_id` field to database schema (same as memory)**
- [ ] **❌ CRITICAL: Generate `name` fields from existing `title` fields**
- [ ] **❌ CRITICAL: Add `storageLocation` field to database schema (jsonb array for multiple values)**
- [ ] **❌ CRITICAL: Remove `averageStorageDuration` field from database schema**
- [ ] **❌ CRITICAL: Remove `storageDistribution` field from database schema (legacy field - not needed in greenfield)**
- [ ] **❌ CRITICAL: Remove `isPublic` from both Gallery AND Memory types**
- [ ] **❌ CRITICAL: Add `sharing_status` enum to both Gallery AND Memory types**
- [ ] **❌ CRITICAL: Replace `MemoryAccess` with sophisticated Web2 sharing system**
- [ ] **❌ CRITICAL: Add bitmask permissions (VIEW/DOWNLOAD/SHARE/MANAGE/OWN)**
- [ ] **❌ CRITICAL: Add role system (owner/superadmin/admin/member/guest)**
- [ ] **❌ CRITICAL: Add magic links with TTL/limits and audit trail**
- [ ] URL-safe names auto-generated from titles
- [ ] Proper relations between galleries and items
- [ ] Gallery sharing functionality implemented
- [ ] Storage status fields properly tracked
- [ ] No breaking changes to existing functionality
- [ ] All tests pass

## Priority Justification

**MEDIUM Priority** because:

- **Architecture Improvement**: Establishes consistent patterns across entities
- **Schema Alignment**: Eliminates backend/database mismatches
- **Future-Proofing**: Makes adding new gallery features easier
- **Maintainability**: Reduces confusion and bugs
- **Not Urgent**: Doesn't block current functionality

## Dependencies

- Backend developer (Rust)
- Frontend developer (TypeScript)
- QA for testing
- Database migration (if needed)

## Timeline

- **Week 1**: Backend type refactoring and implementation
- **Week 2**: Database schema alignment and testing
- **Week 3**: Frontend integration and testing
- **Week 4**: Migration and deployment

**Total Estimated Time**: 3-4 weeks

## Notes

- This refactor should be done after the memory type refactoring
- The new structure will be used for all future gallery features
- Consider this a foundational improvement that enables better architecture
- The metadata pattern can be extended to other entity types (folders, etc.)

## Appendix: Memory Type Reference

### **Memory Type Structure (for reference)**

**File**: `src/backend/src/memories/types.rs`

```rust
/// Main memory structure
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Memory {
    pub id: String,                                         // UUID v7 (not compound)
    pub capsule_id: String,                                 // Capsule context
    pub metadata: MemoryMetadata, // memory-level metadata (title, description, etc.)
    pub access: MemoryAccess,     // who can access + temporal rules
    pub inline_assets: Vec<MemoryAssetInline>, // 0 or more inline assets
    pub blob_internal_assets: Vec<MemoryAssetBlobInternal>, // 0 or more ICP blob assets
    pub blob_external_assets: Vec<MemoryAssetBlobExternal>, // 0 or more external blob assets
}

/// Memory header for listings
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct MemoryHeader {
    pub id: String,         // UUID v7 (not compound)
    pub capsule_id: String, // Capsule context
    pub name: String,
    pub memory_type: MemoryType,
    pub size: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub access: MemoryAccess,

    // NEW: Dashboard-specific fields (pre-computed)
    pub title: Option<String>,             // From metadata
    pub description: Option<String>,       // From metadata
    // ❌ REMOVED: pub is_public: bool,                   // Redundant with sharing_status
    pub shared_count: u32,                 // Count of shared recipients
    pub sharing_status: SharingStatus,     // ✅ ENUM: "public" | "shared" | "private"
    pub total_size: u64,                   // Sum of all asset sizes
    pub asset_count: u32,                  // Total number of assets
    pub thumbnail_url: Option<String>,     // Pre-computed thumbnail URL
    pub primary_asset_url: Option<String>, // Primary asset URL for display
    pub has_thumbnails: bool,              // Whether thumbnails exist
    pub has_previews: bool,                // Whether previews exist
}

/// Enhanced MemoryMetadata (Memory-Level Metadata)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
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

    // Dashboard-specific fields (pre-computed)
    // ❌ REMOVED: pub is_public: bool,                   // Redundant with sharing_status
    pub shared_count: u32,                 // Count of shared recipients
    pub sharing_status: SharingStatus,     // ✅ ENUM: "public" | "shared" | "private"
    pub total_size: u64,                   // Sum of all asset sizes
    pub asset_count: u32,                  // Total number of assets
    pub thumbnail_url: Option<String>,     // Pre-computed thumbnail URL
    pub primary_asset_url: Option<String>, // Primary asset URL for display
    pub has_thumbnails: bool,              // Whether thumbnails exist
    pub has_previews: bool,                // Whether previews exist
}
```

### **Key Memory Pattern Elements for Gallery Mirroring:**

1. **Capsule Context**: `capsule_id: String` - Gallery should have this too
2. **Metadata Struct**: `MemoryMetadata` - Gallery should have `GalleryMetadata`
3. **Pre-computed Fields**: Dashboard fields in metadata - Gallery should have same
4. **Access Control**: `MemoryAccess` - Gallery should have `GalleryShare` system
5. **Storage Assets**: Multiple asset types - Gallery should have multiple storage locations
6. **Header Pattern**: `MemoryHeader` for listings - Gallery should have `GalleryHeader`

### **Gallery Should Mirror This Pattern:**

```rust
pub struct Gallery {
    pub id: String,
    pub capsule_id: String,               // ✅ SAME AS MEMORY
    pub owner_principal: Principal,       // ✅ ICP Principal (Memory doesn't have this)
    pub metadata: GalleryMetadata,        // ✅ LIKE MemoryMetadata
    pub storage_location: GalleryStorageLocation,
    pub memory_entries: Vec<GalleryMemoryEntry>,
    pub created_at: u64,
    pub updated_at: u64,
}
```

## 🚨 **CRITICAL ANALYSIS: MemoryAccess vs Web2 Sharing System**

### **MemoryAccess (ICP Backend) - TOO PRIMITIVE**

**Current MemoryAccess enum:**

```rust
pub enum MemoryAccess {
    Public { owner_secure_code: String },
    Private { owner_secure_code: String },
    Custom {
        individuals: Vec<PersonRef>,
        groups: Vec<String>,
        owner_secure_code: String
    },
    Scheduled {
        accessible_after: u64,
        access: Box<MemoryAccess>,
        owner_secure_code: String
    },
    EventTriggered {
        trigger_event: AccessEvent,
        access: Box<MemoryAccess>,
        owner_secure_code: String
    },
}
```

**Problems with MemoryAccess:**

1. **No Granular Permissions**: Only basic access, no VIEW/DOWNLOAD/SHARE/MANAGE/OWN → **Solution**: Use bitmask permissions (PERM.VIEW | PERM.DOWNLOAD | PERM.SHARE)
2. **No Role System**: No owner/superadmin/admin/member/guest roles
3. **No Provenance Tracking**: No way to track who granted access
4. **No Magic Links**: No token-based sharing with TTL/limits
5. **No Public Modes**: No public-auth vs public-link distinction
6. **No Audit Trail**: No consumption tracking or usage logs
7. **No Group Management**: Groups are just string IDs, no group membership
8. **No Bitmask Permissions**: No efficient permission combination

### **Web2 Sharing System (Sophisticated)**

**Our Web2 system has:**

```typescript
// ✅ Granular Permissions (5 levels)
export const PERM = {
  VIEW: 1 << 0,     // 1
  DOWNLOAD: 1 << 1, // 2
  SHARE: 1 << 2,    // 4
  MANAGE: 1 << 3,   // 8
  OWN: 1 << 4,      // 16
} as const;

// ✅ Role System (5 roles)
type ResourceRole = 'owner' | 'superadmin' | 'admin' | 'member' | 'guest';

// ✅ Provenance Tracking
type GrantSource = 'user' | 'group' | 'magic_link' | 'public_mode' | 'system';

// ✅ Magic Links with TTL/limits
export const magicLink = pgTable('magic_link', {
  tokenHash: text('token_hash').notNull().unique(),
  maxUses: integer('max_uses').notNull().default(1000),
  expiresAt: timestamp('expires_at').notNull(),
  // ... audit trail
});

// ✅ Public Modes
type PublicMode = 'private' | 'public_auth' | 'public_link';

// ✅ Bitmask Permissions
permMask: integer('perm_mask').notNull().default(0),
```

### **RECOMMENDATION: Universal ResourceAccess System**

**For ALL resource types (Memory, Gallery, Folder, Capsule), we should:**

1. **❌ REMOVE**: `MemoryAccess` enum (too primitive)
2. **✅ ADD**: Universal `ResourceAccess` system (works for all resource types)
3. **✅ ADD**: Bitmask permissions (VIEW/DOWNLOAD/SHARE/MANAGE/OWN)
4. **✅ ADD**: Role system (owner/superadmin/admin/member/guest)
5. **✅ ADD**: Magic links with TTL/limits
6. **✅ ADD**: Public modes (private/public-auth/public-link)
7. **✅ ADD**: Provenance tracking (who granted access)
8. **✅ ADD**: Audit trail (consumption tracking)
9. **✅ ADD**: Universal resource types (memory/gallery/folder/capsule)
10. **✅ INTEGRATE**: Capsule connections system (social graph)

**Universal ResourceAccess System:**

```rust
// ✅ UNIVERSAL: Works for Memory, Gallery, Folder, Capsule
pub struct ResourceAccess {
    pub access_entries: Vec<AccessEntry>, // ✅ Universal sharing system
    pub public_policy: Option<PublicPolicy>, // ✅ Public access modes
}

pub struct AccessEntry {
    pub id: String,
    pub person_ref: PersonRef,            // ✅ ICP: Principal or Opaque ID
    pub grant_source: GrantSource,        // ✅ Provenance tracking
    pub source_id: Option<String>,        // ✅ Group/magic_link ID
    pub role: ResourceRole,               // ✅ Role system
    pub perm_mask: u32,                   // ✅ Bitmask permissions
    pub invited_by_person_ref: Option<PersonRef>, // ✅ ICP: Who granted access
    pub created_at: u64,
    pub updated_at: u64,
}

// ✅ UNIVERSAL ENUMS
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum ResourceType {
    Memory,
    Gallery,
    Folder,
    Capsule,
}

// ✅ DESIGN RATIONALE: Why no resource_type/resource_id in ResourceAccess/ResourceShare?
//
// 1. **Context Inference**: resource_type and resource_id are known from the containing struct:
//    - Memory.access → resource_type = Memory, resource_id = Memory.id
//    - Gallery.access → resource_type = Gallery, resource_id = Gallery.id
//    - Folder.access → resource_type = Folder, resource_id = Folder.id
//    - Capsule.access → resource_type = Capsule, resource_id = Capsule.id
//
// 2. **No Redundancy**: Avoids storing the same information twice
// 3. **No Inconsistency Risk**: Can't get out of sync
// 4. **Storage Efficiency**: Less data to store and transfer
// 5. **Simpler API**: Fewer fields to manage
// 6. **Object-Oriented Design**: ResourceAccess is an attribute of the resource, not a separate entity
//
// 7. **Database Design**: Matches our Web2 schema where resource_type and resource_id are
//    stored once in the main resource table, not in every share record
//
// 8. **ID Format Consistency**: All resources use String IDs (UUID v7 for Memory,
//    unique identifiers for Gallery/Capsule/Folder)

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum GrantSource {
    User,           // Direct user grant
    Group,          // Group membership grant
    MagicLink,      // Magic link grant
    PublicMode,     // Public access grant
    System,         // System-generated grant
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum ResourceRole {
    Owner,          // Full ownership
    SuperAdmin,     // Administrative access
    Admin,          // Management access
    Member,         // Standard access
    Guest,          // Limited access
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum PublicMode {
    Private,        // No public access
    PublicAuth,     // Public for authenticated users
    PublicLink,     // Public for anyone with link
}

// ✅ PERMISSION CONSTANTS (same as Web2)
pub const PERM_VIEW: u32 = 1 << 0;      // 1
pub const PERM_DOWNLOAD: u32 = 1 << 1;  // 2
pub const PERM_SHARE: u32 = 1 << 2;     // 4
pub const PERM_MANAGE: u32 = 1 << 3;    // 8
pub const PERM_OWN: u32 = 1 << 4;       // 16
```

**Updated Resource Structures:**

```rust
// ✅ ALL resources use the same access system
pub struct Memory {
    pub id: String,
    pub capsule_id: String,
    pub metadata: MemoryMetadata,
    pub access: ResourceAccess,           // ✅ UNIVERSAL ACCESS SYSTEM
    pub inline_assets: Vec<MemoryAssetInline>,
    pub blob_internal_assets: Vec<MemoryAssetBlobInternal>,
    pub blob_external_assets: Vec<MemoryAssetBlobExternal>,
}

pub struct Gallery {
    pub id: String,
    pub capsule_id: String,
    pub owner_principal: Principal,
    pub metadata: GalleryMetadata,        // ✅ Storage location computed in metadata
    pub memory_entries: Vec<GalleryMemoryEntry>,
    pub access: ResourceAccess,           // ✅ UNIVERSAL ACCESS SYSTEM
    pub created_at: u64,
    pub updated_at: u64,
}

pub struct Folder {
    pub id: String,
    pub capsule_id: String,
    pub metadata: FolderMetadata,
    pub access: ResourceAccess,           // ✅ UNIVERSAL ACCESS SYSTEM
    pub created_at: u64,
    pub updated_at: u64,
}

pub struct Capsule {
    pub id: String,
    pub owner_principal: Principal,
    pub metadata: CapsuleMetadata,
    pub access: ResourceAccess,           // ✅ UNIVERSAL ACCESS SYSTEM
    pub created_at: u64,
    pub updated_at: u64,
}
```

**This gives us:**

- ✅ **Universal System**: Same access control for Memory, Gallery, Folder, Capsule
- ✅ **Granular Permissions**: 5 permission levels (VIEW/DOWNLOAD/SHARE/MANAGE/OWN)
- ✅ **Role System**: 5 role types (owner/superadmin/admin/member/guest)
- ✅ **Magic Links**: Token-based sharing with TTL/limits
- ✅ **Public Modes**: 3 public access modes (private/public-auth/public-link)
- ✅ **Provenance Tracking**: Know who granted access (user/group/magic_link/public_mode/system)
- ✅ **Audit Trail**: Track usage and consumption
- ✅ **Bitmask Efficiency**: Fast permission checking with single integer
- ✅ **Web2 Compatibility**: Same system as database
- ✅ **Consistency**: Identical sharing logic across all resource types
- ✅ **Maintainability**: Single access control system to maintain
- ✅ **Capsule Integration**: Leverages existing capsule connections (social graph)

## **🎯 CAPSULE CONNECTIONS INTEGRATION**

**You're absolutely right!** The capsule system already has a **connections system** that defines the social graph:

### **Existing Capsule Connections System:**

```rust
// From src/backend/src/types.rs
pub struct Capsule {
    pub id: String,
    pub owners: HashMap<PersonRef, OwnerState>,
    pub controllers: HashMap<PersonRef, ControllerState>,
    pub connections: HashMap<PersonRef, Connection>,         // ✅ SOCIAL GRAPH
    pub connection_groups: HashMap<String, ConnectionGroup>, // ✅ ORGANIZED GROUPS
    pub memories: HashMap<String, Memory>,
    pub galleries: HashMap<String, Gallery>,
    // ...
}

pub struct Connection {
    pub peer: PersonRef,                    // ✅ WHO: Connected person
    pub status: ConnectionStatus,           // ✅ STATUS: Pending/Accepted/Blocked/Revoked
    pub created_at: u64,
    pub updated_at: u64,
}

pub enum ConnectionStatus {
    Pending,    // ✅ Invitation sent, waiting for acceptance
    Accepted,   // ✅ Connection established
    Blocked,    // ✅ Connection blocked
    Revoked,    // ✅ Connection revoked
}

pub struct ConnectionGroup {
    pub id: String,
    pub name: String,                       // ✅ "Family", "Close Friends", etc.
    pub description: Option<String>,
    pub members: Vec<PersonRef>,            // ✅ Group members
    pub created_at: u64,
    pub updated_at: u64,
}
```

### **How AccessEntry Integrates with Connections:**

```rust
pub struct AccessEntry {
    pub id: String,
    pub person_ref: PersonRef,              // ✅ WHO: Specific user (from connections)
    pub grant_source: GrantSource,          // ✅ HOW: How they got access
    pub source_id: Option<String>,          // ✅ WHAT: ConnectionGroup ID or magic link
    pub role: ResourceRole,                 // ✅ ROLE: Permission level
    pub perm_mask: u32,                     // ✅ PERMISSIONS: Specific permissions
    pub invited_by_person_ref: Option<PersonRef>, // ✅ BY WHOM: Who granted access
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum GrantSource {
    User,           // ✅ Direct user grant (from connections)
    Group,          // ✅ Group membership grant (from connection_groups)
    MagicLink,      // ✅ Magic link grant (temporary access)
    PublicMode,     // ✅ Public access grant
    System,         // ✅ System-generated grant
}
```

### **Integration Examples:**

```rust
// Example 1: Share with specific connection
let access_entry = AccessEntry {
    id: "access_1",
    person_ref: PersonRef::Principal(alice_principal),
    grant_source: GrantSource::User,        // ✅ Direct connection
    source_id: None,
    role: ResourceRole::Member,
    perm_mask: PERM_VIEW | PERM_DOWNLOAD,
    invited_by_person_ref: Some(owner_principal),
    created_at: 1234567890,
    updated_at: 1234567890,
};

// Example 2: Share with connection group
let access_entry = AccessEntry {
    id: "access_2",
    person_ref: PersonRef::Principal(bob_principal),
    grant_source: GrantSource::Group,       // ✅ From connection group
    source_id: Some("family_group_id"),     // ✅ ConnectionGroup ID
    role: ResourceRole::Admin,
    perm_mask: PERM_VIEW | PERM_DOWNLOAD | PERM_SHARE,
    invited_by_person_ref: Some(owner_principal),
    created_at: 1234567890,
    updated_at: 1234567890,
};

// Example 3: Share with magic link (temporary access)
let access_entry = AccessEntry {
    id: "access_3",
    person_ref: PersonRef::Principal(charlie_principal),
    grant_source: GrantSource::MagicLink,   // ✅ Temporary access
    source_id: Some("magic_link_token_123"),
    role: ResourceRole::Guest,
    perm_mask: PERM_VIEW,
    invited_by_person_ref: Some(owner_principal),
    created_at: 1234567890,
    updated_at: 1234567890,
};
```

### **Key Benefits of Integration:**

1. **✅ Leverages Existing System**: Uses capsule's social graph (connections + connection_groups)
2. **✅ No Duplication**: Doesn't recreate social relationships
3. **✅ Consistent**: Same connection system for all resource types
4. **✅ Efficient**: Connections already established, just add permissions
5. **✅ Scalable**: Can share with individuals, groups, or temporary access
6. **✅ Flexible**: Supports both persistent (connections) and temporary (magic links) access

### **Access Control Flow:**

```
1. User wants to share Memory/Gallery with someone
2. Check if they have a connection (capsule.connections)
3. If yes: Create AccessEntry with GrantSource::User
4. If no: Create AccessEntry with GrantSource::MagicLink (temporary)
5. For groups: Use capsule.connection_groups + GrantSource::Group
6. For public: Use GrantSource::PublicMode
```

**This way, AccessEntry defines WHO has access, but leverages the capsule's existing connections system to determine the social graph!**
