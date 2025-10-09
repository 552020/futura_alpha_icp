# ICP Backend Resource Sharing Mechanism - Decentralized Invite-Based System

**Status**: `OPEN` - Design Analysis Required  
**Priority**: `HIGH` - ICP Backend Architecture  
**Created**: 2025-01-09  
**Related Issues**: [Universal Resource Sharing System](./done-resource-sharing-table-unification.md), [Gallery Sharing Table Enhancement](./done-gallery-sharing-table-enhancement.md), [Centralized Approach](./icp-backend-resource-sharing-mechanism-centralized.md)

## 🎯 **Objective**

Design a **decentralized resource sharing system** for the ICP backend that:

1. **Uses autonomous capsules** - each capsule is its own canister
2. **Implements invite-based sharing** - Capsule A sends invites to Capsule B
3. **Maintains local sharing state** - each capsule manages its own sharing data
4. **Supports the same permission model** (bitmask permissions, roles, magic links)
5. **Enables efficient access checks** with minimal cross-capsule communication

## 📋 **Current State Analysis**

### **Web2 Sharing System (Centralized)**

```typescript
// ✅ COMPLETED: Centralized sharing in Neon database
export const resourceMembership = pgTable("resource_membership", {
  resourceType: text("resource_type", { enum: ["gallery", "memory", "folder"] }),
  resourceId: text("resource_id").notNull(),
  allUserId: text("all_user_id").notNull(), // FK to allUsers.id

  // All sharing data in one central database
  permMask: integer("perm_mask").notNull().default(0),
  role: text("role", { enum: ["owner", "superadmin", "admin", "member", "guest"] }),
  // ... other fields
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
    // ... other variants
}
```

### **Capsule System Architecture**

```rust
// ✅ CURRENT: Autonomous capsule-based storage
pub struct Capsule {
    pub id: String,
    pub owner_principal: Principal,
    pub memories: HashMap<String, Memory>, // memory_id -> Memory
    pub galleries: HashMap<String, Gallery>, // gallery_id -> Gallery
    pub folders: HashMap<String, Folder>, // folder_id -> Folder
    // ... other fields
}
```

## 🚨 **Key Challenges with Centralized Approach**

### **1. The Centralized Problem**

**Problem**: Copying the centralized schema.ts 1:1 won't work because:

- **Web2**: Single database with all sharing data
- **ICP**: Each capsule is autonomous - its own canister
- **No central registry**: Each capsule manages its own sharing data
- **No cross-capsule queries**: Capsule A can't directly query Capsule B's data

### **2. The Access Check Problem**

If we copy the centralized structure:

```rust
// ❌ THIS WON'T WORK: Centralized thinking
pub struct ResourceMembership {
    pub resource_id: String,
    pub capsule_id: String, // Which capsule has access
    // ... other fields
}
```

**Problem**: How does **Capsule A** know that **Capsule B** has been granted access to its resources?

- Capsule A stores: "Capsule B has access to Memory X"
- But Capsule B has **no way to know** it has access unless it queries Capsule A
- This creates a **query dependency** - every access check requires cross-capsule communication

## 🚀 **Proposed Solution: Decentralized Invite-Based System**

### **Core Design Principles**

1. **Autonomous Capsules**: Each capsule manages its own sharing state
2. **Invite-Based Sharing**: Capsule A sends invites to Capsule B
3. **Bidirectional Storage**: Both capsules store relevant sharing information
4. **Local Access Checks**: Capsules can check permissions locally
5. **Minimal Cross-Capsule Communication**: Only for invite sending/acceptance

### **1. Invite-Based Sharing Flow**

```
1. Capsule A (Owner) → Creates invite → Sends to Capsule B
2. Capsule B (Recipient) → Receives invite → Accepts/Rejects
3. Both capsules → Store sharing state locally
4. Access checks → Both capsules check locally
```

### **2. Capsule A (Owner) Local State**

```rust
// ✅ NEW: Sent invites (what I shared with others)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct SentInvite {
    pub invite_id: String,
    pub resource_type: ResourceType,
    pub resource_id: String,
    pub recipient_capsule_id: String,
    pub permissions: u32, // bitmask permissions
    pub role: ResourceRole,
    pub status: InviteStatus,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub accepted_at: Option<u64>,
    pub revoked_at: Option<u64>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum InviteStatus {
    Pending,    // Sent but not yet accepted/rejected
    Accepted,   // Recipient accepted the invite
    Rejected,   // Recipient rejected the invite
    Expired,    // Invite expired
    Revoked,    // Owner revoked the invite
}
```

