# Issue: Clarify Services vs Lib Folder Organization

## Problem Statement

The current codebase has unclear separation between the `src/services/` and `src/lib/` folders, leading to confusion about where to place new functionality and making it difficult for developers to understand the intended architecture.

## Tech Lead Approval ✅

**Status**: APPROVED with guardrails to prevent regression
**Decision**: Implement the proposed separation with strict boundaries and enforcement rules

## Current State Analysis

### Services Folder (`src/services/`)

**Purpose**: Business logic and API integration services
**Contents**:

- `gallery.ts` - Gallery management service (CRUD operations, API calls)
- `memories.ts` - Memory management service (fetch, delete, process dashboard items)
- `upload.ts` - Main upload service (blob-first approach, multiple storage backends)
- `icp-upload.ts` - ICP-specific upload service
- `icp-gallery.ts` - ICP gallery service
- `upload/` subfolder with specialized upload utilities

**Characteristics**:

- Contains business logic and API integration
- Handles external service communication
- Implements domain-specific operations
- Often contains mock data and development utilities
- Includes analytics tracking and event handling

### Lib Folder (`src/lib/`)

**Purpose**: Utility functions, configurations, and infrastructure code
**Contents**:

- `utils.ts` - General utility functions (cn, shortenTitle)
- `auth-utils.ts` - Authentication utilities and session management
- `error-handling.ts` - Centralized error handling and normalization
- `blob.ts` - Blob storage utilities
- `s3-service.ts` - S3-specific utilities
- `s3-utils.ts` - S3 helper functions
- `file-picker.ts` - File picker utilities
- `ii-*.ts` - Internet Identity related utilities
- `storage/` - Storage provider abstractions and configurations
- `ai/` - AI-related utilities

**Characteristics**:

- Contains reusable utility functions
- Infrastructure and configuration code
- Provider abstractions and interfaces
- Low-level helper functions
- Framework-agnostic utilities

## Issues with Current Organization

### 1. **Overlapping Responsibilities**

- Both folders contain upload-related functionality
- Storage management is split between `services/upload.ts` and `lib/storage/`
- File picker utilities exist in both `services/upload/file-picker.ts` and `lib/file-picker.ts`

### 2. **Unclear Boundaries**

- `services/memories.ts` contains utility functions like `processDashboardItems`
- `services/gallery.ts` has mock data and development utilities
- Some services contain both business logic and utility functions

### 3. **Inconsistent Patterns**

- Some services are classes (`ICPUploadService`), others are object exports
- Mixed approaches to error handling and API integration
- Inconsistent naming conventions

## Approved Solution

### Clear Separation of Concerns with Enforceable Boundaries

#### Services Folder (`src/services/`) - Application Layer

**Purpose**: Business orchestration + external I/O
**Should contain**:

- **Business logic & orchestration**: flows that combine multiple providers (presign → upload → commit)
- **Policy checks**: quota, legal hold, retries, metrics emission
- **External API integration**: calling 3rd-party APIs using clients from `lib`
- **Domain services**: `memory.service`, `upload.service`, `gallery.service`
- **Side effects allowed**: network, queue, email, analytics, logging

> **Rule of thumb**: If it encodes a **product rule** or coordinates multiple effects, it's a service.

#### Lib Folder (`src/lib/`) - Foundations

**Purpose**: Reusable primitives + infra clients + pure helpers
**Should contain**:

- **Pure utilities**: formatters, validators, parsing, feature flags, type guards
- **Infra clients + providers** (no product rules): S3/Vercel Blob clients, fetch wrappers, DB client, auth client
- **Shared types/schemas** used across layers (in `src/lib/core/`)
- **Zero product policy**: no SKUs, quotas, pricing, business workflows
- **No network orchestration**—a lib client can make one call, but not coordinate a multi-step flow
- **No React** (except minimal adapters under `lib/react/*` if needed)

> **Rule of thumb**: If you could open-source it without context, it belongs in `lib`.

### Enforceable Boundaries

#### Import Rules (Strict)

- **`lib` can import from `lib` only**
- **`services` can import from `lib` and other `services` only via interfaces**
- **No circular dependencies allowed**

#### Testing Rules

- **`lib`**: Pure unit tests (no network), 100% deterministic
- **`services`**: Integration tests with provider mocks; E2E for critical flows

#### Side Effects Rules

- **`lib`**: Limited to "single responsibility" clients (e.g., `s3Client.putObject`), no orchestration
- **`services`**: Owns retries, backoff, idempotency, metrics

### Approved Folder Structure

```
src/
├── lib/
│   ├── core/                    # shared domain types/schemas only (no logic)
│   │   ├── types.ts
│   │   └── schemas.ts
│   ├── storage/
│   │   ├── s3.client.ts
│   │   ├── vercel-blob.client.ts
│   │   └── types.ts
│   ├── auth/
│   │   ├── session.ts
│   │   └── permissions.ts
│   ├── utils/
│   │   ├── cn.ts
│   │   ├── formatters.ts
│   │   └── file-picker.ts
│   ├── errors/
│   │   ├── codes.ts
│   │   └── AppError.ts
│   └── react/                   # minimal React adapters if needed
│       └── adapters.ts
├── services/
│   ├── upload.service.ts        # orchestrates presign->PUT->commit
│   ├── memory.service.ts        # state machine, commit rules
│   ├── gallery.service.ts
│   ├── analytics.service.ts
│   └── icp.service.ts
```

### Naming Conventions (Enforced)

