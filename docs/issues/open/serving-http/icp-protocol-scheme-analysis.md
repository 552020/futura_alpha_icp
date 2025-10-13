# ICP Protocol Scheme Analysis: Do We Need `icp://` URLs?

## Problem Statement

Our backend generates asset URLs using a custom `icp://` protocol scheme:

- `icp://memory/{memory_id}/blob/{asset_id}`
- `icp://memory/{memory_id}/inline/{asset_id}`
- `icp://blob/{blob_id}`

These URLs cause browser errors when used in HTML `src` attributes because browsers don't understand the `icp://` protocol scheme.

## Current Implementation

### Backend System (Rust) - Where `icp://` URLs are Generated

**Backend generates `icp://` URLs for internal asset tracking:**

**In `src/backend/src/memories/adapters.rs`:**

```rust
// Line 473 - Blob thumbnails
return Some(format!("icp://memory/{}/blob/{}", self.id, asset.asset_id));

// Line 487 - Inline thumbnails
return Some(format!("icp://memory/{}/inline/{}", self.id, asset.asset_id));

// Line 506 - Blob original assets
return Some(format!("icp://memory/{}/blob/{}", self.id, asset.asset_id));

// Line 520 - Inline original assets
return Some(format!("icp://memory/{}/inline/{}", self.id, asset.asset_id));
```

**In `src/backend/src/lib.rs`:**

```rust
// Line 751 - Blob storage location
storage_location: format!("icp://blob/{}", blob_id),
```

### Frontend System (TypeScript) - Where URLs are Consumed

**Frontend expects browser-compatible URLs for HTML elements:**

**In `src/nextjs/src/lib/storage/providers/icp.ts`:**

```typescript
// Frontend ICP provider returns standard HTTPS URLs
getUrl(key: string): string {
  const canisterId = process.env.ICP_CANISTER_ID;
  const networkUrl = process.env.ICP_NETWORK_URL || 'https://ic0.app';
  return `${networkUrl}/?canisterId=${canisterId}&path=${encodeURIComponent(key)}`;
}
```

**Frontend usage in HTML:**

```html
<!-- This works - standard HTTPS URL -->
<img src="https://ic0.app/?canisterId=abc123&path=image.jpg" />

<!-- This fails - custom icp:// protocol -->
<img src="icp://memory/123/blob/456" />
```

## The Real Question: WHO Chose `icp://` and WHY?

**We need to understand:**

1. **WHO** decided to use `icp://` protocol scheme?
2. **WHEN** was this decision made?
3. **WHY** was `icp://` chosen over other options?
4. **WHAT** was the original intent?

**Current Evidence:**

- `icp://` appears in backend Rust code
- No documentation explaining the choice
- No comments in code explaining rationale
- No comparison with alternative schemes

**The Real Problem:**

- We don't know why `icp://` was chosen
- We don't know if it was intentional or accidental
- We don't know if there was a design decision behind it
- We need to find the original reasoning

## Comparison with Other Storage Backends

### URL Formats Across Storage Providers

**S3 URLs:**

```typescript
// Standard HTTPS URLs
"https://bucket.s3.region.amazonaws.com/path/to/file.jpg";
"https://futura0.s3.eu-central-1.amazonaws.com/uploads/image.jpg";
```

**Vercel Blob URLs:**

```typescript
// Standard HTTPS URLs
"https://blob.vercel-storage.com/beach.jpg";
"https://[hash].public.blob.vercel-storage.com/filename";
```

**IPFS URLs:**

```typescript
// Standard HTTPS URLs through gateway
"https://ipfs.io/ipfs/QmHash...";
"https://gateway.pinata.cloud/ipfs/QmHash...";
```

**Arweave URLs:**

```typescript
// Standard HTTPS URLs
"https://arweave.net/transaction-id";
"https://gateway.irys.xyz/transaction-id";
```

**ICP URLs (Our Custom Scheme):**

```rust
// Custom protocol scheme - NOT browser-compatible
"icp://memory/{memory_id}/blob/{asset_id}"
"icp://memory/{memory_id}/inline/{asset_id}"
"icp://blob/{blob_id}"
```

### Key Finding: ICP is the ONLY Backend Using Custom Protocol

**All other storage backends use standard HTTPS URLs:**

- ✅ **S3**: `https://bucket.s3.region.amazonaws.com/...`
- ✅ **Vercel Blob**: `https://blob.vercel-storage.com/...`
- ✅ **IPFS**: `https://ipfs.io/ipfs/...` (through gateways)
- ✅ **Arweave**: `https://arweave.net/...`
- ❌ **ICP**: `icp://memory/...` (custom protocol)

## Rationale Analysis

### Why We Might Have Chosen `icp://`

**1. Storage Backend Distinction:**

- Our system supports multiple storage backends: `['s3', 'vercel_blob', 'icp', 'arweave', 'ipfs', 'neon']`
- `icp://` clearly indicates the storage backend
- Consistent with other backend-specific URL patterns

**2. Internal Reference System:**

