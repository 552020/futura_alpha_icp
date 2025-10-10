# Capsule Module Architecture

**Status**: Approved  
**Created**: 2024-10-10  
**Source**: Tech Lead Architecture Review

## Overview

This document outlines the recommended architecture for the capsule module, following both Rust community best practices and ICP (Internet Computer Protocol) conventions.

## Core Principles

### 1. **Rust Community Best Practices**

- **Modules by responsibility/domain**, not "god" files like `types.rs` or `utils.rs`
- **Facade file** (`capsule.rs`) that re-exports public surface
- **Keep behavior with data**: put methods/impls alongside the types they operate on
- **Boundary types separate**: DTOs/API shapes live in separate modules
- **No `mod.rs`** unless preferred—modern Rust prefers `foo.rs` + `foo/` folder

### 2. **ICP (Rust Canister) Conventions**

- **CQRS maps to ICP**: `#[query]` is reads; `#[update]` is writes
- **Domain → Repo → API layering**:
  - `domain/` — pure Rust (no `ic_cdk`), business rules, permission evaluation
  - `repo/` — persistence (stable structures, migrations, pagination)
  - `api/` — candid-facing functions, caller checks, input validation, mapping errors
- **State & upgrades**: central state in single `State` struct; isolate (de)serialization & versioning in `repo`
- **Stable memory safety**: avoid storing large `Vec` blobs; prefer maps/sharding; cap sizes per key
- **Certification** for hot reads: keep it on the query side only
- **Idempotent updates + version counters** for retries
- **No hidden writes in `#[query]`**; no `ic_cdk::api::time()` or randomness that mutates state in reads

## Recommended Module Structure

```
src/backend/src/
├── lib.rs
├── capsule.rs                // Facade: declares submodules (no re-exports)
└── capsule/
    ├── domain.rs             // Capsule struct, impls, access logic (pure)
    ├── commands.rs           // write handlers; call repo; update projections
    ├── query.rs              // read selectors; mostly pure
    ├── api_types.rs          // request/response/DTOs (Candid/serde)
    └── util.rs               // helper functions, size calculation, migrations
```

## Current Implementation Status

**✅ IMPLEMENTED:**

- **Facade Pattern**: `capsule.rs` declares submodules, no re-exports (clean API surface)
- **Domain Separation**: `domain.rs` contains core business logic, access control, and bitflags
- **CQRS Pattern**: `commands.rs` (5 write functions) vs `query.rs` (6 read functions)
- **API Types**: `api_types.rs` for DTOs separate from domain types
- **Utility Functions**: `util.rs` for helper functions and migration support

**📊 Current Module Breakdown:**

| Module             | Functions   | Types    | Purpose                                         |
| ------------------ | ----------- | -------- | ----------------------------------------------- |
| **`domain.rs`**    | 1 function  | 15 types | Core business logic, access control, bitflags   |
| **`commands.rs`**  | 5 functions | 0 types  | Write operations (create, update, delete, bind) |
| **`query.rs`**     | 6 functions | 0 types  | Read operations (read, list, settings)          |
| **`api_types.rs`** | 0 functions | 6 types  | Request/Response DTOs                           |
| **`util.rs`**      | 5 functions | 0 types  | Helper functions, size calculation, migration   |

**🎯 Key Architectural Decisions Made:**

1. **No Re-exports**: `capsule.rs` only declares modules, doesn't re-export (cleaner API surface)
2. **Access Control Integration**: Bitflags and access types moved to `domain.rs` (Task 1.1-1.2 ✅)
3. **Pure Domain Logic**: `domain.rs` contains business logic without `ic_cdk` dependencies
4. **CQRS Alignment**: Clear separation between commands (writes) and queries (reads)
5. **Migration Support**: `util.rs` contains upgrade/import functions

**📋 Next Steps (From Access Refactoring Plan):**

- Task 1.4: Implement `AccessControlled` trait in `domain.rs`
- Task 1.5: Add access fields to Memory/Gallery structs
- Task 1.6: Optional magic link index (if needed)

## Module Responsibilities

### **`capsule.rs` (Facade)**

```rust
// Declare submodules (no re-exports for cleaner API surface)
pub mod api_types;
pub mod commands;
pub mod domain;
pub mod query;
pub mod util;
```

**Rationale**: No re-exports to maintain explicit module boundaries and prevent API surface pollution.

### **`domain.rs`**

- **Pure Rust** (no `ic_cdk` dependencies)
- `Capsule` struct and all its methods
- **Access control system**: `Perm` bitflags, `AccessEntry`, `PublicPolicy`
- **Universal types**: `ResourceType`, `GrantSource`, `ResourceRole`, `PublicMode`
- **Connection management**: `Connection`, `ConnectionGroup`, `PersonRef`
- **Role templates**: `RoleTemplate` and `get_default_role_templates()`
- Business logic and permission evaluation
- Domain validation

