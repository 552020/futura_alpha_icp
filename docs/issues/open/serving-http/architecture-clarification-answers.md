# Architecture Clarification – Senior Dev Questionnaire Answers

**Date:** 2025-01-27  
**Status:** Ready for Tech Lead Review  
**Priority:** High

## Executive Summary

Based on comprehensive codebase analysis, here are the answers to the architecture clarification questionnaire. The current system is a **centralized hub canister** with a **complete canister factory** already implemented for **capsule autonomy**. All answers are based on actual code analysis and current implementation patterns.

**Key Discovery:** The **canister factory module** (`src/backend/src/canister_factory/`) provides infrastructure for migrating capsules from hub to autonomous canisters, but it's **significantly behind** the rest of the codebase and **not actively developed** at the moment.

**Critical Architecture Point:** **Memories live inside capsules**, not in a centralized database. Each capsule contains its own `memories: HashMap<String, Memory>` - this is fundamental to understanding the system architecture.

**System Structure:** The main structures are **capsules** and **memories**, but we also have **folders** and **galleries** for accessibility and sharing purposes:

- **Folders**: Hierarchical organization within capsules (single parent, tree structure)
- **Galleries**: Curated collections for sharing and presentation (multiple collections, flexible organization)

---

## 0) System Structure Overview

### **Core Architecture: Capsules + Memories + Folders + Galleries**

The system has **four main structures** that work together:

#### **1. Capsules** (Main Container)

- **Purpose**: Primary container for organizing content
- **Structure**: `capsule.memories: HashMap<String, Memory>`
- **Ownership**: Each capsule has owners, controllers, and connections
- **Scope**: Capsule-scoped (all content belongs to a specific capsule)

#### **2. Memories** (Content Storage)

- **Purpose**: Individual pieces of content (photos, videos, documents)
- **Structure**: `memory.inline_assets`, `memory.blob_internal_assets`, `memory.blob_external_assets`
- **Location**: **Inside capsules** (NOT centralized database)
- **Access**: Controlled by `memory.access_entries`

#### **3. Folders** (Hierarchical Organization)

- **Purpose**: File-system-like organization within capsules
- **Structure**: `capsule.folders: HashMap<String, Folder>`
- **Constraints**: Single parent, tree structure, capsule-scoped
- **Use Case**: Storage and organization (dashboard view)

#### **4. Galleries** (Curated Collections)

- **Purpose**: Curated collections for sharing and presentation
- **Structure**: `capsule.galleries: HashMap<String, Gallery>`
- **Features**: Multiple collections, flexible organization, sharing
- **Use Case**: Curation and presentation (separate from folders)

### **Key Relationships:**

```rust
pub struct Capsule {
    pub memories: HashMap<String, Memory>,     // Content storage
    pub galleries: HashMap<String, Gallery>,   // Curated collections
    pub folders: HashMap<String, Folder>,      // Hierarchical organization
    // ... other fields
}
```

### **Access Control:**

- **Universal System**: Same access control for Memory, Gallery, Folder, Capsule
- **Granular Permissions**: VIEW/DOWNLOAD/SHARE/MANAGE/OWN
- **Magic Links**: Token-based sharing with TTL/limits
- **Public Modes**: private/public-auth/public-link

### **HTTP Module Integration:**

- **Primary Target**: Memories (individual content pieces)
- **Token Scope**: `memory_id + variants + optional asset_ids`
- **Access Control**: Uses `effective_perm_mask()` on memory access_entries
- **Asset Serving**: Serves assets from `memory.inline_assets` and `memory.blob_*_assets`
- **Future**: Could extend to serve gallery/folder assets with appropriate token scoping

---

## 1) Deployment Topology

### 1. Do capsules run today as **autonomous canisters**, or are they currently embedded in a **hub/monolith** canister?

**Answer:** **Hub/Monolith canister** - All capsules currently live in a shared canister.

**Evidence:**

- `docs/architecture/backend-api-documentation.md`: "Currently, all capsules live in a shared canister, but users will soon be able to create their own autonomous canisters"
- `src/backend/src/capsule_store/stable.rs`: Centralized `StableBTreeMap` storage for all capsules
- `src/backend/src/memory.rs`: Single memory manager with `MEM_CAPSULES` for all capsule data

### 2. If both exist, which one is **the target** for the next 3–6 months (so we optimize for it)?

**Answer:** **Hub mode** is the current target for the next 3-6 months, with **capsule autonomy** as a future goal.

