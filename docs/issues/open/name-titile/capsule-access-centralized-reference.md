# Capsule Access Control - Centralized Approach Reference

**Status**: `REFERENCE` - For Future Implementation  
**Purpose**: Documentation of the centralized access control system  
**Created**: 2024-12-19  
**Source**: `src/backend/src/capsule/access.rs` (moved and converted)

## Overview

This document serves as a reference for the centralized access control approach using the AccessIndex system. This approach was considered but not implemented in favor of the decentralized approach. It is preserved here for future reference if centralized access control becomes necessary.

## Key Features

- **Centralized AccessIndex**: Single source of truth for all access control
- **ResKey System**: Universal resource identification
- **StableBTreeMap Storage**: Persistent storage with proper memory management
- **Comprehensive Permission Evaluation**: Multi-layered access control logic

## Technical Implementation

### Imports and Dependencies

```rust
use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::time;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap, Storable,
};
use serde::Serialize;
use std::cell::RefCell;

use crate::capsule::domain::{
    AccessEntry, Capsule, Perm, PersonRef, PublicMode, PublicPolicy, ResourceType,
};
```

### Memory Management (Fixed)

```rust
// ============================================================================
// ACCESS INDEX SYSTEM - Tech Lead's Design
// ============================================================================

thread_local! {
    static MEM_MGR: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static ACCESS_INDEX: RefCell<AccessIndex> = RefCell::new(AccessIndex::init());
}
```

### Resource Key System

```rust

#[derive(
    Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize, CandidType,
)]
pub struct ResKey {
    pub r#type: ResourceType,
    pub id: String,
}

impl Storable for ResKey {
    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 1024,
            is_fixed_size: false,
        };

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        candid::encode_one(self).unwrap().into()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
}

impl Storable for AccessEntry {
    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 2048,
            is_fixed_size: false,
        };

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        candid::encode_one(self).unwrap().into()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
}

impl Storable for PublicPolicy {
    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 512,
            is_fixed_size: false,
        };

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        candid::encode_one(self).unwrap().into()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
}

// Custom wrapper for Vec<AccessEntry> to implement Storable
#[derive(Clone, Debug, PartialEq)]
pub struct AccessEntryList(pub Vec<AccessEntry>);

impl Storable for AccessEntryList {
    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 8192,
            is_fixed_size: false,
        };

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        candid::encode_one(&self.0).unwrap().into()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let entries: Vec<AccessEntry> = candid::decode_one(&bytes).unwrap();
        Self(entries)
    }
}

pub struct AccessIndex {
    pub entries: StableBTreeMap<ResKey, AccessEntryList, VirtualMemory<DefaultMemoryImpl>>,
    pub policy: StableBTreeMap<ResKey, PublicPolicy, VirtualMemory<DefaultMemoryImpl>>,
}

impl AccessIndex {
    pub fn init() -> Self {
        MEM_MGR.with(|mgr| {
            let mgr = mgr.borrow();
            let entries_mem = mgr.get(MemoryId::new(0));
            let policy_mem = mgr.get(MemoryId::new(1));

            Self {
                entries: StableBTreeMap::new(entries_mem),
                policy: StableBTreeMap::new(policy_mem),
            }
        })
    }
}

impl Default for AccessIndex {
    fn default() -> Self {
        Self::init()
    }
}

pub fn effective_perm_mask(
    key: &ResKey,
    ctx: &PrincipalContext,
    idx: &AccessIndex,
    capsule: &Capsule,
) -> u32 {
    use Perm as P;

    if is_owner(key, ctx, capsule) {
        return (P::VIEW | P::DOWNLOAD | P::SHARE | P::MANAGE | P::OWN).bits();
    }

    let mut m = 0u32;

    if let Some(v) = idx.entries.get(key) {
        m |= sum_user_and_groups(&v.0, ctx);
    }

    if let Some(token) = &ctx.link {
        m |= link_mask_if_valid(key, token, idx, ctx.now_ns);
    }

    m |= public_mask_if_any(key, idx, ctx);
    m
}

fn is_owner(key: &ResKey, ctx: &PrincipalContext, capsule: &Capsule) -> bool {
    match key.r#type {
        ResourceType::Memory => {
            if let Some(_memory) = capsule.memories.get(&key.id) {
                let person_ref = PersonRef::Principal(ctx.principal);
                capsule.is_owner(&person_ref) || capsule.is_controller(&person_ref)
            } else {
                false
            }
        }
        ResourceType::Gallery => {
            if let Some(_gallery) = capsule.galleries.get(&key.id) {
                let person_ref = PersonRef::Principal(ctx.principal);
                capsule.is_owner(&person_ref) || capsule.is_controller(&person_ref)
            } else {
                false
            }
        }
        ResourceType::Folder => {
            // TODO: Implement when folders are added to capsule
            false
        }
        ResourceType::Capsule => {
            let person_ref = PersonRef::Principal(ctx.principal);
            capsule.is_owner(&person_ref) || capsule.is_controller(&person_ref)
        }
    }
}

fn sum_user_and_groups(entries: &[AccessEntry], ctx: &PrincipalContext) -> u32 {
    let mut mask = 0u32;
    for entry in entries {
        if entry.person_ref == PersonRef::Principal(ctx.principal) {
            mask |= entry.perm_mask;
        }
        // TODO: Add group membership checks
    }
    mask
}

fn link_mask_if_valid(_key: &ResKey, _token: &str, _idx: &AccessIndex, _now_ns: u64) -> u32 {
    // TODO: Implement magic link validation
    // TODO: Hash token, look up in access index, check expiration
    // TODO: Return appropriate permission mask if valid
    0
}

fn public_mask_if_any(key: &ResKey, idx: &AccessIndex, ctx: &PrincipalContext) -> u32 {
    if let Some(policy) = idx.policy.get(key) {
        match policy.mode {
            PublicMode::Private => 0,
            PublicMode::PublicAuth => policy.perm_mask,
            PublicMode::PublicLink => {
                if ctx.link.is_some() {
                    policy.perm_mask
                } else {
                    0
                }
            }
        }
    } else {
        0
    }
}

pub struct PrincipalContext {
    pub principal: Principal,
    pub groups: Vec<String>,
    pub link: Option<String>,
    pub now_ns: u64,
}

impl PrincipalContext {
    pub fn new(principal: Principal, groups: Vec<String>, link: Option<String>) -> Self {
        Self {
            principal,
            groups,
            link,
            now_ns: time(),
        }
    }
}
```
