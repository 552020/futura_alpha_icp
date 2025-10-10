# Capsule Access Refactoring - Phase 1 Implementation

**Status**: `UPDATED` - Tech Lead Decision Applied  
**Priority**: `HIGH` - Foundation for Universal Access System  
**Assigned**: Backend Developer  
**Created**: 2024-12-19  
**Updated**: 2024-12-19 (Tech Lead Decision)  
**Related Issues**: [Gallery Type Refactor - Implementation Plan](./gallery-type-refactor-implementation.md)

## Overview

This document outlines Phase 1 of the gallery type refactor implementation, focusing specifically on implementing the universal access control system within capsules.

**🎯 DECISION**: Use **decentralized access control** (access lives on each resource) as the source of truth, with optional minimal centralized indices for specific queries only.

## Architecture Decision

**🎯 DECISION**: Use **decentralized access control** instead of the centralized approach originally planned.

### **Why Decentralized Approach:**

> **TL;DR**: Use decentralized (access lives on each resource) as the **source of truth**. Keep a **small, optional centralized index** as a derived cache for the few global queries you actually need. This gives you simplicity, shardability, and predictable upgrades—without losing speed where it matters.

**Key Reasons:**

- **Canister boundaries**: Capsules can (and likely will) shard across canisters. Centralized access maps fight that; per-resource access scales naturally.
- **Upgrades & data migration**: Decentralized structs are easier to evolve with versioned fields; a big cross-resource index is brittle.
- **Cycles**: Central map + heavy serialization amplifies write costs; local writes are cheaper. Build indices only when the product needs them.
- **Consistency**: Keep one authority (resource). Indices are derived and can be rebuilt or lazily repaired.

### **When Centralized Index is Justified:**

- "List everything user U can access" across many resources.
- Bulk revocation / auditing across a capsule.
- Magic-link lookup by hash → resource (fast-path map).

If you need these, keep a **small** `StableBTreeMap` per capsule for those keys only.

### **Architecture Alignment:**

The design aligns with ICP's per-capsule autonomy:

- **Each capsule is an autonomous canister**
- **Memories are stored INSIDE the capsule** (`pub memories: HashMap<String, Memory>`)
- **No global access control** - each capsule manages its own access
- **No cross-capsule resource sharing** in current design

## New Implementation Plan (Decentralized)

### **Minimal Design (Hybrid, Local-First)**

1. **On each resource:**

```rust
pub struct AccessEntry { /* … */ }
pub struct PublicPolicy { /* … */ }

pub struct Memory {
  id: String,
  access_entries: Vec<AccessEntry>,
  public_policy: Option<PublicPolicy>,
  // …
}

pub struct Gallery {
  id: String,
  access_entries: Vec<AccessEntry>,
  public_policy: Option<PublicPolicy>,
  // …
}
```

2. **Central, optional derived indices (non-authoritative):**

- `link_hash -> { res_id, perm_mask, expires_at }`
- (optional) `principal -> SmallVec<ResId>` if you truly need "what can U access?"

3. **Single permission function reused everywhere:**

```rust
trait AccessControlled {
  fn access_entries(&self) -> &[AccessEntry];
  fn public_policy(&self) -> Option<&PublicPolicy>;
}
```

Keep evaluation logic shared; don't duplicate it.

### **Implementation Plan (Direct)**

1. Add access fields to Memory/Gallery structs
2. Implement AccessControlled trait
3. Create shared permission evaluation logic
4. Add optional magic link index if needed
5. No migration needed - direct implementation

### **Storage Structure Considerations**

**Important**: If your resources live inside a single `StableCell<Capsule>`, each ACL tweak rewrites a big blob. Consider:

- **Prefer per-resource stable structures** (e.g., `StableBTreeMap` of `id → resource`)
- **Or keep resources in heap** and persist only what you must
- **This plan assumes** "resources in heap + stable snapshots" or "per-resource stable" - make that explicit

### **About ResKey**

Drop it from domain logic. If you keep a small index, use a simple `(ResourceType, id)` only for index keys. The resource itself already carries its ID.

