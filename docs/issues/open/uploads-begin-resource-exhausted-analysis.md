# uploads_begin ResourceExhausted Error - Detailed Analysis

**Priority**: High  
**Type**: Technical Analysis  
**Created**: 2025-01-01  
**Status**: Analysis Complete

## 🎯 Function Analysis: `uploads_begin`

### **Function Location**

- **File**: `src/backend/src/upload/service.rs:26-85`
- **Entry Point**: `src/backend/src/lib.rs:446-456`
- **Error Location**: Line 62-64 in `upload/service.rs`

### **Function Signature**

```rust
pub fn begin_upload(
    &mut self,
    store: &mut Store,
    capsule_id: CapsuleId,
    asset_metadata: AssetMetadata,
    expected_chunks: u32,
    idem: String,
) -> std::result::Result<SessionId, Error>
```

## 🔍 ResourceExhausted Error Conditions

### **Exact Error Location**

```rust
// 3) back-pressure: cap concurrent sessions per caller/capsule
const MAX_ACTIVE_PER_CALLER: usize = 10; // Increased for MVP development
if self.sessions.count_active_for(&capsule_id, &caller) >= MAX_ACTIVE_PER_CALLER {
    return Err(Error::ResourceExhausted); // "too many active uploads"
}
```

### **When ResourceExhausted is Returned**

**Condition**: `self.sessions.count_active_for(&capsule_id, &caller) >= 10`

**What `count_active_for` Does:**

```rust
pub fn count_active_for(&self, capsule_id: &CapsuleId, caller: &candid::Principal) -> usize {
    STABLE_UPLOAD_SESSIONS.with(|sessions| {
        sessions
            .borrow()
            .iter()
            .filter(|(_, session)| {
                &session.capsule_id == capsule_id
                    && &session.caller == caller
                    && matches!(session.status, crate::upload::types::SessionStatus::Pending)
            })
            .count()
    })
}
```

**Key Points:**

1. **Filters by**: Same `capsule_id` AND same `caller` (Principal)
2. **Status Filter**: Only counts sessions with `SessionStatus::Pending`
3. **Returns**: Count of matching sessions
4. **Threshold**: When count >= 10, throws `ResourceExhausted`

## 📊 Session Lifecycle Analysis

### **Session Status States**

```rust
pub enum SessionStatus {
    Pending,                    // ← Counted as "active"
    Committed { blob_id: u64 }, // ← NOT counted as "active"
}
```

### **Session Creation Process**

1. **Check Existing**: Look for pending session with same (capsule, caller, idem)
2. **Count Active**: Count pending sessions for (capsule, caller)
3. **Check Limit**: If count >= 10, throw ResourceExhausted
4. **Create Session**: Create new session with `SessionStatus::Pending`

### **Session Cleanup Process**

**Sessions are cleaned up in these scenarios:**

1. **Successful Upload**: `uploads_finish` → session status becomes `Committed`
2. **Failed Upload**: `uploads_abort` → session is removed
3. **Manual Cleanup**: `sessions.cleanup()` → session and chunks removed

## 🚨 Problem Analysis

### **Root Cause: Session Accumulation**

**What's Happening:**

1. **Previous Test Runs**: Multiple test runs created sessions
2. **Incomplete Cleanup**: Sessions not properly cleaned up
3. **Status Stuck**: Sessions remain in `Pending` status
4. **Count Accumulation**: Session count reaches/exceeds 10
5. **New Uploads Blocked**: All new `uploads_begin` calls fail

### **Evidence from Code Analysis**

**Session Store Structure:**

```rust
// Sessions stored in stable memory
static STABLE_UPLOAD_SESSIONS: RefCell<StableBTreeMap<u64, SessionMeta, Memory>>
```

**Session Cleanup Method:**

```rust
pub fn cleanup(&self, session_id: &SessionId) {
    // Remove session metadata
    STABLE_UPLOAD_SESSIONS.with(|sessions| {
        sessions.borrow_mut().remove(&session_id.0);
    });
    // Remove all chunks for this session
    // ... chunk cleanup logic
}
```

**Critical Issue**: Sessions are only cleaned up when:

1. `uploads_finish` is called successfully
2. `uploads_abort` is called explicitly
3. Manual cleanup is performed

