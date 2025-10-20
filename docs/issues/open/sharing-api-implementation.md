# Sharing API Implementation Plan

**Status**: Open  
**Priority**: High  
**Type**: Implementation  
**Created**: 2025-01-20  
**Assignee**: TBD

## 📋 **Overview**

The sharing service layer is complete, but the API endpoints need to be implemented or updated to use the new universal sharing system. Currently, some sharing APIs are disabled or deprecated, and folder sharing APIs are missing entirely.

## 🎯 **Current Status**

### ✅ **Working APIs:**

- `GET /api/memories/shared` - ✅ Uses `resourceMembership`
- `GET /api/galleries/shared` - ✅ Uses `resourceMembership`
- `POST /api/galleries/[id]/share` - ✅ Uses `resourceMembership`

### ✅ **Recently Enabled APIs:**

- `POST /api/memories/[id]/share` - ✅ **ENABLED** (supports user & public sharing)

### ❌ **Disabled/Deprecated APIs:**

- `GET /api/memories/[id]/share-link` - ❌ **DEPRECATED** (returns 410)

### 🚫 **Missing APIs:**

- Folder sharing endpoints
- Public link generation endpoints
- Share management endpoints

## 🎯 **Implementation Plan**

### **Phase 1: Enable Memory Sharing API**

#### **1.1 Update Memory Share Endpoint** ✅ **COMPLETED**

**File**: `src/app/api/memories/[id]/share/route.ts`

**Status**: ✅ **ENABLED** - Now supports both user-to-user and public link sharing

**Implementation Completed**:

```typescript
// ✅ IMPLEMENTED - Now fully functional
import { createShare, createPublicLink, generateShareableUrl } from "@/services/sharing";

// Supports both user-to-user and public link sharing
// - User sharing: shareType: "user", targetUserId, permissions
// - Public sharing: shareType: "public", expiresAt (optional)
// - Full error handling and validation
// - Authentication support for both regular and onboarding flows
```

**Request Body**:

```json
{
  "shareType": "user" | "public",
  "targetUserId": "string (for user sharing)",
  "permissions": {
    "canView": true,
    "canEdit": false,
    "canDelete": false
  },
  "expiresAt": "2025-01-27T00:00:00Z (optional)"
}
```

**Response**:

```json
{
  "success": true,
  "data": {
    "shareId": "uuid",
    "shareUrl": "https://app.com/shared/token123 (for public links)",
    "permissions": { "canView": true, "canEdit": false, "canDelete": false }
  }
}
```

#### **1.2 Create Public Link Endpoint**

**File**: `src/app/api/memories/[id]/public-link/route.ts` (NEW)

**Implementation**:

```typescript
import { createPublicLink, generateShareableUrl } from "@/services/sharing";

export async function POST(request: NextRequest, context: { params: Promise<{ id: string }> }) {
  const { id: memoryId } = await context.params;
  const body = await request.json();

  const { expiresAt } = body;

  const result = await createPublicLink({
    resourceType: "memory",
    resourceId: memoryId,
    createdBy: authenticatedUserId,
    expiresAt: expiresAt ? new Date(expiresAt) : undefined,
  });

  if (result.success) {
    const shareUrl = generateShareableUrl(result.data.token);
    return NextResponse.json({
      success: true,
      data: {
        token: result.data.token,
        shareUrl,
        expiresAt: result.data.expiresAt,
      },
    });
  }
}
```

#### **1.3 Update Share Link Endpoint**

**File**: `src/app/api/memories/[id]/share-link/route.ts`

**Current State**: Deprecated with 410 error

```typescript
return NextResponse.json(
  {
    error: "Share codes are no longer supported. Please use the new sharing system.",
    suggestion: "Use direct user sharing via resourceMembership instead.",
  },
  { status: 410 }
);
```

**Implementation**:

```typescript
import { validatePublicToken, grantAccessViaToken } from "@/services/sharing";

export async function GET(request: NextRequest, context: { params: Promise<{ id: string }> }) {
  const { id: memoryId } = await context.params;
  const { searchParams } = new URL(request.url);
  const token = searchParams.get("token");

  if (!token) {
    return NextResponse.json({ error: "Token is required" }, { status: 400 });
  }

  const validation = await validatePublicToken(token);

  if (!validation.success || !validation.data?.isValid) {
    return NextResponse.json(
      {
        error: "Invalid or expired token",
        details: validation.data?.error,
      },
      { status: 403 }
    );
  }

  // Grant access to the user
  const accessResult = await grantAccessViaToken(token, authenticatedUserId);

  if (accessResult.success) {
    return NextResponse.json({
      success: true,
      data: {
        memoryId,
        accessGranted: true,
        permissions: { canView: true, canEdit: false, canDelete: false },
      },
    });
  }
}
```

### **Phase 2: Add Folder Sharing APIs**

#### **2.1 Create Folder Share Endpoint**

**File**: `src/app/api/folders/[id]/share/route.ts` (NEW)

**Implementation**:

