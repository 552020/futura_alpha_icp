# Magic Link Index Implementation - Task 6

**Status**: `FUTURE` - Optional Implementation  
**Priority**: `MEDIUM` - Only if magic links are needed  
**Assigned**: Backend Developer  
**Created**: 2024-12-19  
**Related Issues**: [Capsule Access Refactoring - Phase 1 Implementation](../open/name-titile/capsule-access-refactoring.md)

## Overview

This document outlines Task 6 from the capsule access refactoring implementation, focusing on creating an optional minimal centralized index specifically for magic link lookups only.

**🎯 DECISION**: This is an **optional** implementation that should only be done if magic links are actually needed in the product.

## Why Optional?

The decentralized access control approach (Tasks 1-5) provides all the core functionality needed. Magic links are a specific feature that may not be required for the initial implementation.

**When to implement:**

- Product requires magic link functionality
- Need temporary access tokens for sharing
- Want to provide "share with anyone" links

**When to skip:**

- Initial implementation focuses on user-to-user sharing
- No need for temporary access tokens
- Simpler access control is sufficient

## Implementation Plan

### **File**: `src/backend/src/capsule/access.rs` (minimal)

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

## Key Features

### **Minimal Design**

- **Single purpose**: Only for magic link lookups
- **Small footprint**: One `StableBTreeMap` with minimal memory usage
- **Isolated**: Uses separate memory region to avoid conflicts

### **Magic Link Flow**

1. **Create link**: Generate hash, store in index with expiration
2. **Validate link**: Check hash exists and hasn't expired
3. **Grant access**: Use stored permission mask for access
4. **Cleanup**: Remove expired links periodically

### **Integration Points**

- **Time utilities**: Uses `crate::capsule::time::*` for expiration
- **Access control**: Integrates with `effective_perm_mask()` function
- **Resource types**: Works with `ResourceType` enum

## Implementation Steps

### **Step 1: Create Basic Structure**

- Create `src/backend/src/capsule/access.rs`
- Implement `LinkHash` and `LinkEntry` structs
- Add `Storable` implementations

### **Step 2: Implement MagicLinkIndex**

- Create `MagicLinkIndex` struct
- Implement `init()` method with `MemoryManager`
- Add basic CRUD operations

### **Step 3: Integration**

- Update `effective_perm_mask()` to check magic links
- Add magic link validation logic
- Integrate with time utilities

### **Step 4: Testing**

- Unit tests for magic link creation/validation
- Integration tests with access control
- Expiration testing

## Dependencies

- ✅ **Task 1-5 Complete**: Core access control system
- ✅ **Time utilities**: `crate::capsule::time::*` functions
- ✅ **Access control**: `AccessControlled` trait and evaluation logic

## Success Criteria

- [ ] Magic link index compiles successfully
- [ ] Can create and validate magic links
- [ ] Expiration logic works correctly
- [ ] Integrates with existing access control
- [ ] Memory usage is minimal
- [ ] No conflicts with existing stable memory

## Future Considerations

### **Performance**

- Consider periodic cleanup of expired links
- Monitor memory usage of the index
- Optimize hash generation if needed

### **Security**

- Ensure magic link tokens are cryptographically secure
- Consider rate limiting for magic link creation
- Implement proper token invalidation

### **User Experience**

- Provide clear expiration times to users
- Allow magic link regeneration
- Support different permission levels per link

## Related Documents

- [Capsule Access Refactoring - Phase 1 Implementation](../open/name-titile/capsule-access-refactoring.md) - Main implementation plan
- [Capsule Module Architecture](../../architecture/capsule-module-architecture.md) - Foundation module structure
- [Access Control Architecture Decision](../open/access-control-architecture-decision.md) - Centralized vs decentralized discussion

## Notes

This implementation is **optional** and should only be pursued if magic links are actually needed for the product. The core access control system (Tasks 1-5) provides all the essential functionality for user-to-user sharing and access control.

If magic links are not needed, this task can be skipped entirely, and the access control system will work perfectly without it.