### **Complexity & Code Size**

- **Core**: ~30–60 lines added on each resource struct + 1 shared evaluator
- **Optional**: ~50–100 line index module for magic links only
- **Removed**: Most of complex `access.rs` (index plumbing)

## Implementation Tasks (Updated)

### **Task 1.1: Implement Bitflags for Permissions**

**File**: `src/backend/src/capsule/domain.rs` (consolidated with existing domain types)

```rust
// ✅ APPROVED: Bitflags for permissions (single source of truth)
// ✅ CAVEAT: Enable serde feature for bitflags in Cargo.toml
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

// ✅ APPROVED: Role templates as data (stored in capsule)
pub struct RoleTemplate {
    pub name: String,
    pub perm_mask: u32,  // Uses Perm bits
    pub description: String,
}

// ✅ APPROVED: Default role templates
impl Default for RoleTemplate {
    fn default() -> Self {
        Self {
            name: "member".to_string(),
            perm_mask: (Perm::VIEW | Perm::DOWNLOAD).bits(),
            description: "Standard member access".to_string(),
        }
    }
}

// ✅ APPROVED: Role templates for common roles
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

### **Task 1.2: Create Universal Access System Types**

**File**: `src/backend/src/capsule/domain.rs` (consolidated with existing domain types)

```rust
// ✅ UPDATED: Decentralized access control system (access lives on each resource)
// ✅ NEW APPROACH: AccessEntry and PublicPolicy will be embedded in each resource

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

### **Task 1.3: Update Cargo.toml Dependencies**

**File**: `src/backend/Cargo.toml`

```toml
# ✅ CAVEAT: Enable serde feature for bitflags
bitflags = { version = "2.4", features = ["serde"] }
```

**Note**: All types are consolidated in `capsule/domain.rs`. No separate `types.rs` file needed.

## Implementation Details

### **Task 1.4: Create AccessControlled Trait and Shared Evaluation Logic**

**File**: `src/backend/src/capsule/domain.rs`

```rust
// ✅ NEW APPROACH: Shared trait for all resources with access control
pub trait AccessControlled {
    fn access_entries(&self) -> &[AccessEntry];
    fn public_policy(&self) -> Option<&PublicPolicy>;
}

// ✅ NEW APPROACH: Single permission evaluation function
pub fn effective_perm_mask<T: AccessControlled>(
    resource: &T,
    ctx: &PrincipalContext,
    now_ns: u64,
) -> u32 {
    use Perm as P;

    // 1) Ownership fast-path - owners get everything
    if is_owner(resource, ctx) {
        return (P::VIEW | P::DOWNLOAD | P::SHARE | P::MANAGE | P::OWN).bits();
    }

    let mut m = 0u32;

    // 2) Direct grants - check individual access entries
    m |= sum_user_and_groups(resource.access_entries(), ctx);

    // 3) Magic link - check if valid magic link is presented
    if let Some(token) = &ctx.link {
        m |= link_mask_if_valid(resource, token, now_ns);
    }

    // 4) Public policy - check public access rules
    m |= public_mask_if_any(resource.public_policy(), ctx);

    m
}

// ✅ NEW APPROACH: Helper to avoid perm_mask/u32 leaks
pub fn has_perm<T: AccessControlled>(res: &T, ctx: &PrincipalContext, want: Perm) -> bool {
    (effective_perm_mask(res, ctx, ctx.now_ns) & want.bits()) != 0
}
```

### **Task 1.5: Add Access Fields to Memory and Gallery**

**Files**: `src/backend/src/memories/types.rs`, `src/backend/src/types.rs`