```typescript
import { createShare, createPublicLink } from "@/services/sharing";

export async function POST(request: NextRequest, context: { params: Promise<{ id: string }> }) {
  const { id: folderId } = await context.params;
  const body = await request.json();

  const { shareType, targetUserId, permissions, expiresAt } = body;

  if (shareType === "user") {
    const result = await createShare({
      resourceType: "folder",
      resourceId: folderId,
      targetUserId,
      permissions,
      invitedBy: authenticatedUserId,
    });
  } else if (shareType === "public") {
    const result = await createPublicLink({
      resourceType: "folder",
      resourceId: folderId,
      createdBy: authenticatedUserId,
      expiresAt: expiresAt ? new Date(expiresAt) : undefined,
    });
  }
}
```

#### **2.2 Create Shared Folders Endpoint**

**File**: `src/app/api/folders/shared/route.ts` (NEW)

**Implementation**:

```typescript
import { getResourceShares } from "@/services/sharing";

export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const page = parseInt(searchParams.get("page") || "1");
  const limit = parseInt(searchParams.get("limit") || "20");

  // Get all folders shared with the user
  const sharedFolders = await db.query.resourceMembership.findMany({
    where: and(
      eq(resourceMembership.resourceType, "folder"),
      eq(resourceMembership.allUserId, authenticatedUserId),
      ne(resourceMembership.grantSource, "system")
    ),
    orderBy: desc(resourceMembership.createdAt),
    limit,
    offset: (page - 1) * limit,
  });

  return NextResponse.json({
    success: true,
    data: sharedFolders,
    pagination: { page, limit, hasMore: sharedFolders.length === limit },
  });
}
```

#### **2.3 Create Folder Public Link Endpoint**

**File**: `src/app/api/folders/[id]/public-link/route.ts` (NEW)

**Implementation**: Similar to memory public link endpoint but for folders.

### **Phase 3: Add Share Management APIs**

#### **3.1 Get Resource Shares Endpoint**

**File**: `src/app/api/[resourceType]/[id]/shares/route.ts` (NEW)

**Implementation**:

```typescript
import { getResourceShares } from "@/services/sharing";

export async function GET(
  request: NextRequest,
  context: {
    params: Promise<{ resourceType: string; id: string }>;
  }
) {
  const { resourceType, id } = await context.params;

  const result = await getResourceShares({
    resourceType: resourceType as "memory" | "folder" | "gallery",
    resourceId: id,
    includeInactive: false,
  });

  return NextResponse.json(result);
}
```

#### **3.2 Revoke Share Endpoint**

**File**: `src/app/api/shares/[shareId]/route.ts` (NEW)

**Implementation**:

```typescript
import { revokeShare } from "@/services/sharing";

export async function DELETE(
  request: NextRequest,
  context: {
    params: Promise<{ shareId: string }>;
  }
) {
  const { shareId } = await context.params;

  const result = await revokeShare(shareId, authenticatedUserId);

  return NextResponse.json(result);
}
```

#### **3.3 Update Share Permissions Endpoint**

**File**: `src/app/api/shares/[shareId]/permissions/route.ts` (NEW)

**Implementation**:

```typescript
import { updateSharePermissions } from "@/services/sharing";

export async function PUT(
  request: NextRequest,
  context: {
    params: Promise<{ shareId: string }>;
  }
) {
  const { shareId } = await context.params;
  const body = await request.json();

  const { permissions } = body;

  const result = await updateSharePermissions(shareId, permissions, authenticatedUserId);

  return NextResponse.json(result);
}
```

### **Phase 4: Add Public Link Management APIs**

#### **4.1 Validate Public Token Endpoint**

**File**: `src/app/api/shared/[token]/route.ts` (NEW)

**Implementation**:

```typescript
import { validatePublicToken, grantAccessViaToken } from "@/services/sharing";

export async function GET(
  request: NextRequest,
  context: {
    params: Promise<{ token: string }>;
  }
) {
  const { token } = await context.params;

  const validation = await validatePublicToken(token);

  if (!validation.success || !validation.data?.isValid) {
    return NextResponse.json(
      {
        error: "Invalid or expired token",
        details: validation.data?.error,
      },
      { status: 403 }
    );
  }

  // Grant access to the user
  const accessResult = await grantAccessViaToken(token, authenticatedUserId);

  return NextResponse.json({
    success: true,
    data: {
      resourceType: validation.data.record?.resourceType,
      resourceId: validation.data.record?.resourceId,
      accessGranted: accessResult.success,
    },
  });
}
```

#### **4.2 Deactivate Public Link Endpoint**

**File**: `src/app/api/public-links/[tokenId]/route.ts` (NEW)

**Implementation**:

```typescript
import { deactivatePublicLink } from "@/services/sharing";

export async function DELETE(
  request: NextRequest,
  context: {
    params: Promise<{ tokenId: string }>;
  }
) {
  const { tokenId } = await context.params;

  const result = await deactivatePublicLink(tokenId, authenticatedUserId);

  return NextResponse.json(result);
}
```

## 🗄️ **Database Requirements**

### **Already Implemented:**

- ✅ `resourceMembership` table - User-to-user sharing
- ✅ `resourceShareTokens` table - Public link sharing

