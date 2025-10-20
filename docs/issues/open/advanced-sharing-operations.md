# Advanced Sharing Operations - Missing Features

**Status**: Open  
**Priority**: Medium  
**Type**: Enhancement  
**Created**: 2025-01-20  
**Assignee**: TBD

## 📋 **Overview**

The current sharing service provides core functionality for user-to-user sharing and public link sharing. However, several advanced operations are missing that would enhance the sharing experience and provide enterprise-level features.

## 🎯 **Current Coverage**

### ✅ **Implemented Operations:**

- `createShare()` - Create user-to-user shares
- `revokeShare()` - Remove user access
- `getResourceShares()` - List all shares for a resource
- `checkResourceAccess()` - Check user permissions
- `createBulkShares()` - Share multiple resources at once
- `createPublicLink()` - Generate public share tokens
- `validatePublicToken()` - Validate public access
- `grantAccessViaToken()` - Grant access via public link

## 🚫 **Missing Advanced Operations**

### **1. Share Management Operations**

#### **Update Share Permissions**

```typescript
// Missing function
export async function updateSharePermissions(
  shareId: string,
  permissions: SharePermissions,
  updatedBy: string
): Promise<OperationResult<ShareRecord>>;
```

- **Use Case**: "Change John's access from view-only to edit"
- **Implementation**: Update `permMask` in `resourceMembership`
- **Validation**: Ensure updater has permission to modify share

#### **Transfer Ownership**

```typescript
// Missing function
export async function transferOwnership(
  resourceType: ShareableResourceType,
  resourceId: string,
  newOwnerId: string,
  transferredBy: string
): Promise<OperationResult<boolean>>;
```

- **Use Case**: "Transfer this memory to Sarah as the new owner"
- **Implementation**: Update owner in resource table + create new `resourceMembership`
- **Validation**: Only current owner can transfer

#### **Share Expiration**

```typescript
// Missing function
export async function createTimeLimitedShare(
  params: CreateShareParams & { expiresAt: Date }
): Promise<OperationResult<ShareRecord>>;
```

- **Use Case**: "Share this folder with John for 7 days only"
- **Implementation**: Add `expiresAt` field to `resourceMembership`
- **Cleanup**: Automatic cleanup of expired shares

### **2. Group Sharing Operations**

#### **Share with Groups**

```typescript
// Missing function
export async function shareWithGroup(
  resourceType: ShareableResourceType,
  resourceId: string,
  groupId: string,
  permissions: SharePermissions,
  invitedBy: string
): Promise<OperationResult<ShareRecord[]>>;
```

- **Use Case**: "Share this gallery with the 'Family' group"
- **Implementation**: Create individual shares for all group members
- **Features**: Bulk permission assignment

#### **Group Share Management**

```typescript
// Missing function
export async function getGroupShares(groupId: string): Promise<OperationResult<ShareRecord[]>>;
```

- **Use Case**: "Show me all resources shared with the 'Family' group"
- **Implementation**: Query shares by group membership

### **3. Share Notifications & Communication**

#### **Share Notifications**

```typescript
// Missing function
export async function notifyShareCreated(
  shareId: string,
  notificationType: "email" | "in_app" | "both"
): Promise<OperationResult<boolean>>;
```

- **Use Case**: "Send email notification when someone shares with me"
- **Implementation**: Integration with notification service
- **Features**: Customizable notification templates

#### **Share Messages**

```typescript
// Missing function
export async function addShareMessage(
  shareId: string,
  message: string,
  addedBy: string
): Promise<OperationResult<ShareMessage>>;
```

- **Use Case**: "Add a note when sharing: 'Check out these vacation photos!'"
- **Implementation**: New `share_messages` table

### **4. Share Analytics & Reporting**

#### **Share Statistics**

```typescript
// Missing function
export async function getShareStats(
  resourceType: ShareableResourceType,
  resourceId: string
): Promise<OperationResult<ShareStats>>;
```

- **Use Case**: "How many people have accessed this memory?"
- **Implementation**: Aggregate data from `resourceMembership`
- **Metrics**: View counts, access patterns, popular shares

#### **Share History**

```typescript
// Missing function
export async function getShareHistory(
  resourceType: ShareableResourceType,
  resourceId: string
): Promise<OperationResult<ShareHistoryEntry[]>>;
```

- **Use Case**: "Who shared this memory and when?"
- **Implementation**: Audit trail in `share_history` table
- **Features**: Timestamp, user, action, permissions

### **5. Advanced Access Control**

#### **Conditional Sharing**

```typescript
// Missing function
export async function createConditionalShare(
  params: CreateShareParams & {
    conditions: ShareCondition[];
  }
): Promise<OperationResult<ShareRecord>>;
```

