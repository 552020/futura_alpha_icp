# Implementation Plan: ICP Memory Title Placeholder Fix - Phase 2

**Status**: `PLANNING` - Ready for Implementation  
**Priority**: `HIGH` - User Experience Issue  
**Assigned**: Backend Developer + Frontend Developer  
**Created**: 2024-12-19  
**Related Issue**: [ICP Memory Title Placeholder Display Issue](./icp-memory-title-placeholder-display-issue.md)

## Overview

This document outlines the implementation plan for **Phase 2: Proper Architecture** - implementing a unified ICP backend API that handles mixed asset types (inline + blob) and properly supports memory titles.

## Problem Summary

The current ICP upload flow creates a mixed asset scenario that doesn't fit either existing API:

- **3 Blob Assets**: original, display, thumb (uploaded to ICP blob storage)
- **1 Inline Asset**: placeholder (stored inline in memory record)
- **No single API** handles both asset types with proper title support

## Solution Architecture

### New Unified API: `memories_create_unified`

```rust
#[ic_cdk::update]
fn memories_create_unified(
    capsule_id: types::CapsuleId,
    memory_metadata: types::MemoryMetadata,  // ← Includes title!
    inline_assets: Vec<types::InlineAssetInput>,    // ← Placeholder data
    blob_assets: Vec<types::InternalBlobAssetInput>, // ← Original, display, thumb
    idem: String,
) -> types::Result20 {
    // Single API call handles:
    // 1. Title from memory_metadata.title
    // 2. Inline placeholder asset
    // 3. Multiple blob assets (original, display, thumb)
    // 4. Proper MemoryHeader generation with correct title
}
```

## Implementation Plan

### Phase 2A: Backend Implementation

#### 1. Use Existing Types

**File**: `src/backend/src/memories/types.rs`

We already have the required types defined in the `Memory` struct:

```rust
// Existing types we'll use:
pub struct MemoryAssetInline {
    pub bytes: Vec<u8>,
    pub metadata: AssetMetadata,
    pub asset_type: AssetType,
}

pub struct MemoryAssetBlobInternal {
    pub blob_ref: BlobRef,
    pub metadata: AssetMetadata,
    pub asset_type: AssetType,
}

pub struct BlobRef {
    pub locator: String,        // canister+key, URL, CID, etc.
    pub hash: Option<[u8; 32]>, // optional integrity hash
    pub len: u64,               // size in bytes
}
```

**No new types needed** - we'll use the existing `MemoryAssetInline` and `MemoryAssetBlobInternal` directly.

#### 2. Implement Core Business Logic

**File**: `src/backend/src/memories/core/create.rs`

```rust
/// Unified memory creation function - handles both inline and blob assets
pub fn memories_create_unified_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    capsule_id: CapsuleId,
    memory_metadata: MemoryMetadata,
    inline_assets: Vec<MemoryAssetInline>,
    blob_assets: Vec<MemoryAssetBlobInternal>,
    _idem: String,
) -> std::result::Result<MemoryId, Error> {
    // 1. ACL Check
    let caller = env.caller();
    let capsule_access = store
        .get_capsule_for_acl(&capsule_id)
        .ok_or(Error::NotFound)?;
    if !capsule_access.can_write(&caller) {
        return Err(Error::Unauthorized);
    }

    // 2. Generate UUID v7 memory ID
    let memory_id = generate_uuid_v7();

    // 3. Check for existing memory (idempotency)
    if let Some(_existing) = store.get_memory(&capsule_id, &memory_id) {
        return Ok(memory_id);
    }

    // 4. Create memory with mixed assets
    let mut memory = Memory {
        id: memory_id.clone(),
        capsule_id: capsule_id.clone(),
        metadata: memory_metadata, // ← Contains the title!
        access: MemoryAccess::Private {
            owner: caller,
            created_at: env.now(),
            expires_at: None,
        },
        inline_assets: Vec::new(),
        blob_internal_assets: Vec::new(),
        blob_external_assets: Vec::new(),
    };

    // 5. Add inline assets (placeholder) - already in correct format
    memory.inline_assets.extend(inline_assets);

    // 6. Add blob assets (original, display, thumb) - already in correct format
    memory.blob_internal_assets.extend(blob_assets);

    // 7. Compute and store dashboard fields
    memory.update_dashboard_fields();

    // 8. Insert memory into store
    store.insert_memory(&capsule_id, memory)?;

    // 9. Verify memory was created
    if store.get_memory(&capsule_id, &memory_id).is_none() {
        return Err(Error::Internal(
            "Post-write readback failed: memory was not persisted".to_string(),
        ));
    }

    Ok(memory_id)
}
```

