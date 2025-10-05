# 🏗️ Type Consistency Analysis: Core Entity Types

## **📋 Document Index**

### **🏗️ Architectural Considerations**

- [Entity Hierarchy & Relationships](#entity-hierarchy--relationships)
- [Access Control vs Lifecycle Distinction](#access-control-vs-lifecycle-distinction)
- [Resource Tracking Strategy](#resource-tracking-strategy)
- [Asset Lifecycle & Storage Types](#asset-lifecycle--storage-types)

### **❌ Type Inconsistencies Found**

- [1. Naming Inconsistencies](#1-naming-inconsistencies)
- [2. Metadata Structure Inconsistencies](#2-metadata-structure-inconsistencies)
- [3. Header/Info Subtype Analysis](#headerinfo-subtype-analysis)
- [4. Access Control Inconsistency](#4-access-control-inconsistency)

### **🚨 Critical Issues**

- [Asset Index Fragility](#critical-issue-asset-index-fragility)
- [Entity Reference Type Proliferation Problem](#entity-reference-type-proliferation-problem)

### **💡 Proposed Solutions**

- [Typed IDs Solution](#recommended-solution-typed-ids)
- [API Parameter Naming Strategy](#proposed-solution-api-parameter-naming-strategy)
- [Struct Field Naming Strategy](#proposed-solution-struct-field-naming-strategy)
- [Asset ID Implementation](#proposed-solution-asset-ids)

### **📋 Decision Points for Tech Lead**

- [API Parameter Naming](#open-issue-api-parameter-naming-strategy)
- [Sub-Type Field Naming](#open-issue-sub-type-field-naming-preference)
- [Asset ID Implementation](#decision-required-from-tech-lead)
- [Gallery Memory References](#gallery-solution-options)

---

## **Overview**

Analysis of our 4 main entity types (Capsules, Memories, Galleries, Assets, Person) to identify structural inconsistencies and propose a unified architecture for better maintainability and user experience.

## **🏗️ Architectural Considerations**

### **Entity Hierarchy & Lifecycle**

- **Capsules** = Primary entities with full lifecycle management
- **Memories** = Content metadata, lifecycle tied to capsule
- **Galleries** = Organization wrappers, lifecycle tied to capsule
- **Assets** = Content storage references, inherit from parent

### **Access Control vs Lifecycle Distinction**

- **Access Control**: Independent per entity (memories/galleries can have different access than capsule)
- **Lifecycle**: Always tied to capsule (memories/galleries expire with capsule)
- **Granular Permissions**: Owners/controllers can grant specific access to individual memories/galleries
- **No Independent Expiration**: Memories and galleries cannot outlive their parent capsule

### **Resource Tracking Strategy**

- **Capsule-level tracking sufficient** - for memories and galleries (lightweight database entries)
- **Blob storage redundancy** - through different payment systems (S3, Vercel, ICP, etc.)
- **Real storage cost** - in external blob storage, not in database entries
- **Simplified architecture** - capsule-level quotas cover all contained content

### **Asset Lifecycle & Storage Types**

- **Internal blobs** - Same lifecycle as capsule (capsule is a blob with interface)
- **External blobs** - Independent lifecycle (S3, Vercel, ICP external, etc.)
- **E2E encryption** - VetKeys provide privacy regardless of storage location
- **Resource tracking** - Capsule-level for internal, per-asset for external

## **Current Type Definitions**

### **1. Capsule Type**

```rust
pub struct Capsule {
    // Core Identity
    pub id: String,                                          // unique identifier
    pub subject: PersonRef,                                  // who this capsule is about

    // Ownership & Access Control
    pub owners: HashMap<PersonRef, OwnerState>,              // 1..n owners (usually 1)
    pub controllers: HashMap<PersonRef, ControllerState>,    // delegated admins (full control)

    // Social Graph
    pub connections: HashMap<PersonRef, Connection>,         // social graph
    pub connection_groups: HashMap<String, ConnectionGroup>, // organized connection groups

    // Content
    pub memories: HashMap<String, Memory>,                   // content
    pub galleries: HashMap<String, Gallery>,                // galleries (collections of memories)

    // Metadata
    pub created_at: u64,
    pub updated_at: u64,
    pub bound_to_neon: bool,    // Neon database binding status
    pub inline_bytes_used: u64, // Track inline storage consumption

    // NEW: Lifecycle and Resource Management
    pub expiration_date: Option<u64>,        // When this capsule expires (None = never expires)
    pub auto_renewal: bool,                  // Auto-renew before expiration
    pub grace_period_days: u32,             // Grace period after expiration
    pub allocated_storage_bytes: u64,       // Total storage quota allocated to this capsule
    pub used_storage_bytes: u64,            // Current storage usage
    pub allocated_cycles: u64,              // Cycles allocated for this capsule's operations
    pub consumed_cycles: u64,               // Cycles consumed by this capsule's operations
    pub storage_tier: StorageTier,          // Storage tier (Free, Basic, Premium, Enterprise)
    pub cycle_billing_enabled: bool,        // Whether to track cycles
    pub cycle_consumption_rate: f64,        // Cycles per operation
}
```

### **2. Memory Type**

```rust
pub struct Memory {
    // Core Identity
    pub id: String,                                         // unique identifier

    // Content & Assets
    pub inline_assets: Vec<MemoryAssetInline>,            // inline content
    pub blob_internal_assets: Vec<MemoryAssetBlobInternal>, // ICP blob assets
    pub blob_external_assets: Vec<MemoryAssetBlobExternal>, // external storage assets

    // Access Control
    pub access: MemoryAccess,                              // Private/Public access

    // Metadata
    pub metadata: MemoryMetadata,                         // title, description, tags, etc.
}
```

### **3. Gallery Type**

```rust
pub struct Gallery {
    // Core Identity
    pub id: String,                                        // unique identifier
    pub name: String,                                      // gallery name
    pub description: Option<String>,                       // gallery description

    // Content
    pub memories: Vec<String>,                             // memory IDs in this gallery

    // Access Control
    pub access: GalleryAccess,                             // Private/Public access

    // Metadata
    pub created_at: u64,
    pub updated_at: u64,
    pub created_by: Option<String>,                       // creator principal
}
```

### **4. Asset Types**

```rust
// Inline Asset
pub struct MemoryAssetInline {
    pub bytes: Vec<u8>,                                    // inline content
    pub metadata: AssetMetadata,                           // type-specific metadata
}

// Internal Blob Asset
pub struct MemoryAssetBlobInternal {
    pub blob_ref: BlobRef,                                // ICP blob reference
    pub metadata: AssetMetadata,                           // type-specific metadata
}

// External Blob Asset
pub struct MemoryAssetBlobExternal {
    pub location: StorageEdgeBlobType,                    // storage type (S3, Vercel, etc.)
    pub storage_key: String,                              // key in external storage
    pub url: Option<String>,                              // public URL
    pub metadata: AssetMetadata,                           // type-specific metadata
}
```

### **5. Person Type**

```rust
pub enum PersonRef {
    Principal(Principal),                                  // ICP principal
    Opaque(String),                                       // other identity
}
```

## **Structural Analysis**

### **✅ Consistent Patterns**

1. **Core Identity**: All types have `id: String`
2. **Access Control**: Most have access control mechanisms
3. **Metadata**: All have creation/update timestamps
4. **Content**: All can contain or reference content

### **❌ Type Inconsistencies Found**

#### **1. Naming Inconsistencies**

- **Capsule**: `id` vs `capsule_id` (in CapsuleInfo)
- **Memory**: `id` vs `memory_id` (in some contexts)
- **Gallery**: `id` vs `gallery_id` (in some contexts)
- **Assets**: `id` vs `asset_id` (in some contexts)

### **🔍 Technical Analysis: Exact Naming Inconsistencies**

#### **Capsule Naming Issues:**

```rust
// In Capsule struct
pub struct Capsule {
    pub id: String,  // ❌ Uses 'id'
    // ...
}

// In CapsuleInfo struct
pub struct CapsuleInfo {
    pub capsule_id: String,  // ❌ Uses 'capsule_id'
    // ...
}

// In CapsuleHeader struct
pub struct CapsuleHeader {
    pub id: String,  // ❌ Uses 'id' again
    // ...
}
```

#### **Memory Naming Issues:**

```rust
// In Memory struct
pub struct Memory {
    pub id: String,  // ❌ Uses 'id'
    // ...
}

// In MemoryHeader struct
pub struct MemoryHeader {
    pub id: String,  // ❌ Uses 'id'
    // ...
}

// In API calls
fn memories_read(memory_id: String)  // ❌ Parameter uses 'memory_id'
fn memories_delete(memory_id: String)  // ❌ Parameter uses 'memory_id'
```

#### **Gallery Naming Issues:**

```rust
// In Gallery struct
pub struct Gallery {
    pub id: String,  // ❌ Uses 'id'
    // ...
}

// In API calls (if they exist)
fn galleries_read(gallery_id: String)  // ❌ Parameter uses 'gallery_id'
```

#### **Asset Naming Issues:**

```rust
// In asset structs
pub struct MemoryAssetInline {
    // No explicit id field, but referenced by index
}

// In API calls
fn asset_remove_inline(memory_id: String, asset_index: u32)  // ❌ Uses 'asset_index'
fn asset_remove_internal(memory_id: String, asset_index: u32)  // ❌ Uses 'asset_index'
```

### **🚨 Critical Issue: Asset Index Fragility**

#### **The Problem with Index-Only Design:**

```rust
// Current fragile design
pub struct Memory {
    pub inline_assets: Vec<MemoryAssetInline>,     // [asset0, asset1, asset2]
    pub blob_internal_assets: Vec<MemoryAssetBlobInternal>, // [asset3, asset4]
    pub blob_external_assets: Vec<MemoryAssetBlobExternal], // [asset5, asset6]
}

// If we remove asset1 (index 1), everything shifts:
// Before: [asset0, asset1, asset2] -> indices 0,1,2
// After:  [asset0, asset2]         -> indices 0,1
// ❌ asset2 moved from index 2 to index 1!
```

#### **Real-World Failure Scenarios:**

1. **External References**: Gallery references asset at index 2 → becomes invalid when assets are removed
2. **Asset Sharing**: Memory A references asset in Memory B → breaks when Memory B removes assets
3. **Asset Metadata**: Track which assets are used where → references become invalid
4. **Array Reordering**: Any array modification breaks all subsequent index references

#### **The Risk:**

**"We lose the index, we lose the asset"** - This is a critical flaw in the current design!

#### **Important Clarification: Asset Index Scope**

```rust
// Asset indices are INTERNAL to each Memory, not global
pub struct Memory {
    pub id: String,
    pub inline_assets: Vec<MemoryAssetInline>,           // Memory A: [asset0, asset1, asset2]
    pub blob_internal_assets: Vec<MemoryAssetBlobInternal>, // Memory B: [asset0, asset1, asset2]
    pub blob_external_assets: Vec<MemoryAssetBlobExternal>, // Memory C: [asset0, asset1, asset2]
}

// Each memory has its own asset indices (0, 1, 2...)
// Index 2 in Memory A ≠ Index 2 in Memory B
```

**This makes the problem WORSE because:**

- ❌ **Memory-scoped fragility**: Each memory's asset indices are still fragile
- ❌ **External references**: Still break when assets are removed from any memory
- ❌ **Asset sharing**: Cannot reference assets across different memories
- ❌ **Cross-memory references**: Impossible to maintain stable references

### **💡 Proposed Solution: Asset IDs**

#### **Critical Decision Required:**

The current index-only design is too fragile for production. Asset IDs are needed for:

- External references to specific assets
- Asset sharing between memories
- Stable asset identity across array modifications

#### **Implementation Options:**

1. **Add Asset IDs**: Add unique IDs to all asset types
2. **Hybrid Approach**: Support both ID and index-based access
3. **Feature Flags**: Make asset IDs optional/configurable

#### **Key Considerations:**

- **Computational Cost**: Minimal (~1μs per asset, 36 bytes storage)
- **Migration Strategy**: Gradual rollout with backward compatibility
- **Performance Impact**: Negligible overhead for significant safety benefits

#### **Recommendation:**

**Add asset IDs immediately** - the current design is too fragile for a production system.

#### **API Parameter Inconsistencies:**

```rust
// Some functions use entity-specific names
fn capsules_read(capsule_id: String)  // ✅ Consistent
fn memories_read(memory_id: String)  // ❌ Inconsistent with struct field 'id'
fn galleries_read(gallery_id: String)  // ❌ Inconsistent with struct field 'id'

// Some functions use generic names
fn capsules_create(...)  // ✅ No ID parameter needed
fn memories_create(...)  // ✅ No ID parameter needed
```

### **💡 Proposed Solution: API Parameter Naming Strategy**

#### **Recommendation: Keep Specific Parameter Names (Even with Typed IDs)**

**Decision Point**: Keep specific parameter names even with typed IDs.

**Rationale**:

- Mixed parameter calls remain clear
- API is self-documenting
- Backward compatibility

**Key Benefits**:

- ✅ **Type Safety**: Typed IDs prevent parameter mixups
- ✅ **API Clarity**: Parameter names are self-documenting
- ✅ **Consistency**: All API functions follow same naming pattern
- ✅ **Backward Compatibility**: Minimal breaking changes
- ✅ **Mixed Parameters**: Clear when multiple IDs are needed

#### **Struct Field Inconsistencies:**

```rust
// Capsule types
Capsule.id           // ❌ Generic
CapsuleInfo.capsule_id  // ❌ Specific
CapsuleHeader.id     // ❌ Generic

// Memory types
Memory.id            // ❌ Generic
MemoryHeader.id      // ❌ Generic
// But API uses 'memory_id' parameter names
```

### **💡 Proposed Solution: Struct Field Naming Strategy**

#### **Recommendation: Foreign Key vs Self ID Distinction**

**Solution**: Semantic distinction:

- **`id`**: Object's own identifier
- **`{entity}_id`**: Foreign key referencing another entity

**Examples**:

```rust
pub struct Capsule {
    pub id: String,  // ✅ Self ID
}

pub struct CapsuleInfo {
    pub capsule_id: String,  // ✅ Foreign key
}
```

#### **The Gallery Problem - We Already Have an Issue:**

```rust
// ❌ Current problematic design
pub struct Gallery {
    pub id: String,  // ✅ Self ID
    pub memories: Vec<String>,  // ❌ These are memory IDs - should be memory_ids?
}
```

#### **Gallery Solution Options:**

1. **Explicit Foreign Key Names**: `memory_ids: Vec<String>`
2. **Structured References**: `memory_references: Vec<MemoryReference>`
3. **Keep Current**: Accept the inconsistency

#### **Key Benefits**:

- ✅ **Type Safety**: Typed IDs prevent field mixups
- ✅ **Consistency**: All structs use same field naming pattern
- ✅ **Self-Documenting**: Field names indicate relationship type
- ✅ **Database Consistency**: Matches foreign key patterns

### **🎯 Recommended Solution: Typed IDs**

#### **Problem with Current Approach:**

```rust
// Current ambiguous situation
pub struct CapsuleInfo {
    pub capsule_id: String,  // ❌ Is this the capsule's ID or the info's ID?
    // ...
}

pub struct Capsule {
    pub id: String,  // ❌ Generic 'id' - what entity is this?
    // ...
}
```

#### **Proposed Solution: Newtype Wrappers**

```rust
// Typed ID types for type safety
pub struct CapsuleId(String);
pub struct MemoryId(String);
pub struct GalleryId(String);
pub struct AssetId(String);
```

#### **Benefits of Typed IDs:**

- ✅ **Type Safety**: Prevents ID mixups at compile time
- ✅ **Self-Documenting**: Code intent is clear
- ✅ **API Clarity**: Function signatures are unambiguous
- ✅ **Refactoring Safety**: Changes caught at compile time

### **🤔 Decision Point: Sub-Type Field Naming**

#### **Option 1: Use Typed ID with Generic Field Name**

```rust
pub struct CapsuleInfo {
    pub id: CapsuleId,  // ✅ Typed, but generic field name
    pub memory_count: u32,
    pub gallery_count: u32,
}

pub struct CapsuleHeader {
    pub id: CapsuleId,  // ✅ Typed, but generic field name
    pub name: String,
    pub created_at: u64,
}
```

#### **Option 2: Use Typed ID with Specific Field Name**

```rust
pub struct CapsuleInfo {
    pub capsule_id: CapsuleId,  // ✅ Typed AND specific field name
    pub memory_count: u32,
    pub gallery_count: u32,
}

pub struct CapsuleHeader {
    pub capsule_id: CapsuleId,  // ✅ Typed AND specific field name
    pub name: String,
    pub created_at: u64,
}
```

#### **Trade-offs Analysis:**

**Option 1 (Generic `id`):**

- ✅ **Consistent**: All sub-types use `id` field
- ✅ **Shorter**: Less verbose field names
- ✅ **Type Safety**: Still prevents ID mixups
- ❌ **Ambiguity**: Field name doesn't indicate entity type

**Option 2 (Specific `{entity}_id`):**

- ✅ **Explicit**: Field name clearly indicates entity type
- ✅ **Self-Documenting**: Code is more readable
- ✅ **Type Safety**: Still prevents ID mixups
- ❌ **Verbose**: Longer field names
- ❌ **Inconsistent**: Different from main entity `id` field

#### **Recommendation:**

**Option 1 (Generic `id` with typed IDs)** is recommended because:

- The type system provides the safety and clarity
- Field names remain consistent across all sub-types
- Less verbose while maintaining type safety
- Aligns with main entity field naming

### **🤔 Open Issue: API Parameter Naming Strategy**

#### **The Problem:**

```rust
// Current API calls
fn memories_read(memory_id: String)  // ❌ Parameter uses 'memory_id'
fn memories_delete(memory_id: String)  // ❌ Parameter uses 'memory_id'

// Mixed parameter calls become confusing
fn memories_create(capsule_id: String, memory_id: String)  // ❌ Which is which?
```

#### **Option 1: Keep Specific Parameter Names**

```rust
// Even with typed IDs, keep specific parameter names
fn memories_read(memory_id: MemoryId)  // ✅ Clear parameter name
fn memories_delete(memory_id: MemoryId)  // ✅ Clear parameter name
fn memories_create(capsule_id: CapsuleId, memory_id: MemoryId)  // ✅ Clear which is which
```

#### **Option 2: Use Generic Parameter Names**

```rust
// Use generic parameter names with typed IDs
fn memories_read(id: MemoryId)  // ✅ Type makes it clear
fn memories_delete(id: MemoryId)  // ✅ Type makes it clear
fn memories_create(capsule_id: CapsuleId, id: MemoryId)  // ❌ Still confusing
```

#### **Trade-offs:**

- **Specific names**: More verbose but clearer in mixed parameter calls
- **Generic names**: Shorter but can be confusing with multiple parameters
- **Typed IDs**: Provide type safety regardless of parameter naming

### **🤔 Open Issue: Sub-Type Field Naming Preference**

#### **The Contention:**

```rust
// Option 1: Generic field name
pub struct CapsuleInfo {
    pub id: CapsuleId,  // ✅ Consistent with main entity
    // ...
}

// Option 2: Specific field name
pub struct CapsuleInfo {
    pub capsule_id: CapsuleId,  // ✅ Explicit about what ID this refers to
    // ...
}
```

#### **Arguments for Generic `id`:**

- ✅ **Consistency**: Matches main entity field naming
- ✅ **Type Safety**: The type system provides clarity
- ✅ **Less Verbose**: Shorter field names
- ✅ **Uniform**: All sub-types use same pattern

#### **Arguments for Specific `{entity}_id`:**

- ✅ **Explicit**: Field name clearly indicates what ID this refers to
- ✅ **Self-Documenting**: Code is more readable without type annotations
- ✅ **Clear Intent**: Obvious that this refers to a capsule ID
- ✅ **API Consistency**: Matches API parameter naming

#### **The Real Question:**

**Is the type system sufficient for clarity, or do we need explicit field names?**

### **📋 Decision Required from Tech Lead:**

1. **API Parameter Naming**: Should we use specific parameter names (`memory_id`) or generic names (`id`) with typed IDs?

2. **Sub-Type Field Naming**: Should sub-types use generic `id` field or specific `{entity}_id` field?

3. **Consistency Strategy**: Should we prioritize consistency across all types, or clarity in specific contexts?

4. **Type System vs Explicit Naming**: Is the type system sufficient for clarity, or do we need explicit naming for maximum readability?

#### **2. Metadata Structure Inconsistencies**

- **Capsule**: Metadata spread in main struct (created_at, updated_at, etc.)
- **Memory**: Dedicated `MemoryMetadata` struct
- **Gallery**: Basic metadata (name, description, timestamps)
- **Assets**: Type-specific metadata only

#### **3. Header/Info Subtype Inconsistencies**

- **CapsuleInfo**: Summary with counts and permissions
- **CapsuleHeader**: Lightweight for listing
- **MemoryHeader**: Summary with size and access
- **GalleryHeader**: Not defined (inconsistent)
- **AssetHeader**: Not defined (inconsistent)

### **📋 Header/Info Subtype Analysis**

#### **Current Header/Info Types:**

```rust
// Capsule subtypes
pub struct CapsuleInfo {
    pub capsule_id: String,
    pub subject: PersonRef,
    pub is_owner: bool,
    pub is_controller: bool,
    pub is_self_capsule: bool,
    pub bound_to_neon: bool,
    pub created_at: u64,
    pub updated_at: u64,
    pub memory_count: u64,
    pub gallery_count: u64,
    pub connection_count: u64,
}

pub struct CapsuleHeader {
    pub id: String,
    pub subject: PersonRef,
    pub owner_count: u64,
    pub controller_count: u64,
    pub memory_count: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

// Memory subtypes
pub struct MemoryHeader {
    pub id: String,
    pub name: String,
    pub memory_type: MemoryType,
    pub size: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub access: MemoryAccess,
}

// Gallery subtypes - MISSING
// Asset subtypes - MISSING
```

#### **🤔 The Core Question: Do We Need Both Info and Header Types?**

**Current Inconsistencies:**

1. **Naming**: `id` vs `capsule_id` vs `memory_id`
2. **Structure**: Different field sets for similar purposes
3. **Missing types**: No GalleryHeader or AssetHeader
4. **Field consistency**: Some have `created_at`, others don't
5. **Access control**: Some have access info, others don't

#### **📊 Pro/Con Analysis: Keep Both Info and Header Types**

##### **✅ Arguments FOR Keeping Both Types:**

**1. Different Use Cases:**

- **Info Types**: Detailed views, user-specific information, permissions
- **Header Types**: List views, lightweight operations, basic metadata

**2. Performance Optimization:**

- **Info Types**: More computation (counts, permissions, relationships)
- **Header Types**: Lightweight, minimal computation
- **Data Transfer**: Headers are smaller for list operations

**3. API Granularity:**

- **Info Endpoints**: `/capsules/info/{id}` - detailed view
- **Header Endpoints**: `/capsules/header/{id}` - list view
- **Different Clients**: Mobile vs desktop, different data needs

**4. Security Considerations:**

- **Info Types**: Include permission checks (`is_owner`, `is_controller`)
- **Header Types**: Basic metadata only, no sensitive permissions
- **Access Control**: Different security requirements

##### **❌ Arguments AGAINST Keeping Both Types:**

**1. Type Proliferation:**

- **More Types**: CapsuleInfo, CapsuleHeader, MemoryInfo, MemoryHeader, etc.
- **API Complexity**: More endpoints to maintain
- **Documentation**: More types to document and explain

**2. Inconsistency Risk:**

- **Naming Inconsistencies**: `id` vs `capsule_id` patterns
- **Field Inconsistencies**: Different field sets for similar purposes
- **Maintenance Burden**: Two types to keep in sync

**3. Developer Confusion:**

- **Which Type to Use**: When to use Info vs Header?
- **API Choice**: Multiple endpoints for similar data
- **Learning Curve**: More types to understand

**4. Maintenance Overhead:**

- **Code Duplication**: Similar logic in both types
- **Testing**: More types to test
- **Updates**: Changes need to be applied to both types

#### **🎯 Alternative Approaches:**

##### **Option 1: Consolidate into Single Type**

```rust
// Single lightweight type for all use cases
pub struct CapsuleSummary {
    pub id: String,
    pub subject: PersonRef,
    pub created_at: u64,
    pub updated_at: u64,
    // Optional computed fields
    pub memory_count: Option<u64>,
    pub is_owner: Option<bool>,
}
```

**Pros**: Simpler, consistent, less types
**Cons**: Less optimized, always includes optional fields

##### **Option 2: Standardize Both Types**

```rust
// Consistent naming and structure
pub struct CapsuleInfo {
    pub id: String,  // ✅ Consistent naming
    pub subject: PersonRef,
    pub is_owner: bool,
    pub memory_count: u64,
    // ... computed fields
}

pub struct CapsuleHeader {
    pub id: String,  // ✅ Consistent naming
    pub subject: PersonRef,
    pub memory_count: u64,
    // ... basic fields only
}
```

**Pros**: Consistent, optimized for different use cases
**Cons**: Still two types to maintain

##### **Option 3: Choose One Pattern**

```rust
// Either all Info or all Header pattern
pub struct CapsuleInfo { /* detailed */ }
pub struct MemoryInfo { /* detailed */ }
pub struct GalleryInfo { /* detailed */ }
// No Header types
```

**Pros**: Consistent pattern, single approach
**Cons**: Less optimized for different use cases

#### **📋 Missing Types Analysis:**

**Should we add missing types?**

**GalleryHeader:**

```rust
pub struct GalleryHeader {
    pub id: String,
    pub name: String,
    pub memory_count: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub access: GalleryAccess,
}
```

**MemoryInfo:**

```rust
pub struct MemoryInfo {
    pub id: String,
    pub name: String,
    pub memory_type: MemoryType,
    pub size: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub access: MemoryAccess,
    pub asset_count: u64,
    pub is_owner: bool,
}
```

**AssetHeader:**

```rust
pub struct AssetHeader {
    pub id: String,
    pub asset_type: AssetType,
    pub size: u64,
    pub created_at: u64,
    pub storage_type: StorageType,
}
```

#### **🎯 Recommendation:**

**Standardize Both Types** - They serve different use cases:

1. **Keep Info Types**: For detailed views with permissions and counts
2. **Keep Header Types**: For list views with basic metadata
3. **Add Missing Types**: GalleryHeader, MemoryInfo, AssetHeader
4. **Standardize Naming**: Consistent `id` vs `{entity}_id` patterns
5. **Consistent Structure**: Same field patterns across all types

**Key Benefits**:

- ✅ **Performance**: Optimized for different use cases
- ✅ **Consistency**: All entities follow same patterns
- ✅ **Completeness**: All entities have both Info and Header types
- ✅ **API Clarity**: Clear distinction between detailed and list views

#### **📋 Naming Convention Options for Tech Lead Decision:**

**Option 1: "View" Pattern (Industry Standard)**

```rust
pub struct CapsuleView { /* ... */ }
pub struct MemoryView { /* ... */ }
pub struct GalleryView { /* ... */ }
```

- ✅ **Industry Standard**: Used by GitHub, Stripe, AWS APIs
- ✅ **Clear Intent**: Represents a view of the data
- ✅ **Flexible**: Can be different views (list, detail, etc.)
- ✅ **Future-Proof**: Can evolve without breaking changes

**Option 2: "Info" Pattern (Current)**

```rust
pub struct CapsuleInfo { /* ... */ }
pub struct MemoryInfo { /* ... */ }
pub struct GalleryInfo { /* ... */ }
```

- ✅ **Current**: Already in codebase
- ✅ **Clear**: Information about the entity
- ✅ **Simple**: Easy to understand
- ❌ **Might imply**: Always detailed

**Option 3: "Header" Pattern (Current)**

```rust
pub struct CapsuleHeader { /* ... */ }
pub struct MemoryHeader { /* ... */ }
pub struct GalleryHeader { /* ... */ }
```

- ✅ **Clear**: Implies lightweight, list-oriented
- ✅ **Current**: Already in codebase
- ❌ **Less Common**: Not widely used in APIs
- ❌ **Might imply**: Always lightweight

**Option 4: "Details" Pattern**

```rust
pub struct CapsuleDetails { /* ... */ }
pub struct MemoryDetails { /* ... */ }
pub struct GalleryDetails { /* ... */ }
```

- ✅ **Clear**: Detailed information
- ✅ **Common**: Used in many APIs
- ❌ **Might imply**: Always detailed (not flexible)

**Option 5: Consolidate to Single Type**

```rust
// Single type per entity with optional computed fields
pub struct CapsuleSummary {
    pub id: String,
    pub subject: PersonRef,
    pub created_at: u64,
    pub updated_at: u64,
    // Optional computed fields
    pub memory_count: Option<u64>,
    pub is_owner: Option<bool>,
}
```

- ✅ **Simplest**: One type per entity
- ✅ **Flexible**: Include computed fields only when needed
- ✅ **Consistent**: Same pattern for all entities
- ✅ **Less Maintenance**: One type to keep in sync

**Decision Required**: Which naming convention should we adopt for the subtype pattern?

### **🤔 Entity Reference Type Proliferation Problem**

#### **Current Gallery Structure:**

```rust
pub struct Gallery {
    pub memories: Vec<String>,  // Just memory IDs
    // ...
}
```

#### **The Problem:**

- **Vec<String>** → Only IDs, need additional queries for memory info
- **Vec<Memory>** → Full memory struct, too much data
- **Vec<MemoryHeader>** → Lightweight memory info, but missing gallery-specific fields

#### **Proposed Solutions:**

##### **Option 1: MemoryRef (New Type)**

```rust
pub struct MemoryRef {
    pub id: String,
    pub added_at: u64,
    pub added_by: PersonRef,
    pub display_order: u32,
    pub access_level: AccessLevel,
    pub metadata: HashMap<String, String>,
}
```

##### **Option 2: MemoryHeader (Existing Type)**

```rust
pub struct MemoryHeader {
    pub id: String,
    pub name: String,
    pub memory_type: MemoryType,
    pub size: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub access: MemoryAccess,
}
```

##### **Option 3: Generic EntityRef (All Entities)**

```rust
pub struct EntityRef<T> {
    pub id: String,
    pub added_at: u64,
    pub added_by: PersonRef,
    pub display_order: u32,
    pub access_level: AccessLevel,
    pub metadata: HashMap<String, String>,
}

// Usage
pub struct Gallery {
    pub memories: Vec<EntityRef<Memory>>,
    pub galleries: Vec<EntityRef<Gallery>>,
}
```

##### **Option 4: Wrapper with Existing Types**

```rust
pub struct GalleryMemoryEntry {
    pub memory: MemoryHeader,           // Existing type
    pub added_at: u64,                   // Gallery-specific
    pub added_by: PersonRef,             // Gallery-specific
    pub display_order: u32,              // Gallery-specific
    pub gallery_notes: Option<String>,   // Gallery-specific
}
```

#### **Decision Points:**

1. **Type proliferation** - Do we want new types or reuse existing ones?
2. **Gallery-specific fields** - Do we need added_at, added_by, display_order?
3. **Generic vs specific** - Should we have EntityRef<T> or specific types?
4. **Data transfer** - How much data do we want to transfer for gallery views?

#### **Trade-offs:**

- **Vec<String>** → Simple, minimal data, but need additional queries
- **MemoryHeader** → Rich info, but missing gallery-specific fields
- **MemoryRef** → Gallery-specific, but new type proliferation
- **EntityRef<T>** → Generic, reusable, but complex
- **Wrapper approach** → Reuses existing types, but more complex structure

#### **4. Access Control Inconsistency**

- **Capsule**: Complex ownership (`owners`, `controllers`, `connections`)
- **Memory**: Simple access (`MemoryAccess` enum)
- **Gallery**: Simple access (`GalleryAccess` enum)
- **Assets**: No access control (inherited from parent)

#### **2. Metadata Inconsistency**

- **Capsule**: Rich metadata (timestamps, binding status, resource tracking)
- **Memory**: Rich metadata (`MemoryMetadata` struct)
- **Gallery**: Basic metadata (timestamps, creator)
- **Assets**: Type-specific metadata only

#### **3. Lifecycle Management Inconsistency**

- **Capsule**: Full lifecycle (expiration, renewal, grace periods)
- **Memory**: No lifecycle management
- **Gallery**: No lifecycle management
- **Assets**: No lifecycle management

#### **4. Resource Tracking Inconsistency**

- **Capsule**: Full resource tracking (storage, cycles, tiers)
- **Memory**: No resource tracking
- **Gallery**: No resource tracking
- **Assets**: No resource tracking

#### **🤔 Resource Tracking Architecture Decision: Separate Cycles from Storage?**

**Current Approach**: Combined tracking in single `ResourceTracking` struct

```rust
pub struct ResourceTracking {
    pub allocated_storage_bytes: u64,    // Storage quota
    pub used_storage_bytes: u64,        // Storage usage
    pub allocated_cycles: u64,          // Cycle quota
    pub consumed_cycles: u64,           // Cycle usage
    pub storage_tier: StorageTier,      // Storage tier
    pub cycle_billing_enabled: bool,    // Cycle billing flag
}
```

**Alternative Approach**: Separate tracking structs

```rust
pub struct StorageTracking {
    pub allocated_bytes: u64,
    pub used_bytes: u64,
    pub tier: StorageTier,
    pub last_accessed_at: u64,
}

pub struct CycleTracking {
    pub allocated_cycles: u64,
    pub consumed_cycles: u64,
    pub billing_enabled: bool,
    pub consumption_rate: f64,
}
```

**Arguments FOR Separation:**

- ✅ **Different Resource Types**: Storage is persistent, cycles are consumed
- ✅ **Different Ownership Models**:
  - **Storage**: Prepaid ownership (buy 100 years, you own it)
  - **Cycles**: Consumable resource (use it up, need to recharge)
- ✅ **Different Quota Enforcement**: Storage limits are hard, cycle limits are soft
- ✅ **Different Analytics**: Storage trends vs cycle consumption patterns
- ✅ **Independent Scaling**: Storage and compute can scale independently

**Arguments AGAINST Separation:**

- ❌ **Complexity**: Two tracking systems to maintain
- ❌ **API Complexity**: More endpoints and types
- ❌ **Current Working**: Single struct is simpler
- ❌ **Over-Engineering**: May be unnecessary complexity

**Key Questions:**

1. **Ownership Model**:
   - **Self-capsules**: Prepaid ownership (buy 100 years, you own it)
   - **Shared capsules**: Traditional billing (monthly subscriptions)
2. **Quota Enforcement**: Are storage and cycle limits independent?
3. **User Experience**: Do users need separate storage vs cycle dashboards?
4. **Analytics**: Do we need separate storage vs cycle analytics?

**Self-Capsule vs Shared-Capsule Distinction:**

- **Self-Capsules**: "Buy once, own forever" - prepaid ownership model
- **Shared-Capsules**: Traditional billing - monthly subscriptions, usage-based pricing
- **Resource Tracking**: Same technical tracking, different business models

**Recommendation**: **Separate them** - Storage and cycles are fundamentally different resource types with different ownership models (prepaid vs consumable) and enforcement needs.

---

## **🎯 TECH LEAD DECISION: MVP-FOCUSED APPROACH**

### **✅ What's Solid (Keep)**

- **Problem framing is right:** naming drift, asset-index fragility, and header/info drift are the real pain points
- **Asset IDs:** 100% agree. Index-only is brittle. Give every asset a stable `asset_id` (UUID). Keep index purely for **ordering**
- **API param names:** Prefer **specific names** (`capsule_id`, `memory_id`, `asset_id`) in function signatures. It's self-documenting and avoids mixed-ID confusion
- **Gallery references:** Add a wrapper (e.g., `GalleryMemoryEntry`) so we can carry `added_at/added_by/order` alongside the reference

### **⏸️ What to Trim for MVP**

- **Do not unify Access/Lifecycle/Resource into shared mega-structs** right now. Capsules can stay rich; Memories/Galleries keep the lighter access model. We can converge later if needed
- **Typed-ID newtypes everywhere:** great idea, but optional for MVP. Start with **string UUIDs + naming convention**; introduce `CapsuleId`/`MemoryId` newtypes later when churn stabilizes

### **🔧 Concrete Decisions (Tech Lead's Call)**

#### **1. IDs & Naming**

- **In structs:** **self id is `id`**. Foreign keys use `{entity}_id`
- **In APIs:** **parameters use specific names** (`capsule_id`, `memory_id`, `gallery_id`, `asset_id`)
- **In "*Info/*Header" types:** field is **`id`** (same as entity), not `capsule_id`. The type already disambiguates

#### **2. Headers vs Info**

- **Keep both**, but **standardize**:
  - `*Header`: lightweight list item; no computed permissions; always has `id`, `created_at`, `updated_at`, minimal summary fields
  - `*Info`: per-user, computed data (counts/flags like `is_owner`, `is_controller`)
- **Add the missing ones:** `GalleryHeader`, `MemoryInfo`, `AssetHeader`

#### **3. Assets**

- **Introduce `asset_id: string` on all asset variants**
- **Keep arrays; index = order only**. Reordering changes index, not identity
- \*\*Add APIs that accept either `asset_id` (preferred) or `(memory_id, index)` for backward compat; deprecate index-path later

#### **4. Galleries**

- **Replace `Vec<String>` with:**

```rust
pub struct GalleryMemoryEntry {
    pub memory_id: String,
    pub added_at: u64,
    pub added_by: PersonRef,
    pub display_order: u32,
    pub notes: Option<String>,
}

pub struct Gallery {
    pub id: String,
    pub capsule_id: String,
    pub entries: Vec<GalleryMemoryEntry>,
    // access, metadata…
}
```

#### **5. Resource Tracking**

- **Keep simple at capsule-level for MVP**. Defer "separate storage vs cycles" until we actually need distinct policies/UX

### **📋 Minimal Migration Plan**

#### **Phase A (No Breaking)**

- Add `asset_id` to assets; populate for existing items
- Add `GalleryMemoryEntry` and map existing `memory_ids` to entries (default `display_order = i`)
- Standardize API param names (server can accept both new/old; warn on old)

#### **Phase B**

- Add missing `*Header/*Info` types; adjust list/detail endpoints to return the right flavor
- Deprecate index-based asset mutation endpoints

### **🛡️ Guardrails/Checklist**

- **Lint schema/IDL:** forbid introducing `{entity}_id` for self ids in structs; require it for foreign keys
- **Docs:** one page with three tables—
  1. **Entity structs** (self ids & FKs)
  2. **Header vs Info fields** per entity
  3. **ID usage rules** (structs vs API params)
- **Tests:** reordering assets must preserve `asset_id`s; gallery entries keep `added_at/added_by` on reorder

### **❓ Answering Open Questions**

- **Typed IDs vs explicit names?** Start with **explicit names**; add typed IDs later (nice-to-have)
- **Sub-type field naming?** Use **`id`** (generic) in sub-types; the **type** disambiguates. Keep API params specific
- **Entity-ref proliferation?** Use **`GalleryMemoryEntry`** (specific wrapper) rather than a generic `EntityRef<T>`; simpler and clearer

### **🚀 TL;DR Action Items**

- ✅ **Add `asset_id`** (keep index for order)
- ✅ **Standardize:** structs use `id` for self, `{entity}_id` for FKs; APIs use specific param names
- ✅ **Keep `Header` (light) and `Info` (computed)** for all entities; add missing ones
- ✅ **Replace gallery `Vec<String>` with `Vec<GalleryMemoryEntry>`**
- ⏸️ **Defer mega unification** (Access/Lifecycle/Resource) and typed newtypes until after MVP

#### **5. Content Organization Inconsistency**

- **Capsule**: Contains memories and galleries
- **Memory**: Contains assets
- **Gallery**: References memories (IDs only)
- **Assets**: No content organization

## **Proposed Unified Architecture**

### **Base Entity Trait**

```rust
pub trait BaseEntity {
    fn id(&self) -> &String;
    fn created_at(&self) -> u64;
    fn updated_at(&self) -> u64;
    fn access_control(&self) -> &AccessControl;
    fn lifecycle(&self) -> &Lifecycle;
    fn resource_tracking(&self) -> &ResourceTracking;
    fn metadata(&self) -> &EntityMetadata;
}
```

### **Unified Access Control**

```rust
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct AccessControl {
    pub owners: HashMap<PersonRef, OwnerState>,              // 1..n owners
    pub controllers: HashMap<PersonRef, ControllerState>,    // delegated admins
    pub viewers: HashMap<PersonRef, ViewerState>,            // read-only access
    pub access_level: AccessLevel,                          // Public/Private/Shared
    pub permissions: Vec<Permission>,                        // specific permissions
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum AccessLevel {
    Public,                                                  // anyone can read
    Private,                                                 // owners only
    Shared,                                                  // specific people
    Restricted,                                              // controllers only
}
```

### **Unified Lifecycle Management**

```rust
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Lifecycle {
    pub expiration_date: Option<u64>,                        // when entity expires
    pub auto_renewal: bool,                                  // auto-renew before expiration
    pub grace_period_days: u32,                             // grace period after expiration
    pub archived_at: Option<u64>,                           // when archived
    pub deleted_at: Option<u64>,                            // when deleted
    pub retention_policy: RetentionPolicy,                    // how long to keep
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum RetentionPolicy {
    Forever,                                                 // never delete
    Days(u32),                                              // keep for N days
    UntilExpiration,                                        // delete when expired
    Manual,                                                 // manual deletion only
}
```

### **Unified Resource Tracking**

```rust
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ResourceTracking {
    pub allocated_storage_bytes: u64,                       // storage quota
    pub used_storage_bytes: u64,                            // current usage
    pub allocated_cycles: u64,                              // cycle quota
    pub consumed_cycles: u64,                               // current consumption
    pub storage_tier: StorageTier,                          // storage tier
    pub cycle_billing_enabled: bool,                        // whether to track cycles
    pub last_accessed_at: u64,                              // last access time
    pub access_count: u64,                                  // access frequency
}
```

### **Unified Metadata**

```rust
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct EntityMetadata {
    pub title: Option<String>,                              // entity title
    pub description: Option<String>,                        // entity description
    pub tags: Vec<String>,                                  // entity tags
    pub created_by: Option<PersonRef>,                      // creator
    pub updated_by: Option<PersonRef>,                      // last updater
    pub version: u32,                                       // entity version
    pub custom_fields: HashMap<String, String>,             // custom metadata
}
```

## **Refactored Entity Types**

### **1. Unified Capsule**

```rust
pub struct Capsule {
    // Core Identity
    pub id: String,
    pub subject: PersonRef,

    // Unified Components
    pub access_control: AccessControl,
    pub lifecycle: Lifecycle,
    pub resource_tracking: ResourceTracking,
    pub metadata: EntityMetadata,

    // Content
    pub memories: HashMap<String, Memory>,
    pub galleries: HashMap<String, Gallery>,

    // Social Graph
    pub connections: HashMap<PersonRef, Connection>,
    pub connection_groups: HashMap<String, ConnectionGroup>,

    // System
    pub bound_to_neon: bool,
    pub inline_bytes_used: u64,
}
```

### **2. Unified Memory**

```rust
pub struct Memory {
    // Core Identity
    pub id: String,
    pub capsule_id: String,                                // parent capsule

    // Unified Components
    pub access_control: AccessControl,                     // Independent access control
    pub metadata: EntityMetadata,                          // Rich metadata

    // Content
    pub inline_assets: Vec<MemoryAssetInline>,
    pub blob_internal_assets: Vec<MemoryAssetBlobInternal>,
    pub blob_external_assets: Vec<MemoryAssetBlobExternal>,

    // Memory-specific
    pub memory_type: MemoryType,
    pub content_type: String,
    pub parent_folder_id: Option<String>,

    // NOTE: Lifecycle and resource tracking inherited from parent capsule
    // NOTE: No independent expiration - tied to capsule lifecycle
}
```

### **3. Unified Gallery**

```rust
pub struct Gallery {
    // Core Identity
    pub id: String,
    pub capsule_id: String,                                // parent capsule

    // Unified Components
    pub access_control: AccessControl,                     // Independent access control
    pub metadata: EntityMetadata,                          // Rich metadata

    // Content
    pub memories: Vec<String>,                             // memory IDs
    pub galleries: Vec<String>,                             // sub-gallery IDs

    // Gallery-specific
    pub gallery_type: GalleryType,
    pub sort_order: SortOrder,
    pub thumbnail_id: Option<String>,                      // thumbnail memory ID

    // NOTE: Lifecycle and resource tracking inherited from parent capsule
    // NOTE: No independent expiration - tied to capsule lifecycle
}
```

### **4. Unified Asset**

```rust
pub struct Asset {
    // Core Identity
    pub id: String,

    // Unified Components
    pub access_control: AccessControl,
    pub lifecycle: Lifecycle,
    pub resource_tracking: ResourceTracking,
    pub metadata: EntityMetadata,

    // Asset-specific
    pub asset_type: AssetType,
    pub storage_type: StorageType,
    pub content: AssetContent,
    pub parent_memory_id: Option<String>,
}
```

## **Benefits of Unified Architecture**

### **1. Consistency**

- ✅ **Uniform access control** across all entities
- ✅ **Consistent lifecycle management** for all entities
- ✅ **Standardized resource tracking** for all entities
- ✅ **Unified metadata structure** for all entities

### **2. Architectural Clarity**

- ✅ **Capsule = Primary entity** with full lifecycle and resource management
- ✅ **Memory/Gallery = Content entities** with independent access control but inherited lifecycle
- ✅ **Clear hierarchy** - capsule controls lifecycle, entities control access
- ✅ **Simplified resource tracking** - capsule-level quotas cover all content

### **2. Maintainability**

- ✅ **Single source of truth** for common functionality
- ✅ **Easier testing** with consistent interfaces
- ✅ **Simplified API design** with common patterns
- ✅ **Reduced code duplication** across entity types

### **3. User Experience**

- ✅ **Consistent permissions** across all entities
- ✅ **Uniform lifecycle management** for all entities
- ✅ **Standardized resource tracking** for all entities
- ✅ **Predictable behavior** across the platform

### **4. Business Logic**

- ✅ **Unified billing** across all entity types
- ✅ **Consistent quota enforcement** for all entities
- ✅ **Standardized access control** for all entities
- ✅ **Uniform lifecycle policies** for all entities

## **Migration Strategy**

### **Phase 1: Core Types (1-2 weeks)**

- [ ] Create unified base types (`AccessControl`, `Lifecycle`, `ResourceTracking`, `EntityMetadata`)
- [ ] Update `Capsule` to use unified components
- [ ] Add backward compatibility for existing APIs

### **Phase 2: Memory & Gallery (1-2 weeks)**

- [ ] Update `Memory` to use unified components
- [ ] Update `Gallery` to use unified components
- [ ] Migrate existing data to new structure

### **Phase 3: Assets (1-2 weeks)**

- [ ] Create unified `Asset` type
- [ ] Migrate existing asset types to unified structure
- [ ] Update asset management APIs

### **Phase 4: API Integration (1-2 weeks)**

- [ ] Update all APIs to use unified types
- [ ] Add unified access control endpoints
- [ ] Add unified lifecycle management endpoints
- [ ] Add unified resource tracking endpoints

## **Implementation Priority**

### **High Priority**

1. **Access Control Unification** - Critical for security
2. **Lifecycle Management** - Essential for business model
3. **Resource Tracking** - Required for billing

### **Medium Priority**

1. **Metadata Unification** - Improves consistency
2. **API Standardization** - Enhances developer experience

### **Low Priority**

1. **Asset Unification** - Nice to have
2. **Advanced Features** - Future enhancements

## **Next Steps**

1. **Review this analysis** with the tech lead
2. **Prioritize which components** to unify first
3. **Create detailed implementation plan** for chosen components
4. **Begin Phase 1 implementation** with core types

---

**Status**: 🟡 **Analysis Complete**  
**Priority**: 🔥 **High** - Architectural consistency  
**Estimated Effort**: 4-6 weeks  
**Dependencies**: Tech lead review and prioritization

---

## **📋 Recommended Implementation**

**See [Type Consistency Design](type-consistency-design.md) for the complete recommended implementation.**
