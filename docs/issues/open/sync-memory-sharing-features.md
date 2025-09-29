# Sync Memory Sharing Features Between ICP and Database Systems

## 🎯 **Objective**

Synchronize memory sharing functionality between the ICP backend (`MemoryAccess` enum) and the database system (`memoryShares` table) to ensure feature parity and seamless data flow.

## 📋 **Current State Analysis**

### **Database System (`memoryShares` table)**

- ✅ **Table-based sharing** with detailed audit trails
- ✅ **Secure access codes** for invitee authentication
- ✅ **Multiple sharing types**: user, group, relationship-based
- ✅ **Access levels**: read, write permissions
- ✅ **Audit trail**: creation timestamps, secure code tracking
- ✅ **Relationship-based sharing**: family, friends, etc.

### **ICP System (`MemoryAccess` enum)**

- ✅ **Embedded access control** in memory struct
- ✅ **Advanced time-based access**: scheduled revelation
- ✅ **Event-triggered access**: after death, birthdays, etc.
- ✅ **Custom access**: individuals and groups
- ✅ **Public/Private access**: basic access control
- ❌ **No audit trail** for sharing changes
- ❌ **No secure access codes** for invitees
- ❌ **No relationship-based sharing**

## 🔄 **Gap Analysis**

| Feature                    | Database | ICP | Gap          |
| -------------------------- | -------- | --- | ------------ |
| **Audit Trail**            | ✅       | ❌  | **CRITICAL** |
| **Secure Codes**           | ✅       | ❌  | **CRITICAL** |
| **Relationship Sharing**   | ✅       | ❌  | **HIGH**     |
| **Time-based Access**      | ❌       | ✅  | **MEDIUM**   |
| **Event-triggered Access** | ❌       | ✅  | **MEDIUM**   |
| **Group Sharing**          | ✅       | ✅  | **NONE**     |
| **Individual Sharing**     | ✅       | ✅  | **NONE**     |

## 🎯 **Proposed Solution**

### **Phase 1: Database → ICP Sync**

1. **Add audit trail to ICP**

   - Create `MemoryShareAudit` struct
   - Track sharing changes with timestamps
   - Store secure access codes

2. **Add relationship-based sharing**

   - Extend `MemoryAccess::Custom` to include relationship types
   - Map database relationship types to ICP enum

3. **Add secure access codes**
   - Include invitee codes in ICP memory struct
   - Validate codes during access attempts

### **Phase 2: ICP → Database Sync**

1. **Add time-based sharing to database**

   - Extend `memoryShares` table with time-based fields
   - Add scheduled access columns

2. **Add event-triggered sharing**
   - Create `memory_share_events` table
   - Track event triggers and access changes

### **Phase 3: Bidirectional Sync**

1. **Create sync service**
   - Sync sharing changes between systems
   - Handle conflicts and precedence rules
   - Maintain data consistency

## 📝 **Technical Implementation**

### **Database Schema Changes**

```sql
-- Add time-based sharing columns
ALTER TABLE memory_share ADD COLUMN accessible_after TIMESTAMP;
ALTER TABLE memory_share ADD COLUMN access_after_type TEXT; -- 'scheduled', 'event_triggered'

-- Add event-triggered sharing
CREATE TABLE memory_share_events (
  id TEXT PRIMARY KEY,
  memory_share_id TEXT REFERENCES memory_share(id),
  trigger_event TEXT NOT NULL, -- 'after_death', 'birthday', etc.
  trigger_value INTEGER, -- for events like 'birthday_18'
  created_at TIMESTAMP DEFAULT NOW()
);
```

### **ICP Type Changes**

```rust
// Add audit trail to Memory struct
pub struct Memory {
    // ... existing fields ...
    pub share_audit: Vec<MemoryShareAudit>,
    pub invitee_codes: HashMap<String, String>, // secure codes
}

// Add relationship-based sharing
pub enum MemoryAccess {
    // ... existing variants ...
    Custom {
        individuals: Vec<PersonRef>,
        groups: Vec<String>,
        relationships: Vec<RelationshipType>, // NEW
    },
}

// Add audit trail struct
pub struct MemoryShareAudit {
    pub shared_with: PersonRef,
    pub access_level: AccessLevel,
    pub shared_at: u64,
    pub shared_by: PersonRef,
    pub secure_code: Option<String>,
}
```

## 🧪 **Testing Strategy**

### **Unit Tests**

- [ ] Test database schema changes
- [ ] Test ICP type changes
- [ ] Test sharing logic in both systems

### **Integration Tests**

- [ ] Test bidirectional sync
- [ ] Test conflict resolution
- [ ] Test audit trail consistency

### **E2E Tests**

- [ ] Test sharing workflow end-to-end
- [ ] Test time-based access revelation
- [ ] Test event-triggered access

## 📊 **Success Criteria**

- [ ] **Feature Parity**: Both systems support all sharing features
- [ ] **Data Consistency**: Sharing changes sync between systems
- [ ] **Audit Trail**: Complete audit trail in both systems
- [ ] **Performance**: Sync operations complete within 5 seconds
- [ ] **Backward Compatibility**: Existing sharing data preserved

## 🚀 **Implementation Tasks**

### **Database Schema Updates**

- [ ] Add time-based sharing columns
- [ ] Create event-triggered sharing table
- [ ] Update database migration scripts

### **ICP Type Updates**

- [ ] Add audit trail to Memory struct
- [ ] Extend MemoryAccess enum
- [ ] Update memory creation/update logic

### **Sync Service**

- [ ] Create bidirectional sync service
- [ ] Implement conflict resolution
- [ ] Add error handling and retry logic

### **Testing & Validation**

- [ ] Comprehensive testing suite
- [ ] Performance optimization
- [ ] Documentation updates

## 🔗 **Related Issues**

- [ ] Frontend ICP Upload Implementation
- [ ] Memory Data Structure Synchronization
- [ ] Capsule-based Ownership Model

## 📚 **References**

- Database schema: `src/nextjs/src/db/schema.ts` (lines 868-895)
- ICP types: `src/backend/src/types.rs` (lines 635-654)
- Current implementation: `docs/issues/open/frontend-icp-upload-implementation.md`

---

**Priority**: High  
**Effort**: TBD  
**Dependencies**: None  
**Assignee**: TBD
