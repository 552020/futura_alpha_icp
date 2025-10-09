# ICP Upload Client-Side Database Import Error

## Issue Summary

**Error:** `❌ db.ts should NEVER be imported in a client component!`

**Location:** `src/nextjs/src/services/upload/icp-with-processing.ts`

**Impact:** ICP upload flow fails when trying to create storage edges

## Root Cause Analysis

### The Problem

The ICP upload flow is trying to import and use the database directly in a client-side component:

```typescript
// In createStorageEdgesForICPMemory function
const { storageEdges } = await import("@/db/schema");
const { db } = await import("@/db/db"); // ← CLIENT-SIDE DATABASE IMPORT
```

### Why This Happens

1. **Client-Side Execution**: The `icp-with-processing.ts` file runs in the browser
2. **Database Import**: We're importing `@/db/db` which contains server-side database connection code
3. **Next.js Restriction**: Next.js prevents database imports in client components for security reasons

### Error Flow

```
1. User uploads file to ICP
2. File uploads successfully to ICP canister
3. ICP memory record created successfully
4. ❌ FAILS: Trying to create storage edges in Neon database
5. Error: "db.ts should NEVER be imported in a client component!"
```

## Current Implementation Issues

### 1. Architecture Violation

- **Client-side code** is trying to access **server-side database**
- This violates Next.js client/server boundary

### 2. Security Risk

- Database credentials and connection strings would be exposed to client
- Direct database access from browser is a security vulnerability

### 3. Function Location

The problematic function is in `icp-with-processing.ts`:

```typescript
async function createStorageEdgesForICPMemory(
  trackingMemoryId: string,
  icpMemoryId: string,
  blobAssets: Array<{...}>,
  placeholderData: {...}
): Promise<void> {
  // ❌ This runs on client-side but imports server-side database
  const { storageEdges } = await import('@/db/schema');
  const { db } = await import('@/db/db');
  // ... database operations
}
```

## Proposed Solutions

### Option 1: Create API Endpoint (Recommended)

Create a server-side API endpoint to handle storage edge creation:

```typescript
// New API route: /api/icp/storage-edges
export async function POST(request: Request) {
  // Server-side database operations
  // Create storage edges for ICP memory
}
```

### Option 2: Move to Server-Side Upload Flow

Move the entire ICP upload flow to a server-side API route.

### Option 3: Defer Storage Edge Creation

Create storage edges later via a separate server-side process.

## Impact Assessment

### Current State

- ✅ ICP file upload works
- ✅ ICP memory creation works
- ❌ Storage edge creation fails
- ❌ No tracking of where ICP assets are stored

### Business Impact

- **Data Loss Risk**: No record of where ICP assets are stored
- **Incomplete Flow**: Upload appears successful but tracking is broken
- **User Experience**: Upload fails silently after successful ICP operations

## Technical Details

### Files Affected

- `src/nextjs/src/services/upload/icp-with-processing.ts` (main issue)
- `src/nextjs/src/db/db.ts` (imported incorrectly)
- `src/nextjs/src/db/schema.ts` (imported incorrectly)

### Error Location

```typescript
// Line ~915 in icp-with-processing.ts
const { storageEdges } = await import("@/db/schema");
const { db } = await import("@/db/db"); // ← ERROR HERE
```

### Call Stack

```
uploadToICPWithProcessing()
  → createICPMemoryRecordAndEdges()
    → createStorageEdgesForICPMemory()  // ← FAILS HERE
      → import('@/db/db')  // ← CLIENT-SIDE DB IMPORT
```

## Next Steps

1. **Immediate Fix**: Remove database imports from client-side code
2. **Create API Endpoint**: Build server-side storage edge creation
3. **Update Flow**: Call API endpoint instead of direct database access
4. **Test**: Verify complete ICP upload flow works end-to-end

## Priority

**HIGH** - This blocks the complete ICP upload functionality and creates data tracking gaps.

---

**Created:** 2025-10-08  
**Status:** Open  
**Assignee:** TBD  
**Labels:** `bug`, `icp`, `database`, `client-server-boundary`
