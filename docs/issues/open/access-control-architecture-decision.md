# Access Control Architecture Decision

## Issue

We need to decide between two approaches for access control storage:

1. **Decentralized**: Access stored directly on each resource
2. **Centralized**: Access stored in a separate AccessIndex system

## Current Implementation (Centralized)

We have implemented a complex centralized access system with:

- `ResKey` struct to identify resources
- `AccessIndex` with `StableBTreeMap` storage
- Complex `Storable` implementations
- 200+ lines of code in `access.rs`

## Alternative Approach (Decentralized)

Store access information directly on each resource:

```rust
pub struct Memory {
    pub id: String,
    pub title: String,
    pub access_entries: Vec<AccessEntry>,
    pub public_policy: Option<PublicPolicy>,
    // ... other fields
}

pub struct Gallery {
    pub id: String,
    pub title: String,
    pub access_entries: Vec<AccessEntry>,
    pub public_policy: Option<PublicPolicy>,
    // ... other fields
}
```

## Comparison

### Centralized Approach (Current)

**Pros:**

- Single source of truth for all access
- Easier to query "who has access to everything?"
- Consistent permission evaluation logic
- Easier bulk operations on access

**Cons:**

- Complex system with 200+ lines of code
- Requires `ResKey` struct and complex serialization
- Additional storage overhead
- More complex to understand and maintain

### Decentralized Approach (Alternative)

**Pros:**

- Simpler - access is where the resource is
- No complex indexing or `ResKey` system
- Direct access: `memory.access_entries`
- Less code and complexity
- Easier to understand

**Cons:**

- Access information scattered across resources
- Harder to query global access patterns
- Potential for inconsistent access logic
- More complex bulk operations

## Questions for Tech Lead

1. **Is the centralized approach worth the complexity?** We went from simple (access on each resource) to complex (centralized index) - was this the right decision?

2. **Do we actually need the `ResKey` system?** Objects are already uniquely identified within a capsule by their ID. Are we over-engineering this?

3. **What are the real benefits of centralized access?** Are there specific use cases that require the centralized approach?

4. **Should we simplify?** Could we achieve the same functionality with less complexity?

## Recommendation

Consider reverting to the decentralized approach unless there are compelling reasons for the centralized system. The current implementation seems over-engineered for the problem it's solving.

## Files Affected

- `src/backend/src/capsule/access.rs` (200+ lines)
- `src/backend/src/capsule/domain.rs` (access types)
- All resource structs (Memory, Gallery, etc.)

## Status

**Open** - Awaiting tech lead decision
