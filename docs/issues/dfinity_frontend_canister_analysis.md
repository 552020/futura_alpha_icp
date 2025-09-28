# DFINITY Frontend Canister Library Analysis

## Overview

This document provides a comprehensive analysis of the DFINITY SDK's frontend canister library components, their relationships, and implementation strategies for hybrid storage approaches.

## References

- **DFINITY SDK Repository**: https://github.com/dfinity/sdk
- **Frontend Canister Library**: https://github.com/dfinity/sdk/tree/master/src/canisters/frontend
- **ic-certified-assets**: https://github.com/dfinity/sdk/tree/master/src/canisters/frontend/ic-certified-assets
- **ic-frontend-canister**: https://github.com/dfinity/sdk/tree/master/src/canisters/frontend/ic-frontend-canister
- **ic-asset**: https://github.com/dfinity/sdk/tree/master/src/canisters/frontend/ic-asset
- **icx-asset**: https://github.com/dfinity/sdk/tree/master/src/canisters/frontend/icx-asset

# Frontend Canister Directories

## ic-asset/

Library for manipulating assets in an asset canister. Provides Rust APIs for asset management operations.

## ic-certified-assets/

Rust support for asset certification. Allows any Rust canister to serve certified assets by including this library. Handles asset preservation over canister upgrades.

## ic-frontend-canister/

The implementation of the DFX assets canister in Rust with support for asset certification. This is the actual canister that serves frontend assets.

## icx-asset/

Command line tool to manage an asset storage canister. Provides `sync`, `ls`, and `upload` commands for asset management operations.

# Relationships Between Components

## Dependency Chain:

1. **ic-certified-assets** → Base library for asset certification
2. **ic-frontend-canister** → Uses `ic-certified-assets` as dependency
3. **ic-asset** → Standalone library for asset operations (used by icx-asset)
4. **icx-asset** → Uses `ic-asset` as dependency for CLI operations

## Architecture:

- **ic-certified-assets** and **ic-asset** are foundational libraries
- **ic-frontend-canister** is the actual canister implementation that serves assets
- **icx-asset** is the CLI tool for managing assets on deployed canisters

## Usage Flow:

1. Developers use **icx-asset** CLI to upload/manage assets
2. **icx-asset** uses **ic-asset** library internally
3. Assets are served by **ic-frontend-canister** (which uses **ic-certified-assets**)
4. **ic-certified-assets** provides the certification and upgrade-safe storage

# icx-asset CLI Commands

## sync

Synchronize one or more directories to an asset canister.

- Usage: `icx-asset sync <canister id> <source directory>...`
- Example: `icx-asset --pem ~/.config/dfx/identity/default/identity.pem sync <canister id> src/prj_assets/assets dist/prj_assets`
- **Library Methods**: Uses `create_asset()`, `set_asset_content()`, `delete_asset()` from ic-certified-assets

## ls

List assets in the asset canister.

- **Library Methods**: Uses `list()` from ic-certified-assets

## upload

Upload files or directories to the asset canister.

- Usage: `icx-asset upload [<key>=]<file> [[<key>=]<file> ...]`
- Examples:
  - `icx-asset upload a.txt` (upload single file as /a.txt)
  - `icx-asset upload /b.txt=a.txt` (upload file with different name)
  - `icx-asset upload some-dir` (upload directory contents)
  - `icx-asset upload /=src/<project>/assets` (upload entire directory structure)
- **Library Methods**: Uses `create_asset()`, `set_asset_content()`, `store()` from ic-certified-assets

## Command-to-Library Flow:

1. **icx-asset** CLI → **ic-asset** library → Asset Canister → **ic-certified-assets** methods
2. CLI commands are user-friendly wrappers around certified asset operations
3. All operations ultimately call methods exposed by the ic-certified-assets library

# ic-certified-assets Library API

The **ic-certified-assets** library implements functions that are then exposed by the canister through its Candid interface. Here are the main endpoint categories:

## Asset Management

- `create_asset()` - Create a new asset
- `set_asset_content()` - Set content for an asset
- `unset_asset_content()` - Remove content from an asset
- `delete_asset()` - Delete an asset completely
- `store()` - Single-call asset creation with content
- `clear()` - Clear all assets

## Asset Retrieval

- `get()` - Get asset content (query)
- `get_chunk()` - Get specific chunk of large asset (query)
- `list()` - List all assets (query)
- `get_asset_properties()` - Get asset metadata (query)

## Batch Operations

- `create_batch()` - Create a batch for atomic operations
- `create_chunk()` - Create a chunk within a batch
- `create_chunks()` - Create multiple chunks
- `commit_batch()` - Commit all batch operations atomically
- `propose_commit_batch()` - Propose batch for later commit
- `commit_proposed_batch()` - Commit a proposed batch
- `compute_evidence()` - Compute evidence for batch operations
- `delete_batch()` - Delete an uncommitted batch