### **No Additional Tables Needed:**

- All required tables are already in place
- Sharing service uses existing schema

## 🧪 **Testing Requirements**

### **Unit Tests:**

- API endpoint functionality
- Error handling and validation
- Authentication and authorization

### **Integration Tests:**

- End-to-end sharing workflows
- Cross-resource type compatibility
- Public link generation and validation

### **User Acceptance Tests:**

- Share creation and management UI
- Public link access
- Permission changes

## 📊 **Success Metrics**

- **Functionality**: All sharing APIs working correctly
- **Performance**: API responses complete in <500ms
- **User Experience**: Intuitive sharing interface
- **Security**: Proper access control and token validation

## 🔗 **Related Issues**

- [Advanced Sharing Operations](./advanced-sharing-operations.md)
- [Memory Sharing Modal Implementation](./memory-sharing-modal-implementation.md)

## 📝 **Implementation Order**

### **Week 1: Memory Sharing**

1. ✅ Enable `POST /api/memories/[id]/share` - **COMPLETED**
2. ✅ Create `POST /api/memories/[id]/public-link` - **COMPLETED**
3. ✅ Update `GET /api/memories/[id]/share-link` - **COMPLETED**

### **Week 2: Folder Sharing**

4. ✅ Create `POST /api/folders/[id]/share` - **COMPLETED**
5. ✅ Create `GET /api/folders/shared` - **COMPLETED**
6. ✅ Create `POST /api/folders/[id]/public-link` - **COMPLETED**

### **Week 3: Share Management**

7. ✅ Create `GET /api/[resourceType]/[id]/shares` - **COMPLETED**
8. ✅ Create `DELETE /api/shares/[shareId]` - **COMPLETED**
9. ✅ Create `PUT /api/shares/[shareId]/permissions` - **COMPLETED**

### **Week 4: Public Link Management**

10. ✅ Create `GET /api/shared/[token]` - **COMPLETED**
11. ✅ Create `DELETE /api/public-links/[tokenId]` - **COMPLETED**
12. Testing and documentation

## 🎉 **Phase 3 Implementation Summary**

### **✅ Completed APIs:**

#### **3.1 Get Resource Shares Endpoint**

- **File**: `src/app/api/[resourceType]/[id]/shares/route.ts`
- **Method**: `GET`
- **Purpose**: List all shares for a specific resource (memory, folder, or gallery)
- **Features**: Universal resource support, ownership validation, service layer architecture

#### **3.2 Revoke Share Endpoint**

- **File**: `src/app/api/shares/[shareId]/route.ts`
- **Method**: `DELETE`
- **Purpose**: Revoke a specific share
- **Features**: Share revocation, ownership validation, comprehensive error handling

#### **3.3 Update Share Permissions Endpoint**

- **File**: `src/app/api/shares/[shareId]/permissions/route.ts`
- **Method**: `PUT`
- **Purpose**: Update permissions for a specific share
- **Features**: Permission management, validation, type safety

### **🚀 Key Features Implemented:**

✅ **Universal Resource Support** - Works with memories, folders, and galleries  
✅ **Service Layer Architecture** - Uses service functions only  
✅ **Ownership Validation** - Ensures user owns the resource  
✅ **Comprehensive Error Handling** - Detailed error responses  
✅ **Type Safety** - Full TypeScript typing  
✅ **Logging** - Detailed logging for debugging  
✅ **Permission Management** - Update share permissions  
✅ **Share Revocation** - Remove shares completely

## 🎉 **Phase 4 Implementation Summary**

### **✅ Completed APIs:**

#### **4.1 Validate Public Token Endpoint**

- **File**: `src/app/api/shared/[token]/route.ts`
- **Method**: `GET`
- **Purpose**: Validate public tokens and grant access
- **Features**: Token validation, access granting, authentication support

#### **4.2 Deactivate Public Link Endpoint**

- **File**: `src/app/api/public-links/[tokenId]/route.ts`
- **Method**: `DELETE`
- **Purpose**: Deactivate public links
- **Features**: Link deactivation, ownership validation, comprehensive error handling

### **🚀 Key Features Implemented:**

✅ **Public Token Validation** - Validate and grant access via tokens  
✅ **Service Layer Architecture** - Uses service functions only  
✅ **Authentication Support** - Works with both authenticated and anonymous users  
✅ **Comprehensive Error Handling** - Detailed error responses  
✅ **Type Safety** - Full TypeScript typing  
✅ **Logging** - Detailed logging for debugging  
✅ **Link Management** - Deactivate public links  
✅ **Access Control** - Proper ownership validation

## 🎯 **Next Steps**

1. ✅ **Phase 1 COMPLETED** - All memory sharing APIs implemented
2. ✅ **Phase 2 COMPLETED** - All folder sharing APIs implemented
3. ✅ **Phase 3 COMPLETED** - All share management APIs implemented
4. ✅ **Phase 4 COMPLETED** - All public link management APIs implemented
5. **Update frontend** - Integrate with sharing modal

---

**Ready to implement!** The sharing service is complete and ready to be integrated with the API layer. 🚀