#### 3. Add Public API Endpoint

**File**: `src/backend/src/lib.rs`

```rust
/// Create memory with mixed asset types (inline + blob)
#[ic_cdk::update]
fn memories_create_unified(
    capsule_id: types::CapsuleId,
    memory_metadata: types::MemoryMetadata,
    inline_assets: Vec<types::MemoryAssetInline>,
    blob_assets: Vec<types::MemoryAssetBlobInternal>,
    idem: String,
) -> types::Result20 {
    use crate::memories::core::memories_create_unified_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    match memories_create_unified_core(
        &env,
        &mut store,
        capsule_id,
        memory_metadata,
        inline_assets,
        blob_assets,
        idem,
    ) {
        Ok(memory_id) => types::Result20::Ok(memory_id),
        Err(error) => types::Result20::Err(error),
    }
}
```

#### 4. Candid Interface Auto-Generation

**File**: `src/backend/backend.did`

The Candid interface will be **automatically generated** from the Rust code when the canister is deployed. No manual updates needed.

**Note**: `MemoryAssetInline` and `MemoryAssetBlobInternal` are already defined in the existing Candid interface, so they'll be available for the new API.

### Phase 2B: Frontend Implementation

#### 1. Regenerate Type Definitions

**File**: `src/nextjs/src/ic/declarations/backend/backend.did.ts`

After backend deployment, regenerate the TypeScript types from the updated Candid interface:

```bash
# Regenerate types from updated Candid interface
dfx generate
```

The new `memories_create_unified` function and its types will be automatically available in the generated TypeScript declarations.

#### 2. Update ICP Upload Service

**File**: `src/nextjs/src/services/upload/icp-with-processing.ts`

