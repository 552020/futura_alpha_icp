# Issue: Name/Title Semantics Standardization

**Status**: `COMPLETED` - Implementation Complete  
**Priority**: `MEDIUM` - Architecture Improvement  
**Assigned**: Backend Developer + Frontend Developer  
**Created**: 2024-12-19  
**Completed**: 2025-01-10  
**Related Issues**: [ICP Memory Title Placeholder Display Issue](./icp-memory-title-placeholder-display-issue.md)

## ✅ **COMPLETION SUMMARY**

**Implementation Complete (2025-01-10):**

- ✅ **Shared Utility Function**: Created `title_to_name()` function in `src/backend/src/utils/name_conversion.rs`
- ✅ **Backend Implementation**: Updated Memory, Gallery, and Folder modules to use shared function
- ✅ **Schema Implementation**: NextJS schema.ts implements name/title pattern for all user-facing entities
- ✅ **Consistent Pattern**: All entities now follow the standardized name/title semantics
- ✅ **URL-Safe Names**: Auto-generated from titles using shared transformation logic
- ✅ **Comprehensive Tests**: Added test coverage for name conversion functionality

**Current Implementation Status:**
- **Backend**: ✅ **COMPLETE** - All modules use shared `title_to_name()` function
- **NextJS Schema**: ✅ **COMPLETE** - All tables have both `title` and `name` fields
- **Memory Types**: ✅ **COMPLETE** - Uses shared function for name generation
- **Gallery Types**: ✅ **COMPLETE** - Uses shared function for name generation  
- **Folder Types**: ✅ **COMPLETE** - Uses shared function for name generation

## Problem Description

### Current State

Our codebase has inconsistent `name` vs `title` field usage across different entity types, leading to confusion and potential bugs:

```rust
// Current inconsistent patterns:
MemoryHeader {
  name: String,           // Currently: title fallback
  title: Option<String>,  // Currently: actual title
}

// Other entities may have different patterns
```

### Issues Identified

1. **Inconsistent Semantics**: Different entities use `name`/`title` differently
2. **Redundant Fields**: `MemoryHeader` has both `name` and `title` with unclear purposes
3. **No Standard Pattern**: Each entity type defines its own naming convention
4. **URL Generation**: No clear strategy for generating URL-safe identifiers
5. **User Experience**: Unclear what users see vs. what's used internally

## Analysis

### Entity Types in Our System

#### **User-Facing Entities** (Need both `name` and `title`)

- **Memories**: User creates and names these
- **Folders**: User creates and names these
- **Galleries**: User creates and names these
- **Capsules**: User-facing containers (may need naming)

#### **System Entities** (Need only `name`)

- **Assets**: System-generated (original, display, thumb, placeholder)
- **Blobs**: System-generated storage references
- **Storage Edges**: System-generated tracking records

### Industry Best Practices

#### **Standard Pattern:**

- **`title`**: Human-readable display name (what user sees)
- **`name`**: URL-safe identifier (lowercased, no spaces, no special chars)

#### **Examples:**

| Title (Display)         | Name (URL-safe)         |
| ----------------------- | ----------------------- |
| `"Vacation Photo 2024"` | `"vacation-photo-2024"` |
| `"My Dog's Birthday!"`  | `"my-dogs-birthday"`    |
| `"IMG_2024_12_19.jpg"`  | `"img-2024-12-19-jpg"`  |
| `"Beach Sunset 🌅"`     | `"beach-sunset"`        |

### Current Code Analysis

#### **Memory Types - Current Problem:**

```rust
// From src/backend/src/memories/types.rs
pub struct MemoryMetadata {
    pub title: Option<String>,  // ✅ Correct: user-facing title
}

pub struct MemoryHeader {
    pub name: String,           // ❌ PROBLEM: Just a fallback of title (redundant)
    pub title: Option<String>,  // ✅ Correct: user-facing title
}

// From src/backend/src/memories/adapters.rs line 340
name: self.metadata.title.clone().unwrap_or_else(|| "Untitled".to_string()),
title: self.metadata.title.clone(),
```

**The Issue:**

- We're storing the **same information twice** in `MemoryHeader`
- `name` is just a fallback of `title` - no added value
- `name` should be a URL-safe identifier, not a duplicate of title
- This creates confusion about which field to use for what purpose

**Example of Current Redundancy:**

```rust
// For a memory with title "Vacation Photo 2024":
MemoryHeader {
  name: "Vacation Photo 2024",        // ❌ Same as title (redundant)
  title: Some("Vacation Photo 2024"), // ✅ The actual title
}
```

