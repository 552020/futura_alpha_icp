# Upload Workflow Integration with Capsule Storage Architecture

## Issue Description

We need senior guidance on how to integrate the existing upload workflow with our new single-storage capsule architecture. Currently we have parallel storage systems that need to be unified.

## Current Architecture Analysis

### 🏗️ **What We Have: Dual Storage Systems**

#### 1. **Capsule Storage (Target Architecture)**

```rust
STABLE_CAPSULES
└── Capsule {
    memories: HashMap<String, Memory> {
        memory_id: Memory {
            data: MemoryData {
                data: Option<Vec<u8>>,  // Inline data storage
                blob_ref: BlobRef       // Reference/locator
            }
        }
    }
}
```

#### 2. **Upload Storage (Current Implementation)**

```rust
STABLE_UPLOAD_SESSIONS  → UploadSession (temporary)
STABLE_CHUNK_DATA      → ChunkData (temporary)
STABLE_MEMORY_ARTIFACTS → MemoryArtifact (permanent)
```

### 📋 **Current Functions Overview**

#### **Memory Creation Functions**

- `memories_create(capsule_id, memory_data)` → Stores in `STABLE_CAPSULES`
- `add_memory_to_capsule()` → Deprecated, stores in `STABLE_CAPSULES`

#### **Upload Functions**

- `begin_asset_upload()` → Creates session in `STABLE_UPLOAD_SESSIONS`
- `put_chunk()` → Stores chunks in `STABLE_CHUNK_DATA`
- `commit_asset()` → Assembles chunks, stores in `STABLE_MEMORY_ARTIFACTS`
- `cancel_upload()` → Cleans up session and chunks

#### **Metadata Functions**

- `upsert_metadata()` → Stores metadata in `STABLE_MEMORY_ARTIFACTS`
- `memories_ping()` → Checks presence in `STABLE_MEMORY_ARTIFACTS`

#### **Gallery Sync Functions**

- `sync_gallery_memories()` → Uses upload + metadata workflow

## Strategic Questions for Senior Developer

### 1. **Architecture Direction: Single vs Multi-Storage**

**Option A: Unified Capsule Storage (Our Target)**

```rust
// Everything flows through CAPSULES only:
upload_to_memory(capsule_id, file_data)
└── with_store(|store| {
    store.update(&capsule_id, |capsule| {
        capsule.memories.insert(memory_id, Memory {
            data: MemoryData { data: Some(file_bytes) }  // All inline
        })
    })
})
```

**Option B: Hybrid Storage (Current State)**

```rust
// Separate systems for different purposes:
upload_chunks() → STABLE_MEMORY_ARTIFACTS  // Large files
memories_create() → STABLE_CAPSULES        // Memory records
```

**Question**: Should we consolidate to Option A (single storage) or maintain Option B (hybrid)?

### 2. **Upload Session Management: Temporary vs Persistent**

**Current Approach**:

- Upload sessions are temporary (cleaned up after commit)
- Chunks are temporary (cleaned up after assembly)
- Final artifacts are permanent

**Questions**:

- **Should upload sessions remain temporary** or be integrated into capsule storage?
- **Is the chunk-based upload still needed** if we move to direct capsule storage?
- **What's the max file size** we should support for inline storage in capsules?

### 3. **Data Flow Patterns: In-Memory vs Persistent**

**Current Pattern**:

```rust
begin_asset_upload() → put_chunk() × N → commit_asset() → [separate artifact storage]
                                                        ↓
memories_create() → [references artifact] → capsule.memories
```

**Proposed Pattern**:

```rust
upload_to_capsule() → [direct to capsule.memories with inline data]
```

**Questions**:

- **Should uploads bypass temporary storage** and go directly to capsules?
- **How do we handle upload failures** without temporary staging?
- **Is chunked upload still necessary** for the capsule model?

### 4. **Function Migration Strategy**

Given our [Capsule Storage Foundation Plan](./capsule-storage-foundation-plan.md), how should these functions evolve?

#### **Functions to Consolidate**:

- `memories_create()` + upload workflow → Single memory creation flow?
- `upsert_metadata()` + capsule memory storage → Unified metadata handling?
- `sync_gallery_memories()` → Use capsule storage directly?

#### **Functions to Remove**:

- Separate artifact storage functions?
- Temporary session management?
- Duplicate metadata storage?

### 5. **Performance and Storage Considerations**

