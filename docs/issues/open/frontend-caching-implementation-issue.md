# Frontend Caching Implementation Issue

**Priority:** HIGH  
**Type:** Performance Optimization  
**Status:** OPEN  
**Created:** 2025-01-17  
**Assignee:** TBD

## 📋 **Summary**

Implement comprehensive caching mechanisms for the frontend to improve performance and user experience when navigating between dashboard, folder views, and image galleries. Currently, the application makes fresh API calls and re-downloads images on every navigation, causing unnecessary delays and poor UX.

## 🔍 **Current State Analysis**

### **Image Display Flows Identified**

#### **1. Dashboard Flow**
- **Dashboard Grid View** (`src/app/[lang]/dashboard/page.tsx`)
  - Uses `ContentCard` component with `_renderPreview()` function
  - Displays thumbnails via `memory.thumbnail` or derived from `assets` array
  - Image sizes: `IMAGE_SIZES.grid` with blur placeholders
  - **No caching**: Fresh API call on every navigation back

#### **2. Folder Flow** 
- **Folder Grid View** (`src/app/[lang]/dashboard/folder/[id]/page.tsx`)
  - Same `ContentCard` component as dashboard
  - Filters memories by `parentFolderId` after fetching all memories
  - **No caching**: Re-fetches all memories then filters client-side

#### **3. Memory Detail Flow**
- **Single Memory View** (`src/app/[lang]/dashboard/[id]/page.tsx`)
  - Full-size image display with `IMAGE_SIZES.lightbox`
  - Uses `memory.url` or `memory.assets` for display
  - **No caching**: Fresh fetch on every navigation

#### **4. Gallery Flows**
- **Gallery Grid View** (`src/app/[lang]/gallery/[id]/page.tsx`)
  - Uses `GalleryPhotoGrid` → `ContentCard` with `contentType="gallery-photo"`
  - Displays gallery items with `IMAGE_SIZES.gallery`
  - **No caching**: Fresh API call to `/api/galleries/[id]`

- **Gallery Preview/Lightbox** (`src/app/[lang]/gallery/[id]/preview/page.tsx`)
  - Full-screen image display with navigation
  - Uses `IMAGE_SIZES.lightbox` for main image
  - Grid thumbnails with `IMAGE_SIZES.gallery`
  - **No caching**: Images re-downloaded on every view

- **Gallery Selection Panel** (`src/components/galleries/gallery-selection-panel.tsx`)
  - Small thumbnails in selection interface
  - Uses `sizes="(max-width: 768px) 50vw, 150px"`
  - **No caching**: Re-renders on every selection change

#### **5. Memory Viewer Component**
- **Shared Memory Display** (`src/components/memory/memory-viewer.tsx`)
  - Used for shared/public memories
  - Uses `primaryAsset.url` with `aspect-video` container
  - **No caching**: Fresh fetch for each memory

### **Problem Areas**

#### 1. **API Data Caching**

- **Dashboard → Memory Detail → Back to Dashboard** = Fresh API call to `/api/memories?page=1`
- **Dashboard → Folder → Back to Dashboard** = Fresh API call to `/api/memories?page=1`
- **Gallery → Gallery Preview → Back to Gallery** = Fresh API call to `/api/galleries/[id]`
- **Every scroll for pagination** = Additional API calls without caching
- **Component unmounting/remounting** = Complete loss of state

#### 2. **Image Caching**

- **Thumbnail Re-downloading**: Dashboard/folder thumbnails re-downloaded on every navigation
- **Full-size Image Re-downloading**: Memory detail and gallery preview images re-fetched
- **Multiple Format Inefficiency**: thumb/display/original assets not optimally cached
- **Gallery Image Re-fetching**: Gallery images re-downloaded on every gallery view
- **No Preloading Strategy**: No predictive loading of next/previous images

#### 3. **State Management Issues**

- Uses `useState` which is lost on component unmount
- No global state management for shared data
- No persistence across route changes
- Gallery selection state lost on navigation

## 🎯 **Expected Behavior**

### **Ideal User Experience**

```
Dashboard → Memory Detail → Back to Dashboard
✅ Instant return (cached data)
✅ Background refresh if stale
✅ Smooth user experience

Dashboard → Folder → Back to Dashboard
✅ Instant return (cached data)
✅ No loading spinners
✅ Seamless navigation

Gallery → Image Detail → Back to Gallery
✅ Instant return (cached images)
✅ No re-downloading of images
✅ Smooth transitions
```

## 🏗️ **Technical Architecture**

### **Relevant Files Structure**