- `icp://` URLs are used for internal asset tracking
- Database storage references
- Storage edge locations
- Not intended for direct browser consumption

**3. Future-Proofing:**

- May have been designed for future ICP-specific features
- Could support ICP-native protocols or gateways
- Allows for ICP-specific URL handling logic

### Documentation Evidence

**From `src/nextjs/src/db/schema.ts`:**

```typescript
/**
 * STORAGE BACKEND - Where assets are actually stored
 *
 * PROVIDERS:
 * - s3: AWS S3 (large files, high performance, enterprise)
 * - vercel_blob: Vercel Blob Storage (medium files, CDN, easy integration)
 * - icp: ICP Canister Storage (decentralized, user preference, Web3)
 * - arweave: Arweave (permanent storage, immutable, pay-once)
 * - ipfs: IPFS (decentralized, content-addressed, peer-to-peer)
 * - neon: Neon database (small files, metadata, fast access)
 */
```

**From `src/nextjs/src/lib/storage/providers/icp.ts`:**

```typescript
/**
 * ICP STORAGE PROVIDER
 *
 * This provider implements the StorageProvider interface for ICP (Internet Computer Protocol).
 * It provides decentralized, Web3-native storage on the Internet Computer.
 */
```

**However, the ICP provider's `getUrl()` method returns standard HTTPS:**

**From `src/nextjs/src/lib/storage/providers/icp.ts` (lines 55-64):**

```typescript
getUrl(key: string): string {
  const canisterId = process.env.ICP_CANISTER_ID;
  const networkUrl = process.env.ICP_NETWORK_URL || 'https://ic0.app';
  return `${networkUrl}/?canisterId=${canisterId}&path=${encodeURIComponent(key)}`;
}
```

### The Contradiction

**We have TWO different URL formats for ICP:**

1. **Internal References**: `icp://memory/{id}/blob/{asset_id}` (custom protocol)
2. **Public URLs**: `https://ic0.app/?canisterId={id}&path={key}` (standard HTTPS)

This suggests the `icp://` scheme was intended for internal use only, but we're accidentally using it for frontend display.

## Questions for Analysis

### 1. Do We Actually Need `icp://` URLs?

**Current Usage:**

- Asset references in memory metadata
- Storage edge locations
- Frontend display URLs

**Alternatives:**

- Use standard HTTPS URLs with our `http_request` implementation
- Use internal IDs without protocol schemes
- Use relative paths or other reference formats

### 2. Why Did We Choose `icp://`?

**Possible Reasons:**

- Distinguish ICP assets from other storage backends (S3, etc.)
- Indicate the storage location/backend
- Provide a consistent URL format across different asset types
- Future-proofing for ICP-specific features

### 3. What Are the Alternatives?

**Option A: Standard HTTPS URLs**

```
https://canister-id.icp0.io/asset/{memory_id}/{variant}?id={asset_id}&token={token}
```

**Option B: Internal Reference IDs**

```
memory:{memory_id}:blob:{asset_id}
blob:{blob_id}
```

**Option C: Relative Paths**

```
/asset/{memory_id}/{variant}?id={asset_id}&token={token}
```

## Impact Analysis

### Current Problems

- ❌ Browser compatibility issues
- ❌ Next.js Image component errors
- ❌ Cannot use directly in HTML `src` attributes
- ❌ Requires custom handling/transformation

### Benefits of `icp://` Scheme

- ✅ Clear indication of storage backend
- ✅ Consistent format across asset types
- ✅ Distinguishes from other storage providers
- ✅ Self-documenting (shows it's an ICP resource)

## Recommendation

**Keep `icp://` for internal references, but provide HTTPS URLs for browser use:**

1. **Internal Storage**: Continue using `icp://` for:

   - Database storage references
   - Internal asset tracking
   - Storage edge locations

2. **Browser URLs**: Generate HTTPS URLs for:

   - Frontend display
   - HTML `src` attributes
   - Next.js Image component

3. **URL Transformation**: Create a service that converts:
   ```
   icp://memory/{id}/blob/{asset_id} → https://canister.icp0.io/asset/{id}/original?id={asset_id}&token={token}
   ```

## Implementation Plan

### Phase 1: URL Transformation Service

- Create `AssetUrlService` that converts `icp://` URLs to HTTPS URLs
- Add token generation for private assets
- Update frontend to use transformed URLs

### Phase 2: Dual URL System

- Keep `icp://` for internal references
- Generate HTTPS URLs for browser consumption
- Maintain backward compatibility

### Phase 3: Migration (Optional)

- Consider migrating to HTTPS-only URLs
- Update database schema if needed
- Remove `icp://` generation if no longer needed

## Questions for Team

1. **Do we need to distinguish ICP assets from other storage backends?**
2. **Should we maintain the `icp://` scheme for internal references?**
3. **Is URL transformation the right approach, or should we change the generation?**
4. **Are there other systems that depend on the `icp://` format?**

## Related Issues

- [Private Asset Serving Implementation Report](private-asset-serving-implementation-report.md)
- [ICP Image Configuration Issue](icp-image-configuration-nextjs.md)
