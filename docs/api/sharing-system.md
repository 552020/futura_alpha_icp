# Sharing System API Documentation

**Last Updated**: October 20, 2025  
**Backend**: Next.js/Neon  
**Note**: This documentation covers the Next.js/Neon backend sharing APIs. The ICP sharing system has separate APIs.

## 📋 **Overview**

The sharing system provides sharing capabilities for memories, folders, and galleries. It supports both user-to-user sharing and public link sharing with granular permission management.

### **🎯 Key Features**

- **Universal Resource Support**: Works with memories, folders, and galleries
- **User-to-User Sharing**: Share resources with specific users with custom permissions
- **Public Link Sharing**: Generate shareable links with optional expiration
- **Permission Management**: Control over view, edit, and delete permissions
- **Share Management**: List, update, and revoke shares
- **Public Link Management**: Validate and deactivate public links
- **Service Layer Architecture**: Separation of concerns with zero direct database operations

## 📋 **API Endpoints Overview**

### **Memory Sharing APIs**

| Endpoint                         | Method | Purpose                                       |
| -------------------------------- | ------ | --------------------------------------------- |
| `/api/memories/[id]/share`       | POST   | Share memory with users or create public link |
| `/api/memories/[id]/public-link` | POST   | Create public link for memory                 |
| `/api/memories/[id]/share-link`  | GET    | Access memory via public token                |

### **Folder Sharing APIs**

| Endpoint                        | Method | Purpose                                       |
| ------------------------------- | ------ | --------------------------------------------- |
| `/api/folders/[id]/share`       | POST   | Share folder with users or create public link |
| `/api/folders/[id]/public-link` | POST   | Create public link for folder                 |
| `/api/folders/shared`           | GET    | List folders shared with current user         |

### **Share Management APIs**

| Endpoint                            | Method | Purpose                        |
| ----------------------------------- | ------ | ------------------------------ |
| `/api/[resourceType]/[id]/shares`   | GET    | List all shares for a resource |
| `/api/shares/[shareId]`             | DELETE | Revoke a specific share        |
| `/api/shares/[shareId]/permissions` | PUT    | Update share permissions       |

### **Public Link Management APIs**

| Endpoint                      | Method | Purpose                                |
| ----------------------------- | ------ | -------------------------------------- |
| `/api/shared/[token]`         | GET    | Validate public token and grant access |
| `/api/public-links/[tokenId]` | DELETE | Deactivate public link                 |

**Total: 10 API endpoints across 4 categories**

## 🏗️ **Architecture**

### **Service Layer Pattern**

All API endpoints use a clean service layer architecture:

- **API Routes**: Handle HTTP requests/responses, authentication, validation
- **Service Functions**: Business logic, database operations, error handling
- **Database Layer**: Data persistence with proper relations

### **File Structure**

#### **API Routes Layer** (`src/app/api/`)

```
src/app/api/
├── memories/[id]/
│   ├── share/route.ts              # POST - Share memory with users/public
│   ├── public-link/route.ts       # POST - Create memory public link
│   └── share-link/route.ts        # GET - Access memory via token
├── folders/[id]/
│   ├── share/route.ts             # POST - Share folder with users/public
│   └── public-link/route.ts       # POST - Create folder public link
├── folders/
│   └── shared/route.ts            # GET - List folders shared with user
├── [resourceType]/[id]/
│   └── shares/route.ts            # GET - List all shares for resource
├── shares/[shareId]/
│   ├── route.ts                   # DELETE - Revoke share
│   └── permissions/route.ts       # PUT - Update share permissions
├── public-links/[tokenId]/
│   └── route.ts                   # DELETE - Deactivate public link
└── shared/[token]/
    └── route.ts                   # GET - Validate public token
```

#### **Service Layer** (`src/services/`)

