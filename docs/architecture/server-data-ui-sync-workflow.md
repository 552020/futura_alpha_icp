# Server Data and UI Sync Workflow Analysis

_This document maps the current data handling patterns and UI synchronization workflow in the Futura application._

## Overview

The Futura application uses a hybrid data architecture combining:

- **React Query** for client-side state management and caching
- **Next.js API routes** for server-side data operations
- **Dual storage backends** (Neon database + ICP canisters)
- **Optimistic updates** for immediate UI feedback
- **Cross-tab synchronization** via BroadcastChannel and localStorage

## 1. Data Lifecycle

### Data Fetching Patterns

**Current Approach:**

- **React Query** handles all server data fetching with intelligent caching
- **Query keys** are structured hierarchically: `['memories', 'dashboard', userId, lang, dataSource]`
- **Infinite queries** for paginated data (memories, galleries, feed)
- **Regular queries** for user settings, hosting preferences, and single resources

**Caching Strategy:**

```typescript
// Global React Query configuration
defaultOptions: {
  queries: {
    staleTime: 5 * 60_000, // 5 minutes
    gcTime: 10 * 60_000, // 10 minutes
    refetchOnWindowFocus: false,
    refetchOnMount: false,
    refetchOnReconnect: true,
    retry: 2,
  },
}
```

**Data Reuse:**

- ✅ **Memory-based caching**: Data persists across component mounts and route changes
- ✅ **Cross-component sharing**: Same API data available in multiple UI components simultaneously
- ✅ **Session persistence**: User settings and hosting preferences cached for entire session (`staleTime: Infinity`)

### Data Sources

**Dual Backend Architecture:**

1. **Neon Database** (Web2) - Primary for metadata and user data
2. **ICP Canisters** (Web3) - Decentralized storage and capsules
3. **Hybrid Mode** - Both systems synchronized

**Data Source Selection:**

```typescript
// Automatic data source selection based on user preferences
const dataSource = getRecommendedDashboardDataSource(preferences);
const queryKey = qk.memories.dashboard(userId, lang, dataSource);
```

## 2. Freshness and Synchronization

### Automatic Updates

**Current Implementation:**

- ✅ **Window focus**: Disabled (`refetchOnWindowFocus: false`) for performance
- ✅ **Network reconnection**: Enabled (`refetchOnReconnect: true`) for reliability
- ✅ **Cross-tab sync**: BroadcastChannel for identity changes
- ✅ **Manual refresh**: Available via React Query DevTools

**Tab Switching Behavior:**

```typescript
// Identity refresh on tab focus
const onFocus = () => refresh();
const onVis = () => document.visibilityState === "visible" && refresh();
window.addEventListener("focus", onFocus);
document.addEventListener("visibilitychange", onVis);
```

### Post-Mutation Updates

**Optimistic Updates:**

- ✅ **Immediate UI feedback** before server confirmation
- ✅ **Rollback on failure** with previous data restoration
- ✅ **Server reconciliation** after successful mutations

**Example - User Settings Update:**

```typescript
onMutate: async newData => {
  await qc.cancelQueries({ queryKey: ['user-settings', userKey] });
  const previousData = qc.getQueryData<UserSettings>(['user-settings', userKey]);

  if (previousData) {
    qc.setQueryData(['user-settings', userKey], {
      ...previousData,
      ...newData,
      updatedAt: new Date().toISOString(),
    });
  }
  return { previousData };
},

onError: (_err, _vars, ctx) => {
  if (ctx?.previousData) qc.setQueryData(['user-settings', userKey], ctx.previousData);
},
```

## 3. Mutations and Optimistic UI

### Optimistic Updates

**Implemented For:**

- ✅ **User settings** - Immediate UI updates with rollback
- ✅ **Hosting preferences** - Instant feedback for storage changes
- ✅ **Memory operations** - Delete, share, edit with optimistic updates

**Not Implemented For:**

- ❌ **Memory creation** - No optimistic UI for uploads
- ❌ **Bulk operations** - No optimistic feedback for batch actions

