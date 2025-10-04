# ICP Capsule API Documentation Request

## Status: 📋 **DOCUMENTATION REQUEST**

**Priority:** High  
**Effort:** Medium  
**Impact:** High - Frontend team needs complete API documentation for capsule management UI

## Problem Statement

The frontend team needs comprehensive documentation of the ICP capsule system to implement proper capsule management components. Currently, we have partial understanding from existing code, but lack complete API documentation covering all capsule operations: **read, write, create, edit, delete, deploy, and bind**.

## Current Frontend Implementation

We have successfully implemented a basic `CapsuleInfo` component that demonstrates:

- Basic capsule information retrieval (`capsules_read_basic`)
- Specific capsule reading (`capsules_read_full`)
- Structured display of capsule data
- Error handling and session management

However, we need complete documentation to build comprehensive capsule management UI.

## API Documentation Request

### 1. **Capsule CRUD Operations**

#### **CREATE Operations**

```rust
// Create a new capsule
capsules_create(subject: Option<PersonRef>) -> Result<Capsule, Error>
```

**Questions for Backend Team:**

- What are the different ways to create a capsule?
- What is the difference between `capsules_create(Some(subject))` vs `capsules_create(None)`?
- What validation is performed on the `subject` parameter?
- What are the ownership rules for newly created capsules?

#### **READ Operations**

```rust
// Read basic capsule information
capsules_read_basic(capsule_id: Option<String>) -> Result<CapsuleInfo, Error>

// Read full capsule data
capsules_read_full(capsule_id: Option<String>) -> Result<Capsule, Error>

// List capsules
capsules_list() -> Vec<CapsuleHeader>
```

**Questions for Backend Team:**

- What's the difference between `CapsuleInfo` and `Capsule` structures?
- When should we use `capsules_read_basic` vs `capsules_read_full`?
- What are the performance implications of each?
- How does pagination work for `capsules_list()`?

#### **UPDATE Operations**

```rust
// Update capsule properties
capsules_update(capsule_id: String, updates: CapsuleUpdateData) -> Result<Capsule, Error>
```

**Questions for Backend Team:**

- What fields can be updated in `CapsuleUpdateData`?
- Are there immutable fields that cannot be changed?
- What are the authorization rules for updates?
- How do we handle partial updates?

#### **DELETE Operations**

```rust
// Delete a capsule
capsules_delete(capsule_id: String) -> Result<(), Error>
```

**Questions for Backend Team:**

- What are the authorization requirements for deletion?
- What happens to associated memories and galleries?
- Is deletion permanent or can it be undone?
- Are there any cascading effects?

### 2. **Capsule Binding Operations**

#### **Neon Database Binding**

```rust
// Bind capsule to Neon database
capsules_bind_neon(resource_type: ResourceType, resource_id: String, bind: bool) -> Result<(), Error>
```

**Questions for Backend Team:**

- What does "binding to Neon" actually mean?
- What are the different `ResourceType` values?
- What are the implications of binding vs unbinding?
- How does this affect data storage and retrieval?

### 3. **Personal Canister Operations**

#### **Personal Canister Creation**

```rust
// Create personal canister for user
create_personal_canister() -> PersonalCanisterCreationResponse
```

**Questions for Backend Team:**

- What is a "personal canister" and how does it relate to capsules?
- What is the `PersonalCanisterCreationResponse` structure?
- What are the different creation statuses?
- How long does the creation process take?
- What are the costs and resource requirements?

#### **Creation Status Tracking**

```rust
// Check creation status
get_creation_status(principal: Principal) -> CreationStatusResponse
```

**Questions for Backend Team:**

- How do we track the progress of canister creation?
- What are all the possible `CreationStatus` values?
- How do we handle failed creations?
- Can users retry failed creations?

### 4. **Data Structure Documentation**

#### **Core Capsule Types**

**CapsuleInfo Structure:**

```rust
struct CapsuleInfo {
    capsule_id: String,
    subject: PersonRef,
    is_owner: bool,
    is_controller: bool,
    is_self_capsule: bool,
    bound_to_neon: bool,
    created_at: u64,
    updated_at: u64,
    memory_count: u64,
    gallery_count: u64,
    connection_count: u64,
}
```

