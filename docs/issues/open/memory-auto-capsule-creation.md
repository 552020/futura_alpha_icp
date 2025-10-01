# Backend: Auto-Create Capsules in memories_create

## Problem

The backend has inconsistent behavior regarding capsule creation:

- ✅ **Gallery functions** (`galleries_create`, `galleries_create_with_memories`) automatically create capsules if they don't exist
- ❌ **Memory functions** (`memories_create`) return `Error::NotFound` if the capsule doesn't exist

This inconsistency forces the frontend to manage capsule lifecycle manually, adding complexity and potential for errors.

## Current Behavior

### Gallery Functions (✅ Good)

```rust
// In gallery.rs - automatically creates capsule
let capsule = match with_capsule_store(|store| {
    // ... find existing capsule
}) {
    Some(capsule) => Some(capsule),
    None => {
        // No capsule found - create one automatically for first-time users
        match capsules_create(None) {
            Ok(capsule) => Some(capsule),
            Err(e) => return Err(Error::Internal(format!("Failed to create capsule: {e}"))),
        }
    }
};
```

### Memory Functions (❌ Inconsistent)

```rust
// In memories.rs - returns NotFound if capsule doesn't exist
with_capsule_store_mut(|store| {
    store.update_with(&capsule_id, |cap| {
        // ... memory creation logic
    })
})
// If capsule_id doesn't exist, update_with returns Error::NotFound
```

## Impact

### Frontend Complexity

The frontend must handle capsule creation manually:

```typescript
// Current frontend approach - extra complexity
async function getOrCreateCapsuleId(actor: CanisterActor): Promise<string> {
  const capsuleResult = await actor.capsules_read_basic([]);

  if ("Ok" in capsuleResult && capsuleResult.Ok) {
    return capsuleResult.Ok.capsule_id;
  }

  // No capsule found, create one
  const createResult = await actor.capsules_create([]);
  return createResult.Ok.id;
}

// Every upload function needs this
const capsuleId = await getOrCreateCapsuleId(actor);
const result = await uploadFileToICP(file, preferences, onProgress);
```

### User Experience Issues

- Users must understand capsule concepts
- Extra API calls for capsule management
- Potential for capsule-related errors
- Inconsistent behavior across different features

## Proposed Solution

### Make memories_create Auto-Create Capsules

Update the memory creation functions to automatically create capsules when they don't exist, following the same pattern as gallery functions.

#### Option 1: Modify memories_create to Auto-Create (Recommended)

```rust
// In lib.rs - make capsule_id optional
#[ic_cdk::update]
async fn memories_create(
    capsule_id: Option<types::CapsuleId>, // Make optional
    bytes: Option<Vec<u8>>,
    blob_ref: Option<types::BlobRef>,
    external_location: Option<types::StorageEdgeBlobType>,
    external_storage_key: Option<String>,
    external_url: Option<String>,
    external_size: Option<u64>,
    external_hash: Option<Vec<u8>>,
    asset_metadata: types::AssetMetadata,
    idem: String,
) -> types::Result<types::MemoryId> {
    crate::memories::create_memory(
        capsule_id,
        bytes,
        blob_ref,
        external_location,
        external_storage_key,
        external_url,
        external_size,
        external_hash,
        asset_metadata,
        idem,
    )
}
```

```rust
// In memories.rs - add auto-creation logic
pub fn create_memory(
    capsule_id: Option<CapsuleId>, // Make optional
    bytes: Option<Vec<u8>>,
    blob_ref: Option<BlobRef>,
    external_location: Option<StorageEdgeBlobType>,
    external_storage_key: Option<String>,
    external_url: Option<String>,
    external_size: Option<u64>,
    external_hash: Option<Vec<u8>>,
    asset_metadata: AssetMetadata,
    idem: String,
) -> Result<MemoryId> {
    let caller = PersonRef::from_caller();

    // Auto-create capsule if not provided or doesn't exist
    let actual_capsule_id = match capsule_id {
        Some(id) if capsule_exists(&id) => id,
        _ => {
            // Create new capsule for caller (same pattern as galleries)
            match capsules_create(None) {
                Ok(capsule) => capsule.id,
                Err(e) => return Err(Error::Internal(format!("Failed to create capsule: {e}"))),
            }
        }
    };

    // Continue with existing logic using actual_capsule_id
    create_memory_with_capsule(
        actual_capsule_id,
        bytes,
        blob_ref,
        external_location,
        external_storage_key,
        external_url,
        external_size,
        external_hash,
        asset_metadata,
        idem,
    )
}

// Helper function to check if capsule exists
fn capsule_exists(capsule_id: &CapsuleId) -> bool {
    with_capsule_store(|store| store.exists(capsule_id))
}
```