### Retry and Rollback Logic

**Error Handling:**

```typescript
// Retry configuration
retry: 2, // Global retry for all queries

// Storage upload retry with exponential backoff
for (let attempt = 1; attempt <= this.config.maxRetries; attempt++) {
  try {
    return await provider.upload(file, options);
  } catch (error) {
    if (attempt < this.config.maxRetries) {
      const delay = this.config.retryDelay * Math.pow(2, attempt - 1);
      await this.sleep(delay);
    }
  }
}
```

**Rollback Strategy:**

- ✅ **Previous data restoration** on mutation failure
- ✅ **Query invalidation** for server reconciliation
- ✅ **Error classification** for different retry strategies

## 4. Caching and Performance

### Client-Side Caching

**React Query Cache:**

- ✅ **Memory-based caching** with configurable TTL
- ✅ **Query deduplication** - Multiple components share same data
- ✅ **Background updates** - Stale data served while fetching fresh data
- ✅ **Garbage collection** - Automatic cleanup of unused data

**LocalStorage Persistence:**

- ✅ **Onboarding state** - Form data persisted across sessions
- ✅ **User preferences** - Cached for entire session
- ✅ **Cross-tab sync** - BroadcastChannel for identity changes

### Server-Side Caching

**API Response Caching:**

```typescript
// Cache control headers
const res = await fetch("/api/me/hosting-preferences", {
  cache: "no-store", // Always fetch fresh data
  credentials: "include",
});
```

**Database Query Optimization:**

- ✅ **Optimized queries** with gallery joins
- ✅ **Pagination** for large datasets
- ✅ **Indexed queries** for performance

## 5. Pagination and Infinite Scrolling

### Current Implementation

**Infinite Scroll:**

```typescript
// Dashboard infinite scroll
useEffect(() => {
  const handleScroll = () => {
    if (window.innerHeight + window.scrollY >= document.documentElement.scrollHeight - 100) {
      if (!isFetchingNextPage && hasNextPage && !USE_MOCK_DATA) {
        fetchNextPage();
      }
    }
  };
  window.addEventListener("scroll", handleScroll);
  return () => window.removeEventListener("scroll", handleScroll);
}, [isFetchingNextPage, hasNextPage, fetchNextPage]);
```

**Pagination Support:**

- ✅ **Dashboard memories** - Infinite scroll with React Query
- ✅ **Feed items** - Manual pagination with state management
- ✅ **Shared memories** - Page-based pagination
- ✅ **Gallery items** - Configurable page sizes

### Filtering and Search

**Current State:**

- ✅ **Client-side filtering** - Memory type, date ranges
- ✅ **Server-side pagination** - Efficient data loading
- ❌ **Real-time search** - No debounced search implementation
- ❌ **Advanced filters** - Limited filtering options

## 6. Offline and Slow Network Handling

### Current Capabilities

**Offline Support:**

- ✅ **Cached data display** - React Query serves stale data when offline
- ✅ **LocalStorage persistence** - Critical user data survives offline
- ❌ **Offline queuing** - No action queuing for offline scenarios
- ❌ **Service Worker** - No offline-first architecture

**Network Error Handling:**

```typescript
// Comprehensive error classification
export function classifyIcpError(e: unknown): ClassifiedError {
  // Connection errors
  if (msg.includes("Failed to fetch") || msg.includes("NetworkError")) {
    return { kind: "connection", cause: e };
  }

  // Authentication errors
  if (msg.match(/delegation|expired|signature|invalid/i)) {
    return { kind: "auth", code: "delegation_expired", cause: e };
  }

  // Business logic errors
  if (code && ["NotFound", "InvalidArgument", "Conflict"].includes(code)) {
    return { kind: "business", code, message: errorObj?.message };
  }
}
```

**Retry Logic:**

- ✅ **Exponential backoff** for upload operations
- ✅ **Connection retry** for network failures
- ✅ **Authentication refresh** for expired sessions

## 7. Cross-Component Updates

### Data Synchronization