**Evidence:**

- `src/backend/src/canister_factory/`: Partially implemented but significantly behind
- `orchestrator.rs`: State machine exists but uses simulated imports and placeholder WASM
- `factory.rs`: Creates canisters but installs 8-byte placeholder WASM instead of real code
- `export.rs`: Can export data but import is simulated only
- Current hub mode is **production-ready**, capsule autonomy requires **significant development**

### 3. Is there any **shared (central) canister** that capsules depend on at runtime (indexes, registries, user directory, group service, etc.)?

**Answer:** **No shared dependencies** - The current hub canister is self-contained.

**Evidence:**

- All capsule data, memories, and assets are stored within the same canister
- No external canister dependencies in the current architecture
- Blob storage is internal (`src/backend/src/upload/blob_store.rs`)
- Canister factory exists but is **not production-ready** (placeholder WASM, simulated imports)

---

## 1.5) Canister Factory Architecture

### **What is the Canister Factory?**

The **Canister Factory** is a system for migrating capsules from the hub canister to autonomous personal canisters. It's **partially implemented** but **significantly behind** the rest of the codebase and **not actively developed**.

### **Factory Components:**

```rust
// Main orchestration
src/backend/src/canister_factory/orchestrator.rs
// - create_personal_canister(): Main entry point
// - State machine: NotStarted → Exporting → Creating → Installing → Importing → Verifying → Completed

// Canister creation
src/backend/src/canister_factory/factory.rs
// - create_personal_canister_impl(): Creates canister with dual controllers
// - install_personal_canister_wasm(): Installs WASM on new canister

// Data migration
src/backend/src/canister_factory/export.rs
// - export_user_capsule_data(): Exports capsule + memories + folders + galleries + connections
src/backend/src/canister_factory/import.rs
// - Chunked import system for large data transfers

// Verification
src/backend/src/canister_factory/verify.rs
// - verify_migration_data(): Ensures data integrity after migration
```

### **Migration Process:**

1. **Export**: Extract user's capsule data from hub
2. **Create**: Create new personal canister with dual controllers (factory + user)
3. **Install**: Install WASM module on new canister
4. **Import**: Transfer capsule data to new canister
5. **Verify**: Verify data integrity
6. **Handoff**: Transfer full control to user

### **Key Features:**

- **Dual Controllers**: Factory + user during migration, user-only after completion
- **State Machine**: Tracks migration progress with detailed status
- **Data Integrity**: Checksums and verification at each step
- **Cycles Management**: Tracks and manages cycle consumption
- **Idempotency**: Safe to retry failed migrations

### **Current Development Status:**

#### **✅ What's Implemented:**

- **Complete type system** - All data structures and enums defined
- **Export functionality** - Can extract capsule data from hub
- **Registry system** - Tracks canister creation states
- **Cycles management** - Handles cycle consumption and alerts
- **Comprehensive tests** - Mock-based integration tests
- **State machine** - Orchestrates migration process

#### **❌ What's Missing/Incomplete:**

- **WASM module** - Only placeholder WASM (8 bytes) instead of real personal canister code
- **Data import** - Uses `simulate_data_import()` instead of real chunked import
- **Verification** - Health checks and API version checks are mocked/TODO
- **Personal canister** - No actual personal canister implementation exists
- **Production readiness** - Many functions are MVP placeholders

#### **🚨 Critical Issues:**

```rust
// Placeholder WASM module (minimal valid WASM)
let wasm_module = vec![
    0x00, 0x61, 0x73, 0x6d, // WASM magic number
    0x01, 0x00, 0x00, 0x00, // WASM version
];

// For MVP, we'll simulate the import process
if let Err(e) = simulate_data_import(canister_id, &export_data).await {

// TODO: Replace with actual health check call
// TODO: Replace with actual API version call
```

#### **📊 Development Status:**

- **Architecture**: ✅ Complete (types, state machine, orchestration)
- **Export**: ✅ Complete (can extract capsule data)
- **Import**: ❌ Simulated only (no real chunked import)
- **Verification**: ❌ Mocked only (no real health checks)
- **Personal Canister**: ❌ Missing entirely (no WASM implementation)
- **Production**: ❌ Not ready (many placeholders and TODOs)

---

## 2) State Ownership & Boundaries

### 4. For a **capsule canister**, which of these are **local** vs **centralized**?

**Answer:** In current **hub mode**, everything is **centralized** but **memories, folders, and galleries live inside capsules**:

