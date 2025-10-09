# Gallery Type Refactor - Implementation Plan

**Status**: `OPEN` - Implementation Required  
**Priority**: `MEDIUM` - Architecture Improvement  
**Assigned**: Backend Developer + Frontend Developer  
**Created**: 2024-12-19  
**Related Issues**: [Gallery Type Refactor - Schema Normalization](./gallery-type-refactor.md)

## Overview

This document outlines the implementation plan for refactoring gallery types to align with the memory pattern, implement universal sharing system, and standardize metadata structures across all entities (Memory, Gallery, Folder, Capsule).

## Tech Lead Feedback Integration

The tech lead has provided excellent feedback for **Web2 centralized systems**, but there's a fundamental architectural mismatch with the **ICP decentralized capsule system**:

### **Tech Lead's Assumptions (Web2 Model):**

- Central database with global tables
- Top-level indexes across all resources
- Global access control system
- Cross-resource sharing within a single system

### **Actual ICP Architecture:**

- **Each capsule is an autonomous canister**
- **Memories are stored INSIDE the capsule** (`pub memories: HashMap<String, Memory>`)
- **No global access control** - each capsule manages its own access
- **No cross-capsule resource sharing** in current design

### **Applicable Tech Lead Recommendations:**

1. **✅ Single Source of Truth**: Use `bitflags` for permissions in both TS and Rust
2. **❌ Top-Level Indexes**: Not applicable - each capsule is autonomous
3. **✅ Pure Evaluation Pipeline**: Define one evaluation function per capsule
4. **✅ Time Normalization**: Handle ns (ICP) vs ms (Neon) time units
5. **✅ Cached UI Fields**: Treat `sharing_status` as cached UI field only
6. **❌ Storage Location Caching**: Not applicable - no global storage_edges
7. **✅ Idempotent APIs**: Small request structs with upserts
8. **✅ Roles as Data**: Configurable role-to-permission mapping per capsule

## Implementation Phases

### **Phase 1: Access Refactoring (UPDATED)**

#### **Task 1.1: Implement Bitflags for Permissions**

**File**: `src/backend/src/capsule/types.rs`

```rust
// ✅ TECH LEAD APPROVED: Bitflags for permissions (single source of truth)
use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Perm: u32 {
        const VIEW = 1 << 0;      // 1
        const DOWNLOAD = 1 << 1;  // 2
        const SHARE = 1 << 2;     // 4
        const MANAGE = 1 << 3;    // 8
        const OWN = 1 << 4;       // 16
    }
}

// ✅ TECH LEAD APPROVED: Role templates as data (stored in capsule)
pub struct RoleTemplate {
    pub name: String,
    pub perm_mask: u32,  // Uses Perm bits
    pub description: String,
}

// ✅ TECH LEAD APPROVED: Default role templates
impl Default for RoleTemplate {
    fn default() -> Self {
        Self {
            name: "member".to_string(),
            perm_mask: (Perm::VIEW | Perm::DOWNLOAD).bits(),
            description: "Standard member access".to_string(),
        }
    }
}

// ✅ TECH LEAD APPROVED: Role templates for common roles
pub fn get_default_role_templates() -> Vec<RoleTemplate> {
    vec![
        RoleTemplate {
            name: "owner".to_string(),
            perm_mask: (Perm::VIEW | Perm::DOWNLOAD | Perm::SHARE | Perm::MANAGE | Perm::OWN).bits(),
            description: "Full ownership access".to_string(),
        },
        RoleTemplate {
            name: "admin".to_string(),
            perm_mask: (Perm::VIEW | Perm::DOWNLOAD | Perm::SHARE | Perm::MANAGE).bits(),
            description: "Administrative access".to_string(),
        },
        RoleTemplate {
            name: "member".to_string(),
            perm_mask: (Perm::VIEW | Perm::DOWNLOAD).bits(),
            description: "Standard member access".to_string(),
        },
        RoleTemplate {
            name: "guest".to_string(),
            perm_mask: Perm::VIEW.bits(),
            description: "Read-only access".to_string(),
        },
    ]
}
```

#### **Task 1.2: Create Universal Access System Types**

**File**: `src/backend/src/capsule/types.rs`

```rust
// ✅ UPDATED: Centralized access control system (no embedded ResourceAccess)
// ❌ REMOVED: ResourceAccess struct - access is now centralized in AccessIndex

pub struct AccessEntry {
    pub id: String,
    pub person_ref: PersonRef,            // ✅ ICP: Principal or Opaque ID
    pub grant_source: GrantSource,        // ✅ Provenance tracking
    pub source_id: Option<String>,        // ✅ Group/magic_link ID
    pub role: ResourceRole,               // ✅ Role system
    pub perm_mask: u32,                   // ✅ Bitmask permissions (uses Perm bits)
    pub invited_by_person_ref: Option<PersonRef>, // ✅ ICP: Who granted access
    pub created_at: u64,
    pub updated_at: u64,
}

pub struct PublicPolicy {
    pub mode: PublicMode,
    pub perm_mask: u32,                   // ✅ Bitmask permissions (uses Perm bits)
    pub created_at: u64,
    pub updated_at: u64,
}

// ✅ UNIVERSAL ENUMS
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum ResourceType {
    Memory,
    Gallery,
    Folder,
    Capsule,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum GrantSource {
    User,           // Direct user grant (from connections)
    Group,          // Group membership grant (from connection_groups)
    MagicLink,      // Magic link grant (temporary access)
    PublicMode,     // Public access grant
    System,         // System-generated grant
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum ResourceRole {
    Owner,          // Full ownership
    SuperAdmin,     // Administrative access
    Admin,          // Management access
    Member,         // Standard access
    Guest,          // Limited access
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum SharingStatus {
    Public,     // Publicly accessible
    Shared,     // Shared with specific users/groups
    Private,    // Only owner can access
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum PublicMode {
    Private,        // No public access
    PublicAuth,     // Public for authenticated users
    PublicLink,     // Public for anyone with link
}
```

#### **Task 1.2: Create Public Policy Types**

**File**: `src/backend/src/types.rs`

```rust
pub struct PublicPolicy {
    pub id: String,
    pub resource_type: ResourceType,
    pub resource_id: String,
    pub mode: PublicMode,                   // 'private', 'public_auth', 'public_link'
    pub perm_mask: u32,
    pub created_at: u64,
    pub updated_at: u64,
}
```

### **Phase 1.5: Tech Lead Recommendations Integration**

#### **Task 1.5.1: Create Tech Lead's Access Index System**

**File**: `src/backend/src/capsule/access.rs`

