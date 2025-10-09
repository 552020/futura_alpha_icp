# ICP Backend Resource Sharing Mechanism - Capsule-Aware Universal System

**Status**: `OPEN` - Design Analysis Required  
**Priority**: `HIGH` - ICP Backend Architecture  
**Created**: 2025-01-09  
**Related Issues**: [Universal Resource Sharing System](./done-resource-sharing-table-unification.md), [Gallery Sharing Table Enhancement](./done-gallery-sharing-table-enhancement.md)

## 🎯 **Objective**

Design a **mirror resource sharing system** for the ICP backend that:

1. **Mirrors the Web2 schema.ts structure** for eventual integration
2. **Uses capsules instead of users** as the primary identity system
3. **Supports the same permission model** (bitmask permissions, roles, magic links)
4. **Provides infrastructure for future cross-platform integration**
5. **Keeps systems separate** for now but designed for eventual unification

## 📋 **Current State Analysis**

### **Web2 Sharing System (Neon Database)**

```typescript
// ✅ COMPLETED: Universal resource sharing tables
export const resourceMembership = pgTable("resource_membership", {
  resourceType: text("resource_type", { enum: ["gallery", "memory", "folder"] }),
  resourceId: text("resource_id").notNull(),
  allUserId: text("all_user_id").notNull(), // FK to allUsers.id

  // Provenance tracking
  grantSource: text("grant_source", { enum: ["user", "group", "magic_link", "public_mode", "system"] }),
  sourceId: text("source_id"),

  // Bitmask permissions
  permMask: integer("perm_mask").notNull().default(0),
  role: text("role", { enum: ["owner", "superadmin", "admin", "member", "guest"] }),

  // Audit trail
  invitedByAllUserId: text("invited_by_all_user_id"),
  createdAt: timestamp("created_at").notNull().defaultNow(),
});
```

### **ICP Backend Current State**

```rust
// ❌ CURRENT: Simple access control
pub struct Memory {
    pub id: String,
    pub capsule_id: String,
    pub access: MemoryAccess, // Simple enum
    // ... other fields
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum MemoryAccess {
    Public { owner_secure_code: String },
    Private { owner_secure_code: String },
    Custom {
        individuals: Vec<PersonRef>,
        groups: Vec<String>,
        owner_secure_code: String,
    },
    Scheduled {
        accessible_after: u64,
        access: Box<MemoryAccess>,
        owner_secure_code: String,
    },
    EventTriggered {
        trigger_event: AccessEvent,
        access: Box<MemoryAccess>,
        owner_secure_code: String,
    },
}
```

### **Capsule System Architecture**

```rust
// ✅ CURRENT: Capsule-based storage
pub struct Capsule {
    pub id: String,
    pub owner_principal: Principal,
    pub memories: HashMap<String, Memory>, // memory_id -> Memory
    pub galleries: HashMap<String, Gallery>, // gallery_id -> Gallery
    pub folders: HashMap<String, Folder>, // folder_id -> Folder
    // ... other fields
}
```

## 🚨 **Key Challenges**

### **1. Cross-Platform Identity Mapping**

**Problem**: Web2 uses `allUserId` (UUID), ICP uses `Principal` (cryptographic identity)

**Web2 Identity**:

```typescript
// Neon database
allUsers: {
  id: "uuid-v7", // Web2 user ID
  email: "user@example.com",
  // ... other fields
}
```

**ICP Identity**:

```rust
// ICP canister
pub struct User {
    pub principal: Principal, // Web3 cryptographic identity
    pub capsule_id: String,
    // ... other fields
}
```

### **2. Decentralized vs Centralized Sharing**

**Web2 Approach** (Centralized):

- All sharing data in Neon database
- Single source of truth
- Easy to query and manage

**ICP Approach** (Decentralized):

- Each capsule manages its own sharing
- No global sharing registry
- Requires cross-capsule communication

### **3. Permission Synchronization**

**Problem**: How to keep Web2 and ICP permissions in sync?

**Scenarios**:

- User shares memory on Web2 → Should be accessible on ICP
- User shares memory on ICP → Should be accessible on Web2
- User revokes access on Web2 → Should be revoked on ICP
- User revokes access on ICP → Should be revoked on Web2

## 🚀 **Proposed Solution: Mirror Sharing Architecture**

### **Core Design Principles**

