# Frontend Caching Implementation Todo

**Priority:** HIGH  
**Type:** Performance Optimization  
**Status:** READY FOR IMPLEMENTATION  
**Created:** 2025-01-17  
**Assignee:** TBD

## 📋 **Implementation Checklist**

Based on tech lead approval and surgical upgrades from [frontend-caching-implementation-issue.md](./frontend-caching-implementation-issue.md)

### **Phase 1: Core Caching (High Impact, Low Risk)**

#### **1.1 QueryProvider Configuration**

- [ ] **1.1.1** Update `src/components/providers/query-provider.tsx` with v5 configuration
  - [ ] Set `staleTime: 5 * 60_000` (5 minutes)
  - [ ] Set `gcTime: 10 * 60_000` (10 minutes)
  - [ ] Set `refetchOnWindowFocus: false`
  - [ ] Set `refetchOnMount: false`
  - [ ] Set `refetchOnReconnect: true`
- [ ] **1.1.2** Test QueryProvider changes
  - [ ] Verify no breaking changes to existing functionality
  - [ ] Test cache survives route changes

#### **1.2 HTTP Cache Headers (CRITICAL)**

- [ ] **1.2.1** Add cache headers to `src/app/api/memories/get.ts`
  - [ ] Add `Cache-Control: public, max-age=300, stale-while-revalidate=86400`
  - [ ] Add `ETag: etagFrom(data)` (stable hash of payload)
  - [ ] Add `Last-Modified: new Date().toUTCString()`
- [ ] **1.2.2** Add cache headers to `src/app/api/galleries/[id]/route.ts`
  - [ ] Same cache headers as memories API
- [ ] **1.2.3** Add image asset cache headers
  - [ ] **Thumbnails**: `public, max-age=86400, immutable`
  - [ ] **Display**: `public, max-age=3600, stale-while-revalidate=86400`
  - [ ] **Originals**: `public, max-age=0, must-revalidate`
  - [ ] **Versioned URLs**: Append `?v=<asset_hash>` to asset URLs

#### **1.3 Dashboard React Query Conversion**

- [ ] **1.3.1** Convert `src/app/[lang]/dashboard/page.tsx` to useInfiniteQuery
  - [ ] Replace `useState` with `useInfiniteQuery`
  - [ ] Add `placeholderData: keepPreviousData` to prevent flicker
  - [ ] Update query key: `["memories", "dashboard", { userId, lang, filters }]`
  - [ ] Implement proper `getNextPageParam` logic
  - [ ] Flatten pages: `(q.data?.pages ?? []).flatMap((p) => p.items)`
- [ ] **1.3.2** Test dashboard navigation
  - [ ] Test Dashboard → Memory Detail → Back to Dashboard (should be instant)
  - [ ] Test pagination with infinite scroll
  - [ ] Test cache invalidation on memory delete

#### **1.4 Folder View React Query Conversion**

- [ ] **1.4.1** Convert `src/app/[lang]/dashboard/folder/[id]/page.tsx` to useQuery
  - [ ] Replace manual filtering with dedicated folder query
  - [ ] Add `select: (r) => r.items` to return only rendered data
  - [ ] Update query key: `["memories", "folder", folderId, { userId, lang }]`
- [ ] **1.4.2** Test folder navigation
  - [ ] Test Dashboard → Folder → Back to Dashboard (should be instant)
  - [ ] Test folder-specific cache invalidation

#### **1.5 Stable ID Strategy Implementation**

- [ ] **1.5.1** Update `src/components/common/content-card.tsx`
  - [ ] Replace URL storage with stable asset ID usage
  - [ ] Implement `useMemo` for URL generation: `generateAssetUrl(memory.thumbnail_asset_id)`
  - [ ] Update `_renderPreview` function to use stable IDs
- [ ] **1.5.2** Update image utility functions
  - [ ] Modify `src/utils/image-utils.ts` to work with asset IDs
  - [ ] Ensure `getOptimalAssetUrl` works with stable IDs

### **Phase 2: Image Optimization (Medium Impact, Medium Risk)**

#### **2.1 Memory-Leak-Free Image Preloading**

- [ ] **2.1.1** Update `src/utils/image-utils.ts`
  - [ ] Replace `imageCache` Map with `imagePreloadCache` Map
  - [ ] Store Promises instead of HTMLImageElement refs
  - [ ] Implement proper cleanup in `preloadImage` function
