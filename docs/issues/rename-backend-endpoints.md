# Rename Backend Functions to Follow Professional ICP Naming Conventions

## Issue Description

The current backend functions use amateur-sounding names that don't follow professional ICP function naming conventions, making the API feel inconsistent and hard to understand. This refactor will establish a clear, REST-readable naming scheme while maintaining ICP-native patterns.

**Key Discovery**: The backend includes extensive personal canister creation capabilities, allowing users to build their own canisters with their capsule data. This is a major feature that needs proper API design and naming.

## Current → New Function Mapping

### Capsules

```
list_my_capsules           → capsules_list()                   [query]   // caller's own capsules
(admin only)               → capsules_list_all()               [query]   // all capsules in system
(optional)                 → capsules_list_by_owner(principal) [query]   // role-gated cross-account
get_capsule(id)            → capsules_read(id)                 [query]
create_capsule(subject)    → capsules_create(Some(subject))    [update]
register_capsule()         → capsules_create(None)             [update]  // self-registration
```

### Galleries

```
get_my_galleries           → galleries_list                    [query]
get_gallery_by_id(id)      → galleries_read(id)                [query]
store_gallery_forever(d)   → galleries_create(d)               [update]
update_gallery(id, patch)  → galleries_update(id, patch)       [update]
delete_gallery(id)         → galleries_delete(id)               [update]
```

### Memories

```
list_capsule_memories()    → memories_list(capsule_id)         [query] ✅ REMOVED
add_memory_to_capsule(..)  → memories_create(..)               [update]
get_memory_from_capsule(id)→ memories_read(id)                 [query]
update_memory_in_capsule   → memories_update(id, patch)        [update]
delete_memory_from_capsule → memories_delete(id)               [update]
```

### Authentication & Identity

```
register                   → capsules_create(None)              [update]  // self-registration
mark_bound                 → capsules_bind_neon()              [update]  // bind to neon backend
whoami                     → whoami                            [query]   // unchanged - classic API function
verify_nonce(nonce)        → capsules_verify_nonce(nonce)      [update]
```

## Updated Function Map (IC-native, REST-readable)

### Capsules

- `capsules_list()` – caller-scoped
- `capsules_list_all()` – admin-only
- `capsules_list_by_owner(owner: Principal)` – admin-only (optional)
- `capsules_read(id)`
- `capsules_create(subject: Option<PersonRef>)`
- `capsules_bind_neon()` – bind the caller's capsule to Neon
- `capsules_bind_neon_for(capsule_id)` – admin-only (optional)

### Memories

- `memories_list(capsule_id, …)`
- `memories_create(capsule_id, data)`
- `memories_read(id)`
- `memories_update(id, patch)`
- `memories_delete(id)`

### Galleries

- `galleries_list()`
- `galleries_read(id)`
- `galleries_create(data)`
- `galleries_update(id, patch)`
- `galleries_delete(id)`
- `galleries_list_for_capsule(capsule_id, …)` (optional)

### Auth / Identity

- `whoami()`
- `auth_register`
- `auth_nonce_verify(nonce)`

### Personal Canister Creation

- `canisters_create_personal()` – create personal canister for caller's capsule
- `canisters_get_personal_id()` – get caller's personal canister ID
- `canisters_get_personal_id_for(user)` – admin: get personal canister ID for specific user
- `canisters_get_creation_status()` – get caller's canister creation status
- `canisters_get_detailed_status()` – get detailed creation status with progress
- `canisters_migrate_capsule()` – legacy: migrate capsule to personal canister

## Unified Create Function: Single Endpoint for All Capsule Creation

**Decision**: Consolidate capsule creation into a single function with optional parameters instead of separate create/register functions.

### Single Function Approach:

```rust
#[update(name="capsules_create")]
fn capsules_create(subject: Option<PersonRef>) -> CapsuleCreationResult {
    match subject {
        Some(subject) => {
            // Create capsule for someone else (admin/creator role)
            create_capsule_for_other(subject)
        }
        None => {
            // Create capsule for self (self-registration)
            create_capsule_for_self()
        }
    }
}
```