**Missing**: No automatic session expiry or cleanup mechanism

## 🔍 Test Evidence Analysis

### **Test Pattern**

```
❌ Lane A: Original Upload: Upload begin: Resource exhausted: Upload failed: {"ResourceExhausted":null}
```

### **Files Tested (All Failed)**

- ✅ `avocado_big_21mb.jpg` (20.8 MB) → ResourceExhausted
- ✅ `avocado_medium_3.5mb.jpg` (3.5 MB) → ResourceExhausted
- ✅ `avocado_small_372kb.jpg` (372 KB) → ResourceExhausted

### **Key Observations**

1. **Size Independent**: Error occurs regardless of file size
2. **Consistent Error**: Same error for all file sizes
3. **Early Failure**: Error occurs at `uploads_begin`, not during chunk upload
4. **Session Limit**: Clearly hitting the 10-session limit

## 💡 Technical Solutions

### **Immediate Solutions**

#### **1. Clear Session Store (Quick Fix)**

```rust
// Add method to clear all sessions (development only)
pub fn clear_all_sessions(&self) {
    STABLE_UPLOAD_SESSIONS.with(|sessions| {
        sessions.borrow_mut().clear();
    });
}
```

#### **2. Increase Session Limit (Temporary)**

```rust
const MAX_ACTIVE_PER_CALLER: usize = 100; // Temporary increase for development
```

#### **3. Add Session Expiry (Proper Fix)**

```rust
// Add session expiry check
const SESSION_EXPIRY_MS: u64 = 30 * 60 * 1000; // 30 minutes

pub fn cleanup_expired_sessions(&self) {
    let now = ic_cdk::api::time();
    // Remove sessions older than SESSION_EXPIRY_MS
}
```

### **Long-term Solutions**

#### **1. Automatic Session Cleanup**

- Add session expiry mechanism
- Implement background cleanup task
- Add session monitoring/logging

#### **2. Better Session Management**

- Implement session heartbeat
- Add session status monitoring
- Improve error handling

#### **3. Development Tools**

- Add session debugging endpoints
- Implement session store inspection
- Add session cleanup utilities

## 🔧 Implementation Recommendations

### **For Senior Developer**

#### **Immediate Action (5 minutes)**

1. **Check Session Count**: Add logging to see current session count
2. **Clear Sessions**: Implement session store clearing
3. **Test Upload**: Verify upload works after cleanup

#### **Short-term Fix (30 minutes)**

1. **Add Session Expiry**: Implement automatic session cleanup
2. **Increase Limit**: Temporarily increase session limit
3. **Add Monitoring**: Add session count logging

#### **Long-term Solution (2 hours)**

1. **Session Lifecycle**: Implement proper session lifecycle management
2. **Cleanup Strategy**: Add comprehensive cleanup strategy
3. **Error Handling**: Improve error handling and recovery

## 📋 Debugging Steps

### **1. Check Current Session Count**

```rust
// Add to uploads_begin for debugging
let active_count = self.sessions.count_active_for(&capsule_id, &caller);
ic_cdk::println!("Active sessions for caller {}: {}", caller, active_count);
```

### **2. List All Sessions**

```rust
// Add method to list all sessions
pub fn list_all_sessions(&self) -> Vec<SessionMeta> {
    STABLE_UPLOAD_SESSIONS.with(|sessions| {
        sessions.borrow().iter().map(|(_, session)| session.clone()).collect()
    })
}
```

### **3. Force Session Cleanup**

```rust
// Add method to force cleanup all sessions
pub fn force_cleanup_all(&self) {
    // Implementation to remove all sessions
}
```

## 🎯 Success Criteria

- [ ] `uploads_begin` succeeds for all file sizes
- [ ] Session count stays below limit
- [ ] Sessions are properly cleaned up
- [ ] Upload system fully functional
- [ ] No ResourceExhausted errors

## 📚 Related Files

- **Upload Service**: `src/backend/src/upload/service.rs:26-85`
- **Session Store**: `src/backend/src/upload/sessions.rs:158-171`
- **Session Types**: `src/backend/src/upload/types.rs:98-103`
- **Backend Lib**: `src/backend/src/lib.rs:446-456`

---

**Last Updated**: 2025-01-01  
**Status**: Ready for Senior Developer Implementation
