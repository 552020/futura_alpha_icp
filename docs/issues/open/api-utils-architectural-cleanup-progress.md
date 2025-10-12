# API Utils Architectural Cleanup - Progress Report

## Issue Summary

**Status**: 🔄 IN PROGRESS  
**Priority**: High  
**Type**: Architectural Refactoring  
**Assignee**: Development Team  
**Created**: 2024-01-XX  
**Last Updated**: 2024-01-XX

## Background

During the storage edges API schema mismatch fix, we discovered that multiple utility files in `src/nextjs/src/app/api/memories/utils/` were violating our service layer architecture pattern by performing direct database operations instead of using centralized service functions.

## ✅ **COMPLETED WORK**

### 1. **`queries.ts` - ✅ FULLY REFACTORED**

**What was fixed:**

- ❌ **Before**: Direct `db.execute(sql`...`)` calls
- ✅ **After**: Uses `getMemoryRecordsWithGalleries()` from service layer

**Changes made:**

- Created `getMemoryRecordsWithGalleries()` in `src/nextjs/src/services/memory/memory-operations.ts`
- Added `MemoryWithGalleries` type to `src/nextjs/src/services/memory/types.ts`
- Refactored `fetchMemoriesWithGalleries()` to use service layer
- Maintained backward compatibility with existing interfaces

**Files modified:**

- `src/nextjs/src/services/memory/memory-operations.ts` - Added new service function
- `src/nextjs/src/services/memory/types.ts` - Added type definition
- `src/nextjs/src/services/memory/index.ts` - Exported new function and type
- `src/nextjs/src/app/api/memories/utils/queries.ts` - Refactored to use service layer

### 2. **`memory-creation.ts` - ✅ FULLY REFACTORED**

**What was fixed:**

- ❌ **Before**: Direct `db.insert(memories)` and `db.insert(memoryAssets)` calls
- ✅ **After**: Uses `createMemoryWithAssets()` from service layer

**Changes made:**

- Created `createMemoryWithAssets()` in `src/nextjs/src/services/memory/memory-operations.ts`
- Refactored `createMemory()` and `createMemoryFromBlob()` to use service layer
- Maintained backward compatibility with existing interfaces
- Added comprehensive error handling and logging

**Files modified:**

- `src/nextjs/src/services/memory/memory-operations.ts` - Added unified creation function
- `src/nextjs/src/services/memory/index.ts` - Exported new function
- `src/nextjs/src/app/api/memories/utils/memory-creation.ts` - Refactored to use service layer

### 3. **`user-management.ts` - 🔄 PARTIALLY REFACTORED**

**What was fixed:**

- ❌ **Before**: Direct `db.select()` and `db.insert()` calls
- ✅ **After**: Uses dedicated user service layer functions

**Changes made:**

- Created new user service layer: `src/nextjs/src/services/user/`
- Added `getAuthenticatedUserId()`, `getTemporaryUserId()`, `createUserWithAllUser()` functions
- Refactored `getAllUserId()` and `getUserIdForUpload()` to use service layer
- Maintained backward compatibility with existing interfaces

**Files created:**

- `src/nextjs/src/services/user/types.ts` - User service types
- `src/nextjs/src/services/user/user-operations.ts` - User service functions
- `src/nextjs/src/services/user/index.ts` - User service exports

**Files modified:**

- `src/nextjs/src/app/api/memories/utils/user-management.ts` - Refactored to use service layer

**Current Status**: ⚠️ **BUILD ISSUES** - TypeScript compilation errors need to be resolved

## 🚧 **REMAINING WORK**

### 1. **`access.ts` - ❌ NOT STARTED**

**Current Issues:**

- Direct `db.query.memories.findFirst()` calls
- Direct `db.query.resourceMembership.findFirst()` calls
- Violates service layer architecture

**Required Actions:**

- Create access control service functions
- Refactor `getMemoryAccessLevel()` to use service layer
- Maintain backward compatibility