```rust
// ✅ TECH LEAD APPROVED: Key to address any resource inside a capsule
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ResKey {
    pub r#type: ResourceType,   // Memory | Gallery | Folder | Capsule
    pub id: String,             // UUID/ID
}

// ✅ TECH LEAD APPROVED: Access entries and public policy stored once per resource
#[derive(Serialize, Deserialize, Default)]
pub struct AccessIndex {
    // (ResKey) -> Vec<AccessEntry>
    pub entries: StableBTreeMap<ResKey, Vec<AccessEntry>>,
    // (ResKey) -> PublicPolicy
    pub policy:  StableBTreeMap<ResKey, PublicPolicy>,
}

// ✅ TECH LEAD APPROVED: Permission evaluation (pure function)
pub fn effective_perm_mask(
    key: &ResKey,
    ctx: &PrincipalContext,
    idx: &AccessIndex,
    capsule: &Capsule,
    now_ns: u64,
) -> u32 {
    use Perm as P;
    // 1) ownership fast-path
    if is_owner(key, ctx, capsule) {
        return (P::VIEW|P::DOWNLOAD|P::SHARE|P::MANAGE|P::OWN).bits();
    }

    let mut m = 0u32;
    // 2) direct grants
    if let Some(v) = idx.entries.get(key) {
        m |= sum_user_and_groups(v, ctx);
    }
    // 3) magic link
    if let Some(token) = &ctx.link {
        m |= link_mask_if_valid(key, token, idx, now_ns);
    }
    // 4) public policy
    m |= public_mask_if_any(key, &idx.policy, ctx, now_ns);
    m
}

// ✅ Helper functions for permission evaluation
fn is_owner(key: &ResKey, ctx: &PrincipalContext, capsule: &Capsule) -> bool {
    match key.r#type {
        ResourceType::Memory => {
            if let Some(memory) = capsule.memories.get(&key.id) {
                capsule.is_owner(&ctx.principal) || capsule.is_controller(&ctx.principal)
            } else {
                false
            }
        },
        ResourceType::Gallery => {
            if let Some(gallery) = capsule.galleries.get(&key.id) {
                capsule.is_owner(&ctx.principal) || capsule.is_controller(&ctx.principal)
            } else {
                false
            }
        },
        ResourceType::Folder => {
            if let Some(folder) = capsule.folders.get(&key.id) {
                capsule.is_owner(&ctx.principal) || capsule.is_controller(&ctx.principal)
            } else {
                false
            }
        },
        ResourceType::Capsule => {
            capsule.is_owner(&ctx.principal) || capsule.is_controller(&ctx.principal)
        },
    }
}

fn sum_user_and_groups(entries: &[AccessEntry], ctx: &PrincipalContext) -> u32 {
    let mut mask = 0u32;
    for entry in entries {
        if entry.person_ref == PersonRef::Principal(ctx.principal) {
            mask |= entry.perm_mask;
        }
        // TODO: Add group membership checks
    }
    mask
}

fn link_mask_if_valid(key: &ResKey, token: &str, idx: &AccessIndex, now_ns: u64) -> u32 {
    // TODO: Implement magic link validation
    0
}

fn public_mask_if_any(key: &ResKey, policy: &StableBTreeMap<ResKey, PublicPolicy>, ctx: &PrincipalContext, now_ns: u64) -> u32 {
    if let Some(policy) = policy.get(key) {
        match policy.mode {
            PublicMode::Private => 0,
            PublicMode::PublicAuth => policy.perm_mask, // Authenticated users get policy permissions
            PublicMode::PublicLink => {
                if ctx.link.is_some() {
                    policy.perm_mask
                } else {
                    0
                }
            },
        }
    } else {
        0
    }
}

// ✅ TECH LEAD APPROVED: Principal context for evaluation
pub struct PrincipalContext {
    pub principal: Principal,
    pub groups: Vec<String>,           // connection group ids within this capsule
    pub link: Option<String>,          // presented token (opaque)
    pub now_ns: u64,                  // ✅ ICP time (ns)
}
```

#### **Task 1.5.2: Create Idempotent API Request Structs**

**File**: `src/backend/src/capsule/api.rs`

```rust
// ✅ TECH LEAD APPROVED: Small request structs and idempotent upserts
#[derive(Serialize, Deserialize)]
pub struct ShareRequest {
    pub resource_id: String,
    pub person: PersonRef,
    pub role: ResourceRole,
    pub perm_mask: u32
}

#[derive(Serialize, Deserialize)]
pub struct ShareResult {
    pub perm_mask: u32,                // ✅ Return new perm_mask
    pub version: u64,                  // ✅ Monotonically increasing version
    pub success: bool,
}

#[derive(Serialize, Deserialize)]
pub struct PublicPolicyRequest {
    pub resource_id: String,
    pub mode: PublicMode,
    pub perm_mask: u32,
}

#[derive(Serialize, Deserialize)]
pub struct PublicPolicyResult {
    pub perm_mask: u32,
    pub version: u64,
    pub success: bool,
}
```

#### **Task 1.5.3: Create Storage Location Cache**

**File**: `src/backend/src/capsule/storage.rs`

```rust
// ✅ TECH LEAD APPROVED: Avoid scanning storage_edges on reads
pub struct StorageLocationCache {
    // (ResourceType, ResourceId) -> Vec<BlobHosting>
    pub locations: std::collections::HashMap<(ResourceType, String), Vec<BlobHosting>>,
}

impl StorageLocationCache {
    pub fn update_for_gallery(&mut self, gallery_id: &str, memory_ids: &[String]) {
        let mut locations = std::collections::HashSet::new();
        for memory_id in memory_ids {
            if let Some(memory_locations) = self.locations.get(&(ResourceType::Memory, memory_id.clone())) {
                locations.extend(memory_locations);
            }
        }
        self.locations.insert(
            (ResourceType::Gallery, gallery_id.to_string()),
            locations.into_iter().collect()
        );
    }

    pub fn get_locations(&self, res_ty: ResourceType, res_id: &str) -> Vec<BlobHosting> {
        self.locations.get(&(res_ty, res_id.to_string()))
            .cloned()
            .unwrap_or_default()
    }
}
```

#### **Task 1.5.4: Create Time Normalization Utilities**

**File**: `src/backend/src/capsule/time.rs`

```rust
// ✅ TECH LEAD APPROVED: Handle ns (ICP) vs ms (Neon) time units
pub const MAGIC_LINK_TTL_NS: u64 = 7 * 24 * 60 * 60 * 1_000_000_000; // 7 days in nanoseconds

pub fn icp_time_to_neon_ms(icp_time_ns: u64) -> u64 {
    icp_time_ns / 1_000_000
}

pub fn neon_ms_to_icp_time(neon_time_ms: u64) -> u64 {
    neon_time_ms * 1_000_000
}

pub fn now_icp_ns() -> u64 {
    ic_cdk::api::time()
}

pub fn is_expired(created_at_ns: u64, ttl_ns: u64) -> bool {
    now_icp_ns() > created_at_ns + ttl_ns
}
```

#### **Task 1.5.5: Write Flows (Tech Lead's Design)**

**File**: `src/backend/src/capsule.rs`

```rust
// ✅ MODERN RUST: Main capsule module file
pub mod types;
pub mod access;
pub mod api;
pub mod storage;
pub mod time;

// Re-export for convenience
pub use types::*;
pub use access::*;
pub use api::*;
pub use storage::*;
pub use time::*;

// ✅ TECH LEAD APPROVED: Write flows
impl Capsule {
    // When you add/remove a share or change public mode:
    pub fn update_access(&mut self, res_key: &ResKey, access_entry: AccessEntry) {
        // 1. Mutate capsule.access_idx for that ResKey
        self.access_idx.entries.insert(res_key.clone(), vec![access_entry]);

        // 2. Recompute and store the cached fields (shared_count, sharing_status)
        self.recompute_cached_fields(res_key);
    }

    pub fn set_public_policy(&mut self, res_key: &ResKey, policy: PublicPolicy) {
        // 1. Mutate capsule.access_idx.policy for that ResKey
        self.access_idx.policy.insert(res_key.clone(), policy);

        // 2. Recompute and store the cached fields
        self.recompute_cached_fields(res_key);
    }

    fn recompute_cached_fields(&mut self, res_key: &ResKey) {
        match res_key.r#type {
            ResourceType::Memory => {
                if let Some(memory) = self.memories.get_mut(&res_key.id) {
                    // Recompute shared_count and sharing_status from access_idx
                    let entries = self.access_idx.entries.get(res_key).unwrap_or(&vec![]);
                    let policy = self.access_idx.policy.get(res_key);

                    memory.metadata.shared_count = entries.len() as u32;
                    memory.metadata.sharing_status = compute_sharing_status(entries, policy);
                }
            },
            ResourceType::Gallery => {
                if let Some(gallery) = self.galleries.get_mut(&res_key.id) {
                    // Recompute cached fields
                    let entries = self.access_idx.entries.get(res_key).unwrap_or(&vec![]);
                    let policy = self.access_idx.policy.get(res_key);

                    gallery.metadata.shared_count = entries.len() as u32;
                    gallery.metadata.sharing_status = compute_sharing_status(entries, policy);
                }
            },
            // ... similar for Folder and Capsule
        }
    }
}
```

### **Phase 2: Memory Implementation**

