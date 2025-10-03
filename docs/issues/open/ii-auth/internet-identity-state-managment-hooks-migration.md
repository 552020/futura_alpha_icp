# Old Hooks Migration Analysis

## 📋 **Issue Summary**

**Status**: 🔍 **ANALYSIS** - Analysis of components using old `useIICoAuth()` hooks that need migration to `useIILinks()`

**Goal**: Identify all components using old hooks and analyze what needs to be migrated.

## 🎯 **Files Using Old Hooks**

### **1. Test File - DELETE**

- **File**: `src/nextjs/src/hooks/__tests__/use-ii-coauth.test.ts`
- **Action**: ✅ **DELETE** - No migration needed
- **Reason**: Test file for old hook, can be removed

### **2. II Co-Auth Controls + Linked Accounts - MERGE & MIGRATE**

- **Files**:
  - `src/nextjs/src/components/user/ii-coauth-controls.tsx`
  - `src/nextjs/src/components/user/linked-accounts.tsx`
- **Current Usage**:
  ```typescript
  // Both components use similar patterns:
  const {
    hasLinkedII,
    isCoAuthActive, // ❌ REMOVE
    activeIcPrincipal, // ❌ REMOVE
    linkedIcPrincipal, // ❌ SINGLE → MULTIPLE
    statusMessage, // ❌ REMOVE
    statusClass, // ❌ REMOVE
    remainingMinutes, // ❌ REMOVE
    disconnectII, // ❌ REMOVE
    refreshTTL, // ❌ REMOVE
    isExpired, // ❌ REMOVE
    requiresReAuth, // ❌ REMOVE
  } = useIICoAuth();
  ```
- **Migration Required**:
  - **Merge both components** into single "Internet Identity Management" component
  - Replace `useIICoAuth()` with `useICPIdentity()` + `useIILinks()`
  - Remove TTL-related UI elements (progress bars, countdowns)
  - Remove "Activate" and "Disconnect" buttons
  - **Link happens automatically on sign-in** - no "Link new" button needed
  - Add "Unlink" functionality only
  - **Show active principal** (currently signed in with II) - same as avatar
  - Show list of `linkedIcPrincipals` (all linked principals)
  - Combine functionality: active principal + linked principals + unlink actions
- **Priority**: 🔴 **HIGH** - Core II management component (merged)

### **3. ICP Card - MIGRATE**

- **File**: `src/nextjs/src/components/user/icp-card.tsx`
- **Usage**: Used in ICP page (`/user/icp`) - shows ICP status and controls
- **Current Usage**:
  ```typescript
  const {
    hasLinkedII,
    isCoAuthActive, // ❌ REMOVE
    activeIcPrincipal, // ❌ REMOVE
    statusMessage, // ❌ REMOVE
    statusClass, // ❌ REMOVE
    remainingMinutes, // ❌ REMOVE
    disconnectII, // ❌ REMOVE
    refreshTTL, // ❌ REMOVE
  } = useIICoAuth();
  ```
- **Migration Required**:
  - Remove TTL and active state logic
  - Remove disconnect/refresh functionality
  - Show linked principals list instead of active principal
  - Add link/unlink functionality
- **Priority**: 🟡 **MEDIUM** - ICP-specific component

### **4. Forever Storage Modal - MIGRATE**

- **File**: `src/nextjs/src/components/galleries/forever-storage-progress-modal.tsx`
- **Usage**: Used in gallery pages (`/gallery/[id]`, `/gallery/[id]/preview`) - modal for storing galleries permanently on ICP
- **Current Usage**:
  ```typescript
  const {
    hasLinkedII,
    isCoAuthActive, // ❌ REMOVE
    linkedIcPrincipal, // ❌ SINGLE → MULTIPLE
    statusMessage, // ❌ REMOVE
    statusClass, // ❌ REMOVE
    remainingMinutes, // ❌ REMOVE
  } = useIICoAuth();
  ```
