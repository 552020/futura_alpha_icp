# Domain Integration Analysis

**Date:** 2025-01-27  
**Purpose:** Analyze and plan integration of HTTP module with existing Futura domain logic

## 🎯 **Domain Integration Requirements**

### **8.1 ACL Implementation** - Connect to existing `effective_perm_mask()` logic

### **8.2 Asset Store** - Connect to existing `memories` and `blob_store` APIs

### **8.3 Permission Validation** - Integrate with existing user permission system

## 📋 **Current State Analysis**

### ✅ **What We Have (HTTP Module)**

**Current ACL Adapter:**

```rust
// src/backend/src/http/adapters/acl.rs
use candid::Principal;
use crate::http::core_types::Acl;

pub struct FuturaAclAdapter;

impl Acl for FuturaAclAdapter {
    fn can_view(&self, memory_id: &str, who: Principal) -> bool {
        // 🔄 TODO: Connect to existing domain logic
        true // Placeholder
    }
}
```

**Current Asset Store Adapter:**

```rust
// src/backend/src/http/adapters/asset_store.rs
use crate::http::core_types::{AssetStore, InlineAsset};

pub struct FuturaAssetStore;

impl AssetStore for FuturaAssetStore {
    fn get_inline(&self, memory_id: &str, asset_id: &str) -> Option<InlineAsset> {
        // 🔄 TODO: Connect to existing memories API
        None // Placeholder
    }

    fn get_blob_len(&self, memory_id: &str, asset_id: &str) -> Option<(u64, String)> {
        // 🔄 TODO: Connect to existing blob_store API
        None // Placeholder
    }

    fn read_blob_chunk(&self, memory_id: &str, asset_id: &str, offset: u64, len: u64) -> Option<Vec<u8>> {
        // 🔄 TODO: Connect to existing blob_store API
        None // Placeholder
    }
}
```

### ✅ **What We Need to Connect To (Existing Domain)**

**1. ACL System (`effective_perm_mask`):**

```rust
// From: src/backend/src/capsule/domain.rs
pub fn effective_perm_mask<T: AccessControlled>(
    resource: &T,
    ctx: &PrincipalContext,
) -> u32 {
    // Ownership fast-path
    if is_owner(resource, ctx) {
        return (P::VIEW | P::DOWNLOAD | P::SHARE | P::MANAGE | P::OWN).bits();
    }

    let mut m = 0u32;
    // Direct grants, magic links, public policy
    m |= sum_user_and_groups(resource.access_entries(), ctx);
    // ... more logic
    m
}

// From: docs/issues/done/capsule-access-centralized-reference.md
pub fn effective_perm_mask(
    key: &ResKey,
    ctx: &PrincipalContext,
    idx: &AccessIndex,
    capsule: &Capsule,
) -> u32 {
    // Implementation with AccessIndex
}
```

**2. Memory Asset Retrieval:**

```rust
// From: src/backend/src/memories/core/assets.rs
pub fn asset_get_by_id_core<E: Env, S: Store>(
    env: &E,
    store: &S,
    memory_id: String,
    asset_id: String,
) -> std::result::Result<crate::types::MemoryAssetData, Error> {
    // Find memory across accessible capsules
    let accessible_capsules = store.get_accessible_capsules(&env.caller());

    for capsule_id in accessible_capsules {
        if let Some(memory) = store.get_memory(&capsule_id, &memory_id) {
            // Try inline assets
            if let Some(asset) = memory.inline_assets.iter().find(|asset| asset.asset_id == asset_id) {
                return Ok(crate::types::MemoryAssetData::Inline {
                    bytes: asset.bytes.clone(),
                    content_type: asset.metadata.get_base().mime_type.clone(),
                    size: asset.bytes.len() as u64,
                    sha256: asset.metadata.get_base().sha256.map(|h| h.to_vec()),
                });
            }
            // Try blob internal assets
            // Try blob external assets
        }
    }
    Err(Error::NotFound)
}
```

**3. Blob Store Access:**

