# Remove `bound_to_neon` from Backend - Cleanup Issue

## 🎯 Problem

The `bound_to_neon` field is being eliminated from the database and backend, but there are still references to it throughout the codebase, causing TypeScript compilation errors.

## 📋 Current Status

### **✅ What's Working:**

- Frontend build now works (temporary fix applied)
- TypeScript compilation passes

### **❌ What's Broken:**

- Backend still has `bound_to_neon: bool` in `Gallery` struct
- Candid interface still includes `bound_to_neon: boolean`
- Inconsistent state between "removed" comments and actual code

## 🔍 Evidence of the Problem

### **Backend Code Inconsistencies:**

**Still Present:**

```rust
// src/backend/src/types.rs:672
pub struct Gallery {
    // ... other fields
    pub bound_to_neon: bool,  // ← Still here
}

// src/backend/src/capsule.rs:47
bound_to_neon: false, // ← Still being set
```

**Comments Say "Removed":**

```rust
// src/backend/src/capsule_store/hash.rs:457
// bound_to_neon removed - now tracked in database_storage_edges
```

### **Frontend Impact:**

- TypeScript interface requires `bound_to_neon: boolean`
- Code must provide this field or compilation fails
- Temporary fix: `bound_to_neon: false` hardcoded

## 🎯 Goal

**Completely remove `bound_to_neon` from:**

1. Backend Rust structs
2. Candid interface definitions
3. Frontend TypeScript interfaces
4. All related code and logic

## 📋 Tasks

### **Backend Changes:**

- [ ] Remove `bound_to_neon: bool` from `Gallery` struct in `types.rs`
- [ ] Remove `bound_to_neon: bool` from `Capsule` struct in `types.rs`
- [ ] Remove `bound_to_neon: Option<bool>` from `CapsuleUpdateData` in `types.rs`
- [ ] Update all struct initializations to remove `bound_to_neon` field
- [ ] Remove all `bound_to_neon` logic from capsule and gallery operations
- [ ] Update tests to remove `bound_to_neon` references

### **Frontend Changes:**

- [ ] Remove `bound_to_neon: boolean` from `Gallery` interface in `icp-gallery.ts`
- [ ] Remove hardcoded `bound_to_neon: false` from gallery creation
- [ ] Update any UI components that display `bound_to_neon` status
- [ ] Remove `bound_to_neon` from `CapsuleInfo` interface if present

### **Database Changes:**

- [ ] Verify database schema no longer includes `bound_to_neon` fields
- [ ] Update any database queries that reference `bound_to_neon`
- [ ] Remove `bound_to_neon` from any database models

### **Testing:**

- [ ] Update all tests to remove `bound_to_neon` references
- [ ] Verify backend compilation after changes
- [ ] Verify frontend compilation after changes
- [ ] Test gallery and capsule operations work without `bound_to_neon`

## 🚨 Risks

1. **Breaking Changes**: Removing this field might break existing functionality
2. **Data Migration**: Existing data might have `bound_to_neon` values that need handling
3. **API Compatibility**: External consumers might expect this field
4. **Cascade Effects**: Changes in one place might require updates in many others

## 📝 Notes

- The "removed" comments in `hash.rs` suggest this was already attempted but not completed
- The field is still actively used in capsule and gallery operations
- This is a significant refactoring that affects multiple layers of the application

## 🔗 Related Files

**Backend:**

- `src/backend/src/types.rs` - Main struct definitions
- `src/backend/src/capsule.rs` - Capsule operations
- `src/backend/src/gallery.rs` - Gallery operations
- `src/backend/src/capsule_store/` - Storage operations

**Frontend:**

- `src/nextjs/src/services/icp-gallery.ts` - Gallery service
- `src/nextjs/src/ic/declarations/backend/backend.did.d.ts` - Candid interface
- `src/nextjs/src/app/[lang]/user/icp/page.tsx` - UI display

## 🎯 Priority

**High** - This is blocking proper development and creates confusion about the current state of the codebase.

## 📅 Timeline

This should be completed before any major releases to avoid breaking changes.

