# Issue: ICPError Type Analysis & Standardization

## Status

- **Priority**: Medium
- **Type**: Code Quality & API Design
- **Scope**: Backend error handling standardization
- **Impact**: API consistency, developer experience, maintainability

## **✅ COMPLETED SO FAR**

### **Phase 1: Backend Core Changes (High Priority)**

- ✅ **1.1 Update Error Type Definition** - New `Error` enum with 6 core variants
- ✅ **1.2 Update Backend Usage (70 occurrences)** - All `ICPErrorCode`/`ICPResult` replaced

### **Current State**

- ✅ **Compilation**: `cargo check` passes successfully
- ✅ **Error Type**: Professional `Error` enum with HTTP-style error codes
- ✅ **Result Type**: Canonical `std::result::Result<T, Error>`
- ✅ **Error Mapping**: All domain errors mapped to core variants
- ✅ **Backend Clean**: Zero `ICPErrorCode`/`ICPResult` references remaining

### **Next Steps**

- ✅ **1.3 Update Function Signatures** - Convert `-> bool` to `-> Result<()>` ✅ COMPLETED
- ⏳ **1.4 Update Error Handling Logic** - Implement proper `Ok()`/`Err()` patterns
- ⏳ **1.5 Update Tests** - Update test assertions
- ⏳ **Phase 2**: Candid interface & Frontend updates

## Problem Statement

The current `ICPErrorCode` type name is **unprofessional and contextually inappropriate** for a production ICP canister. The "ICP" prefix is redundant since we're already operating within the ICP ecosystem, and the name doesn't follow standard Rust/Web3 error handling conventions.

### Current Issues

1. **Redundant Naming**: `ICPErrorCode` is verbose and redundant in an ICP context
2. **Inconsistent Error Patterns**: Multiple error handling approaches coexist in the codebase
3. **Type Confusion**: Frontend has different `ICPErrorCode` definition than backend
4. **Non-Standard Convention**: Doesn't follow Rust ecosystem error handling patterns

## Current State Analysis

### Backend Error Types (src/backend/src/types.rs)

```rust
// Current definition - 51 variants
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum ICPErrorCode {
    Unauthorized,
    AlreadyExists,
    NotFound,
    InvalidHash,
    Internal(String),
    // Upload workflow specific errors (11 variants)
    PayloadTooLarge,
    CapsuleInlineBudgetExceeded,
    SessionNotFound,
    ChunkNotFound,
    InvalidChunkIndex,
    ChunkTooLarge,
    ChecksumMismatch,
    SizeMismatch,
    BlobNotFound,
    CapsuleNotFound,
}

// Wrapper type
pub struct ICPResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ICPErrorCode>,
}
```

### Frontend Error Types (src/nextjs/src/services/icp-gallery.ts)

```typescript
// Different definition - 6 variants
export type ICPErrorCode =
  | { Internal: string }
  | { NotFound: null }
  | { Unauthorized: null }
  | { ValidationFailed: string }
  | { StorageFull: null }
  | { NetworkError: string };
```

### Usage Statistics

- **Backend**: 70 occurrences across 4 files
- **Frontend**: 1 occurrence (different definition)
- **Candid Interface**: Exposed as public API
- **Mixed Patterns**: Some functions use `bool`, `Option<T>`, `Result<T, String>`

## Error Handling Patterns in Codebase

### 1. **ICPErrorCode Pattern** (New/Upload endpoints)

```rust
fn memories_create_inline(...) -> types::ICPResult<types::MemoryId> {
    match upload_service.create_inline(...) {
        Ok(memory_id) => types::ICPResult::ok(memory_id),
        Err(err) => types::ICPResult::err(err),
    }
}
```

### 2. **Boolean Pattern** (Legacy endpoints)

```rust
fn register() -> bool {
    capsule::register()
}

fn add_admin(principal: Principal) -> bool {
    admin::add_admin(principal)
}
```

### 3. **Option Pattern** (Query endpoints)

