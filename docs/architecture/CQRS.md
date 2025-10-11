# CQRS (Command Query Responsibility Segregation) Pattern

**Status**: Reference Documentation  
**Created**: 2024-12-19  
**Source**: Architecture Documentation

## Overview

CQRS (Command Query Responsibility Segregation) is a design pattern that separates read and write operations for a data store. In the context of our ICP (Internet Computer Protocol) application, CQRS provides a clean architectural foundation that aligns perfectly with ICP's `#[query]` and `#[update]` function annotations.

## Core Principles

### 1. **Separation of Concerns**

- **Commands**: Operations that change state (writes)
- **Queries**: Operations that read state (reads)
- **No overlap**: A single operation should either read OR write, never both

### 2. **ICP Alignment**

- **`#[query]` functions**: Pure read operations, no side effects
- **`#[update]` functions**: State-changing operations, can have side effects
- **Consensus alignment**: Queries are fast and don't require consensus; updates do

### 3. **Performance Benefits**

- **Optimized reads**: Query models can be denormalized for fast access
- **Optimized writes**: Command models focus on business rules and validation
- **Independent scaling**: Read and write operations can be scaled independently

## Implementation in Our Codebase

### **Command Side (`commands.rs`)**

```rust
// Write operations that change state
pub async fn capsules_create(data: CapsuleCreateData) -> Result<CapsuleId, CapsuleError> {
    // Business logic validation
    // State mutation
    // Repository persistence
}

pub async fn capsules_update(id: CapsuleId, data: CapsuleUpdateData) -> Result<(), CapsuleError> {
    // Update business rules
    // State modification
    // Persistence
}

pub async fn capsules_delete(id: CapsuleId) -> Result<(), CapsuleError> {
    // Deletion logic
    // State cleanup
    // Repository update
}
```

**Characteristics:**

- State-changing operations
- Business rule enforcement
- Validation and authorization
- Repository persistence
- Return minimal data (IDs, success/failure)

### **Query Side (`query.rs`)**

```rust
// Read operations that don't change state
pub async fn capsules_read(id: CapsuleId) -> Result<CapsuleInfo, CapsuleError> {
    // Pure data retrieval
    // No state modification
    // Return formatted data
}

pub async fn capsules_list(filters: ListFilters) -> Result<Vec<CapsuleHeader>, CapsuleError> {
    // Data selection
    // Projection/transformation
    // No side effects
}

pub async fn get_user_settings(user_id: UserId) -> Result<UserSettings, CapsuleError> {
    // Read-only access
    // Data formatting
    // No state changes
}
```

**Characteristics:**

- Pure read operations
- No side effects
- Data projection and formatting
- Fast execution
- Rich return data

## Benefits in ICP Context

### **1. Performance Optimization**

- **Query functions** (`#[query]`): Execute locally, no consensus required
- **Update functions** (`#[update]`): Require consensus but handle complex operations
- **Certification**: Read-side certification for data integrity

### **2. Scalability**

- **Independent scaling**: Read and write operations can be optimized separately
- **Caching**: Query results can be cached without affecting write operations
- **Load distribution**: Different strategies for read vs write workloads

### **3. Maintainability**

- **Clear boundaries**: Easy to understand what each function does
- **Testing**: Commands and queries can be tested independently
- **Debugging**: Clear separation makes issues easier to isolate

## Module Structure

```
src/backend/src/capsule/
├── commands.rs          // Write operations (5 functions)
│   ├── capsules_create()
│   ├── capsules_update()
│   ├── capsules_delete()
│   ├── resources_bind_neon()
│   └── update_user_settings()
│
├── query.rs             // Read operations (6 functions)
│   ├── capsules_read()
│   ├── capsules_read_basic()
│   ├── capsule_read_self()
│   ├── capsules_list()
│   └── get_user_settings()
│
└── domain.rs            // Shared business logic
    ├── Capsule struct
    ├── Access control
    └── Business rules
```

## Best Practices

### **Commands (Write Operations)**

✅ **Do:**

- Validate input data thoroughly
- Enforce business rules
- Handle authorization checks
- Return minimal data (IDs, success/failure)
- Use idempotent operations when possible
- Update projections/views after state changes

❌ **Don't:**

- Return large data structures
- Perform complex data transformations
- Mix read and write operations
- Skip validation or authorization

### **Queries (Read Operations)**

✅ **Do:**

- Keep operations pure (no side effects)
- Return rich, formatted data
- Use projections for performance
- Cache results when appropriate
- Handle pagination for large datasets

❌ **Don't:**

- Modify state
- Call external services that change state
- Perform complex business logic
- Return raw database objects

## Error Handling

### **Command Errors**

