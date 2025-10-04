# Capsule Backend: Add Missing Information

## Problem Statement

The current `Capsule` type in the backend is missing several important fields needed for the frontend capsule list component:

1. **Storage tracking** - How much storage a capsule is using
2. **Lifetime/expiration** - When a capsule expires
3. **Canister type** - Whether a capsule is independent or shared

## Current Backend Types

See `docs/backend-api-documentation.md` for current `Capsule` type definition.

## Missing Fields

### Storage Tracking

```rust
pub struct Capsule {
    // ... existing fields ...

    // Storage information
    pub storage_used: u64,        // Current storage usage in bytes
    pub storage_limit: u64,       // Maximum storage allowed in bytes
}
```

### Lifetime/Expiration

```rust
pub struct Capsule {
    // ... existing fields ...

    // Lifetime information
    pub expires_at: u64,          // Expiration timestamp (nanoseconds)
}
```

### Canister Type

```rust
pub enum CanisterType {
    Independent,  // Capsule has its own canister
    Shared,       // Capsule is stored in a shared canister
}

pub struct Capsule {
    // ... existing fields ...

    // Canister information
    pub canister_type: CanisterType,
}
```

## Implementation Plan

### Phase 1: Add Storage Tracking

- [ ] **1.1** Add `storage_used` and `storage_limit` fields to `Capsule` struct
- [ ] **1.2** Update capsule creation to set initial storage values
- [ ] **1.3** Add storage tracking logic for memory/gallery operations
- [ ] **1.4** Update capsule update functions to track storage changes

### Phase 2: Add Lifetime/Expiration

- [ ] **2.1** Add `expires_at` field to `Capsule` struct
- [ ] **2.2** Add expiration logic to capsule creation
- [ ] **2.3** Add expiration checking functions
- [ ] **2.4** Add capsule renewal functionality

### Phase 3: Add Canister Type

- [ ] **3.1** Add `CanisterType` enum
- [ ] **3.2** Add `canister_type` field to `Capsule` struct
- [ ] **3.3** Update capsule creation to set canister type
- [ ] **3.4** Add canister type management functions

## Backend Changes Required

### Files to Modify

- `src/backend/src/capsule.rs` - Add new fields to `Capsule` struct
- `src/backend/src/capsule.rs` - Add new enum `CanisterType`
- `src/backend/src/capsule.rs` - Update creation/update functions
- `src/backend/backend.did` - Update Candid interface

### New Functions Needed

```rust
// Storage management
pub fn update_capsule_storage(capsule_id: &str, storage_used: u64) -> Result<(), Error>
pub fn get_capsule_storage_info(capsule_id: &str) -> Result<(u64, u64), Error>

// Lifetime management
pub fn set_capsule_expiration(capsule_id: &str, expires_at: u64) -> Result<(), Error>
pub fn is_capsule_expired(capsule_id: &str) -> Result<bool, Error>
pub fn renew_capsule(capsule_id: &str, new_expires_at: u64) -> Result<(), Error>

// Canister type management
pub fn set_capsule_canister_type(capsule_id: &str, canister_type: CanisterType) -> Result<(), Error>
pub fn get_capsule_canister_type(capsule_id: &str) -> Result<CanisterType, Error>
```

## Frontend Impact

### Updated Types

```typescript
// Updated Capsule type will include:
interface Capsule {
  // ... existing fields ...
  storage_used: bigint;
  storage_limit: bigint;
  expires_at: bigint;
  canister_type: "independent" | "shared";
}
```

### Display Format

- **Space:** "2.5GB / 10GB" (used/total)
- **Lifetime:** "2029" (expiration year only)
- **Storage:** "Independent" or "Shared"

## Success Criteria

- [ ] All capsules have storage tracking
- [ ] All capsules have expiration dates
- [ ] All capsules have canister type information
- [ ] Frontend can display storage and lifetime information
- [ ] Backend functions work correctly
- [ ] No breaking changes to existing functionality

## Dependencies

- Frontend capsule list component needs these fields
- Storage tracking needs to be integrated with memory/gallery operations
- Expiration logic needs to be integrated with capsule lifecycle

## Future Enhancements

- **Storage quotas** - Different limits for different capsule types
- **Automatic renewal** - Auto-renew expired capsules
- **Storage optimization** - Compress old memories to save space
- **Canister migration** - Move capsules between independent and shared canisters
