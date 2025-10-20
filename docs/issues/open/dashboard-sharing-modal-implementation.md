# Dashboard Sharing Modal Implementation

**Status**: Open  
**Priority**: High  
**Type**: Frontend Implementation  
**Created**: October 20, 2025  
**Assignee**: TBD

## 📋 **Overview**

Implement a sharing modal for the dashboard content cards that allows users to share memories and folders using the enhanced public link access control system. The modal should be triggered by the existing share button and provide options for user-specific sharing, role-based access, and public link generation.

## 🎯 **Current State**

### **✅ Existing Infrastructure:**

- ✅ **Share button** already exists in `ContentCard` component
- ✅ **Enhanced sharing APIs** implemented with access control
- ✅ **Quick edit modal** pattern established for similar functionality
- ✅ **React Query mutations** for data management

### **❌ Missing Frontend Components:**

- ❌ **Sharing modal** for memory/folder sharing
- ❌ **User selection interface** for user-specific sharing
- ❌ **Role-based access controls** in UI
- ❌ **Public link generation** with access restrictions
- ❌ **Share management** (view, revoke, update permissions)

## 🎨 **UI/UX Requirements**

### **Modal Design:**

```
┌─────────────────────────────────────────┐
│  Share Memory: "Family Vacation 2024"  │
├─────────────────────────────────────────┤
│                                         │
│  📧 Share with specific users           │
│  ┌─────────────────────────────────────┐ │
│  │  Search users...                    │ │
│  │  👤 John Doe (john@example.com)     │ │
│  │  👤 Jane Smith (jane@example.com)  │ │
│  └─────────────────────────────────────┘ │
│                                         │
│  🔗 Create public link                  │
│  ☐ Require authentication              │
│  ☐ Restrict to specific users           │
│  ☐ Restrict to specific roles          │
│  📅 Expires: [Date picker]             │
│                                         │
│  [Cancel]                    [Share]     │
└─────────────────────────────────────────┘
```

### **Share Types:**

1. **User-to-User Sharing**

   - Search and select specific users
   - Set permissions (view, edit, delete)
   - Send invitation

2. **Public Link Sharing**
   - Generate shareable URL
   - Set access restrictions
   - Configure expiration
   - Copy link to clipboard

## 🔧 **Technical Implementation**

### **Components to Create:**

#### **1. SharingModal Component**

```typescript
interface SharingModalProps {
  isOpen: boolean;
  onClose: () => void;
  resourceType: "memory" | "folder";
  resourceId: string;
  resourceTitle: string;
  onShareSuccess?: () => void;
}
```

#### **2. UserSearch Component**

```typescript
interface UserSearchProps {
  onUserSelect: (user: User) => void;
  selectedUsers: User[];
  placeholder?: string;
}
```

#### **3. AccessControlSettings Component**

```typescript
interface AccessControlSettingsProps {
  shareType: "user" | "public";
  onSettingsChange: (settings: AccessControlSettings) => void;
}
```

### **API Integration:**

#### **User-to-User Sharing:**

```typescript
// POST /api/memories/[id]/share
const shareMemory = async (data: { shareType: "user"; targetUserId: string; permissions: SharePermissions }) => {
  // Implementation
};
```

#### **Public Link Sharing:**

```typescript
// POST /api/memories/[id]/public-link
const createPublicLink = async (data: {
  expiresAt?: string;
  allowedUsers?: string[];
  allowedRoles?: string[];
  requireAuth?: boolean;
}) => {
  // Implementation
};
```

### **React Query Integration:**

#### **Sharing Mutations:**

```typescript
// User sharing mutation
const useShareMemory = () => {
  return useMutation({
    mutationFn: shareMemory,
    onSuccess: () => {
      queryClient.invalidateQueries(["memories", "dashboard"]);
      toast.success("Memory shared successfully");
    },
  });
};

// Public link mutation
const useCreatePublicLink = () => {
  return useMutation({
    mutationFn: createPublicLink,
    onSuccess: (data) => {
      navigator.clipboard.writeText(data.shareUrl);
      toast.success("Public link copied to clipboard");
    },
  });
};
```

## 📱 **User Experience Flow**

### **1. Trigger Sharing:**

- User clicks share button on memory/folder card
- Modal opens with sharing options

### **2. User-to-User Sharing:**

- Search and select users
- Set permissions (view/edit/delete)
- Send invitation
- Show success message

### **3. Public Link Sharing:**

- Configure access restrictions
- Set expiration date
- Generate link
- Copy to clipboard
- Show link management options

### **4. Share Management:**

- View existing shares
- Revoke shares
- Update permissions
- Deactivate public links

## 🎯 **Acceptance Criteria**

### **Core Functionality:**

- ✅ **Modal opens** when share button is clicked
- ✅ **User search** with autocomplete functionality
- ✅ **Permission settings** for user sharing
- ✅ **Access control options** for public links
- ✅ **Link generation** and clipboard copy
- ✅ **Success/error feedback** with toast notifications

### **Enhanced Features:**

- ✅ **User whitelist** for public links
- ✅ **Role-based restrictions** for public links
- ✅ **Authentication requirements** for public links
- ✅ **Expiration date** setting for public links
- ✅ **Share management** (view, revoke, update)

### **UI/UX Requirements:**

- ✅ **Responsive design** for mobile and desktop
- ✅ **Loading states** during API calls
- ✅ **Error handling** with user-friendly messages
- ✅ **Accessibility** with proper ARIA labels
- ✅ **Consistent styling** with existing design system

## 🚀 **Implementation Plan**

### **Phase 1: Core Modal (Week 1)**

1. Create `SharingModal` component
2. Implement basic user-to-user sharing
3. Add permission settings
4. Integrate with existing share button

### **Phase 2: Public Links (Week 2)**

1. Add public link generation
2. Implement access control settings
3. Add clipboard copy functionality
4. Create shareable URL display

### **Phase 3: Enhanced Features (Week 3)**

1. Add user whitelist functionality
2. Implement role-based restrictions
3. Add expiration date picker
4. Create share management interface

### **Phase 4: Polish & Testing (Week 4)**

1. Add comprehensive error handling
2. Implement loading states
3. Add accessibility features
4. Write unit tests

## 🔗 **Related Issues**

- [Memory Sharing Modal Implementation](./memory-sharing-modal-implementation.md)
- [Sharing API Implementation](./sharing-api-implementation.md)
- [Enhanced Public Link Access Control](../api/sharing-system.md)

## 📝 **Technical Notes**

### **Dependencies:**

- React Query for state management
- Radix UI for modal components
- React Hook Form for form handling
- Date picker for expiration settings
- Toast notifications for feedback

### **State Management:**

- Use React Query mutations for API calls
- Local state for modal visibility and form data
- Optimistic updates for better UX
- Cache invalidation for data consistency

### **Error Handling:**

- Network error handling
- Validation error display
- User-friendly error messages
- Retry mechanisms for failed requests

---

**Ready for implementation!** The enhanced sharing APIs are complete and ready for frontend integration. 🚀
