# Email Invitation Sharing Enhancement

**Status**: Open  
**Priority**: Medium  
**Type**: Frontend Integration  
**Created**: October 20, 2025  
**Assignee**: TBD

## 📋 **Overview**

Integrate email-based sharing into the dashboard sharing modal using the existing temporary user system from the onboarding process. This enables sharing with email addresses by creating temporary users, just like the onboarding flow does.

## 🎯 **Current State**

### **✅ Existing Infrastructure:**

- ✅ **Temporary user system** via `temporary_user` and `allUsers` tables
- ✅ **User creation API** (`POST /api/users`) for temporary users
- ✅ **Memory sharing API** with `isOnboarding` support
- ✅ **Onboarding process** that creates temporary users and shares memories
- ✅ **User promotion system** (temporary → permanent user on signup)
- ✅ **Sharing modal** with email input functionality

### **❌ Missing Integration:**

- ❌ **Frontend integration** of temporary user creation in sharing modal
- ❌ **Email sending** for temporary user invitations
- ❌ **Auth context** for getting current user ID in sharing modal

## 🎨 **User Experience Flow**

### **1. Email Invitation Process:**

```
User clicks share → Enters email address → System creates pending invitation
↓
Email sent to recipient → Recipient clicks invitation link
↓
If user exists: Direct access granted
If new user: Registration flow → Access granted after signup
```

### **2. Invitation States:**

- **📧 Pending** - Invitation sent, awaiting response
- **✅ Accepted** - User registered and accepted invitation
- **⏰ Expired** - Invitation expired (configurable timeframe)
- **❌ Declined** - User declined invitation

## 🔧 **Technical Implementation**

### **Database Schema Enhancement:**

#### **1. New Table: `email_invitations`**

```sql
CREATE TABLE email_invitations (
  id TEXT PRIMARY KEY,
  resource_type TEXT NOT NULL, -- 'memory', 'folder', 'gallery'
  resource_id TEXT NOT NULL,
  email TEXT NOT NULL,
  invited_by_user_id TEXT NOT NULL,
  permissions JSONB NOT NULL,
  invitation_token TEXT UNIQUE NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'accepted', 'expired', 'declined'
  expires_at TIMESTAMP,
  created_at TIMESTAMP DEFAULT NOW(),
  accepted_at TIMESTAMP,
  accepted_by_user_id TEXT
);
```

#### **2. Enhanced `resourceMembership` Table:**

```sql
-- Add optional email field for tracking invitation source
ALTER TABLE resource_membership
ADD COLUMN invited_email TEXT,
ADD COLUMN invitation_id TEXT REFERENCES email_invitations(id);
```

### **API Endpoints:**

#### **1. Create Email Invitation**

```typescript
POST /api/memories/[id]/invite
{
  "email": "user@example.com",
  "permissions": {
    "canView": true,
    "canEdit": false,
    "canDelete": false
  },
  "expiresAt": "2025-11-20T00:00:00Z"
}
```

#### **2. Accept Invitation**

```typescript
POST /api/invitations/[token]/accept
{
  "userId": "user-id-after-registration"
}
```

#### **3. List Pending Invitations**

```typescript
GET / api / invitations / pending;
```

### **Email Templates:**

#### **1. Invitation Email**

```html
Subject: You've been invited to view a memory on Futura Hi there! [Owner Name] has invited you to view a memory on
Futura. Memory: [Memory Title] Permissions: [View/Edit/Delete] Click here to accept: [Invitation Link] This invitation
expires on [Expiration Date].
```

#### **2. New User Registration Flow**

```html
Subject: Complete your Futura registration to access shared content Hi there! You've been invited to view content on
Futura. To access it, please complete your registration. Click here to sign up: [Registration Link] After registration,
you'll automatically have access to the shared content.
```

## 🎯 **Implementation Plan**

### **Phase 1: Database Schema (Week 1)**

1. Create `email_invitations` table
2. Add email fields to `resourceMembership`
3. Create database migrations
4. Update TypeScript types

### **Phase 2: Backend Services (Week 2)**

1. Create invitation service functions
2. Implement email sending system
3. Add invitation acceptance logic
4. Create API endpoints

### **Phase 3: Frontend Integration (Week 3)**

1. Update sharing modal to support email input
2. Add email validation and formatting
3. Implement invitation management UI
4. Add registration flow integration

### **Phase 4: Email System (Week 4)**

1. Set up email service (SendGrid/AWS SES)
2. Create email templates
3. Implement email sending logic
4. Add email tracking and analytics

## 🔗 **Related Issues**

- [Dashboard Sharing Modal Implementation](./dashboard-sharing-modal-implementation.md)
- [Sharing API Implementation](./sharing-api-implementation.md)

## 📝 **Technical Notes**

### **Security Considerations:**

- **Invitation tokens** must be cryptographically secure
- **Expiration dates** prevent indefinite access
- **Email validation** prevents abuse
- **Rate limiting** on invitation creation

### **User Experience:**

- **Seamless registration** flow for new users
- **Clear permission communication** in emails
- **Easy invitation management** for senders
- **Mobile-friendly** email templates

### **Scalability:**

- **Batch email sending** for multiple invitations
- **Email queue system** for reliability
- **Invitation analytics** for usage tracking
- **Cleanup processes** for expired invitations

---

**Ready for implementation!** This enhancement will make the sharing system much more user-friendly by supporting email-based invitations. 🚀
