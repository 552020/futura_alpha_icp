# Pre-compute Dashboard Fields in Memory Creation Flow

**Priority**: High  
**Type**: Backend Enhancement  
**Status**: Open  
**Created**: 2025-01-16  
**Related**: Dashboard memory display, ICP memory integration, Database switching

## 📋 **Issue Summary**

The current ICP memory creation flow doesn't pre-compute dashboard-specific fields, which means `memories_list` queries would be expensive (costing cycles) if we add computed fields like sharing status, thumbnail URLs, and asset counts. We need to modify the memory creation flow to pre-compute and store these fields, making dashboard queries free and fast.

## 🎯 **Problem Statement**

### **Current State**

- `MemoryHeader` contains only basic fields (id, name, type, size, timestamps)
- Dashboard needs computed fields (sharing status, thumbnail URLs, asset counts)
- Computing these fields on every `memories_list` call would cost cycles
- Dashboard switching requires rich memory data for proper display

### **Required Solution**

- Pre-compute dashboard fields during memory creation/update
- Store computed values in `MemoryMetadata`
- Make `memories_list` queries free (no cycle costs)
- Enable efficient dashboard memory display

## 🔍 **Current Memory Creation Flow**

### **Files Involved**

- `src/backend/src/memories/core/create.rs` - Core memory creation logic
- `src/backend/src/memories/types.rs` - Memory data structures
- `src/backend/src/lib.rs` - Public API endpoints

### **Current MemoryMetadata Structure**

```rust
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct MemoryMetadata {
    // Basic info
    pub memory_type: MemoryType,
    pub title: Option<String>,
    pub description: Option<String>,
    pub content_type: String,

    // Timestamps
    pub created_at: u64,
    pub updated_at: u64,
    pub uploaded_at: u64,
    pub date_of_memory: Option<u64>,
    pub file_created_at: Option<u64>,

    // Organization
    pub parent_folder_id: Option<String>,
    pub tags: Vec<String>,
    pub deleted_at: Option<u64>,

    // Content info
    pub people_in_memory: Option<Vec<String>>,
    pub location: Option<String>,
    pub memory_notes: Option<String>,

    // System info
    pub created_by: Option<String>,
    pub database_storage_edges: Vec<StorageEdgeDatabaseType>,
}
```

## 🚀 **Required Changes**

### **1. Extend MemoryMetadata with Dashboard Fields**

```rust
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct MemoryMetadata {
    // ... existing fields ...

    // NEW: Pre-computed dashboard fields
    pub is_public: bool,                    // Computed from access rules
    pub shared_count: u32,                  // Count of shared recipients
    pub sharing_status: String,             // "public" | "shared" | "private"
    pub total_size: u64,                    // Sum of all asset sizes
    pub asset_count: u32,                   // Total number of assets
    pub thumbnail_url: Option<String>,      // Pre-computed thumbnail URL
    pub primary_asset_url: Option<String>,  // Primary asset URL for display
    pub has_thumbnails: bool,               // Whether thumbnails exist
    pub has_previews: bool,                 // Whether previews exist
}
```

### **2. Create Dashboard Field Computation Functions**