## HTTP Interface

- `http_request()` - Handle HTTP requests (query)
- `http_request_streaming_callback()` - Handle streaming responses (query)

## Permissions & Access Control

- `authorize()` - Authorize a principal
- `deauthorize()` - Remove authorization
- `list_authorized()` - List authorized principals
- `grant_permission()` - Grant specific permissions
- `revoke_permission()` - Revoke permissions
- `list_permitted()` - List principals with permissions
- `take_ownership()` - Take ownership of the canister

## Configuration & Validation

- `get_configuration()` - Get canister configuration (query)
- `configure()` - Update canister configuration
- `api_version()` - Get API version (query)
- `certified_tree()` - Get certified asset tree (query)
- `validate_*()` - Various validation functions

## Architecture Note:

The **ic-certified-assets** library provides the implementation, and the canister (like **ic-frontend-canister**) exposes these functions through its Candid interface, making them callable via the Internet Computer's protocol.

# Data Storage Structures

The **ic-certified-assets** library uses several key data structures to store assets:

## Core Storage Structure

```rust
pub struct State {
    assets: HashMap<AssetKey, Asset>,           // Main asset storage
    chunks: HashMap<ChunkId, Chunk>,           // Chunk storage for large files
    batches: HashMap<BatchId, Batch>,          // Batch operations storage
    configuration: Configuration,              // Canister configuration
    commit_principals: BTreeSet<Principal>,    // Permission management
    prepare_principals: BTreeSet<Principal>,   // Permission management
    manage_permissions_principals: BTreeSet<Principal>, // Permission management
    asset_hashes: CertifiedResponses,          // Certification data
}
```

## Asset Structure

```rust
pub struct Asset {
    pub content_type: String,                              // MIME type
    pub encodings: HashMap<String, AssetEncoding>,        // Multiple encodings (gzip, etc.)
    pub max_age: Option<u64>,                             // Cache control
    pub headers: Option<HashMap<String, String>>,         // Custom HTTP headers
    pub is_aliased: Option<bool>,                         // URL aliasing
    pub allow_raw_access: Option<bool>,                   // Raw access permission
}
```

## Asset Encoding Structure

```rust
pub struct AssetEncoding {
    pub modified: Timestamp,                              // Last modified time
    pub content_chunks: Vec<RcBytes>,                     // File content chunks
    pub total_length: usize,                              // Total file size
    pub certified: bool,                                  // Certification status
    pub sha256: [u8; 32],                                // Content hash
    pub certificate_expression: Option<CertificateExpression>, // Certification data
    pub response_hashes: Option<HashMap<u16, [u8; 32]>>, // Response hashes
}
```

## Chunk Structure

```rust
pub struct Chunk {
    pub batch_id: BatchId,    // Associated batch
    pub content: RcBytes,     // Chunk content
}
```

## Batch Structure

```rust
pub struct Batch {
    pub expires_at: Timestamp,                           // Expiration time
    pub commit_batch_arguments: Option<CommitBatchArguments>, // Batch operations
    pub evidence_computation: Option<EvidenceComputation>,    // Evidence for certification
    pub chunk_content_total_size: usize,                 // Total size of chunks
}
```

## Storage Architecture:

- **HashMap<AssetKey, Asset>**: Primary storage for assets by key (URL path)
- **HashMap<ChunkId, Chunk>**: Storage for file chunks (large files split into chunks)
- **HashMap<BatchId, Batch>**: Temporary storage for batch operations
- **BTreeSet<Principal>**: Permission management using sorted sets
- **CertifiedResponses**: Cryptographic certification data for asset integrity

## Key Features:

- **Multi-encoding support**: Assets can have multiple encodings (gzip, brotli, etc.)
- **Chunked storage**: Large files are split into manageable chunks
- **Batch operations**: Atomic operations for multiple asset changes
- **Certification**: Cryptographic verification of asset integrity
- **Permission system**: Fine-grained access control using principals

# Memory Storage Architecture

## Heap vs Stable Memory

The **ic-certified-assets** library uses a **hybrid memory approach**:

### **Runtime Storage (Heap Memory)**

```rust
thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}
```

During normal operation, all assets are stored in **heap memory** using:

- **HashMap<AssetKey, Asset>** - Main asset storage
- **HashMap<ChunkId, Chunk>** - Chunk storage
- **HashMap<BatchId, Batch>** - Batch operations
- **BTreeSet<Principal>** - Permission management

### **Upgrade Persistence (Stable Memory)**