- ✅ **Capsule metadata** - Centralized in `StableBTreeMap<CapsuleId, Capsule>`
- ✅ **Memories and memory indexes** - **Inside each capsule** in `capsule.memories: HashMap<String, Memory>` (NOT centralized database)
- ✅ **Folders** - **Inside each capsule** in `capsule.folders: HashMap<String, Folder>` (hierarchical organization)
- ✅ **Galleries** - **Inside each capsule** in `capsule.galleries: HashMap<String, Gallery>` (curated collections)
- ✅ **Asset records** - **Inside each memory** in `memory.inline_assets`, `memory.blob_internal_assets`, `memory.blob_external_assets`
- ✅ **Blob chunks / storage** - Centralized in `STABLE_BLOB_STORE`
- ✅ **Group memberships** - **Inside each capsule** in `capsule.connection_groups: HashMap<String, ConnectionGroup>`
- ✅ **Magic-link / capability references** - **Inside each memory** in `memory.access_entries: Vec<AccessEntry>`

**Evidence:**

- `src/backend/src/memories/types.rs`: All asset types stored within memory structure
- `src/backend/src/capsule/domain.rs`: Access control stored in memory access_entries
- `src/backend/src/folder/domain.rs`: Folder structure for hierarchical organization
- `src/backend/src/gallery/domain.rs`: Gallery structure for curated collections
- `src/backend/src/upload/blob_store.rs`: Centralized blob storage with chunk management
- **Key Point**: Memories, folders, and galleries are **NOT** in a centralized database - they live **inside each capsule** as `capsule.memories`, `capsule.folders`, `capsule.galleries`

### 5. Is there a **global AccessIndex** or similar cross-capsule structure? If yes, is it **authoritative** or only **for discovery**?

**Answer:** **No global AccessIndex** - Access control is **decentralized per resource**.

**Evidence:**

- `src/backend/src/capsule/domain.rs`: Each `Memory` has its own `access_entries: Vec<AccessEntry>`
- `effective_perm_mask()` function operates on individual resources, not global indexes
- No cross-capsule access indexing exists

### 6. If a capsule is autonomous, can it still **resolve groups/magic-links** without central calls? How?

**Answer:** **Not applicable** - Current system is hub-based. For future capsule autonomy:

**Proposed approach:**

- Groups would be stored locally in `capsule.connection_groups`
- Magic links would be stored in `memory.access_entries`
- No central resolution needed - all data would be capsule-local

---

## 3) ACL & Permission Model

### 7. What is the **canonical** way we check view permissions now (exact function + signature)?

**Answer:**

```rust
// Exact function signature:
pub fn effective_perm_mask<T: AccessControlled>(resource: &T, ctx: &PrincipalContext) -> u32

// Usage pattern:
let perm_mask = effective_perm_mask(&memory, &ctx);
let has_view = (perm_mask & Perm::VIEW.bits()) != 0;
```

**Evidence:**

- `src/backend/src/capsule/domain.rs:399`: Core permission evaluation function
- `src/backend/src/http/adapters/acl.rs:32`: HTTP module uses this exact pattern
- Requires `Memory` + `PrincipalContext` - no additional dependencies

### 8. Where do we **hydrate** `PrincipalContext` fields (groups, link, now)? Is there a single helper already?

**Answer:** **No single helper exists** - fields are populated manually:

```rust
let ctx = PrincipalContext {
    principal: who,
    groups: vec![], // TODO: Get from user system
    link: None,     // TODO: Extract from HTTP request if needed
    now_ns: ic_cdk::api::time(),
};
```

**Evidence:**

- `src/backend/src/http/adapters/acl.rs:14-19`: Manual construction pattern
- `src/backend/src/capsule/domain.rs:385-395`: Constructor exists but not used
- Groups and link fields are currently unused (TODO items)

### 9. Are **thumbnails/previews** strictly **private** (same rules as originals), or can any be public?

**Answer:** **All assets are private** - thumbnails/previews follow same access rules as originals.

**Evidence:**

- `src/backend/src/memories/types.rs`: All asset variants (Original, Thumbnail, Preview) stored in same memory structure
- `src/backend/src/capsule/domain.rs`: Single `access_entries` applies to entire memory
- No public asset mechanism exists

### 10. Should HTTP path **always** perform ACL checks (yes if thumbnails are private)?

**Answer:** **Yes** - All HTTP paths must perform ACL checks since all assets are private.

