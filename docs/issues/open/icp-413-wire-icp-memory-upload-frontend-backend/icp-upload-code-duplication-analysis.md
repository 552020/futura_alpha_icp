# ICP Upload Code Duplication Analysis

## Overview

Both `uploadFileToICP` and `uploadToICPWithProcessing` contain significant code duplication, making maintenance difficult and creating inconsistencies.

## Major Duplicated Code Blocks

### 1. Authentication & Actor Creation

**uploadFileToICP (icp-upload.ts:464):**

```typescript
const { backendActor } = await import("@/ic/backend");
const actor = (await backendActor()) as CanisterActor;
```

**uploadToICPWithProcessing (icp-with-processing.ts:437-447):**

```typescript
const { getAuthClient } = await import("@/ic/ii");
const { backendActor } = await import("@/ic/backend");
const authClient = await getAuthClient();
const identity = authClient.getIdentity();
const backend = await backendActor(identity);
```

**Duplication:** Both create backend actors, but with different authentication patterns.

### 2. Capsule Management

**uploadFileToICP (icp-upload.ts:77-103):**

```typescript
async function getOrCreateCapsuleId(actor: CanisterActor): Promise<string> {
  const capsuleResult = await actor.capsules_read_basic([]);
  if ("Ok" in capsuleResult && capsuleResult.Ok) {
    return capsuleResult.Ok.capsule_id;
  }
  const createResult = await actor.capsules_create([]);
  // ... error handling
}
```

**uploadToICPWithProcessing (icp-with-processing.ts:450-470):**

```typescript
const capsuleResult = (await backend.capsules_read_basic([])) as Result_4;
if ("Ok" in capsuleResult && capsuleResult.Ok) {
  capsuleId = capsuleResult.Ok.capsule_id;
} else {
  const createResult = (await backend.capsules_create([])) as Result_3;
  // ... error handling
}
```

**Duplication:** Identical capsule creation logic with different variable names and error handling.

### 3. Image Processing

**uploadFileToICP (icp-upload.ts:117-140):**

```typescript
async function processImageDerivativesForICP(file: File): Promise<ProcessedBlobs> {
  console.log("🖼️ Starting Lane B image processing for ICP", file.name, file.size, file.type);
  const processedBlobs = await processImageDerivativesPure(file);
  // ... logging
}
```

**uploadToICPWithProcessing (icp-with-processing.ts:107):**

```typescript
laneBPromise = processImageDerivativesPure(file).then((processedBlobs) =>
  uploadProcessedAssetsToICP(processedBlobs, file.name)
);
```

**Duplication:** Both use `processImageDerivativesPure()` but with different wrapper functions.

### 4. Derivative Upload Logic

**uploadFileToICP (icp-upload.ts:145-270):**

```typescript
async function uploadProcessedAssetsToICP(
  processedBlobs: ProcessedBlobs,
  originalFileName: string,
  actor: CanisterActor,
  capsuleId: string
): Promise<ProcessedAssets> {
  // Upload display, thumb, placeholder derivatives
}
```

**uploadToICPWithProcessing (icp-with-processing.ts:314-414):**

```typescript
export async function uploadProcessedAssetsToICP(
  processedBlobs: ProcessedBlobs,
  originalFileName: string
): Promise<ProcessedAssets> {
  // Upload display, thumb, placeholder derivatives
}
```

**Duplication:** Nearly identical derivative upload logic with different function signatures.

### 5. Database Integration

**uploadFileToICP (icp-upload.ts:272-400):**

```typescript
async function createNeonDatabaseRecord(
  file: File,
  icpMemoryId: string,
  derivativesResult?: ProcessedAssets
): Promise<{ memoryId: string; assetId: string }> {
  // Format 2 database creation
}
```

**uploadToICPWithProcessing (icp-with-processing.ts:50-60):**

