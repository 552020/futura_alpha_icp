# User-Capsule Relationship Architecture Pivot

## Status: ✅ **DIRECTION UPDATE**

**Priority:** High  
**Effort:** Low  
**Impact:** High – Simplifies architecture and avoids unnecessary coupling

---

## Executive Summary

After reviewing our design for integrating Web2 (Neon) users with Web3 (ICP) capsules, we have determined that the proposed **user–capsule mapping** is **not necessary** for our MVP.

Capsules are **ICP-native objects**, tied to **principals**. The relationship between Web2 accounts and ICP principals is already managed through **Auth.js `accounts` table**. Attempting to force a direct coupling between `users.id` and `capsule.id` adds complexity without real benefit.

The **real integration point** is at the **memories and storage level**, where users may have content distributed between S3 (Web2) and ICP (Web3).

## Current Architecture

### ICP Backend - Capsule Definition

```rust
// Current capsule structure in backend/src/types.rs
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Capsule {
    pub id: String,                                          // unique capsule identifier
    pub subject: PersonRef,                                  // who this capsule is about
    pub owners: HashMap<PersonRef, OwnerState>,              // 1..n owners (usually 1)
    pub controllers: HashMap<PersonRef, ControllerState>,    // delegated admins (full control)
    pub connections: HashMap<PersonRef, Connection>,         // social graph
    pub connection_groups: HashMap<String, ConnectionGroup>, // organized connection groups
    pub memories: HashMap<String, Memory>,                   // content
    pub galleries: HashMap<String, Gallery>,                 // galleries (collections of memories)
    pub created_at: u64,
    pub updated_at: u64,
    pub bound_to_neon: bool,    // Neon database binding status
    pub inline_bytes_used: u64, // Track inline storage consumption
}
```

### ICP Backend - Capsule ID Generation

```rust
// Current capsule creation in backend/src/capsule.rs
impl Capsule {
    pub fn new(subject: PersonRef, initial_owner: PersonRef) -> Self {
        let now = time();
        let capsule_id = format!("capsule_{now}");  // ← CURRENT ID GENERATION

        // ... rest of capsule creation
    }
}

// Capsule creation function
pub fn capsules_create(subject: Option<PersonRef>) -> Result<Capsule, Error> {
    let caller = PersonRef::from_caller();
    let actual_subject = subject.unwrap_or_else(|| caller.clone());
    let capsule = Capsule::new(actual_subject, caller);  // ← Uses format!("capsule_{now}")
    // ... rest of creation logic
}
```

**Current ID Format**: `"capsule_{timestamp}"` (e.g., `"capsule_1703123456789"`)

### Web2 Database (Neon)

```typescript
// users table
{
  id: "user-uuid-123",           // Primary key (UUID)
  email: "user@example.com",
  name: "John Doe",
  // ... other fields
}

// accounts table (Auth.js)
{
  userId: "user-uuid-123",       // FK to users.id
  provider: "internet-identity", // or "google"
  providerAccountId: "principal-abc", // ICP Principal or Google ID
}
```

### ICP Backend

```typescript
// Capsule structure
{
  id: "capsule-uuid-456",        // Capsule UUID (different from user UUID)
  subject: { Principal: "principal-abc" },
  owners: Map<PersonRef, OwnerState>,
  // ... other capsule fields
}
```

## The Problem

**No direct relationship between Web2 users and ICP capsules:**

1. **Data Synchronization**: Cannot link Web2 user to their ICP capsule
2. **User Experience**: Frontend cannot easily fetch user's capsule data
3. **Migration Issues**: Personal canister migration cannot identify which user to migrate
4. **Multiple Capsules**: Users can own multiple capsules (self + deceased persons + organizations)

### Current ID Generation Issues

**Current Backend Behavior:**

- Capsule IDs are generated as `"capsule_{timestamp}"` (e.g., `"capsule_1703123456789"`)
- No relationship to Web2 user UUIDs
- Timestamp-based IDs are not human-readable or predictable
- No way to identify which capsules belong to which Web2 users

**Impact on Proposed Solutions:**