1. **Schema Mirroring**: ICP backend mirrors the exact structure of `schema.ts`
2. **Capsule-Centric Identity**: Uses `capsule_id` instead of `all_user_id` as primary identity
3. **Separate but Compatible**: Systems work independently but share the same data structures
4. **Future Integration Ready**: Designed for eventual cross-platform synchronization
5. **Backward Compatibility**: Existing `MemoryAccess` enum still works

### **1. ICP Backend Sharing Data Structures (Mirroring schema.ts)**

```rust
// ✅ MIRROR: Exact same structure as schema.ts but with capsule-centric identity

// ============================================================================
// UNIVERSAL RESOURCE SHARING SYSTEM (ICP Backend Mirror)
// ============================================================================

// ✅ MIRROR: Permission bits (same as Web2)
pub const PERM: u32 = {
    const VIEW: u32 = 1 << 0;      // 1
    const DOWNLOAD: u32 = 1 << 1;  // 2
    const SHARE: u32 = 1 << 2;     // 4
    const MANAGE: u32 = 1 << 3;    // 8
    const OWN: u32 = 1 << 4;       // 16
};

// ✅ MIRROR: Role templates (same as Web2)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct RoleTemplate {
    pub role: ResourceRole,
    pub resource_type: ResourceType,
    pub perm_mask: u32, // sum of PERM bits
    pub created_at: u64,
    pub updated_at: u64,
}

// ✅ MIRROR: Resource registry (same as Web2)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct ResourceRegistry {
    pub id: String, // mirrors galleries.id / memories.id / folders.id
    pub resource_type: ResourceType,
    pub owner_capsule_id: String, // ✅ CAPSULE-CENTRIC: FK to capsule.id instead of allUsers.id
    pub created_at: u64,
}

// ✅ MIRROR: Resource membership (same as Web2)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct ResourceMembership {
    pub id: String,
    pub resource_type: ResourceType,
    pub resource_id: String,
    pub capsule_id: String, // ✅ CAPSULE-CENTRIC: FK to capsule.id instead of allUsers.id

    // Provenance of the grant
    pub grant_source: GrantSource,
    pub source_id: Option<String>, // e.g., group id or magic_link id
    pub role: ResourceRole,
    pub perm_mask: u32,
    pub invited_by_capsule_id: Option<String>, // ✅ CAPSULE-CENTRIC: FK to capsule.id
    pub created_at: u64,
    pub updated_at: u64,
}

// ✅ MIRROR: Public access policy (same as Web2)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct ResourcePublicPolicy {
    pub id: String,
    pub resource_type: ResourceType,
    pub resource_id: String,
    pub mode: PublicMode,
    pub link_token_hash: Option<String>, // sha-256 of token (public_link only)
    pub perm_mask: u32,
    pub expires_at: Option<u64>,
    pub revoked_at: Option<u64>,
    pub created_at: u64,
    pub updated_at: u64,
}

// ✅ MIRROR: Magic links (same as Web2)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct MagicLink {
    pub id: String,
    pub token_hash: String, // sha-256 of opaque token
    pub link_type: MagicLinkType,
    pub resource_type: ResourceType,
    pub resource_id: String,
    pub inviter_capsule_id: String, // ✅ CAPSULE-CENTRIC: FK to capsule.id
    pub intended_email: Option<String>, // for admin_invite
    pub admin_subtype: Option<AdminSubtype>, // for admin_invite
    pub preset_perm_mask: u32,
    pub max_uses: u32,
    pub used_count: u32,
    pub expires_at: u64,
    pub revoked_at: Option<u64>,
    pub created_at: u64,
    pub updated_at: u64,
    pub last_used_at: Option<u64>,
}

// ✅ MIRROR: Magic link consumption (same as Web2)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct MagicLinkConsumption {
    pub id: String,
    pub magic_link_id: String, // FK to magicLink.id
    pub capsule_id: Option<String>, // ✅ CAPSULE-CENTRIC: set after login/registration
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub used_at: u64,
    pub result: MagicLinkResult,
}

// ✅ MIRROR: Enums (same as Web2)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum ResourceType {
    Gallery,
    Memory,
    Folder,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum ResourceRole {
    Owner,
    Superadmin,
    Admin,
    Member,
    Guest,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum GrantSource {
    User,
    Group,
    MagicLink,
    PublicMode,
    System,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum PublicMode {
    Private,
    PublicAuth,
    PublicLink,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum MagicLinkType {
    AdminInvite,
    GuestShare,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum AdminSubtype {
    Superadmin,
    Admin,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum MagicLinkResult {
    Success,
    Expired,
    Revoked,
    LimitExceeded,
}
```