```typescript
const commitResponse = await fetch("/api/upload/complete", {
  method: "POST",
  body: JSON.stringify({
    fileKey: `icp-${Date.now()}-${file.name}`,
    originalName: file.name,
    size: file.size,
    type: file.type,
  }),
});
```

**Duplication:** Both create database records but with different approaches (Format 2 vs Format 3).

### 6. Logging Patterns

**uploadFileToICP:**

```typescript
console.log("🚀 Starting ICP file upload", { fileName, fileSize, mimeType });
console.log("📤 Starting derivative uploads to ICP", { fileName });
console.log("✅ Successfully created Neon database record");
```

**uploadToICPWithProcessing:**

```typescript
logger.info(`🔄 ICP upload started: ${file.name} (${file.size} bytes)`);
logger.info("🔍 Getting capsule...");
logger.info(`✅ Using existing capsule: ${capsuleId}`);
```

**Duplication:** Both have extensive logging but with different formats (console.log vs logger.info).

## Impact of Duplication

### 1. Maintenance Burden

- Bug fixes need to be applied to multiple functions
- Feature additions require changes in multiple places
- Inconsistent behavior between functions

### 2. Code Quality Issues

- Violates DRY (Don't Repeat Yourself) principle
- Increases codebase size unnecessarily
- Makes testing more complex

### 3. Developer Confusion

- Two similar functions with different names
- Unclear which function to use when
- Different error handling patterns

### 4. Performance Impact

- Duplicate code increases bundle size
- Multiple implementations of same logic
- Potential for different optimizations

## Specific Duplicated Functions

### Core Upload Logic

- `getOrCreateCapsuleId()` vs inline capsule creation
- `processImageDerivativesForICP()` vs `processImageDerivativesPure()`
- `uploadProcessedAssetsToICP()` (two different implementations)
- `createNeonDatabaseRecord()` vs Format 2/3 database calls

### Authentication Patterns

- `backendActor()` vs `getAuthClient() + backendActor()`
- Identity management
- Error handling for authentication failures

### Logging Systems

- Console.log vs logger.info
- Different log formats and levels
- Inconsistent debugging information

## Recommendations

### 1. Extract Common Functions

```typescript
// Shared authentication
async function getAuthenticatedBackend(): Promise<CanisterActor>;

// Shared capsule management
async function getOrCreateCapsule(actor: CanisterActor): Promise<string>;

// Shared derivative processing
async function processAndUploadDerivatives(
  file: File,
  actor: CanisterActor,
  capsuleId: string
): Promise<ProcessedAssets>;
```

### 2. Unify Logging

```typescript
// Single logging interface
interface ICPUploadLogger {
  startUpload(file: File): void;
  processDerivatives(file: File): void;
  createDatabaseRecord(memoryId: string): void;
}
```

### 3. Consolidate Database Integration

```typescript
// Single database integration function
async function createDatabaseRecord(
  file: File,
  icpMemoryId: string,
  derivatives: ProcessedAssets,
  format: "format2" | "format3"
): Promise<{ memoryId: string; assetId: string }>;
```

### 4. Create Unified Upload Function

```typescript
// Single upload function with options
async function uploadToICP(
  file: File,
  options: {
    useParallelProcessing: boolean;
    useFinalizeAssets: boolean;
    loggingLevel: "console" | "logger";
  }
): Promise<UploadServiceResult>;
```

## Files with Duplication

- `src/nextjs/src/services/upload/icp-upload.ts` (uploadFileToICP)
- `src/nextjs/src/services/upload/icp-with-processing.ts` (uploadToICPWithProcessing)
- `src/nextjs/src/services/upload/image-derivatives.ts` (shared but duplicated usage)
- `src/nextjs/src/services/upload/shared-utils.ts` (authentication helpers)

## Priority

**High** - Code duplication is a major maintenance issue that should be addressed to:

- Reduce codebase complexity
- Improve maintainability
- Ensure consistent behavior
- Simplify debugging and testing


