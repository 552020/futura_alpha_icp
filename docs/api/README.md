# API Documentation

This directory contains comprehensive documentation for the Futura API system.

## 📚 **Available Documentation**

### **Sharing System**

- **[Sharing System API](./sharing-system.md)** - Complete documentation for the sharing API system
  - 12 sharing endpoints across 4 phases
  - User-to-user and public link sharing
  - Share management and permission control
  - Public link validation and deactivation

## 🎯 **Quick Reference**

### **Sharing APIs Overview**

#### **Memory Sharing**

- `POST /api/memories/[id]/share` - Share memory with users or create public links
- `POST /api/memories/[id]/public-link` - Create public shareable links
- `GET /api/memories/[id]/share-link` - Access memory via public token

#### **Folder Sharing**

- `POST /api/folders/[id]/share` - Share folder with users or create public links
- `GET /api/folders/shared` - List folders shared with current user
- `POST /api/folders/[id]/public-link` - Create folder public links

#### **Share Management**

- `GET /api/[resourceType]/[id]/shares` - List all shares for a resource
- `DELETE /api/shares/[shareId]` - Revoke a specific share
- `PUT /api/shares/[shareId]/permissions` - Update share permissions

#### **Public Link Management**

- `GET /api/shared/[token]` - Validate public tokens
- `DELETE /api/public-links/[tokenId]` - Deactivate public links

## 🏗️ **Architecture**

All APIs follow a clean service layer architecture:

- **API Routes**: HTTP handling, authentication, validation
- **Service Functions**: Business logic, database operations
- **Database Layer**: Data persistence with proper relations

## 🔐 **Security**

- **Authentication**: Required for all management operations
- **Ownership Validation**: Only resource owners can manage shares
- **Permission Granularity**: Fine-grained control over user access
- **Token Security**: Public links with expiration and deactivation

## 📊 **Features**

- **Universal Resource Support**: Works with memories, folders, and galleries
- **User-to-User Sharing**: Share with specific users with custom permissions
- **Public Link Sharing**: Generate shareable links with optional expiration
- **Share Management**: List, update, and revoke shares
- **Public Link Management**: Validate and deactivate public links
- **Service Layer Architecture**: Clean separation of concerns

---

**For detailed API documentation, see [Sharing System API](./sharing-system.md)**
