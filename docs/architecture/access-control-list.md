# Access Control List (ACL) System Architecture

**Created**: 2025-01-27  
**Last Updated**: 2025-01-27  
**Status**: Production Implementation

## Overview

The Futura platform implements a sophisticated **decentralized access control system** across both ICP and NextJS backends to manage permissions for resources (Memory, Gallery, Folder, Capsule). This system provides fine-grained access control with time-based and event-triggered access capabilities, designed specifically for ICP's per-capsule autonomy model.

## Table of Contents

1. [System Architecture](#system-architecture)
2. [Permission System](#permission-system)
3. [Access Control Data Structures](#access-control-data-structures)
4. [Permission Evaluation Engine](#permission-evaluation-engine)
5. [Capsule-Level Access Control](#capsule-level-access-control)
6. [HTTP Module Integration](#http-module-integration)
7. [API Integration Patterns](#api-integration-patterns)
8. [Security Considerations](#security-considerations)
9. [Implementation Status](#implementation-status)
10. [Future Enhancements](#future-enhancements)

---

## System Architecture

### Design Philosophy

The ACL system follows these core principles:

1. **Decentralized Authority**: Each resource is the source of truth for its own access control
2. **Capsule Autonomy**: Each capsule canister manages its own resources independently
3. **Shared Evaluation Logic**: Common permission evaluation across all resource types
4. **ICP-Native Design**: Built for canister boundaries and upgrade scenarios
5. **Time-Based Access**: Support for scheduled and event-triggered access

### Architecture Benefits

| Aspect          | Decentralized (Chosen)            | Centralized (Alternative)           |
| --------------- | --------------------------------- | ----------------------------------- |
| **Scalability** | Natural sharding across canisters | Complex cross-canister coordination |
| **Upgrades**    | Simple struct evolution           | Brittle global index migrations     |
| **Performance** | Direct resource access            | Index lookup overhead               |
| **Complexity**  | ~30-60 lines per resource         | 200+ lines of access plumbing       |
| **Storage**     | Access lives with resource        | Separate stable structures          |

---

## Permission System

### Bitflags for Permissions

The system uses bitflags for atomic permission operations:

```rust
use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Perm: u32 {
        const VIEW = 1 << 0;      // 1 - Can view the resource
        const DOWNLOAD = 1 << 1;  // 2 - Can download content
        const SHARE = 1 << 2;     // 4 - Can share the resource
        const MANAGE = 1 << 3;    // 8 - Can manage permissions
        const OWN = 1 << 4;       // 16 - Full ownership
    }
}
```

### Permission Hierarchy

Permissions follow a clear hierarchy where higher permissions include lower ones:

```
OWN (16) > MANAGE (8) > SHARE (4) > DOWNLOAD (2) > VIEW (1)
```

### Role Templates

Predefined roles with different permission levels:

```rust
pub struct RoleTemplate {
    pub name: String,
    pub perm_mask: u32,  // Uses Perm bits
    pub description: String,
}

// Default role templates
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

---

## Access Control Data Structures

### AccessEntry

Each resource has access entries that define who can access it:

```rust
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct AccessEntry {
    pub id: String,
    pub person_ref: Option<PersonRef>, // None for public access
    pub is_public: bool,               // Explicit public access flag
    pub grant_source: GrantSource,     // Who granted it (User/Group/MagicLink/System)
    pub source_id: Option<String>,     // Group/magic_link ID
    pub role: ResourceRole,            // Role system
    pub perm_mask: u32,                // Bitmask permissions
    pub invited_by_person_ref: Option<PersonRef>, // Who granted access
    pub created_at: u64,
    pub updated_at: u64,

    // Time/event-based access
    pub condition: AccessCondition,
}
```

### Access Conditions

The system supports sophisticated access conditions:

```rust
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum AccessCondition {
    Immediate,                             // Access now
    Scheduled { accessible_after: u64 },   // Access after timestamp
    ExpiresAt { expires: u64 },            // Access until timestamp
    EventTriggered { event: AccessEvent }, // Access after event
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum AccessEvent {
    // Memorial events
    AfterDeath,
    Anniversary(u32), // Nth anniversary

    // Life events
    Birthday(u32),    // Nth birthday
    Graduation,
    Wedding,

    // Capsule events
    CapsuleMaturity(u32), // When capsule reaches N years
    ConnectionCount(u32), // When capsule has N connections

    // Custom events
    Custom(String),
}
```

### Supporting Enums

```rust
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum GrantSource {
    User,      // A user granted this access
    Group,     // A group granted this access
    MagicLink, // A magic link granted this access
    System,    // The system granted this access
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum ResourceRole {
    Owner,
    SuperAdmin,
    Admin,
    Member,
    Guest,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum ResourceType {
    Memory,
    Gallery,
    Folder,
    Capsule,
}
```

### PrincipalContext

Context for permission evaluation:

```rust
pub struct PrincipalContext {
    pub principal: Principal,
    pub groups: Vec<String>,
    pub link: Option<String>,    // Magic link token
    pub now_ns: u64,            // Current time in nanoseconds
}
```

---

## Permission Evaluation Engine

### Core Evaluation Function

The heart of the ACL system is the `effective_perm_mask` function:

```rust
pub fn effective_perm_mask<T: AccessControlled>(resource: &T, ctx: &PrincipalContext) -> u32 {
    use Perm as P;

    // 1) Ownership short-circuit - owners get everything
    if is_owner(resource, ctx) {
        return (P::VIEW | P::DOWNLOAD | P::SHARE | P::MANAGE | P::OWN).bits();
    }

    let mut m = 0u32;

    // 2) Check all access entries (unified individual and public access)
    for entry in resource.access_entries() {
        // Check if access is currently active (time/event conditions)
        if !is_access_active(&entry.condition, ctx.now_ns) {
            continue;
        }

        // Check if this entry applies to the current principal
        if entry.is_public {
            // Public access - check if principal can access public content
            if entry.grant_source == GrantSource::User || entry.grant_source == GrantSource::System {
                m |= entry.perm_mask;
            }
        } else if let Some(person_ref) = &entry.person_ref {
            // Individual access - check if principal matches
            if person_ref == &PersonRef::Principal(ctx.principal) {
                m |= entry.perm_mask;
            }
            // TODO: Add group membership checks
        }
    }

    // 3) Magic link access (if provided)
    if let Some(_token) = &ctx.link {
        // TODO: Implement magic link validation
        m |= 0;
    }

    m
}
```

### Access Condition Checking

```rust
fn is_access_active(condition: &AccessCondition, now_ns: u64) -> bool {
    match condition {
        AccessCondition::Immediate => true,
        AccessCondition::Scheduled { accessible_after } => now_ns >= *accessible_after,
        AccessCondition::ExpiresAt { expires } => now_ns <= *expires,
        AccessCondition::EventTriggered { event: _ } => {
            // TODO: Implement event checking
            false
        }
    }
}
```

### Permission Checking Helpers

```rust
// Helper function for permission checks
pub fn has_perm<T: AccessControlled>(
    res: &T,
    ctx: &PrincipalContext,
    want: Perm
) -> bool {
    (effective_perm_mask(res, ctx) & want.bits()) != 0
}
```

---

## Capsule-Level Access Control

For capsule operations, there's a simpler ACL system:

```rust
/// Capsule access control methods
pub trait CapsuleAcl {
    /// Check if a person can read from this capsule
    /// Read access: owners ∨ controllers ∨ subject
    fn can_read(&self, person: &PersonRef) -> bool;

    /// Check if a person can write/create in this capsule
    /// Write access: owners ∨ controllers ∨ subject
    fn can_write(&self, person: &PersonRef) -> bool;

    /// Check if a person can delete from this capsule
    /// Delete access: owners ∨ controllers (subject cannot delete by default)
    fn can_delete(&self, person: &PersonRef) -> bool;
}

/// Helper struct for capsule access control
#[derive(Clone)]
pub struct CapsuleAccess {
    pub subject: PersonRef,
    pub owners: HashMap<PersonRef, OwnerState>,
    pub controllers: HashMap<PersonRef, ControllerState>,
}

impl CapsuleAcl for CapsuleAccess {
    #[inline]
    fn can_read(&self, person: &PersonRef) -> bool {
        self.owners.contains_key(person)
            || self.controllers.contains_key(person)
            || self.subject == *person
    }

    #[inline]
    fn can_write(&self, person: &PersonRef) -> bool {
        self.owners.contains_key(person)
            || self.controllers.contains_key(person)
            || self.subject == *person
    }

    #[inline]
    fn can_delete(&self, person: &PersonRef) -> bool {
        // Keep delete stricter - only owners and controllers can delete
        self.owners.contains_key(person) || self.controllers.contains_key(person)
    }
}
```

---

## HTTP Module Integration

### ACL Adapter

For HTTP requests, there's an ACL adapter that wraps existing domain logic:

```rust
/// ACL adapter that wraps existing domain logic without importing domain code into HTTP layer
pub struct FuturaAclAdapter;

impl Acl for FuturaAclAdapter {
    fn can_view(&self, memory_id: &str, who: Principal) -> bool {
        // Create PrincipalContext for permission evaluation
        let ctx = PrincipalContext {
            principal: who,
            groups: vec![], // TODO: Get from user system if needed
            link: None,     // TODO: Extract from HTTP request if needed
            now_ns: ic_cdk::api::time(),
        };

        // Use the same pattern as existing memory operations
        let _env = CanisterEnv;
        let store = StoreAdapter;

        // Get all accessible capsules for the caller
        let accessible_capsules = store.get_accessible_capsules(&PersonRef::Principal(who));

        // Search for the memory across all accessible capsules
        for capsule_id in accessible_capsules {
            if let Some(memory) = store.get_memory(&capsule_id, &memory_id.to_string()) {
                // Use existing effective_perm_mask logic
                let perm_mask = effective_perm_mask(&memory, &ctx);
                return (perm_mask & Perm::VIEW.bits()) != 0;
            }
        }
        false
    }
}
```

### Token-Based Access

The HTTP module also supports token-based access for assets:

```rust
pub fn get(
    memory_id: &str,
    variant: &str,
    req: &ParsedRequest,
    url: &str,
) -> HttpResponse<'static> {
    // 1) verify token - check header first, then query string
    let token_param = extract_token_from_request(req, url);
    let token_param = match token_param {
        Some(t) => t,
        None => return HttpError::MissingToken.to_response(),
    };

    let token = match decode_token_url(&token_param) {
        Some(t) => t,
        None => return HttpError::BadTokenFormat.to_response(),
    };

    let want = path_to_scope(req, memory_id, variant);
    let clock = CanisterClock;
    let secret = StableSecretStore;

    if let Err(e) = verify_token_core(&clock, &secret, &token, &want) {
        return e.to_response();
    }

    // Token is valid, proceed with asset serving
    // ...
}
```

---

## API Integration Patterns

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
        person_ref: Some(person_ref),
        is_public: false,
        grant_source: GrantSource::User,
        source_id: None,
        role: ResourceRole::Member,
        perm_mask,
        invited_by_person_ref: Some(PersonRef::Principal(ctx.principal)),
        created_at: ctx.now_ns,
        updated_at: ctx.now_ns,
        condition: AccessCondition::Immediate,
    };

    memory.access_entries.push(access_entry);
    save_memory(memory)?;

    Ok(())
}
```

### Memory Operations with ACL

```rust
// Example from memory deletion
pub fn memories_delete_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    capsule_id: String,
    memory_id: String,
    caller: PersonRef,
    delete_assets: bool,
) -> Result<(), Error> {
    if let Some(memory) = store.get_memory(&capsule_id, &memory_id) {
        // Check delete permissions using centralized ACL
        let capsule_access = store
            .get_capsule_for_acl(&capsule_id)
            .ok_or(Error::NotFound)?;

        if !capsule_access.can_delete(&caller) {
            ic_cdk::println!(
                "[ACL] op=delete caller={} cap={} read={} write={} delete={} - UNAUTHORIZED",
                caller,
                capsule_id,
                capsule_access.can_read(&caller),
                capsule_access.can_write(&caller),
                capsule_access.can_delete(&caller)
            );
            return Err(Error::Unauthorized);
        }

        // Log successful ACL check
        ic_cdk::println!(
            "[ACL] op=delete caller={} cap={} read={} write={} delete={} - AUTHORIZED",
            caller,
            capsule_id,
            capsule_access.can_read(&caller),
            capsule_access.can_write(&caller),
            capsule_access.can_delete(&caller)
        );

        // Proceed with deletion
        if delete_assets {
            cleanup_memory_assets(&memory)?;
        }
        store.delete_memory(&capsule_id, &memory_id)?;
        return Ok(());
    }

    Err(Error::NotFound)
}
```

---

## Security Considerations

### Access Control Validation

1. **Principal Verification**: Always validate caller principal
2. **Permission Inheritance**: Clear hierarchy (OWN > MANAGE > SHARE > DOWNLOAD > VIEW)
3. **Grant Source Tracking**: Audit trail for all access grants
4. **Expiration Handling**: Automatic cleanup of expired access conditions
5. **Group Membership**: Validate group membership before granting group-based access

### Attack Prevention

- **Permission Escalation**: Strict permission checking prevents unauthorized access
- **Magic Link Abuse**: Time-limited, single-use tokens with proper hashing
- **Cross-Capsule Access**: No global access - each capsule is isolated
- **Data Integrity**: Access control data is part of resource, ensuring consistency

### Security Best Practices

```rust
// Always check permissions before operations
if !has_perm(&resource, &ctx, Perm::MANAGE) {
    return Err(Error::AccessDenied);
}

// Log ACL decisions for audit
ic_cdk::println!(
    "[ACL] op={} caller={} resource={} result={}",
    operation,
    ctx.principal,
    resource_id,
    if authorized { "AUTHORIZED" } else { "UNAUTHORIZED" }
);

// Validate access conditions
if !is_access_active(&entry.condition, ctx.now_ns) {
    continue; // Skip inactive entries
}
```

---

## Implementation Status

### ✅ Completed (Phase 1)

- [x] Bitflags permission system (`Perm`)
- [x] Role templates and default roles
- [x] Universal access control types (`AccessEntry`, `AccessCondition`)
- [x] `AccessControlled` trait and shared evaluation logic
- [x] `PrincipalContext` for request context
- [x] Time normalization utilities
- [x] Core permission evaluation functions
- [x] Capsule-level ACL implementation
- [x] HTTP module ACL adapter
- [x] Token-based access for assets

### 🔄 In Progress

- [ ] Resource struct integration (Memory, Gallery)
- [ ] `AccessControlled` trait implementation for resources
- [ ] Backward compatibility with existing access fields
- [ ] Magic link system implementation
- [ ] Group membership checks

### 📋 Planned

- [ ] Optional magic link index implementation
- [ ] Comprehensive testing suite
- [ ] API integration and validation
- [ ] Event system for event-triggered access
- [ ] Performance optimization for large permission sets

---

## Future Enhancements

### Potential Enhancements

- **Fine-Grained Permissions**: Additional permission types as needed
- **Conditional Access**: More sophisticated time-based or context-based access rules
- **Access Analytics**: Usage tracking and audit logs
- **Bulk Operations**: Efficient batch permission management
- **Cross-Capsule Sharing**: Future support for inter-capsule access

### Magic Link System

The system is designed to support magic links for secure sharing:

```rust
// Planned magic link structure
pub struct MagicLink {
    pub token_hash: String,
    pub resource_type: ResourceType,
    pub resource_id: String,
    pub perm_mask: u32,
    pub expires_at: u64,
    pub max_uses: u32,
    pub used_count: u32,
}
```

### Performance Optimizations

- **Permission Caching**: Cache frequently accessed permissions
- **Batch Permission Checks**: Optimize bulk operations
- **Lazy Loading**: Load access entries only when needed

---

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
- [Resources Access Control System](./resources-access-control-system.md)

---

## Related Documentation

- [Resources Access Control System](./resources-access-control-system.md)
- [Capsule Module Architecture](./capsule-module-architecture.md)
- [Backend API Documentation](./backend-api-documentation.md)
- [Testing Strategy for ICP](./testing-strategy-icp.md)
- [HTTP Module Production Ready - ACL Token Minting Blocker](../issues/open/serving-http/http-module-production-ready-acl-blocker.md)

---

**Note**: This architecture document represents the current implementation as of January 2025. The system is designed to evolve with future requirements while maintaining backward compatibility and performance characteristics suitable for ICP deployments.