### 2. **Fix Build Issues for `user-management.ts`**

**Current Problems:**

- TypeScript compilation errors in user service layer
- Type assertion issues with service layer return types
- Need to resolve all build errors before proceeding

**Required Actions:**

- Fix TypeScript type issues in `src/nextjs/src/services/user/`
- Ensure proper type definitions for all service functions
- Verify successful build compilation

### 3. **Move Utility Files to Appropriate Locations**

**Current Issues:**

- Utility files are in API folder instead of `lib/` or service layer
- Violates separation of concerns

**Required Actions:**

- Move pure utility functions to `src/nextjs/src/lib/`
- Move orchestration functions to appropriate service layers
- Update all import statements across the codebase

## 📊 **Progress Summary**

| **File**             | **Status**     | **Direct DB Ops** | **Service Layer** | **Build Status** |
| -------------------- | -------------- | ----------------- | ----------------- | ---------------- |
| `queries.ts`         | ✅ Complete    | ❌ Eliminated     | ✅ Implemented    | ✅ Working       |
| `memory-creation.ts` | ✅ Complete    | ❌ Eliminated     | ✅ Implemented    | ✅ Working       |
| `user-management.ts` | 🔄 Partial     | ❌ Eliminated     | ✅ Implemented    | ⚠️ Build Issues  |
| `access.ts`          | ❌ Not Started | ❌ Present        | ❌ Missing        | ❌ Unknown       |

## 🎯 **Next Steps (Priority Order)**

### **Immediate (High Priority)**

1. **Fix build issues** in `user-management.ts` refactoring
2. **Complete `access.ts` refactoring** to use service layer
3. **Verify all builds pass** after refactoring

### **Secondary (Medium Priority)**

4. **Move utility files** to appropriate locations (`lib/` vs service layer)
5. **Update import statements** across the codebase
6. **Add comprehensive tests** for new service layer functions

### **Future (Low Priority)**

7. **Documentation updates** for new service layer architecture
8. **Performance optimization** of service layer functions
9. **Code review** of all refactored files

## 🔍 **Architectural Benefits Achieved**

### **Service Layer Architecture**

- ✅ Centralized database operations
- ✅ Reusable service functions
- ✅ Better error handling and logging
- ✅ Improved testability
- ✅ Type safety improvements

### **Code Quality**

- ✅ Eliminated code duplication
- ✅ Better separation of concerns
- ✅ Consistent error handling patterns
- ✅ Comprehensive logging

### **Maintainability**

- ✅ Single source of truth for database operations
- ✅ Easier to modify business logic
- ✅ Better code organization
- ✅ Reduced coupling between API routes and database

## 🚨 **Critical Issues to Address**

1. **Build Failures**: TypeScript compilation errors must be resolved
2. **Type Safety**: Ensure all service layer functions have proper type definitions
3. **Backward Compatibility**: Verify all existing API endpoints continue to work
4. **Testing**: Add tests for new service layer functions

## 📝 **Files That Need Import Updates**

After moving utility files, these files will need import path updates:

- `src/nextjs/src/app/api/upload/s3/presign/route.ts`
- `src/nextjs/src/app/api/upload/vercel-blob/grant/route.ts`
- `src/nextjs/src/app/api/folders/route.ts`
- `src/nextjs/src/app/api/memories/get.ts`

## 🎉 **Success Criteria**

- [ ] All direct database operations eliminated from utility files
- [ ] All builds pass without TypeScript errors
- [ ] All existing API endpoints continue to work
- [ ] Service layer functions are properly tested
- [ ] Utility files moved to appropriate locations
- [ ] All import statements updated
- [ ] Documentation updated

---

**Related Issues**:

- Storage Edges API Schema Mismatch Critical Bug
- Memory Database Utils Architectural Decision

**Estimated Completion**: 2-3 days (depending on build issue complexity)