### **2. Enhanced Capsule Structure (Mirroring schema.ts)**

```rust
// ✅ UPDATED: Capsule with mirror sharing data (same structure as schema.ts)
pub struct Capsule {
    pub id: String,
    pub owner_principal: Principal,

    // Resources
    pub memories: HashMap<String, Memory>,
    pub galleries: HashMap<String, Gallery>,
    pub folders: HashMap<String, Folder>,

    // ✅ MIRROR: Sharing data (exact same structure as schema.ts)
    pub role_templates: HashMap<String, RoleTemplate>, // role -> RoleTemplate
    pub resource_registry: HashMap<String, ResourceRegistry>, // resource_id -> ResourceRegistry
    pub resource_membership: HashMap<String, ResourceMembership>, // membership_id -> ResourceMembership
    pub resource_public_policy: HashMap<String, ResourcePublicPolicy>, // policy_id -> ResourcePublicPolicy
    pub magic_links: HashMap<String, MagicLink>, // link_id -> MagicLink
    pub magic_link_consumption: HashMap<String, MagicLinkConsumption>, // consumption_id -> MagicLinkConsumption

    // ✅ FUTURE: Cross-platform sync infrastructure (not implemented yet)
    pub web2_sync_enabled: bool, // Flag for future sync capability
    pub last_sync_at: Option<u64>, // Timestamp of last sync (if any)

    // ... other fields
}
```

### **3. Permission Checking System (Mirroring schema.ts)**

```rust
// ✅ MIRROR: Universal permission checking (same logic as Web2)
impl Capsule {
    /// Check if a capsule has permission on a resource
    pub fn check_permission(
        &self,
        requesting_capsule_id: &str,
        resource_type: &ResourceType,
        resource_id: &str,
        required_permission: Permission,
    ) -> bool {
        // 1. Check if requesting capsule is owner
        if requesting_capsule_id == self.id {
            return true; // Owner has all permissions
        }

        // 2. Get effective permission mask
        let effective_mask = self.get_effective_permissions(requesting_capsule_id, resource_type, resource_id);

        // 3. Check if permission is granted
        has_permission(effective_mask, required_permission)
    }

    /// Get effective permission mask for a capsule on a resource (mirrors Web2 logic)
    pub fn get_effective_permissions(
        &self,
        requesting_capsule_id: &str,
        resource_type: &ResourceType,
        resource_id: &str,
    ) -> u32 {
        let mut effective_mask = 0u32;

        // Combine all permission sources (same as Web2)
        effective_mask |= self.get_membership_permissions(requesting_capsule_id, resource_type, resource_id);
        effective_mask |= self.get_magic_link_permissions(requesting_capsule_id, resource_type, resource_id);
        effective_mask |= self.get_public_permissions(resource_type, resource_id);

        effective_mask
    }

    /// Get membership permissions (mirrors resourceMembership table)
    fn get_membership_permissions(
        &self,
        requesting_capsule_id: &str,
        resource_type: &ResourceType,
        resource_id: &str,
    ) -> u32 {
        self.resource_membership
            .values()
            .filter(|membership| {
                membership.resource_type == *resource_type
                    && membership.resource_id == resource_id
                    && membership.capsule_id == requesting_capsule_id
            })
            .map(|membership| membership.perm_mask)
            .fold(0u32, |acc, mask| acc | mask)
    }

    /// Get magic link permissions (mirrors magicLink consumption)
    fn get_magic_link_permissions(
        &self,
        requesting_capsule_id: &str,
        resource_type: &ResourceType,
        resource_id: &str,
    ) -> u32 {
        // Find magic links for this resource
        let magic_links: Vec<&MagicLink> = self.magic_links
            .values()
            .filter(|link| {
                link.resource_type == *resource_type
                    && link.resource_id == resource_id
                    && link.revoked_at.is_none()
                    && link.expires_at > ic_cdk::api::time()
            })
            .collect();

        // Check if requesting capsule has consumed any of these magic links
        let mut effective_mask = 0u32;
        for link in magic_links {
            let has_consumed = self.magic_link_consumption
                .values()
                .any(|consumption| {
                    consumption.magic_link_id == link.id
                        && consumption.capsule_id.as_ref() == Some(&requesting_capsule_id.to_string())
                        && consumption.result == MagicLinkResult::Success
                });

            if has_consumed {
                effective_mask |= link.preset_perm_mask;
            }
        }

        effective_mask
    }

    /// Get public access permissions (mirrors resourcePublicPolicy table)
    fn get_public_permissions(
        &self,
        resource_type: &ResourceType,
        resource_id: &str,
    ) -> u32 {
        self.resource_public_policy
            .values()
            .find(|policy| {
                policy.resource_type == *resource_type
                    && policy.resource_id == resource_id
                    && policy.revoked_at.is_none()
                    && (policy.expires_at.is_none() || policy.expires_at.unwrap() > ic_cdk::api::time())
            })
            .map(|policy| policy.perm_mask)
            .unwrap_or(0u32)
    }
}

// ✅ MIRROR: Permission bitmask helpers (exact same as Web2)
pub const PERM: u32 = {
    const VIEW: u32 = 1 << 0;      // 1
    const DOWNLOAD: u32 = 1 << 1;  // 2
    const SHARE: u32 = 1 << 2;     // 4
    const MANAGE: u32 = 1 << 3;    // 8
    const OWN: u32 = 1 << 4;       // 16
};

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum Permission {
    View,
    Download,
    Share,
    Manage,
    Own,
}

// ✅ MIRROR: Bitmask helpers (exact same as Web2)
pub fn has_permission(mask: u32, permission: Permission) -> bool {
    let bit = match permission {
        Permission::View => PERM::VIEW,
        Permission::Download => PERM::DOWNLOAD,
        Permission::Share => PERM::SHARE,
        Permission::Manage => PERM::MANAGE,
        Permission::Own => PERM::OWN,
    };
    (mask & bit) != 0
}

pub fn add_permission(mask: u32, permission: Permission) -> u32 {
    mask | match permission {
        Permission::View => PERM::VIEW,
        Permission::Download => PERM::DOWNLOAD,
        Permission::Share => PERM::SHARE,
        Permission::Manage => PERM::MANAGE,
        Permission::Own => PERM::OWN,
    }
}

pub fn remove_permission(mask: u32, permission: Permission) -> u32 {
    mask & !match permission {
        Permission::View => PERM::VIEW,
        Permission::Download => PERM::DOWNLOAD,
        Permission::Share => PERM::SHARE,
        Permission::Manage => PERM::MANAGE,
        Permission::Own => PERM::OWN,
    }
}
```