**What We Want:**

```rust
// For a memory with title "Vacation Photo 2024":
MemoryHeader {
  title: "Vacation Photo 2024",       // ✅ What user sees
  name: "vacation-photo-2024",        // ✅ URL-safe identifier
}
```

#### **Gallery Types - Current Implementation:**

```rust
// From src/backend/src/types.rs
pub struct Gallery {
    pub id: String,
    pub title: String,                    // ❌ PROBLEM: Should be in metadata
    pub description: Option<String>,      // ❌ PROBLEM: Should be in metadata
    pub is_public: bool,                  // ❌ PROBLEM: Should be in metadata
    // ... other fields
}

pub struct GalleryHeader {
    pub id: String,
    pub name: String,                     // ❌ PROBLEM: Just a copy of title
    pub memory_count: u64,
    // ... other fields
}

// From src/backend/src/gallery.rs line 486
impl Gallery {
    pub fn to_header(&self) -> GalleryHeader {
        GalleryHeader {
            name: self.title.clone(),     // ❌ Same redundancy as MemoryHeader
            // ... other fields
        }
    }
}
```

**The Issues:**

1. **Inconsistent Architecture**: Gallery uses direct fields, Memory uses metadata pattern
2. **Same Redundancy**: `name` is just a copy of `title` (like MemoryHeader)
3. **No URL-safe Identifier**: No proper name generation for URLs
4. **Mixed Concerns**: Metadata fields scattered in main struct

#### **Proposed Gallery Structure (Consistent with Memory):**

```rust
// New consistent structure
pub struct Gallery {
    pub id: String,
    pub metadata: GalleryMetadata,        // ✅ Consistent with Memory pattern
    pub storage_location: GalleryStorageLocation,
    pub memory_entries: Vec<GalleryMemoryEntry>,
    // ✅ Removed bound_to_neon - not needed for galleries
}

pub struct GalleryMetadata {
    pub title: String,                    // ✅ User-facing title
    pub description: Option<String>,      // ✅ User-facing description
    pub is_public: bool,                  // ✅ Access control
    pub created_at: u64,
    pub updated_at: u64,
    // ... other metadata fields
}

pub struct GalleryHeader {
    pub id: String,
    pub title: String,                    // ✅ What user sees
    pub name: String,                     // ✅ URL-safe identifier (auto-generated)
    pub memory_count: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub storage_location: GalleryStorageLocation,
}

// New implementation
impl Gallery {
    pub fn to_header(&self) -> GalleryHeader {
        let title = self.metadata.title.clone();
        let name = title_to_name(&title);  // ✅ Generate URL-safe name

        GalleryHeader {
            id: self.id.clone(),
            title,
            name,                          // ✅ Now properly generated
            memory_count: self.memory_entries.len() as u64,
            created_at: self.metadata.created_at,
            updated_at: self.metadata.updated_at,
            storage_location: self.storage_location.clone(),
        }
    }
}
```

#### **Capsule Types - Current Implementation:**

```rust
// From src/backend/src/types.rs
pub struct Capsule {
    pub id: String,                       // ✅ System identifier
    pub subject: PersonRef,               // ✅ User reference
    // No title or name fields - capsules are system entities
}

pub struct CapsuleHeader {
    pub id: String,                       // ✅ System identifier
    pub subject: PersonRef,               // ✅ User reference
    // No title or name fields - capsules are system entities
}
```

**Analysis:**

- ✅ **CORRECT**: Capsules don't need user-facing names
- They're system containers, not user-created content
- `subject` field identifies the person the capsule belongs to

#### **Folder Types - Not Found:**

```rust
// TODO: Search for folder types in codebase
// May not exist yet, or may be implemented differently
```

**Analysis:**

- No folder types found in current codebase
- May be implemented as part of memory organization
- If implemented, should follow same pattern as memories/galleries

#### **Asset Types:**

```rust
// ✅ CORRECT: Assets only have internal names
pub struct MemoryAssetInline {
    // No title field - just internal metadata
}

pub struct MemoryAssetBlobInternal {
    // No title field - just internal metadata
}
```

### Transformation Rules Needed

```rust
fn title_to_name(title: &str) -> String {
    title
        .to_lowercase()
        .replace(" ", "-")           // spaces to hyphens
        .replace("_", "-")           // underscores to hyphens
        .replace(".", "-")           // dots to hyphens
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')  // only alphanumeric + hyphens
        .collect::<String>()
        .trim_matches('-')           // remove leading/trailing hyphens
        .to_string()
}
```