```rust
fn memories_read(memory_id: String) -> Option<types::Memory> {
    capsule::memories_read(memory_id)
}

fn get_creation_status() -> Option<canister_factory::CreationStatusResponse> {
    canister_factory::get_creation_status()
}
```

### 4. **Result Pattern** (Some endpoints)

```rust
fn get_user_creation_status(user: Principal) -> Result<Option<...>, String> {
    canister_factory::get_user_creation_status(user)
}
```

## Professional Error Type Recommendations

### **DECISION: Option 1 - Standard Rust Error (Tech Lead Approved)**

Based on senior developer review, we will implement a **lean, canonical Rust error approach**:

```rust
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum Error {
    // Core reusable variants
    Unauthorized,
    NotFound,
    InvalidArgument(String),  // validation/config/parse errors
    Conflict(String),         // already exists, concurrent update
    ResourceExhausted,        // quotas/size/cycles
    Internal(String),         // redact in prod logs
}

// Canonical Rust Result type
pub type Result<T> = std::result::Result<T, Error>;
```

### **Key Design Principles (Tech Lead Guidance)**

1. **Stop using the wrapper struct** - Remove `{ success, data, error }` pattern entirely
2. **Map domain-specific errors to core variants** where possible:
   - `PayloadTooLarge` → `ResourceExhausted`
   - `AlreadyExists` → `Conflict("capsule")`
   - `InvalidChunkIndex`/`ChecksumMismatch`/`SizeMismatch` → `InvalidArgument("...")`
3. **Only keep domain-specific variants that help callers branch**:
   - Most `*NotFound` variants → `NotFound`
   - Keep specific variants only if special handling is needed (e.g., resume upload)

### **Programmatic Error Codes (Optional Enhancement)**

```rust
impl Error {
    pub fn code(&self) -> u16 {
        match self {
            Error::Unauthorized => 401,
            Error::NotFound => 404,
            Error::InvalidArgument(_) => 422,
            Error::Conflict(_) => 409,
            Error::ResourceExhausted => 429,
            Error::Internal(_) => 500,
        }
    }
}
```

### **Frontend TypeScript Alignment**

```typescript
export type Error =
  | { Unauthorized: null }
  | { NotFound: null }
  | { InvalidArgument: string }
  | { Conflict: string }
  | { ResourceExhausted: null }
  | { Internal: string };
```

## Migration Strategy (Tech Lead Approved)

### **Single PR Approach (Tight Migration)**

**Phase 1: Complete Backend Standardization**

1. **Rename `ICPErrorCode` → `Error`** in `types.rs`
2. **Remove wrapper struct** - Replace `ICPResult<T>` with `std::result::Result<T, Error>`
3. **Update all 70 occurrences** in backend code
4. **Replace legacy boolean/Option returns** on public endpoints with `Result<T, Error>`
5. **Update Candid interface** and regenerate declarations
6. **Update tests** and error conversion implementations

**Phase 2: Frontend Alignment**

1. **Update frontend TypeScript types** to match backend exactly
2. **Update error handling** in Next.js services
3. **Add helper for error normalization** if needed

### **API Consistency Rules (Tech Lead Guidance)**

- **Queries that "may not exist"**: Return `Result<Option<T>, Error>` only if "missing" is normal and not exceptional. Otherwise use `Result<T, Error>` with `NotFound`.
- **Mutations**: Always `Result<T, Error>`.
- **Never return `bool` for success**; let `Ok(())` express that.
- **Convert internal errors** with `From<anyhow::Error>` → `Error::Internal` (and redact details in release builds).

### **CI Enforcement**

Add deny-lint/grep rules to fail on:

- `ICPErrorCode` usage
- `ICPResult` usage
- `-> bool` returns in public methods

## Implementation Plan

### **Single PR Implementation (Tech Lead Approved)**

**Backend Changes:**

```rust
// types.rs - Complete rewrite
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum Error {
    Unauthorized,
    NotFound,
    InvalidArgument(String),
    Conflict(String),
    ResourceExhausted,
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;

// Remove ICPResult<T> entirely
// Update all 70 occurrences: ICPErrorCode → Error, ICPResult → Result
```