- **Option A**: Requires backend changes to accept custom UUIDs
- **Option D**: Requires backend changes to support UUID extension pattern
- **Option E**: Requires backend changes to support inheritance model
- **Options B & C**: No backend changes needed (use mapping tables)

## Three User Scenarios

### Scenario 1: Internet Identity User

```typescript
// User has ICP access via Internet Identity
const user = {
  id: "user-uuid-123",
  email: "user@example.com",
};

const account = {
  userId: "user-uuid-123",
  provider: "internet-identity",
  providerAccountId: "principal-abc", // ICP Principal
};
```

**Result**: ✅ Can create and manage capsules

### Scenario 2: Google-Only User

```typescript
// User has only Google account, no ICP access
const user = {
  id: "user-uuid-123",
  email: "user@gmail.com",
};

const account = {
  userId: "user-uuid-123",
  provider: "google",
  providerAccountId: "google-user-id",
};
```

**Result**: ❌ Cannot access capsules (no ICP account)

### Scenario 3: Google User + Later ICP Connection

```typescript
// User starts with Google, later connects Internet Identity
const user = {
  id: "user-uuid-123", // SAME user ID
  email: "user@gmail.com",
};

// Multiple accounts for same user
const googleAccount = {
  userId: "user-uuid-123",
  provider: "google",
  providerAccountId: "google-user-id",
};

const iiAccount = {
  userId: "user-uuid-123", // SAME user ID
  provider: "internet-identity",
  providerAccountId: "principal-abc",
};
```

**Result**: ✅ Can create and manage capsules after ICP connection

## Proposed Solutions

### Option A: Enforce User UUID = Capsule UUID

```typescript
// Self-capsule uses user's UUID as capsule ID
const selfCapsule = await actor.capsules_create({
  subject: { Principal: "principal-abc" },
});
// Force: capsule.id = user.id ("user-uuid-123")
```

**Pros:**

- Perfect 1:1 mapping for self-capsules
- No additional database fields needed
- Consistent identity across systems
- Easy to find user's self-capsule

**Cons:**

- ICP backend must accept pre-generated UUIDs
- Non-self capsules still need different UUIDs
- Requires backend changes

### Option B: Add Capsule ID to Users Table

```typescript
export const users = pgTable("user", {
  // ... existing fields
  capsuleId: text("capsule_id").unique(), // Link to ICP capsule
});
```

**Pros:**

- Simple 1:1 relationship
- Easy to query user's capsule
- No backend changes needed

**Cons:**

- Only works for self-capsules
- Doesn't handle multiple capsules per user
- Requires migration for existing users

### Option C: Create User-Capsule Mapping Table

```typescript
export const userCapsules = pgTable("user_capsules", {
  id: uuid("id").primaryKey().defaultRandom(),
  userId: text("user_id")
    .notNull()
    .references(() => users.id, { onDelete: "cascade" }),
  capsuleId: text("capsule_id").notNull(),
  capsuleType: text("capsule_type", {
    enum: ["self", "deceased", "organization", "other"],
  }).notNull(),
  isOwner: boolean("is_owner").default(true),
  isController: boolean("is_controller").default(false),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});
```

**Pros:**

- Handles multiple capsules per user
- Flexible relationship model
- Tracks ownership and control
- No backend changes needed

**Cons:**

- More complex queries
- Additional table to maintain
- Requires migration strategy

### Option D: UUID Extension Pattern

```typescript
// Base user UUID: "user-uuid-123"
// Self-capsule: "user-uuid-123" (same as user ID)
// Other capsules: "user-uuid-123_1", "user-uuid-123_2", etc.

const user = {
  id: "user-uuid-123",
  email: "user@example.com",
};

const capsules = [
  {
    id: "user-uuid-123", // Self-capsule (same as user ID)
    subject: { Principal: "principal-abc" },
    type: "self",
  },
  {
    id: "user-uuid-123_1", // Deceased person capsule
    subject: { Opaque: "deceased-dad" },
    type: "deceased",
  },
  {
    id: "user-uuid-123_2", // Organization capsule
    subject: { Opaque: "company-xyz" },
    type: "organization",
  },
];
```

