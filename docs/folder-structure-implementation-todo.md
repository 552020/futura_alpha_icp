# Folder Structure Implementation Todo

## Overview

This document outlines the implementation plan for proper folder structure support in the upload system, transitioning from the old virtual folder approach to the new relational folder design using the `folders` table and `memories.parentFolderId` foreign key.

## Current State

- ✅ **Schema**: Proper relational design with `folders` table and `memories.parentFolderId` FK
- ❌ **Upload Logic**: Missing folder creation and parent-child relationship setup
- ❌ **Display Logic**: Dashboard still expects old `metadata.folderName` approach
- ❌ **Navigation**: No folder-specific pages or navigation

## Implementation Phases

### Phase 1: Folder Upload Logic (Backend)

#### 1.1 Create Folder Entity Before Processing Files

**File**: `src/app/api/memories/post.ts` - `handleFolderUpload()` function

**Current Issue**: Files are processed individually without creating a parent folder

**Required Changes**:

```typescript
// Extract folder name from first file
const folderName = files[0]?.name.split("/")[0] || "Ungrouped";

// Create folder entity first
const folder = await db.insert(folders).values({
  ownerId: allUserId,
  name: folderName,
  parentFolderId: null, // Root level for now
});

// Then process files with parentFolderId
const uploadTasks = files.map((file) =>
  limit(async () => {
    // ... existing validation and upload logic

    const result = await storeInNewDatabase({
      type: memoryType,
      ownerId: allUserId,
      url,
      file: {
        ...file,
        name: file.name.split("/").pop() || file.name, // Clean filename
      },
      parentFolderId: folder.id, // ✅ Link to folder
      metadata: {
        originalPath: file.name, // ✅ Preserve upload path
        uploadedAt: new Date().toISOString(),
        originalName: file.name,
        size: file.size,
        mimeType,
      },
    });
  })
);
```

#### 1.2 Extract Folder Information

**Function**: Create `extractFolderInfo()` utility function

```typescript
function extractFolderInfo(fileName: string): { originalPath: string; folderName: string } {
  const pathParts = fileName.split("/");
  const folderName = pathParts.length > 1 ? pathParts[0] : "Ungrouped";

  return {
    originalPath: fileName,
    folderName: folderName,
  };
}
```

#### 1.3 Update storeInNewDatabase Function

**File**: `src/app/api/memories/utils/memory-database.ts`

**Required Changes**:

- Accept `parentFolderId` parameter
- Pass `parentFolderId` to memory creation
- Ensure `originalPath` is stored in metadata

### Phase 2: Dashboard Display Logic (Frontend)

#### 2.1 Update processDashboardItems Function

**File**: `src/services/memories.ts`

**Current Issue**: Groups by `metadata.folderName` (old approach)

**Required Changes**:

```typescript
export const processDashboardItems = (memories: NormalizedMemory[]): DashboardItem[] => {
  // Group memories by parentFolderId
  const folderGroups = memories.reduce((groups, memory) => {
    const parentFolderId = memory.parentFolderId;
    if (parentFolderId) {
      if (!groups[parentFolderId]) {
        groups[parentFolderId] = [];
      }
      groups[parentFolderId].push(memory);
    }
    return groups;
  }, {} as Record<string, NormalizedMemory[]>);

  // Create FolderItems for each group
  const folderItems: FolderItem[] = Object.entries(folderGroups).map(([folderId, folderMemories]) => ({
    id: `folder-${folderId}`,
    type: "folder" as const,
    title: folderMemories[0]?.folder?.name || "Unknown Folder",
    description: `${folderMemories.length} items`,
    itemCount: folderMemories.length,
    memories: folderMemories,
    folderId: folderId, // Store actual folder ID
    createdAt: folderMemories[0]?.createdAt,
    updatedAt: folderMemories[0]?.updatedAt,
  }));

  return [...folderItems, ...individualMemories];
};
```

#### 2.2 Update Memory Queries

**File**: `src/app/api/memories/route.ts`

**Required Changes**:

- Include folder information in memory queries
- Join with `folders` table to get folder names
- Return folder data with memories

```typescript
// Include folder information in queries
const memories = await db.query.memories.findMany({
  where: eq(memories.ownerId, allUserId),
  with: {
    folder: true, // Include folder information
    assets: true,
  },
});
```

#### 2.3 Update Dashboard Components

**Files**:

- `src/components/dashboard/memory-grid.tsx`
- `src/components/dashboard/memory-card.tsx`

**Required Changes**:

- Handle folder items differently from regular memories
- Show folder icon and item count
- Implement folder click navigation

### Phase 3: Folder Navigation (Frontend)

#### 3.1 Create Folder Page

**File**: `src/app/[lang]/dashboard/folder/[id]/page.tsx`

**Required Changes**:

- Create new page for folder contents
- Fetch memories by `parentFolderId`
- Display folder contents in grid layout
- Show folder breadcrumb navigation

#### 3.2 Update Navigation

**File**: `src/components/dashboard/folder-top-bar.tsx`

**Required Changes**:

- Add folder breadcrumb navigation
- Show current folder name
- Add "Back to Dashboard" link