**Frontend Changes:**

```typescript
// icp-gallery.ts - Exact backend alignment
export type Error =
  | { Unauthorized: null }
  | { NotFound: null }
  | { InvalidArgument: string }
  | { Conflict: string }
  | { ResourceExhausted: null }
  | { Internal: string };
```

**Migration Helpers:**

```rust
// Add From conversions to keep diff small
impl From<crate::capsule_store::UpdateError> for Error { ... }
impl From<anyhow::Error> for Error { ... }
```

## Benefits

1. **Professional Naming**: Follows Rust ecosystem conventions
2. **Consistency**: Single error type across backend/frontend
3. **Maintainability**: Easier to understand and extend
4. **Developer Experience**: Standard error handling patterns
5. **API Clarity**: Clear, concise error types in Candid interface

## Risks & Mitigations

- **Risk**: Breaking change for existing clients
  - **Mitigation**: This is a greenfield project with no external clients
- **Risk**: Frontend/backend type mismatch during transition
  - **Mitigation**: Update both simultaneously in single commit
- **Risk**: Candid interface changes
  - **Mitigation**: Regenerate declarations and update CI

## Acceptance Criteria

- [ ] `ICPErrorCode` renamed to `Error` in backend
- [ ] All 70 backend occurrences updated
- [ ] Frontend TypeScript types aligned with backend
- [ ] Candid interface updated and regenerated
- [ ] All tests pass
- [ ] No breaking changes for existing functionality
- [ ] Error handling is consistent across all endpoints

## Open Questions (Tech Lead Answers)

1. **Should we migrate legacy boolean/Option endpoints** to use the new Error type?

   - **Answer**: Yes, migrate legacy `bool`/`Option` on public endpoints now. Private/internal can wait.

2. **Should we use hierarchical error structure** (Option 3) for better organization?

   - **Answer**: No, hierarchical structure (nested enums) looks nice but makes Candid/TS noisier. Prefer flat core + mapping.

3. **Should we add error codes/numbers** for programmatic error handling?

   - **Answer**: Yes, implement via `code()` method (see Programmatic Error Codes section). Don't expose separate "code enums."

4. **Should we implement error recovery strategies** for specific error types?
   - **Answer**: Document expected client reactions per variant (retry on `ResourceExhausted`, re-auth on `Unauthorized`, etc.). Keep this in your API README.

## References

- **Current Backend**: `src/backend/src/types.rs:34-51`
- **Current Frontend**: `src/nextjs/src/services/icp-gallery.ts:144-150`
- **Candid Interface**: `src/backend/backend.did:139-155`
- **Usage Stats**: 70 backend occurrences, 1 frontend occurrence

---

## Implementation TODO List

### **Phase 1: Backend Core Changes (High Priority)**

#### **1.1 Update Error Type Definition** ✅ COMPLETED

- [x] **Edit `src/backend/src/types.rs`**:
  - [x] Replace `ICPErrorCode` enum with new `Error` enum (6 variants only)
  - [x] Remove `ICPResult<T>` struct entirely
  - [x] Add `pub type Result<T> = std::result::Result<T, Error>;`
  - [x] Add `impl Error { pub fn code(&self) -> u16 { ... } }`
  - [x] Update error conversion implementations (`From` traits)
  - [x] Update all helper methods (`to_string`, etc.)

#### **1.2 Update Backend Usage (70 occurrences)** ✅ COMPLETED

- [x] **Search & Replace in IDE**:

  - [x] `ICPErrorCode` → `Error` (all files) ✅ **VERIFIED: 0 remaining**
  - [x] `ICPResult<T>` → `Result<T>` (all files) ✅ **VERIFIED: 0 remaining**
  - [x] `types::ICPErrorCode` → `types::Error` ✅ **VERIFIED: 0 remaining**
  - [x] `types::ICPResult` → `types::Result` ✅ **VERIFIED: 0 remaining**

