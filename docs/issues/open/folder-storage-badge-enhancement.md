# Folder Storage Badge Enhancement

**Priority**: Medium  
**Type**: Feature Enhancement  
**Status**: Open  
**Created**: 2025-01-16  
**Related**: Storage status display, folder management

## 📋 **Issue Summary**

Currently, folders in the dashboard don't show storage badges, while individual memories do. This creates inconsistent UX and makes it hard for users to understand where their organized content is stored. Folders should display storage badges that reflect the storage status of their contents.

## 🎯 **Current Behavior**

- **Individual memories**: Show storage badges (ICP/NEON/ICP\*)
- **Folders**: No storage badge displayed
- **User impact**: Can't see storage status of organized content at a glance

## 🎯 **Desired Behavior**

- **Individual memories**: Continue showing storage badges
- **Folders**: Show computed storage badges based on contents
- **User benefit**: Clear visibility into where organized content is stored

## 🔧 **Implementation Plan**

### **Phase 1: Data Structure Updates**

1. **Update FolderItem interface** to include storage summary:

   ```typescript
   interface FolderItem {
     // ... existing properties
     storageSummary?: {
       storageLocations: string[]; // Array of storage locations: ['icp'], ['neon'], ['icp', 'neon']
     };
   }
   ```

2. **Backend computation** (if needed):
   - Add folder storage summary to `processDashboardItems()` function
   - Query storage status for all memories in each folder
   - Compute aggregated storage statistics

### **Phase 2: Frontend Updates**

3. **Update renderStorageBadge function**:

   ```typescript
   function renderStorageBadge(item: FlexibleItem) {
     if ("type" in item && item.type && item.type !== "folder") {
       // Existing memory logic
       return <MemoryStorageBadge memoryId={item.id} memoryType={item.type} size="xs" />;
     }

     if ("type" in item && item.type === "folder") {
       // New folder logic
       return <FolderStorageBadge storageSummary={item.storageSummary} size="xs" />;
     }

     return null;
   }
   ```

4. **Create FolderStorageBadge component**:

   ```typescript
   function FolderStorageBadge({ storageSummary, size }: { storageSummary: StorageSummary; size: string }) {
     if (!storageSummary?.storageLocations?.length) return null;

     const { storageLocations } = storageSummary;

     // Simple logic: show badge for each storage location
     return (
       <div className="flex gap-1">
         {storageLocations.map((location) => (
           <Badge key={location} size={size}>
             {location.toUpperCase()}
           </Badge>
         ))}
       </div>
     );
   }
   ```

### **Phase 3: Storage Status Computation**

5. **Update processDashboardItems function**:

   ```typescript
   const folderItems: FolderItem[] = Object.entries(folderGroups).map(([folderId, folderMemories]) => {
     // Compute storage summary from existing memory data (using NEW array approach)
     const allStorageLocations = new Set<string>();

     folderMemories.forEach((memory) => {
       const locations = memory.storageStatus?.storageLocations || [];
       locations.forEach((location) => allStorageLocations.add(location));
     });

     return {
       // ... existing properties
       storageSummary: {
         storageLocations: Array.from(allStorageLocations),
       },
       memories: folderMemories,
     };
   });
   ```

## 🎨 **Badge Display Logic (Simplified)**

| Condition            | Badge          | Tooltip                         |
| -------------------- | -------------- | ------------------------------- |
| All memories on ICP  | `ICP`          | "All memories on ICP"           |
| All memories on Neon | `NEON`         | "All memories on Neon"          |
| Mixed storage        | `ICP` + `NEON` | "Memories on both ICP and Neon" |

**Simple approach:**

- Show **1 badge** if all memories are in same storage
- Show **2 badges** if memories are in different storages
- No colors, no complex states - just clear text badges

## 📊 **Example Scenarios**

### **Scenario 1: All ICP**

- Folder has 5 memories, all on ICP
- Badge: **ICP**
- Tooltip: "All memories on ICP"