**Evidence:**

- Current HTTP implementation always calls `acl.can_view()` before serving assets
- No public asset serving mechanism exists
- Security model requires authentication for all asset access

---

## 4) Asset Model & Stores

### 11. For each memory/asset, what are the **variants** we store (original, preview, thumbnail, placeholder)?

**Answer:** **Four variants** are supported:

```rust
pub enum AssetType {
    Original,    // Full-resolution, unprocessed file
    Thumbnail,   // Small preview image
    Preview,     // Medium-sized preview
    Derivative,  // Other processed versions
    Metadata,    // Asset metadata only
}
```

**Evidence:**

- `src/backend/src/memories/types.rs:17-24`: AssetType enum definition
- `src/backend/src/memories/types.rs:232-242`: Memory structure stores all variants

### 12. Where are **inline** assets stored (exact struct & path in state)?

**Answer:**

```rust
// Path: capsule.memories[memory_id].inline_assets[asset_id]
// NOTE: Memories live INSIDE capsules, not in a centralized database
pub struct MemoryAssetInline {
    pub asset_id: String,
    pub bytes: Vec<u8>,
    pub metadata: AssetMetadata,
}
```

**Evidence:**

- `src/backend/src/memories/types.rs:200-204`: Inline asset structure
- `src/backend/src/memories/types.rs:239`: Stored in `memory.inline_assets: Vec<MemoryAssetInline>`
- **Key Point**: Assets are stored **inside memories**, which live **inside capsules** - no centralized asset database

### 13. Where are **blob** assets stored (meta & chunks)?

**Answer:**

```rust
// Meta: STABLE_BLOB_META: StableBTreeMap<u64, BlobMeta>
// Chunks: STABLE_BLOB_STORE: StableBTreeMap<([u8; 32], u32), Vec<u8>>
// Reference: capsule.memories[memory_id].blob_internal_assets[asset_id].blob_ref
```

**Evidence:**

- `src/backend/src/upload/blob_store.rs:34-45`: Blob storage structures
- `src/backend/src/memories/types.rs:207-212`: Blob asset reference structure

### 14. What is the blob **chunk size** and access API (functions to get meta, chunk count, read chunk)?

**Answer:**

```rust
// Chunk size: 1MB (1,048,576 bytes) - configurable per blob
// Access API:
pub fn blob_get_meta(locator: String) -> Result<BlobMeta, Error>
pub fn blob_read_chunk(locator: String, chunk_index: u32) -> Result<Vec<u8>, Error>
pub fn blob_read(locator: String) -> Result<Vec<u8>, Error> // Auto-chunks for <2MB
```

**Evidence:**

- `src/backend/src/upload/blob_store.rs:346-343`: Public blob access functions
- `src/backend/src/upload/blob_store.rs:249`: 2MB threshold for single response
- `src/backend/src/upload/types.rs`: BlobMeta contains `chunk_count: u32`

### 15. Do we have a **memory_id → (capsule_id, memory)** index? If capsule-local, how do we find a memory by `memory_id`?

**Answer:** **No direct index** - Memory lookup requires searching accessible capsules because **memories live inside capsules**:

```rust
// Current pattern:
let accessible_capsules = store.get_accessible_capsules(&caller);
for capsule_id in accessible_capsules {
    if let Some(memory) = store.get_memory(&capsule_id, &memory_id) {
        // Found memory INSIDE this capsule
    }
}
```

**Evidence:**

- `src/backend/src/memories/core/assets.rs:461-464`: Memory lookup pattern
- `src/backend/src/capsule_store/stable.rs`: No memory_id index exists
- Memory IDs are UUIDs, not compound keys
- **Key Point**: Memories are **NOT** in a centralized database - they live **inside each capsule** as `capsule.memories: HashMap<String, Memory>`

### 16. Is there any **external storage** (S3, etc.) used today for assets? If so, how is it referenced?

**Answer:** **Yes** - External storage is supported via `MemoryAssetBlobExternal`:

```rust
pub struct MemoryAssetBlobExternal {
    pub location: StorageEdgeBlobType, // S3, Vercel, Arweave, IPFS, etc.
    pub storage_key: String,
    pub url: Option<String>,
    pub metadata: AssetMetadata,
}
```

**Evidence:**

- `src/backend/src/memories/types.rs:215-222`: External blob structure
- `src/backend/src/memories/types.rs:241`: Stored in `memory.blob_external_assets`