```rust
// ✅ NEW APPROACH: Add access fields directly to Memory struct
pub struct Memory {
    pub id: String,
    pub capsule_id: String,
    pub metadata: MemoryMetadata,
    pub access: MemoryAccess,                    // Keep existing for backward compatibility
    pub access_entries: Vec<AccessEntry>,        // ✅ NEW: Direct access control
    pub public_policy: Option<PublicPolicy>,     // ✅ NEW: Public access rules
    pub inline_assets: Vec<MemoryAssetInline>,
    pub blob_internal_assets: Vec<MemoryAssetBlobInternal>,
    pub blob_external_assets: Vec<MemoryAssetBlobExternal>,
}

// ✅ NEW APPROACH: Add access fields directly to Gallery struct
pub struct Gallery {
    pub id: String,
    pub owner_principal: Principal,
    pub title: String,
    pub description: Option<String>,
    pub is_public: bool,                         // Keep existing for backward compatibility
    pub access_entries: Vec<AccessEntry>,        // ✅ NEW: Direct access control
    pub public_policy: Option<PublicPolicy>,     // ✅ NEW: Public access rules
    pub created_at: u64,
    pub updated_at: u64,
    pub storage_location: GalleryStorageLocation,
    pub memory_entries: Vec<GalleryMemoryEntry>,
    pub bound_to_neon: bool,
}
```

### **Task 1.6: Create Optional Magic Link Index (Optional)**

**File**: `src/backend/src/capsule/access.rs` (minimal)

```rust
// ✅ NEW APPROACH: Truly optional index - only for magic links
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap, Storable,
};
use std::cell::RefCell;

thread_local! {
    static MEM_MGR: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

// Simple key for magic links
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize, CandidType)]
pub struct LinkHash(String);

impl Storable for LinkHash {
    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 64,  // Hash size
            is_fixed_size: true,
        };

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        self.0.as_bytes().into()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Self(String::from_utf8(bytes.to_vec()).unwrap())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, CandidType)]
pub struct LinkEntry {
    pub resource_type: ResourceType,
    pub resource_id: String,
    pub perm_mask: u32,
    pub expires_at: u64,
}

// ✅ NEW APPROACH: Single stable map for magic links only
pub struct MagicLinkIndex {
    pub links: StableBTreeMap<LinkHash, LinkEntry, VirtualMemory<DefaultMemoryImpl>>,
}

impl MagicLinkIndex {
    pub fn init() -> Self {
        MEM_MGR.with(|mgr| {
            let mgr = mgr.borrow();
            let links_mem = mgr.get(MemoryId::new(0));  // Single memory region

            Self {
                links: StableBTreeMap::new(links_mem),
            }
        })
    }
}
```

### **Task 1.7: Create Time Normalization Utilities**

**File**: `src/backend/src/capsule/time.rs`

```rust
// ✅ APPROVED: Handle ns (ICP) vs ms (Neon) time units
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

---

## Implementation Tasks (Final - Clean Version)

### **Task 1 – Core Types and Bitflags** ✅ **COMPLETED**

- ✅ Define `Perm`, `RoleTemplate`, and `get_default_role_templates()`
- ✅ Consolidate all enums and structs (`AccessEntry`, `PublicPolicy`, `ResourceRole`, `PublicMode`, etc.) in `capsule/domain.rs`

### **Task 2 – AccessControlled Trait + Shared Evaluator** ✅ **COMPLETED**

**File**: `src/backend/src/capsule/domain.rs`

- ✅ Implement:

```rust
  pub trait AccessControlled {
      fn access_entries(&self) -> &[AccessEntry];
      fn public_policy(&self) -> Option<&PublicPolicy>;
  }

  pub fn effective_perm_mask<T: AccessControlled>(
      resource: &T,
      ctx: &PrincipalContext,
  ) -> u32 { /* ... */ }

  pub fn has_perm<T: AccessControlled>(
      res: &T,
      ctx: &PrincipalContext,
      want: Perm
  ) -> bool {
      (effective_perm_mask(res, ctx) & want.bits()) != 0
  }
```

### **Task 3 – Integrate in Resource Structs** ✅ **COMPLETED**

✅ Add access fields:

```rust
pub struct Memory {
    pub id: String,
    pub access_entries: Vec<AccessEntry>,
    pub public_policy: Option<PublicPolicy>,
    // …
}