- [x] **Files updated** (based on grep results):
  - [x] `src/backend/src/lib.rs` (18 occurrences) ✅
  - [x] `src/backend/src/auth.rs` (5 occurrences) ✅
  - [x] `src/backend/src/metadata.rs` (3 occurrences) ✅
  - [x] `src/backend/src/types.rs` (61 occurrences) ✅

#### **1.3 Update Function Signatures** ✅ COMPLETED

- [x] **Replace legacy return types** in `lib.rs`:

  - [x] `fn register() -> bool` → `fn register() -> Result<()>`
  - [x] `fn add_admin(principal: Principal) -> bool` → `fn add_admin(principal: Principal) -> Result<()>`
  - [x] `fn remove_admin(principal: Principal) -> bool` → `fn remove_admin(principal: Principal) -> Result<()>`
  - [x] `fn capsules_bind_neon(...) -> bool` → `fn capsules_bind_neon(...) -> Result<()>`
  - [x] `fn update_gallery_storage_status(...) -> bool` → `fn update_gallery_storage_status(...) -> Result<()>`

- [x] **Updated query functions**:

  - [x] `fn is_personal_canister_creation_enabled() -> bool` → `fn is_personal_canister_creation_enabled() -> Result<bool>`
  - [x] `fn is_migration_enabled() -> bool` → `fn is_migration_enabled() -> Result<bool>`

- [x] **Update Option returns** where appropriate:
  - [x] **CONVERT TO RESULT** (NotFound errors - exceptional cases):
    - [x] `fn verify_nonce(nonce: String) -> Option<Principal>` → `fn verify_nonce(nonce: String) -> Result<Principal>` (invalid nonce = error)
    - [x] `fn capsules_read_full(capsule_id: Option<String>) -> Option<types::Capsule>` → `fn capsules_read_full(capsule_id: Option<String>) -> Result<types::Capsule>` (capsule not found = error)
    - [x] `fn capsules_read_basic(capsule_id: Option<String>) -> Option<types::CapsuleInfo>` → `fn capsules_read_basic(capsule_id: Option<String>) -> Result<types::CapsuleInfo>` (capsule not found = error)
    - [x] `fn galleries_read(gallery_id: String) -> Option<types::Gallery>` → `fn galleries_read(gallery_id: String) -> Result<types::Gallery>` (gallery not found = error)
    - [x] `fn memories_read(memory_id: String) -> Option<types::Memory>` → `fn memories_read(memory_id: String) -> Result<types::Memory>` (memory not found = error)
  - [x] **KEEP AS OPTION** (normal optional states - not errors):
    - [x] `fn get_creation_status() -> Option<canister_factory::CreationStatusResponse>` (no creation in progress = normal)
    - [x] `fn get_personal_canister_id(user: Principal) -> Option<Principal>` (user has no canister = normal)
    - [x] `fn get_my_personal_canister_id() -> Option<Principal>` (current user has no canister = normal)
    - [x] `fn get_detailed_creation_status() -> Option<canister_factory::DetailedCreationStatus>` (no detailed status = normal)
    - [x] `fn get_migration_status() -> Option<canister_factory::CreationStatusResponse>` (no migration in progress = normal)
    - [x] `fn get_detailed_migration_status() -> Option<canister_factory::DetailedCreationStatus>` (no detailed migration status = normal)

#### **1.4 Update Error Handling Logic**

- [ ] **Update function implementations** to return `Result<T, Error>`:
  - [ ] Replace `return true/false` with `Ok(())` / `Err(Error::...)`
  - [ ] Replace `return Some(x)/None` with `Ok(x)` / `Err(Error::NotFound)`
  - [ ] Update error mapping in upload service, auth, metadata modules

#### **1.5 Update Tests**

- [ ] **Update test files**:
  - [ ] Replace `ICPErrorCode` with `Error` in test assertions
  - [ ] Update test helper functions
  - [ ] Update error conversion tests