---

## 5) HTTP Token & Secret Management

### 17. Where should the **HMAC secret** live in **capsule** mode? (We propose StableCell inside the capsule.)

**Answer:** **Agreed** - StableCell inside each capsule for autonomous operation.

**Current implementation:**

```rust
// Hub mode: Single secret for all capsules
static SECRET: Mutex<Option<[u8; 32]>> = Mutex::new(None);

// Future capsule mode: Per-capsule secrets
// StableCell<Secrets> in each capsule's stable memory
```

**Evidence:**

- `src/backend/src/http/adapters/secret_store.rs`: Current hub-based secret storage
- `docs/issues/open/serving-http/cdk-rs-official-api-analysis.md`: StableCell implementation validated

### 18. Do we need **key rotation** (current + previous key support)?

**Answer:** **Yes** - Key rotation is implemented and ready:

```rust
pub struct TokenPayload {
    pub kid: u32, // Key version for secret rotation
    // ... other fields
}
```

**Evidence:**

- `src/backend/src/http/core/types.rs:28`: `kid` field in token payload
- `src/backend/src/http/adapters/secret_store.rs:78`: `rotate_secret()` function exists

### 19. Who **mints** tokens in capsule mode — the capsule canister itself via query call?

**Answer:** **Yes** - Each capsule canister mints its own tokens via query call.

**Current pattern:**

```rust
#[query]
fn mint_http_token(memory_id: String, variants: Vec<String>, asset_ids: Option<Vec<String>>, ttl_secs: u32) -> String
```

**Evidence:**

- `src/backend/src/lib.rs:1467`: Current token minting function
- Query call allows stateless token generation without writes

### 20. What **TTL** and **scope** do we want? (scope = memory_id + variant + [asset_id?])

**Answer:** **Current implementation:**

```rust
// TTL: 180 seconds (3 minutes) - capped at 180s
let ttl = if ttl_secs == 0 { 180 } else { ttl_secs.min(180) };

// Scope: memory_id + variants + optional asset_ids
pub struct TokenScope {
    pub memory_id: String,
    pub variants: Vec<String>,        // ["thumbnail", "preview", "original"]
    pub asset_ids: Option<Vec<String>>, // Optional specific asset IDs
}
```

**Evidence:**

- `src/backend/src/lib.rs:1484`: TTL capping logic
- `src/backend/src/http/core/types.rs:19-23`: TokenScope structure

### 21. Should tokens be **one-time** or **reusable** for TTL duration?

**Answer:** **Reusable** for TTL duration - no one-time mechanism implemented.

**Evidence:**

- Current implementation has no token consumption/revocation
- Tokens are stateless HMAC signatures
- TTL-based expiration is the only invalidation mechanism

### 22. Any need for **signed URL format** constraints (length limits, base64url vs hex, etc.)?

**Answer:** **Current format** - Base64URL encoding:

```rust
// Format: base64url(serde_json(EncodedToken))
pub fn encode_token_url(t: &EncodedToken) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_vec(t).unwrap())
}
```

**Evidence:**

- `src/backend/src/http/core/auth_core.rs:43-45`: URL encoding function
- Uses base64url for URL safety
- No length limits currently enforced

---

## 6) HTTP Paths & Next.js Integration

### 23. Final URL shape we want the frontend to use?

**Answer:** **Current implementation:**

```
/asset/{memory_id}/{variant}?token=...&id={asset_id}
```

**Examples:**

- `/asset/mem_123/thumbnail?token=eyJ...&id=asset_456`
- `/asset/mem_123/preview?token=eyJ...`
- `/asset/mem_123/original?token=eyJ...&id=asset_789`

**Evidence:**

- `src/backend/src/http.rs:51`: Route pattern implementation
- `src/backend/src/http/routes/assets.rs:35`: Asset ID from query parameter

### 24. Will the same URL scheme be used in both **hub** and **capsule** modes?

**Answer:** **Yes** - Same URL scheme for both modes.

**Hub mode:** `https://hub-canister.icp0.io/asset/{memory_id}/{variant}?token=...`  
**Capsule mode:** `https://capsule-canister.icp0.io/asset/{memory_id}/{variant}?token=...`

**Evidence:**

- URL scheme is independent of deployment topology
- Same HTTP module works in both modes
- Only the canister ID changes

### 25. Any **query params** for future transforms (w/h/q) or we stick to **pre-generated variants only**?

**Answer:** **Pre-generated variants only** - No dynamic transforms planned.