#### **Task 2.1: Update Memory Types to Use Universal Access**

**File**: `src/backend/src/memories/types.rs`

```rust
// ✅ TECH LEAD APPROVED: Remove embedded access, use capsule.access_idx
pub struct Memory {
    pub id: String,                                         // UUID v7 (not compound)
    pub capsule_id: String,                                 // Capsule context
    pub metadata: MemoryMetadata, // memory-level metadata (title, description, etc.)
    // ❌ REMOVED: pub access: ResourceAccess,     // Use capsule.access_idx instead
    pub inline_assets: Vec<MemoryAssetInline>, // 0 or more inline assets
    pub blob_internal_assets: Vec<MemoryAssetBlobInternal>, // 0 or more ICP blob assets
    pub blob_external_assets: Vec<MemoryAssetBlobExternal>, // 0 or more external blob assets
}

// ✅ UPDATE: MemoryMetadata to remove is_public
pub struct MemoryMetadata {
    // Basic info
    pub memory_type: MemoryType,
    pub title: Option<String>,       // Optional title (matches database)
    pub description: Option<String>, // Optional description (matches database)
    pub content_type: String,

    // Timestamps
    pub created_at: u64,
    pub updated_at: u64,
    pub uploaded_at: u64,
    pub date_of_memory: Option<u64>, // when the actual event happened
    pub file_created_at: Option<u64>, // when the original file was created

    // Organization
    pub parent_folder_id: Option<String>,
    pub tags: Vec<String>,

    // Dashboard-specific fields (pre-computed)
    // ❌ REMOVED: pub is_public: bool,                   // Redundant with sharing_status
    pub shared_count: u32,                 // Count of shared recipients
    pub sharing_status: SharingStatus,     // ✅ ENUM: "public" | "shared" | "private"
    pub total_size: u64,                   // Sum of all asset sizes
    pub asset_count: u32,                  // Total number of assets
    pub thumbnail_url: Option<String>,     // Pre-computed thumbnail URL
    pub primary_asset_url: Option<String>, // Primary asset URL for display
    pub has_thumbnails: bool,              // Whether thumbnails exist
    pub has_previews: bool,                // Whether previews exist
}

// ✅ TECH LEAD APPROVED: MemoryHeader with cached UI fields only
pub struct MemoryHeader {
    pub id: String,         // UUID v7 (not compound)
    pub capsule_id: String, // Capsule context
    pub name: String,
    pub memory_type: MemoryType,
    pub size: u64,
    pub created_at: u64,
    pub updated_at: u64,
    // ❌ REMOVED: pub access: ResourceAccess, // Use capsule.access_idx instead

    // NEW: Dashboard-specific fields (pre-computed)
    pub title: Option<String>,             // From metadata
    pub description: Option<String>,       // From metadata
    // ❌ REMOVED: pub is_public: bool,                   // Redundant with sharing_status
    pub shared_count: u32,                 // Count of shared recipients (cached)
    pub sharing_status: SharingStatus,     // ✅ ENUM: "public" | "shared" | "private" (cached)
    pub total_size: u64,                   // Sum of all asset sizes
    pub asset_count: u32,                  // Total number of assets
    pub thumbnail_url: Option<String>,     // Pre-computed thumbnail URL
    pub primary_asset_url: Option<String>, // Primary asset URL for display
    pub has_thumbnails: bool,              // Whether thumbnails exist
    pub has_previews: bool,                // Whether previews exist
}
```

#### **Task 2.2: Update Memory Implementation**

**File**: `src/backend/src/memories/core/`

- Update all memory creation functions to use centralized access system instead of `MemoryAccess`
- Update memory update functions to handle `sharing_status` instead of `is_public`
- Update memory listing functions to compute `sharing_status` from access entries
- Update memory sharing functions to use `AccessEntry` system

#### **Task 2.3: Update Memory API Endpoints**

**File**: `src/backend/src/lib.rs`

- Update `memories_create` to use centralized access system instead of `MemoryAccess`
- Update `memories_update` to handle `sharing_status` changes
- Update `memories_list` to return `sharing_status` instead of `is_public`
- Update `memories_share` to use `AccessEntry` system

#### **Task 2.4: Update Memory Tests**

**File**: `src/backend/src/memories/core/`

- Update all memory tests to use centralized access system instead of `MemoryAccess`
- Update test assertions to check `sharing_status` instead of `is_public`
- Add tests for `AccessEntry` creation and management
- Add tests for `PublicPolicy` handling

### **Phase 3: Gallery Implementation**

**Module Structure**: Using modern Rust module organization without `mod.rs`:

- `src/backend/src/gallery.rs` - Main gallery implementation
- `src/backend/src/gallery/types.rs` - Gallery type definitions
- `src/backend/src/gallery/api.rs` - Gallery API endpoints
- `src/backend/src/gallery/tests.rs` - Gallery tests

#### **Task 3.1: Create Gallery Types**

**File**: `src/backend/src/gallery/types.rs`

```rust
// ✅ TECH LEAD APPROVED: Entities only cache UI fields (no embedded access)
pub struct Gallery {
    pub id: String,
    pub capsule_id: String,               // ✅ SAME AS MEMORY: Capsule context
    pub metadata: GalleryMetadata,        // ✅ includes cached shared_count/sharing_status
    pub items: Vec<GalleryItem>,          // ✅ Renamed from memory_entries to items
    pub cover_memory_id: Option<String>,  // ✅ NEW: Reference to memory ID for cover image
    pub created_at: u64,                  // ✅ From database schema
    pub updated_at: u64,                  // ✅ From database schema
    // ❌ NO embedded AccessEntry/PublicPolicy here - stored in capsule.access_idx
}

pub struct GalleryMetadata {
    pub title: Option<String>,            // ✅ User-editable title (optional - if None, use name)
    pub name: String,                     // ✅ URL-safe identifier (auto-generated, never empty)
    pub description: Option<String>,      // ✅ User-facing description (from schema.ts)

    // ✅ PRE-COMPUTED: Dashboard fields (from schema.ts)
    pub shared_count: u32,                // Count of active shares
    pub sharing_status: SharingStatus,    // ✅ ENUM: "public" | "shared" | "private"
    pub total_memories: u32,              // Count of memories

    // ✅ COMPUTED: Storage location (computed from memory storage_edges)
    pub storage_location: Vec<BlobHosting>, // ✅ COMPUTED: Where gallery memories are stored
}

pub struct GalleryHeader {
    pub id: String,
    pub title: Option<String>,            // ✅ User-editable title (optional - if None, use name)
    pub name: String,                     // ✅ URL-safe identifier (auto-generated, never empty)
    pub memory_count: u64,                // ✅ Count of memories in gallery
    pub created_at: u64,                  // ✅ From database schema
    pub updated_at: u64,                  // ✅ From database schema

    // ✅ PRE-COMPUTED: Dashboard fields (from schema.ts)
    pub shared_count: u32,                // Count of active shares
    pub sharing_status: SharingStatus,    // ✅ ENUM: "public" | "shared" | "private"
    pub total_memories: u32,              // Count of memories

    // ✅ COMPUTED: Storage location (computed from memory storage_edges)
    pub storage_location: Vec<BlobHosting>, // ✅ COMPUTED: Where gallery memories are stored
}

pub struct GalleryItem {
    pub memory_id: String,                // ✅ UUID from database (references memory)
    pub memory_type: MemoryType,          // ✅ Enum from database
    pub position: u32,                    // ✅ From database
    pub caption: Option<String>,          // ✅ From database
    pub is_featured: bool,                // ✅ From database
    pub metadata: std::collections::HashMap<String, serde_json::Value>, // ✅ From database
    // ❌ NO separate id field - identified by (gallery_id, memory_id) combination
    // ❌ NO timestamps - it's a relationship table (from database schema)
}
```

#### **Task 3.2: Create Gallery Implementation**

**File**: `src/backend/src/gallery.rs`

