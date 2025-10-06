# Upload Flow Refactor Analysis

## 🎯 **Objective**

Separate blob upload from memory creation while maintaining the existing upload infrastructure.

## 📋 **Current Upload Flow**

### **Existing Functions (Keep These)**

```rust
// Current upload flow (coupled)
uploads_begin(capsule_id, asset_metadata, expected_chunks, idem) -> session_id
uploads_put_chunk(session_id, chunk_idx, bytes) -> ()
uploads_finish(session_id, expected_sha256, total_len) -> (blob_id, memory_id)  // ← PROBLEM: Creates memory
```

### **The Problem**

The current `uploads_finish()` function **automatically creates a memory** and returns both `blob_id` and `memory_id`. This couples blob upload to memory creation.

## 💡 **Proposed Solution**

### **Option 1: Modify Existing Functions (Recommended)**

```rust
// Modified upload flow (decoupled)
uploads_begin(capsule_id, asset_metadata, expected_chunks, idem) -> session_id
uploads_put_chunk(session_id, chunk_idx, bytes) -> ()
uploads_finish(session_id, expected_sha256, total_len) -> blob_id  // ← CHANGE: Only return blob_id

// New memory creation endpoints
memories_create(capsule_id, title, description, tags) -> memory_id
memories_link_blob(memory_id, blob_id) -> ()
```

### **Option 2: Keep Current + Add New**

```rust
// Keep current for convenience (deprecated)
uploads_begin/put_chunk/finish -> (blob_id, memory_id)  // existing

// Add new decoupled flow
blobs_upload_to_capsule(capsule_id, file_data, metadata) -> blob_id  // new
memories_create(capsule_id, title, description) -> memory_id  // new
memories_link_blob(memory_id, blob_id) -> ()  // new
```

## 🔧 **Implementation Plan**

### **Phase 1: Modify Existing Functions**

- [ ] **Modify `uploads_finish()`**

  - [ ] Remove memory creation logic
  - [ ] Return only `blob_id` instead of `(blob_id, memory_id)`
  - [ ] Update return type in `lib.rs`

- [ ] **Add memory creation endpoints**

  - [ ] `memories_create(capsule_id, title, description, tags)` - Create empty memory
  - [ ] `memories_update(memory_id, updates)` - Update memory metadata
  - [ ] `memories_get(memory_id)` - Get memory details

- [ ] **Add blob-memory linking**
  - [ ] `memories_link_blob(memory_id, blob_id)` - Link blob to memory
  - [ ] `memories_unlink_blob(memory_id, blob_id)` - Remove blob from memory
  - [ ] `memories_list_blobs(memory_id)` - List blobs in memory

### **Phase 2: Convenience Endpoints**

- [ ] **Add combined workflows**
  - [ ] `memories_create_with_upload(capsule_id, title, description, file_data, metadata)` - Upload and create memory in one call
  - [ ] `memories_add_blob(memory_id, file_data, metadata)` - Upload blob and link to existing memory

## 🎯 **Benefits of This Approach**

### **Separation of Concerns**

- ✅ **Blob upload** is independent and reusable
- ✅ **Memory creation** is independent of upload
- ✅ **Linking** is explicit and controllable

### **Flexibility**

- ✅ **Upload blobs** without creating memories
- ✅ **Create memories** with existing blobs
- ✅ **Reuse blobs** across multiple memories
- ✅ **Batch operations** for efficiency

### **Minimal Changes**

- ✅ **Keep existing upload infrastructure**
- ✅ **Modify only `uploads_finish()`**
- ✅ **Add new memory management endpoints**
- ✅ **No duplicate functionality**

## 🤔 **Key Questions**

1. **Should we modify `uploads_finish()`** to remove memory creation?
2. **Or should we keep it and add new endpoints** for the decoupled flow?
3. **How do we handle the transition** for existing frontend code?

## 📝 **Next Steps**

1. **Decide on approach** (modify existing vs. add new)
2. **Implement memory creation endpoints**
3. **Implement blob-memory linking**
4. **Update frontend to use new flow**
5. **Remove old coupled behavior**

## 🚫 **What NOT to Do**

- ❌ **Don't create new blob upload endpoints** - we already have `uploads_begin/put_chunk/finish`
- ❌ **Don't duplicate functionality** - use existing upload infrastructure
- ❌ **Don't break existing API** without a clear migration path
- ❌ **Don't over-engineer** - focus on the core separation of concerns