### **3. Capsule B (Recipient) Local State**

```rust
// ✅ NEW: Received invites (what others shared with me)
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct ReceivedInvite {
    pub invite_id: String,
    pub sender_capsule_id: String,
    pub resource_type: ResourceType,
    pub resource_id: String,
    pub permissions: u32, // bitmask permissions
    pub role: ResourceRole,
    pub status: InviteStatus,
    pub received_at: u64,
    pub accepted_at: Option<u64>,
    pub expires_at: Option<u64>,
}
```

### **4. Enhanced Capsule Structure**

```rust
// ✅ UPDATED: Capsule with decentralized sharing data
pub struct Capsule {
    pub id: String,
    pub owner_principal: Principal,

    // Resources
    pub memories: HashMap<String, Memory>,
    pub galleries: HashMap<String, Gallery>,
    pub folders: HashMap<String, Folder>,

    // ✅ NEW: Decentralized sharing data
    pub sent_invites: HashMap<String, SentInvite>, // invite_id -> SentInvite
    pub received_invites: HashMap<String, ReceivedInvite>, // invite_id -> ReceivedInvite

    // ✅ NEW: Magic links (for public sharing)
    pub magic_links: HashMap<String, MagicLink>, // link_id -> MagicLink
    pub magic_link_consumption: HashMap<String, MagicLinkConsumption>, // consumption_id -> MagicLinkConsumption

    // ✅ NEW: Public access policies
    pub public_policies: HashMap<String, PublicAccessPolicy>, // policy_id -> PublicAccessPolicy

    // ... other fields
}
```

### **5. Permission Checking System**

```rust
// ✅ NEW: Decentralized permission checking
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

        // 2. Check if I sent an accepted invite to this capsule
        let sent_permissions = self.get_sent_invite_permissions(requesting_capsule_id, resource_type, resource_id);
        if has_permission(sent_permissions, required_permission) {
            return true;
        }

        // 3. Check public access
        let public_permissions = self.get_public_permissions(resource_type, resource_id);
        if has_permission(public_permissions, required_permission) {
            return true;
        }

        false
    }

    /// Get permissions from sent invites (owner's perspective)
    fn get_sent_invite_permissions(
        &self,
        requesting_capsule_id: &str,
        resource_type: &ResourceType,
        resource_id: &str,
    ) -> u32 {
        self.sent_invites
            .values()
            .filter(|invite| {
                invite.resource_type == *resource_type
                    && invite.resource_id == resource_id
                    && invite.recipient_capsule_id == requesting_capsule_id
                    && invite.status == InviteStatus::Accepted
                    && invite.revoked_at.is_none()
                    && (invite.expires_at.is_none() || invite.expires_at.unwrap() > ic_cdk::api::time())
            })
            .map(|invite| invite.permissions)
            .fold(0u32, |acc, perms| acc | perms)
    }

    /// Get public access permissions
    fn get_public_permissions(
        &self,
        resource_type: &ResourceType,
        resource_id: &str,
    ) -> u32 {
        self.public_policies
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

// ✅ NEW: Recipient's permission checking
impl Capsule {
    /// Check if I have permission on a resource (recipient's perspective)
    pub fn check_my_permission(
        &self,
        sender_capsule_id: &str,
        resource_type: &ResourceType,
        resource_id: &str,
        required_permission: Permission,
    ) -> bool {
        // Check if I have an accepted invite from this capsule
        let received_permissions = self.get_received_invite_permissions(sender_capsule_id, resource_type, resource_id);
        has_permission(received_permissions, required_permission)
    }

    /// Get permissions from received invites (recipient's perspective)
    fn get_received_invite_permissions(
        &self,
        sender_capsule_id: &str,
        resource_type: &ResourceType,
        resource_id: &str,
    ) -> u32 {
        self.received_invites
            .values()
            .filter(|invite| {
                invite.resource_type == *resource_type
                    && invite.resource_id == resource_id
                    && invite.sender_capsule_id == sender_capsule_id
                    && invite.status == InviteStatus::Accepted
                    && (invite.expires_at.is_none() || invite.expires_at.unwrap() > ic_cdk::api::time())
            })
            .map(|invite| invite.permissions)
            .fold(0u32, |acc, perms| acc | perms)
    }
}
```

