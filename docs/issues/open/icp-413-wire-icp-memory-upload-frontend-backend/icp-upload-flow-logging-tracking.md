# ICP Upload Flow Logging & Tracking Issue

**Priority**: High  
**Type**: Feature Enhancement  
**Status**: Open  
**Created**: 2025-01-16  
**Related**: ICP upload system, Database switching, Logger system

## **Issue Summary**

Implement comprehensive logging and tracking for the complete ICP upload flow to enable debugging and monitoring from upload button click to memory rendering. This will help track the flow when users switch to ICP-only mode (database=ICP, backend=ICP, blob=ICP).

## **Current State**

- ✅ ICP upload system is implemented with 2-lane + 4-asset processing
- ✅ Database switching toggle is functional
- ✅ Hosting preferences system is complete
- ✅ Logger system is configured but needs ICP-specific tagging
- ❌ **Missing**: Comprehensive flow tracking for ICP uploads

## **Objective**

Create a complete audit trail for ICP uploads with tagged logging at every critical decision point and processing step.

## **Critical Flow Points to Track**

### **1. Upload Initiation & Routing**

- **Location**: `src/hooks/use-file-upload.ts`
- **Trigger**: Upload button click
- **Key Decisions**:
  - Hosting preferences check
  - ICP authentication verification
  - Upload service routing decision
- **Tags**: `['icp:upload:init', 'icp:routing', 'icp:auth']`

### **2. File Processing & Lane Assignment**

- **Location**: `src/services/upload/single-file-processor.ts`
- **Trigger**: File selection and processing start
- **Key Decisions**:
  - File type detection
  - Lane A vs Lane B assignment
  - ICP service selection
- **Tags**: `['icp:processing', 'icp:lanes', 'icp:file-type']`

### **3. ICP Canister Interaction**

- **Location**: `src/services/upload/icp-with-processing.ts`
- **Trigger**: ICP upload service calls
- **Key Decisions**:
  - Capsule creation/retrieval
  - Upload session management
  - Chunked upload progress
  - Memory creation
- **Tags**: `['icp:canister', 'icp:session', 'icp:chunks', 'icp:memory']`

### **4. Database Integration**

- **Location**: `src/services/upload/finalize.ts`
- **Trigger**: Asset finalization
- **Key Decisions**:
  - Neon database record creation
  - ICP memory edge creation
  - Dual storage linking
- **Tags**: `['icp:database', 'icp:edges', 'icp:dual-storage']`

### **5. Dashboard Rendering**

- **Location**: `src/app/[lang]/dashboard/page.tsx`
- **Trigger**: Memory display after upload
- **Key Decisions**:
  - Data source selection (ICP vs Neon)
  - Memory transformation
  - UI rendering
- **Tags**: `['icp:dashboard', 'icp:rendering', 'icp:display']`

## **Proposed Logging Strategy**

### **Log Levels & Context**

- **Debug**: Detailed flow information, decision points
- **Info**: Major milestones, successful operations
- **Warn**: Fallbacks, non-critical issues
- **Error**: Failures, critical issues

### **Tag Structure**

```typescript
// Primary tags for ICP flow
const ICP_TAGS = {
  UPLOAD_INIT: "icp:upload:init",
  ROUTING: "icp:routing",
  AUTH: "icp:auth",
  PROCESSING: "icp:processing",
  LANES: "icp:lanes",
  CANISTER: "icp:canister",
  SESSION: "icp:session",
  CHUNKS: "icp:chunks",
  MEMORY: "icp:memory",
  DATABASE: "icp:database",
  EDGES: "icp:edges",
  DUAL_STORAGE: "icp:dual-storage",
  DASHBOARD: "icp:dashboard",
  RENDERING: "icp:rendering",
  DISPLAY: "icp:display",
};
```

### **Key Metrics to Track**

1. **Upload Success Rate**: Successful uploads vs failures
2. **Processing Time**: Time from click to completion
3. **Lane Performance**: Lane A vs Lane B completion times
4. **Database Sync**: ICP-Neon synchronization success
5. **Rendering Performance**: Dashboard display time

## **Implementation Plan**

### **Phase 1: Core Upload Flow**

- Add logging to `use-file-upload.ts` for routing decisions
- Add logging to `single-file-processor.ts` for processing steps
- Add logging to `icp-with-processing.ts` for canister interactions

### **Phase 2: Database Integration**

- Add logging to `finalize.ts` for database operations
- Add logging to `createICPMemoryEdge` for edge creation
- Add logging for dual storage verification

### **Phase 3: Dashboard Integration**

- Add logging to dashboard page for data source selection
- Add logging to memory transformation functions
- Add logging for rendering performance

### **Phase 4: Error Handling & Monitoring**

- Add comprehensive error logging with context
- Add performance metrics collection
- Add flow completion tracking

## **Expected Log Flow Example**

```
[DEBUG] [icp:upload:init] Upload button clicked, checking hosting preferences
[INFO] [icp:routing] Hosting preferences: {blobHosting: ['icp'], databaseHosting: ['icp']}
[DEBUG] [icp:auth] Checking ICP authentication status
[INFO] [icp:auth] ICP authentication successful
[DEBUG] [icp:processing] File selected: image.jpg (2.5MB)
[INFO] [icp:lanes] Starting 2-lane processing: Lane A (original) + Lane B (derivatives)
[DEBUG] [icp:canister] Creating/retrieving capsule for user
[INFO] [icp:session] Upload session created: session_123
[DEBUG] [icp:chunks] Uploading chunk 1/5 (512KB)
[INFO] [icp:memory] Memory created in ICP canister: memory_456
[DEBUG] [icp:database] Creating Neon database record
[INFO] [icp:edges] Creating ICP memory edge: memory_456 -> neon_789
[DEBUG] [icp:dual-storage] Dual storage verification successful
[INFO] [icp:dashboard] Switching to ICP data source
[DEBUG] [icp:rendering] Rendering memories from ICP canister
[INFO] [icp:display] Memory displayed successfully in dashboard
```

## **Success Criteria**

- [ ] Complete audit trail from upload click to memory display
- [ ] All critical decision points are logged with appropriate tags
- [ ] Performance metrics are collected and trackable
- [ ] Error scenarios are properly logged with context
- [ ] Flow can be easily debugged and monitored
- [ ] Logs are structured and searchable by tags

## **Files to Modify**

### **Frontend Files**

- `src/hooks/use-file-upload.ts`
- `src/services/upload/single-file-processor.ts`
- `src/services/upload/icp-with-processing.ts`
- `src/services/upload/finalize.ts`
- `src/app/[lang]/dashboard/page.tsx`
- `src/services/memories.ts`

### **Logger Configuration**

- `src/lib/logger/fat-logger/index.ts` (enable ICP tags)
- `src/lib/logger/fat-logger/types.ts` (add ICP tag constants)

## **Testing Scenarios**

1. **Happy Path**: Complete ICP upload flow with all steps successful
2. **Authentication Failure**: ICP auth fails, fallback behavior
3. **Upload Failure**: Chunked upload fails, retry logic
4. **Database Sync Failure**: ICP success but Neon failure
5. **Rendering Failure**: Upload success but dashboard display fails

## **Priority**

**High** - This is essential for debugging and monitoring the ICP upload system in production.
