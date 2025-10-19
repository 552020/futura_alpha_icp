# Sharing System Implementation

## Overview

This document outlines the implementation of the universal resource sharing system for memories, galleries, and folders.

## Architecture

### Universal Sharing System

The sharing system uses a single `resourceMembership` table to handle sharing for all resource types:

- **Memories**: Individual files/photos
- **Galleries**: Collections of memories
- **Folders**: Directory structures

### Database Schema

```typescript
resourceMembership {
  id: string (primary key)
  resourceType: 'memory' | 'gallery' | 'folder'
  resourceId: string (ID of the resource being shared)
  allUserId: string (ID of user being granted access)
  grantSource: 'user' | 'group' | 'magic_link' | 'public_mode' | 'system'
  role: 'owner' | 'superadmin' | 'admin' | 'member' | 'guest'
  permMask: number (permission bitmask)
  invitedByAllUserId: string (who invited this user)
  createdAt: timestamp
  updatedAt: timestamp
}
```

## Implemented Service Functions

### Memory Sharing (`src/services/memory/memory-sharing-operations.ts`)

#### `shareMemoryWithUser(params: ShareMemoryParams)`

- Shares a memory with a user
- Updates memory's `sharedCount` and `sharingStatus`
- Returns the created share record

#### `getMemoryShares(memoryId: string)`

- Gets all shares for a specific memory
- Returns array of share records

#### `checkMemoryAccess(params: MemoryAccessCheckParams)`

- Checks if user has access to a memory
- Returns: `'owner' | 'shared' | 'public' | null`
- Handles ownership, public access, and shared access

### Folder Sharing (`src/services/folder/folder-operations.ts`)

#### `shareFolderWithUser(params: ShareFolderParams)`

- Shares a folder with a user
- Creates resource membership record
- Returns the created share record

#### `getFolderShares(folderId: string)`

- Gets all shares for a specific folder
- Returns array of share records

#### `checkFolderAccess(params: FolderAccessCheckParams)`

- Checks if user has access to a folder
- Returns: `'owner' | 'shared' | null`
- Handles ownership and shared access

### Gallery Sharing (Already Implemented)

#### `shareGalleryWithUser(params: ShareGalleryParams)`

- Shares a gallery with a user
- Updates gallery's `sharedCount` and `sharingStatus`

#### `getGalleryShares(galleryId: string)`

- Gets all shares for a specific gallery

#### `checkGalleryAccess(params: GalleryAccessCheckParams)`

- Checks if user has access to a gallery
- Returns: `'owner' | 'shared' | null`

## Access Control Logic

### Memory Access

1. **Owner**: User who created the memory
2. **Public**: Memory marked as public
3. **Shared**: User has explicit share record
4. **Gallery Access**: Access through gallery membership (inherited)

### Folder Access

1. **Owner**: User who created the folder
2. **Shared**: User has explicit share record

### Gallery Access

1. **Owner**: User who created the gallery
2. **Shared**: User has explicit share record
3. **Memory Access**: Memories inherit access through gallery membership

## Usage Examples

### Share a Memory

```typescript
import { shareMemoryWithUser } from "@/services/memory";

const result = await shareMemoryWithUser({
  memoryId: "memory-123",
  allUserId: "user-456",
  grantSource: "user",
  role: "member",
  invitedByAllUserId: "current-user-id",
});
```

### Check Memory Access

```typescript
import { checkMemoryAccess } from "@/services/memory";

const access = await checkMemoryAccess({
  memoryId: "memory-123",
  userId: "user-456",
});
// Returns: 'owner' | 'shared' | 'public' | null
```

### Share a Folder

```typescript
import { shareFolderWithUser } from "@/services/folder";

const result = await shareFolderWithUser({
  folderId: "folder-123",
  allUserId: "user-456",
  grantSource: "user",
  role: "member",
  invitedByAllUserId: "current-user-id",
});
```

## Benefits

### 1. **Unified System**

- Single table for all resource types
- Consistent sharing logic across memories, galleries, and folders
- Easy to extend for new resource types

### 2. **Efficient Access Control**

- Gallery sharing = automatic memory access
- Container-level permissions
- No permission duplication

### 3. **Scalable Architecture**

- Works with any number of resources
- Efficient database queries
- Clean separation of concerns

### 4. **Flexible Permissions**

- Role-based access control
- Multiple grant sources
- Permission bitmasks for fine-grained control

## File Structure

```
src/services/
├── memory/
│   ├── memory-operations.ts          # Core memory operations
│   ├── memory-sharing-operations.ts  # Memory sharing functions
│   ├── types.ts                      # Memory type definitions
│   └── index.ts                      # Exports
├── gallery/
│   ├── gallery-operations.ts         # Gallery operations + sharing
│   └── index.ts                      # Exports
└── folder/
    ├── folder-operations.ts          # Folder operations + sharing
    └── index.ts                      # Exports
```

## Next Steps

1. **API Endpoints**: Create REST endpoints for sharing operations
2. **Frontend Integration**: Connect sharing functions to UI components
3. **Testing**: Add comprehensive tests for all sharing functions
4. **Documentation**: Update API documentation with sharing endpoints
5. **Migration**: Migrate existing sharing data to new system

## Status

✅ **Memory Sharing**: Implemented  
✅ **Folder Sharing**: Implemented  
✅ **Gallery Sharing**: Already implemented  
✅ **Service Functions**: Complete  
⏳ **API Endpoints**: Pending  
⏳ **Frontend Integration**: Pending  
⏳ **Testing**: Pending
