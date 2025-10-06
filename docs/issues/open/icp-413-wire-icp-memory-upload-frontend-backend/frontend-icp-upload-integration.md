# Frontend ICP Upload Integration

## Issue Summary

**Status**: Open  
**Priority**: High  
**Type**: Feature Implementation  
**Assignee**: TBD

The frontend currently lacks direct integration with the ICP canister for uploads. When users select "ICP only" storage preference, files are still uploaded to Neon/Vercel Blob instead of directly to the ICP canister.

## Problem Description

### Current State

- ✅ ICP canister has working upload endpoints (`uploads_begin`, `uploads_put_chunk`, `uploads_finish`)
- ✅ Frontend has ICP agent utilities (`src/ic/agent.ts`, `src/ic/actor-factory.ts`)
- ✅ Upload intent/verify endpoints are implemented
- ❌ **Missing**: Direct frontend → ICP canister upload logic

### Current Upload Flow

```
Frontend → Next.js API → Neon/Vercel Blob (regardless of storage preference)
```

### Intended Upload Flow

```
Frontend → ICP Canister (direct) → /api/upload/verify (record in control plane)
```

## Root Cause Analysis

The `user-file-upload.ts` hook only calls Next.js API endpoints and doesn't implement the direct ICP upload path:

```typescript
// Current code in user-file-upload.ts
const endpoint = isOnboarding ? '/api/memories/upload/onboarding/folder' : '/api/memories/upload/folder';
const response = await fetch(endpoint, {
  method: 'POST',
  body: formData,
});
```

**Missing Logic**: When `intent.chosen_backend === "icp-canister"`, the frontend should:

1. Create ICP agent with user's Internet Identity
2. Call `uploads_begin()` on canister
3. Upload chunks via `uploads_put_chunk()`
4. Call `uploads_finish()` to commit
5. Call `/api/upload/verify` to record in control plane

## Technical Requirements

### 1. ICP Upload Service Implementation

Create a new service to handle direct ICP uploads:

**File**: `src/nextjs/src/services/icp-upload.ts`

```typescript
interface ICPUploadService {
  uploadFile(file: File, intent: UploadIntent): Promise<UploadResult>;
  uploadFolder(files: File[], intent: UploadIntent): Promise<UploadResult[]>;
}

interface UploadResult {
  memoryId: string;
  size: number;
  checksum_sha256: string;
  remote_id: string;
}
```

### 2. Internet Identity Integration

**Requirements**:

- Use existing Internet Identity authentication
- Create authenticated agent for canister calls
- Handle authentication errors gracefully

**Dependencies**:

- `@dfinity/agent` (already installed)
- `@dfinity/auth-client` (check if installed)
- User's Internet Identity principal

### 3. Upload Flow Implementation

**For Single Files**:

```typescript
async function uploadToICP(file: File, intent: UploadIntent): Promise<UploadResult> {
  // 1. Create authenticated agent
  const agent = await createAuthenticatedAgent();
  const actor = makeActor(backendIdlFactory, intent.icp.canister_id, agent);

  // 2. Calculate file size and determine upload strategy
  const fileSize = file.size;
  const isInline = fileSize <= intent.limits.inline_max;

  if (isInline) {
    // 3a. Inline upload (≤32KB)
    const fileBytes = await file.arrayBuffer();
    const memoryId = await actor.memories_create(capsuleId, {
      Inline: {
        data: Array.from(new Uint8Array(fileBytes)),
        metadata: {
          /* file metadata */
        },
        idempotency_key: intent.idem,
      },
    });
    return { memoryId, size: fileSize, checksum_sha256: null, remote_id: memoryId };
  } else {
    // 3b. Chunked upload (>32KB)
    const sessionId = await actor.uploads_begin(capsuleId, metadata, expectedChunks, intent.idem);

    // Upload chunks
    const chunkSize = intent.limits.chunk_size;
    for (let i = 0; i < expectedChunks; i++) {
      const start = i * chunkSize;
      const end = Math.min(start + chunkSize, fileSize);
      const chunk = await file.slice(start, end).arrayBuffer();
      await actor.uploads_put_chunk(sessionId, i, Array.from(new Uint8Array(chunk)));
    }

    // Finish upload
    const memoryId = await actor.uploads_finish(sessionId, expectedHash, fileSize);
    return { memoryId, size: fileSize, checksum_sha256: expectedHash, remote_id: memoryId };
  }
}
```

**For Folder Uploads**:

- Process each file individually
- Handle progress tracking across multiple files
- Aggregate results for verification

### 4. Error Handling

**ICP-Specific Errors**:

- Authentication failures (Internet Identity not connected)
- Canister communication errors
- Upload session timeouts
- Chunk upload failures
- Verification failures

**Fallback Strategy**:

- If ICP upload fails, fall back to Neon with user notification
- Preserve user's original intent for retry

### 5. Progress Tracking

**Requirements**:

- Real-time upload progress for chunked uploads
- Progress aggregation for folder uploads
- Cancellation support for long-running uploads

## Implementation Plan

### Phase 1: Core ICP Upload Service

- [ ] **1.1** Create `ICPUploadService` class
- [ ] **1.2** Implement authenticated agent creation
- [ ] **1.3** Implement inline upload path (≤32KB)
- [ ] **1.4** Implement chunked upload path (>32KB)
- [ ] **1.5** Add comprehensive error handling
- [ ] **1.6** Add progress tracking and cancellation