```rust
pub fn pre_upgrade() -> StableStateV2 {
    STATE.with(|s| s.take().into())
}

pub fn post_upgrade(stable_state: StableStateV2, args: Option<AssetCanisterArgs>) {
    with_state_mut(|s| {
        *s = State::from(stable_state);
        // ... restore state
    });
}
```

During canister upgrades, the entire state is:

1. **Serialized** from heap memory into `StableStateV2` structure
2. **Saved** to stable memory via `pre_upgrade()`
3. **Restored** from stable memory via `post_upgrade()`
4. **Deserialized** back into heap memory for runtime operations

### **Stable State Structure**

```rust
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StableStateV2 {
    pub(super) authorized: Vec<Principal>,
    pub(super) permissions: Option<StableStatePermissionsV2>,
    pub(super) stable_assets: HashMap<String, StableAssetV2>,
    pub(super) next_batch_id: Option<u64>,
    pub(super) configuration: Option<StableConfigurationV2>,
}
```

## **Why This Architecture?**

- **Performance**: Heap memory provides fast access during normal operations
- **Persistence**: Stable memory ensures data survives canister upgrades
- **Efficiency**: Only serializes/deserializes during upgrades, not during normal operations
- **Compatibility**: Supports both v1 and v2 stable state formats for migration

## **Memory Lifecycle:**

1. **Normal Operation**: All data in heap memory (fast access)
2. **Upgrade Trigger**: `pre_upgrade()` serializes heap → stable memory
3. **Upgrade Complete**: `post_upgrade()` deserializes stable memory → heap
4. **Resume Operation**: All data back in heap memory (fast access)

# Hybrid Storage Strategy: Heap + Stable Memory

## **The Problem**

The ic-certified-assets library only stores assets in heap memory, which means:

- ✅ Fast access during normal operation
- ❌ All assets are lost during canister upgrades
- ❌ No selective persistence for important assets

## **The Solution: Wrapper Approach**

Create a wrapper around the library that leverages its functionality while adding stable memory persistence:

### **Core Strategy:**

1. **Use library's heap storage** for all operations (chunking, encoding, certification)
2. **Copy important assets** from heap to stable memory for persistence
3. **On retrieval**: Check heap first (fast), then stable memory (slow)
4. **On upgrade**: Important assets survive in stable memory

### **Implementation Pattern:**

```rust
pub struct HybridAssetManager {
    // Library's heap state (use all its features)
    heap_state: State,

    // Your stable storage for persistence
    stable_assets: StableBTreeMap<String, Vec<u8>>,

    // Track what's persisted
    persisted_assets: HashSet<String>,
}

impl HybridAssetManager {
    // Store: Use library's functions, then copy to stable
    pub fn store_with_persistence(&mut self, key: &str, content: &[u8], persist: bool) -> Result<(), String> {
        // 1. Use library's store function (handles chunking, encoding, etc.)
        self.heap_state.store(StoreArg { /* ... */ }, time())?;

        // 2. Optionally copy to stable memory
        if persist {
            let asset = self.heap_state.assets.get(key).unwrap();
            let serialized = serialize_asset(asset)?;
            self.stable_assets.insert(key.to_string(), serialized);
            self.persisted_assets.insert(key.to_string());
        }

        Ok(())
    }

    // Retrieve: Check heap first, then stable
    pub fn get_with_fallback(&mut self, key: &str) -> Result<EncodedAsset, String> {
        // 1. Fast path: check heap first
        if self.heap_state.assets.contains_key(key) {
            return self.heap_state.get(GetArg { /* ... */ });
        }

        // 2. Slow path: load from stable memory
        if let Some(stable_data) = self.stable_assets.get(key) {
            let asset = deserialize_asset(stable_data)?;
            // Copy back to heap for future fast access
            self.heap_state.assets.insert(key.to_string(), asset);
            return self.heap_state.get(GetArg { /* ... */ });
        }

        Err("asset not found".to_string())
    }
}
```

### **Key Benefits:**

- ✅ **Leverage library's features**: Chunking, encoding, certification, HTTP serving
- ✅ **Selective persistence**: Choose which assets survive upgrades
- ✅ **Performance**: Fast access for active assets (heap cache)
- ✅ **No library modification**: Clean wrapper approach
- ✅ **Flexible**: Can implement different persistence strategies

### **Use Cases:**

- **Critical assets**: Store important files in stable memory
- **Temporary assets**: Keep only in heap for performance
- **Large files**: Use library's chunking, persist selectively
- **Frequent access**: Cache in heap, persist for upgrades

### **Implementation Notes:**

- Use `StableBTreeMap` for stable storage
- Serialize/deserialize assets when copying between heap and stable
- Consider memory limits when deciding what to persist
- Implement cleanup strategies for heap cache management
