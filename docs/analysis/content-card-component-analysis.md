# ContentCard Component Analysis

## 📊 **Component Overview**

The `ContentCard` component is the primary card component used throughout the Futura application for displaying memories, gallery photos, and galleries. It provides a consistent interface with action buttons (Edit, Share, Delete) and supports both grid and list view modes.

## 📍 Location and Usage

- **Saved at:** `src/nextjs/src/components/common/content-card.tsx`
- **Used in:**
  - `src/nextjs/src/components/memory/memory-grid.tsx` (dashboard memories)
  - `src/nextjs/src/components/galleries/gallery-photo-grid.tsx` (gallery photos)
  - `src/nextjs/src/components/galleries/gallery-grid.tsx` (galleries)

## 🏗️ **Component Hierarchy**

```
Dashboard Page
    ↓
MemoryGrid Component
    ↓
ContentCard Component
    ↓
BaseCard Component (renders the actual card)
```

## 🎯 **Current Button Layout**

The action buttons are rendered in the footer of each card in this order:

1. **Edit** (Pencil icon) - `onEdit` prop
2. **Share** (Share2 icon) - `onShare` prop
3. **Delete** (Trash2 icon) - `onDelete` prop

### **Button Implementation in BaseCard:**

```typescript
{
  /* Right side - Action buttons */
}
<div className="flex items-center gap-0.5">
  {onEdit && (
    <Button
      variant="ghost"
      size="icon"
      onClick={(e) => {
        e.stopPropagation();
        onEdit(item);
      }}
    >
      <Pencil className="h-4 w-4" />
    </Button>
  )}
  {onShare && (
    <Button
      variant="ghost"
      size="icon"
      onClick={(e) => {
        e.stopPropagation();
        onShare(item);
      }}
    >
      <Share2 className="h-4 w-4" />
    </Button>
  )}
  {onDelete && (
    <Button
      variant="ghost"
      size="icon"
      onClick={(e) => {
        e.stopPropagation();
        onDelete(item);
      }}
    >
      <Trash2 className="h-4 w-4" />
    </Button>
  )}
</div>;
```

## 📍 **Components Using ContentCard**

### **1. MemoryGrid** (`src/components/memory/memory-grid.tsx`)

- **Usage:** Dashboard memory cards
- **Props passed:** `onEdit`, `onShare`, `onDelete`
- **Current order:** Edit → Share → Delete
- **Status:** ✅ Edit button supported, needs to be wired up in dashboard

### **2. GalleryPhotoGrid** (`src/components/galleries/gallery-photo-grid.tsx`)

- **Usage:** Gallery photo cards
- **Props passed:** `onEdit`, `onShare`, `onDelete`
- **Current order:** Edit → Share → Delete
- **Status:** ✅ Fully implemented

### **3. GalleryGrid** (`src/components/galleries/gallery-grid.tsx`)

- **Usage:** Gallery cards
- **Props passed:** `onGalleryEdit`, `onGalleryShare`, `onGalleryDelete`
- **Current order:** Edit → Share → Delete
- **Status:** ✅ Fully implemented

## 🔧 **Current Dashboard Implementation**

### **MemoryGrid Usage in Dashboard:**

```typescript
<MemoryGrid
  memories={filteredMemories}
  onDelete={handleDelete} // ✅ Implemented
  onShare={handleShare} // ✅ Implemented
  onEdit={handleEdit} // ❌ MISSING - needs to be added
  onClick={handleMemoryClick}
  viewMode={viewMode}
  useReactQuery={true}
/>
```

### **Missing Implementation:**

The dashboard page (`src/app/[lang]/dashboard/page.tsx`) is missing:

1. `handleEdit` function
2. `onEdit` prop passed to MemoryGrid

## 🎨 **View Modes Supported**

### **Grid View (Default)**

- Cards displayed in a responsive grid
- Action buttons in footer
- Hover effects and transitions

### **List View**

- Horizontal layout with icon, title, description
- Action buttons on the right side
- Compact design for better space utilization

## 📋 **ContentCard Props Interface**

```typescript
interface ContentCardProps {
  item: FlexibleItem;
  onClick: (item: FlexibleItem) => void;
  onEdit?: (item: FlexibleItem) => void; // ✅ Supported
  onShare?: (item: FlexibleItem) => void; // ✅ Implemented
  onDelete?: (item: FlexibleItem) => void; // ✅ Implemented

  // Selection mode props (for gallery photos)
  selectionMode?: boolean;
  isSelected?: boolean;
  onSelectionToggle?: (checked: boolean) => void;

  // Rating props (for gallery photos)
  rating?: number;
  onRate?: (rating: number) => void;

  // Hide/Unhide props (for gallery photos)
  isHidden?: boolean;
  onHide?: () => void;
  onUnhide?: () => void;

  // Image error handling
  onImageError?: (url: string) => void;

  // View mode
  viewMode?: "grid" | "list";

  // Content type identification
  contentType?: "memory" | "gallery-photo" | "gallery";
}
```

## 🚀 **Required Changes for Dashboard Edit Button**

### **1. Add handleEdit function to dashboard page:**

```typescript
const handleEdit = (id: string) => {
  // Navigate to edit page or open edit modal
  router.push(`/dashboard/${id}/edit`);
};
```

### **2. Pass onEdit prop to MemoryGrid:**

```typescript
<MemoryGrid
  memories={filteredMemories}
  onDelete={handleDelete}
  onShare={handleShare}
  onEdit={handleEdit} // ✅ Add this line
  onClick={handleMemoryClick}
  viewMode={viewMode}
  useReactQuery={true}
/>
```

## 📊 **Usage Statistics**

- **Total files using ContentCard:** 3
- **Components with Edit button:** 2/3 (GalleryPhotoGrid, GalleryGrid)
- **Components missing Edit button:** 1/3 (MemoryGrid in Dashboard)
- **Edit button implementation status:** 66% complete

## 🎯 **Next Steps**

1. **Immediate:** Add `handleEdit` function to dashboard page
2. **Immediate:** Pass `onEdit` prop to MemoryGrid
3. **Future:** Consider implementing edit functionality (modal or dedicated page)
4. **Future:** Ensure consistent edit behavior across all components

## 📁 **Related Files**

- `src/components/common/content-card.tsx` - Main ContentCard component
- `src/components/common/base-card.tsx` - BaseCard component with button layout
- `src/components/memory/memory-grid.tsx` - MemoryGrid using ContentCard
- `src/components/galleries/gallery-photo-grid.tsx` - GalleryPhotoGrid using ContentCard
- `src/components/galleries/gallery-grid.tsx` - GalleryGrid using ContentCard
- `src/app/[lang]/dashboard/page.tsx` - Dashboard page using MemoryGrid

---

**Note:** The edit button functionality is already implemented in the ContentCard component. It only needs to be wired up in the dashboard by adding the `handleEdit` function and passing the `onEdit` prop to MemoryGrid.