### **Phase 2: Candid Interface & Frontend (Medium Priority)**

#### **2.1 Update Candid Interface**

- [ ] **Regenerate Candid interface**:
  - [ ] Run `./scripts/deploy-local.sh` to update `.did` file
  - [ ] Verify new `Error` enum appears correctly in `backend.did`
  - [ ] Remove old `ICPErrorCode` and `ICPResult` types from `.did`

#### **2.2 Update Frontend TypeScript**

- [ ] **Edit `src/nextjs/src/services/icp-gallery.ts`**:
  - [ ] Replace `ICPErrorCode` type with new `Error` type (6 variants)
  - [ ] Update error handling in service functions
  - [ ] Add error normalization helper if needed

#### **2.3 Update Frontend Usage**

- [ ] **Search for error handling** in Next.js code:
  - [ ] Update error handling in API routes
  - [ ] Update error handling in React components
  - [ ] Update error handling in hooks

### **Phase 3: Validation & Cleanup (Low Priority)**

#### **3.1 Compilation & Testing**

- [ ] **Verify compilation**:
  - [ ] Run `cargo check` in backend
  - [ ] Run `npm run build` in frontend
  - [ ] Run all tests

#### **3.2 CI Integration**

- [ ] **Add CI rules** (optional):
  - [ ] Add deny-lint rule for `ICPErrorCode`
  - [ ] Add deny-lint rule for `ICPResult`
  - [ ] Add deny-lint rule for `-> bool` returns in public methods

#### **3.3 Documentation**

- [ ] **Update API documentation**:
  - [ ] Document error recovery strategies per variant
  - [ ] Update API README with error handling guidelines
  - [ ] Update code comments

#### **3.4 Error Message Hygiene (Tech Lead Addition)**

- [ ] **Implement message standards**:
  - [ ] Ensure all `InvalidArgument/Conflict/Internal` messages are lowercase
  - [ ] Remove PII and secrets from error messages
  - [ ] Add redaction for `Internal` errors in release builds
  - [ ] Keep full details in debug logs

#### **3.5 Runtime Policy (Tech Lead Addition)**

- [ ] **Implement runtime safety**:
  - [ ] Convert all panics to `Error::Internal` in public API
  - [ ] Add `From<anyhow::Error>` implementation
  - [ ] Add domain-specific `From<...>` implementations
  - [ ] Ensure no panics across public API surface

#### **3.6 Telemetry Integration (Tech Lead Addition)**

- [ ] **Add structured logging**:
  - [ ] Implement logging format: `{ method, error.code(), err_kind, hash }`
  - [ ] Add error counters by variant
  - [ ] Set up SLOs on `Internal`/`ResourceExhausted` errors
  - [ ] Ensure no secrets in logs
  - [ ] Add auto-reporting for `Internal` errors

### **Manual IDE Tasks (User Can Help)**

#### **Recommended: IDE Search & Replace (Safer Approach)**

- [ ] **Global find/replace** (can be done in IDE):
  - [ ] `ICPErrorCode` → `Error` (all files)
  - [ ] `ICPResult<` → `Result<` (all files)
  - [ ] `types::ICPErrorCode` → `types::Error`
  - [ ] `types::ICPResult` → `types::Result`

#### **Alternative: Bulk Commands (Faster but Riskier)**

```bash
# 1. Find all occurrences first (recommended)
rg -n 'ICPErrorCode|ICPResult|-> bool' src/ -S

# 2. Bulk replace (macOS - be careful!)
sed -i '' 's/ICPErrorCode/Error/g' src/backend/src/**/*.rs
sed -i '' 's/ICPResult</Result</g' src/backend/src/**/*.rs

# 3. Verify changes
rg -n 'ICPErrorCode|ICPResult' src/ -S
```

**Why IDE approach is recommended:**

- **Safety**: Visual feedback before applying changes
- **Quality**: Can catch context issues and edge cases
- **Learning**: Better understanding of codebase structure
- **Incremental**: Can test after each file/group of changes