```
src/services/
├── sharing/
│   ├── index.ts                   # Export all sharing functions
│   ├── share-operations.ts        # User-to-user sharing logic
│   ├── token-operations.ts       # Public link operations
│   └── types.ts                  # Sharing-specific types
├── shared/
│   └── types.ts                  # Shared types (OperationResult)
├── user/
│   └── user-operations.ts         # User record operations
├── memory/
│   └── memory-operations.ts       # Memory operations
└── folder/
    └── index.ts                  # Folder operations
```

#### **Database Layer** (`src/db/`)

```
src/db/
├── tables.ts                     # Database schema definitions
└── relations.ts                  # Table relationships
```

### **Service Layer Pattern Implementation**

#### **API Route Structure**

```typescript
// Example: src/app/api/memories/[id]/share/route.ts
export async function POST(request: NextRequest, context: { params: Promise<{ id: string }> }) {
  // 1. Authentication
  const session = await auth();

  // 2. Get user record using service function
  const userResult = await getAllUserRecord(session.user.id);

  // 3. Validate resource ownership using service function
  const memoryResult = await getMemoryWithRelations(memoryId, allUserRecord.id);

  // 4. Perform business logic using service function
  const shareResult = await createShare({...});

  // 5. Return response
  return NextResponse.json({...});
}
```

#### **Service Function Structure**

```typescript
// Example: src/services/sharing/share-operations.ts
export async function createShare(params: CreateShareParams): Promise<OperationResult> {
  try {
    // 1. Validate input
    // 2. Check business rules
    // 3. Perform database operations
    // 4. Return result
  } catch (error) {
    // Handle errors and return standardized result
  }
}
```

### **Database Schema**

- **`resourceMembership`**: User-to-user sharing relationships
- **`resourceShareTokens`**: Public link tokens with expiration support

## 📚 **API Endpoints**

### **Phase 1: Memory Sharing APIs**

#### **1.1 Share Memory**

