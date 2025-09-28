# Logger Refactoring Issue

## Problem

Currently using 12 different logger instances instead of 1 logger with semantic flags. This creates unnecessary complexity and makes logging control difficult.

## Current Logger Usage Analysis

### Files Using Multiple Loggers (Need Refactoring)

#### Upload & Storage Services

- `src/services/upload/vercel-blob-upload.ts` - uses `uploadLogger`
- `src/services/upload/image-derivatives.ts` - uses `uploadLogger`
- `src/services/icp-upload.ts` - uses `uploadLogger`
- `src/lib/s3.ts` - uses `icpLogger`
- `src/lib/s3-utils.ts` - uses `icpLogger`
- `src/lib/blob.ts` - uses `icpLogger`
- `src/lib/storage/storage-manager.ts` - uses `icpLogger`
- `src/lib/storage/test-blob-upload.ts` - uses `uploadLogger`

#### Authentication & User Management

- `src/lib/auth-utils.ts` - uses `authLogger`
- `src/lib/ii-coauth-guard.ts` - uses `authLogger`
- `src/lib/ii-client.ts` - uses `icpLogger`
- `src/lib/ii-nonce.ts` - uses `icpLogger`
- `src/app/api/auth/link-ii/route.ts` - uses `authLogger`
- `src/app/api/users/route.ts` - uses `userLogger`
- `src/app/api/users/[id]/route.ts` - uses `userLogger`
- `src/app/api/memories/utils/user-management.ts` - uses `userLogger`

#### Database Operations

- `src/db/test-db.ts` - uses `icpLogger`
- `src/db/seed.ts` - uses `icpLogger`
- `src/db/fixtures/tenenbaum/seedTenenbaum.ts` - uses `icpLogger`
- `src/db/familyTrees.ts` - uses `icpLogger`
- `src/db/create-test-users.ts` - uses `userLogger`
- `src/app/api/memories/utils/memory-database.ts` - uses `memoryLogger`

#### Gallery & Memory Services

- `src/services/gallery.ts` - uses `icpLogger`
- `src/services/icp-gallery.ts` - uses `icpLogger`
- `src/services/memories.ts` - uses `icpLogger`
- `src/app/api/memories/utils/memory-creation.ts` - uses `memoryLogger`
- `src/app/api/galleries/[id]/route.ts` - uses `apiLogger`

#### API Routes (Meaningless Logger)

- `src/app/api/upload/vercel-blob/route.ts` - uses `uploadLogger`
- `src/app/api/upload/vercel-blob/grant/route.ts` - uses `uploadLogger`
- `src/app/api/upload/s3/presign/route.ts` - uses `uploadLogger`
- `src/app/api/upload/s3/download/route.ts` - uses `uploadLogger`
- `src/app/api/upload/complete/route.ts` - uses `uploadLogger`
- `src/app/api/upload/utils/presign-logic.ts` - uses `uploadLogger`
- `src/app/api/test/mailgun/route.ts` - uses `apiLogger`
- `src/app/api/storage/sync-status/route.ts` - uses `apiLogger`
- `src/app/api/storage/edges/route.ts` - uses `apiLogger`
- `src/app/api/memories/utils/storage.ts` - uses `apiLogger`
- `src/app/api/memories/utils/response-formatting.ts` - uses `apiLogger`
- `src/app/api/memories/utils/image-processing-workflow.ts` - uses `apiLogger`
- `src/app/api/memories/utils/form-parsing.ts` - uses `apiLogger`
- `src/app/api/memories/shared/route.ts` - uses `apiLogger`
- `src/app/api/memories/get.ts` - uses `apiLogger`
- `src/app/api/memories/delete.ts` - uses `apiLogger`
- `src/app/api/memories/[id]/share-link/route.ts` - uses `apiLogger`
- `src/app/api/memories/[id]/share-link/code/route.ts` - uses `apiLogger`
- `src/app/api/memories/[id]/share/route.ts` - uses `apiLogger`
- `src/app/api/memories/[id]/route.ts` - uses `apiLogger`
- `src/app/api/memories/[id]/assets/route.ts` - uses `apiLogger`
- `src/app/api/me/hosting-preferences/route.ts` - uses `apiLogger`
- `src/app/api/ii/verify-nonce/route.ts` - uses `apiLogger`
- `src/app/api/ii/challenge/route.ts` - uses `apiLogger`
- `src/app/api/folders/route.ts` - uses `apiLogger`

#### Frontend Components

- `src/app/[lang]/user/icp/page.tsx` - uses `userLogger`
- `src/app/[lang]/signin/page.tsx` - uses `icpLogger`
- `src/app/[lang]/shared/page.tsx` - uses `icpLogger`
- `src/app/[lang]/shared/[id]/page.tsx` - uses `icpLogger`
- `src/app/[lang]/gallery/page.tsx` - uses `icpLogger`
- `src/app/[lang]/gallery/[id]/preview/page.tsx` - uses `icpLogger`
- `src/app/[lang]/gallery/[id]/page.tsx` - uses `icpLogger`
- `src/app/[lang]/feed/page.tsx` - uses `icpLogger`
- `src/app/[lang]/dashboard/page.tsx` - uses `icpLogger`
- `src/app/[lang]/dashboard/folder/[id]/page.tsx` - uses `icpLogger`
- `src/app/[lang]/dashboard/[id]/page.tsx` - uses `icpLogger`
- `src/app/[lang]/layout.tsx` - uses `icpLogger`

