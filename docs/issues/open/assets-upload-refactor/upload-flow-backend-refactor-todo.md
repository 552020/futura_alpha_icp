# Upload Flow Backend Refactor - TODO List

## 🎯 **Objective**

Refactor the backend upload system to separate blob upload from memory creation while maintaining the capsule-memory-asset hierarchy.

## 📋 **Phase 1: Foundation - Separate Upload from Memory Creation**

### **1.1 Independent Blob Upload API**

- [ ] **Create new blob upload endpoints**

  - [ ] `blobs_upload_to_capsule(capsule_id, file_data, metadata)` - Upload blob independently
  - [ ] `blobs_get(capsule_id, blob_id)` - Retrieve blob data
  - [ ] `blobs_delete(capsule_id, blob_id)` - Delete blob
  - [ ] `blobs_list(capsule_id)` - List blobs in capsule

- [ ] **Implement `BlobLocation` type**

  ```rust
  struct BlobLocation {
      blob_id: String,
      capsule_id: String,
      size: u64,
      checksum_sha256: [u8; 32],
      storage_path: String,
      created_at: u64,
  }
  ```

- [ ] **Add capsule access control**
  - [ ] Verify caller has write access to capsule for uploads
  - [ ] Verify caller has read access to capsule for retrievals
  - [ ] Test access control with different user scenarios

### **1.2 Independent Memory Creation**

- [ ] **Create new memory creation endpoints**

  - [ ] `memories_create(capsule_id, title, description, tags)` - Create empty memory
  - [ ] `memories_update(memory_id, updates)` - Update memory metadata
  - [ ] `memories_get(memory_id)` - Get memory details

- [ ] **Implement memory-blob linking**
  - [ ] `memories_link_blob(memory_id, blob_location)` - Link blob to memory
  - [ ] `memories_unlink_blob(memory_id, blob_id)` - Remove blob from memory
  - [ ] `memories_list_blobs(memory_id)` - List blobs in memory

### **1.3 Convenience Endpoints**

- [ ] **Create combined workflows**
  - [ ] `memories_create_with_upload(capsule_id, title, description, file_data, metadata)` - Upload and create memory in one call
  - [ ] `memories_add_blob(memory_id, file_data, metadata)` - Upload blob and link to existing memory

## 📋 **Phase 2: Memory Integration - Blob-Memory Linking**

### **2.1 Memory Structure Updates**

- [ ] **Update Memory type to reference blobs**

  - [ ] Add `blob_references: Vec<BlobLocation>` to Memory struct
  - [ ] Update memory serialization/deserialization
  - [ ] Maintain backward compatibility with existing memories

- [ ] **Create blob-memory relationship tracking**
  - [ ] Add `blob_to_memories` mapping for reverse lookups
  - [ ] Track which memories reference which blobs
  - [ ] Implement efficient blob cleanup when memories are deleted

### **2.2 Memory-Bob Management**

- [ ] **Update memory retrieval to include blobs**

  - [ ] Modify `memories_read()` to include blob data
  - [ ] Add `memories_get_blobs(memory_id)` endpoint
  - [ ] Implement efficient blob streaming for large files

- [ ] **Add blob-memory unlinking**
  - [ ] `memories_unlink_blob(memory_id, blob_id)` - Remove blob from memory
  - [ ] Handle blob cleanup when no memories reference it
  - [ ] Implement reference counting for blob lifecycle

## 📋 **Phase 3: Advanced Features**

### **3.1 Batch Operations**

- [ ] **Implement batch blob upload**
  - [ ] `blobs_upload_multiple(capsule_id, files)` - Upload multiple blobs
  - [ ] `memories_create_with_multiple_blobs(capsule_id, title, description, blob_locations)` - Create memory with multiple blobs

### **3.2 Blob Management**

- [ ] **Add blob deduplication**

  - [ ] Check for existing blobs with same content hash
  - [ ] Reuse existing blobs instead of creating duplicates
  - [ ] Update reference counting for deduplicated blobs

- [ ] **Add blob compression and optimization**
  - [ ] Implement blob compression for storage efficiency
  - [ ] Add blob format conversion (e.g., image resizing)
  - [ ] Implement blob caching for frequently accessed files