## Proposed Solution

### **Standardized Entity Schema**

#### **User-Facing Entities** (Memories, Folders, Galleries, Capsules)

```rust
pub struct UserFacingEntity {
    pub id: String,
    pub title: String,        // Human-readable display name
    pub name: String,         // URL-safe identifier (auto-generated from title)
    // ... other fields
}
```

#### **System Entities** (Assets, Blobs, Storage Edges)

```rust
pub struct SystemEntity {
    pub id: String,
    pub name: String,         // Internal identifier only
    // ... other fields
}
```

### **Implementation Plan**

#### **Phase 1: Define Standards**

1. **Create naming utility functions**
2. **Define entity type categories**
3. **Document naming conventions**

#### **Phase 2: Update Memory Types**

1. **Fix `MemoryHeader` name/title logic**
2. **Implement auto-generation of `name` from `title`**
3. **Update memory creation functions**

#### **Phase 3: Extend to Other Entities**

1. **Update Folder types**
2. **Update Gallery types**
3. **Update Capsule types (if needed)**

#### **Phase 4: Frontend Integration**

1. **Update display logic**
2. **Update URL generation**
3. **Update search functionality**

### **Specific Changes Needed**

#### **Backend Changes**

**File**: `src/backend/src/memories/types.rs`

```rust
// Add utility function
pub fn title_to_name(title: &str) -> String {
    // Implementation as shown above
}

// Update MemoryHeader creation
impl From<&Memory> for MemoryHeader {
    fn from(memory: &Memory) -> Self {
        let title = memory.metadata.title.clone().unwrap_or_else(|| "Untitled".to_string());
        let name = title_to_name(&title);

        MemoryHeader {
            id: memory.id.clone(),
            title: Some(title),
            name,  // Now properly generated from title
            // ... other fields
        }
    }
}
```

**File**: `src/backend/src/memories/adapters.rs`

```rust
// Remove the confusing name fallback logic (line 340)
// Replace with proper title-to-name generation
```

#### **Frontend Changes**

**File**: `src/nextjs/src/services/memories.ts`

```typescript
// Update transformICPMemoryHeaderToNeon to use consistent naming
const transformICPMemoryHeaderToNeon = (header: MemoryHeader): MemoryWithFolder => {
  return {
    // Use title for display, name for URLs if needed
    title: header.title || header.name || "Untitled",
    // ... other fields
  };
};
```

### **Benefits**

1. **Consistency**: All user-facing entities follow the same pattern
2. **Clarity**: Clear distinction between display names and identifiers
3. **URL Safety**: Automatic generation of URL-safe identifiers
4. **Maintainability**: Single source of truth for naming logic
5. **User Experience**: Predictable behavior across all entities

### **Migration Strategy**

#### **Existing Data**

- **Memories with "placeholder" titles**: Fix during memory creation update
- **Existing valid titles**: Auto-generate names from titles
- **No data loss**: All existing titles preserved

#### **Backward Compatibility**

- **API**: Keep existing field names, just fix the logic
- **Frontend**: No breaking changes to display logic
- **Database**: No schema changes needed

## Testing Scenarios

### **Unit Tests**

```rust
#[test]
fn test_title_to_name_conversion() {
    assert_eq!(title_to_name("Vacation Photo 2024"), "vacation-photo-2024");
    assert_eq!(title_to_name("My Dog's Birthday!"), "my-dogs-birthday");
    assert_eq!(title_to_name("IMG_2024_12_19.jpg"), "img-2024-12-19-jpg");
    assert_eq!(title_to_name("Beach Sunset 🌅"), "beach-sunset");
}

#[test]
fn test_memory_header_name_generation() {
    let memory = create_test_memory_with_title("Summer Photos");
    let header = MemoryHeader::from(&memory);
    assert_eq!(header.title, Some("Summer Photos".to_string()));
    assert_eq!(header.name, "summer-photos");
}
```

### **Integration Tests**

- **Memory creation**: Verify name is auto-generated from title
- **URL generation**: Verify URLs use name field correctly
- **Search**: Verify search works with both title and name
- **Display**: Verify UI shows title, not name

## Success Criteria

- [x] **All user-facing entities have consistent `name`/`title` pattern** ✅ **COMPLETED**
  - Memory, Gallery, and Folder types all implement the pattern
  - NextJS schema.ts has both fields for all user-facing entities