### Benefits:

1. **One endpoint** instead of two confusing ones
2. **Clear parameter meaning** - `subject: None` = self, `subject: Some(person)` = for other
3. **Simpler API** - less cognitive overhead
4. **More flexible** - easy to extend with additional optional parameters
5. **Eliminates confusion** between create vs register semantics

### Usage Examples:

```rust
// Self-registration
capsules_create(None)

// Create for someone else
capsules_create(Some(PersonRef::from_principal(other_principal)))
```

- **Cons**: Users have to call two functions instead of one

## Pagination & Filters

### Default Limits

- **Default limit**: 50 items per page
- **Maximum limit**: 100 items per page
- **Stable sort key**: `created_at desc, id desc` (deterministic pagination)

### Filter Options

- **Capsules**: by owner, creation date, status
- **Galleries**: by capsule, visibility, creation date
- **Memories**: by type, date, capsule

## Minimal Signatures (Rust/Candid sketch)

```rust
#[query(name="capsules_list")]
fn capsules_list(p: Option<Pagination>, f: Option<CapsuleFilter>) -> Result<Vec<CapsuleDto>, ApiError>;

#[query(name="capsules_list_all")]
fn capsules_list_all(p: Option<Pagination>, f: Option<CapsuleFilter>) -> Result<Vec<CapsuleDto>, ApiError>;

#[query(name="capsules_list_by_owner")]
fn capsules_list_by_owner(owner: Principal, p: Option<Pagination>, f: Option<CapsuleFilter>) -> Result<Vec<CapsuleDto>, ApiError>;

#[query(name="capsules_read")]
fn capsules_read(id: CapsuleId) -> Result<CapsuleDto, ApiError>;

#[update(name="capsules_create")]
fn capsules_create(subject: Option<PersonRef>) -> Result<CapsuleId, ApiError>;

#[update(name="capsules_bind_neon")]
fn capsules_bind_neon(input: NeonBindInput) -> Result<NeonBindResult, ApiError>; // caller's capsule

// Optional admin variant
#[update(name="capsules_bind_neon_for")]
fn capsules_bind_neon_for(capsule_id: CapsuleId, input: NeonBindInput) -> Result<NeonBindResult, ApiError>;

#[update(name="auth_register")]
fn auth_register(input: AuthRegisterInput) -> Result<AuthRegisterResult, ApiError>;

#[update(name="auth_nonce_verify")]
fn auth_nonce_verify(nonce: Text) -> Result<VerifyResult, ApiError>;

#[query(name="whoami")]
fn whoami() -> Principal;
```

Types -you'll likely want:

```rust
struct NeonBindInput { api_key: Text, project_id: Text, scope: Option<Text> }
struct NeonBindResult { bound: bool, bound_at: Timestamp, scope: Option<Text> }
```

## Role Matrix (with Neon binding)