## 📋 **Phase 4: Cleanup and Optimization**

### **4.1 Code Cleanup**

- [ ] **Remove old API endpoints**

  - [ ] Remove `uploads_begin/put_chunk/finish` API
  - [ ] Remove old blob storage logic
  - [ ] Clean up unused code and dependencies

- [ ] **Optimize new implementation**
  - [ ] Remove any temporary code or workarounds
  - [ ] Optimize blob storage performance
  - [ ] Clean up error handling and logging

### **4.2 Frontend Integration**

- [ ] **Update frontend to use new API**

  - [ ] Modify upload service to use new blob endpoints
  - [ ] Update memory creation to use blob linking
  - [ ] Test end-to-end upload workflows

- [ ] **Add new frontend features**
  - [ ] Blob management interface
  - [ ] Memory-blob relationship visualization
  - [ ] Batch upload capabilities

### **4.3 Testing and Validation**

- [ ] **Comprehensive testing suite**

  - [ ] Unit tests for all new blob APIs
  - [ ] Integration tests for memory-blob linking
  - [ ] Performance tests for large file uploads
  - [ ] Security tests for capsule access control

- [ ] **Performance testing**
  - [ ] Test blob upload performance with large files
  - [ ] Test memory creation with multiple blobs
  - [ ] Test capsule-scoped storage efficiency

## 📋 **Phase 5: Future Enhancements (Deferred)**

### **5.1 Capsule-Scoped Blob Storage**

- [ ] **Implement capsule-scoped blob storage**
  - [ ] Update `STABLE_BLOB_STORE` to use capsule-scoped keys
  - [ ] Change key format from `(pmid_hash, chunk_idx)` to `(capsule_id, blob_id, chunk_idx)`
  - [ ] Update blob metadata to include capsule_id
  - [ ] Test storage isolation between capsules

### **5.2 Cross-Capsule Access**

- [ ] **Implement capsule "entering" mechanism**
  - [ ] Add cross-capsule blob access control
  - [ ] Implement capsule-to-capsule blob sharing
  - [ ] Add audit logging for cross-capsule access

### **5.3 Blob Lifecycle Management**

- [ ] **Implement blob cleanup policies**
  - [ ] Automatic cleanup of unreferenced blobs
  - [ ] Blob expiration and archival
  - [ ] Storage quota management per capsule

## 🎯 **Success Criteria**

### **Functional Requirements**

- [ ] ✅ Blobs can be uploaded independently of memories
- [ ] ✅ Memories can be created with existing blobs
- [ ] ✅ Blobs are properly isolated by capsule
- [ ] ✅ Cross-capsule access works with proper authorization
- [ ] ✅ New blob and memory APIs work correctly

### **Performance Requirements**

- [ ] ✅ Upload performance is maintained or improved
- [ ] ✅ Memory usage is optimized with capsule-scoped storage
- [ ] ✅ Blob retrieval is efficient for large files
- [ ] ✅ Batch operations are supported

### **Security Requirements**

- [ ] ✅ Capsule access control is properly enforced
- [ ] ✅ Blob data is isolated between capsules
- [ ] ✅ Cross-capsule access requires proper authorization
- [ ] ✅ All operations are properly audited

## 📝 **Notes**

### **Implementation Priority**

1. **Phase 1** - Foundation (separate upload from memory creation)
2. **Phase 2** - Memory integration (blob-memory linking)
3. **Phase 3** - Advanced features (batch operations, deduplication)
4. **Phase 4** - Cleanup and optimization
5. **Phase 5** - Future enhancements (capsule-scoped storage, cross-capsule access)

### **Risk Mitigation**

- [ ] **Test new implementation** thoroughly before deployment
- [ ] **Implement proper error handling** for all new APIs
- [ ] **Monitor performance** during development and testing
- [ ] **Validate capsule isolation** with comprehensive tests

### **Dependencies**

- [ ] **Backend refactoring** must be completed before frontend updates
- [ ] **Testing infrastructure** must be in place before development starts
- [ ] **Frontend integration** can begin once backend APIs are stable