```rust
// From: src/backend/src/upload/blob_store.rs
pub struct BlobStore;

impl BlobStore {
    pub fn get_chunk(&self, blob_id: &str, chunk_idx: u32) -> Option<Vec<u8>> {
        // Access STABLE_BLOB_STORE
    }

    pub fn get_meta(&self, blob_id: &str) -> Option<BlobMeta> {
        // Access STABLE_BLOB_META
    }
}
```

## 🔧 **Integration Implementation Plan**

### **Phase 1: ACL Integration**

**Step 1.1: Update ACL Adapter**

```rust
// src/backend/src/http/adapters/acl.rs
use candid::Principal;
use crate::http::core_types::Acl;
use crate::capsule::domain::{effective_perm_mask, PrincipalContext};
use crate::capsule_store::Store;
use crate::memory::with_capsule_store;

pub struct FuturaAclAdapter;

impl Acl for FuturaAclAdapter {
    fn can_view(&self, memory_id: &str, who: Principal) -> bool {
        // Create PrincipalContext
        let ctx = PrincipalContext {
            principal: who,
            groups: vec![], // TODO: Get from user system
            link: None,     // TODO: Extract from HTTP request if needed
            now_ns: ic_cdk::api::time(),
        };

        // Find memory and check permissions
        with_capsule_store(|store| {
            let accessible_capsules = store.get_accessible_capsules(&crate::types::PersonRef::Principal(who));

            for capsule_id in accessible_capsules {
                if let Some(memory) = store.get_memory(&capsule_id, &memory_id.to_string()) {
                    // Use existing effective_perm_mask logic
                    let perm_mask = effective_perm_mask(&memory, &ctx);
                    return (perm_mask & crate::capsule::domain::Perm::VIEW.bits()) != 0;
                }
            }
            false
        })
    }
}
```

**Step 1.2: Handle Memory Access Control**

```rust
// Need to implement AccessControlled trait for Memory
impl crate::capsule::domain::AccessControlled for crate::types::Memory {
    fn access_entries(&self) -> &[crate::capsule::domain::AccessEntry] {
        &self.access_entries
    }
}
```

### **Phase 2: Asset Store Integration**

**Step 2.1: Update Asset Store Adapter**

```rust
// src/backend/src/http/adapters/asset_store.rs
use crate::http::core_types::{AssetStore, InlineAsset};
use crate::memories::core::asset_get_by_id_core;
use crate::memories::adapters::{CanisterEnv, CanisterStore};
use crate::types::MemoryAssetData;

pub struct FuturaAssetStore;

impl AssetStore for FuturaAssetStore {
    fn get_inline(&self, memory_id: &str, asset_id: &str) -> Option<InlineAsset> {
        let env = CanisterEnv;
        let store = CanisterStore;

        match asset_get_by_id_core(&env, &store, memory_id.to_string(), asset_id.to_string()) {
            Ok(MemoryAssetData::Inline { bytes, content_type, size, sha256 }) => {
                Some(InlineAsset {
                    bytes,
                    content_type,
                    size,
                    sha256,
                })
            }
            _ => None,
        }
    }

    fn get_blob_len(&self, memory_id: &str, asset_id: &str) -> Option<(u64, String)> {
        let env = CanisterEnv;
        let store = CanisterStore;

        match asset_get_by_id_core(&env, &store, memory_id.to_string(), asset_id.to_string()) {
            Ok(MemoryAssetData::InternalBlob { blob_id, size, sha256 }) => {
                Some((size, blob_id))
            }
            _ => None,
        }
    }

    fn read_blob_chunk(&self, memory_id: &str, asset_id: &str, offset: u64, len: u64) -> Option<Vec<u8>> {
        // Get blob_id first
        let (_, blob_id) = self.get_blob_len(memory_id, asset_id)?;

        // Calculate chunk index from offset
        let chunk_size = 1024 * 1024; // 1MB chunks (adjust as needed)
        let chunk_idx = (offset / chunk_size) as u32;

        // Access blob store
        crate::upload::blob_store::BlobStore::get_chunk(&blob_id, chunk_idx)
    }
}
```

