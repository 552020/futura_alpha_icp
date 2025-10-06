# Memories Create Function - Code Definition and Implementation

## Overview

The `memories_create` function is the main entry point for creating memories in the ICP backend. It's implemented as a public API function that delegates to a core business logic function.

## Code Structure

### 1. Public API Function

**Location**: `src/backend/src/lib.rs` (lines 265-297)

```rust
fn memories_create(
    capsule_id: types::CapsuleId,
    bytes: Option<Vec<u8>>,
    blob_ref: Option<types::BlobRef>,
    external_location: Option<types::StorageEdgeBlobType>,
    external_storage_key: Option<String>,
    external_url: Option<String>,
    external_size: Option<u64>,
    external_hash: Option<Vec<u8>>,
    asset_metadata: types::AssetMetadata,
    idem: String,
) -> std::result::Result<types::MemoryId, Error> {
    use crate::memories::core::memories_create_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    memories_create_core(
        &env,
        &mut store,
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

**Purpose**:

- Public API entry point for the ICP canister
- Delegates to the core business logic function
- Provides environment and store adapters

### 2. Core Business Logic Function

**Location**: `src/backend/src/memories/core/create.rs` (lines 14-137)

```rust
pub fn memories_create_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    capsule_id: CapsuleId,
    inline_bytes: Option<Vec<u8>>,
    blob_ref: Option<BlobRef>,
    storage_type: Option<StorageEdgeBlobType>,
    external_storage_key: Option<String>,
    external_url: Option<String>,
    _external_size: Option<u64>,
    _external_hash: Option<Vec<u8>>,
    asset_metadata: AssetMetadata,
    idempotency_key: String,
) -> std::result::Result<MemoryId, Error> {
    // Implementation details below...
}
```

**Purpose**:

- Pure business logic for memory creation
- Testable with mock environments and stores
- Handles all the core logic for creating memories

## Implementation Details

### 1. Function Parameters

| Parameter              | Type                          | Purpose                                   |
| ---------------------- | ----------------------------- | ----------------------------------------- |
| `capsule_id`           | `CapsuleId`                   | ID of the capsule to create memory in     |
| `inline_bytes`         | `Option<Vec<u8>>`             | Inline data for small files (≤32KB)       |
| `blob_ref`             | `Option<BlobRef>`             | Reference to existing blob in ICP storage |
| `storage_type`         | `Option<StorageEdgeBlobType>` | External storage type (S3, Vercel, etc.)  |
| `external_storage_key` | `Option<String>`              | Key for external storage                  |
| `external_url`         | `Option<String>`              | URL for external storage                  |
| `external_size`        | `Option<u64>`                 | Size of external asset (unused)           |
| `external_hash`        | `Option<Vec<u8>>`             | Hash of external asset (unused)           |
| `asset_metadata`       | `AssetMetadata`               | Metadata about the asset                  |
| `idempotency_key`      | `String`                      | Key for idempotent operations             |

### 2. Core Implementation Steps

#### Step 1: Permission Check

```rust
let caller = env.caller();
let capsule_access = store
    .get_capsule_for_acl(&capsule_id)
    .ok_or(Error::NotFound)?;

if !capsule_access.can_write(&caller) {
    return Err(Error::Unauthorized);
}
```

#### Step 2: Idempotency Check

```rust
let memory_id = format!("mem_{}", idempotency_key);

if let Some(existing_memory) = store.get_memory(&capsule_id, &memory_id) {
    return Ok(existing_memory.id);
}
```

#### Step 3: Memory Type Determination

```rust
let memory_type = memory_type_from_asset(&asset_metadata);
```

#### Step 4: Metadata Creation

```rust
let base = match &asset_metadata {
    AssetMetadata::Image(meta) => &meta.base,
    AssetMetadata::Video(meta) => &meta.base,
    AssetMetadata::Audio(meta) => &meta.base,
    AssetMetadata::Document(meta) => &meta.base,
    AssetMetadata::Note(meta) => &meta.base,
};

let metadata = MemoryMetadata {
    memory_type,
    title: Some(base.name.clone()),
    description: base.description.clone(),
    content_type: base.mime_type.clone(),
    created_at: now,
    updated_at: now,
    uploaded_at: now,
    // ... other fields
    created_by: Some(caller.to_string()),
    database_storage_edges: vec![],
};
```

#### Step 5: Access Control

```rust
let access = MemoryAccess::Private {
    owner_secure_code: format!("secure_{}", now),
};
```

#### Step 6: Asset Handling

```rust
let mut inline_assets = Vec::new();
let mut blob_internal_assets = Vec::new();
let mut blob_external_assets = Vec::new();

// Handle inline assets
if let Some(bytes) = inline_bytes {
    let inline_asset = MemoryAssetInline {
        bytes,
        metadata: asset_metadata.clone(),
    };
    inline_assets.push(inline_asset);
}

// Handle internal blob assets
if let Some(blob_ref) = blob_ref {
    let internal_asset = MemoryAssetBlobInternal {
        blob_ref,
        metadata: asset_metadata.clone(),
    };
    blob_internal_assets.push(internal_asset);
}

