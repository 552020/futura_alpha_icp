# Folder Sharing Implementation - Analysis and Design

**Status**: `OPEN` - Analysis and Design Phase  
**Priority**: `MEDIUM` - Future Enhancement  
**Assigned**: Backend Developer + Frontend Developer  
**Created**: 2024-12-19  
**Related Issues**: [Gallery Type Refactor](./gallery-type-refactor.md), [Gallery Sharing Table Enhancement](./gallery-sharing-table-enhancement.md)

## 🎯 **Objective**

Analyze the feasibility and design requirements for implementing folder sharing functionality, building upon the existing memory sharing system and the proposed gallery sharing enhancements.

## 📋 **Current State Analysis**

### **Existing Folder System**

```typescript
// Current folder structure (from schema.ts)
export const folders = pgTable("folder", {
  id: uuid("id").primaryKey().defaultRandom(),
  ownerId: text("owner_id")
    .notNull()
    .references(() => allUsers.id, { onDelete: "cascade" }),
  name: text("name").notNull(),
  parentFolderId: uuid("parent_folder_id"), // Self-referencing for nested folders
  createdAt: timestamp("created_at").defaultNow().notNull(),
  updatedAt: timestamp("updated_at").defaultNow().notNull(),
});
```

### **Current Memory Sharing System**

```typescript
// Existing memory sharing (from schema.ts)
export const memoryShares = pgTable("memory_share", {
  id: text("id")
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID()),
  memoryId: uuid("memory_id").notNull(),
  memoryType: text("memory_type", { enum: MEMORY_TYPES }).notNull(),
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

### **What We Have**

- ✅ **Folder hierarchy** with parent-child relationships
- ✅ **Memory sharing** with user/group/relationship-based access
- ✅ **Access levels** (read/write) for individual memories
- ✅ **Secure codes** for invitee access

### **What We're Missing**

- ❌ **Folder-level sharing** (share entire folder structure)
- ❌ **Inheritance rules** (how folder permissions affect child folders/memories)
- ❌ **Bulk operations** (share all memories in a folder)
- ❌ **Folder-specific permissions** (different from memory permissions)

## 🤔 **Folder Sharing Use Cases**

### **1. Family Photo Organization**

- **Scenario**: Parent creates "2024 Family Photos" folder with subfolders by month
- **Need**: Share entire folder structure with family members
- **Requirements**:
  - Share folder + all subfolders + all memories
  - Different permissions for different family members
  - Ability to add new memories to shared folders

### **2. Wedding Event Organization**

- **Scenario**: Photographer creates "Wedding Day" folder with ceremony, reception, etc. subfolders
- **Need**: Share with wedding party, family, guests
- **Requirements**:
  - Role-based access (bride/groom: full access, guests: view only)
  - Ability to create new subfolders within shared folder
  - Bulk download permissions

### **3. Project Collaboration**

- **Scenario**: Team creates "Project Alpha" folder for shared documents
- **Need**: Collaborate on folder structure and contents
- **Requirements**:
  - Real-time folder structure updates
  - Permission inheritance for new items
  - Audit trail for folder changes

## 🏗️ **Implementation Approaches**

### **Approach 1: Folder-Specific Sharing Table**

```typescript
// New table for folder sharing
export const folderShares = pgTable("folder_share", {
  id: text("id")
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID()),
  folderId: uuid("folder_id")
    .notNull()
    .references(() => folders.id, { onDelete: "cascade" }),
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

  // Folder-specific permissions
  canViewFolder: boolean("can_view_folder").default(true).notNull(),
  canCreateSubfolders: boolean("can_create_subfolders").default(false).notNull(),
  canAddMemories: boolean("can_add_memories").default(false).notNull(),
  canRemoveMemories: boolean("can_remove_memories").default(false).notNull(),
  canCreateSelection: boolean("can_create_selection").default(false).notNull(), // ✅ FOLDER FEATURE
  canManageFolder: boolean("can_manage_folder").default(false).notNull(),

  // Inheritance rules
  inheritToSubfolders: boolean("inherit_to_subfolders").default(true).notNull(),
  inheritToMemories: boolean("inherit_to_memories").default(true).notNull(),

  accessLevel: text("access_level", { enum: ACCESS_LEVELS }).default("read").notNull(),
  inviteeSecureCode: text("invitee_secure_code").notNull(),
  inviteeSecureCodeCreatedAt: timestamp("secure_code_created_at", { mode: "date" }).notNull().defaultNow(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});