```
src/nextjs/
├── src/
│   ├── app/
│   │   ├── [lang]/
│   │   │   ├── dashboard/
│   │   │   │   ├── page.tsx                    # Main dashboard (no caching)
│   │   │   │   ├── [id]/page.tsx              # Memory detail (no caching)
│   │   │   │   └── folder/[id]/page.tsx       # Folder view (no caching)
│   │   │   └── gallery/
│   │   │       ├── [id]/page.tsx              # Gallery view (no caching)
│   │   │       └── [id]/preview/page.tsx      # Gallery preview (no caching)
│   │   └── api/
│   │       ├── memories/
│   │       │   ├── get.ts                     # API endpoint (no cache headers)
│   │       │   └── [id]/route.ts              # Memory detail API
│   │       └── galleries/
│   │           └── [id]/route.ts              # Gallery API
│   ├── components/
│   │   ├── providers/
│   │   │   └── query-provider.tsx             # React Query setup (basic config)
│   │   ├── memory/
│   │   │   ├── memory-grid.tsx                # Memory grid wrapper
│   │   │   └── memory-viewer.tsx              # Shared memory display (no caching)
│   │   ├── common/
│   │   │   ├── content-card.tsx               # Main image display component
│   │   │   └── base-grid.tsx                  # Grid layout component
│   │   └── galleries/
│   │       ├── gallery-photo-grid.tsx         # Gallery grid component
│   │       ├── gallery-image-modal.tsx        # Image modal (no caching)
│   │       └── gallery-selection-panel.tsx    # Gallery selection (no caching)
│   ├── services/
│   │   └── memories.ts                        # API service layer (no caching)
│   ├── utils/
│   │   └── image-utils.ts                     # Image optimization utilities
│   └── hooks/
│       ├── use-hosting-preferences.ts         # Uses React Query
│       └── use-user-settings.ts               # Uses React Query
```

### **Current Infrastructure**

#### ✅ **Available**

- **React Query**: Already installed and configured (`@tanstack/react-query`)
- **QueryProvider**: Set up with basic 30-second stale time
- **Next.js Image**: Optimized image component with WebP/AVIF support
- **Image Utils**: Asset URL optimization and blur placeholders
- **API Endpoints**: RESTful APIs with pagination support

#### ❌ **Missing**

- **Query-based data fetching**: Dashboard/folder views use plain `useState` + `useEffect`
- **Cache persistence**: No sessionStorage/localStorage integration
- **Request deduplication**: Multiple identical requests not deduplicated
- **Background refetching**: No stale-while-revalidate pattern
- **Cache invalidation**: No smart cache invalidation on mutations

## 🚀 **Implementation Plan**

### **Phase 1: React Query Integration (High Priority)**

#### **1.1 Update QueryProvider Configuration**

**Files:** `src/components/providers/query-provider.tsx`

```typescript
// Current: Basic 30-second stale time
staleTime: 30_000,
refetchOnWindowFocus: true,

// Target: Optimized caching configuration
staleTime: 5 * 60_000,        // 5 minutes
gcTime: 10 * 60_000,          // 10 minutes (v5 name)
refetchOnWindowFocus: false,  // Avoid surprise refetches
refetchOnMount: false,        // Use cache when available
refetchOnReconnect: true,     // Refresh on network reconnect
```

#### **1.2 Convert Dashboard to React Query**

**Files:** `src/app/[lang]/dashboard/page.tsx`

```typescript
// Current: useState + useEffect pattern
const [memories, setMemories] = useState<DashboardItem[]>([]);
const fetchDashboardMemories = useCallback(async () => {
  const result = await fetchMemories(currentPage);
  setMemories((prev) => (currentPage === 1 ? result.memories : [...prev, ...result.memories]));
}, [currentPage]);

// Target: React Query pattern
const {
  data: memories,
  isLoading,
  hasNextPage,
  fetchNextPage,
} = useInfiniteQuery({
  queryKey: ["memories", "dashboard", { userId, lang }],
  queryFn: ({ pageParam = 1 }) => fetchMemories(pageParam),
  getNextPageParam: (lastPage, pages) => (lastPage.hasMore ? pages.length + 1 : undefined),
  staleTime: 5 * 60_000,
});
```

#### **1.3 Convert Folder View to React Query**

**Files:** `src/app/[lang]/dashboard/folder/[id]/page.tsx`

```typescript
// Current: Manual filtering after fetch
const result = await fetchMemories(1);
const folderMemories = result.memories.filter((memory) => memory.parentFolderId === folderId);

// Target: Dedicated folder query
const { data: folderMemories, isLoading } = useQuery({
  queryKey: ["memories", "folder", folderId, { userId, lang }],
  queryFn: () => fetchFolderMemories(folderId),
  staleTime: 5 * 60_000,
});
```

### **Phase 2: Image Caching Strategy (Medium Priority)**

#### **2.1 Multi-Level Image Caching System**

**Files:** `src/utils/image-utils.ts`, `src/components/common/content-card.tsx`

