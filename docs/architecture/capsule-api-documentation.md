# ICP Capsule API Documentation

## Overview

This document provides complete technical documentation for the ICP Capsule API, covering all endpoints, data structures, and error handling patterns for frontend implementation.

**Status**: ✅ IMPLEMENTED  
**Implementation Date**: October 6, 2025  
**Key Commits**:

- `4d9580e` - Complete upload service refactoring with multiple asset support
- `56e48b1` - Add bulk memory API tests and reorganize test structure
- `fe711b0` - Refactor memories core module and add bulk memory APIs
  **Location**: `src/backend/src/lib.rs` and `src/backend/src/capsule.rs`

## Table of Contents

1. [API Endpoints](#api-endpoints)
2. [Data Structures](#data-structures)
3. [Error Handling](#error-handling)
4. [Authorization](#authorization)
5. [Performance Guidelines](#performance-guidelines)

---

## API Endpoints

### Capsule CRUD Operations

#### Create Capsule

```rust
capsules_create(subject: Option<PersonRef>) -> Result<Capsule, Error>
```

**Parameters:**

- `subject: Option<PersonRef>` - Optional subject for the capsule. If `None`, uses caller as subject.

**Returns:**

- `Result<Capsule, Error>` - Created capsule or error

**Behavior:**

- If `subject` is `None`, creates a self-capsule for the caller
- If `subject` is provided, creates a capsule for that subject
- Only one self-capsule per principal is allowed
- Returns existing self-capsule if attempting to create duplicate

**Example:**

```rust
// Create self-capsule
let self_capsule = capsules_create(None)?;

// Create capsule for another person
let other_capsule = capsules_create(Some(PersonRef::Opaque("deceased_person_id".to_string())))?;
```

#### Read Capsule

```rust
capsules_read(capsule_id: String) -> Result<Capsule, Error>
```

**Parameters:**

- `capsule_id: String` - Unique identifier of the capsule

**Returns:**

- `Result<Capsule, Error>` - Full capsule data or error

**Behavior:**

- Returns complete capsule with all memories, galleries, and connections
- Requires ownership or controller access
- Returns `NotFound` if capsule doesn't exist
- Returns `Unauthorized` if caller lacks access

#### Read Basic Capsule Info

```rust
capsules_read_basic(capsule_id: Option<String>) -> Result<CapsuleInfo, Error>
```

**Parameters:**

- `capsule_id: Option<String>` - Optional capsule ID. If `None`, returns caller's self-capsule info.

**Returns:**

- `Result<CapsuleInfo, Error>` - Basic capsule information or error

**Behavior:**

- Returns lightweight capsule information without full data
- If `capsule_id` is `None`, returns caller's self-capsule
- More efficient than `capsules_read()` for list views

#### Update Capsule

```rust
capsules_update(capsule_id: String, updates: CapsuleUpdateData) -> Result<Capsule, Error>
```

**Parameters:**

- `capsule_id: String` - Unique identifier of the capsule
- `updates: CapsuleUpdateData` - Fields to update

**Returns:**

- `Result<Capsule, Error>` - Updated capsule or error

**Behavior:**

- Updates specified fields in the capsule
- Requires ownership or controller access
- Most fields are immutable (subject, owners, etc.)
- Only certain fields can be updated

#### Delete Capsule

```rust
capsules_delete(capsule_id: String) -> Result<(), Error>
```

**Parameters:**

- `capsule_id: String` - Unique identifier of the capsule

**Returns:**

- `Result<(), Error>` - Success or error

**Behavior:**

- Permanently deletes the capsule and all associated data
- Requires ownership or controller access
- Cannot be undone
- Cascades to memories, galleries, and connections

#### List Capsules

```rust
capsules_list() -> Vec<CapsuleHeader>
```

**Returns:**

- `Vec<CapsuleHeader>` - List of capsule headers

**Behavior:**

- Returns all capsules accessible to the caller
- Includes ownership and control status
- Returns basic information for each capsule
- Efficient for displaying capsule lists

### Personal Canister Operations

#### Create Personal Canister

```rust
create_personal_canister() -> PersonalCanisterCreationResponse
```

**Returns:**

- `PersonalCanisterCreationResponse` - Creation status and canister ID

**Behavior:**

- Initiates creation of a personal canister for the caller
- Returns immediately with status
- Creation happens asynchronously
- Use `get_creation_status()` to track progress

#### Get Creation Status

```rust
get_creation_status(principal: Principal) -> CreationStatusResponse
```

**Parameters:**

- `principal: Principal` - Principal to check status for

**Returns:**

- `CreationStatusResponse` - Current creation status

**Behavior:**

- Returns current status of personal canister creation
- Status can be: `pending`, `in_progress`, `completed`, `failed`
- Includes canister ID when completed
- Includes error message when failed

#### Get Personal Canister ID

```rust
get_personal_canister_id(principal: Principal) -> Result<Principal, Error>
```

**Parameters:**

- `principal: Principal` - Principal to get canister ID for

**Returns:**

- `Result<Principal, Error>` - Canister ID or error

**Behavior:**

- Returns the personal canister ID for a principal
- Returns `NotFound` if no personal canister exists
- Returns `Unauthorized` if caller lacks access

### Binding Operations

#### Bind to Neon Database

```rust
capsules_bind_neon(resource_type: ResourceType, resource_id: String, bind: bool) -> Result<(), Error>
```

**Parameters:**

- `resource_type: ResourceType` - Type of resource to bind
- `resource_id: String` - ID of the resource
- `bind: bool` - Whether to bind or unbind

**Returns:**

- `Result<(), Error>` - Success or error

**Behavior:**

- Binds or unbinds capsule to Neon database
- Enables external storage and retrieval
- Affects data persistence and access patterns
- Requires ownership or controller access

---

## Data Structures

### Core Types

#### Capsule

```rust
pub struct Capsule {
    pub id: String,                                          // unique capsule identifier
    pub subject: PersonRef,                                  // who this capsule is about
    pub owners: HashMap<PersonRef, OwnerState>,              // 1..n owners (usually 1)
    pub controllers: HashMap<PersonRef, ControllerState>,    // delegated admins (full control)
    pub connections: HashMap<PersonRef, Connection>,         // social graph
    pub connection_groups: HashMap<String, ConnectionGroup>, // organized connection groups
    pub memories: HashMap<String, Memory>,                   // content
    pub galleries: HashMap<String, Gallery>,                 // galleries (collections of memories)
    pub created_at: u64,
    pub updated_at: u64,
    pub bound_to_neon: bool,    // Neon database binding status
    pub inline_bytes_used: u64, // Track inline storage consumption
}
```

#### CapsuleInfo

```rust
pub struct CapsuleInfo {
    pub capsule_id: String,
    pub subject_name: String,
    pub subject_type: String,
    pub is_owner: bool,
    pub is_controller: bool,
    pub is_self_capsule: bool,
    pub bound_to_neon: bool,
    pub created_at: u64,
    pub updated_at: u64,
    pub memory_count: u64,
    pub gallery_count: u64,
    pub connection_count: u64,
}
```

#### CapsuleHeader

```rust
pub struct CapsuleHeader {
    pub id: String,
    pub subject_name: String,
    pub subject_type: String,
    pub is_owner: bool,
    pub is_controller: bool,
    pub is_self_capsule: bool,
    pub bound_to_neon: bool,
    pub created_at: u64,
    pub updated_at: u64,
    pub memory_count: u64,
    pub gallery_count: u64,
    pub connection_count: u64,
}
```

#### PersonRef

```rust
pub enum PersonRef {
    Principal(Principal),  // Internet Identity principal
    Opaque(String),       // Non-principal (deceased, org, etc.)
}
```

#### OwnerState

```rust
pub struct OwnerState {
    pub added_at: u64,
    pub added_by: PersonRef,
    pub permissions: Vec<String>,
}
```

#### ControllerState

```rust
pub struct ControllerState {
    pub added_at: u64,
    pub added_by: PersonRef,
    pub permissions: Vec<String>,
}
```

#### Connection

```rust
pub struct Connection {
    pub id: String,
    pub name: String,
    pub relationship: String,
    pub connection_type: String,
    pub added_at: u64,
    pub added_by: PersonRef,
}
```

#### ConnectionGroup

```rust
pub struct ConnectionGroup {
    pub id: String,
    pub name: String,
    pub description: String,
    pub connections: Vec<String>,
    pub created_at: u64,
    pub created_by: PersonRef,
}
```

#### Memory

```rust
pub struct Memory {
    pub id: String,
    pub title: String,
    pub content: String,
    pub memory_type: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub created_by: PersonRef,
    pub tags: Vec<String>,
    pub attachments: Vec<String>,
}
```

#### Gallery

```rust
pub struct Gallery {
    pub id: String,
    pub name: String,
    pub description: String,
    pub items: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub created_by: PersonRef,
}
```

#### PersonalCanisterCreationResponse

```rust
pub struct PersonalCanisterCreationResponse {
    pub status: CreationStatus,
    pub canister_id: Option<Principal>,
    pub error_message: Option<String>,
}
```

#### CreationStatusResponse

```rust
pub struct CreationStatusResponse {
    pub status: CreationStatus,
    pub canister_id: Option<Principal>,
    pub error_message: Option<String>,
    pub progress: Option<u8>, // 0-100
}
```

#### CreationStatus

```rust
pub enum CreationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}
```

#### ResourceType

```rust
pub enum ResourceType {
    User,
    Organization,
    Project,
}
```

#### CapsuleUpdateData

```rust
pub struct CapsuleUpdateData {
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, String>>,
}
```

---

## Error Handling

### Error Types

```rust
pub enum Error {
    Internal(String),           // System errors
    NotFound,                   // Resource not found
    Unauthorized,              // Access denied
    InvalidArgument(String),   // Invalid input
    ResourceExhausted,         // Rate limit exceeded
    NotImplemented(String),    // Feature not available
    Conflict(String),          // Conflicting operation
}
```

### Error Handling Patterns

#### Internal Errors

- **Trigger**: System failures, database errors, network issues
- **Response**: Show generic error message, log details
- **Recovery**: Retry operation, contact support if persistent

#### NotFound Errors

- **Trigger**: Capsule doesn't exist, invalid ID
- **Response**: Show 404 page, redirect to capsule list
- **Recovery**: Verify capsule ID, check permissions

#### Unauthorized Errors

- **Trigger**: Insufficient permissions, expired session
- **Response**: Redirect to login, show access denied message
- **Recovery**: Re-authenticate, check permissions

#### InvalidArgument Errors

- **Trigger**: Invalid input data, validation failures
- **Response**: Show validation errors, highlight invalid fields
- **Recovery**: Fix input data, resubmit form

#### ResourceExhausted Errors

- **Trigger**: Rate limits exceeded, storage limits reached
- **Response**: Show retry message, suggest alternatives
- **Recovery**: Wait and retry, upgrade storage if needed

#### NotImplemented Errors

- **Trigger**: Feature not available, deprecated functionality
- **Response**: Show "coming soon" message, disable feature
- **Recovery**: Use alternative approach, wait for implementation

#### Conflict Errors

- **Trigger**: Conflicting operations, concurrent modifications
- **Response**: Show conflict resolution options
- **Recovery**: Resolve conflict, retry operation

---

## Authorization

### Access Control Model

#### Ownership

- **Owners**: Full control over capsule, can modify, delete, transfer
- **Controllers**: Delegated administrative access, can manage content
- **Connections**: Social graph relationships, limited access

#### Permission Levels

1. **Owner**: Full control

   - Create, read, update, delete capsule
   - Manage owners and controllers
   - Bind/unbind to external systems
   - Transfer ownership

2. **Controller**: Administrative access

   - Read, update capsule content
   - Manage memories and galleries
   - Manage connections
   - Cannot transfer ownership

3. **Connection**: Limited access
   - Read capsule information
   - View public memories
   - Cannot modify content

#### Authorization Rules

- **Self-capsules**: Only the subject can be the owner
- **Other capsules**: Owner can be different from subject
- **Controllers**: Can be added by owners
- **Connections**: Can be added by owners or controllers

---

## Performance Guidelines

### API Usage Patterns

#### Efficient List Operations

- Use `capsules_list()` for displaying capsule lists
- Use `capsules_read_basic()` for basic capsule information
- Use `capsules_read()` only when displaying full capsule details

#### Caching Strategies

- Cache capsule headers for list views
- Cache full capsule data for recently viewed capsules
- Implement cache invalidation on updates

#### Pagination

- Implement pagination for large capsule lists
- Use appropriate page sizes (10-50 items)
- Provide navigation controls

#### Error Handling

- Implement retry logic for transient errors
- Handle offline scenarios gracefully
- Provide meaningful error messages

### Best Practices

#### Data Loading

- Load basic information first, details on demand
- Use loading states to indicate progress
- Implement error boundaries for error handling

#### User Experience

- Provide clear feedback for all operations
- Handle edge cases gracefully
- Implement proper loading and error states

#### Security

- Validate all input data
- Implement proper authentication checks
- Handle authorization errors appropriately

---

## Summary

This documentation provides complete coverage of the ICP Capsule API for frontend implementation. The system supports:

- **Capsule Management**: Full CRUD operations with proper access control
- **Personal Canister Migration**: Seamless data migration to personal canisters
- **Social Graph**: Connections and groups for organizing relationships
- **Flexible Subjects**: Support for live users, deceased persons, and organizations
- **Error Handling**: Comprehensive error handling patterns and recovery strategies
- **Performance**: Best practices for efficient API usage

The API is designed to be intuitive for frontend developers while providing powerful capabilities for managing digital capsules and their associated data.