- **Services**: `*.service.ts` (export interface + default implementation)
- **Lib clients**: `*.client.ts` or `*.provider.ts`
- **Helpers**: `lib/utils/*.ts` with descriptive filenames
- **Errors**: Centralized in `lib/errors/*.ts` (`AppError`, typed error codes)

## Migration Plan (Pragmatic)

### Phase 1 – Codify Rules (1 day)

- [ ] Add ESLint boundaries & path aliases (`@lib/*`, `@services/*`)
- [ ] Add `lib/core` for shared types; move cross-cutting types there
- [ ] Create `lib/errors/AppError` + standard codes
- [ ] Set up `eslint-plugin-boundaries` or `import/no-restricted-paths` to forbid `lib` → `services`

### Phase 2 – High-leverage Moves (1–2 days)

- [ ] Move **file-picker** helpers to `lib/utils`
- [ ] Split **S3/Vercel** code into `lib/storage/*.client.ts`
- [ ] Make `upload.service.ts` orchestrate via those clients
- [ ] Extract `processDashboardItems` → `lib/utils/formatters.ts` (if pure) or keep in `services/gallery.service.ts` (if business logic)

### Phase 3 – Sweep & Standardize (2–3 days)

- [ ] Rename to `*.service.ts` / `*.client.ts` conventions
- [ ] Move pure helpers out of `services/*` into `lib/*`
- [ ] Kill duplicates (file-picker duplication)
- [ ] Move mock data to `tests/fixtures` or `__mocks__` (avoid shipping in services runtime)

### Phase 4 – Locks & Docs

- [ ] Turn on ESLint rules as **errors**
- [ ] Add ADR ("Lib vs Services") in `/docs/adr-00x.md`
- [ ] Update team docs with examples
- [ ] Ensure API routes call **services** only; never call `lib` directly (except trivial pass-through reads)

## Benefits of This Organization

### 1. **Clear Mental Model**

- Developers know exactly where to place new code
- Easy to find existing functionality
- Consistent patterns across the codebase

### 2. **Better Maintainability**

- Services focus on business logic
- Utilities are reusable and testable
- Clear separation of concerns

### 3. **Improved Testing**

- Services can be mocked easily
- Utilities can be unit tested in isolation
- Better test organization

### 4. **Enhanced Reusability**

- Utilities can be shared across services
- Services can be composed together
- Better code organization

## Tech Lead Answers ✅

1. **Do you agree with this proposed separation?**

   - **Yes**. Add the **"no lib → services"** rule, and a `lib/core` for shared types.

2. **Are there any specific patterns or conventions you'd like to enforce?**

   - **`*.service.ts` with interface + default impl**
   - **Errors via `AppError`**
   - **API routes call services only**
   - **No business rules in `lib`**

3. **Should we implement this migration gradually or all at once?**

   - **Gradual, by feature area**. Start with upload stack (highest confusion), then memories/gallery.

4. **Are there any existing files that should be exceptions to these rules?**

   - **Very small cross-layer constants** (e.g., MIME allow-list) can live in `lib/core` because both layers need them.
   - **Avoid any other exceptions**; they accumulate debt fast.

5. **What naming conventions should we use for services vs utilities?**
   - **Services**: `noun.service.ts`, methods are verbs (`create`, `commit`, `delete`)
   - **Lib clients**: `thing.client.ts` or `thing.provider.ts`
   - **Utils**: `lib/utils/*.ts` with descriptive filenames

## Guardrails That Make It Stick

### 1. Architecture Lints

- **`eslint-plugin-boundaries`**: Disallow `lib` importing from `services`
- **`import/no-cycle`**: Catch circular deps early

### 2. Barrels Carefully

- Barrel files (`index.ts`) are fine in `lib/utils` and `lib/storage`
- **Don't** create a giant `lib/index.ts` that re-exports everything (causes accidental deps and cycles)

### 3. Error Model

- One `AppError` in `lib/errors` with `code`, `httpStatus?`, `details?`
- Services throw typed errors; handlers map to HTTP

### 4. Interface-First Services

- Define `export interface UploadService { createPresigns(...): Promise<...>; commit(...): ... }`
- Default impl uses lib clients; tests can inject fakes

### 5. No Business Logic in API Routes

- Next.js route handlers should call **services**; never call `lib` directly (except trivial pass-through reads)

## Next Steps

1. ✅ **Get approval** from tech lead on the proposed organization
2. **Begin Phase 1** - Set up ESLint boundaries and path aliases
3. **Create `lib/core`** for shared types and schemas
4. **Implement `AppError`** in `lib/errors`
5. **Start with upload stack migration** (highest confusion area)
6. **Update team documentation** with the new conventions

## Current Pain Points Addressed

### Upload Overlap

- **Keep S3/Vercel clients** in `lib/storage/*`
- **Keep upload orchestration** (batching, retries, tombstones, metrics) in `services/upload.service.ts`
- **UI glue** (file input) in components/hooks, with tiny DOM helpers in `lib/utils/file-picker.ts`

### Memories Mixing Utilities

- **Extract `processDashboardItems`** → `lib/utils/formatters.ts` (if pure) or `services/gallery.service.ts` (if business logic)

### Mock Data

- **Dev/test fixtures** under `tests/fixtures` or `__mocks__`
- **Avoid shipping mocks** in `services` runtime

---

**Priority**: High
**Effort**: Medium (4-6 days total)
**Impact**: High (Developer Experience, Code Maintainability)
**Status**: ✅ Approved by Tech Lead
**Dependencies**: Team consensus on implementation timeline