```

**Pros**:

- ✅ Explicit folder-level permissions
- ✅ Clear inheritance rules
- ✅ Folder-specific operations (create subfolders, manage structure)

**Cons**:

- ❌ Duplicate sharing logic (similar to memoryShares)
- ❌ Complex permission inheritance calculations
- ❌ Potential for permission conflicts

### **Approach 2: Unified Sharing with Resource Types**

```typescript
// Enhanced sharing table that handles both memories and folders
export const resourceShares = pgTable("resource_share", {
  id: text("id")
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID()),

  // Resource identification
  resourceType: text("resource_type", { enum: ["memory", "folder", "gallery"] }).notNull(),
  resourceId: text("resource_id").notNull(), // Can be UUID (memory/folder) or text (gallery)

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

  // Unified permissions (resource-type specific)
  permissions: json("permissions")
    .$type<{
      // Memory permissions
      canView?: boolean;
      canDownload?: boolean;

      // Folder permissions
      canViewFolder?: boolean;
      canCreateSubfolders?: boolean;
      canAddMemories?: boolean;
      canRemoveMemories?: boolean;
      canManageFolder?: boolean;

      // Gallery permissions (from gallery sharing enhancement)
      canDownloadWeb?: boolean;
      canDownloadOriginals?: boolean;
      canReshare?: boolean;
      canManageShares?: boolean;
      canCreateSelection?: boolean;
      canMakePublic?: boolean;
    }>()
    .notNull(),

  // Inheritance rules
  inheritToChildren: boolean("inherit_to_children").default(true).notNull(),

  accessLevel: text("access_level", { enum: ACCESS_LEVELS }).default("read").notNull(),
  inviteeSecureCode: text("invitee_secure_code").notNull(),
  inviteeSecureCodeCreatedAt: timestamp("secure_code_created_at", { mode: "date" }).notNull().defaultNow(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});
