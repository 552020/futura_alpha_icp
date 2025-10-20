# React Query Architecture

## Overview

This document explains how React Query (TanStack Query) is used in the Futura application for data fetching, caching, and state management.

## What is React Query?

React Query is a powerful data synchronization library that provides:

- **Server state management** - Handles data fetching, caching, and synchronization
- **Automatic background refetching** - Keeps data fresh automatically
- **Optimistic updates** - Update UI before server confirms changes
- **Error handling** - Built-in retry logic and error states
- **DevTools** - Excellent debugging experience

## Project Setup

### Provider Configuration

```typescript
// src/components/providers/query-provider.tsx
export function QueryProvider({ children }: { children: ReactNode }) {
  const [client] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 5 * 60_000, // 5 min - data stays fresh for 5 minutes
            gcTime: 10 * 60_000, // 10 min - cache garbage collection
            refetchOnWindowFocus: false, // Don't refetch when window regains focus
            refetchOnMount: false, // Don't refetch on component mount
            refetchOnReconnect: true, // Refetch when network reconnects
            retry: 2, // Retry failed requests 2 times
          },
        },
      })
  );

  return (
    <QueryClientProvider client={client}>
      {children}
      <ReactQueryDevtools initialIsOpen={false} />
    </QueryClientProvider>
  );
}
```

### Layout Integration

```typescript
// src/app/[lang]/layout.tsx
export default function RootLayout({ children, params }: RootLayoutProps) {
  return (
    <html lang={params.lang}>
      <body>
        <QueryProvider>
          <SessionProvider>
            {/* Other providers */}
            {children}
          </SessionProvider>
        </QueryProvider>
      </body>
    </html>
  );
}
```

## Core Concepts

### 1. Queries (Data Fetching)

**Purpose**: Fetch and cache server data

```typescript
import { useQuery } from "@tanstack/react-query";

function useMemories() {
  return useQuery({
    queryKey: ["memories"],
    queryFn: async () => {
      const response = await fetch("/api/memories");
      return response.json();
    },
    staleTime: 5 * 60_000, // 5 minutes
  });
}

// Usage in component
function MemoryList() {
  const { data: memories, isLoading, error } = useMemories();

  if (isLoading) return <div>Loading...</div>;
  if (error) return <div>Error: {error.message}</div>;

  return (
    <div>
      {memories?.map((memory) => (
        <div key={memory.id}>{memory.title}</div>
      ))}
    </div>
  );
}
```

### 2. Mutations (Data Updates)

**Purpose**: Create, update, or delete server data

```typescript
import { useMutation, useQueryClient } from "@tanstack/react-query";

function useDeleteMemory() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (memoryId: string) => {
      const response = await fetch(`/api/memories/${memoryId}`, {
        method: "DELETE",
      });
      if (!response.ok) throw new Error("Failed to delete memory");
      return memoryId;
    },
    onSuccess: (memoryId) => {
      // Option 1: Invalidate and refetch
      queryClient.invalidateQueries({ queryKey: ["memories"] });

      // Option 2: Optimistic update (remove from cache immediately)
      queryClient.setQueryData(["memories"], (old: any) => old?.filter((memory: any) => memory.id !== memoryId));
    },
  });
}

// Usage in component
function MemoryCard({ memory }) {
  const deleteMemory = useDeleteMemory();

  const handleDelete = () => {
    deleteMemory.mutate(memory.id);
  };

  return (
    <div>
      <h3>{memory.title}</h3>
      <button onClick={handleDelete} disabled={deleteMemory.isPending}>
        {deleteMemory.isPending ? "Deleting..." : "Delete"}
      </button>
    </div>
  );
}
```

### 3. Query Keys

**Purpose**: Unique identifiers for cached data

```typescript
// src/lib/query-keys.ts
export const qk = {
  memories: {
    all: ["memories"] as const,
    lists: () => [...qk.memories.all, "list"] as const,
    list: (filters: Record<string, any>) => [...qk.memories.lists(), { filters }] as const,
    details: () => [...qk.memories.all, "detail"] as const,
    detail: (id: string) => [...qk.memories.details(), id] as const,
  },
  users: {
    all: ["users"] as const,
    profile: (id: string) => [...qk.users.all, "profile", id] as const,
  },
} as const;

// Usage
const { data } = useQuery({
  queryKey: qk.memories.list({ type: "image" }),
  queryFn: () => fetchMemories({ type: "image" }),
});
```

## Advanced Patterns

### 1. Infinite Queries (Pagination)

```typescript
import { useInfiniteQuery } from "@tanstack/react-query";

function useInfiniteMemories() {
  return useInfiniteQuery({
    queryKey: ["memories", "infinite"],
    queryFn: async ({ pageParam = 0 }) => {
      const response = await fetch(`/api/memories?page=${pageParam}`);
      return response.json();
    },
    getNextPageParam: (lastPage, pages) => {
      return lastPage.hasMore ? pages.length : undefined;
    },
    initialPageParam: 0,
  });
}

// Usage
function InfiniteMemoryList() {
  const { data, fetchNextPage, hasNextPage, isFetchingNextPage } = useInfiniteMemories();

  return (
    <div>
      {data?.pages.map((page, i) => (
        <div key={i}>
          {page.memories.map((memory) => (
            <div key={memory.id}>{memory.title}</div>
          ))}
        </div>
      ))}
      <button onClick={() => fetchNextPage()} disabled={!hasNextPage || isFetchingNextPage}>
        {isFetchingNextPage ? "Loading..." : "Load More"}
      </button>
    </div>
  );
}
```

### 2. Optimistic Updates