**Pros:**

- Clear hierarchy: base UUID + extensions
- Easy to identify which capsules belong to which user
- No additional database tables needed
- Self-capsule maintains 1:1 relationship with user
- Human-readable capsule IDs

**Cons:**

- Requires backend changes to support custom UUID patterns
- Need validation for UUID extension format
- Potential conflicts if user manually creates "\_1" extension

### Option E: Base Capsule + Inheritance Pattern

```typescript
// Base capsule (abstract) - contains user's core data
interface BaseCapsule {
  id: "user-uuid-123"; // Base UUID (same as user ID)
  owner: { Principal: "principal-abc" };
  type: "base";
  // Core user data, relationships, etc.
}

// Self-capsule (inherits from base)
interface SelfCapsule extends BaseCapsule {
  id: "user-uuid-123"; // Same as base
  subject: { Principal: "principal-abc" }; // Owner = Subject
  type: "self";
  // Self-specific data
}

// Other capsules (inherit from base)
interface OtherCapsule extends BaseCapsule {
  id: "deceased-dad-456"; // Different UUID
  subject: { Opaque: "deceased-dad" }; // Owner ≠ Subject
  type: "deceased";
  // Other-specific data
}

// Inheritance relationship
const baseCapsule = {
  id: "user-uuid-123",
  owner: { Principal: "principal-abc" },
  type: "base",
};

const selfCapsule = {
  id: "user-uuid-123", // Inherits base ID
  subject: { Principal: "principal-abc" },
  type: "self",
  baseCapsuleId: "user-uuid-123", // Reference to base
};

const otherCapsule = {
  id: "deceased-dad-456", // Own UUID
  subject: { Opaque: "deceased-dad" },
  type: "deceased",
  baseCapsuleId: "user-uuid-123", // Reference to base
};
```

**Pros:**

- Clear inheritance hierarchy
- Base capsule contains shared user data
- Self-capsule maintains 1:1 relationship
- Other capsules can have different UUIDs
- Flexible and extensible design

**Cons:**

- More complex data structure
- Requires backend changes to support inheritance
- Need to manage base capsule lifecycle
- Potential for orphaned capsules if base is deleted

## Recommended Approach

**Option D: UUID Extension Pattern** (Recommended)

This approach provides the best balance of simplicity and functionality:

1. **Self-capsule**: Uses `user.id` directly (perfect 1:1 mapping)
2. **Other capsules**: Use `user.id + "_N"` pattern (clear hierarchy)
3. **No additional database tables needed**
4. **Human-readable and intuitive**

```typescript
// User and their capsules
const user = {
  id: "user-uuid-123",
  email: "user@example.com",
};

const capsules = [
  {
    id: "user-uuid-123", // Self-capsule (same as user ID)
    subject: { Principal: "principal-abc" },
    type: "self",
  },
  {
    id: "user-uuid-123_1", // Deceased person capsule
    subject: { Opaque: "deceased-dad" },
    type: "deceased",
  },
  {
    id: "user-uuid-123_2", // Organization capsule
    subject: { Opaque: "company-xyz" },
    type: "organization",
  },
];
```

**Alternative: Option E (Base Capsule + Inheritance)**

For more complex scenarios requiring shared data between capsules:

```typescript
// Base capsule contains shared user data
const baseCapsule = {
  id: "user-uuid-123",
  owner: { Principal: "principal-abc" },
  type: "base",
};

// Self-capsule inherits base ID
const selfCapsule = {
  id: "user-uuid-123", // Same as base
  subject: { Principal: "principal-abc" },
  type: "self",
  baseCapsuleId: "user-uuid-123",
};

// Other capsules have their own UUIDs but reference base
const otherCapsule = {
  id: "deceased-dad-456", // Different UUID
  subject: { Opaque: "deceased-dad" },
  type: "deceased",
  baseCapsuleId: "user-uuid-123", // Reference to base
};
```

## Technical Questions for Backend Team

### A) Capsule ID Policy

**1. Who generates `id: String` today?**

```rust
// Current implementation in backend/src/capsule.rs
let capsule_id = format!("capsule_{now}");  // ← CANISTER GENERATED
```