pub struct Gallery {
    pub id: String,
    pub access_entries: Vec<AccessEntry>,
    pub public_policy: Option<PublicPolicy>,
    // …
}
```

### **Task 4 – PrincipalContext** ✅ **COMPLETED**

✅ Implemented in `src/backend/src/capsule/domain.rs`:

```rust
pub struct PrincipalContext {
    pub principal: Principal,
    pub groups: Vec<String>,
    pub link: Option<String>,
    pub now_ns: u64, // use ic_cdk::api::time()
}
```

### **Task 5 – Time Utilities** ✅ **COMPLETED**

✅ Keep nanosecond precision and conversion helpers (`icp_time_to_neon_ms`, `is_expired`, etc.).

### **Task 6 – Optional Magic Link Index**

If needed later:

- Create `LinkHash` and `LinkEntry` structs.
- Implement minimal `StableBTreeMap` using a reserved `MemoryId`.
- Add `link_mask_if_valid()` using hash + expiry check.

### **Task 7 – Testing & Validation**

- Unit tests for `effective_perm_mask` + `has_perm`
- Test `PublicPolicy` combinations (`Private`, `PublicAuth`, `PublicLink`)
- Validate capsule upgrades: access entries survive as part of resource structs

## Success Criteria (Final – Decentralized Core + Optional Index)

### **Core (Decentralized Access Control)**

| Area                       | Description                                                                                                | Status |
| -------------------------- | ---------------------------------------------------------------------------------------------------------- | ------ |
| **AccessControlled Trait** | Shared trait for all resources with access control (`access_entries()` + `public_policy()`)                | [x]    |
| **Resource Integration**   | Add `Vec<AccessEntry>` and `Option<PublicPolicy>` directly to `Memory`, `Gallery`, `Folder`, and `Capsule` | [ ]    |
| **Permission Evaluation**  | Implement `effective_perm_mask()` and `has_perm()` helper functions (pure, deterministic)                  | [x]    |
| **PrincipalContext**       | Contains `principal`, `groups`, `link`, and `now_ns` (single time source)                                  | [ ]    |
| **Bitflags Permissions**   | Implement `Perm` bitflags (`VIEW`, `DOWNLOAD`, `SHARE`, `MANAGE`, `OWN`)                                   | [x]    |
| **Role Templates**         | Provide predefined roles (`owner`, `admin`, `member`, `guest`) as data, not code                           | [x]    |
| **Time Utilities**         | Normalize to ns across canisters; convert at API edges                                                     | [x]    |
| **Storage Model**          | Access lives _with the resource_, stored in the capsule's existing stable structure                        | [ ]    |
| **Heap Cache (Optional)**  | Allow temporary in-memory cache for fast permission reads (rebuildable)                                    | [ ]    |

**Outcome:**

- Access control is **self-contained inside each resource**
- Evaluation logic is **shared and consistent**
- No redundant or overlapping stable memory structures

---

### **Optional (Small Centralized Index for Magic Links)**

| Area                   | Description                                                                              | Status |
| ---------------------- | ---------------------------------------------------------------------------------------- | ------ |
| **LinkHash Index**     | `StableBTreeMap<LinkHash, LinkEntry, DefaultMemoryImpl>` (for `magic_link` lookups only) | [ ]    |
| **LinkEntry Struct**   | `{ resource_type, resource_id, perm_mask, expires_at }`                                  | [ ]    |
| **Token Hashing**      | `link_mask_if_valid()` hashes the token, checks expiration, returns mask                 | [ ]    |
| **MemoryManager Slot** | Assign one `MemoryId` for the link index; isolate via VirtualMemory                      | [ ]    |

**Outcome:**

- Optional persistent lookup for `magic_link` tokens
- Everything else (users, groups) stays decentralized

### **Quality Assurance**

- [ ] All tests pass
- [ ] Clean compilation with no errors
- [ ] No clippy warnings
- [ ] Documentation updated

---

## Design Summary

| Aspect                   | Decision                                                |
| ------------------------ | ------------------------------------------------------- |
| **Access Model**         | Decentralized (per resource)                            |
| **Authority**            | Resource is the source of truth                         |
| **Indices**              | Optional, minimal (magic link only)                     |
| **Serialization**        | Candid (`#[derive(CandidType, Deserialize, Serialize]`) |
| **Time Units**           | Nanoseconds (ICP standard)                              |
| **Storage Scope**        | Within each capsule canister                            |
| **Upgrades**             | Stable-compatible (no migration required)               |
| **Performance Strategy** | Optional heap cache, no persistent global maps          |