### **6. Invite Management System**

```rust
// ✅ NEW: Invite management
impl Capsule {
    /// Send invite to another capsule
    pub async fn send_invite(
        &mut self,
        recipient_capsule_id: String,
        resource_type: ResourceType,
        resource_id: String,
        permissions: u32,
        role: ResourceRole,
        expires_at: Option<u64>,
    ) -> Result<String, String> {
        // 1. Generate invite ID
        let invite_id = self.generate_invite_id();

        // 2. Create sent invite
        let sent_invite = SentInvite {
            invite_id: invite_id.clone(),
            resource_type: resource_type.clone(),
            resource_id: resource_id.clone(),
            recipient_capsule_id: recipient_capsule_id.clone(),
            permissions,
            role: role.clone(),
            status: InviteStatus::Pending,
            created_at: ic_cdk::api::time(),
            expires_at,
            accepted_at: None,
            revoked_at: None,
        };

        // 3. Store sent invite locally
        self.sent_invites.insert(invite_id.clone(), sent_invite);

        // 4. Send invite to recipient capsule
        self.deliver_invite_to_capsule(recipient_capsule_id, invite_id.clone()).await?;

        Ok(invite_id)
    }

    /// Receive invite from another capsule
    pub async fn receive_invite(
        &mut self,
        invite: ReceivedInvite,
    ) -> Result<(), String> {
        // Store received invite locally
        self.received_invites.insert(invite.invite_id.clone(), invite);

        Ok(())
    }

    /// Accept an invite
    pub async fn accept_invite(
        &mut self,
        invite_id: String,
    ) -> Result<(), String> {
        // 1. Update received invite status
        if let Some(received_invite) = self.received_invites.get_mut(&invite_id) {
            received_invite.status = InviteStatus::Accepted;
            received_invite.accepted_at = Some(ic_cdk::api::time());
        }

        // 2. Notify sender capsule of acceptance
        if let Some(received_invite) = self.received_invites.get(&invite_id) {
            self.notify_invite_accepted(received_invite.sender_capsule_id.clone(), invite_id).await?;
        }

        Ok(())
    }

    /// Reject an invite
    pub async fn reject_invite(
        &mut self,
        invite_id: String,
    ) -> Result<(), String> {
        // 1. Update received invite status
        if let Some(received_invite) = self.received_invites.get_mut(&invite_id) {
            received_invite.status = InviteStatus::Rejected;
        }

        // 2. Notify sender capsule of rejection
        if let Some(received_invite) = self.received_invites.get(&invite_id) {
            self.notify_invite_rejected(received_invite.sender_capsule_id.clone(), invite_id).await?;
        }

        Ok(())
    }

    /// Revoke an invite (owner's action)
    pub async fn revoke_invite(
        &mut self,
        invite_id: String,
    ) -> Result<(), String> {
        // 1. Update sent invite status
        if let Some(sent_invite) = self.sent_invites.get_mut(&invite_id) {
            sent_invite.status = InviteStatus::Revoked;
            sent_invite.revoked_at = Some(ic_cdk::api::time());
        }

        // 2. Notify recipient capsule of revocation
        if let Some(sent_invite) = self.sent_invites.get(&invite_id) {
            self.notify_invite_revoked(sent_invite.recipient_capsule_id.clone(), invite_id).await?;
        }

        Ok(())
    }
}
```

### **7. Cross-Capsule Communication**