### **4. Future Cross-Platform Integration Infrastructure**

```rust
// ✅ FUTURE: Cross-platform sync system (not implemented yet, but infrastructure ready)
impl Capsule {
    /// Future: Sync sharing data with Web2 system
    /// This is designed but not implemented - systems remain separate for now
    pub async fn sync_with_web2(&mut self) -> Result<(), String> {
        if !self.web2_sync_enabled {
            return Err("Web2 sync not enabled for this capsule".to_string());
        }

        // 1. Get sync token for this capsule
        let sync_token = self.get_sync_token().await?;

        // 2. Fetch Web2 sharing data (mirrors schema.ts structure)
        let web2_memberships = self.fetch_web2_memberships(sync_token).await?;
        let web2_public_policies = self.fetch_web2_public_policies(sync_token).await?;
        let web2_magic_links = self.fetch_web2_magic_links(sync_token).await?;

        // 3. Update ICP sharing data (same structure, different identity system)
        for membership in web2_memberships {
            self.update_membership_from_web2(membership).await?;
        }

        for policy in web2_public_policies {
            self.update_public_policy_from_web2(policy).await?;
        }

        for magic_link in web2_magic_links {
            self.update_magic_link_from_web2(magic_link).await?;
        }

        // 4. Update sync timestamp
        self.last_sync_at = Some(ic_cdk::api::time());

        Ok(())
    }

    /// Future: Update membership from Web2 data (mirrors resourceMembership table)
    async fn update_membership_from_web2(&mut self, web2_membership: Web2ResourceMembership) -> Result<(), String> {
        // Convert Web2 membership to ICP membership (same structure, different identity)
        let icp_membership = ResourceMembership {
            id: web2_membership.id,
            resource_type: self.map_resource_type(web2_membership.resource_type),
            resource_id: web2_membership.resource_id,
            capsule_id: self.map_all_user_id_to_capsule_id(&web2_membership.all_user_id).await?, // ✅ CAPSULE-CENTRIC

            // Same structure, different identity system
            grant_source: self.map_grant_source(web2_membership.grant_source),
            source_id: web2_membership.source_id,
            role: self.map_role(web2_membership.role),
            perm_mask: web2_membership.perm_mask,
            invited_by_capsule_id: self.map_all_user_id_to_capsule_id(&web2_membership.invited_by_all_user_id).await?, // ✅ CAPSULE-CENTRIC
            created_at: web2_membership.created_at,
            updated_at: web2_membership.updated_at,
        };

        // Store in capsule (same structure as schema.ts)
        self.resource_membership.insert(icp_membership.id.clone(), icp_membership);

        Ok(())
    }

    /// Future: Map all_user_id to capsule_id (cross-platform identity mapping)
    async fn map_all_user_id_to_capsule_id(&self, all_user_id: &str) -> Result<String, String> {
        // This would query a cross-platform identity mapping service
        // For now, return error - implementation depends on identity mapping service
        Err("Cross-platform identity mapping not implemented yet".to_string())
    }
}

// ✅ FUTURE: Web2 data structures for sync (mirrors schema.ts exactly)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct Web2ResourceMembership {
    pub id: String,
    pub resource_type: String, // "gallery", "memory", "folder"
    pub resource_id: String,
    pub all_user_id: String, // ✅ WEB2: FK to allUsers.id
    pub grant_source: String,
    pub source_id: Option<String>,
    pub role: String,
    pub perm_mask: u32,
    pub invited_by_all_user_id: Option<String>, // ✅ WEB2: FK to allUsers.id
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct Web2ResourcePublicPolicy {
    pub id: String,
    pub resource_type: String,
    pub resource_id: String,
    pub mode: String,
    pub link_token_hash: Option<String>,
    pub perm_mask: u32,
    pub expires_at: Option<u64>,
    pub revoked_at: Option<u64>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct Web2MagicLink {
    pub id: String,
    pub token_hash: String,
    pub link_type: String,
    pub resource_type: String,
    pub resource_id: String,
    pub inviter_all_user_id: String, // ✅ WEB2: FK to allUsers.id
    pub intended_email: Option<String>,
    pub admin_subtype: Option<String>,
    pub preset_perm_mask: u32,
    pub max_uses: u32,
    pub used_count: u32,
    pub expires_at: u64,
    pub revoked_at: Option<u64>,
    pub created_at: u64,
    pub updated_at: u64,
    pub last_used_at: Option<u64>,
}
```

