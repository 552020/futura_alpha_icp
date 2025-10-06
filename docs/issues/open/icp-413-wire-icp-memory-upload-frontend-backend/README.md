# ICP-413: Wire ICP Memory Upload Frontend-Backend

**Branch**: `icp-413-wire-icp-memory-upload-frontend-backend`  
**Created**: January 2025  
**Status**: In Progress

## Overview

This directory contains documentation and issues related to the current branch work on wiring ICP memory upload between frontend and backend.

## Current Work

### Backend Refactoring (Completed)

- ✅ UploadService refactored from OOP to functional approach
- ✅ Memory creation extracted from upload flow (pure blob upload)
- ✅ Multiple asset support implemented for memories
- ✅ Unit tests with decoupling pattern
- ✅ Fast integration tests for 2-asset workflows

### Frontend Integration (In Progress)

- ✅ ICP upload service implemented (`icp-upload.ts`, `icp-with-processing.ts`)
- ✅ Hosting preferences hook working
- ✅ Settings UI allows ICP selection
- ⚠️ **Hardcoded ICP preferences** in `use-file-upload.ts` (for testing)
- 🔄 **Next**: Remove hardcode and implement proper user preference routing

## Key Files

### Backend

- `src/backend/src/upload/service.rs` - Functional upload service
- `src/backend/src/memories/core/create.rs` - Multiple asset memory creation
- `src/backend/src/util/blob_id.rs` - Blob ID parsing utility

### Frontend

- `src/nextjs/src/hooks/use-file-upload.ts` - **Contains hardcoded ICP preferences**
- `src/nextjs/src/services/upload/icp-upload.ts` - ICP upload service
- `src/nextjs/src/services/upload/icp-with-processing.ts` - ICP processing service
- `src/nextjs/src/hooks/use-hosting-preferences.ts` - Hosting preferences

## Related Documentation

### Core Implementation

- `frontend-icp-upload.md` - Main implementation documentation (Oct 2, 2024)
- `frontend-icp-upload-integration.md` - Integration docs (Sep 16, 2024)
- `frontend-icp-e2e-upload-flow.md` - E2E upload flow documentation (Sep 16, 2024)
- `icp-backend-upload-flow.md` - Backend upload flow analysis (Sep 15, 2024)

### Architecture & Design

- `frontend-icp-2lane-4asset-integration.md` - 2-lane + 4-asset system implementation (Oct 3, 2024)
- `frontend-icp-upload-types.md` - Backend data structure comparison (Sep 29, 2024)
- `frontend-icp-upload-types-metadata-refactor.md` - Metadata refactoring architecture (Sep 29, 2024)

### Code Organization

- `frontend-icp-upload-routes-comparison.md` - Upload routes comparison and migration (Sep 29, 2024)
- `hardcode-removal-task.md` - Specific task for removing hardcoded preferences

## Next Steps

1. Remove hardcoded ICP preferences from `use-file-upload.ts`
2. Test user preference routing
3. Verify complete upload flow with user choice
4. Update documentation to reflect production-ready status