### **Phase 3: Permission Validation Integration**

**Step 3.1: Update Token Minting with Full ACL**

```rust
// src/backend/src/lib.rs - mint_http_token function
#[query]
fn mint_http_token(memory_id: String, variants: Vec<String>, asset_ids: Option<Vec<String>>, ttl_secs: u32) -> String {
    use crate::http::{
        core_types::{TokenPayload, TokenScope},
        auth_core::{sign_token_core, encode_token_url},
        adapters::{canister_env::CanisterClock, secret_store::StableSecretStore, acl::FuturaAclAdapter},
    };

    let caller = ic_cdk::api::msg_caller();
    let acl = FuturaAclAdapter;

    // ✅ Enhanced: Use ACL adapter for authorization
    if !acl.can_view(&memory_id, caller) {
        ic_cdk::trap("forbidden: insufficient permissions");
    }

    // ✅ Enhanced: Validate asset_ids if provided
    if let Some(asset_ids) = &asset_ids {
        let asset_store = crate::http::adapters::asset_store::FuturaAssetStore;
        for asset_id in asset_ids {
            if asset_store.get_inline(&memory_id, asset_id).is_none()
                && asset_store.get_blob_len(&memory_id, asset_id).is_none() {
                ic_cdk::trap(&format!("asset not found: {}", asset_id));
            }
        }
    }

    // Create token payload
    let payload = TokenPayload {
        ver: 1,
        exp_ns: CanisterClock.now_ns() + (ttl_secs as u64 * 1_000_000_000),
        nonce: rand::rngs::StdRng::from_entropy().gen::<[u8; 12]>(),
        scope: TokenScope {
            memory_id,
            variants,
            asset_ids,
        },
        sub: Some(caller),
    };

    let token = sign_token_core(&StableSecretStore, &payload);
    encode_token_url(&token)
}
```

## 🚀 **Implementation Steps**

### **Step 1: ACL Integration (Priority: High)**

1. **Update ACL Adapter** - Connect to `effective_perm_mask`
2. **Implement AccessControlled** - Add trait implementation for Memory
3. **Test ACL Integration** - Verify permission checks work

### **Step 2: Asset Store Integration (Priority: High)**

1. **Update Asset Store Adapter** - Connect to `asset_get_by_id_core`
2. **Implement Blob Access** - Connect to blob store for chunked assets
3. **Test Asset Retrieval** - Verify inline and blob assets work

### **Step 3: Permission Validation (Priority: Medium)**

1. **Enhanced Token Minting** - Add asset validation
2. **Error Handling** - Proper error messages for missing assets
3. **Integration Testing** - End-to-end token minting and asset serving

## 📊 **Integration Complexity Assessment**

| Component             | Complexity | Dependencies                              | Status       |
| --------------------- | ---------- | ----------------------------------------- | ------------ |
| ACL Integration       | **Medium** | `effective_perm_mask`, `PrincipalContext` | 🔄 **Ready** |
| Asset Store           | **Medium** | `asset_get_by_id_core`, `BlobStore`       | 🔄 **Ready** |
| Permission Validation | **Low**    | ACL + Asset Store                         | 🔄 **Ready** |

## ✅ **Benefits of Integration**

1. **✅ Unified Permission System** - HTTP tokens respect existing ACL
2. **✅ Consistent Asset Access** - Same logic for HTTP and API access
3. **✅ Security Compliance** - All access goes through permission checks
4. **✅ Maintainability** - Single source of truth for permissions
5. **✅ Testability** - Can test HTTP module with existing test infrastructure

## 🎯 **Next Actions**

1. **Implement ACL Integration** - Connect to existing permission system
2. **Implement Asset Store Integration** - Connect to existing memory/blob APIs
3. **Test Integration** - Verify end-to-end functionality
4. **Document Integration** - Update API documentation

---

**Conclusion:** The domain integration is **straightforward** and leverages existing, well-tested domain logic. The HTTP module will seamlessly integrate with the existing Futura permission and asset systems! 🚀