```rust
impl Gallery {
    pub fn compute_storage_location(&self) -> Vec<BlobHosting> {
        // ✅ OPTIMIZATION: All memories in a gallery should be in the same location
        // Just check the first memory's storage location instead of iterating through all
        if let Some(first_item) = self.items.first() {
            // Query storage_edges for the first memory only
            get_storage_locations_for_memory(&first_item.memory_id)
        } else {
            // Empty gallery - return default location
            vec![BlobHosting::S3]
        }
    }

    pub fn to_header(&self) -> GalleryHeader {
        GalleryHeader {
            id: self.id.clone(),
            title: self.metadata.title.clone(), // ✅ Optional - if None, frontend uses name
            name: self.metadata.name.clone(),
            memory_count: self.items.len() as u64,
            created_at: self.created_at,
            updated_at: self.updated_at,
            shared_count: self.metadata.shared_count,
            sharing_status: self.metadata.sharing_status.clone(),
            total_memories: self.metadata.total_memories,
            storage_location: self.compute_storage_location(), // ✅ COMPUTED
        }
    }

    pub fn add_item(&mut self, memory_id: String, memory_type: MemoryType, position: u32) {
        let item = GalleryItem {
            memory_id: memory_id.clone(),
            memory_type,
            position,
            caption: None,
            is_featured: false,
            metadata: std::collections::HashMap::new(),
        };
        self.items.push(item);
        self.metadata.total_memories += 1;
        self.updated_at = ic_cdk::api::time();
    }

    pub fn remove_memory(&mut self, memory_id: &str) {
        // If removing the cover memory, clear the cover reference
        if self.cover_memory_id.as_ref() == Some(&memory_id.to_string()) {
            self.cover_memory_id = None;
        }

        self.items.retain(|item| item.memory_id != memory_id);
        self.metadata.total_memories = self.items.len() as u32;
        self.updated_at = ic_cdk::api::time();
    }

    pub fn set_cover_memory(&mut self, memory_id: &str) -> Result<(), String> {
        // Verify the memory exists in this gallery
        if !self.items.iter().any(|item| item.memory_id == memory_id) {
            return Err("Memory not found in gallery".to_string());
        }

        self.cover_memory_id = Some(memory_id.to_string());
        self.updated_at = ic_cdk::api::time();
        Ok(())
    }

    pub fn get_cover_item(&self) -> Option<&GalleryItem> {
        self.cover_memory_id.as_ref()
            .and_then(|cover_memory_id| self.items.iter().find(|item| item.memory_id == *cover_memory_id))
    }

    // ✅ Access control is now handled by capsule.access_idx, not embedded
    // pub fn add_access_entry(&mut self, access_entry: AccessEntry) { ... }
    // pub fn remove_access_entry(&mut self, access_entry_id: &str) { ... }
}
```

#### **Task 3.3: Create Gallery API Endpoints**

**File**: `src/backend/src/gallery/api.rs`

```rust
// ✅ NEW: Gallery API endpoints
#[update]
pub async fn galleries_create(
    title: Option<String>,
    description: Option<String>,
    memory_ids: Vec<String>,
) -> Result<String, String> {
    // 1. Generate gallery name from title
    let name = generate_gallery_name(title.as_ref());

    // 2. Create gallery metadata
    let metadata = GalleryMetadata {
        title,
        name: name.clone(),
        description,
        shared_count: 0,
        sharing_status: SharingStatus::Private,
        total_memories: memory_ids.len() as u32,
        storage_location: Vec::new(), // Will be computed
    };

    // 3. Create gallery
    let gallery = Gallery {
        id: generate_uuid_v7(),
        capsule_id: get_caller_capsule_id(),
        owner_principal: ic_cdk::api::msg_caller(),
        metadata,
        memory_entries: Vec::new(),
        // ❌ REMOVED: access field - using centralized access system
        created_at: ic_cdk::api::time(),
        updated_at: ic_cdk::api::time(),
    };

    // 4. Add memories to gallery
    for (position, memory_id) in memory_ids.iter().enumerate() {
        gallery.add_item(memory_id.clone(), MemoryType::Image, position as u32);
    }

    // 5. Store gallery
    store_gallery(gallery);

    Ok(gallery.id)
}

#[query]
pub async fn galleries_list() -> Result<Vec<GalleryHeader>, String> {
    let caller_capsule_id = get_caller_capsule_id();
    let galleries = get_galleries_by_capsule(caller_capsule_id);

    let headers: Vec<GalleryHeader> = galleries.iter()
        .map(|gallery| gallery.to_header())
        .collect();

    Ok(headers)
}

#[update]
pub async fn galleries_share(
    gallery_id: String,
    person_ref: PersonRef,
    role: ResourceRole,
    perm_mask: u32,
) -> Result<String, String> {
    // 1. Get gallery
    let mut gallery = get_gallery(&gallery_id)?;

    // 2. Create access entry
    let access_entry = AccessEntry {
        id: generate_uuid_v7(),
        person_ref,
        grant_source: GrantSource::User,
        source_id: None,
        role,
        perm_mask,
        invited_by_person_ref: Some(ic_cdk::api::msg_caller()),
        created_at: ic_cdk::api::time(),
        updated_at: ic_cdk::api::time(),
    };

    // 3. Add access entry
    gallery.add_access_entry(access_entry);

    // 4. Update sharing status
    gallery.metadata.sharing_status = SharingStatus::Shared;

    // 5. Store gallery
    store_gallery(gallery);

    Ok("Gallery shared successfully".to_string())
}

#[update]
pub async fn galleries_set_cover(
    gallery_id: String,
    memory_id: String,
) -> Result<String, String> {
    // 1. Get gallery
    let mut gallery = get_gallery(&gallery_id)?;

    // 2. Set cover memory
    gallery.set_cover_memory(&memory_id)?;

    // 3. Store gallery
    store_gallery(gallery);

    Ok("Cover memory set successfully".to_string())
}
```

#### **Task 3.4: Create Gallery Tests**

**File**: `src/backend/src/gallery/tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gallery_creation() {
        let metadata = GalleryMetadata {
            title: Some("Summer Photos".to_string()),
            name: "summer-photos".to_string(),
            description: Some("Photos from summer vacation".to_string()),
            shared_count: 0,
            sharing_status: SharingStatus::Private,
            total_memories: 0,
            storage_location: Vec::new(),
        };

        let gallery = Gallery {
            id: "gallery-123".to_string(),
            capsule_id: "capsule-456".to_string(),
            metadata,
            items: Vec::new(),
            cover_memory_id: None,
            created_at: 1234567890,
            updated_at: 1234567890,
        };

        let header = gallery.to_header();
        assert_eq!(header.title, Some("Summer Photos".to_string()));
        assert_eq!(header.name, "summer-photos");
        assert_eq!(header.sharing_status, SharingStatus::Private);
    }

    #[test]
    fn test_gallery_memory_management() {
        let mut gallery = create_test_gallery();

        // Add memory
        gallery.add_item("memory-1".to_string(), MemoryType::Image, 1);
        assert_eq!(gallery.metadata.total_memories, 1);
        assert_eq!(gallery.items.len(), 1);

        // Set as cover
        gallery.set_cover_memory("memory-1").unwrap();
        assert_eq!(gallery.cover_memory_id, Some("memory-1".to_string()));

        // Remove memory
        gallery.remove_memory("memory-1");
        assert_eq!(gallery.metadata.total_memories, 0);
        assert_eq!(gallery.items.len(), 0);
        assert_eq!(gallery.cover_memory_id, None); // Cover should be cleared
    }

    #[test]
    fn test_gallery_cover_management() {
        let mut gallery = create_test_gallery();

        // Add memories
        gallery.add_item("memory-1".to_string(), MemoryType::Image, 1);
        gallery.add_item("memory-2".to_string(), MemoryType::Image, 2);

        // Set first memory as cover
        gallery.set_cover_memory("memory-1").unwrap();
        assert_eq!(gallery.cover_memory_id, Some("memory-1".to_string()));

        // Get cover item
        let cover_item = gallery.get_cover_item().unwrap();
        assert_eq!(cover_item.memory_id, "memory-1");

        // Change cover to second memory
        gallery.set_cover_memory("memory-2").unwrap();
        assert_eq!(gallery.cover_memory_id, Some("memory-2".to_string()));

        // Try to set cover to non-existent memory
        let result = gallery.set_cover_memory("non-existent");
        assert!(result.is_err());
    }
}
```