```rust
// In src/backend/src/memories/types.rs
impl Memory {
    /// Compute and update dashboard fields in metadata
    pub fn update_dashboard_fields(&mut self) {
        self.metadata.is_public = self.compute_is_public();
        self.metadata.shared_count = self.count_shared_recipients();
        self.metadata.sharing_status = self.compute_sharing_status();
        self.metadata.total_size = self.calculate_total_size();
        self.metadata.asset_count = self.count_assets();
        self.metadata.thumbnail_url = self.generate_thumbnail_url();
        self.metadata.primary_asset_url = self.generate_primary_asset_url();
        self.metadata.has_thumbnails = self.has_thumbnails();
        self.metadata.has_previews = self.has_previews();
    }

    /// Check if memory is public based on access rules
    fn compute_is_public(&self) -> bool {
        match &self.access {
            MemoryAccess::Public => true,
            MemoryAccess::Private => false,
            MemoryAccess::Shared { recipients, .. } => recipients.is_empty(),
            MemoryAccess::Temporal { access, .. } => match access {
                MemoryAccess::Public => true,
                _ => false,
            },
        }
    }

    /// Count number of shared recipients
    fn count_shared_recipients(&self) -> u32 {
        match &self.access {
            MemoryAccess::Shared { recipients, .. } => recipients.len() as u32,
            _ => 0,
        }
    }

    /// Compute sharing status string
    fn compute_sharing_status(&self) -> String {
        match &self.access {
            MemoryAccess::Public => "public".to_string(),
            MemoryAccess::Private => "private".to_string(),
            MemoryAccess::Shared { recipients, .. } => {
                if recipients.is_empty() {
                    "public".to_string()
                } else {
                    "shared".to_string()
                }
            },
            MemoryAccess::Temporal { .. } => "private".to_string(),
        }
    }

    /// Calculate total size of all assets
    fn calculate_total_size(&self) -> u64 {
        let mut total = 0u64;

        // Add inline assets
        for asset in &self.inline_assets {
            total += asset.bytes.len() as u64;
        }

        // Add blob internal assets
        for asset in &self.blob_internal_assets {
            total += asset.blob_ref.len;
        }

        // Add blob external assets
        for asset in &self.blob_external_assets {
            total += asset.metadata.bytes;
        }

        total
    }

    /// Count total number of assets
    fn count_assets(&self) -> u32 {
        (self.inline_assets.len() +
         self.blob_internal_assets.len() +
         self.blob_external_assets.len()) as u32
    }

    /// Generate thumbnail URL if available
    fn generate_thumbnail_url(&self) -> Option<String> {
        // Look for thumbnail in blob internal assets
        for asset in &self.blob_internal_assets {
            if matches!(asset.metadata.asset_type, AssetType::Thumbnail) {
                return Some(format!("icp://memory/{}/blob/{}", self.id, asset.asset_id));
            }
        }

        // Look for thumbnail in inline assets
        for asset in &self.inline_assets {
            if matches!(asset.metadata.asset_type, AssetType::Thumbnail) {
                return Some(format!("icp://memory/{}/inline/{}", self.id, asset.asset_id));
            }
        }

        None
    }

    /// Generate primary asset URL for display
    fn generate_primary_asset_url(&self) -> Option<String> {
        // Look for original asset in blob internal assets
        for asset in &self.blob_internal_assets {
            if matches!(asset.metadata.asset_type, AssetType::Original) {
                return Some(format!("icp://memory/{}/blob/{}", self.id, asset.asset_id));
            }
        }

        // Look for original asset in inline assets
        for asset in &self.inline_assets {
            if matches!(asset.metadata.asset_type, AssetType::Original) {
                return Some(format!("icp://memory/{}/inline/{}", self.id, asset.asset_id));
            }
        }

        None
    }

    /// Check if memory has thumbnails
    fn has_thumbnails(&self) -> bool {
        self.blob_internal_assets.iter().any(|asset|
            matches!(asset.metadata.asset_type, AssetType::Thumbnail)
        ) || self.inline_assets.iter().any(|asset|
            matches!(asset.metadata.asset_type, AssetType::Thumbnail)
        )
    }

    /// Check if memory has previews
    fn has_previews(&self) -> bool {
        self.blob_internal_assets.iter().any(|asset|
            matches!(asset.metadata.asset_type, AssetType::Preview)
        ) || self.inline_assets.iter().any(|asset|
            matches!(asset.metadata.asset_type, AssetType::Preview)
        )
    }
}
```

### **3. Update Memory Creation Flow**

```rust
// In src/backend/src/memories/core/create.rs
pub fn memories_create_core(
    env: &impl CanisterEnv,
    store: &mut impl StoreAdapter,
    capsule_id: types::CapsuleId,
    bytes: Option<Vec<u8>>,
    blob_ref: Option<types::BlobRef>,
    external_location: Option<types::StorageEdgeBlobType>,
    external_storage_key: Option<String>,
    external_url: Option<String>,
    external_size: Option<u64>,
    external_hash: Option<Vec<u8>>,
    asset_metadata: types::AssetMetadata,
    idem: String,
) -> Result<String, String> {
    // ... existing creation logic ...

    // Create memory with basic metadata
    let mut memory = Memory {
        id: memory_id.clone(),
        metadata: memory_metadata,
        access: access_control,
        inline_assets: vec![],
        blob_internal_assets: vec![],
        blob_external_assets: vec![],
    };

    // Add assets based on storage type
    // ... existing asset addition logic ...

    // NEW: Compute and store dashboard fields
    memory.update_dashboard_fields();

    // Store the memory
    store.store_memory(&memory)?;

    Ok(memory_id)
}
```

### **4. Update Memory Update Flow**

```rust
// In src/backend/src/memories/core/update.rs
pub fn memories_update_core(
    env: &impl CanisterEnv,
    store: &mut impl StoreAdapter,
    memory_id: String,
    update_data: MemoryUpdateData,
) -> Result<(), String> {
    // ... existing update logic ...

    // Update memory fields
    if let Some(new_metadata) = update_data.metadata {
        memory.metadata = new_metadata;
    }

    if let Some(new_access) = update_data.access {
        memory.access = new_access;
    }

    // NEW: Recompute dashboard fields after update
    memory.update_dashboard_fields();

    // Store updated memory
    store.store_memory(&memory)?;

    Ok(())
}
```

### **5. Update MemoryHeader Generation**