**Answer**: Currently **canister-generated only**. No client-supplied ID support.

**2. Can the create API accept client-supplied `id`?**
**Answer**: **NO** - Current API only accepts `subject: Option<PersonRef>`, no custom ID parameter.

**3. Constraints for client-supplied IDs (if implemented):**

- **Charset**: Would need to support UUID format (36 chars with hyphens)
- **Length**: Standard UUID length (36 characters)
- **Format**: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`

**4. Idempotency:**
**Answer**: Currently **NOT IDEMPOTENT** - each call creates new capsule with new timestamp ID.

**5. Immutability:**
**Answer**: **IMMUTABLE** - `id` field is never modified after creation.

**6. Indexing:**
**Answer**: **YES** - Capsules are stored in `CapsuleStore` with O(1) lookup by ID.

### B) Create/Lookup API Surface

**Current API Signatures:**

```rust
// Current functions in backend/src/capsule.rs
pub fn capsules_create(subject: Option<PersonRef>) -> Result<Capsule, Error>
pub fn capsules_read(capsule_id: String) -> Result<Capsule, Error>
pub fn capsules_read_basic(capsule_id: String) -> Result<CapsuleInfo, Error>
pub fn capsules_list() -> Vec<CapsuleHeader>
pub fn capsules_update(capsule_id: String, updates: CapsuleUpdateData) -> Result<Capsule, Error>
pub fn capsules_delete(capsule_id: String) -> Result<(), Error>
```

**Missing APIs:**

- ❌ `capsules_list_by_owner(owner: PersonRef) -> Vec<CapsuleId>`
- ❌ `capsules_list_by_subject(subject: PersonRef) -> Vec<CapsuleId>`
- ❌ Pagination support (currently uses `u32::MAX` limit)

### C) PersonRef & Principals

**PersonRef Definition:**

```rust
// From backend/src/types.rs
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PersonRef {
    Principal(Principal),  // Internet Identity principal
    Opaque(String),       // Non-principal (deceased, org, etc.)
}
```

**Answer**:

- **Type**: Union of `Principal` and `Opaque(String)`
- **II Principal**: Yes, stored in `PersonRef::Principal(principal)`
- **Normalization**: No special rules, direct principal comparison

### D) Ownership, Control, Authorization

**Current Authorization:**

```rust
// From backend/src/capsule.rs
pub fn capsules_create(subject: Option<PersonRef>) -> Result<Capsule, Error> {
    let caller = PersonRef::from_caller();  // ← Gets caller principal
    // ... creation logic
}

// Access checks
pub fn has_write_access(&self, person: &PersonRef) -> bool {
    self.is_owner(person) || self.is_controller(person)
}
```

**Answer**:

- **Multiple Owners**: **YES** - `HashMap<PersonRef, OwnerState>` supports multiple
- **Caller Checks**: **YES** - Only owners/controllers can modify
- **Transfer**: **NOT IMPLEMENTED** - No built-in transfer flow
- **Google-only Users**: **NO** - Must have principal to create capsules

### E) Neon Linkage Fields

**Current Field:**

```rust
pub struct Capsule {
    // ... other fields
    pub bound_to_neon: bool,    // ← ONLY CURRENT FIELD
    // ... other fields
}
```

**Missing Fields:**

- ❌ `neon_user_id: Option<String>`
- ❌ `external_ids: Vec<(String, String)>`

**Answer**: **NEEDS TO BE ADDED** - Current implementation only has boolean flag.

### F) Multi-Capsule Per User

**Current Policy:**

```rust
// From backend/src/capsule.rs - Self-capsule check
let existing_self_capsule = all_capsules
    .items
    .into_iter()
    .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller));