**Endpoint**: `POST /api/memories/[id]/share`  
**Purpose**: Share a memory with users or create public links  
**Authentication**: Required

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
  "expiresAt": "2025-01-27T00:00:00Z (optional for public links)"
}
```

**Response**:

```json
{
  "success": true,
  "data": {
    "shareId": "uuid",
    "shareType": "user" | "public",
    "shareUrl": "https://app.com/shared/token123 (for public links)",
    "permissions": { "canView": true, "canEdit": false, "canDelete": false }
  }
}
```

#### **1.2 Create Memory Public Link**

**Endpoint**: `POST /api/memories/[id]/public-link`  
**Purpose**: Create a public shareable link for a memory  
**Authentication**: Required

**Request Body**:

```json
{
  "expiresAt": "2025-01-27T00:00:00Z (optional)",
  "isActive": true
}
```

**Response**:

```json
{
  "success": true,
  "data": {
    "shareId": "uuid",
    "token": "secure_token",
    "shareUrl": "https://app.com/shared/token123",
    "expiresAt": "2025-01-27T00:00:00Z",
    "isActive": true,
    "createdAt": "2025-01-20T10:30:00Z"
  }
}
```

#### **1.3 Access Memory via Share Link**

**Endpoint**: `GET /api/memories/[id]/share-link?token=xxx`  
**Purpose**: Access a memory via public token  
**Authentication**: Optional (enhanced permissions if authenticated)

**Response**:

```json
{
  "success": true,
  "data": {
    "memoryId": "uuid",
    "memory": {
      "id": "uuid",
      "type": "image",
      "title": "Memory Title",
      "description": "Memory Description",
      "createdAt": "2025-01-20T10:30:00Z"
    },
    "accessGranted": true,
    "permissions": { "canView": true, "canEdit": false, "canDelete": false },
    "shareInfo": {
      "tokenId": "uuid",
      "expiresAt": "2025-01-27T00:00:00Z",
      "isActive": true
    }
  }
}
```

### **Phase 2: Folder Sharing APIs**

#### **2.1 Share Folder**

**Endpoint**: `POST /api/folders/[id]/share`  
**Purpose**: Share a folder with users or create public links  
**Authentication**: Required

**Request Body**: Same as memory sharing
**Response**: Same format as memory sharing

#### **2.2 List Shared Folders**

**Endpoint**: `GET /api/folders/shared`  
**Purpose**: Get folders shared with the current user  
**Authentication**: Required

**Query Parameters**:

- `page`: Page number (default: 1)
- `limit`: Items per page (default: 20)
- `orderBy`: Sort order (default: 'sharedAt')

**Response**:

```json
{
  "success": true,
  "data": [
    {
      "id": "folder-uuid",
      "name": "Folder Name",
      "title": "Folder Title",
      "ownerId": "owner-uuid",
      "createdAt": "2025-01-20T10:30:00Z",
      "updatedAt": "2025-01-20T10:30:00Z",
      "shareInfo": {
        "shareId": "share-uuid",
        "sharedAt": "2025-01-20T10:30:00Z",
        "permissions": { "canView": true, "canEdit": false, "canDelete": false },
        "grantSource": "user",
        "invitedBy": "inviter-uuid"
      }
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 5,
    "hasMore": false
  }
}
```

#### **2.3 Create Folder Public Link**

**Endpoint**: `POST /api/folders/[id]/public-link`  
**Purpose**: Create a public shareable link for a folder  
**Authentication**: Required

**Request/Response**: Same format as memory public links

### **Phase 3: Share Management APIs**

#### **3.1 List Resource Shares**

**Endpoint**: `GET /api/[resourceType]/[id]/shares`  
**Purpose**: Get all shares for a specific resource  
**Authentication**: Required  
**Resource Types**: `memory`, `folder`, `gallery`

**Response**:

```json
{
  "success": true,
  "data": {
    "resourceType": "memory",
    "resourceId": "uuid",
    "shares": [
      {
        "id": "share-uuid",
        "type": "user" | "public",
        "targetUserId": "user-uuid (for user shares)",
        "permissions": { "canView": true, "canEdit": false, "canDelete": false },
        "createdAt": "2025-01-20T10:30:00Z",
        "expiresAt": "2025-01-27T00:00:00Z (for public links)"
      }
    ],
    "totalCount": 3
  }
}
```

#### **3.2 Revoke Share**

**Endpoint**: `DELETE /api/shares/[shareId]`  
**Purpose**: Revoke a specific share  
**Authentication**: Required

**Response**:

```json
{
  "success": true,
  "data": {
    "shareId": "uuid",
    "revoked": true,
    "revokedAt": "2025-01-20T10:30:00Z"
  }
}
```

#### **3.3 Update Share Permissions**

**Endpoint**: `PUT /api/shares/[shareId]/permissions`  
**Purpose**: Update permissions for a specific share  
**Authentication**: Required

**Request Body**:

```json
{
  "permissions": {
    "canView": true,
    "canEdit": false,
    "canDelete": false
  }
}
```

**Response**:

```json
{
  "success": true,
  "data": {
    "shareId": "uuid",
    "permissions": { "canView": true, "canEdit": false, "canDelete": false },
    "updatedAt": "2025-01-20T10:30:00Z"
  }
}
```

### **Phase 4: Public Link Management APIs**

#### **4.1 Validate Public Token**

**Endpoint**: `GET /api/shared/[token]`  
**Purpose**: Validate a public token and grant access  
**Authentication**: Optional (enhanced permissions if authenticated)

**Response**:

```json
{
  "success": true,
  "data": {
    "token": "secure_token",
    "resourceType": "memory",
    "resourceId": "uuid",
    "accessGranted": true,
    "permissions": { "canView": true, "canEdit": false, "canDelete": false },
    "shareInfo": {
      "tokenId": "uuid",
      "expiresAt": "2025-01-27T00:00:00Z",
      "isActive": true,
      "createdAt": "2025-01-20T10:30:00Z"
    }
  }
}
```

#### **4.2 Deactivate Public Link**

**Endpoint**: `DELETE /api/public-links/[tokenId]`  
**Purpose**: Deactivate a public link  
**Authentication**: Required

**Response**:

```json
{
  "success": true,
  "data": {
    "tokenId": "uuid",
    "deactivated": true,
    "deactivatedAt": "2025-01-20T10:30:00Z"
  }
}
```

## 🔐 **Security & Permissions**

### **Permission Levels**

- **`canView`**: View the resource content
- **`canEdit`**: Modify the resource (title, description, etc.)
- **`canDelete`**: Delete the resource

### **Access Control**

- **Ownership Validation**: Only resource owners can create/manage shares
- **Token Validation**: Public tokens are validated for expiration and activity
- **Permission Inheritance**: Public links inherit basic view permissions
- **Authentication**: Enhanced permissions for authenticated users

### **Security Features**

- **Token Expiration**: Public links can have expiration dates
- **Token Deactivation**: Public links can be deactivated instantly
- **Permission Granularity**: Fine-grained control over user access
- **Audit Trail**: All sharing actions are logged

## 🚀 **Usage Examples**

### **Share a Memory with a User**

```bash
curl -X POST /api/memories/memory-123/share \
  -H "Content-Type: application/json" \
  -d '{
    "shareType": "user",
    "targetUserId": "user-456",
    "permissions": {
      "canView": true,
      "canEdit": false,
      "canDelete": false
    }
  }'