```typescript
/**
 * Create ICP memory record with mixed assets using unified API
 */
async function createICPMemoryRecordUnified(
  trackingMemoryId: string,
  blobAssets: Array<{
    blobId: string;
    assetType: "original" | "display" | "thumb";
    size: number;
    hash: string;
    mimeType: string;
  }>,
  placeholderData: {
    dataUrl: string;
    size: number;
    mimeType: string;
  },
  memoryMetadata: MemoryMetadata
): Promise<string> {
  try {
    console.log(`🔗 Creating ICP memory with unified API for tracking ID: ${trackingMemoryId}`);

    // Get authenticated backend actor
    const authClient = await getAuthClient();
    if (!authClient.isAuthenticated()) {
      throw new Error("Please connect your Internet Identity to create ICP memory records");
    }

    const identity = authClient.getIdentity();
    const backend = await backendActor(identity);

    // Get capsule ID
    const capsuleResult = await backend.capsules_read_basic([]);
    if (!("Ok" in capsuleResult)) {
      throw new Error("Failed to get user capsule");
    }
    const capsuleId = capsuleResult.Ok.capsule_id;

    // Prepare inline assets (placeholder) - using existing MemoryAssetInline type
    const inlineAssets: MemoryAssetInline[] = [];
    if (placeholderData) {
      const placeholderBytes = dataURLtoBytes(placeholderData.dataUrl);
      const placeholderAssetMetadata: AssetMetadata = {
        Image: {
          dpi: [],
          color_space: [],
          base: {
            url: [],
            height: [],
            updated_at: BigInt(Date.now()),
            asset_type: { Preview: null } as AssetType,
            sha256: [],
            name: "placeholder",
            storage_key: [],
            tags: [],
            processing_error: [],
            mime_type: placeholderData.mimeType,
            description: [],
            created_at: BigInt(Date.now()),
            deleted_at: [],
            bytes: BigInt(placeholderBytes.length),
            asset_location: [],
            width: [],
            processing_status: [],
            bucket: [],
          },
          exif_data: [],
          compression_ratio: [],
          orientation: [],
        },
      };

      inlineAssets.push({
        bytes: placeholderBytes,
        metadata: placeholderAssetMetadata,
        asset_type: { Preview: null } as AssetType,
      });
    }

    // Prepare blob assets (original, display, thumb) - using existing MemoryAssetBlobInternal type
    const blobAssetsInput: MemoryAssetBlobInternal[] = blobAssets.map((asset) => ({
      blob_ref: {
        locator: asset.blobId,
        hash: null, // TODO: Get from blob store
        len: BigInt(asset.size),
      },
      metadata: {
        Image: {
          dpi: [],
          color_space: [],
          base: {
            url: [],
            height: [],
            updated_at: BigInt(Date.now()),
            asset_type: { Original: null } as AssetType,
            sha256: [],
            name: asset.assetType,
            storage_key: [],
            tags: [],
            processing_error: [],
            mime_type: asset.mimeType,
            description: [],
            created_at: BigInt(Date.now()),
            deleted_at: [],
            bytes: BigInt(asset.size),
            asset_location: [],
            width: [],
            processing_status: [],
            bucket: [],
          },
          exif_data: [],
          compression_ratio: [],
          orientation: [],
        },
      },
      asset_type: { Original: null } as AssetType,
    }));

    // Create memory using unified API
    const result = await backend.memories_create_unified(
      capsuleId,
      memoryMetadata, // ← Contains the extracted title!
      inlineAssets,
      blobAssetsInput,
      trackingMemoryId
    );

    if ("Ok" in result) {
      const icpMemoryId = result.Ok;
      console.log(`✅ ICP memory created with unified API: ${icpMemoryId} for tracking ID: ${trackingMemoryId}`);

      // Create storage edges for each artifact via API
      await createStorageEdgesViaAPI(trackingMemoryId, icpMemoryId, blobAssets, placeholderData);

      return icpMemoryId;
    } else {
      throw new Error(`Failed to create ICP memory: ${JSON.stringify(result.Err)}`);
    }
  } catch (error) {
    console.log(
      "❌ Failed to create ICP memory with unified API:",
      error instanceof Error ? error.message : "Unknown error"
    );
    throw error;
  }
}
```

#### 3. Update Upload Functions

**File**: `src/nextjs/src/services/upload/icp-with-processing.ts`

```typescript
// Replace the createICPMemoryRecordAndEdges call with:
await createICPMemoryRecordUnified(memoryId, blobAssets, placeholderData, memoryMetadata);
```

### Phase 2C: Testing

#### 1. Unit Tests

**File**: `src/backend/tests/memories_unified_test.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::memories::core::create::memories_create_unified_core;
    use crate::memories::test_helpers::*;

    #[test]
    fn test_memories_create_unified_with_mixed_assets() {
        // Test unified API with both inline and blob assets
        let mut store = MockStore::new();
        let env = MockEnv::new();

        let memory_metadata = create_test_memory_metadata();
        let inline_assets = vec![create_test_memory_asset_inline()];
        let blob_assets = vec![create_test_memory_asset_blob_internal()];

        let result = memories_create_unified_core(
            &env,
            &mut store,
            "test_capsule".to_string(),
            memory_metadata,
            inline_assets,
            blob_assets,
            "test_idem".to_string(),
        );

        assert!(result.is_ok());
        let memory_id = result.unwrap();

        // Verify memory was created with correct title
        let memory = store.get_memory("test_capsule", &memory_id).unwrap();
        assert_eq!(memory.metadata.title, Some("test_title".to_string()));
        assert_eq!(memory.inline_assets.len(), 1);
        assert_eq!(memory.blob_internal_assets.len(), 1);
    }
}
```