if let Some(capsule) = existing_self_capsule {
    // Return existing self-capsule
}
```

**Answer**:

- **Self-capsule**: **ONE PER USER** - Enforced by existing logic
- **Other capsules**: **UNLIMITED** - No constraints on deceased/org capsules
- **Capsule Types**: **NOT ENCODED** - No `capsule_type` field

### G) Consistency, Concurrency, Errors

**Current Error Types:**

```rust
// From backend/src/types.rs
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum Error {
    Internal(String),
    NotFound,
    Unauthorized,
    InvalidArgument(String),
    ResourceExhausted,
    NotImplemented(String),
    Conflict(String),
}
```

**Answer**:

- **Atomicity**: **YES** - Capsule creation is atomic
- **Error Handling**: **COMPREHENSIVE** - Covers all major error cases
- **Rate Limits**: **NOT IMPLEMENTED** - No current rate limiting

### H) Versioning & Migration

**Current Schema:**

```rust
pub struct Capsule {
    // ... fields
    pub created_at: u64,
    pub updated_at: u64,
    // ... fields
}
```

**Answer**:

- **Schema Version**: **NOT IMPLEMENTED** - No version field
- **Migration Path**: **NOT IMPLEMENTED** - No migration support
- **Backward Compatibility**: **NOT GUARANTEED** - Schema changes would break existing capsules

## Required Backend Changes

### 1. **Add Client-Supplied ID Support**

```rust
// New function needed
pub fn capsules_create_with_id(
    capsule_id: String,
    subject: Option<PersonRef>
) -> Result<Capsule, Error> {
    // Validate UUID format
    // Check for existing capsule with same ID
    // Create capsule with provided ID
}
```

### 2. **Add Neon User ID Field**

```rust
pub struct Capsule {
    // ... existing fields
    pub neon_user_id: Option<String>,  // ← ADD THIS FIELD
    // ... other fields
}
```

### 3. **Add Missing Query APIs**

```rust
// New functions needed
pub fn capsules_list_by_owner(owner: PersonRef) -> Vec<String>
pub fn capsules_list_by_subject(subject: PersonRef) -> Vec<String>
pub fn capsules_list_by_neon_user_id(neon_user_id: String) -> Vec<String>
```

### 4. **Add Capsule Type Field**

```rust
pub enum CapsuleType {
    Self,
    Deceased,
    Organization,
    Other,
}

pub struct Capsule {
    // ... existing fields
    pub capsule_type: CapsuleType,  // ← ADD THIS FIELD
    // ... other fields
}
```

## Implementation Questions

### 1. **Capsule ID Strategy**

- Should self-capsules use `user.id` as capsule ID?
- How do we handle non-self capsules (different UUIDs)?
- Does ICP backend support pre-generated UUIDs?

### 2. **Multiple Capsule Management**

- How do we track which capsules belong to which users?
- Do we need the `userCapsules` mapping table?
- How do we handle capsule ownership vs. control?

### 3. **Authentication Flow**

- When user connects ICP account, do we enforce same UUID?
- How do we handle existing users without ICP access?
- What happens during personal canister migration?

### 4. **Data Migration**

- How do we migrate existing Web2 users to have capsules?
- What about users who already have ICP accounts?
- Do we need a migration script?

### 5. **Capsule Creation Flow**

- Who creates the capsule ID? Web2 or ICP?
- How do we ensure consistency between systems?
- What happens if capsule creation fails?

## Technical Requirements

### Backend Changes Required

#### Option A: Custom UUID Support

```rust
// Required changes to backend/src/capsule.rs
impl Capsule {
    pub fn new_with_id(id: String, subject: PersonRef, initial_owner: PersonRef) -> Self {
        let now = time();
        // Use provided ID instead of generating timestamp-based ID
        let capsule_id = id;  // ← Accept custom ID

        // ... rest of capsule creation
    }
}