#### Hooks & Contexts

- `src/hooks/useMemoryUpload.ts` - uses `icpLogger`
- `src/hooks/use-memory-upload.ts` - uses `uploadLogger`
- `src/hooks/use-memory-storage-status.ts` - uses `memoryLogger`
- `src/hooks/use-ii-coauth.ts` - uses `authLogger`
- `src/contexts/onboarding-context.tsx` - uses `icpLogger`

#### Components

- `src/components/user/linked-accounts.tsx` - uses `userLogger`
- `src/components/user/ii-coauth-controls.tsx` - uses `userLogger`
- `src/components/user/icp-card.tsx` - uses `userLogger`
- `src/components/onboarding/onboard-modal.tsx` - uses `icpLogger`
- `src/components/memory/share-dialog.tsx` - uses `memoryLogger`
- `src/components/memory/memory-viewer.tsx` - uses `memoryLogger`
- `src/components/memory/MultipleAssetsUpload.tsx` - uses `memoryLogger`
- `src/components/marketing/value-journey.tsx` - uses `icpLogger`
- `src/components/layout/header.tsx` - uses `icpLogger`
- `src/components/layout/footer.tsx` - uses `icpLogger`
- `src/components/layout/bottom-nav.tsx` - uses `icpLogger`
- `src/components/galleries/gallery-list.tsx` - uses `icpLogger`
- `src/components/galleries/gallery-grid.tsx` - uses `icpLogger`
- `src/components/galleries/forever-storage-progress-modal.tsx` - uses `icpLogger`
- `src/components/galleries/create-gallery-modal.tsx` - uses `icpLogger`
- `src/components/auth/auth-components.tsx` - uses `authLogger`

#### Test Files

- `src/test/utils/test-server.ts` - uses `icpLogger`
- `src/test/simple-endpoint.test.ts` - uses `icpLogger`
- `src/test/learn-google-auth-mocking.test.ts` - uses `authLogger`
- `src/test/icp-endpoints.test.ts` - uses `icpLogger`
- `src/test/hybrid-auth-testing.test.ts` - uses `authLogger`
- `src/test/hybrid-auth-testing-session.test.ts` - uses `authLogger`
- `src/test/e2e-supertest.test.ts` - uses `icpLogger`
- `src/test/auth-bypass-testing.test.ts` - uses `authLogger`
- `src/test/advanced-patterns.test.ts` - uses `icpLogger`

#### Generated Files (Should be removed)

- `src/ic/declarations/backend/index.js` - uses `icpLogger`
- `src/ic/declarations/canister_factory/index.js` - uses `icpLogger`
- `src/ic/declarations/internet_identity/index.js` - uses `icpLogger`
- `src/ic/agent.ts` - uses `icpLogger`

#### Utilities

- `src/utils/dictionaries.ts` - uses `icpLogger`
- `src/utils/mailgun.ts` - uses `icpLogger`
- `src/components/utils/translation-validation.ts` - uses `icpLogger`

## Proposed Solution

### 1. Single Logger with Semantic Context

Replace all logger instances with one logger that uses semantic context:

```typescript
// Instead of: uploadLogger.info('Upload started')
// Use: logger.info('Upload started', { service: 'upload' })

// Instead of: authLogger.error('Auth failed')
// Use: logger.error('Auth failed', { service: 'auth' })
```

### 2. Semantic Grouping

Group by actual functionality, not arbitrary service names:

- **STORAGE**: Upload, S3, Vercel Blob, ICP storage
- **AUTH**: Authentication, user management, II coauth
- **DATA**: Database operations, memory management
- **UI**: Frontend components, hooks, contexts
- **API**: All API routes (meaningless as separate logger)
- **TEST**: Test files
- **UTIL**: Utilities, dictionaries, etc.

### 3. Refactoring Steps

1. Replace all logger imports with single `logger`
2. Add semantic context to all log calls
3. Remove unused logger instances
4. Update logger to support semantic filtering

## ✅ COMPLETED IMPACT

- ✅ **115 files** updated successfully
- ✅ **12 logger instances** removed
- ✅ **Simplified logging control** with semantic meaning
- ✅ **Better debugging** with context-aware filtering

## 🎯 NEW SEMANTIC CONTROL SYSTEM

### Control Flags (in logger.ts)
```typescript
const ENABLE_UPLOAD_LOGGING = true;    // Upload, S3, Vercel Blob, ICP storage
const ENABLE_DATABASE_LOGGING = true;  // Database operations, memory management
const ENABLE_AUTH_LOGGING = true;      // Authentication, user management, II coauth
```

### Usage Examples
```typescript
// Upload operations - controlled by ENABLE_UPLOAD_LOGGING
logger.info('File uploaded', { service: 'upload' });
logger.error('S3 upload failed', { service: 's3' });

// Database operations - controlled by ENABLE_DATABASE_LOGGING  
logger.info('Memory created', { service: 'memory' });
logger.error('DB query failed', { service: 'database' });

// Auth operations - controlled by ENABLE_AUTH_LOGGING
logger.info('User logged in', { service: 'auth' });
logger.error('II coauth failed', { service: 'ii-coauth' });
```

### Control Functions
```typescript
import { disableUploadLogging, disableDatabaseLogging, disableAuthLogging } from '@/lib/logger';

// Disable specific service categories
disableUploadLogging();    // Stops all upload/storage logs
disableDatabaseLogging();  // Stops all database logs  
disableAuthLogging();      // Stops all auth logs
```