| Function                        | Admin          | User (caller)         | Service |
| ------------------------------- | -------------- | --------------------- | ------- |
| `capsules_list`                 | ✅ own         | ✅ own                | ❌      |
| `capsules_list_all`             | ✅ all         | ❌                    | ❌      |
| `capsules_list_by_owner`        | ✅ any         | ❌                    | ❌      |
| `capsules_create`               | ✅             | ✅                    | ❌      |
| `capsules_bind_neon`            | ❌ n/a         | ✅ (caller's capsule) | ❌      |
| `capsules_bind_neon_for`        | ✅ any capsule | ❌                    | ❌      |
| `galleries_list`                | ❌ n/a         | ✅ own                | ❌      |
| `whoami`                        | ✅             | ✅                    | ✅      |
| `auth_register`                 | ✅             | ✅                    | ❌      |
| `auth_nonce_verify`             | ✅             | ✅                    | ❌      |
| `canisters_create_personal`     | ❌ n/a         | ✅ (caller's capsule) | ❌      |
| `canisters_get_personal_id`     | ❌ n/a         | ✅ (caller's capsule) | ❌      |
| `canisters_get_personal_id_for` | ✅ any user    | ❌                    | ❌      |

## Deprecation Strategy

1. **Phase 1**: Add new functions alongside old ones
2. **Phase 2**: Mark old functions as deprecated
3. **Phase 3**: Remove old functions in next major version
4. **Migration guide**: Provide examples of old → new calls

### Deprecation Headers

- Include `deprecation: Option<Text>` field in Result errors for old entrypoints
- Log deprecation warnings to stderr during development

## Security & Idempotency

### Security

- **No principal args for caller-scoped reads**; use `ic_cdk::caller()` internally
- **Admin functions** require explicit admin role checks
- **Cross-account queries** only available to admins

### Security & Idempotency notes (binding)

- `capsules_bind_neon` is **caller-scoped**: resolve the caller's capsule via `ic_cdk::caller()`; no principal arg.
- Store a **hashed** representation of secrets (if possible) or encrypt at rest; never log credentials.
- Make binding **idempotent** by upserting on `(capsule_id, project_id)` and returning the existing binding if identical.

### Idempotency Table

| Function                 | Safe to Retry | Notes                                                     |
| ------------------------ | ------------- | --------------------------------------------------------- |
| `capsules_register`      | ✅            | Creates if not exists, updates if exists                  |
| `auth_nonce_verify`      | ✅            | Consumes nonce on first call, returns error on subsequent |
| `galleries_create`       | ❌            | May create duplicates                                     |
| `memories_create`        | ❌            | May create duplicates                                     |
| `capsules_create`        | ❌            | May create duplicates                                     |
| `capsules_bind_neon`     | ✅            | Upsert by `(capsule, project_id)`                         |
| `capsules_bind_neon_for` | ✅            | Same, admin-only                                          |

## Telemetry & Audit

### Admin Access Logging

- **Function**: `capsules_list`
- **Log**: Principal, cursor, filter, timestamp
- **Format**: `ADMIN_ACCESS: principal={}, cursor={}, filter={}, ts={}`

### Audit Events

- **Create**: `CAPSULE_CREATED: id={}, owner={}, subject={}, ts={}`
- **Register**: `CAPSULE_REGISTERED: id={}, owner={}, ts={}`
- **Delete**: `CAPSULE_DELETED: id={}, owner={}, ts={}, reason={}`

### Metrics

- Function call counts by role
- Response time percentiles
- Error rate by error type

## Implementation Priority

### Phase 1: Core Functions (Week 1)

- [ ] `capsules_list`, `capsules_read`, `capsules_create`
- [ ] `galleries_list`, `galleries_read`, `galleries_create`
- [ ] `whoami` (verify unchanged)

### Phase 2: Management Functions (Week 2)

- [ ] `capsules_register` (after decision on create vs register)
- [ ] `galleries_update`, `galleries_delete`
- [ ] `memories_*` functions

### Phase 3: Admin & Advanced (Week 3)

- [ ] `capsules_list_all` (admin-only)
- [ ] `capsules_list_by_owner` (admin-only, optional)
- [ ] `auth_register`, `auth_nonce_verify`
- [ ] Personal canister creation functions
- [ ] Admin personal canister management functions
- [ ] Error schema implementation

### Phase 4: Cleanup (Week 4)

- [ ] Remove deprecated functions
- [ ] Update documentation
- [ ] Performance testing

## Open Questions (To be answered by code review)

1. **Does `register` mint credentials/tokens or simply flip a flag?**

   - Check if `register_capsule()` creates new access tokens
   - Verify if it binds additional capabilities

2. **Are there per-user quotas enforced at create or register?**

   - Check quota validation logic
   - Determine if quotas apply to creation or activation

3. **Are capsules transferable? If yes, how does `register` handle ownership?**

   - Verify ownership transfer mechanisms
   - Check if `register` can change capsule ownership

4. **What side-effects does `register` have beyond state changes?**
   - Webhook notifications
   - Event emissions
   - External service calls

## Benefits of Professional Function Naming

1. **Professional Appearance** - Follows ICP development standards
2. **Developer Experience** - Predictable and intuitive function names
3. **Consistency** - Uniform naming patterns across all functions
4. **Maintainability** - Easier to understand and work with
5. **Documentation** - Self-documenting function purposes
6. **Role-based Access Control** - Clear separation of admin vs user functions
7. **Audit Trail** - Comprehensive logging and monitoring
8. **API Stability** - Clear deprecation and versioning strategy

## Migration Table

| Old Function                         | New Function                          | Status     | Removal |
| ------------------------------------ | ------------------------------------- | ---------- | ------- |
| `list_my_capsules()`                 | `capsules_list()`                     | Deprecated | v0.8    |
| `get_capsule(id)`                    | `capsules_read(id)`                   | Deprecated | v0.8    |
| `create_capsule(subject)`            | `capsules_create(Some(subject))`      | Deprecated | v0.8    |
| `register_capsule()`                 | `capsules_create(None)`               | Deprecated | v0.8    |
| `get_my_galleries()`                 | `galleries_list()`                    | Deprecated | v0.8    |
| `get_gallery_by_id(id)`              | `galleries_read(id)`                  | Deprecated | v0.8    |
| `store_gallery_forever(data)`        | `galleries_create(data)`              | Deprecated | v0.8    |
| `update_gallery(id, data)`           | `galleries_update(id, data)`          | Deprecated | v0.8    |
| `delete_gallery(id)`                 | `galleries_delete(id)`                | Deprecated | v0.8    |
| `list_capsule_memories()`            | `memories_list(capsule_id)`           | ✅ REMOVED | v0.7    |
| `add_memory_to_capsule(id, data)`    | `memories_create(id, data)`           | Deprecated | v0.8    |
| `get_memory_from_capsule(id)`        | `memories_read(id)`                   | Deprecated | v0.8    |
| `update_memory_in_capsule(id, data)` | `memories_update(id, data)`           | Deprecated | v0.8    |
| `delete_memory_from_capsule(id)`     | `memories_delete(id)`                 | Deprecated | v0.8    |
| `register()`                         | `auth_register`                       | Deprecated | v0.8    |
| `mark_bound()`                       | `capsules_bind_neon()`                | Deprecated | v0.8    |
| `verify_nonce(nonce)`                | `auth_nonce_verify(nonce)`            | Deprecated | v0.8    |
| `whoami()`                           | `whoami()`                            | —          | —       |
| `create_personal_canister()`         | `canisters_create_personal()`         | Deprecated | v0.8    |
| `get_my_personal_canister_id()`      | `canisters_get_personal_id()`         | Deprecated | v0.8    |
| `get_personal_canister_id(user)`     | `canisters_get_personal_id_for(user)` | Deprecated | v0.8    |
| `get_creation_status()`              | `canisters_get_creation_status()`     | Deprecated | v0.8    |
| `get_detailed_creation_status()`     | `canisters_get_detailed_status()`     | Deprecated | v0.8    |
| `migrate_capsule()`                  | `canisters_migrate_capsule()`         | Deprecated | v0.8    |

## Related Issues

- [ ] Add proper HTTP status codes for error responses
- [ ] Evaluate adding API versioning strategy
- [ ] Review error response formats and consistency
- [ ] Implement comprehensive logging and monitoring
- [ ] Design rate limiting and quota management
- [ ] Plan for future API extensions and backward compatibility

## Future Considerations

### Vendor-Agnostic Binding

If you want this to be vendor-agnostic later, keep the method name but add a `provider` field in `NeonBindInput` (e.g., `"neon"`), so you can support others without more endpoints.
