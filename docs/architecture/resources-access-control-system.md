# Resources Access Control System Architecture

**Created**: 2025-10-10  
**Last Updated**: 2025-10-10  
**Commit**: 0368c72

## Overview

The Futura platform implements access control systems across both ICP and NextJS backends to manage permissions for resources (Memory, Gallery, Folder, Capsule). This document describes the architecture for both backend implementations.

## Table of Contents

1. [ICP Backend Access Control](#icp-backend-access-control)
2. [NextJS Backend Access Control](#nextjs-backend-access-control)

---

# ICP Backend Access Control

## Overview

The Futura ICP backend implements a **decentralized access control system** where access permissions are stored directly on each resource (Memory, Gallery, Folder, Capsule) rather than in a centralized index. This design provides simplicity, scalability, and aligns with ICP's per-capsule autonomy model.

## Design Philosophy

### Core Principles

1. **Decentralized Authority**: Each resource is the source of truth for its own access control
2. **Capsule Autonomy**: Each capsule canister manages its own resources independently
3. **Shared Evaluation Logic**: Common permission evaluation across all resource types
4. **Optional Centralized Indices**: Minimal indices only for specific use cases (e.g., magic links)
5. **ICP-Native Design**: Built for canister boundaries and upgrade scenarios

### Why Decentralized Over Centralized

| Aspect          | Decentralized (Chosen)            | Centralized (Alternative)           |
| --------------- | --------------------------------- | ----------------------------------- |
| **Scalability** | Natural sharding across canisters | Complex cross-canister coordination |
| **Upgrades**    | Simple struct evolution           | Brittle global index migrations     |
| **Performance** | Direct resource access            | Index lookup overhead               |
| **Complexity**  | ~30-60 lines per resource         | 200+ lines of access plumbing       |
| **Storage**     | Access lives with resource        | Separate stable structures          |

## Architecture Components

### 1. Permission System

#### Bitflags for Permissions

```rust
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
```

#### Role Templates

```rust
pub struct RoleTemplate {
    pub name: String,
    pub perm_mask: u32,  // Uses Perm bits
    pub description: String,
}

// Predefined roles
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

### 2. Access Control Data Structures

#### AccessEntry

```rust
pub struct AccessEntry {
    pub id: String,
    pub person_ref: PersonRef,            // Principal or Opaque ID
    pub grant_source: GrantSource,        // Provenance tracking
    pub source_id: Option<String>,        // Group/magic_link ID
    pub role: ResourceRole,               // Role system
    pub perm_mask: u32,                   // Bitmask permissions
    pub invited_by_person_ref: Option<PersonRef>, // Who granted access
    pub created_at: u64,
    pub updated_at: u64,
}
```

#### PublicPolicy

```rust
pub struct PublicPolicy {
    pub mode: PublicMode,
    pub perm_mask: u32,                   // Bitmask permissions
    pub created_at: u64,
    pub updated_at: u64,
}
```

#### Supporting Enums

```rust
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum ResourceType {
    Memory,
    Gallery,
    Folder,
    Capsule,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum GrantSource {
    User,           // Direct user grant
    Group,          // Group membership grant
    MagicLink,      // Magic link grant
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
pub enum PublicMode {
    Private,        // No public access
    PublicAuth,     // Public for authenticated users
    PublicLink,     // Public for anyone with link
}
```

### 3. Universal Access Control Interface

#### AccessControlled Trait

```rust
pub trait AccessControlled {
    fn access_entries(&self) -> &[AccessEntry];
    fn public_policy(&self) -> Option<&PublicPolicy>;
}
```

#### PrincipalContext

```rust
pub struct PrincipalContext {
    pub principal: Principal,
    pub groups: Vec<String>,
    pub link: Option<String>,
    pub now_ns: u64, // use ic_cdk::api::time()
}
```

### 4. Permission Evaluation Engine

#### Core Evaluation Function

```rust
pub fn effective_perm_mask<T: AccessControlled>(
    resource: &T,
    ctx: &PrincipalContext,
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
        m |= link_mask_if_valid(resource, token, ctx.now_ns);
    }

    // 4) Public policy - check public access rules
    m |= public_mask_if_any(resource.public_policy(), ctx);

    m
}

// Helper function for permission checks
pub fn has_perm<T: AccessControlled>(
    res: &T,
    ctx: &PrincipalContext,
    want: Perm
) -> bool {
    (effective_perm_mask(res, ctx) & want.bits()) != 0
}
```

## Resource Integration

### Memory Structure

```rust
pub struct Memory {
    pub id: String,
    pub capsule_id: String,
    pub metadata: MemoryMetadata,
    pub access: MemoryAccess,                    // Legacy field (backward compatibility)
    pub access_entries: Vec<AccessEntry>,        // NEW: Direct access control
    pub public_policy: Option<PublicPolicy>,     // NEW: Public access rules
    pub inline_assets: Vec<MemoryAssetInline>,
    pub blob_internal_assets: Vec<MemoryAssetBlobInternal>,
    pub blob_external_assets: Vec<MemoryAssetBlobExternal>,
}
```

### Gallery Structure

```rust
pub struct Gallery {
    pub id: String,
    pub owner_principal: Principal,
    pub title: String,
    pub description: Option<String>,
    pub is_public: bool,                         // Legacy field (backward compatibility)
    pub access_entries: Vec<AccessEntry>,        // NEW: Direct access control
    pub public_policy: Option<PublicPolicy>,     // NEW: Public access rules
    pub created_at: u64,
    pub updated_at: u64,
    pub storage_location: GalleryStorageLocation,
    pub memory_entries: Vec<GalleryMemoryEntry>,
    pub bound_to_neon: bool,
}
```

## Optional Centralized Components

### Magic Link Index (Optional)

For performance-critical magic link lookups, a minimal centralized index can be implemented:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize, CandidType)]
pub struct LinkHash(String);

#[derive(Clone, Debug, Serialize, Deserialize, CandidType)]
pub struct LinkEntry {
    pub resource_type: ResourceType,
    pub resource_id: String,
    pub perm_mask: u32,
    pub expires_at: u64,
}

pub struct MagicLinkIndex {
    pub links: StableBTreeMap<LinkHash, LinkEntry, VirtualMemory<DefaultMemoryImpl>>,
}
```

**When to Use Centralized Index:**

- "List everything user U can access" across many resources
- Bulk revocation/auditing across a capsule
- Magic-link lookup by hash → resource (fast-path map)

## Time Management

### ICP Time Standardization

```rust
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

## Storage Architecture

### Capsule-Level Storage

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

### Storage Considerations

- **Resources in Heap**: Access control data lives with the resource in memory
- **Stable Snapshots**: Periodic persistence to stable storage
- **Per-Resource Stable**: Alternative approach using `StableBTreeMap<id, resource>`
- **No Global Maps**: Avoid complex cross-resource stable structures

## Implementation Status

### ✅ Completed (Phase 1)

- [x] Bitflags permission system (`Perm`)
- [x] Role templates and default roles
- [x] Universal access control types (`AccessEntry`, `PublicPolicy`)
- [x] `AccessControlled` trait and shared evaluation logic
- [x] `PrincipalContext` for request context
- [x] Time normalization utilities
- [x] Core permission evaluation functions

### 🔄 In Progress

- [ ] Resource struct integration (Memory, Gallery)
- [ ] `AccessControlled` trait implementation for resources
- [ ] Backward compatibility with existing access fields

### 📋 Planned

- [ ] Optional magic link index implementation
- [ ] Comprehensive testing suite
- [ ] API integration and validation
- [ ] Documentation and examples

## API Integration

### Permission Check Pattern

```rust
// Example usage in API endpoints
pub fn get_memory(memory_id: String, ctx: PrincipalContext) -> Result<Memory, Error> {
    let memory = get_memory_by_id(&memory_id)?;

    // Check VIEW permission
    if !has_perm(&memory, &ctx, Perm::VIEW) {
        return Err(Error::AccessDenied);
    }

    Ok(memory)
}
```

### Access Grant Pattern

```rust
// Example: Grant access to a memory
pub fn grant_memory_access(
    memory_id: String,
    person_ref: PersonRef,
    perm_mask: u32,
    ctx: PrincipalContext,
) -> Result<(), Error> {
    let mut memory = get_memory_by_id(&memory_id)?;

    // Check MANAGE permission
    if !has_perm(&memory, &ctx, Perm::MANAGE) {
        return Err(Error::AccessDenied);
    }

    // Add access entry
    let access_entry = AccessEntry {
        id: generate_id(),
        person_ref,
        grant_source: GrantSource::User,
        source_id: None,
        role: ResourceRole::Member,
        perm_mask,
        invited_by_person_ref: Some(PersonRef::Principal(ctx.principal)),
        created_at: ctx.now_ns,
        updated_at: ctx.now_ns,
    };

    memory.access_entries.push(access_entry);
    save_memory(memory)?;

    Ok(())
}
```

## Security Considerations

### Access Control Validation

1. **Principal Verification**: Always validate caller principal
2. **Permission Inheritance**: Clear hierarchy (OWN > MANAGE > SHARE > DOWNLOAD > VIEW)
3. **Grant Source Tracking**: Audit trail for all access grants
4. **Expiration Handling**: Automatic cleanup of expired magic links
5. **Group Membership**: Validate group membership before granting group-based access

### Attack Prevention

- **Permission Escalation**: Strict permission checking prevents unauthorized access
- **Magic Link Abuse**: Time-limited, single-use tokens with proper hashing
- **Cross-Capsule Access**: No global access - each capsule is isolated
- **Data Integrity**: Access control data is part of resource, ensuring consistency

## Performance Characteristics

### Decentralized Benefits

- **O(1) Access Checks**: Direct field access on resource
- **No Index Overhead**: No separate stable structure lookups
- **Cache Friendly**: Access data co-located with resource data
- **Sharding Ready**: Natural distribution across canister boundaries

### Scalability Metrics

- **Memory Overhead**: ~100-200 bytes per access entry
- **Evaluation Time**: <1ms for typical permission checks
- **Storage Growth**: Linear with number of access grants
- **Upgrade Impact**: Minimal - access data survives canister upgrades

## Migration Strategy

### Backward Compatibility

- **Legacy Fields**: Keep existing `access` and `is_public` fields
- **Dual Support**: Support both old and new access control during transition
- **Gradual Migration**: Migrate resources on access/modification
- **No Breaking Changes**: Existing APIs continue to work

### Migration Path

1. **Phase 1**: Implement new access control alongside existing system
2. **Phase 2**: Migrate resources to new system on access
3. **Phase 3**: Deprecate legacy access fields
4. **Phase 4**: Remove legacy code after full migration

## Future Considerations

### Potential Enhancements

- **Fine-Grained Permissions**: Additional permission types as needed
- **Conditional Access**: Time-based or context-based access rules
- **Access Analytics**: Usage tracking and audit logs
- **Bulk Operations**: Efficient batch permission management
- **Cross-Capsule Sharing**: Future support for inter-capsule access

### Centralized Approach Reference

If future requirements demand centralized access control, a reference implementation is available in [Capsule Access Control - Centralized Approach Reference](../issues/open/name-titile/capsule-access-centralized-reference.md).

**When to Consider Centralized:**

- Global access queries across many resources
- Bulk revocation/auditing operations
- Complex cross-resource access patterns
- Centralized access management requirements

## Dependencies

### Rust Crates

```toml
[dependencies]
bitflags = { version = "2.4", features = ["serde"] }
ic-cdk = "0.12"
ic-stable-structures = "0.5"
candid = "0.9"
serde = { version = "1.0", features = ["derive"] }
```

### Internal Dependencies

- [Capsule Module Architecture](./capsule-module-architecture.md)
- [Backend API Documentation](./backend-api-documentation.md)
- [Memory Creation API](./backend-memory-creation-api.md)
- [Gallery Type Refactor Implementation](../issues/open/name-titile/gallery-type-refactor-implementation.md)

## Related Documentation

- [Capsule Access Refactoring - Phase 1 Implementation](../issues/open/name-titile/capsule-access-refactoring.md)
- [Capsule Access Control - Centralized Approach Reference](../issues/open/name-titile/capsule-access-centralized-reference.md)
- [Backend Type Consistency Analysis](./type-consistency-analysis.md)
- [Testing Strategy for ICP](./testing-strategy-icp.md)

---

---

# NextJS Backend Access Control

## Overview

The Futura NextJS backend implements a **Universal Resource Sharing System** that provides unified access control for galleries, memories, and folders using bitmask permissions with comprehensive provenance tracking. This system is built on PostgreSQL with Drizzle ORM and integrates seamlessly with the ICP backend.

## Design Philosophy

### Core Principles

1. **Universal Resource Support**: Single system works for all resource types (galleries, memories, folders)
2. **Bitmask Permissions**: Atomic permission operations with clear hierarchy
3. **Provenance Tracking**: Complete audit trail of permission grants and sources
4. **Magic Link System**: Time-limited, use-limited access tokens
5. **Public Access Policies**: First-class support for public sharing modes
6. **Pure Database Implementation**: No generated columns, triggers, or views

## Architecture Components

### 1. Permission System

#### Bitmask Permissions

```typescript
// Permission bits (TypeScript constants; stored as single integer permMask)
export const PERM = {
  VIEW: 1 << 0, // 1
  DOWNLOAD: 1 << 1, // 2
  SHARE: 1 << 2, // 4
  MANAGE: 1 << 3, // 8
  OWN: 1 << 4, // 16
} as const;

// Permission checking helpers
export const canView = (mask: number) => has(mask, PERM.VIEW);
export const canDownload = (mask: number) => has(mask, PERM.DOWNLOAD);
export const canShare = (mask: number) => has(mask, PERM.SHARE);
export const canManage = (mask: number) => has(mask, PERM.MANAGE);
export const canOwn = (mask: number) => has(mask, PERM.OWN);
```

#### Role Templates

```typescript
export const roleTemplates = pgTable("role_template", {
  role: text("role", {
    enum: ["owner", "superadmin", "admin", "member", "guest"],
  }).primaryKey(),
  resourceType: text("resource_type", {
    enum: ["gallery", "memory", "folder"],
  }).notNull(),
  permMask: integer("perm_mask").notNull(), // sum of PERM bits
  createdAt: timestamp("created_at").notNull().defaultNow(),
  updatedAt: timestamp("updated_at").notNull().defaultNow(),
});
```

### 2. Core Database Schema

#### Resource Membership (Core Access Control)

```typescript
export const resourceMembership = pgTable("resource_membership", {
  id: text("id")
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID()),
  resourceType: text("resource_type", {
    enum: ["gallery", "memory", "folder"],
  }).notNull(),
  resourceId: text("resource_id").notNull(),
  allUserId: text("all_user_id").notNull(), // FK to allUsers.id

  // Provenance of the grant
  grantSource: text("grant_source", {
    enum: ["user", "group", "magic_link", "public_mode", "system"],
  }).notNull(),
  sourceId: text("source_id"), // e.g., group id or magic_link id
  role: text("role", {
    enum: ["owner", "superadmin", "admin", "member", "guest"],
  }).notNull(),
  permMask: integer("perm_mask").notNull().default(0),
  invitedByAllUserId: text("invited_by_all_user_id"),
  createdAt: timestamp("created_at").notNull().defaultNow(),
  updatedAt: timestamp("updated_at").notNull().defaultNow(),
});
```

#### Public Access Policy

```typescript
export const resourcePublicPolicy = pgTable("resource_public_policy", {
  id: text("id")
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID()),
  resourceType: text("resource_type", {
    enum: ["gallery", "memory", "folder"],
  }).notNull(),
  resourceId: text("resource_id").notNull(),
  mode: text("mode", {
    enum: ["private", "public_auth", "public_link"],
  })
    .notNull()
    .default("private"),
  linkTokenHash: text("link_token_hash"), // sha-256 of token (public_link only)
  permMask: integer("perm_mask").notNull().default(PERM.VIEW),
  expiresAt: timestamp("expires_at"),
  revokedAt: timestamp("revoked_at"),
  createdAt: timestamp("created_at").notNull().defaultNow(),
  updatedAt: timestamp("updated_at").notNull().defaultNow(),
});
```

#### Magic Link System

```typescript
export const magicLink = pgTable("magic_link", {
  id: text("id")
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID()),
  tokenHash: text("token_hash").notNull().unique(),
  type: text("type", {
    enum: ["admin_invite", "guest_share"],
  }).notNull(),
  resourceType: text("resource_type", {
    enum: ["gallery", "memory", "folder"],
  }).notNull(),
  resourceId: text("resource_id").notNull(),
  inviterAllUserId: text("inviter_all_user_id").notNull(),
  intendedEmail: text("intended_email"), // for admin_invite
  adminSubtype: text("admin_subtype", {
    enum: ["superadmin", "admin"],
  }), // for admin_invite
  presetPermMask: integer("preset_perm_mask").notNull().default(PERM.VIEW),
  maxUses: integer("max_uses").notNull().default(1000),
  usedCount: integer("used_count").notNull().default(0),
  expiresAt: timestamp("expires_at").notNull(),
  revokedAt: timestamp("revoked_at"),
  lastUsedAt: timestamp("last_used_at"),
  createdAt: timestamp("created_at").notNull().defaultNow(),
  updatedAt: timestamp("updated_at").notNull().defaultNow(),
});
```

### 3. User Management System

#### All Users Table (Universal User Reference)

```typescript
export const allUsers = pgTable("all_user", {
  id: text("id")
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID()),
  type: text("type", { enum: ["user", "temporary"] }).notNull(),
  userId: text("user_id"), // FK to registered users
  temporaryUserId: text("temporary_user_id"), // FK to temporary users
  createdAt: timestamp("created_at").defaultNow().notNull(),
});
```

#### Registered Users

```typescript
export const users = pgTable("user", {
  id: text("id")
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID()),
  name: text("name"),
  email: text("email").unique(),
  emailVerified: timestamp("emailVerified", { mode: "date" }),
  image: text("image"),
  // Platform role and permissions
  role: text("role", {
    enum: ["user", "moderator", "admin", "developer", "superadmin"],
  })
    .default("user")
    .notNull(),
  plan: text("plan", {
    enum: ["free", "premium"],
  })
    .default("free")
    .notNull(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
  updatedAt: timestamp("updated_at").defaultNow().notNull(),
});
```

### 4. API Implementation

#### Resource Access Checking

```typescript
// Example from memories/shared/route.ts
export async function GET(request: NextRequest) {
  const session = await auth();
  let allUserId: string;

  if (session?.user?.id) {
    // Handle authenticated user
    const allUserRecord = await db.query.allUsers.findFirst({
      where: eq(allUsers.userId, session.user.id),
    });
    allUserId = allUserRecord.id;
  } else {
    // Handle temporary user via allUserId in request body
    const body = await request.json();
    allUserId = body.allUserId;
  }

  // Query shared memories using resourceMembership table
  const sharedMemories = await db
    .select()
    .from(memories)
    .innerJoin(resourceMembership, eq(memories.id, resourceMembership.resourceId))
    .where(and(eq(resourceMembership.allUserId, allUserId), eq(resourceMembership.resourceType, "memory")));
}
```

#### Magic Link Access

```typescript
// Example from memories/[id]/share-link/route.ts
export async function GET(request: NextRequest, context: { params: Promise<{ id: string }> }) {
  const { id } = await context.params;
  const { searchParams } = new URL(request.url);
  const secureCode = searchParams.get("code");

  // Check if this is an owner's secure code
  if (memory.ownerSecureCode === secureCode) {
    return NextResponse.json({
      type: memory.type,
      data: memory,
      isOwner: true,
    });
  }

  // Check if it's a valid share code via resourceMembership
  const share = await db.query.resourceMembership.findFirst({
    where: and(
      eq(resourceMembership.resourceId, id),
      eq(resourceMembership.grantSource, "magic_link")
      // Additional magic link validation logic
    ),
  });
}
```

### 5. Permission Evaluation Logic

#### Effective Permission Calculation

```typescript
// Permission helper functions
export const has = (mask: number, bit: number) => (mask & bit) !== 0;
export const add = (mask: number, bit: number) => mask | bit;
export const remove = (mask: number, bit: number) => mask & ~bit;
export const merge = (...masks: number[]) => masks.reduce((acc, m) => acc | m, 0);