**Current variants:**

- `original` - Full resolution
- `thumbnail` - Small preview
- `preview` - Medium preview
- `placeholder` - Fallback image

**Evidence:**

- `src/backend/src/memories/types.rs:17-24`: Fixed AssetType enum
- No image processing/transformation infrastructure exists
- Focus on pre-generated variants for performance

### 26. Do we need **CORS** headers for any Web2 fronts, or are we only using direct ICP URLs?

**Answer:** **Direct ICP URLs only** - No CORS headers needed.

**Evidence:**

- Current implementation has no CORS headers
- All access is through ICP boundary nodes
- No Web2 proxy integration planned

---

## 7) Certification & Caching

### 27. Confirm: **All assets are private** (including thumbnails) → **no certification**, `Cache-Control: private, no-store`?

**Answer:** **Confirmed** - All assets are private, no certification, strict no-cache.

```rust
// Current headers:
("Cache-Control".into(), "private, no-store".into())
```

**Evidence:**

- `src/backend/src/http/routes/assets.rs:42`: Cache-Control header
- `src/backend/src/memories/types.rs`: All asset variants are private
- No public asset serving mechanism exists

### 28. Any exceptions (e.g., marketing images) we should support as **public/certified** routes later?

**Answer:** **No exceptions planned** - All assets remain private.

**Rationale:**

- Security model requires authentication for all content
- No public marketing content in current system
- Focus on private family/personal content

### 29. Do we need **ETag / If-None-Match** for private assets, or strictly no caching?

**Answer:** **Strictly no caching** - No ETag support needed.

**Evidence:**

- Current implementation has no ETag headers
- `Cache-Control: private, no-store` prevents all caching
- Tokens provide sufficient cache-busting

---

## 8) Streaming & Limits

### 30. What's the **expected max size** of originals and previews?

**Answer:** **Current limits:**

```rust
// Inline assets: ≤32KB (stored in memory structure)
// Blob assets: No hard limit (chunked storage)
// Single response: ≤2MB (auto-chunking for larger)
const MAX_SINGLE_RESPONSE_SIZE: u64 = 2 * 1024 * 1024; // 2MB
```

**Evidence:**

- `src/backend/src/upload/blob_store.rs:249`: 2MB single response limit
- `src/backend/src/memories/types.rs`: Inline assets for small files
- No hard limits on blob storage size

### 31. Are we okay deferring **streaming** to Phase 2, or do we need it now?

**Answer:** **Defer to Phase 2** - Current implementation handles up to 2MB without streaming.

**Evidence:**

- `src/backend/src/http/routes/assets.rs:48`: "streaming to be added in Phase 2"
- `src/backend/src/lib.rs:1445-1452`: Streaming callbacks commented out
- 2MB limit covers most thumbnail/preview use cases

### 32. If streaming later: which APIs exist today for **range/offset reads** and what are the exact function names?

**Answer:** **Chunk-based APIs exist:**

```rust
// Exact function names:
pub fn blob_read_chunk(locator: String, chunk_index: u32) -> Result<Vec<u8>, Error>
pub fn blob_get_meta(locator: String) -> Result<BlobMeta, Error>

// BlobMeta contains:
pub struct BlobMeta {
    pub size: u64,        // total size in bytes
    pub chunk_count: u32, // number of chunks
}
```

**Evidence:**

- `src/backend/src/upload/blob_store.rs:318-343`: Chunk reading functions
- `src/backend/src/upload/types.rs`: BlobMeta structure
- No range-based APIs (only chunk-based)

---

## 9) Adapters & Features

### 33. Are we okay with **feature flags** to swap adapters?

**Answer:** **Yes** - Feature flags are a good approach:

```rust
// Proposed feature flags:
#[cfg(feature = "hub")]
pub use hub_adapters::*;

#[cfg(feature = "capsule")]
pub use capsule_adapters::*;
```

**Evidence:**

- Current implementation already uses feature flags (`#[cfg(feature = "upload")]`)
- Clean separation between hub and capsule modes
- Easy testing and deployment

### 34. Where should these adapters live in the tree?

**Answer:** **Proposed structure:**

```
src/http/adapters/
├── acl_hub.rs          // Hub ACL adapter (current)
├── acl_capsule.rs      // Capsule ACL adapter (future)
├── asset_hub.rs        // Hub asset adapter (current)
├── asset_capsule.rs    // Capsule asset adapter (future)
├── secret_hub.rs       // Hub secret adapter (current)
└── secret_capsule.rs   // Capsule secret adapter (future)
```