### Phase 2: Integration with Existing Upload Hook

- [ ] **2.1** Modify `user-file-upload.ts` to detect ICP backend
- [ ] **2.2** Route to ICP service when `chosen_backend === "icp-canister"`
- [ ] **2.3** Maintain existing Neon upload path
- [ ] **2.4** Add fallback logic for ICP failures
- [ ] **2.5** Update progress tracking for both backends

### Phase 3: Testing and Validation

- [ ] **3.1** Test single file uploads to ICP
- [ ] **3.2** Test folder uploads to ICP
- [ ] **3.3** Test error scenarios and fallbacks
- [ ] **3.4** Test progress tracking and cancellation
- [ ] **3.5** Test with different file sizes (inline vs chunked)

### Phase 4: User Experience Enhancements

- [ ] **4.1** Add ICP-specific upload progress indicators
- [ ] **4.2** Add clear error messages for ICP failures
- [ ] **4.3** Add retry mechanisms for failed uploads
- [ ] **4.4** Add upload cancellation support

## Technical Considerations

### Authentication

- **Challenge**: Ensure user is authenticated with Internet Identity
- **Solution**: Check authentication state before attempting ICP upload
- **Fallback**: Redirect to ICP authentication if not connected

### File Size Limits

- **Inline Limit**: 32KB (enforced by canister)
- **Chunk Size**: 64KB (configurable via intent)
- **Max Chunks**: 512 (configurable via intent)
- **Total Limit**: ~32MB (512 × 64KB)

### Network Reliability

- **Challenge**: ICP network can be slower/less reliable than Web2
- **Solution**: Implement retry logic with exponential backoff
- **Fallback**: Auto-fallback to Neon after N retries

### Progress Tracking

- **Challenge**: Chunked uploads need granular progress
- **Solution**: Track progress per chunk and aggregate
- **UI**: Show both individual file and overall folder progress

## Dependencies

### Required Packages

- `@dfinity/agent` ✅ (already installed)
- `@dfinity/auth-client` ❓ (check if installed)
- `@dfinity/candid` ❓ (for type definitions)

### Environment Variables

- `NEXT_PUBLIC_ICP_CANISTER_ID` ✅ (already configured)
- `NEXT_PUBLIC_ICP_NETWORK` ✅ (already configured)
- `NEXT_PUBLIC_IC_HOST` ✅ (already configured)

### Canister Dependencies

- Backend canister with upload endpoints ✅ (already implemented)
- Internet Identity canister ✅ (already configured)

## Acceptance Criteria

### Functional Requirements

- [ ] Users can upload single files to ICP when "ICP only" is selected
- [ ] Users can upload folders to ICP when "ICP only" is selected
- [ ] Upload progress is tracked and displayed accurately
- [ ] Failed uploads fall back to Neon with user notification
- [ ] Upload verification works correctly with control plane

### Non-Functional Requirements

- [ ] Upload performance is acceptable for typical file sizes
- [ ] Error messages are clear and actionable
- [ ] Upload cancellation works reliably
- [ ] Memory usage is reasonable for large files
- [ ] Network timeouts are handled gracefully

### Edge Cases

- [ ] Very large files (>32MB) are handled appropriately
- [ ] Network interruptions during upload are recovered
- [ ] Multiple concurrent uploads work correctly
- [ ] Browser refresh during upload is handled
- [ ] Authentication expires during upload is handled

## Risk Assessment

### High Risk

- **ICP Network Reliability**: ICP can be slower/less reliable than Web2
- **Authentication Complexity**: Internet Identity integration adds complexity
- **File Size Limits**: Large files may hit canister limits

### Medium Risk

- **Progress Tracking**: Chunked uploads need careful progress management
- **Error Handling**: ICP errors may be less user-friendly than Web2
- **Browser Compatibility**: Some browsers may have issues with large file uploads

### Low Risk

- **Agent Creation**: Well-established pattern in existing codebase
- **Canister Integration**: Upload endpoints are already implemented and tested

## Success Metrics

- [ ] 95%+ success rate for ICP uploads under normal conditions
- [ ] <5% fallback rate to Neon due to ICP failures
- [ ] Upload progress accuracy within 5% of actual progress
- [ ] User satisfaction with ICP upload experience
- [ ] No significant performance degradation compared to Neon uploads

## Related Issues

- [Storage Backend Selection Feature](../storage-backend-selection-feature.md)
- [Storage Backend Selection Feature Phase 2](../storage-backend-selection-feature-phase-2.md)
- [Bug: Storage Toggles Mutation Error Feedback](../bug-storage-toggles-mutation-error-feedback.md)

## Implementation Notes

### Code Organization

- Keep ICP upload logic separate from existing Neon upload logic
- Use dependency injection for easier testing
- Maintain backward compatibility with existing upload flows

### Testing Strategy

- Unit tests for ICP upload service
- Integration tests with mock canister
- E2E tests with real ICP network (staging)
- Performance tests with various file sizes

### Documentation

- Update API documentation for new upload flows
- Add troubleshooting guide for ICP upload issues
- Document configuration requirements for ICP uploads

---

**Created**: 2024-12-19  
**Last Updated**: 2024-12-19  
**Estimated Effort**: 2-3 weeks  
**Complexity**: High