#### 2. Integration Tests

**File**: `src/backend/tests/memories_pocket_ic.rs`

```rust
#[tokio::test]
async fn test_memories_create_unified_integration() {
    // Test the full API endpoint with PocketIC
    let pic = PocketIcBuilder::new().build();
    let canister_id = setup_test_canister(&pic).await;

    // Test unified memory creation
    let result = call_memories_create_unified(
        &pic,
        canister_id,
        "test_capsule",
        test_memory_metadata(),
        vec![test_inline_asset()],
        vec![test_blob_asset()],
        "test_idem",
    ).await;

    assert!(result.is_ok());
}
```

#### 3. Frontend Tests

**File**: `src/nextjs/src/services/upload/__tests__/icp-with-processing.test.ts`

```typescript
describe("createICPMemoryRecordUnified", () => {
  it("should create memory with mixed assets and correct title", async () => {
    const mockBackend = {
      memories_create_unified: jest.fn().mockResolvedValue({ Ok: "test-memory-id" }),
      capsules_read_basic: jest.fn().mockResolvedValue({ Ok: { capsule_id: "test-capsule" } }),
    };

    const result = await createICPMemoryRecordUnified(
      "test-tracking-id",
      [{ blobId: "blob-1", assetType: "original", size: 1000, hash: "hash1", mimeType: "image/jpeg" }],
      { dataUrl: "data:image/jpeg;base64,...", size: 500, mimeType: "image/jpeg" },
      { title: ["test-title"] /* ... other fields */ }
    );

    expect(result).toBe("test-memory-id");
    expect(mockBackend.memories_create_unified).toHaveBeenCalledWith(
      "test-capsule",
      expect.objectContaining({ title: ["test-title"] }),
      expect.arrayContaining([expect.objectContaining({ bytes: expect.any(Uint8Array) })]),
      expect.arrayContaining([expect.objectContaining({ blob_id: "blob-1" })]),
      "test-tracking-id"
    );
  });
});
```

## Deployment Plan

### Step 1: Backend Deployment

1. Implement new types and core logic
2. Add public API endpoint
3. Run unit tests
4. Deploy to testnet
5. Run integration tests
6. Candid interface auto-generated on deployment

### Step 2: Frontend Integration

1. Regenerate Candid types
2. Update upload service
3. Test with testnet
4. Deploy to staging

### Step 3: Production Deployment

1. Deploy backend to mainnet
2. Deploy frontend to production
3. Monitor for issues
4. Clean up old code (optional)

## Success Criteria

- [ ] ICP memories display correct filenames instead of "placeholder"
- [ ] Single API call creates memory with mixed assets
- [ ] All existing functionality continues to work
- [ ] Performance is equal or better than current implementation
- [ ] Code is maintainable and well-tested

## Rollback Plan

If issues arise:

1. Revert frontend to use existing `memories_create_with_internal_blobs`
2. Convert placeholder to blob asset (Phase 1 approach)
3. Keep new backend API for future use
4. No data loss or corruption

## Timeline

- **Week 1**: Backend implementation and testing
- **Week 2**: Frontend integration and testing
- **Week 3**: Deployment and monitoring

**Total Estimated Time**: 2-3 weeks

## Dependencies

- Backend developer (Rust/ICP)
- Frontend developer (TypeScript/React)
- DevOps for deployment
- QA for testing

## Notes

- This implementation provides the optimal long-term architecture
- The unified API can handle any combination of inline and blob assets
- Future memory types (video, audio, documents) can use the same API
- Maintains backward compatibility with existing APIs
