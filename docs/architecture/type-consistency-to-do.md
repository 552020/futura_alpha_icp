# Type Consistency Implementation TODO

## **Phase A: Non-Breaking Changes**

### **1. Asset ID Implementation**

#### **1.1 Add Asset ID to All Asset Variants**

- [ ] Add `asset_id: String` field to `MemoryAssetInline`
- [ ] Add `asset_id: String` field to `MemoryAssetBlobInternal`
- [ ] Add `asset_id: String` field to `MemoryAssetBlobExternal`
- [ ] Update asset creation logic to generate UUID for `asset_id`
- [ ] Update asset retrieval logic to use `asset_id` for lookups

#### **1.2 Asset API Updates**

- [ ] Add new API endpoints that accept `asset_id` parameter
- [ ] Update documentation to use `asset_id` parameter

#### **1.3 Asset ID Migration**

- [ ] Create migration script to populate `asset_id` for existing assets
- [ ] Generate UUIDs for all existing assets
- [ ] Verify no asset ID conflicts
- [ ] Test asset retrieval with both old and new methods

### **2. Gallery Memory Entry Implementation**

#### **2.1 Create GalleryMemoryEntry Struct**

- [ ] Define `GalleryMemoryEntry` struct with fields:
  - [ ] `memory_id: String`
  - [ ] `added_at: u64`
  - [ ] `added_by: PersonRef`
  - [ ] `display_order: u32`
  - [ ] `notes: Option<String>`

#### **2.2 Update Gallery Struct**

- [ ] Replace `Vec<String>` with `Vec<GalleryMemoryEntry>` in `Gallery` struct
- [ ] Update gallery creation logic
- [ ] Update gallery modification logic
- [ ] Update gallery serialization/deserialization

#### **2.3 Gallery Migration**

- [ ] Create migration script to convert existing `Vec<String>` to `Vec<GalleryMemoryEntry>`
- [ ] Set default values: `added_at = created_at`, `added_by = creator`, `display_order = index`
- [ ] Verify all existing galleries migrate correctly
- [ ] Test gallery operations with new structure

### **3. API Parameter Standardization**

#### **3.1 Standardize API Parameter Names**

- [ ] Update all API endpoints to use specific parameter names:
  - [ ] `capsule_id` instead of generic `id` for capsule operations
  - [ ] `memory_id` instead of generic `id` for memory operations
  - [ ] `gallery_id` instead of generic `id` for gallery operations
  - [ ] `asset_id` instead of generic `id` for asset operations

#### **3.2 API Documentation Updates**

- [ ] Update API documentation to show new parameter names
- [ ] Update client code to use new parameter names

## **Phase B: Type System Enhancements**

### **4. Missing Header/Info Types**

#### **4.1 Create Missing Header Types**

- [ ] Create `GalleryHeader` struct
- [ ] Create `AssetHeader` struct
- [ ] Ensure all header types have: `id`, `created_at`, `updated_at`, minimal summary fields
- [ ] No computed permissions in header types

#### **4.2 Create Missing Info Types**

- [ ] Create `MemoryInfo` struct
- [ ] Ensure all info types have: `id`, computed data (counts/flags like `is_owner`, `is_controller`)
- [ ] Per-user computed data only

#### **4.3 Update List/Detail Endpoints**

- [ ] Update list endpoints to return appropriate `*Header` types
- [ ] Update detail endpoints to return appropriate `*Info` types
- [ ] Ensure consistent field naming across all types
- [ ] Update frontend to handle new type structure

### **5. Struct Field Naming Standardization**

#### **5.1 Self ID vs Foreign Key Naming**

- [ ] Ensure all structs use `id` for self-identifier
- [ ] Ensure all structs use `{entity}_id` for foreign keys
- [ ] Update existing structs that don't follow this pattern
- [ ] Add linting rules to enforce this pattern

#### **5.2 Update Existing Structs**

- [ ] Review all existing structs for naming consistency
- [ ] Update any structs that use `{entity}_id` for self-identifier
- [ ] Update any structs that use generic `id` for foreign keys
- [ ] Test all struct serialization/deserialization

## **Phase C: Deprecation and Cleanup**

### **6. Deprecate Index-Based Asset Operations**