- [x] **System entities use only `name` field** ✅ **COMPLETED**
  - Assets, blobs, and other system entities use only internal names
- [x] **URL-safe names are auto-generated from titles** ✅ **COMPLETED**
  - Shared `title_to_name()` function handles all transformations
  - Comprehensive test coverage for edge cases
- [x] **No breaking changes to existing functionality** ✅ **COMPLETED**
  - All existing APIs maintained, only internal logic improved
- [x] **Clear documentation of naming conventions** ✅ **COMPLETED**
  - Function documentation with examples
  - This document serves as the reference
- [x] **All tests pass** ✅ **COMPLETED**
  - Backend tests pass with new utility function
  - Schema validation passes

## ✅ **IMPLEMENTATION DETAILS**

### **Backend Implementation (Rust)**

**Shared Utility Function**: `src/backend/src/utils/name_conversion.rs`
```rust
pub fn title_to_name(title: &str) -> String {
    if title.trim().is_empty() {
        return "untitled".to_string();
    }
    
    title
        .to_lowercase()
        .replace(" ", "-")           // spaces to hyphens
        .replace("_", "-")           // underscores to hyphens
        .replace(".", "-")           // dots to hyphens
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')  // only alphanumeric + hyphens
        .collect::<String>()
        .trim_matches('-')           // remove leading/trailing hyphens
        .to_string()
}
```

**Memory Implementation**: `src/backend/src/memories/adapters.rs`
```rust
let title = self.metadata.title.clone();
let name = title.as_ref()
    .map(|t| crate::utils::title_to_name(t))
    .unwrap_or_else(|| "untitled".to_string());
```

**Gallery Implementation**: `src/backend/src/gallery/domain.rs`
```rust
let title = self.metadata.title.clone();
let name = title.as_ref()
    .map(|t| crate::utils::title_to_name(t))
    .unwrap_or_else(|| "untitled".to_string());
```

**Folder Implementation**: `src/backend/src/folder/domain.rs`
```rust
let title = self.metadata.title.clone();
let name = title.as_ref()
    .map(|t| crate::utils::title_to_name(t))
    .unwrap_or_else(|| "untitled".to_string());
```

### **NextJS Schema Implementation (TypeScript)**

**Memory Table**: `src/nextjs/src/db/schema.ts`
```typescript
export const memories = pgTable('memories', {
  // ...
  title: text('title'),
  name: text('name'), // ✅ URL-safe identifier (auto-generated from title)
  // ...
});
```

**Folder Table**: `src/nextjs/src/db/schema.ts`
```typescript
export const folders = pgTable('folders', {
  // ...
  title: text('title').notNull(), // ✅ User-facing display name
  name: text('name').notNull(), // ✅ URL-safe identifier (auto-generated from title)
  // ...
});
```

**Gallery Table**: `src/nextjs/src/db/schema.ts`
```typescript
export const galleries = pgTable('gallery', {
  // ...
  title: text('title').notNull(),
  name: text('name').notNull(), // ✅ URL-safe identifier (auto-generated from title)
  // ...
});
```

### **Test Coverage**

**Backend Tests**: `src/backend/src/utils/name_conversion.rs`
```rust
#[test]
fn test_title_to_name_conversion() {
    assert_eq!(title_to_name("Vacation Photo 2024"), "vacation-photo-2024");
    assert_eq!(title_to_name("My Dog's Birthday!"), "my-dogs-birthday");
    assert_eq!(title_to_name("IMG_2024_12_19.jpg"), "img-2024-12-19-jpg");
    assert_eq!(title_to_name("Beach Sunset 🌅"), "beach-sunset");
    assert_eq!(title_to_name(""), "untitled");
}
```

## Priority Justification

**MEDIUM Priority** because:

- **Architecture Improvement**: Establishes consistent patterns
- **Future-Proofing**: Makes adding new entities easier
- **Maintainability**: Reduces confusion and bugs
- **User Experience**: Ensures predictable behavior
- **Not Urgent**: Doesn't block current functionality

## Dependencies

- Backend developer (Rust)
- Frontend developer (TypeScript)
- QA for testing
- Documentation updates

## Timeline

- **Week 1**: Define standards and utility functions
- **Week 2**: Update Memory types and logic
- **Week 3**: Extend to other entities
- **Week 4**: Frontend integration and testing

**Total Estimated Time**: 3-4 weeks

## Notes

- This issue should be resolved before implementing the unified ICP memory creation API
- The naming standards will be used across all future entity types
- Consider this a foundational improvement that enables better architecture