// Compute effective permissions from multiple grants
type Grant = { permMask: number };
export function effectiveMask(grants: Grant[]): number {
  return merge(...grants.map((g) => g.permMask));
}

// Role-based permission defaults
export const getRolePermissionMask = (role: ResourceRole, _resourceType: ResourceType): number => {
  const defaults: Record<ResourceRole, number> = {
    owner: PERM.VIEW | PERM.DOWNLOAD | PERM.SHARE | PERM.MANAGE | PERM.OWN,
    superadmin: PERM.VIEW | PERM.DOWNLOAD | PERM.SHARE | PERM.MANAGE,
    admin: PERM.VIEW | PERM.DOWNLOAD | PERM.SHARE | PERM.MANAGE,
    member: PERM.VIEW | PERM.DOWNLOAD,
    guest: PERM.VIEW,
  };
  return defaults[role];
};
```

## Implementation Status

### ✅ Completed

- [x] Universal resource sharing system with bitmask permissions
- [x] Resource membership table with provenance tracking
- [x] Magic link system with TTL and use limits
- [x] Public access policy system
- [x] Role templates and permission helpers
- [x] All users system (registered + temporary users)
- [x] API routes for memory and gallery sharing
- [x] Secure code-based access for legacy compatibility

### 🔄 In Progress

- [ ] ICP backend synchronization
- [ ] Advanced permission caching
- [ ] Bulk permission operations

### 📋 Planned

- [ ] Real-time permission updates
- [ ] Comprehensive audit logging
- [ ] Performance optimization for large permission sets

## Security Considerations

### Access Control Validation

1. **Bitmask Security**: Atomic permission operations prevent race conditions
2. **Provenance Tracking**: Complete audit trail of all permission grants
3. **Magic Link Security**: Time-limited, use-limited tokens with proper hashing
4. **Public Access Control**: Granular public sharing with expiration support
5. **User Type Support**: Handles both registered and temporary users securely

### Attack Prevention

- **Permission Escalation**: Strict role hierarchy and bitmask validation
- **Magic Link Abuse**: Proper token hashing, expiration, and use limits
- **SQL Injection**: Drizzle ORM with parameterized queries
- **Session Security**: NextAuth.js with secure session handling
- **Data Integrity**: Foreign key constraints and unique indexes

---

**Note**: This architecture document represents the design specification as of October 2025. The system is designed to evolve with future requirements while maintaining backward compatibility and performance characteristics suitable for both ICP and NextJS deployments.