### **Scenario 2: All Neon**

- Folder has 3 memories, all on Neon
- Badge: **NEON**
- Tooltip: "All memories on Neon"

### **Scenario 3: Mixed Storage**

- Folder has 2 ICP memories, 3 Neon memories
- Badges: **ICP** + **NEON**
- Tooltip: "Memories on both ICP and Neon"

## 🚨 **CRITICAL ISSUE: Confusing Storage Status Names**

**Problem**: The current storage status values use confusing and inconsistent naming that makes the code hard to understand and maintain.

### **Current Bad Names:**

- `stored_forever` - What does "forever" mean?
- `web2_only` - Why not just "neon"?
- `partially_stored` - Partially where?

### **Files Using Bad Names:**

1. **`src/nextjs/src/hooks/use-memory-storage-status.ts`** (line 4):

   ```typescript
   export type MemoryStorageStatus = "stored_forever" | "partially_stored" | "web2_only" | "loading" | "error";
   ```

2. **`src/nextjs/src/app/api/memories/[id]/route.ts`** (line 17):

   ```typescript
   let overallStatus: "stored_forever" | "partially_stored" | "web2_only";
   ```

3. **`src/nextjs/src/app/api/galleries/utils.ts`** (line 10):
   ```typescript
   status: "stored_forever" | "partially_stored" | "web2_only";
   ```

### **Proposed Better Design:**

The database **already supports multiple storage locations** via the `storageEdges` table! We should use this existing structure:

**Database Structure (Already Exists):**

```sql
-- storageEdges table tracks where each memory artifact is stored
CREATE TABLE storage_edges (
  memory_id UUID,
  artifact VARCHAR, -- 'metadata' | 'asset'
  location_metadata VARCHAR, -- 'neon' | 'icp' (for metadata)
  location_asset VARCHAR, -- 's3' | 'vercel_blob' | 'icp' | 'arweave' | 'ipfs' (for assets)
  present BOOLEAN
);
```

**Proposed API Response:**

```typescript
// Instead of confusing single values, return actual storage locations
storageStatus: {
  storageLocations: ['neon', 'icp'], // Based on actual storageEdges data
  // or
  storageLocations: ['s3', 'icp'], // Multiple asset storage locations
}
```

**Benefits:**

- **Uses existing database structure** - no schema changes needed
- **Accurate data** - based on actual storage edges, not assumptions
- **Future-proof** - supports any number of storage locations
- **Clear naming** - `neon`, `icp`, `s3`, `arweave`, etc.

### **Impact:**

- **Code readability**: Current names are confusing
- **Maintenance**: Hard to understand what each status means
- **Consistency**: Inconsistent with database enum values (`neon` | `icp`)
- **Folder badges**: Can't implement properly with confusing names

### **Required Changes:**

1. **Remove `overallStatus` completely** - this confusing field should be deleted
2. **Query storageEdges table** instead of using hardcoded assumptions
3. **Update API functions** to return `storageLocations: string[]` from database
4. **Update type definitions** to remove `overallStatus` and use `storageLocations: string[]`
5. **Update badge components** to handle arrays of storage locations
6. **Remove hardcoded logic** in `addStorageStatusToMemory()` function

### **API Response Changes:**

**BEFORE (Bad):**

```typescript
storageStatus: {
  metaNeon: boolean,
  assetBlob: boolean,
  metaIcp: boolean,
  assetIcp: boolean,
  overallStatus: 'stored_forever' | 'partially_stored' | 'web2_only' // REMOVE THIS
}
```

**AFTER (Good):**

```typescript
storageStatus: {
  storageLocations: string[] // ['neon', 'icp'] or ['s3', 'icp'] etc.
}
```

## 🔧 **Technical Considerations**

### **Performance**

- **Lazy computation**: Only compute when folder is displayed
- **Caching**: Cache storage summaries to avoid repeated API calls
- **Batch queries**: Use existing `useBatchMemoryStorageStatus` hook