```rust
pub enum CapsuleError {
    ValidationError(String),
    AuthorizationError(String),
    NotFound(String),
    Conflict(String),
    InternalError(String),
}
```

### **Query Errors**

```rust
pub enum QueryError {
    NotFound(String),
    AccessDenied(String),
    InvalidParameters(String),
    InternalError(String),
}
```

## Data Flow

### **Command Flow**

```
Client Request → Command Handler → Domain Logic → Repository → State Update → Response
```

### **Query Flow**

```
Client Request → Query Handler → Repository → Data Projection → Response
```

## CQRS vs CRUD

### **Traditional CRUD Approach**

CRUD (Create, Read, Update, Delete) is the traditional approach where a single model serves both read and write operations:

```rust
// Traditional CRUD - Single model for everything
pub struct CapsuleService {
    repository: CapsuleRepository,
}

impl CapsuleService {
    // All operations use the same model
    pub async fn create(&self, data: CapsuleData) -> Capsule { }
    pub async fn read(&self, id: CapsuleId) -> Capsule { }
    pub async fn update(&self, id: CapsuleId, data: CapsuleData) -> Capsule { }
    pub async fn delete(&self, id: CapsuleId) -> () { }
}
```

**CRUD Characteristics:**

- Single model for all operations
- Same data structure for reads and writes
- Simple to understand and implement
- Good for simple applications
- Limited optimization opportunities

### **CQRS Approach**

CQRS separates read and write models, allowing each to be optimized for its specific purpose:

```rust
// CQRS - Separate models for commands and queries
pub struct CapsuleCommands {
    repository: CapsuleRepository,
}

pub struct CapsuleQueries {
    repository: CapsuleRepository,
    projection_cache: ProjectionCache,
}

impl CapsuleCommands {
    // Write operations with business logic
    pub async fn create(&self, data: CreateCapsuleCommand) -> Result<CapsuleId, Error> { }
    pub async fn update(&self, id: CapsuleId, data: UpdateCapsuleCommand) -> Result<(), Error> { }
    pub async fn delete(&self, id: CapsuleId) -> Result<(), Error> { }
}

impl CapsuleQueries {
    // Read operations with optimized projections
    pub async fn get_capsule(&self, id: CapsuleId) -> Result<CapsuleView, Error> { }
    pub async fn list_capsules(&self, filters: ListFilters) -> Result<Vec<CapsuleSummary>, Error> { }
    pub async fn get_capsule_analytics(&self, id: CapsuleId) -> Result<CapsuleAnalytics, Error> { }
}
```

**CQRS Characteristics:**

- Separate models for reads and writes
- Optimized data structures for each operation
- More complex but more flexible
- Better performance for complex applications
- Enables advanced patterns (event sourcing, projections)

### **Comparison Table**

| Aspect             | CRUD                 | CQRS                       |
| ------------------ | -------------------- | -------------------------- |
| **Complexity**     | Simple               | More complex               |
| **Models**         | Single model         | Separate read/write models |
| **Performance**    | Limited optimization | Highly optimized           |
| **Scalability**    | Monolithic scaling   | Independent scaling        |
| **Flexibility**    | Rigid                | Very flexible              |
| **Learning Curve** | Easy                 | Steeper                    |
| **Use Case**       | Simple applications  | Complex applications       |

### **When to Use CRUD**

✅ **CRUD is suitable when:**

- Simple data operations
- Small to medium applications
- Team is new to complex patterns
- Performance requirements are modest
- Data model is stable and simple

**Example CRUD Use Case:**

```rust
// Simple user profile management
pub struct UserProfile {
    id: UserId,
    name: String,
    email: String,
    created_at: Timestamp,
}

// CRUD operations are sufficient
impl UserService {
    pub async fn create_user(&self, data: CreateUserData) -> UserProfile { }
    pub async fn get_user(&self, id: UserId) -> UserProfile { }
    pub async fn update_user(&self, id: UserId, data: UpdateUserData) -> UserProfile { }
    pub async fn delete_user(&self, id: UserId) -> () { }
}
```

### **When to Use CQRS**

✅ **CQRS is beneficial when:**

- Complex business logic
- High performance requirements
- Different read and write patterns
- Need for audit trails
- Complex authorization requirements
- Event-driven architecture

**Example CQRS Use Case:**