#### **Task 3.5: Update Main Library Module**

**File**: `src/backend/src/lib.rs`

```rust
// ✅ MODERN RUST: Main library module declarations
pub mod capsule;        // ✅ Access control system
pub mod gallery;        // ✅ Gallery implementation
pub mod folder;         // ✅ Folder implementation
pub mod memories;       // ✅ Memory implementation

// ✅ Re-export for convenience
pub use capsule::*;     // Access control types and functions
pub use gallery::types::*;
pub use gallery::api::*;
pub use folder::types::*;
pub use folder::api::*;
```

### **Phase 4: Folder Implementation**

**Module Structure**: Using modern Rust module organization:

- `src/backend/src/folder.rs` - Main folder implementation
- `src/backend/src/folder/types.rs` - Folder type definitions
- `src/backend/src/folder/api.rs` - Folder API endpoints
- `src/backend/src/folder/tests.rs` - Folder tests

#### **Task 4.1: Create Folder Types**

**File**: `src/backend/src/folder/types.rs`

```rust
// ✅ SIMPLIFIED: Folder struct (memories reference folder via parent_folder_id)
pub struct Folder {
    pub id: String,
    pub capsule_id: String,               // ✅ SAME AS MEMORY: Capsule context
    pub metadata: FolderMetadata,         // ✅ Consistent with Memory pattern
    // ❌ NO memory_entries - memories reference folder via parent_folder_id
    pub created_at: u64,                  // ✅ From database schema
    pub updated_at: u64,                  // ✅ From database schema
}

pub struct FolderMetadata {
    pub title: Option<String>,            // ✅ User-editable title (optional - if None, use name)
    pub name: String,                     // ✅ URL-safe identifier (auto-generated, never empty)
    pub description: Option<String>,      // ✅ User-facing description

    // ✅ PRE-COMPUTED: Dashboard fields
    pub shared_count: u32,                // Count of active shares
    pub sharing_status: SharingStatus,    // ✅ ENUM: "public" | "shared" | "private"
    pub total_memories: u32,              // Count of memories in folder (computed from memory queries)

    // ✅ COMPUTED: Storage location (computed from memory storage_edges)
    pub storage_location: Vec<BlobHosting>, // ✅ COMPUTED: Where folder memories are stored
}

pub struct FolderHeader {
    pub id: String,
    pub title: Option<String>,            // ✅ User-editable title (optional - if None, use name)
    pub name: String,                     // ✅ URL-safe identifier (auto-generated, never empty)
    pub memory_count: u64,                // ✅ Count of memories in folder (computed from memory queries)
    pub created_at: u64,                  // ✅ From database schema
    pub updated_at: u64,                  // ✅ From database schema

    // ✅ PRE-COMPUTED: Dashboard fields
    pub shared_count: u32,                // Count of active shares
    pub sharing_status: SharingStatus,    // ✅ ENUM: "public" | "shared" | "private"
    pub total_memories: u32,              // Count of memories

    // ✅ COMPUTED: Storage location (computed from memory storage_edges)
    pub storage_location: Vec<BlobHosting>, // ✅ COMPUTED: Where folder memories are stored
}

// ❌ REMOVED: FolderMemoryEntry - not needed, memories reference folder via parent_folder_id
```

#### **Task 4.2: Create Folder Implementation**

**File**: `src/backend/src/folder.rs`

```rust
impl Folder {
    pub fn compute_storage_location(&self, capsule: &Capsule) -> Vec<BlobHosting> {
        // ✅ SIMPLIFIED: Find first memory in this folder and get its storage location
        // All memories in a folder should be in the same location
        if let Some(first_memory) = capsule.memories.values()
            .find(|memory| memory.parent_folder_id.as_ref() == Some(&self.id)) {
            get_storage_locations_for_memory(&first_memory.id)
        } else {
            // Empty folder - return default location
            vec![BlobHosting::S3]
        }
    }

    pub fn compute_memory_count(&self, capsule: &Capsule) -> u32 {
        // ✅ SIMPLIFIED: Count memories that reference this folder
        capsule.memories.values()
            .filter(|memory| memory.parent_folder_id.as_ref() == Some(&self.id))
            .count() as u32
    }

    pub fn to_header(&self, capsule: &Capsule) -> FolderHeader {
        let memory_count = self.compute_memory_count(capsule);
        FolderHeader {
            id: self.id.clone(),
            title: self.metadata.title.clone(),
            name: self.metadata.name.clone(),
            memory_count: memory_count as u64,
            created_at: self.created_at,
            updated_at: self.updated_at,
            shared_count: self.metadata.shared_count,
            sharing_status: self.metadata.sharing_status.clone(),
            total_memories: memory_count,
            storage_location: self.compute_storage_location(capsule),
        }
    }

    // ✅ SIMPLIFIED: No add_memory/remove_memory methods
    // Memories are moved to folders by updating their parent_folder_id field
}
```

#### **Task 4.3: Create Folder API Endpoints**

**File**: `src/backend/src/folder/api.rs`

```rust
// ✅ SIMPLIFIED: Folder API endpoints
#[update]
pub async fn folders_create(
    title: Option<String>,
    description: Option<String>,
) -> Result<String, String> {
    // 1. Generate folder name from title
    let name = generate_folder_name(title.as_ref());

    // 2. Create folder metadata
    let metadata = FolderMetadata {
        title,
        name: name.clone(),
        description,
        shared_count: 0,
        sharing_status: SharingStatus::Private,
        total_memories: 0, // Will be computed from memory queries
        storage_location: Vec::new(), // Will be computed
    };

    // 3. Create folder
    let folder = Folder {
        id: generate_uuid_v7(),
        capsule_id: get_caller_capsule_id(),
        metadata,
        created_at: ic_cdk::api::time(),
        updated_at: ic_cdk::api::time(),
    };

    // 4. Store folder
    store_folder(folder);

    Ok(folder.id)
}

#[query]
pub async fn folders_list() -> Result<Vec<FolderHeader>, String> {
    let caller_capsule_id = get_caller_capsule_id();
    let capsule = get_capsule(caller_capsule_id)?;
    let folders = get_folders_by_capsule(caller_capsule_id);

    let headers: Vec<FolderHeader> = folders.iter()
        .map(|folder| folder.to_header(&capsule))
        .collect();

    Ok(headers)
}

#[update]
pub async fn folders_move_memory(
    folder_id: String,
    memory_id: String,
) -> Result<String, String> {
    // 1. Get capsule
    let mut capsule = get_caller_capsule()?;

    // 2. Update memory's parent_folder_id
    if let Some(memory) = capsule.memories.get_mut(&memory_id) {
        memory.parent_folder_id = Some(folder_id);
        memory.updated_at = ic_cdk::api::time();
    } else {
        return Err("Memory not found".to_string());
    }

    // 3. Store capsule
    store_capsule(capsule);

    Ok("Memory moved to folder successfully".to_string())
}
```

### **Phase 5: Capsule Implementation**

#### **Task 5.1: Create Capsule Metadata**

**File**: `src/backend/src/types.rs`