---

## Clean Architecture Summary

```
┌───────────────────────────────┐
│ Capsule (Canister)            │
│ ┌───────────────────────────┐ │
│ │ Memory                    │ │
│ │  ├─ access_entries[]      │ │
│ │  ├─ public_policy         │ │
│ │  └─ AccessControlled impl │ │
│ ├───────────────────────────┤ │
│ │ Gallery                   │ │
│ │  ├─ access_entries[]      │ │
│ │  ├─ public_policy         │ │
│ │  └─ AccessControlled impl │ │
│ ├───────────────────────────┤ │
│ │ (Optional) MagicLinkIndex │ │
│ └───────────────────────────┘ │
└───────────────────────────────┘
```

---

## Comparison: What Changed from Centralized Plan

| Component                 | Old (Centralized)                   | New (Decentralized)        | Reason                 |
| ------------------------- | ----------------------------------- | -------------------------- | ---------------------- |
| **Access Storage**        | `AccessIndex` with `StableBTreeMap` | Direct fields on resources | Simpler, more scalable |
| **Resource Keys**         | `ResKey` struct for identification  | Resource ID directly       | Less abstraction       |
| **Memory Management**     | Complex `MemoryManager` setup       | Simple resource fields     | Reduced complexity     |
| **Permission Evaluation** | Centralized index lookups           | Direct resource access     | Better performance     |
| **Magic Links**           | Part of main index                  | Optional separate index    | Cleaner separation     |
| **Migration**             | Dual-write pattern required         | Direct implementation      | No migration needed    |
| **Code Size**             | ~200+ lines of access code          | ~30-60 lines per resource  | Significant reduction  |

## Dependencies

- [Capsule Module Architecture](../../architecture/capsule-module-architecture.md) - Foundation module structure
- [Gallery Type Refactor Implementation](./gallery-type-refactor-implementation.md) - Main implementation plan

## Next Steps

Once Phase 1 is complete, proceed to:

- **Phase 2**: Memory Implementation
- **Phase 3**: Gallery Implementation
- **Phase 4**: Folder Implementation
- **Phase 5**: Capsule Implementation

## Archived: Original Centralized Approach

The original centralized approach (Tasks 1.4-1.8 in the sections above) is preserved for future reference. This approach used:

- `ResKey` struct for resource identification
- `AccessIndex` with `StableBTreeMap` for centralized storage
- Complex `Storable` implementations
- 200+ lines of access control code

**Why it was changed**: The centralized approach was determined to be over-engineered for the problem it was solving. The decentralized approach provides the same functionality with significantly less complexity.

**Future use**: If specific use cases arise that require centralized access control (bulk operations, cross-resource queries), the archived approach can be referenced and adapted.

## Appendix: Centralized Approach Reference

For future reference, the centralized access control approach using the AccessIndex system has been documented separately. This approach was considered but not implemented in favor of the decentralized approach.

**Reference Document**: [Capsule Access Control - Centralized Approach Reference](./capsule-access-centralized-reference.md)

This reference document contains:

- Complete implementation of the AccessIndex system
- Fixed memory overlap issues
- Proper Storable implementations
- Full permission evaluation logic
- Technical improvements and fixes

**When to Consider Centralized Approach:**

- Need for global access queries across many resources
- Bulk revocation/auditing operations
- Complex cross-resource access patterns
- Centralized access management requirements

## Related Documents

- [Capsule Module Architecture](../../architecture/capsule-module-architecture.md)
- [Gallery Type Refactor Implementation](./gallery-type-refactor-implementation.md) - Main implementation plan (Phases 2-5)
- [Access Control Architecture Decision](./access-control-architecture-decision.md) - Discussion of centralized vs decentralized approaches
- [Capsule Access Control - Centralized Approach Reference](./capsule-access-centralized-reference.md) - Future implementation reference