```rust
// ✅ NEW: Cross-capsule communication for invites
impl Capsule {
    /// Deliver invite to recipient capsule
    async fn deliver_invite_to_capsule(
        &self,
        recipient_capsule_id: String,
        invite_id: String,
    ) -> Result<(), String> {
        // Get recipient capsule canister
        let recipient_canister = self.get_capsule_canister(&recipient_capsule_id)?;

        // Create received invite
        let sent_invite = self.sent_invites.get(&invite_id).unwrap();
        let received_invite = ReceivedInvite {
            invite_id: sent_invite.invite_id.clone(),
            sender_capsule_id: self.id.clone(),
            resource_type: sent_invite.resource_type.clone(),
            resource_id: sent_invite.resource_id.clone(),
            permissions: sent_invite.permissions,
            role: sent_invite.role.clone(),
            status: InviteStatus::Pending,
            received_at: ic_cdk::api::time(),
            accepted_at: None,
            expires_at: sent_invite.expires_at,
        };

        // Send to recipient capsule
        recipient_canister.receive_invite(received_invite).await?;

        Ok(())
    }

    /// Notify sender of invite acceptance
    async fn notify_invite_accepted(
        &self,
        sender_capsule_id: String,
        invite_id: String,
    ) -> Result<(), String> {
        let sender_canister = self.get_capsule_canister(&sender_capsule_id)?;
        sender_canister.invite_accepted(invite_id).await?;
        Ok(())
    }

    /// Notify sender of invite rejection
    async fn notify_invite_rejected(
        &self,
        sender_capsule_id: String,
        invite_id: String,
    ) -> Result<(), String> {
        let sender_canister = self.get_capsule_canister(&sender_capsule_id)?;
        sender_canister.invite_rejected(invite_id).await?;
        Ok(())
    }

    /// Notify recipient of invite revocation
    async fn notify_invite_revoked(
        &self,
        recipient_capsule_id: String,
        invite_id: String,
    ) -> Result<(), String> {
        let recipient_canister = self.get_capsule_canister(&recipient_capsule_id)?;
        recipient_canister.invite_revoked(invite_id).await?;
        Ok(())
    }

    /// Update sent invite status (called by recipient)
    pub async fn invite_accepted(&mut self, invite_id: String) -> Result<(), String> {
        if let Some(sent_invite) = self.sent_invites.get_mut(&invite_id) {
            sent_invite.status = InviteStatus::Accepted;
            sent_invite.accepted_at = Some(ic_cdk::api::time());
        }
        Ok(())
    }

    pub async fn invite_rejected(&mut self, invite_id: String) -> Result<(), String> {
        if let Some(sent_invite) = self.sent_invites.get_mut(&invite_id) {
            sent_invite.status = InviteStatus::Rejected;
        }
        Ok(())
    }

    pub async fn invite_revoked(&mut self, invite_id: String) -> Result<(), String> {
        if let Some(received_invite) = self.received_invites.get_mut(&invite_id) {
            received_invite.status = InviteStatus::Revoked;
        }
        Ok(())
    }
}
```

### **8. Magic Links and Public Access**

```rust
// ✅ NEW: Magic links for public sharing
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct MagicLink {
    pub id: String,
    pub token_hash: String, // sha-256 of opaque token
    pub link_type: MagicLinkType,
    pub resource_type: ResourceType,
    pub resource_id: String,
    pub preset_perm_mask: u32,
    pub max_uses: u32,
    pub used_count: u32,
    pub expires_at: u64,
    pub revoked_at: Option<u64>,
    pub created_at: u64,
    pub last_used_at: Option<u64>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum MagicLinkType {
    PublicShare,    // Anyone with link can access
    GuestInvite,    // Invite for specific email
}

// ✅ NEW: Public access policies
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct PublicAccessPolicy {
    pub id: String,
    pub resource_type: ResourceType,
    pub resource_id: String,
    pub mode: PublicMode,
    pub link_token_hash: Option<String>, // For public_link mode
    pub perm_mask: u32,
    pub expires_at: Option<u64>,
    pub revoked_at: Option<u64>,
    pub created_at: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum PublicMode {
    Private,
    PublicAuth,  // Any logged-in user
    PublicLink,  // Anyone with link
}
```

### **9. Permission Bitmask System**

```rust
// ✅ SAME: Permission bitmask helpers (same as Web2)
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
```

## 🚀 **Implementation Plan**

### **Phase 1: Core Invite System**

1. **Add invite data structures** to Capsule
2. **Implement invite sending/receiving** between capsules
3. **Add invite acceptance/rejection** logic
4. **Test basic capsule-to-capsule** sharing

### **Phase 2: Permission System**

1. **Implement permission checking** with invites
2. **Add magic link system** for public sharing
3. **Create public access policies**
4. **Test advanced sharing** features

### **Phase 3: Integration**

1. **Integrate with existing** memory/gallery systems
2. **Add backward compatibility** with MemoryAccess
3. **Performance optimization**
4. **Security audit**

### **Phase 4: Advanced Features**

1. **Add invite expiration** and cleanup
2. **Implement bulk operations**
3. **Add sharing analytics**
4. **Documentation and examples**

## 🎯 **Benefits**