### **5. Backward Compatibility with MemoryAccess**

```rust
// ✅ UPDATED: MemoryAccess enum with sharing integration
impl Memory {
    /// Check access using new sharing system
    pub fn check_access(&self, capsule: &Capsule, principal: &Principal) -> bool {
        // 1. Try new sharing system first
        if capsule.check_permission(principal, &ResourceType::Memory, &self.id, Permission::View) {
            return true;
        }

        // 2. Fall back to legacy MemoryAccess enum
        match &self.access {
            MemoryAccess::Public { .. } => true,
            MemoryAccess::Private { .. } => false,
            MemoryAccess::Custom { individuals, .. } => {
                individuals.iter().any(|person| person.principal == *principal)
            },
            MemoryAccess::Scheduled { accessible_after, access, .. } => {
                if ic_cdk::api::time() >= *accessible_after {
                    // Check nested access
                    self.check_legacy_access(access, principal)
                } else {
                    false
                }
            },
            MemoryAccess::EventTriggered { trigger_event, access, .. } => {
                // Check if event has been triggered
                if self.is_event_triggered(trigger_event) {
                    self.check_legacy_access(access, principal)
                } else {
                    false
                }
            },
        }
    }

    /// Check legacy access (helper method)
    fn check_legacy_access(&self, access: &MemoryAccess, principal: &Principal) -> bool {
        match access {
            MemoryAccess::Public { .. } => true,
            MemoryAccess::Private { .. } => false,
            MemoryAccess::Custom { individuals, .. } => {
                individuals.iter().any(|person| person.principal == *principal)
            },
            _ => false, // Handle other cases as needed
        }
    }
}
```

## 🔄 **Cross-Platform Synchronization Strategy**

### **1. Identity Mapping Service**