### **`commands.rs`**

- **Write operations** (state-changing)
- `capsules_create()` - Create new capsules
- `capsules_update()` - Update capsule data
- `capsules_delete()` - Delete capsules
- `resources_bind_neon()` - Bind to Neon database
- `update_user_settings()` - Update user preferences
- Call repository for persistence
- Handle business logic for mutations
- Return domain results

### **`query.rs`**

- **Read operations** (pure selectors)
- `capsules_read()` - Get full capsule data
- `capsules_read_basic()` - Get capsule info
- `capsule_read_self()` - Get caller's self-capsule
- `capsules_list()` - List all accessible capsules
- `get_user_settings()` - Get user preferences
- Mostly pure functions
- Call repository for data retrieval
- Return domain data
- No side effects

### **`api_types.rs`**

- **Request/Response DTOs**
- `CapsuleInfo` - Capsule information for user queries
- `CapsuleHeader` - Lightweight capsule data for listings
- `CapsuleUpdateData` - Update request structure
- `UserSettingsUpdateData` - User settings update structure
- `UserSettingsResponse` - User settings response
- Candid-compatible types
- API-specific data structures
- Separate from domain types for evolution

### **`util.rs`**

- **Helper functions** for capsule operations
- `calculate_capsule_size()` - Calculate serialized capsule size
- `find_self_capsule()` - Find caller's self-capsule
- `update_capsule_activity()` - Update activity timestamps
- `export_capsules_for_upgrade()` - Export for canister upgrades
- `import_capsules_from_upgrade()` - Import from canister upgrades
- Migration and upgrade support
- Size accounting utilities

## Do's and Don'ts

### **Do:**

- ✅ Keep `impl Capsule` and its logic in `domain.rs`
- ✅ Keep Candid types (`CapsuleHeader`, `CapsuleInfo`, request/response) in `api_types.rs`
- ✅ Keep domain code pure (no `ic_cdk` dependencies)
- ✅ Use CQRS pattern aligned with ICP `#[query]`/`#[update]`
- ✅ Implement idempotent updates with version counters
- ✅ Use stable memory boundaries in repository layer
- ✅ Separate access control types in `domain.rs`
- ✅ Use `util.rs` for helper functions and migration support
- ✅ Maintain explicit module boundaries (no re-exports)

### **Don't:**

- ❌ Create giant `types.rs` files
- ❌ Call `ic_cdk` from domain code—only in `commands`/`query`/`util`
- ❌ Mutate state in `#[query]` functions
- ❌ Store large `Vec` blobs in single values
- ❌ Mix domain logic with framework concerns
- ❌ Re-export everything from facade (maintain explicit boundaries)

## Benefits

### **Architectural Benefits:**

- **Clean separation** of concerns
- **Testable** domain logic (no canister context)
- **Reusable** business logic in migrations/off-chain tools
- **Maintainable** code structure

### **ICP-Specific Benefits:**

- **Performance**: Fast `#[query]` operations with projections
- **Certification**: Read-side certification support
- **Upgrades**: Safe state migration patterns
- **Consensus**: Proper `#[update]`/`#[query]` separation

### **Development Benefits:**

- **Easy navigation** and code organization
- **Clear module boundaries**
- **Reduced coupling** between layers
- **Framework independence** for core logic

## Implementation Strategy

**✅ COMPLETED:**

1. **Start with facade** (`capsule.rs`) - ✅ Declares submodules
2. **Extract domain** (`domain.rs`) - ✅ Pure business logic + access control
3. **Implement commands** (`commands.rs`) - ✅ Write operations (5 functions)
4. **Implement queries** (`query.rs`) - ✅ Read operations (6 functions)
5. **Add API types** (`api_types.rs`) - ✅ DTOs (6 types)
6. **Add utilities** (`util.rs`) - ✅ Helper functions + migration support

**📋 NEXT STEPS (From Access Refactoring Plan):** 7. **Implement AccessControlled trait** in `domain.rs` 8. **Add access fields** to Memory/Gallery structs 9. **Optional magic link index** (if needed) 10. **Testing & validation** of access control system

## Related Documents

- [Capsule Module Refactoring Issue](../issues/open/capsule-module-refactoring.md)
- [Gallery Type Refactor Implementation](../issues/open/name-titile/gallery-type-refactor-implementation.md)

## References

- [The Rust Book - Modules](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [ICP Developer Documentation](https://internetcomputer.org/docs/current/developer-docs/)
- [CQRS Pattern](https://martinfowler.com/bliki/CQRS.html)