// Updated creation function
pub fn capsules_create_with_id(
    capsule_id: String,
    subject: Option<PersonRef>
) -> Result<Capsule, Error> {
    // Validate UUID format
    if !is_valid_uuid(&capsule_id) {
        return Err(Error::InvalidArgument("Invalid UUID format".to_string()));
    }

    let caller = PersonRef::from_caller();
    let actual_subject = subject.unwrap_or_else(|| caller.clone());
    let capsule = Capsule::new_with_id(capsule_id, actual_subject, caller);
    // ... rest of creation logic
}
```

#### Option D: UUID Extension Pattern

```rust
// Required changes to support UUID extensions
pub fn capsules_create_with_extension(
    base_uuid: String,
    extension: Option<String>,
    subject: Option<PersonRef>
) -> Result<Capsule, Error> {
    let caller = PersonRef::from_caller();
    let actual_subject = subject.unwrap_or_else(|| caller.clone());

    // Generate capsule ID based on pattern
    let capsule_id = if let Some(ext) = extension {
        format!("{}_{}", base_uuid, ext)  // e.g., "user-uuid-123_1"
    } else {
        base_uuid.clone()  // Self-capsule uses base UUID
    };

    let capsule = Capsule::new_with_id(capsule_id, actual_subject, caller);
    // ... rest of creation logic
}
```

#### Option E: Base Capsule + Inheritance

```rust
// Required changes to support base capsules
pub struct BaseCapsule {
    pub id: String,
    pub owner: PersonRef,
    pub created_at: u64,
    // ... base capsule fields
}