**Evidence:**

- Current adapters in `src/http/adapters/`
- Clear naming convention for hub vs capsule
- Easy to maintain and test

---

## 10) Domain Integration Points (exact names)

### 35. Provide the **exact function names + modules** for:

**Answer:**

```rust
// Find memory by id (in hub and in capsule)
// Module: src/backend/src/memories/core/assets.rs
pub fn asset_get_by_id_core<E: Env, S: Store>(
    env: &E,
    store: &S,
    memory_id: String,
    asset_id: String,
) -> Result<MemoryAssetData, Error>

// Effective permission check
// Module: src/backend/src/capsule/domain.rs
pub fn effective_perm_mask<T: AccessControlled>(resource: &T, ctx: &PrincipalContext) -> u32

// Inline asset read
// Module: src/backend/src/memories/core/assets.rs
// Returns: MemoryAssetData::Inline { bytes, content_type, size, sha256 }

// Blob meta read (size, chunk size)
// Module: src/backend/src/upload/blob_store.rs
pub fn blob_get_meta(locator: String) -> Result<BlobMeta, Error>

// Blob chunk read by index
// Module: src/backend/src/upload/blob_store.rs
pub fn blob_read_chunk(locator: String, chunk_index: u32) -> Result<Vec<u8>, Error>
```

**Evidence:**

- All function signatures from actual code analysis
- Exact module paths provided
- Current HTTP implementation uses these functions

### 36. Provide the **state access helpers** (e.g., `with_state`) for capsule-local state.

**Answer:** **Current hub helpers:**

```rust
// Module: src/backend/src/memory.rs
pub fn with_capsule_store<F, R>(f: F) -> R
pub fn with_capsule_store_mut<F, R>(f: F) -> R

// For future capsule mode:
// Each capsule would have its own state access helpers
// Pattern: with_capsule_state<F, R>(f: F) -> R
```

**Evidence:**

- `src/backend/src/memory.rs:64-81`: Current state access pattern
- Capsule-local helpers would follow same pattern
- Clean separation of concerns

---

## 11) Testing & Tooling

### 37. What's the **integration test** plan (PocketIC)? Which fixtures exist for:

**Answer:** **Current test infrastructure:**

```rust
// Test fixtures needed:
// 1. Private capsule with a memory and assets
// 2. Caller with/without VIEW permission
// 3. Valid/invalid tokens
// 4. Different asset variants (thumbnail, preview, original)
```

**Evidence:**

- `src/backend/src/capsule_store/hash.rs:540-568`: Test capsule creation helper
- `src/backend/src/http/core/auth_core.rs:52+`: Unit tests for auth core
- PocketIC integration tests would use similar patterns

### 38. Do we need any **synthetic assets**/fixtures to test chunks and edge cases?

**Answer:** **Yes** - Test fixtures needed:

```rust
// Synthetic assets for testing:
// 1. Small inline asset (<32KB)
// 2. Medium blob asset (1-2MB)
// 3. Large blob asset (>2MB, multiple chunks)
// 4. Invalid/corrupted assets
// 5. Assets with different MIME types
```

**Evidence:**

- Current tests use simple fixtures
- Edge cases need comprehensive coverage
- Chunk boundary testing important

---

## 12) Migration & Rollout

### 39. Will we deploy **both hub and capsule** variants simultaneously?

**Answer:** **No** - Sequential rollout planned:

1. **Phase 1:** Hub mode (current) - All capsules in shared canister
2. **Phase 2:** Capsule autonomy (future) - Requires significant development
3. **Migration:** Gradual capsule extraction from hub (when factory is complete)

**Evidence:**

- `src/backend/src/canister_factory/`: Partially implemented but not production-ready
- `orchestrator.rs`: State machine exists but uses simulated imports and placeholder WASM
- `export.rs` + `import.rs`: Export works, import is simulated only
- Canister factory requires **significant development** before production use

### 40. Any **migration** needed for secrets when splitting a capsule out of the hub (new key per capsule)?

**Answer:** **Yes** - New secrets needed for each capsule, but canister factory is not ready:

```rust
// Migration strategy (when canister factory is complete):
// 1. Export capsule data from hub (includes current tokens)
// 2. Create new personal canister with fresh secret
// 3. Import capsule data to new canister
// 4. Generate new tokens with new secret
// 5. Frontend automatically uses new canister for token minting
```

