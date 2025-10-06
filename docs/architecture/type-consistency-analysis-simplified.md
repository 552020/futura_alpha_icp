# **Type Consistency Analysis: Current State & Decision Points**

## **📋 Overview**

This document analyzes type consistency issues across the main entity types (Capsules, Memories, Galleries, Assets, Person) and identifies decision points for the tech lead.

---

## **🔍 Current Type Analysis**

### **Main Entity Types:**

- **Capsule**: Core entity with `id: String`
- **Memory**: Content entity with `id: String`
- **Gallery**: Collection entity with `id: String`
- **Asset**: Resource entity (no direct ID, uses indices)
- **Person**: Identity entity with `id: String`

### **Sub-Types:**

- **CapsuleInfo**: Summary with `capsule_id: String` (foreign key)
- **CapsuleHeader**: Lightweight with `id: String` (self ID)
- **MemoryHeader**: Lightweight with `id: String` (self ID)

---

## **🚨 Critical Issues**

### **1. Asset Index Fragility**

**Problem**: Assets are referenced by array index, making references unstable when assets are removed.

**Impact**:

- External references break when assets are deleted
- No stable asset identity across modifications
- Cannot share assets between memories

**Example**:

```rust
// ❌ Fragile: Index 0 becomes invalid if first asset is removed
fn asset_remove_by_index(memory_id: String, asset_index: u32)
```

### **2. Entity Reference Type Proliferation Problem**

**Problem**: Multiple ways to reference entities within other entities, leading to type confusion.

**Current Inconsistencies**:

- `CapsuleInfo.capsule_id` vs `Capsule.id`
- `Gallery.memories: Vec<String>` (are these memory IDs?)
- Mixed self-ID vs foreign key patterns

---

## **💡 Proposed Solutions**

### **1. Typed IDs Solution**

**Problem**: Current `String` IDs allow parameter mixups and lack type safety.

**Solution**: Create typed ID wrappers for type safety:

```rust
pub struct CapsuleId(String);
pub struct MemoryId(String);
pub struct GalleryId(String);
pub struct AssetId(String);
```

**Benefits**:

- ✅ **Type Safety**: Prevents ID mixups at compile time
- ✅ **API Clarity**: Function signatures are self-documenting
- ✅ **Consistency**: All entities use same pattern

### **2. API Parameter Naming Strategy**

**Problem**: With typed IDs, parameter names could be generic (`id`) or specific (`memory_id`).

**Decision Point**: Keep specific parameter names even with typed IDs.

**Rationale**:

- Mixed parameter calls remain clear
- API is self-documenting
- Backward compatibility

### **3. Struct Field Naming Strategy**

**Problem**: Inconsistent naming between self IDs and foreign keys.

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

### **4. Asset ID Implementation**

**Problem**: Current index-only design is too fragile for production.

**Solution**: Add unique asset IDs to all asset types.

**Benefits**:

- ✅ **Stable References**: External references remain valid
- ✅ **Asset Sharing**: Can reference assets across memories
- ✅ **Type Safety**: Prevents index/ID confusion

**Implementation Options**:

1. **Add Asset IDs**: Add unique IDs to all asset types
2. **Hybrid Approach**: Support both ID and index-based access
3. **Feature Flags**: Make asset IDs optional/configurable

---

## **📋 Decision Points for Tech Lead**

### **1. API Parameter Naming**

**Question**: With typed IDs, should we use generic (`id`) or specific (`memory_id`) parameter names?

**Options**:

- **Generic**: `fn memories_read(id: MemoryId)`
- **Specific**: `fn memories_read(memory_id: MemoryId)`

**Recommendation**: **Specific names** - clearer for mixed parameter calls.

### **2. Struct Field Naming**

**Question**: How should we name fields that reference other entities?

**Options**:

- **Generic**: `id` for all identifiers
- **Semantic**: `id` for self, `{entity}_id` for foreign keys

**Recommendation**: **Semantic distinction** - matches database patterns.

### **3. Asset ID Implementation**

**Question**: Should we add asset IDs to resolve the fragility problem?

**Options**:

- **Keep Current**: Index-only (fragile but simple)
- **Add Asset IDs**: Stable references (safe but more complex)

**Recommendation**: **Add Asset IDs** - current design too fragile for production.

### **4. Gallery Memory References**

**Question**: How should galleries reference memories?

**Options**:

- **Keep Current**: `memories: Vec<String>` (inconsistent)
- **Explicit Foreign Keys**: `memory_ids: Vec<String>`
- **Structured References**: `memory_references: Vec<MemoryReference>`

**Recommendation**: **Explicit foreign keys** - clearest and most consistent.

---

## **🎯 Success Criteria**

- ✅ **Type Safety**: All ID mixups prevented at compile time
- ✅ **API Clarity**: All function signatures are self-documenting
- ✅ **Consistency**: All structs follow same naming patterns
- ✅ **Asset Safety**: External references to assets remain valid
- ✅ **Backward Compatibility**: Minimal breaking changes to existing APIs
- ✅ **Performance**: No significant performance impact from typed IDs
- ✅ **Maintainability**: Clear, consistent patterns across all entity types

---

## **📋 Recommended Implementation**

**See [Type Consistency Design](type-consistency-design.md) for the complete recommended implementation.**