#### Option 2: Create New Function (Alternative)

```rust
// Create a new function that auto-creates capsules
#[ic_cdk::update]
async fn memories_create_auto(
    bytes: Option<Vec<u8>>,
    blob_ref: Option<types::BlobRef>,
    // ... other params
) -> types::Result<types::MemoryId> {
    // Auto-create capsule and call existing function
    let capsule = capsules_create(None)?;
    crate::memories::create_memory(
        Some(capsule.id),
        bytes,
        blob_ref,
        // ... other params
    )
}
```

## Benefits

### 1. Consistent API

- All functions behave the same way
- No need to remember which functions auto-create capsules
- Predictable behavior across the entire API

### 2. Simpler Frontend

```typescript
// After fix - much simpler
export async function uploadFileToICP(file, preferences, onProgress) {
  const actor = await backendActor();
  // No capsule management needed!
  return uploadInlineToICP(file, actor, null, idem, onProgress);
}
```

### 3. Better User Experience

- Users don't need to understand capsules
- Seamless upload experience
- No capsule-related errors
- Follows Web2 patterns (S3, Google Drive, etc.)

### 4. Reduced Complexity

- Fewer API calls
- Less error handling
- Simpler documentation
- Easier testing

## Implementation Plan

### Phase 1: Backend Changes

1. **Update `memories_create`** to make `capsule_id` optional
2. **Add auto-creation logic** following gallery pattern
3. **Update `uploads_begin`** to also auto-create capsules
4. **Add helper functions** for capsule existence checking
5. **Update tests** to cover auto-creation scenarios

### Phase 2: Frontend Simplification

1. **Remove capsule management** from upload functions
2. **Update function signatures** to not require capsule IDs
3. **Simplify error handling**
4. **Update documentation**

### Phase 3: Documentation & Examples

1. **Update API documentation**
2. **Create migration guide**
3. **Update example code**
4. **Add changelog entry**

## Backward Compatibility

### Option 1: Gradual Migration

- Keep existing `memories_create` function
- Add new `memories_create_auto` function
- Deprecate old function gradually
- Frontend can migrate at its own pace

### Option 2: Breaking Change (Recommended for Greenfield)

- Update existing function to auto-create
- Update frontend immediately
- Cleaner API with no legacy functions

## Testing

### Test Cases

1. **Auto-creation**: Call `memories_create` without existing capsule
2. **Existing capsule**: Call `memories_create` with existing capsule ID
3. **Invalid capsule**: Call `memories_create` with non-existent capsule ID
4. **Error handling**: Test capsule creation failures
5. **Authorization**: Ensure only caller can create capsules

### Test Files to Update

- `tests/backend/shared-capsule/memories/test_memories_create.sh`
- `tests/backend/shared-capsule/memories/test_memories_read.sh`
- Add new test cases for auto-creation scenarios

## Priority

**High Priority** - This affects the core user experience and API consistency.

## Related Issues

- Frontend upload service complexity
- Inconsistent backend behavior
- User experience improvements
- API design consistency

## Acceptance Criteria

- [ ] `memories_create` automatically creates capsules when needed
- [ ] `uploads_begin` automatically creates capsules when needed
- [ ] All memory-related functions have consistent behavior
- [ ] Frontend can be simplified to remove capsule management
- [ ] Tests cover auto-creation scenarios
- [ ] Documentation is updated
- [ ] Backward compatibility is maintained (if applicable)

## Notes

This change aligns with modern API design principles where the backend handles infrastructure concerns (like container creation) automatically, allowing the frontend to focus on user experience and business logic.

The gallery functions already demonstrate this pattern works well, so extending it to memory functions is a natural evolution.