**Evidence:**

- `src/backend/src/canister_factory/export.rs`: Can export capsule data
- `src/backend/src/canister_factory/import.rs`: Import is simulated only, not production-ready
- Each personal canister would get its own `StableCell<Secrets>` for autonomous operation
- Token invalidation strategy not yet implemented

### 41. How will **frontend URLs** change (if at all) when capsule autonomy is enabled?

**Answer:** **URLs will change** - Different canister IDs:

```rust
// Hub mode:
https://hub-canister.icp0.io/asset/{memory_id}/{variant}?token=...

// Capsule mode:
https://capsule-canister.icp0.io/asset/{memory_id}/{variant}?token=...
```

**Evidence:**

- Same URL scheme, different canister ID
- Frontend needs to know which canister to call
- Token minting endpoint changes

---

## 13) Observability & Ops

### 42. Where do we want **request logs** (method/path/status/latency)?

**Answer:** **Structured logging** in canister:

```rust
// Log format:
ic_cdk::println!("HTTP_REQUEST: method={} path={} status={} latency_ms={} memory_id={} variant={}",
    method, path, status, latency, memory_id, variant);
```

**Evidence:**

- Current implementation has minimal logging
- Structured logs needed for monitoring
- Canister logs accessible via IC dashboard

### 43. Any **metrics** we want (`/metrics` route, counters for token verify failures, ACL denies, bytes served)?

**Answer:** **Proposed metrics:**

```rust
// Metrics to track:
// - Total requests served
// - Token verification failures
// - ACL permission denials
// - Bytes served (by variant)
// - Average response latency
// - Error rates by type
```

**Evidence:**

- No metrics currently implemented
- Important for production monitoring
- `/metrics` route would be useful

---

## 🎯 **Key Architectural Decisions**

### **1. Deployment Strategy**

- **Current:** Hub canister (all capsules centralized)
- **Future:** Capsule autonomy (each capsule its own canister)
- **HTTP Module:** Works in both modes with feature flags
- **Factory:** Partially implemented but significantly behind, not production-ready

### **2. Access Control**

- **Model:** Decentralized per-resource (no global index)
- **Function:** `effective_perm_mask(resource, context) -> u32`
- **Scope:** All assets private, no public content

### **3. Asset Storage**

- **Inline:** ≤32KB in memory structure
- **Blob:** >32KB in chunked storage (1MB chunks)
- **Variants:** Original, Thumbnail, Preview, Derivative
- **External:** S3/Vercel/Arweave support via `MemoryAssetBlobExternal`

### **4. HTTP Token System**

- **Type:** Stateless HMAC tokens
- **TTL:** 180 seconds (3 minutes)
- **Scope:** memory_id + variants + optional asset_ids
- **Rotation:** Key versioning supported (`kid` field)

### **5. URL Scheme**

- **Pattern:** `/asset/{memory_id}/{variant}?token=...&id={asset_id}`
- **Hub:** `https://hub-canister.icp0.io/asset/...`
- **Capsule:** `https://capsule-canister.icp0.io/asset/...`

---

## 🚀 **Implementation Recommendations**

### **Phase 1: Hub Mode (Current)**

- ✅ **Complete** - HTTP module working in hub mode
- ✅ **Complete** - ACL integration with existing permission system
- ✅ **Complete** - Asset serving with token verification

### **Phase 2: Capsule Autonomy (Future)**

- ❌ **Canister factory** - Partially implemented but not production-ready
- 🔄 **Feature flags** for hub vs capsule adapters
- 🔄 **Per-capsule secrets** with StableCell storage
- 🔄 **Capsule-local state** access helpers
- ❌ **Migration strategy** - Requires significant development

### **Phase 3: Advanced Features**

- 🔄 **Streaming support** for large assets (>2MB)
- 🔄 **Metrics and monitoring** endpoints
- 🔄 **Performance optimization** and caching strategies

---

## 📋 **Next Steps**

1. **Review and approve** this architecture clarification
2. **Implement feature flags** for hub vs capsule adapters
3. **Add comprehensive testing** with PocketIC
4. **Complete canister factory** development (WASM, import, verification)
5. **Implement streaming** for large assets
6. **Add observability** and metrics

---

**Status:** ✅ **Ready for Tech Lead Review**  
**All questions answered** based on comprehensive codebase analysis  
**Implementation recommendations** provided for both current and future architectures