- [ ] **2.1.2** Implement `preloadMemoryImages` function
  - [ ] Preload thumb first (fastest)
  - [ ] Then display (medium)
  - [ ] Finally original (slowest, only if needed)
  - [ ] Return `Promise.allSettled` for all preload promises

#### **2.2 Dashboard Thumbnail Preloading**

- [ ] **2.2.1** Add thumbnail preloading to dashboard
  - [ ] Preload first 20 visible memories on dashboard load
  - [ ] Only preload for `memory.type === 'image'`
  - [ ] Use `useEffect` to trigger preloading when memories change
- [ ] **2.2.2** Add thumbnail preloading to folder view
  - [ ] Same preloading strategy as dashboard
  - [ ] Preload based on folder-specific memories

#### **2.3 Gallery Image Optimization**

- [ ] **2.3.1** Convert `src/app/[lang]/gallery/[id]/page.tsx` to React Query
  - [ ] Add `useQuery` for gallery data
  - [ ] Set `staleTime: 10 * 60_000` (longer cache for galleries)
  - [ ] Update query key: `["gallery", galleryId]`
- [ ] **2.3.2** Implement gallery image preloading
  - [ ] Preload first 10 images immediately
  - [ ] Preload next 20 images after 1 second delay
  - [ ] Use `preloadMemoryImages` for each gallery item

#### **2.4 Lightbox Adjacent Prefetch**

- [ ] **2.4.1** Update `src/app/[lang]/gallery/[id]/preview/page.tsx`
  - [ ] Add `useEffect` for adjacent image prefetch
  - [ ] Use `<link rel="prefetch" as="image">` for next/previous images
  - [ ] Implement proper cleanup for prefetch links
- [ ] **2.4.2** Test lightbox navigation
  - [ ] Verify smooth transitions between images
  - [ ] Test prefetch effectiveness

#### **2.5 Memory Detail Caching**

- [ ] **2.5.1** Convert `src/app/[lang]/dashboard/[id]/page.tsx` to React Query
  - [ ] Add `useQuery` for memory detail
  - [ ] Set `staleTime: 15 * 60_000` (longer cache for detail views)
  - [ ] Update query key: `["memories", "detail", memoryId]`
- [ ] **2.5.2** Implement full-size image preloading
  - [ ] Preload display version first
  - [ ] Then preload original version
  - [ ] Only for `memory.type === 'image'`

### **Phase 3: Advanced Features (High Impact, High Risk)**

#### **3.1 Service Worker Implementation**

- [ ] **3.1.1** Install and configure Workbox
  - [ ] Add `workbox-webpack-plugin` to Next.js config
  - [ ] Configure service worker generation
- [ ] **3.1.2** Implement caching strategies
  - [ ] **Thumbnails**: `StaleWhileRevalidate` (7 days, 2000 entries)
  - [ ] **Display**: `CacheFirst` (1 day, 500 entries)
  - [ ] **Originals**: `NetworkOnly` (or CacheFirst behind user toggle)
- [ ] **3.1.3** Test service worker functionality
  - [ ] Verify offline image access
  - [ ] Test cache eviction policies

#### **3.2 Navigation State Persistence**

- [ ] **3.2.1** Install and configure Zustand
  - [ ] Add `zustand` package
  - [ ] Create `src/stores/navigation-store.ts`
- [ ] **3.2.2** Implement navigation state management
  - [ ] Store scroll positions by route
  - [ ] Store current pages by route
  - [ ] Store selections by route
- [ ] **3.2.3** Integrate with dashboard and folder views
  - [ ] Save scroll position on navigation
  - [ ] Restore scroll position on return
  - [ ] Save and restore selection state

#### **3.3 SessionStorage Persistence**

- [ ] **3.3.1** Install `@tanstack/react-query-persist-client`
  - [ ] Add package to dependencies
  - [ ] Configure sessionStorage persister
- [ ] **3.3.2** Update QueryProvider with persistence
  - [ ] Add `PersistQueryClientProvider`
  - [ ] Configure persister for dashboard + folder lists only
  - [ ] **Do NOT persist** detail objects with PII
- [ ] **3.3.3** Test persistence behavior
  - [ ] Test cache survives page reload
  - [ ] Verify no sensitive data persistence

#### **3.4 Cache Analytics and Monitoring**

- [ ] **3.4.1** Implement performance metrics tracking
  - [ ] Track "back to dashboard" navigation times
  - [ ] Track API call reduction percentages
  - [ ] Track image cache hit rates
  - [ ] Track memory usage patterns
- [ ] **3.4.2** Add monitoring dashboard
  - [ ] Create performance metrics display
  - [ ] Add cache hit rate visualization
  - [ ] Implement memory usage monitoring