```rust
// ✅ NEW: Capsule metadata struct
pub struct CapsuleMetadata {
    pub title: Option<String>,            // ✅ User-editable title (optional - if None, use name)
    pub name: String,                     // ✅ URL-safe identifier (auto-generated, never empty)
    pub description: Option<String>,      // ✅ User-facing description

    // ✅ PRE-COMPUTED: Dashboard fields
    pub shared_count: u32,                // Count of active shares
    pub sharing_status: SharingStatus,    // ✅ ENUM: "public" | "shared" | "private"
    pub total_memories: u32,              // Count of memories in capsule
    pub total_galleries: u32,             // Count of galleries in capsule
    pub total_folders: u32,               // Count of folders in capsule

    // ✅ COMPUTED: Storage location (computed from all content storage_edges)
    pub storage_location: Vec<BlobHosting>, // ✅ COMPUTED: Where capsule content is stored
}

// ✅ TECH LEAD APPROVED: Capsule holds the indexes; entities just keep their IDs + cached counters
pub struct Capsule {
    pub id: String,
    pub subject: PersonRef,
    // content
    pub memories: HashMap<String, Memory>,
    pub galleries: HashMap<String, Gallery>,
    pub folders: HashMap<String, Folder>,
    // universal access (single source of truth)
    pub access_idx: AccessIndex,
    // cached dashboard fields (updated on writes)
    pub metadata: CapsuleMetadata,
    pub owners: HashMap<PersonRef, OwnerState>,              // 1..n owners (usually 1)
    pub controllers: HashMap<PersonRef, ControllerState>,    // delegated admins (full control)
    pub connections: HashMap<PersonRef, Connection>,         // social graph
    pub connection_groups: HashMap<String, ConnectionGroup>, // organized connection groups
    pub created_at: u64,
    pub updated_at: u64,
    pub bound_to_neon: bool,         // Neon database binding status
    pub inline_bytes_used: u64,      // Track inline storage consumption
    pub has_advanced_settings: bool, // Controls whether user sees advanced settings panels
    pub hosting_preferences: HostingPreferences, // User's preferred hosting providers
}
```

#### **Task 5.2: Update Capsule Implementation**

**File**: `src/backend/src/capsule.rs`

```rust
impl Capsule {
    pub fn compute_storage_location(&self) -> Vec<BlobHosting> {
        let mut storage_locations = std::collections::HashSet::new();

        // Get storage locations from memories
        for memory in self.memories.values() {
            // Query storage_edges for each memory
            let locations = get_storage_locations_for_memory(&memory.id);
            storage_locations.extend(locations);
        }

        // Get storage locations from galleries
        for gallery in self.galleries.values() {
            storage_locations.extend(gallery.compute_storage_location());
        }

        // Get storage locations from folders
        for folder in self.folders.values() {
            storage_locations.extend(folder.compute_storage_location());
        }

        storage_locations.into_iter().collect()
    }

    // ✅ Access system methods
    pub fn is_owner(&self, principal: &Principal) -> bool {
        self.owners.contains_key(&PersonRef::Principal(*principal))
    }

    pub fn is_controller(&self, principal: &Principal) -> bool {
        self.controllers.contains_key(&PersonRef::Principal(*principal))
    }

    pub fn update_access(&mut self, key: ResKey, entries: Vec<AccessEntry>) {
        // 1. Update access index
        self.access_idx.entries.insert(key.clone(), entries);

        // 2. Recompute cached fields on affected entities
        self.recompute_cached_fields(&key);
    }

    pub fn set_public_policy(&mut self, key: ResKey, policy: PublicPolicy) {
        // 1. Update public policy
        self.access_idx.policy.insert(key.clone(), policy);

        // 2. Recompute cached fields on affected entities
        self.recompute_cached_fields(&key);
    }

    fn recompute_cached_fields(&mut self, key: &ResKey) {
        match key.r#type {
            ResourceType::Memory => {
                if let Some(memory) = self.memories.get_mut(&key.id) {
                    self.recompute_memory_sharing_status(memory, key);
                }
            },
            ResourceType::Gallery => {
                if let Some(gallery) = self.galleries.get_mut(&key.id) {
                    self.recompute_gallery_sharing_status(gallery, key);
                }
            },
            ResourceType::Folder => {
                if let Some(folder) = self.folders.get_mut(&key.id) {
                    self.recompute_folder_sharing_status(folder, key);
                }
            },
            ResourceType::Capsule => {
                self.recompute_capsule_metadata();
            },
        }
    }

    fn recompute_memory_sharing_status(&mut self, memory: &mut Memory, key: &ResKey) {
        let entries = self.access_idx.entries.get(key).unwrap_or(&Vec::new());
        let policy = self.access_idx.policy.get(key);

        memory.metadata.shared_count = entries.len() as u32;
        memory.metadata.sharing_status = if policy.is_some() {
            SharingStatus::Public
        } else if !entries.is_empty() {
            SharingStatus::Shared
        } else {
            SharingStatus::Private
        };
    }

    fn recompute_gallery_sharing_status(&mut self, gallery: &mut Gallery, key: &ResKey) {
        let entries = self.access_idx.entries.get(key).unwrap_or(&Vec::new());
        let policy = self.access_idx.policy.get(key);

        gallery.metadata.shared_count = entries.len() as u32;
        gallery.metadata.sharing_status = if policy.is_some() {
            SharingStatus::Public
        } else if !entries.is_empty() {
            SharingStatus::Shared
        } else {
            SharingStatus::Private
        };
    }

    fn recompute_folder_sharing_status(&mut self, folder: &mut Folder, key: &ResKey) {
        let entries = self.access_idx.entries.get(key).unwrap_or(&Vec::new());
        let policy = self.access_idx.policy.get(key);

        folder.metadata.shared_count = entries.len() as u32;
        folder.metadata.sharing_status = if policy.is_some() {
            SharingStatus::Public
        } else if !entries.is_empty() {
            SharingStatus::Shared
        } else {
            SharingStatus::Private
        };
    }

    fn recompute_capsule_metadata(&mut self) {
        // Update capsule-level metadata based on access changes
        self.metadata.total_memories = self.memories.len() as u32;
        self.metadata.total_galleries = self.galleries.len() as u32;
        self.metadata.total_folders = self.folders.len() as u32;
        self.metadata.storage_location = self.compute_storage_location();

        // Count total shared resources
        let mut total_shared = 0;
        for entries in self.access_idx.entries.values() {
            total_shared += entries.len();
        }
        self.metadata.shared_count = total_shared as u32;

        // Update sharing status based on access entries
        if !self.access_idx.policy.is_empty() {
            self.metadata.sharing_status = SharingStatus::Public;
        } else if total_shared > 0 {
            self.metadata.sharing_status = SharingStatus::Shared;
        } else {
            self.metadata.sharing_status = SharingStatus::Private;
        }

        self.updated_at = ic_cdk::api::time();
    }
}
```

#### **Task 5.3: Update Capsule API Endpoints**

**File**: `src/backend/src/lib.rs`