### **1. Truly Decentralized**

- ✅ **Each capsule is autonomous** - no central registry
- ✅ **No single point of failure** - each capsule manages its own data
- ✅ **Scalable to millions of capsules** - no global state
- ✅ **Capsule-to-capsule sharing** without central coordination

### **2. Efficient Access Checks**

- ✅ **Local permission checks** - no cross-capsule queries needed
- ✅ **Bidirectional storage** - both capsules know about the sharing
- ✅ **Minimal communication** - only for invite management
- ✅ **Fast access decisions** - no network calls for permission checks

### **3. Same Permission Model**

- ✅ **Same bitmask permissions** (VIEW, DOWNLOAD, SHARE, MANAGE, OWN)
- ✅ **Same role system** (owner, superadmin, admin, member, guest)
- ✅ **Same magic link system** with TTL and consumption tracking
- ✅ **Same public access modes** (private, public_auth, public_link)

### **4. Backward Compatibility**

- ✅ **Existing MemoryAccess enum** still works
- ✅ **Gradual migration path** from old to new system
- ✅ **No breaking changes** to existing APIs
- ✅ **Legacy code continues** to function

## 🚨 **Challenges and Solutions**

### **1. Cross-Capsule Communication**

**Challenge**: How to send invites between capsules?

**Solution**:

- Direct canister-to-canister calls
- Invite delivery with retry logic
- Offline handling with message queues

### **2. Invite Synchronization**

**Challenge**: Keeping sent/received invites in sync?

**Solution**:

- Bidirectional storage (both capsules store relevant data)
- Status updates via cross-capsule calls
- Conflict resolution strategies

### **3. Offline Capsules**

**Challenge**: What if recipient capsule is offline?

**Solution**:

- Store invites until recipient comes online
- Expiration and cleanup mechanisms
- Retry logic for failed deliveries

### **4. Security and Privacy**

**Challenge**: Ensuring secure invite delivery?

**Solution**:

- Cryptographic verification of capsule identities
- Secure token exchange
- Audit trails for all sharing actions

## 📊 **Performance Considerations**

### **1. Local Access Checks**

```rust
// ✅ Fast local permission checking
impl Capsule {
    pub fn check_permission_fast(
        &self,
        requesting_capsule_id: &str,
        resource_type: &ResourceType,
        resource_id: &str,
        required_permission: Permission,
    ) -> bool {
        // All checks are local - no network calls
        if requesting_capsule_id == self.id {
            return true;
        }

        // Check local invite data
        let permissions = self.get_sent_invite_permissions(requesting_capsule_id, resource_type, resource_id);
        has_permission(permissions, required_permission)
    }
}
```

### **2. Batch Invite Operations**

```rust
// ✅ Batch invite operations
impl Capsule {
    pub async fn batch_send_invites(
        &mut self,
        invites: Vec<InviteRequest>,
    ) -> Result<Vec<String>, String> {
        let mut invite_ids = Vec::new();

        for invite_request in invites {
            let invite_id = self.send_invite(
                invite_request.recipient_capsule_id,
                invite_request.resource_type,
                invite_request.resource_id,
                invite_request.permissions,
                invite_request.role,
                invite_request.expires_at,
            ).await?;

            invite_ids.push(invite_id);
        }

        Ok(invite_ids)
    }
}
```

## 🎯 **Success Criteria**

- ✅ Truly decentralized sharing system
- ✅ Autonomous capsules with local sharing state
- ✅ Efficient access checks without cross-capsule queries
- ✅ Same permission model as Web2 system
- ✅ Backward compatibility with existing systems
- ✅ Performance meets requirements
- ✅ Security and privacy requirements satisfied

## 📚 **References**

- [Universal Resource Sharing System](./done-resource-sharing-table-unification.md) - **COMPLETED**
- [Gallery Sharing Table Enhancement](./done-gallery-sharing-table-enhancement.md) - **COMPLETED**
- [Centralized Approach](./icp-backend-resource-sharing-mechanism-centralized.md) - **Alternative Design**
- [ICP Backend Architecture](../../../src/backend/README.md)
- [Capsule System Documentation](../../../src/backend/src/capsule/README.md)

---

**This document provides a comprehensive design for implementing a truly decentralized, invite-based resource sharing system in the ICP backend that maintains the same permission model as the Web2 system while enabling autonomous capsule operation.**
