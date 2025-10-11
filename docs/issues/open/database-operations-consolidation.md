# Database Operations Consolidation Issue

## **Problem Statement**

Database operations are scattered across 16+ API route files with 44+ direct database calls. This creates several problems:

1. **Code Duplication**: Same database operations repeated across multiple files
2. **Inconsistent Error Handling**: Different error handling patterns in each route
3. **Hard to Test**: Database logic mixed with HTTP handling makes unit testing difficult
4. **Maintenance Burden**: Schema changes require updates in multiple places
5. **No Business Logic Centralization**: Database operations scattered instead of centralized

## **Current State Analysis**

### **Files with Direct Database Operations (16 files, 44+ operations):**

#### **Memory-Related Operations:**

- `src/app/api/upload/complete/route.ts` (3 operations)

  - Memory creation with assets
  - Asset upsert operations
  - User resolution logic

- `src/app/api/memories/[id]/route.ts` (1 operation)

  - Memory updates and storage status

- `src/app/api/memories/[id]/assets/route.ts` (1 operation)

  - Asset upsert operations

- `src/app/api/memories/[id]/share/route.ts` (1 operation)

  - Memory sharing operations

- `src/app/api/memories/delete.ts` (4 operations)

  - Memory deletion logic
  - Asset cleanup

- `src/app/api/memories/utils/memory-creation.ts` (2 operations)

  - Memory creation utilities
  - Asset creation utilities

- `src/app/api/memories/utils/memory-database.ts` (9 operations)

  - Complex memory queries
  - Asset management
  - User resolution

- `src/app/api/memories/utils/user-management.ts` (6 operations)

  - User creation and resolution
  - AllUsers table management

- `src/app/api/memories/utils/image-processing-workflow.ts` (1 operation)
  - Processing status updates

#### **Gallery-Related Operations:**

- `src/app/api/galleries/route.ts` (2 operations)

  - Gallery creation and queries

- `src/app/api/galleries/[id]/route.ts` (2 operations)
  - Gallery updates and management

#### **User-Related Operations:**

- `src/app/api/users/route.ts` (1 operation)

  - User creation

- `src/app/api/users/[id]/route.ts` (3 operations)

  - User updates and deletion
  - Temporary user management

- `src/app/api/users/[id]/business-relationship/route.ts` (1 operation)
  - Business relationship queries

#### **Storage-Related Operations:**

- `src/app/api/storage/edges/route.ts` (1 operation)

  - Storage edge management

- `src/app/api/user-settings/route.ts` (2 operations)
  - User settings CRUD

## **Specific Database Operations Found:**

### **Memory Operations:**

```typescript
// Repeated across multiple files:
db.insert(memories).values({...}).returning()
db.update(memories).set({...}).where(eq(memories.id, id))
db.delete(memories).where(eq(memories.id, id))
db.query.memories.findFirst({...})
db.query.memories.findMany({...})
```

### **Asset Operations:**

```typescript
// Repeated across multiple files:
db.insert(memoryAssets).values({...}).returning()
db.update(memoryAssets).set({...}).where(eq(memoryAssets.id, id))
db.query.memoryAssets.findFirst({...})
db.query.memoryAssets.findMany({...})
```

### **User Operations:**

```typescript
// Repeated across multiple files:
db.insert(allUsers).values({...}).returning()
db.insert(users).values({...}).returning()
db.update(users).set({...}).where(eq(users.id, id))
db.query.allUsers.findFirst({...})
```

### **Gallery Operations:**

```typescript
// Repeated across multiple files:
db.insert(galleries).values({...}).returning()
db.update(galleries).set({...}).where(eq(galleries.id, id))
db.query.galleries.findFirst({...})
```

## **Proposed Solution**

### **1. Create Service Layer Architecture:**