- **Migration Required**:
  - Replace `isCoAuthActive` check with `hasLinkedII` check
  - Update logic to work with multiple linked principals
  - Remove TTL-related status display
- **Priority**: 🟡 **MEDIUM** - Gallery storage component

### **5. User Button with II - MIGRATE**

- **File**: `src/nextjs/src/components/auth/user-button-client-with-ii.tsx`
- **Usage**: Used in header (`/components/layout/header.tsx`) - shows user avatar with II status in dropdown
- **Current Usage**:
  ```typescript
  const {
    isCoAuthActive, // ❌ REMOVE
    activeIcPrincipal, // ❌ REMOVE
    statusMessage, // ❌ REMOVE
    statusClass, // ❌ REMOVE
  } = useIICoAuth();
  ```
- **Migration Required**:
  - Remove active state display
  - Show linked principals count instead
  - Remove TTL status
- **Priority**: 🟡 **MEDIUM** - UI component
- **Related Issue**: [Active Principal Detection Issue](./active-principal-detection-issue.md) - Critical UX issue for showing active principal

### **6. User Button - MIGRATE**

- **File**: `src/nextjs/src/components/auth/user-button-client.tsx`
- **Usage**: **NOT CURRENTLY USED** - This is the old version without II support. The header uses `user-button-client-with-ii.tsx` instead.
- **Current Usage**:
  ```typescript
  const {
    isCoAuthActive, // ❌ REMOVE
    activeIcPrincipal, // ❌ REMOVE
    statusMessage, // ❌ REMOVE
    statusClass, // ❌ REMOVE
  } = useIICoAuth();
  ```
- **Migration Required**:
  - Remove active state display
  - Show linked principals count instead
  - Remove TTL status
- **Priority**: 🟢 **LOW** - Unused component (can be deleted)

## 📊 **Migration Summary**

### **Components to Migrate: 5**

- **High Priority**: 1 (II Co-Auth Controls + Linked Accounts - merged)
- **Medium Priority**: 4 (ICP Card, Forever Storage Modal, User Buttons)

### **Common Migration Patterns**

#### **1. State Changes**

- ❌ Remove: `isCoAuthActive`, `activeIcPrincipal`, `assertedAt`
- ❌ Remove: `ttlStatus`, `isExpired`, `isInGracePeriod`, `isWarning`
- ❌ Remove: `statusMessage`, `statusClass`, `remainingMinutes`
- ✅ Add: `linkedIcPrincipals` (array), `hasLinkedII`

#### **2. Action Changes**

- ❌ Remove: `activateII()`, `disconnectII()`, `refreshTTL()`
- ✅ Add: `linkII()`, `unlinkII()`, `refreshLinks()`

#### **3. UI Changes**

- ❌ Remove: TTL progress bars, countdown timers, "Activate" buttons
- ❌ Remove: "Disconnect for this session" buttons
- ✅ Add: List of linked principals, "Link new" buttons, "Unlink" buttons

## 🎯 **Migration Strategy**

### **Phase 1: High Priority Components**

1. **II Co-Auth Controls + Linked Accounts** - Merge into single "Internet Identity Management" component

### **Phase 2: Medium Priority Components**

2. **ICP Card** - ICP-specific functionality
3. **Forever Storage Modal** - Gallery storage
4. **User Button with II** - UI component
5. **User Button** - UI component

### **Phase 3: Cleanup**

6. **Delete test file** - Remove old test ✅ **COMPLETED**
7. **Delete old hook file** - Remove `use-ii-coauth.ts`

## 🚀 **Next Steps**

1. **Merge II Co-Auth Controls + Linked Accounts** - Create single "Internet Identity Management" component
2. **Then remaining components** - Simpler migrations
3. **Test each migration** - Ensure functionality works
4. **Clean up old code** - Remove deprecated files

---

**Priority**: 🔴 **HIGH** - Core architecture change affecting 5 components.

**Estimated Effort**: 2-3 days for complete migration and testing.

**Dependencies**: New `useIILinks()` hook must be working before starting migrations.