pub fn capsules_create_inherited(
    base_capsule_id: String,
    capsule_id: String,
    subject: PersonRef
) -> Result<Capsule, Error> {
    // Validate base capsule exists
    // Create capsule with inheritance reference
    // ... implementation
}
```

### Database Changes (if Option C)

- New `userCapsules` table
- Migration script for existing users
- Updated queries to handle capsule relationships

### Frontend Changes

- Updated API calls to handle capsule relationships
- New components for managing multiple capsules
- Error handling for capsule creation failures

## Next Steps

1. **Tech Lead Review**: Discuss approach and get approval
2. **Backend Team**: Confirm ICP backend capabilities
3. **Database Migration**: Plan migration strategy
4. **Frontend Implementation**: Update components and API calls
5. **Testing**: Comprehensive testing of all scenarios

## Risks and Mitigation

### Risk: Data Inconsistency

**Mitigation**: Implement proper validation and error handling

### Risk: Migration Complexity

**Mitigation**: Create comprehensive migration scripts and rollback plans

### Risk: Performance Impact

**Mitigation**: Add proper database indexes and query optimization

## Conclusion

The user-capsule relationship is critical for the success of the Web2/ICP integration. We need to choose an approach that balances simplicity with flexibility, ensuring we can handle all user scenarios while maintaining data consistency.

**Recommendation**: Implement the hybrid solution (Option A + Option C) to handle both self-capsules and multiple capsules per user effectively.

---

## Tech Lead Analysis & Final Recommendation

### **Tech Lead's Assessment of UUID Extension Pattern**

**What UUID-extension gives you:**

- ✅ Easy mental model: self capsule = userId, others = userId_1, userId_2
- ✅ No extra Neon table (if only querying "my capsules")
- ✅ Simple routing/URLs (can derive ownership from ID shape)

**Hidden costs & risks:**

- ❌ **Backend change required right now** - needs new APIs + validation + collision handling
- ❌ **Tight coupling & rigidity** - Neon's userId baked into ICP identifiers
- ❌ **Multiple owners/role changes** - ID becomes misleading after ownership changes
- ❌ **Enumerability & privacy** - Sequential suffixes are guessable
- ❌ **Non-self subjects** - Tying ID to owner's userId is semantically leaky
- ❌ **Inconsistent constraints** - Still need mapping for edge cases

### **Tech Lead's Recommendation: Option C (Mapping Table)**

**Why mapping table is preferred:**

- ✅ **Works today** (no canister change required)
- ✅ **Decouples systems** (IDs can evolve independently)
- ✅ **Handles all shapes** (self, deceased, org, transferred, multi-owner)
- ✅ **Clear invariants** (one self per user, dedupe, indexes, auditing)
- ✅ **Doesn't leak** internal userIds into public ICP IDs

**Practical recommendation:**

1. **MVP**: Implement `userCapsules` mapping table now
2. **Optional sugar later**: If canister adds `create_with_id()`, set self capsule id = userId _in addition_ to mapping
3. **Avoid `_N` suffixes**: Use opaque UUIDs for non-self to avoid enumeration

## Final Recommendation

**Option C: User-Capsule Mapping Table** (Tech Lead Approved)

This approach provides the best balance of safety, flexibility, and immediate implementation:

1. **Works with current backend** (no changes required)
2. **Decouples systems** (IDs can evolve independently)
3. **Handles all scenarios** (self, deceased, org, transferred, multi-owner)
4. **Clear invariants** (enforceable in database)
5. **Future-proof** (can add sugar later)

```typescript
// Mapping table approach
export const userCapsules = pgTable("user_capsules", {
  id: uuid("id").primaryKey().defaultRandom(),
  userId: text("user_id")
    .notNull()
    .references(() => users.id, { onDelete: "cascade" }),
  capsuleId: text("capsule_id").notNull(),
  capsuleType: text("capsule_type", {
    enum: ["self", "deceased", "organization", "other"],
  }).notNull(),
  isOwner: boolean("is_owner").default(true),
  isController: boolean("is_controller").default(false),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

// Usage examples
const userCapsules = [
  {
    userId: "user-uuid-123",
    capsuleId: "capsule_1703123456789", // Current backend-generated ID
    capsuleType: "self",
    isOwner: true,
  },
  {
    userId: "user-uuid-123",
    capsuleId: "capsule_1703123456790", // Different backend-generated ID
    capsuleType: "deceased",
    isOwner: true,
  },
];
```

**Future Enhancement (Optional):**

```typescript
// If backend adds create_with_id() support later
const selfCapsule = await actor.capsules_create_with_id(
  user.id, // Use user ID as capsule ID
  { Principal: user.principal }
);

// Keep mapping table as source of truth
await db.insert(userCapsules).values({
  userId: user.id,
  capsuleId: user.id, // Same as user ID
  capsuleType: "self",
  isOwner: true,
});
```

## Implementation Plan

### **Phase 1: Immediate (Option C)**

1. **Add `userCapsules` table** to schema
2. **Create migration script** for existing users
3. **Update frontend** to use mapping table
4. **Start building capsule management UI**

### **Phase 2: Future Enhancement (Optional)**

1. **Backend team implements** `create_with_id()` support
2. **Add self-capsule sugar** (user.id = capsule.id)
3. **Keep mapping table** as source of truth
4. **Optimize queries** using direct ID relationship

## Conclusion

The tech lead's analysis is spot-on. The UUID extension pattern, while elegant on paper, has significant hidden costs and risks. The mapping table approach provides:

- **Immediate implementation** (no backend changes)
- **System decoupling** (flexible architecture)
- **Future flexibility** (can add sugar later)
- **Clear invariants** (database-enforced constraints)

**Final Decision: Implement Option C (User-Capsule Mapping Table) for MVP, with optional enhancement to Option A (self-capsule = user.id) later if backend supports it.**

## Tech Lead's Complete Implementation Guide

### **1) Neon Schema (Drizzle)**

```typescript
import { pgTable, text, boolean, timestamp, uuid, index, uniqueIndex, sql } from "drizzle-orm/pg-core";
import { users } from "./users";

export const userCapsules = pgTable(
  "user_capsules",
  {
    id: uuid("id").primaryKey().defaultRandom(),

    userId: text("user_id")
      .notNull()
      .references(() => users.id, { onDelete: "cascade" }),

    capsuleId: text("capsule_id").notNull(), // matches ICP Capsule.id (String)

    capsuleType: text("capsule_type", {
      enum: ["self", "deceased", "organization", "other"],
    }).notNull(),

    isOwner: boolean("is_owner").notNull().default(true),
    isController: boolean("is_controller").notNull().default(false),

    createdAt: timestamp("created_at").notNull().defaultNow(),
  },
  (t) => [
    // Fast by-user lookups
    index("uc_user_idx").on(t.userId),

    // One mapping per (user, capsule)
    uniqueIndex("uc_user_capsule_unique").on(t.userId, t.capsuleId),

    // At most one self per user (partial unique)
    uniqueIndex("uc_user_self_unique")
      .on(t.userId, t.capsuleType)
      .where(sql`capsule_type = 'self'`),
  ]
);
```

**Optional Sugar (not Source of Truth):**

```typescript
// users table (optional sugar, not SoT)
selfCapsuleId: text("self_capsule_id").unique().nullable();
```

### **2) Service: ensureSelfCapsule**

```typescript
// server/service/capsules.ts
import { db } from "@/db/db";
import { userCapsules } from "@/db/schema";
import { eq, and } from "drizzle-orm";
import { auth } from "@/nextjs/auth"; // Auth.js server helper
import { createActor } from "@/ic/actor"; // your canister client

type EnsureSelfCapsuleResult = { capsuleId: string };

export async function ensureSelfCapsule(): Promise<EnsureSelfCapsuleResult> {
  const session = await auth();
  if (!session?.user?.id) throw new Error("Not authenticated");

  const userId = session.user.id;
  const principal = session.user.icpPrincipal || session.user.linkedIcPrincipal;
  if (!principal) {
    throw new Error("Internet Identity required to create a capsule");
  }

  // 1) Check existing mapping
  const existing = await db.query.userCapsules.findFirst({
    where: (uc, { and, eq }) => and(eq(uc.userId, userId), eq(uc.capsuleType, "self")),
    columns: { capsuleId: true },
  });
  if (existing?.capsuleId) return { capsuleId: existing.capsuleId };

  // 2) Create capsule in ICP (current API: canister generates ID)
  const actor = await createActor({ principal }); // identity-aware if needed
  const subject = { Principal: principal }; // PersonRef::Principal
  const created = await actor.capsules_create([subject]); // subject: Option<PersonRef>
  // adjust call to your real candid signature (Ok/Err handling omitted for brevity)

  const capsuleId: string = created.id; // from canister response

  // 3) Insert mapping (idempotent upsert)
  await db
    .insert(userCapsules)
    .values({
      userId,
      capsuleId,
      capsuleType: "self",
      isOwner: true,
      isController: false,
    })
    .onConflictDoNothing();

  // 4) (Optional sugar) backfill users.selfCapsuleId if you added it
  // await db.update(users).set({ selfCapsuleId: capsuleId }).where(eq(users.id, userId));

  return { capsuleId };
}
```

### **3) List User's Capsules**

```typescript
export async function listUserCapsules(userId: string) {
  // Neon is SoT for "which capsules belong to this user"
  const rows = await db.query.userCapsules.findMany({
    where: (uc, { eq }) => eq(uc.userId, userId),
    columns: { capsuleId: true, capsuleType: true, isOwner: true, isController: true },
  });

  // If you need capsule metadata, fetch in parallel from ICP by id
  // return await Promise.all(rows.map(r => actor.capsules_read(r.capsuleId)));

  return rows;
}
```

### **4) Unlink / Delete Mapping**

```typescript
export async function unlinkCapsule(userId: string, capsuleId: string) {
  await db.delete(userCapsules).where(and(eq(userCapsules.userId, userId), eq(userCapsules.capsuleId, capsuleId)));
}
```

### **5) Future Enhancement: "Self = userId"**

When backend adds `capsules_create_with_id()` support:

```typescript
// In ensureSelfCapsule, replace the create call with:
const id = userId; // use users.id
const created = await actor.capsules_create_with_id(id, [subject]);
const capsuleId = created.id; // == userId

// Keep the mapping table (covers non-self capsules, transfers, imports)
// Optionally set users.selfCapsuleId = userId for convenience
```

### **6) Optional: ACL Sync**

```typescript
// Periodic sync to mirror canister ACLs in Neon (for UI hints)
// list capsules by owner/controller in ICP (when APIs exist)
// upsert isOwner / isController flags in userCapsules
// Neon remains a cache; the canister is the authority for permissions
```

## Benefits of This Implementation

✅ **Works today** (no canister changes required)  
✅ **Clean invariants** (one self per user)  
✅ **Flexibility** for multi-capsule use cases  
✅ **Straightforward upgrade path** to "self = userId" without rework  
✅ **Production-ready** with proper error handling and constraints

---

**Document prepared by:** Backend Team  
**Date:** [Current Date]  
**Status:** ✅ **Tech Lead Approved - Complete Implementation Guide Provided**