```typescript
function useUpdateMemory() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ id, updates }) => {
      const response = await fetch(`/api/memories/${id}`, {
        method: "PATCH",
        body: JSON.stringify(updates),
      });
      return response.json();
    },
    onMutate: async ({ id, updates }) => {
      // Cancel outgoing refetches
      await queryClient.cancelQueries({ queryKey: ["memories"] });

      // Snapshot previous value
      const previousMemories = queryClient.getQueryData(["memories"]);

      // Optimistically update
      queryClient.setQueryData(["memories"], (old: any) =>
        old?.map((memory: any) => (memory.id === id ? { ...memory, ...updates } : memory))
      );

      return { previousMemories };
    },
    onError: (err, variables, context) => {
      // Rollback on error
      if (context?.previousMemories) {
        queryClient.setQueryData(["memories"], context.previousMemories);
      }
    },
    onSettled: () => {
      // Always refetch after error or success
      queryClient.invalidateQueries({ queryKey: ["memories"] });
    },
  });
}
```

### 3. Background Refetching

```typescript
// Automatically refetch when data becomes stale
const { data } = useQuery({
  queryKey: ["memories"],
  queryFn: fetchMemories,
  staleTime: 5 * 60_000, // Consider stale after 5 minutes
  refetchInterval: 30 * 1000, // Refetch every 30 seconds
  refetchIntervalInBackground: true, // Continue refetching in background
});
```

## Best Practices

### 1. Query Key Structure

```typescript
// ✅ Good: Hierarchical and consistent
const queryKeys = {
  memories: {
    all: ["memories"],
    lists: () => [...queryKeys.memories.all, "list"],
    list: (filters) => [...queryKeys.memories.lists(), { filters }],
    details: () => [...queryKeys.memories.all, "detail"],
    detail: (id) => [...queryKeys.memories.details(), id],
  },
};

// ❌ Bad: Inconsistent and hard to invalidate
const badKeys = ["memories", "list", "page1"]; // Hard to invalidate all memories
```

### 2. Error Handling

```typescript
function useMemories() {
  return useQuery({
    queryKey: ["memories"],
    queryFn: fetchMemories,
    retry: (failureCount, error) => {
      // Don't retry on 404 errors
      if (error.status === 404) return false;
      // Retry up to 3 times for other errors
      return failureCount < 3;
    },
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });
}
```

### 3. Loading States

```typescript
function MemoryList() {
  const { data, isLoading, isFetching, isError, error } = useMemories();

  // Show loading spinner on initial load
  if (isLoading) return <Spinner />;

  // Show error state
  if (isError) return <ErrorMessage error={error} />;

  return (
    <div>
      {/* Show subtle indicator for background refetching */}
      {isFetching && <div className="text-sm text-gray-500">Updating...</div>}
      {data?.map((memory) => (
        <MemoryCard key={memory.id} memory={memory} />
      ))}
    </div>
  );
}
```

## Common Patterns in Futura

### 1. Memory Management

```typescript
// Fetch memories with infinite scroll
const useMemories = () => {
  return useInfiniteQuery({
    queryKey: qk.memories.lists(),
    queryFn: ({ pageParam = 0 }) => fetchMemories({ page: pageParam }),
    getNextPageParam: (lastPage) => lastPage.nextPage,
  });
};

// Delete memory with optimistic update
const useDeleteMemory = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: deleteMemory,
    onSuccess: (deletedId) => {
      // Remove from all memory queries
      queryClient.setQueryData(qk.memories.lists(), (old: any) =>
        old?.pages.map((page: any) => ({
          ...page,
          memories: page.memories.filter((m: any) => m.id !== deletedId),
        }))
      );
    },
  });
};
```

### 2. User Settings

```typescript
// Fetch user settings
const useUserSettings = () => {
  return useQuery({
    queryKey: qk.users.settings(),
    queryFn: fetchUserSettings,
    staleTime: 10 * 60_000, // 10 minutes
  });
};

// Update user settings
const useUpdateUserSettings = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: updateUserSettings,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: qk.users.settings() });
    },
  });
};
```

## DevTools

React Query DevTools are automatically included in development:

```typescript
// Already configured in QueryProvider
<ReactQueryDevtools initialIsOpen={false} />
```

**Features:**

- View all queries and their states
- Inspect cache contents
- Trigger refetches manually
- Monitor background updates
- Debug query keys and dependencies

## Migration from SWR

If migrating from SWR:

```typescript
// SWR
const { data, error, mutate } = useSWR("/api/memories", fetcher);

// React Query equivalent
const { data, error, refetch } = useQuery({
  queryKey: ["memories"],
  queryFn: fetcher,
});
```

## Performance Considerations

1. **Stale Time**: Set appropriate stale times to avoid unnecessary refetches
2. **Cache Time**: Keep frequently accessed data in cache longer
3. **Background Refetching**: Use sparingly to avoid excessive API calls
4. **Query Invalidation**: Be specific about which queries to invalidate
5. **Optimistic Updates**: Use for better UX but handle rollbacks properly

## Troubleshooting

### Common Issues

1. **Queries not refetching**: Check query keys and invalidation
2. **Stale data**: Adjust stale time or use refetch
3. **Memory leaks**: Ensure proper cleanup in useEffect
4. **Race conditions**: Use proper query key structure

### Debug Tips

1. Use React Query DevTools
2. Check browser network tab
3. Log query keys and cache contents
4. Monitor background refetching behavior

## Resources

- [React Query Documentation](https://tanstack.com/query/latest)
- [React Query DevTools](https://tanstack.com/query/latest/docs/react/devtools)
- [Query Key Factory Pattern](https://tkdodo.eu/blog/effective-react-query-keys)
- [Common Mistakes](https://tkdodo.eu/blog/common-mistakes-with-react-query)