**Capsule Structure:**

```rust
struct Capsule {
    id: String,
    subject: PersonRef,
    owners: HashMap<PersonRef, OwnerState>,
    controllers: HashMap<PersonRef, ControllerState>,
    connections: HashMap<PersonRef, Connection>,
    connection_groups: HashMap<String, ConnectionGroup>,
    memories: HashMap<String, Memory>,
    galleries: HashMap<String, Gallery>,
    created_at: u64,
    updated_at: u64,
    bound_to_neon: bool,
    inline_bytes_used: u64,
}
```

**Questions for Backend Team:**

- What is the difference between `owners` and `controllers`?
- How do `connections` and `connection_groups` work?
- What are the relationships between capsules, memories, and galleries?
- How is `inline_bytes_used` calculated?

#### **PersonRef Types**

```rust
enum PersonRef {
    Principal(Principal),
    Opaque(String),
}
```

**Questions for Backend Team:**

- When do we use `Principal` vs `Opaque`?
- What does the `Opaque` variant represent?
- How do we handle different identity types?

### 5. **Error Handling Documentation**

#### **Error Types**

```rust
enum Error {
    Internal(String),
    NotFound,
    Unauthorized,
    InvalidArgument(String),
    ResourceExhausted,
    NotImplemented(String),
    Conflict(String),
}
```

**Questions for Backend Team:**

- What triggers each error type?
- How do we handle authentication/authorization errors?
- What are the retry strategies for different errors?
- How do we provide user-friendly error messages?

### 6. **Frontend Component Requirements**

Based on our analysis, we need to build:

#### **Capsule Management Dashboard**

- List all user's capsules
- Create new capsules
- View capsule details
- Edit capsule properties
- Delete capsules
- Bind/unbind to Neon

#### **Personal Canister Management**

- Initiate personal canister creation
- Track creation progress
- View creation status
- Handle failed creations
- Manage canister lifecycle

#### **Capsule Details View**

- Display capsule information
- Show ownership and control status
- Display memory and gallery counts
- Show connection information
- Manage capsule settings

### 7. **Specific Questions for Backend Team**

1. **Authorization Model**: What are the exact rules for who can read/write/delete capsules?

2. **Data Relationships**: How do capsules relate to memories, galleries, and connections?

3. **Personal Canisters**: What is the complete lifecycle of personal canister creation?

4. **Binding Operations**: What does binding to Neon actually do, and what are the alternatives?

5. **Performance**: What are the performance characteristics of different operations?

6. **Error Recovery**: How do we handle and recover from various error conditions?

7. **Real-time Updates**: Are there any real-time notifications for capsule changes?

8. **Bulk Operations**: Are there any bulk operations for managing multiple capsules?

## Expected Deliverables

1. **Complete API Reference**: All endpoints with parameters, return types, and examples
2. **Data Structure Documentation**: Complete type definitions with field descriptions
3. **Error Handling Guide**: Comprehensive error handling patterns and recovery strategies
4. **Authorization Matrix**: Clear rules for who can perform what operations
5. **Performance Guidelines**: Best practices for efficient API usage
6. **Example Responses**: Sample API responses for different scenarios

## Frontend Implementation Plan

Once we receive the documentation, we plan to implement:

1. **Enhanced CapsuleInfo Component**: Complete capsule management functionality
2. **CapsuleList Component**: Browse and manage multiple capsules
3. **PersonalCanisterManager Component**: Handle personal canister creation and management
4. **CapsuleEditor Component**: Edit capsule properties and settings
5. **CapsuleBindingManager Component**: Manage Neon binding and storage preferences

## Timeline

- **Documentation Request**: Immediate
- **Backend Team Response**: Within 1 week
- **Frontend Implementation**: 2-3 weeks after documentation received
- **Testing and Integration**: 1 week

## Contact

**Frontend Team Lead**: [Name]  
**Backend Team Lead**: [Name]  
**Project Manager**: [Name]

---

_This documentation request is critical for implementing comprehensive capsule management functionality in the frontend. Please prioritize providing complete and accurate API documentation._