```typescript
// Implement comprehensive image caching
export const imageCache = new Map<string, HTMLImageElement>();
export const imageBlobCache = new Map<string, string>(); // For blob URLs

export const preloadImage = (src: string): Promise<HTMLImageElement> => {
  if (imageCache.has(src)) {
    return Promise.resolve(imageCache.get(src)!);
  }
  
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => {
      imageCache.set(src, img);
      resolve(img);
    };
    img.onerror = reject;
    img.src = src;
  });
};

// Preload multiple image formats for a memory
export const preloadMemoryImages = async (memory: MemoryWithAssets) => {
  const assets = memory.assets || [];
  const preloadPromises = [];
  
  // Preload thumb first (fastest)
  const thumbAsset = assets.find(a => a.assetType === 'thumb');
  if (thumbAsset) preloadPromises.push(preloadImage(thumbAsset.url));
  
  // Then display (medium)
  const displayAsset = assets.find(a => a.assetType === 'display');
  if (displayAsset) preloadPromises.push(preloadImage(displayAsset.url));
  
  // Finally original (slowest, but only if needed)
  const originalAsset = assets.find(a => a.assetType === 'original');
  if (originalAsset && memory.type === 'image') {
    preloadPromises.push(preloadImage(originalAsset.url));
  }
  
  return Promise.allSettled(preloadPromises);
};
```

#### **2.2 Dashboard & Folder Image Caching**

**Files:** `src/app/[lang]/dashboard/page.tsx`, `src/app/[lang]/dashboard/folder/[id]/page.tsx`

```typescript
// Preload dashboard thumbnails
const { data: memories } = useInfiniteQuery({
  queryKey: ["memories", "dashboard", { userId, lang }],
  queryFn: ({ pageParam = 1 }) => fetchMemories(pageParam),
  staleTime: 5 * 60_000,
});

// Preload thumbnails for visible items
useEffect(() => {
  if (memories?.pages) {
    const visibleMemories = memories.pages.flat().slice(0, 20); // First 20 items
    visibleMemories.forEach(memory => {
      if (memory.type === 'image') {
        preloadMemoryImages(memory);
      }
    });
  }
}, [memories]);
```

#### **2.3 Gallery Image Optimization**

**Files:** `src/app/[lang]/gallery/[id]/page.tsx`, `src/components/galleries/gallery-image-modal.tsx`

```typescript
// Gallery with aggressive preloading
const { data: gallery } = useQuery({
  queryKey: ["gallery", galleryId],
  queryFn: () => fetchGallery(galleryId),
  staleTime: 10 * 60_000, // Longer cache for galleries
});

// Preload gallery images in batches
useEffect(() => {
  if (gallery?.items) {
    // Preload first 10 images immediately
    gallery.items.slice(0, 10).forEach((item) => {
      preloadMemoryImages(item.memory);
    });
    
    // Preload next 20 images after a delay
    setTimeout(() => {
      gallery.items.slice(10, 30).forEach((item) => {
        preloadMemoryImages(item.memory);
      });
    }, 1000);
  }
}, [gallery]);
```

#### **2.4 Gallery Preview/Lightbox Caching**

**Files:** `src/app/[lang]/gallery/[id]/preview/page.tsx`

```typescript
// Preload adjacent images in lightbox
useEffect(() => {
  if (gallery?.items && selectedImageIndex !== null) {
    const currentIndex = selectedImageIndex;
    
    // Preload previous image
    if (currentIndex > 0) {
      const prevItem = gallery.items[currentIndex - 1];
      preloadMemoryImages(prevItem.memory);
    }
    
    // Preload next image
    if (currentIndex < gallery.items.length - 1) {
      const nextItem = gallery.items[currentIndex + 1];
      preloadMemoryImages(nextItem.memory);
    }
  }
}, [gallery, selectedImageIndex]);
```

#### **2.5 Memory Detail Caching**

**Files:** `src/app/[lang]/dashboard/[id]/page.tsx`

```typescript
// Cache full-size memory images
const { data: memory } = useQuery({
  queryKey: ["memories", "detail", memoryId],
  queryFn: () => fetchMemory(memoryId),
  staleTime: 15 * 60_000, // Longer cache for detail views
});

// Preload full-size image when memory loads
useEffect(() => {
  if (memory?.type === 'image' && memory.assets) {
    const displayAsset = memory.assets.find(a => a.assetType === 'display');
    const originalAsset = memory.assets.find(a => a.assetType === 'original');
    
    // Preload display version first, then original
    if (displayAsset) preloadImage(displayAsset.url);
    if (originalAsset) preloadImage(originalAsset.url);
  }
}, [memory]);
```

### **Phase 3: Cache Persistence (Low Priority)**