```rust
// In src/backend/src/memories/types.rs
impl Memory {
    /// Convert to MemoryHeader with pre-computed dashboard fields
    pub fn to_dashboard_header(&self) -> MemoryHeader {
        MemoryHeader {
            // Existing fields
            id: self.id.clone(),
            name: self.metadata.title.clone().unwrap_or_else(|| "Untitled".to_string()),
            memory_type: self.metadata.memory_type.clone(),
            size: self.metadata.total_size, // Use pre-computed value
            created_at: self.metadata.created_at,
            updated_at: self.metadata.updated_at,
            access: self.access.clone(),

            // NEW: Dashboard-specific fields (pre-computed)
            title: self.metadata.title.clone(),
            description: self.metadata.description.clone(),
            parent_folder_id: self.metadata.parent_folder_id.clone(),
            tags: self.metadata.tags.clone(),
            is_public: self.metadata.is_public,
            shared_count: self.metadata.shared_count,
            sharing_status: self.metadata.sharing_status.clone(),
            asset_count: self.metadata.asset_count,
            thumbnail_url: self.metadata.thumbnail_url.clone(),
            primary_asset_url: self.metadata.primary_asset_url.clone(),
            has_thumbnails: self.metadata.has_thumbnails,
            has_previews: self.metadata.has_previews,
        }
    }
}
```

## 📊 **Impact Analysis**

### **Performance Benefits**

- ✅ **Free queries** - `memories_list` stays a query call (no cycle costs)
- ✅ **Fast dashboard loading** - Pre-computed values ready immediately
- ✅ **Consistent performance** - No surprise costs for users
- ✅ **Efficient pagination** - No computation overhead per page

### **Cost Implications**

- ✅ **One-time cost** - Computation happens only during creation/update
- ✅ **Predictable costs** - Users know when they'll pay (memory operations)
- ✅ **No query costs** - Dashboard browsing is free

### **Storage Impact**

- ⚠️ **Slightly larger memory objects** - Additional fields in metadata
- ✅ **Negligible overhead** - Dashboard fields are small (strings, booleans, numbers)

## 🧪 **Testing Strategy**

### **Unit Tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_fields_computation() {
        let mut memory = create_test_memory();
        memory.update_dashboard_fields();

        assert!(memory.metadata.is_public);
        assert_eq!(memory.metadata.shared_count, 0);
        assert_eq!(memory.metadata.sharing_status, "public");
        assert!(memory.metadata.total_size > 0);
        assert!(memory.metadata.asset_count > 0);
    }

    #[test]
    fn test_memory_header_generation() {
        let memory = create_test_memory_with_dashboard_fields();
        let header = memory.to_dashboard_header();

        assert_eq!(header.is_public, memory.metadata.is_public);
        assert_eq!(header.shared_count, memory.metadata.shared_count);
        assert_eq!(header.sharing_status, memory.metadata.sharing_status);
    }
}
```

### **Integration Tests**

- Test memory creation with dashboard field computation
- Test memory update with dashboard field recomputation
- Test `memories_list` returns pre-computed values
- Test dashboard field consistency across operations

## 📋 **Implementation Checklist**

### **Phase 1: Data Structure Updates (High Priority)**

- [x] Extend `MemoryMetadata` with dashboard fields
- [x] Add dashboard field computation functions
- [x] Update `MemoryHeader` to include dashboard fields
- [x] Regenerate Candid types (auto-generated after Rust changes)

### **Phase 2: Memory Creation Flow (High Priority)**

- [x] Update `memories_create_core` to compute dashboard fields
- [x] Update `memories_create_with_internal_blobs` to compute dashboard fields
- [x] Test memory creation with new fields
- [x] Verify dashboard fields are stored correctly

### **Phase 3: Memory Update Flow (Medium Priority)**

- [ ] Update `memories_update_core` to recompute dashboard fields
- [ ] Test memory updates with field recomputation
- [ ] Verify dashboard fields are updated correctly

### **Phase 4: API Integration (Medium Priority)**

- [ ] Update `memories_list` to use pre-computed values
- [ ] Test `memories_list` performance
- [ ] Verify no cycle costs for queries

### **Phase 5: Testing & Validation (High Priority)**

- [ ] Write unit tests for dashboard field computation
- [ ] Write integration tests for memory creation/update
- [ ] Test with real memory data
- [ ] Validate performance improvements

## 🎯 **Success Criteria**

### **Functional Requirements**

- [ ] Dashboard fields are computed during memory creation
- [ ] Dashboard fields are recomputed during memory updates
- [ ] `memories_list` returns pre-computed dashboard values
- [ ] No computation overhead in query calls

### **Performance Requirements**

- [ ] Memory creation time increases by < 10%
- [ ] `memories_list` queries remain free (no cycle costs)
- [ ] Dashboard field computation completes in < 100ms
- [ ] Memory storage size increases by < 5%

### **Compatibility Requirements**

- [ ] Existing memory creation flow remains functional
- [ ] Existing memory update flow remains functional
- [ ] Existing `memories_list` API remains compatible
- [ ] No breaking changes to existing data structures

---

**Last Updated**: 2025-01-16  
**Status**: Open - Ready for Implementation  
**Priority**: High - Prerequisite for dashboard switching functionality