```rust
// ✅ NEW: Cross-platform identity mapping
pub struct IdentityMapping {
    pub all_user_id: String,      // Web2 UUID
    pub principal: Principal,     // ICP cryptographic identity
    pub email: String,            // Common identifier
    pub created_at: u64,
    pub updated_at: u64,
}

impl Capsule {
    /// Get or create identity mapping
    pub async fn get_identity_mapping(&self, all_user_id: &str) -> Result<Option<IdentityMapping>, String> {
        // Query identity mapping service
        // This could be a separate canister or external service
        Ok(None) // Placeholder
    }

    /// Create identity mapping
    pub async fn create_identity_mapping(&self, all_user_id: String, principal: Principal, email: String) -> Result<IdentityMapping, String> {
        let mapping = IdentityMapping {
            all_user_id,
            principal,
            email,
            created_at: ic_cdk::api::time(),
            updated_at: ic_cdk::api::time(),
        };

        // Store in identity mapping service
        // This would be implemented based on the chosen identity service

        Ok(mapping)
    }
}
```

### **2. Sync Triggers**

```rust
// ✅ NEW: Sync trigger system
impl Capsule {
    /// Trigger sync with Web2 system
    pub async fn trigger_web2_sync(&mut self) -> Result<(), String> {
        // 1. Check if sync is needed
        let time_since_last_sync = ic_cdk::api::time() - self.last_sync_at;
        if time_since_last_sync < 60_000_000_000 { // 1 minute in nanoseconds
            return Ok(()); // Skip sync if too recent
        }

        // 2. Perform sync
        self.sync_with_web2().await?;

        Ok(())
    }

    /// Auto-sync on sharing changes
    pub async fn share_resource(&mut self, share: ResourceShare) -> Result<(), String> {
        // 1. Store share in capsule
        self.resource_shares.insert(share.id.clone(), share);

        // 2. Trigger sync with Web2
        self.trigger_web2_sync().await?;

        Ok(())
    }
}
```

### **3. Conflict Resolution**

```rust
// ✅ NEW: Conflict resolution system
impl Capsule {
    /// Resolve conflicts between Web2 and ICP sharing data
    pub async fn resolve_sharing_conflicts(&mut self) -> Result<(), String> {
        // 1. Get Web2 sharing data
        let web2_shares = self.fetch_web2_shares(self.get_sync_token().await?).await?;

        // 2. Compare with ICP sharing data
        for web2_share in web2_shares {
            if let Some(icp_share) = self.resource_shares.get(&web2_share.id) {
                // 3. Check for conflicts
                if self.has_sharing_conflict(icp_share, &web2_share) {
                    // 4. Resolve conflict (Web2 wins for now)
                    self.update_share_from_web2(web2_share).await?;
                }
            }
        }

        Ok(())
    }

    /// Check if there's a conflict between Web2 and ICP shares
    fn has_sharing_conflict(&self, icp_share: &ResourceShare, web2_share: &Web2ResourceShare) -> bool {
        // Compare key fields
        icp_share.perm_mask != web2_share.perm_mask
            || icp_share.role != self.map_role(web2_share.role.clone())
            || icp_share.updated_at < web2_share.updated_at
    }
}
```

## 🚀 **Implementation Plan**

### **Phase 1: Mirror Data Structures (Current Focus)**

1. **Add mirror sharing types** to ICP backend (exact same as schema.ts)
2. **Update Capsule structure** with mirror sharing data
3. **Implement capsule-centric permission checking** system
4. **Add backward compatibility** with existing MemoryAccess enum
5. **Test capsule-to-capsule sharing** within ICP

### **Phase 2: Core Sharing Functionality**

1. **Implement resource membership** management
2. **Add magic link** system with consumption tracking
3. **Create public access** policies
4. **Test advanced sharing** features within ICP
5. **Performance optimization** for capsule operations

### **Phase 3: Integration with Existing Systems**

1. **Integrate with existing** memory/gallery systems
2. **Update memory/gallery APIs** to use new sharing system
3. **Test backward compatibility** with MemoryAccess
4. **Security audit** of sharing system
5. **Documentation and examples**

### **Phase 4: Future Cross-Platform Integration (Optional)**

1. **Implement identity mapping** service (when needed)
2. **Add sync triggers** and conflict resolution (when needed)
3. **Create Web2 sync API** endpoints (when needed)
4. **Test cross-platform** sharing (when needed)

**Note**: Phases 1-3 focus on creating a complete, standalone ICP sharing system that mirrors the Web2 structure. Phase 4 is optional and only needed when cross-platform integration is required.

## 🎯 **Benefits**

### **1. Schema Mirroring**