```
src/services/
├── memory/
│   ├── memory-service.ts          ✅ CREATED
│   ├── asset-service.ts           ✅ CREATED
│   └── memory-orchestration.ts    🔄 TODO
├── gallery/
│   ├── gallery-service.ts         🔄 TODO
│   └── gallery-orchestration.ts   🔄 TODO
├── user/
│   ├── user-service.ts            🔄 TODO
│   ├── all-user-service.ts        🔄 TODO
│   └── user-orchestration.ts      🔄 TODO
├── storage/
│   ├── storage-edge-service.ts    🔄 TODO
│   └── storage-orchestration.ts   🔄 TODO
└── settings/
    └── user-settings-service.ts   🔄 TODO
```

### **2. Migration Strategy:**

#### **Phase 1: Memory & Asset Services** ✅ COMPLETED

- [x] Created `MemoryService` with all CRUD operations
- [x] Created `AssetService` with all CRUD operations
- [x] Added clear "Record" naming to indicate database-only operations
- [x] Added comprehensive error handling and logging

#### **Phase 2: Refactor Upload Complete Route** 🔄 IN PROGRESS

- [ ] Replace direct database calls with service methods
- [ ] Remove duplicated logic
- [ ] Improve error handling consistency

#### **Phase 3: Create Remaining Services** 📋 TODO

- [ ] `GalleryService` - Gallery CRUD operations
- [ ] `UserService` - User CRUD operations
- [ ] `AllUserService` - AllUsers table management
- [ ] `StorageEdgeService` - Storage edge management
- [ ] `UserSettingsService` - Settings CRUD operations

#### **Phase 4: Create Orchestration Services** 📋 TODO

- [ ] `MemoryOrchestrationService` - Complete memory operations (DB + Storage)
- [ ] `GalleryOrchestrationService` - Complete gallery operations
- [ ] `UserOrchestrationService` - Complete user operations

#### **Phase 5: Refactor All API Routes** 📋 TODO

- [ ] Replace all direct database calls with service methods
- [ ] Remove utility files that duplicate service functionality
- [ ] Standardize error handling across all routes

### **3. Benefits After Consolidation:**

#### **Code Quality:**

- **Single Responsibility**: Each service handles one domain
- **DRY Principle**: No more duplicated database operations
- **Consistent Error Handling**: Standardized across all operations
- **Type Safety**: Centralized interfaces and types

#### **Maintainability:**

- **Schema Changes**: Update only service layer, not 16+ files
- **Business Logic**: Centralized and reusable
- **Testing**: Services can be unit tested independently
- **Documentation**: Clear service boundaries and responsibilities

#### **Performance:**

- **Connection Pooling**: Centralized database connection management
- **Query Optimization**: Reusable optimized queries
- **Caching**: Service-level caching opportunities

## **Implementation Priority:**

### **High Priority (Immediate):**

1. **Refactor `/api/upload/complete`** - Most complex route with most duplication
2. **Create `GalleryService`** - High usage across multiple routes
3. **Create `UserService`** - Critical for user management

### **Medium Priority:**

4. **Create `StorageEdgeService`** - Storage management operations
5. **Create `UserSettingsService`** - Settings management
6. **Refactor memory-related routes** - Use existing services

### **Low Priority:**

7. **Create orchestration services** - For complete operations
8. **Refactor remaining routes** - Clean up remaining direct DB calls
9. **Remove utility files** - After services are in place

## **Success Metrics:**

- **Reduce database operations** from 44+ scattered calls to ~10 service methods
- **Reduce code duplication** by 70%+ in API routes
- **Improve test coverage** by enabling service-level unit tests
- **Reduce maintenance burden** for schema changes
- **Standardize error handling** across all operations

## **Related Issues:**

- [Upload Complete Route Analysis](./upload-complete-route-analysis.md) - Detailed analysis of the most complex route
- [Memory Shares Migration](./memory-shares-migration.md) - Schema migration that highlighted this issue

## **Next Steps:**

1. **Start with `/api/upload/complete` refactoring** using existing services
2. **Create `GalleryService`** to handle gallery operations
3. **Create `UserService`** to handle user operations
4. **Gradually migrate other routes** to use services
5. **Remove utility files** once services are in place

---

**Created:** 2024-01-XX  
**Priority:** High  
**Effort:** Large (2-3 weeks)  
**Impact:** High (maintainability, testability, code quality)
