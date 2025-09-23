# Capsule Architecture and User Identity Management

**Status:** Open Issue  
**Priority:** High  
**Created:** January 2025  
**Assigned:** Tech Lead & Architecture Team

## Problem Description

The current `PersonRef` and capsule architecture doesn't properly handle the dual nature of our system (ICP + Web2). We need to design a clear architecture that supports:

1. **ICP-only users** (purists who want no Web2 contact)
2. **Web2 users** who can also have capsules
3. **Cross-platform users** who exist in both systems
4. **Deceased person management** (owners vs subjects)

## Current Architecture Issues

### 1. PersonRef Limitation

```rust
pub enum PersonRef {
    Principal(Principal), // ICP user
    Opaque(String),       // Non-principal (deceased, etc.)
}
```

**Problem**: Doesn't indicate where user data is stored or how to resolve identity across systems.

### 2. Capsule Ownership vs Subject Confusion

```rust
pub struct Capsule {
    pub subject: PersonRef,                    // Who this capsule is about
    pub owners: HashMap<PersonRef, OwnerState>, // Who owns/manages this capsule
    // ...
}
```

**Problem**: No clear distinction between:

- **Self-capsule**: `subject == owner` (user's personal capsule)
- **Managed capsule**: `subject != owner` (e.g., deceased person managed by family)

### 3. Storage Location Unknown

- **ICP users**: Data stored in ICP canister + optionally Neon DB
- **Web2 users**: Data stored only in Neon DB
- **Cross-platform**: Data in both systems
- **But**: `PersonRef` doesn't tell us which database contains the user!

## Proposed Architecture

### 1. Enhanced PersonRef with Storage Location

```rust
pub enum PersonRef {
    Principal(Principal),                    // ICP-native user (Internet Identity)
    Opaque(String),                         // Non-principal subject (deceased, etc.)

    // NEW: Cross-platform user with storage location
    CrossPlatform {
        principal: Option<Principal>,        // ICP principal (if any)
        neon_user_id: Option<String>,        // Neon DB user ID (if any)
        storage_location: StorageLocation,   // Where user data lives
    },
}

pub enum StorageLocation {
    ICPOnly,        // User exists only on ICP (purist)
    NeonOnly,       // User exists only in Neon DB (Web2-only)
    Both,           // User exists in both systems (hybrid)
}
```

### 2. Capsule Type Classification

```rust
// Base capsule trait
pub trait CapsuleType {
    fn get_subject(&self) -> &PersonRef;
    fn get_owners(&self) -> &HashMap<PersonRef, OwnerState>;
    fn is_self_capsule(&self) -> bool;
    fn is_managed_capsule(&self) -> bool;
}

// Self-capsule: subject == owner (user's personal capsule)
pub struct SelfCapsule {
    pub capsule: Capsule,
}

impl CapsuleType for SelfCapsule {
    fn is_self_capsule(&self) -> bool { true }
    fn is_managed_capsule(&self) -> bool { false }
}

// Managed capsule: subject != owner (e.g., deceased person)
pub struct ManagedCapsule {
    pub capsule: Capsule,
    pub management_type: ManagementType,
}

pub enum ManagementType {
    DeceasedPerson,     // Managing deceased person's memories
    MinorChild,         // Managing child's capsule
    IncapacitatedAdult, // Managing adult who can't manage their own
    LegacyCapsule,      // Inherited capsule
}

impl CapsuleType for ManagedCapsule {
    fn is_self_capsule(&self) -> bool { false }
    fn is_managed_capsule(&self) -> bool { true }
}
```

### 3. User Visibility and Contact Rules

```rust
pub struct UserVisibilityRules {
    pub can_see_web2_users: bool,    // Can this user see Web2 users?
    pub visible_to_web2_users: bool, // Can Web2 users see this user?
    pub can_contact_web2_users: bool, // Can this user contact Web2 users?
    pub contactable_by_web2_users: bool, // Can Web2 users contact this user?
}

// Default rules for different user types
impl UserVisibilityRules {
    pub fn icp_purist() -> Self {
        Self {
            can_see_web2_users: false,
            visible_to_web2_users: false,
            can_contact_web2_users: false,
            contactable_by_web2_users: false,
        }
    }

    pub fn web2_only() -> Self {
        Self {
            can_see_web2_users: true,
            visible_to_web2_users: true,
            can_contact_web2_users: true,
            contactable_by_web2_users: true,
        }
    }

    pub fn hybrid() -> Self {
        Self {
            can_see_web2_users: true,
            visible_to_web2_users: true,
            can_contact_web2_users: true,
            contactable_by_web2_users: true,
        }
    }
}
```

## User Types and Capabilities

### 1. ICP Purist User

- **Authentication**: Internet Identity only
- **Storage**: ICP canister only
- **Visibility**: Not visible to Web2 users
- **Contact**: Cannot contact Web2 users
- **Capsules**: Can create and manage self-capsules
- **Use Case**: Privacy-focused users who want ICP-only experience

### 2. Web2-Only User

- **Authentication**: Google, email/password, etc.
- **Storage**: Neon DB only
- **Visibility**: Visible to other Web2 users
- **Contact**: Can contact other Web2 users
- **Capsules**: Can create capsules (stored in ICP but managed via Web2)
- **Use Case**: Traditional web users who don't want to learn Internet Identity

### 3. Hybrid User

- **Authentication**: Both Internet Identity and Web2 providers
- **Storage**: Both ICP canister and Neon DB
- **Visibility**: Visible to both ICP and Web2 users
- **Contact**: Can contact users in both systems
- **Capsules**: Full access to all capsule features
- **Use Case**: Users who want the best of both worlds

## Capsule Creation Rules

### Self-Capsule Creation

```rust
pub fn create_self_capsule(caller: PersonRef) -> Result<SelfCapsule> {
    match caller {
        PersonRef::Principal(principal) => {
            // ICP user - create capsule directly
            create_icp_capsule(principal)
        },
        PersonRef::CrossPlatform { neon_user_id: Some(id), .. } => {
            // Web2 user - create capsule via Web2 API
            create_web2_capsule(id)
        },
        _ => Err(Error::Unauthorized("Cannot create self-capsule"))
    }
}
```

### Managed Capsule Creation

```rust
pub fn create_managed_capsule(
    owner: PersonRef,
    subject: PersonRef,
    management_type: ManagementType
) -> Result<ManagedCapsule> {
    // Verify owner has permission to manage subject
    verify_management_permission(&owner, &subject, &management_type)?;

    // Create capsule with subject != owner
    let capsule = Capsule::new(subject, owner);
    Ok(ManagedCapsule { capsule, management_type })
}
```

## Database Schema Updates

### Neon DB Updates

```sql
-- Add storage location to users table
ALTER TABLE users ADD COLUMN storage_location VARCHAR(20) DEFAULT 'neon_only';
ALTER TABLE users ADD COLUMN icp_principal TEXT;
ALTER TABLE users ADD COLUMN visibility_rules JSONB DEFAULT '{}';

-- Add capsule management table
CREATE TABLE capsule_management (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    capsule_id TEXT NOT NULL,
    owner_user_id TEXT NOT NULL REFERENCES users(id),
    subject_user_id TEXT, -- NULL for self-capsules
    management_type VARCHAR(50) NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

### ICP Canister Updates

```rust
// Add to canister state
pub struct CanisterState {
    // ... existing fields
    pub user_visibility_rules: HashMap<Principal, UserVisibilityRules>,
    pub capsule_management: HashMap<String, ManagementType>,
}
```

## Implementation Phases

### Phase 1: Enhanced PersonRef

- [ ] Extend `PersonRef` with storage location
- [ ] Update all existing code to handle new variants
- [ ] Add migration for existing data

### Phase 2: Capsule Type System

- [ ] Implement `CapsuleType` trait
- [ ] Create `SelfCapsule` and `ManagedCapsule` structs
- [ ] Update capsule creation logic

### Phase 3: User Visibility Rules

- [ ] Implement visibility and contact rules
- [ ] Update user search and discovery
- [ ] Add privacy controls

### Phase 4: Managed Capsules

- [ ] Implement deceased person management
- [ ] Add family/guardian management
- [ ] Create inheritance and transfer mechanisms

## Technical Challenges

### 1. Identity Resolution

- How to resolve user identity across ICP and Neon DB
- How to handle users who exist in both systems
- How to merge user data from both sources

### 2. Privacy and Visibility

- How to enforce visibility rules across systems
- How to prevent unauthorized contact
- How to handle cross-platform social features

### 3. Data Consistency

- How to keep user data in sync between systems
- How to handle conflicts between ICP and Neon DB
- How to ensure data integrity

### 4. Migration Strategy

- How to migrate existing users to new architecture
- How to handle users who want to change their type
- How to preserve existing capsule data

## Success Metrics

- [ ] All user types can create and manage capsules
- [ ] Privacy rules are properly enforced
- [ ] Cross-platform users have seamless experience
- [ ] Deceased person management works correctly
- [ ] No data loss during migration

## Related Issues

- [Mainnet Capsules Create Failure](./mainnet_capsules_create_failure.md)
- [ICP-First User Architecture](./icp_first_user_architecture.md)
- [Personal Canister Management Functions](./personal_canister_management_functions.md)

## Next Steps

1. **Architecture Review**: Get team consensus on proposed architecture
2. **Technical Design**: Create detailed technical specifications
3. **Migration Plan**: Design migration strategy for existing users
4. **Implementation**: Start with Phase 1 (Enhanced PersonRef)
5. **Testing**: Comprehensive testing of all user types and scenarios