- **Use Case**: "Share only if user is in 'Family' group"
- **Implementation**: Conditional logic in access checking
- **Features**: Role-based, time-based, location-based conditions

#### **Share Approval Workflow**

```typescript
// Missing function
export async function createPendingShare(params: CreateShareParams): Promise<OperationResult<PendingShare>>;
```

- **Use Case**: "Request access to this private memory"
- **Implementation**: `pending_shares` table with approval workflow
- **Features**: Owner approval, auto-approval rules

### **6. Share Templates & Presets**

#### **Share Templates**

```typescript
// Missing function
export async function createShareTemplate(
  name: string,
  permissions: SharePermissions,
  createdBy: string
): Promise<OperationResult<ShareTemplate>>;
```

- **Use Case**: "Create 'Family View' template with read-only access"
- **Implementation**: `share_templates` table
- **Features**: Reusable permission sets

#### **Quick Share Presets**

```typescript
// Missing function
export async function quickShareWithPreset(
  resourceType: ShareableResourceType,
  resourceId: string,
  presetName: string,
  targetUserId: string,
  invitedBy: string
): Promise<OperationResult<ShareRecord>>;
```

- **Use Case**: "Quick share with 'Family' preset"
- **Implementation**: Apply template permissions
- **Features**: One-click sharing with predefined settings

## 🗄️ **Database Schema Additions Needed**

### **New Tables:**

```sql
-- Share messages
CREATE TABLE share_messages (
  id UUID PRIMARY KEY,
  share_id UUID REFERENCES resource_membership(id),
  message TEXT,
  added_by TEXT,
  created_at TIMESTAMP DEFAULT NOW()
);

-- Share history
CREATE TABLE share_history (
  id UUID PRIMARY KEY,
  resource_type resource_type_t,
  resource_id TEXT,
  action TEXT, -- 'created', 'updated', 'revoked'
  user_id TEXT,
  details JSONB,
  created_at TIMESTAMP DEFAULT NOW()
);

-- Share templates
CREATE TABLE share_templates (
  id UUID PRIMARY KEY,
  name TEXT NOT NULL,
  permissions JSONB,
  created_by TEXT,
  created_at TIMESTAMP DEFAULT NOW()
);

-- Pending shares
CREATE TABLE pending_shares (
  id UUID PRIMARY KEY,
  resource_type resource_type_t,
  resource_id TEXT,
  requested_by TEXT,
  status TEXT, -- 'pending', 'approved', 'rejected'
  created_at TIMESTAMP DEFAULT NOW()
);
```

### **Schema Updates:**

```sql
-- Add expiration to resource_membership
ALTER TABLE resource_membership
ADD COLUMN expires_at TIMESTAMP;

-- Add conditions to resource_membership
ALTER TABLE resource_membership
ADD COLUMN conditions JSONB;
```

## 🎯 **Implementation Priority**

### **Phase 1 (High Priority):**

1. **Update Share Permissions** - Essential for share management
2. **Transfer Ownership** - Core ownership management
3. **Share Expiration** - Time-limited sharing

### **Phase 2 (Medium Priority):**

4. **Group Sharing** - Bulk sharing capabilities
5. **Share Notifications** - User experience enhancement
6. **Share Statistics** - Basic analytics

### **Phase 3 (Low Priority):**

7. **Share Templates** - Advanced user experience
8. **Conditional Sharing** - Enterprise features
9. **Share Approval Workflow** - Advanced access control

## 🧪 **Testing Requirements**

### **Unit Tests:**

- Permission validation
- Expiration logic
- Group membership handling
- Notification delivery

### **Integration Tests:**

- End-to-end sharing workflows
- Cross-resource type compatibility
- Performance with large groups

### **User Acceptance Tests:**

- Share creation and management UI
- Notification delivery
- Permission changes

## 📊 **Success Metrics**

- **Functionality**: All advanced operations working correctly
- **Performance**: Share operations complete in <500ms
- **User Experience**: Intuitive share management interface
- **Security**: Proper access control and permission validation

## 🔗 **Related Issues**

- [Memory Sharing Modal Implementation](./memory-sharing-modal-implementation.md)
- [Folder Storage Badge Missing](./folder-storage-badge-missing-storage-status.md)

## 📝 **Notes**

- Consider implementing these features incrementally
- Each feature should be backward compatible
- Focus on user experience and security
- Consider enterprise requirements for advanced features

---

**Next Steps:**

1. Review and prioritize missing operations
2. Create implementation plan for Phase 1 features
3. Design database schema additions
4. Begin implementation of high-priority operations