#### 3.3 Update Memory Card Component

**File**: `src/components/dashboard/memory-card.tsx`

**Required Changes**:

- Show folder context when memory is in a folder
- Add folder indicator/badge
- Handle folder vs. individual memory display

### Phase 4: Folder Management APIs

#### 4.1 Folder Contents API

**File**: `src/app/api/folders/[id]/contents/route.ts`

```typescript
export async function GET(request: NextRequest, { params }: { params: { id: string } }) {
  const folderId = params.id;

  // Get folder information
  const folder = await db.query.folders.findFirst({
    where: eq(folders.id, folderId),
  });

  // Get folder contents
  const contents = await db.query.memories.findMany({
    where: eq(memories.parentFolderId, folderId),
    with: {
      assets: true,
    },
  });

  return NextResponse.json({
    folder,
    contents,
  });
}
```

#### 4.2 Folder Hierarchy API

**File**: `src/app/api/folders/hierarchy/route.ts`

```typescript
export async function GET(request: NextRequest) {
  // Get all folders for user with hierarchy
  const folders = await db.query.folders.findMany({
    where: eq(folders.ownerId, allUserId),
    with: {
      subfolders: true,
      memories: true,
    },
  });

  return NextResponse.json(folders);
}
```

#### 4.3 Folder Operations API

**File**: `src/app/api/folders/[id]/route.ts`

```typescript
// PATCH - Rename folder
export async function PATCH(request: NextRequest, { params }: { params: { id: string } }) {
  const { name } = await request.json();

  const updatedFolder = await db
    .update(folders)
    .set({ name, updatedAt: new Date() })
    .where(eq(folders.id, params.id))
    .returning();

  return NextResponse.json(updatedFolder[0]);
}

// DELETE - Delete folder and contents
export async function DELETE(request: NextRequest, { params }: { params: { id: string } }) {
  // Delete folder (cascade will handle memories)
  await db.delete(folders).where(eq(folders.id, params.id));

  return NextResponse.json({ success: true });
}
```

### Phase 5: Testing and Validation

#### 5.1 Upload Testing

- Test folder upload with nested structure
- Test folder upload with mixed file types
- Test folder upload with special characters in names
- Test folder upload with large number of files

#### 5.2 Display Testing

- Test dashboard with multiple folders
- Test folder navigation and content display
- Test folder with no contents
- Test folder with subfolders

#### 5.3 API Testing

- Test folder creation API
- Test folder contents API
- Test folder operations (rename, delete)
- Test folder hierarchy API

## Database Schema Validation

### Current Schema (Verified ✅)

```sql
-- Folders table
CREATE TABLE folders (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  owner_id TEXT NOT NULL REFERENCES all_user(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  parent_folder_id TEXT REFERENCES folders(id),
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Memories table
CREATE TABLE memories (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  owner_id TEXT NOT NULL REFERENCES all_user(id) ON DELETE CASCADE,
  type memory_type_t NOT NULL,
  title TEXT,
  description TEXT,
  parent_folder_id TEXT REFERENCES folders(id),
  -- ... other fields
);
```

### Required Indexes

```sql
-- Performance indexes for folder queries
CREATE INDEX idx_folders_owner ON folders(owner_id);
CREATE INDEX idx_folders_parent ON folders(parent_folder_id);
CREATE INDEX idx_memories_parent_folder ON memories(parent_folder_id);
CREATE INDEX idx_memories_owner_created ON memories(owner_id, created_at DESC);
```

## Migration Strategy

### Phase 1: Backend Changes

1. Update `handleFolderUpload()` to create folder entities
2. Update `storeInNewDatabase()` to accept `parentFolderId`
3. Test folder upload functionality

### Phase 2: Frontend Changes

1. Update `processDashboardItems()` to use `parentFolderId`
2. Update memory queries to include folder information
3. Test dashboard display

### Phase 3: Navigation

1. Create folder page and navigation
2. Update memory card components
3. Test folder navigation

### Phase 4: APIs

1. Create folder management APIs
2. Test folder operations
3. Add error handling and validation

## Success Criteria

- ✅ Folder upload creates proper folder entity and links files
- ✅ Dashboard displays folders as single items with file counts
- ✅ Clicking folder shows folder contents
- ✅ Folder navigation works correctly
- ✅ Folder operations (rename, delete) work
- ✅ Performance is maintained with proper indexing
- ✅ Backward compatibility with existing memories

## Risk Mitigation

1. **Data Migration**: Existing memories without folders will continue to work
2. **Performance**: Proper indexing ensures fast folder queries
3. **Testing**: Comprehensive testing at each phase
4. **Rollback**: Changes are additive and can be rolled back if needed

## Timeline Estimate

- **Phase 1**: 2-3 days (Backend folder upload logic)
- **Phase 2**: 2-3 days (Dashboard display updates)
- **Phase 3**: 2-3 days (Folder navigation)
- **Phase 4**: 1-2 days (Folder management APIs)
- **Phase 5**: 1-2 days (Testing and validation)

**Total**: 8-13 days for complete implementation