// Handle external assets
if let (Some(storage_type), Some(storage_key)) = (storage_type, external_storage_key) {
    if !is_valid_storage_type(&storage_type) {
        return Err(Error::InvalidArgument("Invalid storage type".to_string()));
    }

    let external_asset = MemoryAssetBlobExternal {
        location: storage_type,
        storage_key,
        url: external_url,
        metadata: asset_metadata,
    };
    blob_external_assets.push(external_asset);
}
```

#### Step 7: Memory Creation and Storage

```rust
let memory = Memory {
    id: memory_id.clone(),
    metadata,
    access,
    inline_assets,
    blob_internal_assets,
    blob_external_assets,
};

store.insert_memory(&capsule_id, memory)?;
Ok(memory_id)
```

## Type Definitions

### Core Types Used

```rust
// From src/backend/src/types.rs
pub type CapsuleId = String;
pub type MemoryId = String;

pub enum AssetMetadata {
    Image(ImageAssetMetadata),
    Video(VideoAssetMetadata),
    Audio(AudioAssetMetadata),
    Document(DocumentAssetMetadata),
    Note(NoteAssetMetadata),
}

pub enum MemoryAccess {
    Private { owner_secure_code: String },
    Custom { groups: Vec<String>, individuals: Vec<PersonRef>, owner_secure_code: String },
    EventTriggered { access: MemoryAccess, trigger_event: AccessEvent, owner_secure_code: String },
    Public { owner_secure_code: String },
    Scheduled { access: MemoryAccess, accessible_after: u64, owner_secure_code: String },
}

pub struct Memory {
    pub id: String,
    pub metadata: MemoryMetadata,
    pub access: MemoryAccess,
    pub inline_assets: Vec<MemoryAssetInline>,
    pub blob_internal_assets: Vec<MemoryAssetBlobInternal>,
    pub blob_external_assets: Vec<MemoryAssetBlobExternal>,
}
```

### Asset Types

```rust
pub struct MemoryAssetInline {
    pub bytes: Vec<u8>,
    pub metadata: AssetMetadata,
}

pub struct MemoryAssetBlobInternal {
    pub blob_ref: BlobRef,
    pub metadata: AssetMetadata,
}

pub struct MemoryAssetBlobExternal {
    pub location: StorageEdgeBlobType,
    pub storage_key: String,
    pub url: Option<String>,
    pub metadata: AssetMetadata,
}
```

## Dependencies

### Imports

```rust
use super::{model_helpers::*, traits::*};
use crate::capsule_acl::CapsuleAcl;
use crate::types::{
    AssetMetadata, BlobRef, CapsuleId, Error, Memory, MemoryAccess, MemoryAssetBlobExternal,
    MemoryAssetBlobInternal, MemoryAssetInline, MemoryId, MemoryMetadata, StorageEdgeBlobType,
};
```

### Traits Required

- `Env`: Provides `caller()` and `now()` methods
- `Store`: Provides memory storage operations

### Helper Functions

- `memory_type_from_asset()`: Determines memory type from asset metadata
- `is_valid_storage_type()`: Validates external storage types

## Error Handling

The function can return the following errors:

- `Error::NotFound`: Capsule not found
- `Error::Unauthorized`: Caller doesn't have write permissions
- `Error::InvalidArgument`: Invalid storage type or other validation errors
- Store operation errors from `insert_memory()`

## Testing

### Unit Tests

**Location**: `src/backend/src/memories/core/create.rs` (lines 139-415)

Two main unit tests:

1. `test_memories_create_core_inline_asset()` - Tests inline asset creation
2. `test_memories_create_core_idempotency()` - Tests idempotency behavior

**Current Status**: ❌ **FAILING** with `CheckSequenceNotMatch` errors

### Integration Tests

**Location**: `src/backend/tests/memories_pocket_ic.rs`

Multiple integration tests using PocketIC framework.

**Current Status**: ❌ **FAILING** with type mismatches

## Candid Interface

The function is exposed via the Candid interface as:

```candid
memories_create : (
    text,                    // capsule_id
    opt blob,               // inline_bytes
    opt BlobRef,            // blob_ref
    opt StorageEdgeBlobType, // storage_type
    opt text,               // external_storage_key
    opt text,               // external_url
    opt nat64,              // external_size
    opt blob,               // external_hash
    AssetMetadata,          // asset_metadata
    text,                   // idempotency_key
) -> Result<MemoryId, Error>
```

## Current Issues

### 1. Unit Test Failures

- **Error**: `CheckSequenceNotMatch`
- **Location**: `src/backend/src/memories/core/create.rs`
- **Tests Affected**: Both unit tests failing

### 2. Integration Test Failures

- **Error**: Type mismatches (`text` vs `principal`)
- **Location**: JavaScript and shell tests
- **Root Cause**: Interface mismatch

### 3. Candid Parsing Issues

- **Error**: Candid syntax errors in shell tests
- **Location**: Shell test scripts
- **Root Cause**: Argument construction problems

## File Structure

```
src/backend/src/
├── lib.rs                          # Public API function
├── memories/
│   ├── core/
│   │   ├── create.rs              # Core implementation + unit tests
│   │   ├── traits.rs              # Env and Store traits
│   │   └── model_helpers.rs       # Helper functions
│   └── types.rs                   # Type definitions
└── types.rs                       # Main type definitions
```

## Conclusion

The `memories_create` function is well-structured with clear separation between the public API and core business logic. However, it's currently experiencing multiple issues across different testing environments, suggesting either:

1. **Backend implementation bugs** (unit test failures)
2. **Interface mismatches** (integration test failures)
3. **Environment issues** (deployment or configuration problems)

The fact that there are existing memories in the capsule proves the function was working before, indicating that something has changed in the backend or environment that's causing these issues.