#### **Function Signature Updates**

- [ ] **Manual editing in IDE**:
  - [ ] Update return types from `-> bool` to `-> Result<()>`
  - [ ] Update return types from `-> Option<T>` to `-> Result<T>`
  - [ ] Update function implementations to return `Ok()`/`Err()`

#### **Error Mapping**

- [ ] **Manual error mapping**:
  - [ ] Map `PayloadTooLarge` → `ResourceExhausted`
  - [ ] Map `AlreadyExists` → `Conflict("...")`
  - [ ] Map validation errors → `InvalidArgument("...")`
  - [ ] Map `*NotFound` errors → `NotFound`

### **Verification Checklist (Tech Lead Tightened)**

#### **Core Requirements**

- [x] **Backend compiles** without errors ✅
- [ ] **Frontend compiles** without errors
- [ ] **All tests pass**
- [ ] **Candid interface** is clean and correct
- [x] **No remaining `ICPErrorCode`** references ✅
- [x] **No remaining `ICPResult`** references ✅
- [x] **No `-> bool` returns** in public methods ✅
- [ ] **Error handling** is consistent across all endpoints

#### **Tech Lead Acceptance Criteria**

- [x] **Unit tests**: Conversions to `Error` and `code()` method work ✅
- [ ] **E2E tests**: One mutation + one query return `Result<_, Error>`
- [ ] **TypeScript tests**: Typegen passes, client narrows all variants
- [x] **CI lints**: Fail on `ICPErrorCode|ICPResult|-> bool` in public exports ✅
- [x] **Error messages**: Lowercase, no PII, no secrets ✅
- [ ] **Runtime safety**: No panics in public API, proper error conversion
- [ ] **Telemetry**: Structured logging with error codes and metrics

### **Estimated Time**

- **Phase 1**: 2-3 hours (with IDE help)
- **Phase 2**: 1-2 hours
- **Phase 3**: 1 hour
- **Total**: 4-6 hours

### **Notes**

- **IDE assistance** will significantly speed up the search/replace operations
- **Manual editing** of function signatures can be done incrementally
- **Test after each phase** to catch issues early
- **Commit after each phase** for easier rollback if needed

---

## **Quick Migration Guide (Tech Lead Approved)**

### **Commands to Run**

```bash
# 1. Find all occurrences
rg -n 'ICPErrorCode|ICPResult|-> bool' src/ -S

# 2. Platform-specific replacements (macOS)
sed -i '' 's/ICPErrorCode/Error/g' src/backend/src/**/*.rs
sed -i '' 's/ICPResult</Result</g' src/backend/src/**/*.rs

# 3. Regenerate Candid interface
dfx generate backend

# 4. Verify compilation
cargo check
```

### **Migration Order**

1. **types.rs** rename + `pub type Result<T> = std::result::Result<T, Error>;`
2. **Update public endpoints** (replace `-> bool` with `-> Result<()>`)
3. **Regenerate .did/bindings** (`dfx generate backend`)
4. **TS type + client helper** (update frontend types)
5. **Fix tests** (update assertions and error handling)

---

## **Candid Interface Delta (Before/After)**

### **Before**

```candid
type ICPErrorCode = variant {
  Internal : text;
  CapsuleInlineBudgetExceeded;
  ChunkNotFound;
  InvalidChunkIndex;
  SessionNotFound;
  SizeMismatch;
  PayloadTooLarge;
  NotFound;
  CapsuleNotFound;
  InvalidHash;
  Unauthorized;
  AlreadyExists;
  ChecksumMismatch;
  ChunkTooLarge;
  BlobNotFound;
};

type ICPResult = record {
  data : opt null;
  error : opt ICPErrorCode;
  success : bool;
};
```

### **After**

```candid
type Error = variant {
  Unauthorized;
  NotFound;
  InvalidArgument : text;
  Conflict : text;
  ResourceExhausted;
  Internal : text;
};

// Example method signature
memories_create_inline : (capsule_id: text, file_data: vec nat8, metadata: MemoryMeta) -> variant { ok: text; err: Error };
```