```rust
// ✅ NEW: Universal access system API endpoints
#[update]
pub async fn resource_share(
    resource_type: ResourceType,
    resource_id: String,
    person_ref: PersonRef,
    role: ResourceRole,
    perm_mask: u32,
) -> Result<ShareResult, String> {
    // 1. Get caller's capsule
    let mut capsule = get_caller_capsule()?;

    // 2. Create access entry
    let access_entry = AccessEntry {
        id: generate_uuid_v7(),
        person_ref,
        grant_source: GrantSource::User,
        source_id: None,
        role,
        perm_mask,
        invited_by_person_ref: Some(PersonRef::Principal(ic_cdk::api::caller())),
        created_at: ic_cdk::api::time(),
        updated_at: ic_cdk::api::time(),
    };

    // 3. Create resource key
    let key = ResKey {
        r#type: resource_type,
        id: resource_id,
    };

    // 4. Get existing entries or create new
    let mut entries = capsule.access_idx.entries.get(&key).unwrap_or(&Vec::new()).clone();
    entries.push(access_entry);

    // 5. Update access index
    capsule.update_access(key, entries);

    // 6. Store capsule
    store_capsule(capsule);

    Ok(ShareResult {
        perm_mask,
        version: ic_cdk::api::time(),
        success: true,
    })
}

#[update]
pub async fn resource_set_public_policy(
    resource_type: ResourceType,
    resource_id: String,
    mode: PublicMode,
    perm_mask: u32,
) -> Result<PublicPolicyResult, String> {
    // 1. Get caller's capsule
    let mut capsule = get_caller_capsule()?;

    // 2. Create public policy
    let policy = PublicPolicy {
        mode,
        perm_mask,
        created_at: ic_cdk::api::time(),
        updated_at: ic_cdk::api::time(),
    };

    // 3. Create resource key
    let key = ResKey {
        r#type: resource_type,
        id: resource_id,
    };

    // 4. Set public policy
    capsule.set_public_policy(key, policy);

    // 5. Store capsule
    store_capsule(capsule);

    Ok(PublicPolicyResult {
        perm_mask,
        version: ic_cdk::api::time(),
        success: true,
    })
}

#[query]
pub async fn resource_get_effective_permissions(
    resource_type: ResourceType,
    resource_id: String,
    principal: Principal,
) -> Result<u32, String> {
    // 1. Get capsule
    let capsule = get_caller_capsule()?;

    // 2. Create resource key
    let key = ResKey {
        r#type: resource_type,
        id: resource_id,
    };

    // 3. Create principal context
    let ctx = PrincipalContext {
        principal,
        groups: Vec::new(), // TODO: Get from capsule connections
        link: None, // TODO: Extract from request if provided
        now_ns: ic_cdk::api::time(),
    };

    // 4. Compute effective permissions
    let perm_mask = effective_perm_mask(&key, &ctx, &capsule.access_idx, &capsule, ctx.now_ns);

    Ok(perm_mask)
}
```

## **🚨 CRITICAL PRE-FLIGHT CHECKS**

**⚠️ MUST COMPLETE BEFORE GOING LIVE:**

### **1. Close the TODOs (Must-Have)**

#### **Task: Implement Magic Link Path**

```rust
// File: src/backend/src/capsule/access.rs
fn link_mask_if_valid(key: &ResKey, token: &str, idx: &AccessIndex, now_ns: u64) -> u32 {
    // TODO: Implement magic link validation
    // - Hash token with HMAC salt/pepper (aligned with Web2)
    // - Check TTL expiry
    // - Check use limits (max_uses)
    // - Check revoked_at timestamp
    // - Return perm_mask if valid, 0 if invalid
    0
}
```

#### **Task: Finish Groups Implementation**

```rust
// File: src/backend/src/capsule/access.rs
fn sum_user_and_groups(entries: &[AccessEntry], ctx: &PrincipalContext) -> u32 {
    let mut mask = 0u32;
    for entry in entries {
        if entry.person_ref == PersonRef::Principal(ctx.principal) {
            mask |= entry.perm_mask;
        }
        // TODO: Add group membership checks
        // - Resolve group IDs to perm masks
        // - Check if ctx.principal is member of group
        // - Add group permissions to mask
    }
    mask
}
```

#### **Task: Ownership Fast-Path Unit Tests**

```rust
// File: src/backend/src/capsule/tests.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ownership_fast_path_memory() {
        // Test owner/controller access for Memory resources
    }

    #[test]
    fn test_ownership_fast_path_gallery() {
        // Test owner/controller access for Gallery resources
    }

    #[test]
    fn test_ownership_fast_path_folder() {
        // Test owner/controller access for Folder resources
    }

    #[test]
    fn test_ownership_fast_path_capsule() {
        // Test owner/controller access for Capsule resources
    }
}
```

### **2. Versioning & Idempotency**

#### **Task: Add Capsule Version Counter**

```rust
// File: src/backend/src/capsule/types.rs
pub struct Capsule {
    pub id: String,
    pub subject: PersonRef,
    // ... existing fields ...
    pub access_idx: AccessIndex,
    pub version: u64,  // ✅ NEW: Monotonic version counter
    // ... rest of fields ...
}

impl Capsule {
    pub fn increment_version(&mut self) {
        self.version += 1;
    }
}
```

#### **Task: Idempotent Upsert Key**

```rust
// File: src/backend/src/capsule/access.rs
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccessEntryKey {
    pub res_key: ResKey,
    pub person_ref: PersonRef,
    pub grant_source: GrantSource,
    pub source_id: Option<String>,
}

// Use this as the key for idempotent upserts
pub fn upsert_access_entry(
    capsule: &mut Capsule,
    key: AccessEntryKey,
    entry: AccessEntry,
) -> Result<u64, String> {
    // 1. Check if entry already exists with same key
    // 2. If exists, update; if not, insert
    // 3. Increment version
    // 4. Return new version
}
```

### **3. Stable Memory Safety**

#### **Task: Cap Unbounded Vec<AccessEntry>**

```rust
// File: src/backend/src/capsule/access.rs
// Option A: One row per grantee (RECOMMENDED)
#[derive(Serialize, Deserialize, Default)]
pub struct AccessIndex {
    // (ResKey, Principal) -> AccessEntry (one per grantee)
    pub entries: StableBTreeMap<(ResKey, Principal), AccessEntry>,
    // (ResKey, GroupId) -> AccessEntry (group grants)
    pub group_entries: StableBTreeMap<(ResKey, String), AccessEntry>,
    // (ResKey) -> PublicPolicy
    pub policy: StableBTreeMap<ResKey, PublicPolicy>,
}

// Option B: Keep Vec but enforce hard limit
const MAX_ACCESS_ENTRIES_PER_RESOURCE: usize = 100;

pub fn add_access_entry(
    capsule: &mut Capsule,
    res_key: ResKey,
    entry: AccessEntry,
) -> Result<(), String> {
    let entries = capsule.access_idx.entries.get(&res_key).unwrap_or(&Vec::new());
    if entries.len() >= MAX_ACCESS_ENTRIES_PER_RESOURCE {
        return Err("Too many access entries for this resource".to_string());
    }
    // Add entry...
}
```

### **4. Time Normalization**

#### **Task: Pick One Time Strategy**

```rust
// File: src/backend/src/capsule/time.rs
// Strategy: Compare in ns inside canister, convert on boundary

pub fn is_expired(created_at_ns: u64, ttl_ns: u64) -> bool {
    now_icp_ns() > created_at_ns + ttl_ns
}

// Add boundary tests for expiry at exact now
#[cfg(test)]
mod tests {
    #[test]
    fn test_expiry_at_exact_now() {
        let now = now_icp_ns();
        let created_at = now - 1000; // 1 microsecond ago
        let ttl = 1000; // 1 microsecond TTL

        // Should be expired
        assert!(is_expired(created_at, ttl));
    }
}
```

### **5. Consistency on Write**

#### **Task: Wire Cache Recomputation**

```rust
// File: src/backend/src/capsule.rs
impl Capsule {
    pub fn update_access(&mut self, key: ResKey, entries: Vec<AccessEntry>) {
        // 1. Update access index
        self.access_idx.entries.insert(key.clone(), entries);

        // 2. ✅ CRITICAL: Recompute cached fields on affected entities
        self.recompute_cached_fields(&key);

        // 3. Increment version
        self.increment_version();
    }

    fn recompute_cached_fields(&mut self, key: &ResKey) {
        // ✅ ENSURE THIS IS WIRED: Update shared_count/sharing_status
        match key.r#type {
            ResourceType::Memory => {
                if let Some(memory) = self.memories.get_mut(&key.id) {
                    self.recompute_memory_sharing_status(memory, key);
                }
            },
            ResourceType::Gallery => {
                if let Some(gallery) = self.galleries.get_mut(&key.id) {
                    self.recompute_gallery_sharing_status(gallery, key);
                }
            },
            ResourceType::Folder => {
                if let Some(folder) = self.folders.get_mut(&key.id) {
                    self.recompute_folder_sharing_status(folder, key);
                }
            },
            ResourceType::Capsule => {
                self.recompute_capsule_metadata();
            },
        }
    }
}
```

### **6. Migration/Cutover**

#### **Task: Backfill Script**