#### **3.1 Session Storage Integration**

**Files:** `src/components/providers/query-provider.tsx`

```typescript
import { PersistQueryClientProvider, createSyncStoragePersister } from "@tanstack/react-query-persist-client";

const persister = createSyncStoragePersister({
  storage: window.sessionStorage,
  key: "futura-cache",
});

export function QueryProvider({ children }: { children: React.ReactNode }) {
  return (
    <PersistQueryClientProvider client={queryClient} persistOptions={{ persister }}>
      {children}
    </PersistQueryClientProvider>
  );
}
```

## 📊 **Performance Impact**

### **Expected Improvements**

#### **User Experience**

- **Navigation Speed**: 1-3 second delay → Instant return
- **Loading States**: Eliminate unnecessary loading spinners
- **Data Flickering**: Smooth transitions without content disappearing
- **Mobile Performance**: Significant improvement on slower connections

#### **Server Load**

- **API Calls**: 60-80% reduction in redundant requests
- **Database Queries**: Reduced load from repeated identical queries
- **Bandwidth**: Lower data usage for returning users

#### **Network Usage**

- **Mobile Data**: Reduced consumption for users on limited plans
- **Battery Life**: Less network activity on mobile devices

## 🔧 **Implementation Details**

### **Query Key Strategy**

```typescript
// Consistent query keys across components
const queryKeys = {
  memories: {
    dashboard: (userId: string, lang: string) => ["memories", "dashboard", { userId, lang }],
    folder: (folderId: string, userId: string, lang: string) => ["memories", "folder", folderId, { userId, lang }],
    detail: (memoryId: string) => ["memories", "detail", memoryId],
  },
  galleries: {
    list: (userId: string, lang: string) => ["galleries", "list", { userId, lang }],
    detail: (galleryId: string) => ["galleries", "detail", galleryId],
  },
};
```

### **Cache Invalidation Strategy**

```typescript
// Invalidate related caches on mutations
const deleteMemoryMutation = useMutation({
  mutationFn: deleteMemory,
  onSuccess: (_, memoryId) => {
    // Invalidate dashboard and folder caches
    queryClient.invalidateQueries({ queryKey: ["memories", "dashboard"] });
    queryClient.invalidateQueries({ queryKey: ["memories", "folder"] });
    // Remove specific memory from cache
    queryClient.removeQueries({ queryKey: ["memories", "detail", memoryId] });
  },
});
```

### **Error Handling & Fallbacks**

```typescript
// Graceful degradation when cache fails
const {
  data: memories,
  error,
  isError,
} = useQuery({
  queryKey: ["memories", "dashboard"],
  queryFn: fetchMemories,
  retry: 2,
  retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  staleTime: 5 * 60_000,
  // Fallback to fresh data if cache is corrupted
  onError: () => {
    queryClient.removeQueries({ queryKey: ["memories", "dashboard"] });
  },
});
```

## 🧪 **Testing Strategy**

### **Unit Tests**

- Query key consistency across components
- Cache invalidation logic
- Image preloading functionality
- Error handling and fallbacks

### **Integration Tests**

- Navigation flow: Dashboard → Detail → Back
- Navigation flow: Dashboard → Folder → Back
- Gallery image loading and caching
- Cache persistence across page reloads

### **Performance Tests**

- Measure API call reduction
- Monitor memory usage with caching
- Test on slow network conditions
- Mobile device performance testing

## 📋 **Acceptance Criteria**

### **Must Have**

- [ ] Dashboard data cached for 5 minutes
- [ ] Folder data cached for 5 minutes
- [ ] Instant navigation back to previously viewed content
- [ ] Background refresh when data is stale
- [ ] Proper cache invalidation on mutations (delete, update)

### **Should Have**

- [ ] Gallery images preloaded and cached
- [ ] Image preloading for next/previous in galleries
- [ ] Cache persistence across page reloads (sessionStorage)
- [ ] Optimistic updates for better UX

### **Could Have**

- [ ] Cache persistence across browser sessions (localStorage)
- [ ] Advanced image caching with service worker
- [ ] Predictive preloading based on user behavior
- [ ] Cache analytics and monitoring

## 🔗 **Related Issues**

- [Dashboard Performance Issue: No Caching of API Results](./dashboard-no-caching-performance-issue.md)
- [Dashboard Caching Implementation Todo](./dashboard-caching-implementation-todo.md)
- [Next.js Image Component Optimization Analysis](../done/nextjs-image-component-optimization-analysis.md)

## 📝 **Notes**

- React Query is already installed and partially configured
- Current QueryProvider has basic setup but not used for main data fetching
- Image optimization is already implemented with Next.js Image component
- Need to maintain backward compatibility with existing mock data system
- Consider implementing cache warming strategies for better initial load performance