### **Testing & Quality Assurance**

#### **4.1 Unit Tests**

- [ ] **4.1.1** Test React Query implementations
  - [ ] Test query key consistency
  - [ ] Test cache invalidation logic
  - [ ] Test error handling and fallbacks
- [ ] **4.1.2** Test image preloading functions
  - [ ] Test `preloadImage` function
  - [ ] Test `preloadMemoryImages` function
  - [ ] Test memory leak prevention

#### **4.2 Integration Tests**

- [ ] **4.2.1** Test navigation flows
  - [ ] Dashboard → Memory Detail → Back to Dashboard
  - [ ] Dashboard → Folder → Back to Dashboard
  - [ ] Gallery → Gallery Preview → Back to Gallery
- [ ] **4.2.2** Test cache persistence
  - [ ] Test cache survives page reloads
  - [ ] Test cache invalidation on mutations
  - [ ] Test data consistency across views

#### **4.3 Performance Tests**

- [ ] **4.3.1** Measure performance improvements
  - [ ] Measure API call reduction
  - [ ] Monitor memory usage with caching
  - [ ] Test on slow network conditions
- [ ] **4.3.2** Mobile device testing
  - [ ] Test on various mobile devices
  - [ ] Measure battery usage impact
  - [ ] Test offline functionality

### **Deployment & Rollout**

#### **5.1 Feature Flag Implementation**

- [ ] **5.1.1** Add feature flag configuration
  - [ ] Add `NEXT_PUBLIC_CACHE_V1` environment variable
  - [ ] Implement feature flag checks in components
- [ ] **5.1.2** Test feature flag behavior
  - [ ] Test with flag enabled/disabled
  - [ ] Verify graceful fallback behavior

#### **5.2 Staged Rollout**

- [ ] **5.2.1** Week 1: Phase 1 deployment
  - [ ] Deploy QueryProvider + API headers
  - [ ] Deploy dashboard + folder React Query conversion
  - [ ] Monitor performance metrics
- [ ] **5.2.2** Week 2: Phase 2 deployment
  - [ ] Deploy image optimization features
  - [ ] Deploy service worker
  - [ ] Monitor cache hit rates
- [ ] **5.2.3** Week 3: Phase 3 deployment
  - [ ] Deploy advanced features
  - [ ] Deploy sessionStorage persistence
  - [ ] Full feature rollout
- [ ] **5.2.4** Week 4: Optimization
  - [ ] Remove feature flag
  - [ ] Optimize based on metrics
  - [ ] Document lessons learned

### **Success Criteria Verification**

#### **6.1 Performance Targets**

- [ ] **6.1.1** Navigation Speed
  - [ ] Median "back to dashboard" time < 100ms ✅
  - [ ] 60-80% drop in `/api/memories` requests per session ✅
- [ ] **6.1.2** Image Performance
  - [ ] Image cache hit rate > 85% for thumbnails ✅
  - [ ] Image cache hit rate > 60% for display images ✅
- [ ] **6.1.3** Memory Usage
  - [ ] Memory usage stable: no growth after 10 mins of gallery navigation ✅

#### **6.2 User Experience**

- [ ] **6.2.1** Navigation Smoothness
  - [ ] No loading spinners on return navigation
  - [ ] Smooth transitions between views
  - [ ] No content flickering
- [ ] **6.2.2** Mobile Performance
  - [ ] Improved mobile navigation speed
  - [ ] Reduced battery usage
  - [ ] Better offline experience

## 📊 **Progress Tracking**

### **Overall Progress**

- [ ] Phase 1: Core Caching (0/5 tasks completed)
- [ ] Phase 2: Image Optimization (0/5 tasks completed)
- [ ] Phase 3: Advanced Features (0/4 tasks completed)
- [ ] Testing & QA (0/3 tasks completed)
- [ ] Deployment & Rollout (0/2 tasks completed)

### **Current Sprint Focus**

- [ ] **Sprint 1**: Phase 1.1 - QueryProvider Configuration
- [ ] **Sprint 2**: Phase 1.2 - HTTP Cache Headers
- [ ] **Sprint 3**: Phase 1.3 - Dashboard React Query Conversion

## 📝 **Notes**

- All tasks are based on tech lead approved surgical upgrades
- Each task includes specific file paths and implementation details
- Success criteria are measurable and time-bound
- Feature flag approach allows for safe rollout and rollback
- Testing is integrated throughout each phase, not just at the end