**Upload Characteristics**:

- **Supposed to happen quickly** (short-lived operations)
- **End up in stable capsule storage** (permanent destination)
- **May involve large files** (images, videos, documents)

**Questions**:

- **Memory limits**: What's the practical size limit for inline storage in capsules?
- **Performance impact**: Does storing large files inline affect capsule operations?
- **Cleanup strategy**: How do we handle partial uploads or failures?

## Proposed Migration Approach

### **Option 1: Direct Integration (Aggressive)**

1. **Remove**: `STABLE_UPLOAD_SESSIONS`, `STABLE_CHUNK_DATA`, `STABLE_MEMORY_ARTIFACTS`
2. **Modify**: `memories_create()` to handle chunked uploads directly
3. **Update**: All upload functions to use capsule storage only
4. **Timeline**: 1-2 weeks, higher risk

### **Option 2: Gradual Migration (Conservative)**

1. **Keep**: Current upload infrastructure temporarily
2. **Add**: New direct-to-capsule upload functions
3. **Migrate**: Gradually move endpoints to new pattern
4. **Remove**: Old infrastructure after migration complete
5. **Timeline**: 3-4 weeks, lower risk

### **Option 3: Hybrid Persistence (Middle Ground)**

1. **Keep**: Upload sessions for large files
2. **Direct**: Small files go straight to capsules
3. **Threshold**: Size-based routing (e.g., <1MB direct, >1MB chunked)
4. **Timeline**: 2-3 weeks, balanced approach

## Implementation Questions

### **File Size Thresholds**

- What's the cutoff for inline vs external storage?
- How do we handle files that exceed capsule storage limits?

### **Error Handling**

- How do we rollback partial uploads in the direct-to-capsule model?
- What happens if capsule update fails during upload?

### **Backward Compatibility**

- Do existing artifacts need migration to capsule storage?
- Should we maintain old endpoints during transition?

### **Testing Strategy**

- How do we test large file uploads with the new architecture?
- What's the testing strategy for upload failures and edge cases?

## Current State Analysis

### **What Works Well**:

✅ Chunked upload handles large files efficiently  
✅ Temporary storage provides failure isolation  
✅ Clear separation between upload and memory creation

### **What's Problematic**:

❌ Duplicate storage systems (artifacts + capsules)  
❌ Complex data flow across multiple storage layers  
❌ Separate metadata tracking in parallel systems  
❌ Upload workflow not aligned with capsule architecture

### **What's Unclear**:

🤔 Optimal file size limits for inline storage  
🤔 Performance impact of large files in capsules  
🤔 Migration path for existing artifacts  
🤔 Error handling in direct-to-capsule uploads

## Request for Senior Developer

### **Primary Ask**:

**Choose the integration approach** that aligns with our capsule storage foundation and provide guidance on:

1. **Architecture decision**: Single storage vs hybrid approach
2. **File size limits**: What's practical for inline capsule storage
3. **Migration strategy**: Aggressive, gradual, or hybrid approach
4. **Function consolidation**: Which functions to merge/remove/keep
5. **Error handling**: Best practices for upload failure scenarios

### **Secondary Ask**:

**Implementation guidance** on:

- Specific code patterns for direct capsule uploads
- Testing approach for large file scenarios
- Performance optimization strategies
- Backward compatibility requirements

## Success Criteria

### **Architecture Alignment**:

- [ ] Upload workflow fully integrated with capsule storage
- [ ] Single source of truth for all memory data
- [ ] Clean separation between temporary and permanent storage

### **Performance**:

- [ ] No degradation in upload performance
- [ ] Efficient handling of both small and large files
- [ ] Minimal memory overhead during uploads

### **Maintainability**:

- [ ] Simplified codebase with fewer storage systems
- [ ] Clear data flow from upload to permanent storage
- [ ] Consistent error handling across all upload scenarios

---

**Status**: 🔴 BLOCKED - Awaiting Senior Developer Architecture Decision  
**Priority**: HIGH - Critical for capsule storage foundation  
**Assignee**: Senior Developer (Architecture Decision) + Team (Implementation)  
**Created**: Current Session  
**Updated**: Current Session

**Related Issues**:

- [Capsule Storage Foundation Plan](./capsule-storage-foundation-plan.md)
- [Stable Memory Migration API Compatibility](./stable-memory-migration-api-compatibility.md)
