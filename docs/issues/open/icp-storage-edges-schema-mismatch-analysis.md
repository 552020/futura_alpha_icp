# ICP Storage Edges Schema Mismatch Analysis

## Issue Summary

ICP memories are showing "NEON" storage status instead of "ICP" due to a schema mismatch between the frontend storage edges API and the database schema, plus the ICP backend not reading from storage edges.

## Root Cause Analysis

### 1. Memory Creation on ICP Backend

**Location**: `src/backend/src/memories/adapters.rs:286`

```rust
database_storage_edges: vec![crate::types::StorageEdgeDatabaseType::Icp],
```

- ✅ **CORRECT**: ICP memories are created with `StorageEdgeDatabaseType::Icp` by default
- ✅ **CONFIRMED**: The backend is NOT reading from storage edges database - it uses hardcoded values

### 2. Storage Edges Creation (Frontend)

**Location**: `src/nextjs/src/services/upload/icp-with-processing.ts:817`

```typescript
edges.push({
  memoryId: trackingMemoryId,
  memoryType: "image",
  artifact: "metadata",
  backend: "icp-canister", // ❌ WRONG VALUE
  present: true,
  location: `icp://memory/${icpMemoryId}`,
  // ...
});
```

**Problem**: The frontend is sending `backend: 'icp-canister'`, but this value doesn't exist in the database schema.

### 3. Database Schema Analysis

**Location**: `src/nextjs/src/db/schema.ts:1181-1182`

**Current Schema**:

```typescript
locationMetadata: database_hosting_t('location_metadata'), // 'neon' | 'icp' (for metadata artifacts)
locationAsset: blob_hosting_t('location_asset'), // 's3' | 'vercel_blob' | 'icp' | 'arweave' | 'ipfs' (for asset artifacts)
```

**Enum Definitions**:

```typescript
export const database_hosting_t = pgEnum("database_hosting_t", ["neon", "icp"]);
export const blob_hosting_t = pgEnum("blob_hosting_t", ["s3", "vercel_blob", "icp", "arweave", "ipfs", "neon"]);
```

**Valid Values**:

- `database_hosting_t`: `'neon'` | `'icp'`
- `blob_hosting_t`: `'s3'` | `'vercel_blob'` | `'icp'` | `'arweave'` | `'ipfs'` | `'neon'`

### 4. Storage Edges API Mismatch

**Location**: `src/nextjs/src/app/api/storage/edges/route.ts:66`

**Problem**: The API is trying to insert a `backend` field that doesn't exist in the current schema:

```typescript
const edgeData = {
  memoryId,
  memoryType: memoryType as "image" | "video" | "note" | "document" | "audio",
  artifact: artifact as "metadata" | "asset",
  backend: backend as "neon-db" | "vercel-blob" | "icp-canister", // ❌ FIELD DOESN'T EXIST
  // ...
};
```

**Current Schema Expects**:

- `locationMetadata` (for metadata artifacts): `'neon'` | `'icp'`
- `locationAsset` (for asset artifacts): `'s3'` | `'vercel_blob'` | `'icp'` | `'arweave'` | `'ipfs'` | `'neon'`

## The Complete Problem Chain

1. **Frontend sends wrong value**: `backend: 'icp-canister'` (doesn't exist in schema)
2. **API tries to insert non-existent field**: `backend` field doesn't exist in current schema
3. **Database insert fails silently** or maps incorrectly
4. **ICP backend doesn't read storage edges**: Uses hardcoded `StorageEdgeDatabaseType::Icp`
5. **But somehow memories end up with Neon storage edges**: This suggests the hardcoded value is being overridden somewhere

## Investigation Needed

### A. Check if storage edges are actually being created

- Are the 5 storage edges being successfully inserted into the database?
- What values are actually stored in `locationMetadata` and `locationAsset` fields?

### B. Check if ICP backend is reading storage edges

- The backend currently uses hardcoded values, but maybe there's logic that reads from storage edges
- Check if there's any code that populates `database_storage_edges` from actual storage edges

### C. Check if memories are being bound to Neon after creation

- The binding logic in `resources_bind_neon` adds `StorageEdgeDatabaseType::Neon` to memories
- Check if this is being called automatically somewhere

## Immediate Fixes Needed

### 1. Fix Storage Edges API

Update the storage edges API to use the correct schema:

```typescript
// Instead of:
backend: backend as 'neon-db' | 'vercel-blob' | 'icp-canister'

// Use:
locationMetadata: artifact === 'metadata' ? (backend === 'icp-canister' ? 'icp' : 'neon') : undefined,
locationAsset: artifact === 'asset' ? (backend === 'icp-canister' ? 'icp' : backend) : undefined,
```

### 2. Fix Frontend Storage Edge Creation

Update the frontend to send correct values:

```typescript
// Instead of:
backend: "icp-canister";

// Use:
// For metadata artifacts:
locationMetadata: "icp";
// For asset artifacts:
locationAsset: "icp";
```

### 3. Fix ICP Backend to Read Storage Edges

The backend should read from storage edges database to populate `database_storage_edges` instead of using hardcoded values.

## Status

**Open** - Schema mismatch identified, fixes needed in multiple layers