- ✅ **Exact same structure** as Web2 schema.ts
- ✅ **Same bitmask permissions** (VIEW, DOWNLOAD, SHARE, MANAGE, OWN)
- ✅ **Same role system** (owner, superadmin, admin, member, guest)
- ✅ **Same magic link system** with TTL and consumption tracking
- ✅ **Same public access modes** (private, public_auth, public_link)

### **2. Capsule-Centric Architecture**

- ✅ **Each capsule manages its own sharing** data
- ✅ **No global sharing registry** needed
- ✅ **Decentralized access control** within ICP
- ✅ **Scalable to millions of capsules**
- ✅ **Capsule-to-capsule sharing** within ICP ecosystem

### **3. Future Integration Ready**

- ✅ **Infrastructure designed** for eventual cross-platform sync
- ✅ **Same data structures** make integration straightforward
- ✅ **Identity mapping ready** (when needed)
- ✅ **Sync mechanisms designed** (when needed)
- ✅ **Systems remain separate** until integration is required

### **4. Backward Compatibility**

- ✅ **Existing MemoryAccess enum** still works
- ✅ **Gradual migration path** from old to new system
- ✅ **No breaking changes** to existing APIs
- ✅ **Legacy code continues** to function

## 🚨 **Challenges and Solutions**

### **1. Identity Mapping Complexity**

**Challenge**: Mapping between Web2 UUIDs and ICP Principals

**Solution**:

- Identity mapping service
- Email as common identifier
- Lazy mapping creation
- Caching for performance

### **2. Sync Performance**

**Challenge**: Keeping Web2 and ICP sharing data in sync

**Solution**:

- Lazy sync triggers
- Conflict resolution strategies
- Batch sync operations
- Caching and optimization

### **3. Cross-Capsule Communication**

**Challenge**: Sharing resources across different capsules

**Solution**:

- Cross-capsule permission queries
- Shared resource registry
- Efficient permission checking
- Caching strategies

### **4. Security and Privacy**

**Challenge**: Ensuring secure cross-platform sharing

**Solution**:

- Cryptographic identity verification
- Secure token exchange
- Audit trails
- Privacy-preserving sharing

## 📊 **Performance Considerations**

### **1. Permission Checking**

```rust
// ✅ Optimized permission checking
impl Capsule {
    /// Fast permission check with caching
    pub fn check_permission_cached(
        &self,
        principal: &Principal,
        resource_type: &ResourceType,
        resource_id: &str,
        required_permission: Permission,
    ) -> bool {
        // 1. Check cache first
        if let Some(cached_result) = self.get_permission_cache(principal, resource_type, resource_id) {
            return has_permission(cached_result, required_permission);
        }

        // 2. Compute permissions
        let perm_mask = self.get_effective_permissions(principal, resource_type, resource_id);

        // 3. Cache result
        self.set_permission_cache(principal, resource_type, resource_id, perm_mask);

        // 4. Return result
        has_permission(perm_mask, required_permission)
    }
}
```

### **2. Batch Operations**

```rust
// ✅ Batch sharing operations
impl Capsule {
    /// Batch share multiple resources
    pub async fn batch_share_resources(&mut self, shares: Vec<ResourceShare>) -> Result<(), String> {
        // 1. Validate all shares
        for share in &shares {
            self.validate_share(share)?;
        }

        // 2. Store all shares
        for share in shares {
            self.resource_shares.insert(share.id.clone(), share);
        }

        // 3. Single sync operation
        self.trigger_web2_sync().await?;

        Ok(())
    }
}
```

## 🎯 **Success Criteria**

- ✅ Universal permission model across Web2 and ICP
- ✅ Capsule-centric sharing architecture
- ✅ Cross-platform identity mapping
- ✅ Backward compatibility with existing systems
- ✅ Performance meets requirements
- ✅ Security and privacy requirements satisfied
- ✅ Seamless user experience across platforms

## 📚 **References**

- [Universal Resource Sharing System](./done-resource-sharing-table-unification.md) - **COMPLETED**
- [Gallery Sharing Table Enhancement](./done-gallery-sharing-table-enhancement.md) - **COMPLETED**
- [ICP Backend Architecture](../../../src/backend/README.md)
- [Capsule System Documentation](../../../src/backend/src/capsule/README.md)

---

**This document provides a comprehensive design for implementing universal resource sharing in the ICP backend while maintaining compatibility with the Web2 sharing system and leveraging the capsule architecture.**
