# Client-Server Boundary Violation - Database Import in Client Components

**Priority:** High  
**Type:** Bug  
**Component:** Architecture/Client-Server Boundaries  
**Created:** 2025-01-14  
**Status:** Open

## Problem Description

Client components are importing server-side database services directly, causing build errors when switching from Neon to PostgreSQL driver. The `postgres` package uses Node.js modules (`fs`, `os`) that are not available in browser environments.

## Root Cause

**Import Chain Violation:**

```
Client Component (Browser) → Server Service (Node.js) → Database (Node.js)
     ❌ NOT ALLOWED ❌
```

**Step-by-step breakdown:**

1. **Client Component:** `src/app/[lang]/dashboard/page.tsx`

   - Marked with `'use client'` (runs in browser)
   - **Imports:** `import { fetchMemories } from '@/services/memories'` ❌

2. **Server Service:** `@/services/memories`

   - **Actually calls API:** `fetch('/api/memories?page=${page}')` ✅
   - **But also imports:** `@/services/memory/memory-operations.ts` ❌
   - **Which imports:** `import { db } from '@/db/db'` ❌

3. **Database Connection:** `@/db/db`
   - **Uses:** `postgres` package with Node.js modules (`fs`, `os`)
   - **Problem:** Node.js modules don't exist in browser ❌

**The Real Issue:** Even though `fetchMemories` calls an API endpoint, the **import statement** still pulls in server-side code that uses the database, causing the build error.

**What Should Happen Instead:**

1. **Client Component:** Should call API endpoints
2. **API Route:** Should import and use `@/services/memories`
3. **Services:** Should import `@/db/db` (server-side only)

**Files Involved:**

- `src/app/[lang]/dashboard/page.tsx` - Client component
- `src/services/memory/memory-operations.ts` - Server service with `db` import
- `src/services/memory/asset-operations.ts` - Server service with `db` import

## Error Details

```
Module not found: Can't resolve 'fs'
./node_modules/.pnpm/postgres@3.4.7/node_modules/postgres/src/index.js:2:1
Module not found: Can't resolve 'fs'
  1 | import os from 'os'
> 2 | import fs from 'fs'
    | ^^^^^^^^^^^^^^^^^^^
```

**Import Traces:**

```
Client Component Browser:
  ./src/db/db.ts [Client Component Browser]
  ./src/services/memory/memory-operations.ts [Client Component Browser]
  ./src/app/[lang]/dashboard/page.tsx [Client Component Browser]
```

## Impact

- **Build failures** when using PostgreSQL driver
- **Architecture violation** - client components accessing server-side database
- **Security risk** - database connection exposed to client-side code
- **Performance issues** - database operations in client components

## Proposed Solutions

### Option 1: Refactor Dashboard to Server Component

- Remove `'use client'` from dashboard page
- Move data fetching to server-side
- Use server components for data operations

### Option 2: Create API Routes

- Move database operations to API routes
- Client components call API endpoints
- Maintain proper client-server separation

### Option 3: Use Server Actions

- Create server actions for database operations
- Client components call server actions
- Keep client-side interactivity

## Files Affected

**Client Components (Violating Boundaries):**

- `src/app/[lang]/dashboard/page.tsx`
- `src/hooks/use-file-upload.ts`
- `src/components/memory/item-upload-button.tsx`

**Server Services (Being Imported by Clients):**

- `src/services/memory/memory-operations.ts`
- `src/services/memory/asset-operations.ts`

## Acceptance Criteria

- [ ] No client components import `db.ts` directly or indirectly
- [ ] Database operations only in server-side code
- [ ] Client components use API routes or server actions
- [ ] Build succeeds with PostgreSQL driver
- [ ] Proper client-server boundary separation

## Technical Details

### Current Violation Pattern

```typescript
// ❌ WRONG: Client component importing server service directly
"use client";
import { fetchMemories } from "@/services/memories"; // → imports db.ts
```

### Correct Patterns

```typescript
// ✅ CORRECT: Client component calling API endpoint
"use client";
const { data } = useQuery(["memories"], () => fetch("/api/memories"));

// ✅ CORRECT: API route using server services
// src/app/api/memories/route.ts
import { fetchMemories } from "@/services/memories"; // ✅ Server-side only
export async function GET() {
  const memories = await fetchMemories();
  return Response.json(memories);
}

// ✅ CORRECT: Server component for data fetching
export default async function DashboardPage() {
  const memories = await fetchMemories(); // ✅ Server-side only
  return <DashboardClient memories={memories} />;
}
```

## Next Steps

1. **Audit:** Find all client components importing server services
2. **Refactor:** Move database operations to server-side
3. **Test:** Verify build works with PostgreSQL driver
4. **Document:** Update architecture guidelines

## Related Issues

- Database connection mismatch (Neon vs PostgreSQL)
- Client-server boundary violations
- Build configuration issues