```rust
// File: scripts/migrate_access_system.rs
pub fn migrate_old_to_new_access_system() -> Result<(), String> {
    // 1. Read old access fields from all entities
    // 2. Convert to new (ResKey -> entries/policy) format
    // 3. Populate AccessIndex
    // 4. Recompute all cached fields
    // 5. Validate migration
}
```

#### **Task: Shadow Read Validation**

```rust
// File: src/backend/src/capsule/validation.rs
pub fn shadow_read_validation(
    key: &ResKey,
    ctx: &PrincipalContext,
    capsule: &Capsule,
) -> Result<(), String> {
    // 1. Compute old effective mask (from old fields)
    let old_mask = compute_old_effective_mask(key, ctx, capsule);

    // 2. Compute new effective mask (from AccessIndex)
    let new_mask = effective_perm_mask(key, ctx, &capsule.access_idx, capsule, ctx.now_ns);

    // 3. Log differences
    if old_mask != new_mask {
        ic_cdk::println!("MISMATCH: key={:?}, old={}, new={}", key, old_mask, new_mask);
    }

    Ok(())
}
```

## **✅ NICE-TO-HAVES (Don't Block GA)**

### **Role Templates Persisted Per Capsule**

```rust
// File: src/backend/src/capsule/types.rs
pub struct Capsule {
    // ... existing fields ...
    pub role_templates: HashMap<String, RoleTemplate>, // ✅ Editable via admin endpoint
}
```

### **Token Hashing Strategy Aligned with Web2**

```rust
// File: src/backend/src/capsule/access.rs
fn hash_magic_link_token(token: &str) -> String {
    // Use same HMAC salt/pepper as Web2 system
    // Ensure compatibility for cross-platform sharing
}
```

### **Optional "Download Originals" Extra Bit**

```rust
// File: src/backend/src/capsule/types.rs
bitflags! {
    pub struct Perm: u32 {
        const VIEW = 1 << 0;
        const DOWNLOAD = 1 << 1;
        const SHARE = 1 << 2;
        const MANAGE = 1 << 3;
        const OWN = 1 << 4;
        const DOWNLOAD_ORIGINALS = 1 << 5; // ✅ Future pricing lever
    }
}
```

## **✅ DRIZZLE PARITY**

**Web2 side is already expressible with plain Drizzle (no generated cols/views). Keep `perm_mask` + helpers in TS.**

```typescript
// File: src/nextjs/src/lib/access.ts
export const PERM_VIEW = 1 << 0;
export const PERM_DOWNLOAD = 1 << 1;
export const PERM_SHARE = 1 << 2;
export const PERM_MANAGE = 1 << 3;
export const PERM_OWN = 1 << 4;

export function hasPermission(mask: number, perm: number): boolean {
  return (mask & perm) !== 0;
}
```

## **🎯 VERDICT: GREEN LIGHT ONCE (1) AND (3) ARE DONE**

**Critical Path:**

1. ✅ Implement magic link validation
2. ✅ Finish groups implementation
3. ✅ Add ownership fast-path unit tests
4. ✅ Cap unbounded Vec<AccessEntry> for stable memory safety

**Then: Deploy with confidence! 🚀**

## Testing Strategy

### **Unit Tests**

- Test all new types and enums
- Test access entry creation and management
- Test storage location computation
- Test title/name logic
- Test sharing status computation

### **Integration Tests**

- Test gallery creation with memories
- Test folder creation with memories
- Test capsule sharing with access entries
- Test storage location computation across all entities
- Test universal access system across all entities

### **End-to-End Tests**

- Test complete gallery workflow (create, share, access)
- Test complete folder workflow (create, share, access)
- Test complete capsule workflow (share, access)
- Test cross-entity sharing (gallery shared with folder access)

## Migration Strategy

### **Database Migration**

- Update `galleries` table to remove `is_public` and add `sharing_status`
- Update `memories` table to remove `is_public` and add `sharing_status`
- Add `folders` table with new structure
- Update `capsules` table to add metadata fields

### **Backward Compatibility**

- Keep old API endpoints with deprecation warnings
- Provide migration scripts for existing data
- Maintain old field names in API responses during transition period

## Success Criteria

- [ ] All entities (Memory, Gallery, Folder, Capsule) use universal access system
- [ ] All entities have consistent metadata structure
- [ ] All entities use `sharing_status` instead of `is_public`
- [ ] All entities have computed storage location
- [ ] All entities follow title/name pattern
- [ ] All tests pass
- [ ] No breaking changes to existing functionality
- [ ] Performance is maintained or improved

## Timeline

- **Week 1**: Phase 1 (Access Refactoring)
- **Week 2**: Phase 2 (Memory Tweaks)
- **Week 3**: Phase 3 (Gallery Implementation)
- **Week 4**: Phase 4 (Folder Implementation)
- **Week 5**: Phase 5 (Capsule Implementation)
- **Week 6**: Testing and Migration

**Total Estimated Time**: 6 weeks

## Architectural Decision: Tech Lead's Centralized Access Control Within Capsules

### **Tech Lead's Clarification:**

The tech lead has clarified the architecture: **Centralize access in per-capsule indexes instead of embedding permission rows inside every Memory/Gallery/Folder**.

### **Why This Approach:**

1. **✅ Single source of truth** → fewer writes, no drift
2. **✅ Fast reads** with one evaluation path (no scatter across entities)
3. **✅ Clean revocation/audit** by grant source (user/group/link/public)
4. **✅ Drizzle ↔ ICP symmetry**: Web2 has `resource_membership` table; ICP has `access_idx` keyed by `(type,id)`

### **Tech Lead's Design:**

1. **✅ Bitflags for Permissions**: Use `bitflags` crate with identical bits in TS and Rust
2. **✅ Centralized Access Index**: `capsule.access_idx` with `ResKey` → `Vec<AccessEntry>` and `PublicPolicy`
3. **✅ Pure Evaluation Pipeline**: Single `effective_perm_mask()` function using `ResKey`
4. **✅ Time Normalization**: Handle ns (ICP) vs ms (Neon) at module boundaries
5. **✅ Cached UI Fields**: `sharing_status` is computed from indexes, not stored
6. **✅ Write Flows**: Mutate `capsule.access_idx`, then recompute cached fields
7. **✅ Idempotent APIs**: Small request structs with upserts and version numbers
8. **✅ Roles as Data**: Configurable role-to-permission mapping per capsule
9. **✅ Magic Link Security**: Store hashes, not raw tokens
10. **✅ Deterministic Slug Generation**: Guarantee uniqueness per capsule

### **Key Structural Changes:**

1. **✅ Removed embedded access from entities**: No more `pub access: ResourceAccess` in Memory/Gallery/Folder
2. **✅ Added centralized access index**: `capsule.access_idx: AccessIndex` with `ResKey` → access data
3. **✅ Entities only cache UI fields**: `shared_count`, `sharing_status` computed from access index
4. **✅ Write flows**: Mutate `capsule.access_idx`, then recompute cached fields
5. **✅ Permission evaluation**: Use `ResKey` to look up access in centralized index
6. **✅ Improved Gallery design**: `GalleryItem` without separate IDs, using `cover_memory_id` reference
7. **✅ Storage location optimization**: Gallery storage computed from first memory only (all memories in same location)

### **Performance Benefits:**

- **Fast Permission Checks**: O(1) ownership lookup, efficient bitmask operations
- **No Data Duplication**: Single source of truth in centralized access index
- **Cached Computations**: UI fields computed on writes, not reads
- **Efficient Storage**: No embedded access data in every entity
- **Optimized Gallery Storage**: O(1) storage location lookup (check first memory only)

### **Maintainability Benefits:**

- **Single Evaluation Logic**: One function for all permission checks using `ResKey`
- **Consistent Time Handling**: Normalized at module boundaries
- **Idempotent Operations**: Safe to retry API calls
- **Configurable Roles**: Can be updated without redeployment

### **Security Benefits:**

- **Token Hashing**: Magic links stored as hashes, not raw tokens
- **Least Privilege**: Public access never exceeds role grants
- **Audit Trail**: All access changes tracked with provenance