**React Query Integration:**

- ✅ **Automatic updates** - Components using same query key update automatically
- ✅ **Optimistic updates** - Changes propagate immediately across components
- ✅ **Cache invalidation** - Targeted cache updates for specific data

**Cross-Tab Communication:**

```typescript
// Identity changes broadcast across tabs
const bc = new BroadcastChannel("icp-auth");
bc.onmessage = (e) => e.data?.type === "identity-changed" && refresh();

// Storage-based fallback
const onStorage = (ev: StorageEvent) => ev.key === key && refresh();
window.addEventListener("storage", onStorage);
```

**Component Update Patterns:**

- ✅ **Memory deletion** - Updates dashboard, shared, and feed views
- ✅ **User settings** - Propagates to all components using settings
- ✅ **Hosting preferences** - Updates storage-related components

## 8. Development Experience

### Debugging Tools

**React Query DevTools:**

- ✅ **Query inspection** - View all active queries and their state
- ✅ **Cache exploration** - Inspect cached data and metadata
- ✅ **Mutation tracking** - Monitor ongoing mutations
- ✅ **Performance metrics** - Query timing and success rates

**Custom Debug Features:**

```typescript
// Memory asset debugging
async function debugMemoriesPage(memoriesPage: { items: MemoryHeader[] }): Promise<void> {
  console.log("🔍 [DEBUG] Full memoriesPage object:", memoriesPage);
  console.log("🔍 [DEBUG] Memories page contains", memoriesPage.items.length, "memories");

  // Asset size validation
  const assetSummary = memoriesPage.items.map((header) => ({
    memoryId: header.id,
    display: header.assets.display[0]?.bytes,
    thumbnail: header.assets.thumbnail[0]?.bytes,
  }));
}
```

**Logging System:**

- ✅ **Structured logging** - `fatLogger` with service context
- ✅ **Error tracking** - Comprehensive error classification
- ✅ **Performance monitoring** - Query timing and upload progress
- ✅ **Debug modes** - Environment-based logging levels

### Testing Support

**Mock Data System:**

```typescript
// Demo mode with mock data
const USE_MOCK_DATA = process.env.NEXT_PUBLIC_USE_MOCK_DATA_DASHBOARD === "true";

if (USE_MOCK_DATA) {
  return processDashboardItems(sampleDashboardMemories as MemoryWithFolder[]);
}
```

**Test Utilities:**

- ✅ **Auth bypass testing** - Test authenticated endpoints without NextAuth
- ✅ **Mock data generation** - Scripts for creating sample data
- ✅ **Database testing** - Test database with isolated test data

## Key Findings and Recommendations

### Strengths

1. **Robust caching strategy** with React Query
2. **Optimistic updates** for immediate UI feedback
3. **Comprehensive error handling** with retry logic
4. **Cross-tab synchronization** for identity changes
5. **Dual backend support** with automatic failover

### Areas for Improvement

1. **Offline queuing** - No action queuing for offline scenarios
2. **Real-time search** - No debounced search implementation
3. **Service Worker** - No offline-first architecture
4. **Advanced filtering** - Limited filtering options
5. **Bulk operations** - No optimistic UI for batch actions

### Questions for Senior Developer

1. **Data lifecycle**: Current approach keeps data in memory and reuses it effectively. Is this the desired pattern?

2. **Freshness and synchronization**: Automatic updates are limited to network reconnection. Should we implement more aggressive refresh strategies?

3. **Mutations and optimistic UI**: Optimistic updates work well for settings but not for memory operations. Should we expand optimistic UI coverage?

4. **Caching and performance**: Current caching strategy is effective. Are there specific performance bottlenecks to address?

5. **Pagination**: Infinite scroll works well for dashboard. Should we standardize on infinite scroll across all list views?

6. **Offline handling**: Limited offline support currently. Is offline-first architecture a priority?

7. **Cross-component updates**: React Query handles this well. Are there specific synchronization issues to address?

8. **Dev experience**: Good debugging tools available. Are there additional development features needed?