```

### **Create a Public Link**

```bash
curl -X POST /api/memories/memory-123/public-link \
  -H "Content-Type: application/json" \
  -d '{
    "expiresAt": "2025-02-01T00:00:00Z"
  }'
```

### **List All Shares for a Resource**

```bash
curl -X GET /api/memory/memory-123/shares
```

### **Revoke a Share**

```bash
curl -X DELETE /api/shares/share-789
```

## 📊 **Error Handling**

### **Common Error Responses**

#### **401 Unauthorized**

```json
{
  "error": "Unauthorized",
  "details": "Authentication required"
}
```

#### **403 Forbidden**

```json
{
  "error": "Access denied",
  "details": "You don't have permission to perform this action"
}
```

#### **404 Not Found**

```json
{
  "error": "Resource not found",
  "details": "The requested resource does not exist"
}
```

#### **400 Bad Request**

```json
{
  "error": "Invalid request",
  "details": "Missing required field: targetUserId"
}
```

## 🧪 **Testing**

### **Test Scenarios**

1. **User-to-User Sharing**: Share resources with specific users
2. **Public Link Creation**: Generate shareable links with expiration
3. **Permission Management**: Update and revoke permissions
4. **Access Control**: Verify proper access restrictions
5. **Token Validation**: Test public link access and expiration
6. **Error Handling**: Verify proper error responses

### **Integration Tests**

- End-to-end sharing workflows
- Cross-resource type compatibility
- Public link generation and validation
- Permission inheritance and updates

## 📈 **Performance**

### **Optimizations**

- **Batch Queries**: Efficient database operations
- **Service Layer**: Clean separation reduces complexity
- **Caching**: Query results cached where appropriate
- **Pagination**: Large result sets are paginated

### **Response Times**

- **Target**: <500ms for all operations
- **Monitoring**: All endpoints include performance logging
- **Optimization**: Service layer enables easy performance tuning

## 🔗 **Related Documentation**

- [Sharing API Implementation Plan](../issues/open/sharing-api-implementation.md)
- [Advanced Sharing Operations](../issues/open/advanced-sharing-operations.md)
- [Memory Sharing Modal Implementation](../issues/open/memory-sharing-modal-implementation.md)

---

**🎉 The Next.js/Neon sharing system is complete and ready for frontend integration!**
