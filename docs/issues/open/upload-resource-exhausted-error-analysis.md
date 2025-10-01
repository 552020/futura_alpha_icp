# Upload ResourceExhausted Error Analysis

**Priority**: High  
**Type**: Bug Analysis  
**Assigned To**: Senior Developer  
**Created**: 2025-01-01  
**Status**: Analysis Complete - Ready for Senior Review

## 🎯 Issue Summary

**Problem**: All file uploads (regardless of size) are failing with `ResourceExhausted` error during the `uploads_begin` call.

**Impact**: Complete upload system failure - no files can be uploaded to ICP backend.

## 🔍 Technical Analysis

### **Failing Function**

- **Function**: `uploads_begin` in `src/backend/src/lib.rs:446-456`
- **Backend Service**: `src/backend/src/upload/service.rs:26-85`
- **Error Location**: Line 63 in `upload/service.rs`

### **Root Cause Identified**

The ResourceExhausted error is triggered by this specific condition in `upload/service.rs:62-64`:

```rust
const MAX_ACTIVE_PER_CALLER: usize = 10; // Increased for MVP development
if self.sessions.count_active_for(&capsule_id, &caller) >= MAX_ACTIVE_PER_CALLER {
    return Err(Error::ResourceExhausted); // "too many active uploads"
}
```

### **Error Analysis**

**What's Happening:**

1. Test calls `uploads_begin()` with valid parameters
2. Backend checks active sessions for the caller/capsule combination
3. `sessions.count_active_for()` returns >= 10
4. Backend throws `ResourceExhausted` error
5. Test receives `{"ResourceExhausted":null}` response

**Why This Is Happening:**

- **Session Cleanup Issue**: Previous upload sessions are not being properly cleaned up
- **Session Store State**: The session store contains stale/abandoned sessions
- **Development Testing**: Multiple test runs have created accumulated sessions
- **No Session Expiry**: Sessions may not have automatic cleanup/expiry

## 📊 Test Evidence

### **Files Tested (All Failed with Same Error):**

- ✅ `avocado_big_21mb.jpg` (20.8 MB) → ResourceExhausted
- ✅ `avocado_medium_3.5mb.jpg` (3.5 MB) → ResourceExhausted
- ✅ `avocado_small_372kb.jpg` (372 KB) → ResourceExhausted

### **Error Pattern:**

```
❌ Lane A: Original Upload: Upload begin: Resource exhausted: Upload failed: {"ResourceExhausted":null}
```

### **Successful Components:**

- ✅ **Lane B (Image Processing)**: Working perfectly for all file sizes
- ✅ **Helper Functions**: All working correctly
- ✅ **Network Connectivity**: Canister communication working
- ✅ **Response Handling**: Fixed and working

## 🔧 Backend Code Analysis

### **uploads_begin Function Signature:**

```rust
fn uploads_begin(
    capsule_id: types::CapsuleId,
    asset_metadata: types::AssetMetadata,
    expected_chunks: u32,
    idem: String,
) -> std::result::Result<upload::types::SessionId, Error>
```

### **Session Management Logic:**

```rust
// 3) back-pressure: cap concurrent sessions per caller/capsule
const MAX_ACTIVE_PER_CALLER: usize = 10; // Increased for MVP development
if self.sessions.count_active_for(&capsule_id, &caller) >= MAX_ACTIVE_PER_CALLER {
    return Err(Error::ResourceExhausted); // "too many active uploads"
}
```

### **Session Store Methods:**

- `sessions.count_active_for(&capsule_id, &caller)` - Counts active sessions
- `sessions.create(session_id, session_meta)` - Creates new session
- `sessions.find_pending(&capsule_id, &caller, &idem)` - Finds existing sessions

## 🚨 Immediate Issues

### **1. Session Cleanup Problem**

- **Issue**: Sessions are not being cleaned up after upload completion/failure
- **Evidence**: Even 372KB files fail, suggesting accumulated sessions
- **Impact**: Complete upload system unusable

### **2. Session Store State**

- **Issue**: Session store contains stale sessions from previous test runs
- **Evidence**: Multiple test runs have created accumulated sessions
- **Impact**: New uploads blocked by old sessions

### **3. No Session Expiry**

- **Issue**: Sessions don't have automatic expiry/cleanup
- **Evidence**: Sessions persist indefinitely
- **Impact**: Resource exhaustion over time

## 💡 Recommended Solutions

### **Immediate Fix (Senior Developer)**

1. **Clear Session Store**: Reset/clear the session store state
2. **Add Session Cleanup**: Implement session expiry/cleanup
3. **Increase Limit Temporarily**: Increase `MAX_ACTIVE_PER_CALLER` for development
4. **Add Session Management**: Implement proper session lifecycle

### **Long-term Solutions**

1. **Session Expiry**: Add automatic session cleanup after timeout
2. **Session Monitoring**: Add logging/monitoring for session counts
3. **Graceful Cleanup**: Implement proper session cleanup on upload completion
4. **Resource Management**: Better resource management for development

## 🔍 Investigation Questions for Senior

1. **Session Store State**: What is the current state of the session store?
2. **Session Cleanup**: How should sessions be cleaned up after upload completion?
3. **Session Expiry**: Should sessions have automatic expiry timeouts?
4. **Development Limits**: Should `MAX_ACTIVE_PER_CALLER` be higher for development?
5. **Session Monitoring**: How can we monitor session counts and cleanup?

## 📋 Next Steps

### **For Senior Developer:**

1. **Investigate Session Store**: Check current session store state
2. **Implement Cleanup**: Add session cleanup mechanism
3. **Test Upload System**: Verify upload system works after cleanup
4. **Monitor Sessions**: Add session monitoring/logging

### **For Development Team:**

1. **Wait for Fix**: Hold on testing until session issue resolved
2. **Prepare Test Cases**: Prepare comprehensive test cases for after fix
3. **Document Process**: Document session management process

## 🎯 Success Criteria

- [ ] `uploads_begin` calls succeed for all file sizes
- [ ] Session store properly manages session lifecycle
- [ ] No ResourceExhausted errors for normal uploads
- [ ] Upload system fully functional for 2-lane + 4-asset testing

## 📚 Related Files

- **Backend Upload Service**: `src/backend/src/upload/service.rs`
- **Backend Lib**: `src/backend/src/lib.rs:446-456`
- **Session Store**: `src/backend/src/upload/sessions.rs`
- **Test Script**: `tests/backend/shared-capsule/upload/test_upload_2lane_4asset_system.mjs`
- **Helper Functions**: `tests/backend/shared-capsule/upload/helpers.mjs`

## 🔗 References

- [Upload Chunk Size Optimization Issue](./upload-chunk-size-optimization-issue.md) - ✅ Resolved
- [2-Lane + 4-Asset Implementation Issue](./implement-2lane-4asset-icp-system.md) - Current
- [Backend Memory Allocation Memo](../backend-memory-allocation-memo.md)

---

**Last Updated**: 2025-01-01  
**Next Review**: After senior developer investigation  
**Status**: Ready for Senior Review
