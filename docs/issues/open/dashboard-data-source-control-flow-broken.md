# Dashboard Data Source Control Flow Broken

## Issue Summary

The dashboard is not respecting the data source selection (ICP vs Neon) and is always fetching from Neon database, even when:

- Hosting preferences are set to Web3 only (ICP)
- Dashboard switch is manually set to ICP
- User expects to see ICP memories

## Current Behavior

- Dashboard switch shows "ICP"
- Hosting preferences are Web3-only (ICP backend/database)
- But system still calls `/api/memories?page=1` (Neon endpoint)
- No calls to ICP canister are made
- User sees Neon memories instead of ICP memories

## Expected Behavior

- When data source is set to ICP → call `fetchMemoriesFromICP()`
- When data source is set to Neon → call `fetchMemoriesFromNeon()`
- Dashboard should show memories from the selected data source

## Root Cause Analysis

### 1. Control Flow Issue

The `fetchMemories` function in `src/services/memories.ts` should route based on `dataSource` parameter:

```typescript
export const fetchMemories = async (
  page: number,
  dataSource: "neon" | "icp" = "neon"
): Promise<FetchMemoriesResult> => {
  if (dataSource === "icp") {
    return await fetchMemoriesFromICP(page); // Should call this
  } else {
    return await fetchMemoriesFromNeon(page); // Currently calling this
  }
};
```

### 2. Dashboard Integration Issue

The dashboard page (`src/app/[lang]/dashboard/page.tsx`) should be passing the correct `dataSource` to the query:

```typescript
const {
  data,
  isLoading: isLoadingMemories,
  // ...
} = useInfiniteQuery({
  queryKey: qk.memories.dashboard(userId, params.lang as string, dataSource),
  queryFn: ({ pageParam = 1 }) => fetchMemories(pageParam as number, dataSource),
  // ...
});
```

### 3. Automatic Data Source Selection Issue

The automatic data source selection based on hosting preferences might not be working:

```typescript
const [dataSource, setDataSource] = useState<"neon" | "icp">(() => getRecommendedDataSource(hostingPreferences));
```

## Investigation Needed

### 1. Check Dashboard State

- Is `dataSource` state actually set to `'icp'`?
- Is the automatic selection working correctly?
- Is the manual toggle working?

### 2. Check Query Key

- Is the React Query key including the correct `dataSource`?
- Is the query being invalidated when `dataSource` changes?

### 3. Check API Routing

- Is the `fetchMemories` function receiving the correct `dataSource` parameter?
- Is the routing logic working as expected?

## Evidence from Logs

```
[DEBUG] [be] [] [] Found allUserRecord { userId: '8a875028-17d8-4c40-8b79-60fd712fbd72' }
[DEBUG] [be] [] [] Built whereCondition { ownerId: '8a875028-17d8-4c40-8b79-60fd712fbd72' }
[DEBUG] [be] [] [] Fetching memories with whereCondition
GET /api/memories?page=1 200 in 4671ms
```

This shows:

- Neon database queries are being executed
- `/api/memories` endpoint is being called (Neon endpoint)
- No ICP canister calls are being made

## Impact

- **Critical UX Issue**: Users cannot see their ICP memories
- **Data Source Confusion**: Dashboard shows wrong data source
- **Feature Broken**: ICP integration is not working for memory viewing
- **Trust Issue**: Users expect to see their ICP data when ICP is selected

## Priority

**HIGH** - This breaks the core ICP functionality and user experience.

## Files to Investigate

1. `src/app/[lang]/dashboard/page.tsx` - Dashboard state management
2. `src/services/memories.ts` - Memory fetching logic
3. `src/hooks/use-hosting-preferences.ts` - Automatic data source selection
4. `src/components/dashboard/dashboard-top-bar.tsx` - Manual toggle

## Test Cases

1. Set hosting preferences to Web3-only → Dashboard should auto-select ICP
2. Manually toggle dashboard to ICP → Should fetch from ICP canister
3. Manually toggle dashboard to Neon → Should fetch from Neon database
4. Verify React Query keys change when data source changes
5. Verify API calls go to correct endpoints

## Status

**OPEN** - Needs immediate investigation and fix.