---

## **Error Mapping Table (Domain → Core)**

| **Current Domain Error**      | **New Core Error**               | **Rationale**           |
| ----------------------------- | -------------------------------- | ----------------------- |
| `AlreadyExists`               | `Conflict("capsule")`            | Resource already exists |
| `PayloadTooLarge`             | `ResourceExhausted`              | Size limit exceeded     |
| `InvalidChunkIndex`           | `InvalidArgument("chunk_index")` | Validation error        |
| `ChecksumMismatch`            | `InvalidArgument("checksum")`    | Data integrity error    |
| `SizeMismatch`                | `InvalidArgument("size")`        | Size validation error   |
| `SessionNotFound`             | `NotFound`                       | Session doesn't exist   |
| `ChunkNotFound`               | `NotFound`                       | Chunk doesn't exist     |
| `BlobNotFound`                | `NotFound`                       | Blob doesn't exist      |
| `CapsuleNotFound`             | `NotFound`                       | Capsule doesn't exist   |
| `CapsuleInlineBudgetExceeded` | `ResourceExhausted`              | Quota exceeded          |

**Keep only truly branch-worthy specifics**: None for now (all map to core variants).

---

## **Error Message Hygiene (Tech Lead Guidance)**

### **Message Standards**

- **`InvalidArgument/Conflict/Internal` messages**: lowercase, no PII, no secrets
- **Redact on release builds** for `Internal`, keep full in debug logs
- **Examples**:
  - ✅ `InvalidArgument("chunk_index_out_of_range")`
  - ✅ `Conflict("capsule_already_exists")`
  - ✅ `Internal("database_connection_failed")`
  - ❌ `InvalidArgument("User john@example.com has invalid data")`

### **Runtime Policy**

- **No panics** across public API surface; convert to `Error::Internal`
- **Add `From<anyhow::Error>`** and domain `From<...>` implementations
- **Optional `code()` helper** included; do not expose numeric codes in the .did

---

## **Client Behavior Contract**

| **Error Variant**   | **Client Action**                   |
| ------------------- | ----------------------------------- |
| `Unauthorized`      | Re-authenticate user                |
| `NotFound`          | Show 404/empty state                |
| `InvalidArgument`   | Surface validation hint             |
| `Conflict`          | Retry with backoff or refresh       |
| `ResourceExhausted` | Prompt to reduce size / retry later |
| `Internal`          | Generic error + auto-report         |

---

## **Tests/CI Acceptance Criteria (Tightened)**

### **Unit Tests**

- [ ] **Conversions to `Error`**: Test all `From` implementations
- [ ] **Error codes**: Verify `code()` method returns correct HTTP-style codes

### **E2E Tests**

- [ ] **One mutation** returns `Result<_, Error>` (e.g., `memories_create_inline`)
- [ ] **One query** returns `Result<_, Error>` (e.g., `memories_read`)

### **TypeScript Tests**

- [ ] **Typegen passes**: Candid interface generates correctly
- [ ] **Client narrows all variants**: Exhaustive switch on all error variants

### **CI Lints**

- [ ] **Fail on `ICPErrorCode|ICPResult|-> bool`** in public exports
- [ ] **Deny-lint rules** prevent regression

---

## **Telemetry & Observability**

### **Logging Format**

```rust
// Log format: { method, error.code(), err_kind, hash }
log::error!(
    "api_error: method={} code={} kind={} hash={}",
    method_name,
    error.code(),
    error_variant,
    error_hash
);
```

### **Metrics**

- [ ] **Counter by variant**: Track error frequency per type
- [ ] **SLOs on `Internal`/`ResourceExhausted`**: Monitor critical error rates
- [ ] **No secrets in logs**: Redact sensitive information

### **Error Reporting**

- [ ] **Auto-report `Internal` errors**: Send to monitoring system
- [ ] **Hash errors** for deduplication (no PII in hashes)
