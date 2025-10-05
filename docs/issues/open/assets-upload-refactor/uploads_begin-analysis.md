# `uploads_begin` Function Analysis

## 📋 **Function Signature**

### **API Wrapper** (`src/backend/src/lib.rs:472-494`)

```rust
#[ic_cdk::update]
fn uploads_begin(
    capsule_id: types::CapsuleId,
    asset_metadata: types::AssetMetadata,
    expected_chunks: u32,
    idem: String,
) -> Result_13 {
    match with_capsule_store_mut(|store| {
        let mut upload_service = upload::service::UploadService::new();
        upload_service.begin_upload(store, capsule_id, asset_metadata, expected_chunks, idem)
    }) {
        Ok(session_id) => {
            let sid = session_id.0;
            // Initialize rolling hash for this session
            UPLOAD_HASH.with(|m| {
                m.borrow_mut().insert(sid, Sha256::new());
            });
            ic_cdk::println!("UPLOAD_HASH_INIT sid={}", sid);
            Result_13::Ok(sid)
        }
        Err(error) => Result_13::Err(error),
    }
}
```

### **Implementation** (`src/backend/src/upload/service.rs:45-128`)

```rust
pub fn begin_upload(
    &mut self,
    store: &mut Store,
    capsule_id: CapsuleId,
    asset_metadata: AssetMetadata,
    expected_chunks: u32,
    idem: String,
) -> std::result::Result<SessionId, Error> {
    // Implementation details below...
}
```

## 🔍 **Function Analysis**

### **Purpose**

Initiates a new upload session for chunked file uploads to a specific capsule.

### **Parameters**

- **`capsule_id`**: Target capsule for the upload
- **`asset_metadata`**: Metadata about the asset being uploaded
- **`expected_chunks`**: Number of chunks the file will be split into
- **`idem`**: Idempotency key to prevent duplicate uploads

### **Return Value**

- **Success**: `SessionId` - Unique identifier for the upload session
- **Failure**: `Error` - Various error types (validation, auth, resource limits)

## 🛠️ **Implementation Details**

### **1. Input Validation**

```rust
// Validate expected_chunks
if expected_chunks == 0 {
    return Err(Error::InvalidArgument("expected_chunks_zero".into()));
}
const MAX_CHUNKS: u32 = 16_384;
if expected_chunks > MAX_CHUNKS {
    return Err(Error::InvalidArgument("expected_chunks_too_large".into()));
}
```

### **2. Authentication & Authorization**

```rust
let caller = ic_cdk::api::msg_caller();
let person_ref = PersonRef::Principal(caller);
if let Some(capsule) = store.get(&capsule_id) {
    if !capsule.has_write_access(&person_ref) {
        return Err(Error::Unauthorized);
    }
} else {
    return Err(Error::NotFound);
}
```

### **3. Idempotency Check**

```rust
// Check if pending session with same (capsule, caller, idem) exists
if let Some(existing) = with_session_compat(|sessions|
    sessions.find_pending(&capsule_id, &caller, &idem)
) {
    return Ok(existing);
}
```

### **4. Session Cleanup**

```rust
const SESSION_EXPIRY_MS: u64 = 2 * 60 * 60 * 1000; // 2 hours
with_session_compat(|sessions| {
    sessions.cleanup_expired_sessions_for_caller(&capsule_id, &caller, SESSION_EXPIRY_MS)
});
```

### **5. Resource Limits (Back-pressure)**

```rust
const MAX_ACTIVE_PER_CALLER: usize = 100;
let active_count = with_session_compat(|sessions|
    sessions.count_active_for(&capsule_id, &caller)
);

if active_count >= MAX_ACTIVE_PER_CALLER {
    return Err(Error::ResourceExhausted);
}
```

### **6. Session Creation**

```rust
let session_id = SessionId::new();
let provisional_memory_id = MemoryId::new();

let upload_meta = crate::session::UploadSessionMeta {
    session_id: session_id.0,
    capsule_id,
    caller,
    created_at: ic_cdk::api::time(),
    expected_chunks,
    status: SessionStatus::Pending,
    chunk_count: expected_chunks,
    asset_metadata,
    provisional_memory_id: provisional_memory_id.to_string(),
    chunk_size: crate::upload::types::CHUNK_SIZE,
    idem: idem.clone(),
    blob_id: None, // No blob ID yet (pending)
};

with_session_compat(|sessions| sessions.create(session_id.clone(), upload_meta))?;
```

### **7. Hash Initialization** (in lib.rs wrapper)

```rust
// Initialize rolling hash for this session
UPLOAD_HASH.with(|m| {
    m.borrow_mut().insert(sid, Sha256::new());
});
```

## 🎯 **Key Observations**

### **✅ What Works Well**

- **Robust validation** - Input sanitization and limits
- **Security** - Proper authentication and authorization
- **Idempotency** - Prevents duplicate uploads
- **Resource management** - Session limits and cleanup
- **Monitoring** - Logging for debugging

### **🔍 Critical Details**

- **Creates `provisional_memory_id`** - This is where the coupling happens!
- **Session status is `Pending`** - Not committed until `uploads_finish`
- **No blob ID yet** - Blob is created during the upload process
- **Hash tracking** - SHA256 rolling hash for integrity

### **⚠️ Potential Issues**

- **Memory coupling** - Creates `provisional_memory_id` even though memory shouldn't exist yet
- **Session complexity** - Lots of state management
- **Resource limits** - May need tuning for production

## 🔄 **Flow Context**

### **Before `uploads_begin`**

- User has a file to upload
- User knows the target capsule
- User has calculated expected chunks

### **After `uploads_begin`**

- Session is created and tracked
- Hash computation is initialized
- Ready for `uploads_put_chunk` calls

### **What Happens Next**

1. **`uploads_put_chunk`** - Upload individual chunks
2. **`uploads_finish`** - Complete upload and create memory

## 💡 **Refactoring Implications**

### **Current Coupling**

The function creates a `provisional_memory_id` which suggests the system is already thinking about memory creation during upload initiation.

### **Decoupling Strategy**

- **Keep session creation** - This is pure upload logic
- **Remove `provisional_memory_id`** - Don't create memory IDs during upload
- **Focus on blob storage** - Session should only track blob upload progress

### **Minimal Changes Needed**

- Remove `provisional_memory_id` from `UploadSessionMeta`
- Update session tracking to focus on blob upload only
- Memory creation becomes a separate concern

## 📝 **Summary**

The `uploads_begin` function is well-designed for upload session management but contains the seeds of the coupling problem (provisional memory ID). The core upload logic is solid and should be preserved, but the memory-related assumptions need to be removed for proper decoupling.