```rust
// Complex capsule management with analytics
pub struct CreateCapsuleCommand {
    owner_id: UserId,
    title: String,
    content: CapsuleContent,
    access_policy: AccessPolicy,
    metadata: CapsuleMetadata,
}

pub struct CapsuleView {
    id: CapsuleId,
    title: String,
    summary: String,
    access_level: AccessLevel,
    analytics: CapsuleAnalytics,
    related_capsules: Vec<CapsuleSummary>,
}

// CQRS enables optimized operations
impl CapsuleCommands {
    pub async fn create_capsule(&self, cmd: CreateCapsuleCommand) -> Result<CapsuleId, Error> {
        // Complex business logic
        // Access control validation
        // Content processing
        // Event generation
    }
}

impl CapsuleQueries {
    pub async fn get_capsule_view(&self, id: CapsuleId) -> Result<CapsuleView, Error> {
        // Optimized read with projections
        // Cached analytics data
        // Related content lookup
    }
}
```

### **Migration from CRUD to CQRS**

If you're considering migrating from CRUD to CQRS:

1. **Identify pain points:**

   - Performance bottlenecks in reads
   - Complex business logic in simple operations
   - Different data needs for reads vs writes

2. **Start gradually:**

   - Begin with one domain/entity
   - Extract read operations first
   - Then extract write operations
   - Validate the approach before expanding

3. **Our current approach:**
   - Started with capsule module
   - Separated commands and queries
   - Maintained domain logic in shared module
   - Applied to other modules as needed

### **Hybrid Approach**

You can also use a hybrid approach where some modules use CRUD and others use CQRS:

```rust
// Simple modules use CRUD
pub mod user_profile;  // CRUD approach

// Complex modules use CQRS
pub mod capsule;       // CQRS approach
pub mod memory;        // CQRS approach
pub mod analytics;     // CQRS approach
```

This allows you to apply the right pattern to each domain based on its complexity and requirements.

## Integration with Other Patterns

### **Repository Pattern**

- Commands use repositories for persistence
- Queries use repositories for data retrieval
- Clean separation between data access and business logic

### **Domain-Driven Design**

- Commands represent business operations
- Queries represent data access needs
- Domain models shared between both sides

### **Event Sourcing** (Future)

- Commands generate events
- Queries can read from event streams
- Enables audit trails and replay capabilities

## Monitoring and Observability

### **Command Metrics**

- Execution time
- Success/failure rates
- Business rule violations
- Authorization failures

### **Query Metrics**

- Response times
- Cache hit rates
- Data access patterns
- Performance bottlenecks

## Migration Strategy

When implementing CQRS in existing code:

1. **Identify operations**: Separate read vs write operations
2. **Extract commands**: Move write operations to `commands.rs`
3. **Extract queries**: Move read operations to `query.rs`
4. **Refactor domain**: Move shared logic to `domain.rs`
5. **Update API**: Ensure proper `#[query]`/`#[update]` annotations
6. **Test thoroughly**: Verify no side effects in queries

## Common Pitfalls

### **1. Query Side Effects**

```rust
// ❌ BAD: Query with side effects
#[query]
pub async fn get_capsule_with_analytics(id: CapsuleId) -> CapsuleInfo {
    let capsule = repository.get(id);
    analytics.track_view(id); // Side effect in query!
    capsule
}

// ✅ GOOD: Separate query and command
#[query]
pub async fn get_capsule(id: CapsuleId) -> CapsuleInfo {
    repository.get(id)
}

#[update]
pub async fn track_capsule_view(id: CapsuleId) -> Result<(), Error> {
    analytics.track_view(id);
    Ok(())
}
```

### **2. Command Data Leakage**

```rust
// ❌ BAD: Command returning too much data
#[update]
pub async fn create_capsule(data: CreateData) -> CapsuleInfo {
    let capsule = repository.create(data);
    // ... complex formatting logic
    capsule // Too much data returned
}

// ✅ GOOD: Command returns minimal data
#[update]
pub async fn create_capsule(data: CreateData) -> Result<CapsuleId, Error> {
    let id = repository.create(data);
    Ok(id)
}
```

## Future Enhancements

### **1. Event Sourcing**

- Commands generate domain events
- Queries can read from event streams
- Enable audit trails and replay

### **2. Read Models**

- Denormalized views for specific query needs
- Optimized data structures for common queries
- Materialized views for complex aggregations

### **3. Caching Strategy**

- Query result caching
- Invalidation on command execution
- Distributed cache for multi-canister setups

## References

- [CQRS Pattern - Martin Fowler](https://martinfowler.com/bliki/CQRS.html)
- [ICP Developer Documentation](https://internetcomputer.org/docs/current/developer-docs/)
- [Domain-Driven Design](https://martinfowler.com/bliki/DomainDrivenDesign.html)
- [Repository Pattern](https://martinfowler.com/eaaCatalog/repository.html)

## Related Documents

- [Capsule Module Architecture](./capsule-module-architecture.md)
- [Backend API Documentation](./backend-api-documentation.md)
- [Testing Strategy ICP](./testing-strategy-icp.md)