```

**Pros**:

- ✅ Unified sharing logic for all resource types
- ✅ Extensible for future resource types
- ✅ Consistent permission model

**Cons**:

- ❌ Complex permission JSON structure
- ❌ Migration complexity from existing tables
- ❌ Potential performance issues with JSON queries

### **Approach 3: Hierarchical Permission Inheritance**

```typescript
// Keep existing memoryShares, add folderShares, implement inheritance logic
export const folderShares = pgTable("folder_share", {
  id: text("id")
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID()),
  folderId: uuid("folder_id")
    .notNull()
    .references(() => folders.id, { onDelete: "cascade" }),
  ownerId: text("owner_id")
    .notNull()
    .references(() => allUsers.id, { onDelete: "cascade" }),

  // Same structure as memoryShares but for folders
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

// Permission inheritance logic (application-level)
export class FolderPermissionService {
  async getUserPermissions(
    userId: string,
    resourceId: string,
    resourceType: "memory" | "folder"
  ): Promise<AccessLevel | null> {
    if (resourceType === "memory") {
      // Check direct memory share
      const memoryShare = await this.getMemoryShare(userId, resourceId);
      if (memoryShare) return memoryShare.accessLevel;

      // Check folder inheritance
      const memory = await this.getMemory(resourceId);
      if (memory.parentFolderId) {
        return this.getUserPermissions(userId, memory.parentFolderId, "folder");
      }
    }

    if (resourceType === "folder") {
      // Check direct folder share
      const folderShare = await this.getFolderShare(userId, resourceId);
      if (folderShare) return folderShare.accessLevel;

      // Check parent folder inheritance
      const folder = await this.getFolder(resourceId);
      if (folder.parentFolderId) {
        return this.getUserPermissions(userId, folder.parentFolderId, "folder");
      }
    }

    return null;
  }
}
```

**Pros**:

- ✅ Minimal schema changes
- ✅ Clear inheritance logic
- ✅ Backward compatibility

**Cons**:

- ❌ Complex application logic
- ❌ Potential performance issues with recursive queries
- ❌ Difficult to optimize

## 🎯 **Recommended Approach**

### **Hybrid Approach: Folder Shares + Smart Inheritance**

```typescript
// 1. Add folder sharing table (similar to memoryShares)
export const folderShares = pgTable("folder_share", {
  id: text("id")
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID()),
  folderId: uuid("folder_id")
    .notNull()
    .references(() => folders.id, { onDelete: "cascade" }),
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

  // Folder-specific permissions
  canViewFolder: boolean("can_view_folder").default(true).notNull(),
  canCreateSubfolders: boolean("can_create_subfolders").default(false).notNull(),
  canAddMemories: boolean("can_add_memories").default(false).notNull(),
  canRemoveMemories: boolean("can_remove_memories").default(false).notNull(),
  canCreateSelection: boolean("can_create_selection").default(false).notNull(), // ✅ FOLDER FEATURE
  canManageFolder: boolean("can_manage_folder").default(false).notNull(),

  // Inheritance settings
  inheritToSubfolders: boolean("inherit_to_subfolders").default(true).notNull(),
  inheritToMemories: boolean("inherit_to_memories").default(true).notNull(),

  accessLevel: text("access_level", { enum: ACCESS_LEVELS }).default("read").notNull(),
  inviteeSecureCode: text("invitee_secure_code").notNull(),
  inviteeSecureCodeCreatedAt: timestamp("secure_code_created_at", { mode: "date" }).notNull().defaultNow(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

// 2. Enhanced permission checking service
export class EnhancedPermissionService {
  async getUserAccessLevel(
    userId: string,
    resourceId: string,
    resourceType: "memory" | "folder"
  ): Promise<AccessLevel | null> {
    // Direct access check
    const directAccess = await this.getDirectAccess(userId, resourceId, resourceType);
    if (directAccess) return directAccess;

    // Inheritance check
    if (resourceType === "memory") {
      const memory = await this.getMemory(resourceId);
      if (memory.parentFolderId) {
        return this.getUserAccessLevel(userId, memory.parentFolderId, "folder");
      }
    }

    if (resourceType === "folder") {
      const folder = await this.getFolder(resourceId);
      if (folder.parentFolderId) {
        return this.getUserAccessLevel(userId, folder.parentFolderId, "folder");
      }
    }

    return null;
  }

  async getFolderPermissions(userId: string, folderId: string): Promise<FolderPermissions | null> {
    const folderShare = await this.getFolderShare(userId, folderId);
    if (folderShare) {
      return {
        canViewFolder: folderShare.canViewFolder,
        canCreateSubfolders: folderShare.canCreateSubfolders,
        canAddMemories: folderShare.canAddMemories,
        canRemoveMemories: folderShare.canRemoveMemories,
        canManageFolder: folderShare.canManageFolder,
        accessLevel: folderShare.accessLevel,
      };
    }

    // Check parent folder inheritance
    const folder = await this.getFolder(folderId);
    if (folder.parentFolderId) {
      return this.getFolderPermissions(userId, folder.parentFolderId);
    }

    return null;
  }
}
```

## 🔧 **Implementation Plan**

### **Phase 1: Database Schema (Week 1-2)**

1. **Add `folderShares` table**
2. **Add indexes** for performance
3. **Create migration scripts**
4. **Add type definitions**

### **Phase 2: Backend API (Week 3-4)**

1. **Folder sharing endpoints**

   - `POST /api/folders/:id/share`
   - `GET /api/folders/:id/shares`
   - `DELETE /api/folders/:id/shares/:shareId`
   - `PUT /api/folders/:id/shares/:shareId`

2. **Permission checking service**

   - Enhanced permission inheritance logic
   - Bulk permission calculations
   - Caching for performance

3. **Folder operations with permissions**
   - Create subfolder (check permissions)
   - Add memory to folder (check permissions)
   - Remove memory from folder (check permissions)

### **Phase 3: Frontend Integration (Week 5-6)**

1. **Folder sharing UI**

   - Share folder dialog
   - Permission management interface
   - Share status indicators

2. **Folder operations**
   - Permission-aware folder actions
   - Visual indicators for shared folders
   - Bulk operations for shared folders

### **Phase 4: Testing & Optimization (Week 7-8)**

1. **Comprehensive testing**

   - Permission inheritance scenarios
   - Edge cases and error handling
   - Performance testing

2. **Optimization**
   - Query optimization
   - Caching strategies
   - Bulk operation improvements

## 📊 **API Surface**

### **Folder Sharing Endpoints**

```typescript
// Share folder with user/group/relationship
POST /api/folders/:id/share
{
  "sharedWithType": "user" | "group" | "relationship",
  "sharedWithId": string,
  "groupId": string,
  "sharedRelationshipType": string,
  "canViewFolder": boolean,
  "canCreateSubfolders": boolean,
  "canAddMemories": boolean,
  "canRemoveMemories": boolean,
  "canManageFolder": boolean,
  "inheritToSubfolders": boolean,
  "inheritToMemories": boolean,
  "accessLevel": "read" | "write"
}

// Get folder shares
GET /api/folders/:id/shares

// Update folder share
PUT /api/folders/:id/shares/:shareId

// Remove folder share
DELETE /api/folders/:id/shares/:shareId

// Get user's folder permissions
GET /api/folders/:id/permissions
```

### **Enhanced Folder Operations**

```typescript
// Create subfolder (with permission check)
POST /api/folders/:parentId/subfolders
{
  "name": string
}

// Add memory to folder (with permission check)
POST /api/folders/:folderId/memories
{
  "memoryId": string
}

// Remove memory from folder (with permission check)
DELETE /api/folders/:folderId/memories/:memoryId
```

## 🧪 **Testing Strategy**

### **Unit Tests**

- Permission inheritance logic
- Folder sharing operations
- Edge cases (circular references, deep nesting)

### **Integration Tests**

- End-to-end folder sharing workflows
- Permission inheritance across folder hierarchy
- Bulk operations on shared folders

### **Performance Tests**

- Deep folder hierarchy permission checks
- Bulk folder operations
- Concurrent access scenarios

## 🚀 **Future Enhancements**

### **Advanced Features**

1. **Selective inheritance** - Choose which permissions to inherit
2. **Time-based sharing** - Temporary folder access
3. **Folder templates** - Pre-configured sharing setups
4. **Bulk folder operations** - Share multiple folders at once
5. **Folder analytics** - Track folder access and usage

### **Integration Opportunities**

1. **Gallery integration** - Share folders as galleries
2. **Team collaboration** - Real-time folder updates
3. **External sharing** - Public folder links
4. **Mobile optimization** - Folder sharing on mobile devices

## 📚 **References**

- [Gallery Type Refactor](./gallery-type-refactor.md)
- [Gallery Sharing Table Enhancement](./gallery-sharing-table-enhancement.md)
- [Memory Sharing System](../../../src/nextjs/src/db/schema.ts)
- [Folder Schema](../../../src/nextjs/src/db/schema.ts)