### **Data Flow**

1. `processDashboardItems()` groups memories by folder
2. For each folder, query storage status of all contained memories
3. Compute aggregated storage summary
4. Pass summary to `FolderStorageBadge` component

### **Error Handling**

- **Loading state**: Show loading indicator while computing
- **Error state**: Show "?" badge if computation fails
- **Empty folders**: No badge (or "EMPTY" badge)

## 📋 **Implementation Checklist**

### **Phase 1: Fix Storage Status Names (CRITICAL) - ✅ COMPLETED**

**Summary**: Successfully removed confusing `overallStatus` field and replaced with proper `storageLocations: string[]` array that queries the actual `storageEdges` table.

**Files Modified**:

- `src/nextjs/src/app/api/memories/[id]/route.ts` - Updated `addStorageStatusToMemory()` to query `storageEdges` table
- `src/nextjs/src/hooks/use-memory-storage-status.ts` - Updated types and logic to use `storageLocations` array
- `src/nextjs/src/app/api/galleries/utils.ts` - Updated gallery storage status to use new format
- `src/nextjs/src/components/common/memory-storage-badge.tsx` - Updated badge component to handle multiple storage locations
- `src/nextjs/src/components/galleries/gallery-storage-summary.tsx` - Updated to work with new storage summary format

**Key Changes**:

- [x] **Remove `overallStatus` field** from all API responses and type definitions
- [x] **Query storageEdges table** to get actual storage locations per memory
- [x] **Update API functions** to return `storageLocations: string[]` from database
- [x] **Remove hardcoded assumptions** in `addStorageStatusToMemory()` function
- [x] **Update type definitions** to remove `overallStatus` and use `storageLocations: string[]`
- [x] **Update badge components** to handle multiple storage locations
- [x] **Test with real storageEdges data**

**Result**: Storage badges now show accurate data from the database instead of hardcoded assumptions. Badges can display multiple storage locations (e.g., "ICP+NEON") and are future-proof for new storage providers.

### **Phase 2: Data Structure - ✅ COMPLETED**

**Summary**: Successfully created `FolderStorageBadge` component and updated `renderStorageBadge` function to handle folders.

**Files Modified**:

- `src/nextjs/src/components/common/content-card.tsx` - Added `FolderStorageBadge` component and updated `renderStorageBadge` function

**Key Changes**:

- [x] **Create `FolderStorageBadge` component** - Simple component that displays multiple storage location badges
- [x] **Update `renderStorageBadge` to handle folders** - Now supports both individual memories and folders
- [x] **Update `MemoryItem` interface** - Added `storageSummary` property with `storageLocations` array
- [x] **Add appropriate styling and colors** - Uses consistent Badge styling with secondary variant

**Result**: The UI components are ready to display folder storage badges. The `renderStorageBadge` function now properly handles both individual memories (using `MemoryStorageBadge`) and folders (using `FolderStorageBadge`).

### **Phase 3: Data Computation - ⏳ IN PROGRESS**

- [ ] Update `processDashboardItems()` to compute storage summaries
- [ ] Test with existing folder data

### **Phase 3: Integration**

- [ ] Test with mixed storage scenarios
- [ ] Verify performance with large folders
- [ ] Update documentation

## 🎯 **Success Criteria**

- [ ] Folders display storage badges consistently with memories
- [ ] Badge accurately reflects storage status of folder contents
- [ ] Performance remains acceptable with large folders
- [ ] Tooltips provide useful detailed information
- [ ] Visual design is consistent with existing storage badges

## 🔗 **Related Issues**

- [Dashboard Memory Display Flow Analysis](./dashboard-memory-display-flow-analysis.md)
- [Memory Storage Badge Implementation](../common/memory-storage-badge.tsx)

---

**Last Updated**: 2025-01-16  
**Status**: Open - Ready for Implementation  
**Priority**: Medium - UX Enhancement