#### **6.1 Remove Index-Based Endpoints**

- [ ] Remove index-based asset endpoints
- [ ] Update all client code to use `asset_id` instead of index
- [ ] Remove index-based asset logic from backend
- [ ] Clean up unused code

### **7. Documentation and Guardrails**

#### **7.1 Create ID Usage Documentation**

- [ ] Create documentation page with three tables:
  - [ ] **Entity structs** (self ids & FKs)
  - [ ] **Header vs Info fields** per entity
  - [ ] **ID usage rules** (structs vs API params)
- [ ] Include examples for each pattern
- [ ] Include migration examples

#### **7.2 Add Linting Rules**

- [ ] Add schema/IDL linting to forbid `{entity}_id` for self ids in structs
- [ ] Add linting to require `{entity}_id` for foreign keys
- [ ] Add linting to enforce API parameter naming conventions
- [ ] Add linting to enforce header vs info type distinctions

#### **7.3 Add Test Requirements**

- [ ] Add tests for asset reordering (must preserve `asset_id`s)
- [ ] Add tests for gallery entry reordering (must preserve `added_at/added_by`)
- [ ] Add tests for struct field naming consistency
- [ ] Add tests for API parameter naming consistency

## **Phase D: Future Enhancements (Post-MVP)**

### **8. Typed ID Newtypes (Deferred)**

#### **8.1 Introduce Typed IDs**

- [ ] Create `CapsuleId` newtype wrapper
- [ ] Create `MemoryId` newtype wrapper
- [ ] Create `GalleryId` newtype wrapper
- [ ] Create `AssetId` newtype wrapper
- [ ] Update all structs to use typed IDs

#### **8.2 Update API Signatures**

- [ ] Update all API endpoints to use typed IDs
- [ ] Update frontend to handle typed IDs
- [ ] Update serialization/deserialization for typed IDs
- [ ] Add type safety benefits

### **9. Unified Access/Lifecycle/Resource (Deferred)**

#### **9.1 Create Unified Base Types**

- [ ] Create `AccessControl` base type
- [ ] Create `Lifecycle` base type
- [ ] Create `ResourceTracking` base type
- [ ] Create `EntityMetadata` base type

#### **9.2 Apply Unified Types**

- [ ] Update all entities to use unified base types
- [ ] Ensure consistent behavior across all entities
- [ ] Add unified API endpoints
- [ ] Update frontend to handle unified types

## **Testing Requirements**

### **10. Comprehensive Testing**

#### **10.1 Asset ID Testing**

- [ ] Test asset creation with `asset_id`
- [ ] Test asset retrieval by `asset_id`
- [ ] Test asset reordering preserves `asset_id`s

#### **10.2 Gallery Entry Testing**

- [ ] Test gallery creation with `GalleryMemoryEntry`
- [ ] Test gallery modification preserves entry metadata
- [ ] Test gallery reordering preserves `added_at/added_by`
- [ ] Test migration from old `Vec<String>` format

#### **10.3 API Parameter Testing**

- [ ] Test all API endpoints with new parameter names
- [ ] Test parameter validation

#### **10.4 Type System Testing**

- [ ] Test all `*Header` types have required fields
- [ ] Test all `*Info` types have computed fields
- [ ] Test struct field naming consistency
- [ ] Test API parameter naming consistency

## **Success Criteria**

### **11. Completion Checklist**

#### **11.1 Phase A Success**

- [ ] All assets have `asset_id` field
- [ ] All galleries use `GalleryMemoryEntry` structure
- [ ] All API parameters use specific names
- [ ] No breaking changes to existing functionality

#### **11.2 Phase B Success**

- [ ] All missing `*Header` and `*Info` types exist
- [ ] All list/detail endpoints return appropriate types
- [ ] All structs follow naming conventions
- [ ] Frontend handles new type structure

#### **11.3 Phase C Success**

- [ ] All deprecated endpoints removed
- [ ] All client code uses new patterns
- [ ] Documentation is complete and accurate
- [ ] Linting rules are enforced

#### **11.4 Overall Success**

- [ ] No naming drift between entities
- [ ] No asset index fragility
- [ ] No header/info type drift
- [ ] Consistent API patterns across all entities
- [ ] Clear migration path for future changes
