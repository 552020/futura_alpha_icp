# Backend Test Suite Overview

This document provides an overview of all test types in the Futura ICP backend and how to run them.

## 📁 Test Structure

### 🧪 **Integration Tests Directory** (`/tests/`)

Located at the workspace root for end-to-end testing.

#### `capsule_store_integration.rs`

**Purpose**: High-level integration tests for the capsule storage foundation.
**Coverage**: Tests both HashMap and Stable backends through the `Store` enum.
**Key Tests**:

- `test_store_enum_delegation()` - Verifies runtime polymorphism between backends
- `test_store_backend_identification()` - Confirms backend type identification
- `test_store_api_completeness()` - Complete API surface testing
- `test_index_updates_on_upsert_and_update()` - Index consistency during updates
- `test_pagination_cursor_semantics()` - Cursor-based pagination logic

**Run Command**:

```bash
cargo test --test capsule_store_integration -- --nocapture
```

---

### 🐚 **Bash Test Scripts** (`scripts/tests/backend/`)

End-to-end integration tests that test the deployed canister against real ICP network.

#### **Test Categories & Structure:**

```
scripts/tests/backend/
├── canister-capsule/          # Basic canister-capsule operations
├── general/                   # General capsule operations
├── shared-capsule/           # Advanced shared capsule functionality
│   ├── memories/             # Memory CRUD operations
│   └── galleries/            # Gallery CRUD operations
├── test_config.sh            # Shared configuration
└── test_utils.sh             # Shared utilities
```

#### **Main Test Suites:**

##### **📦 Canister-Capsule Tests** (`canister-capsule/`)

- **`test_canister_capsule.sh`** - Basic canister-capsule integration
- Tests fundamental canister-capsule communication

##### **🔧 General Capsule Tests** (`general/`)

- **`test_capsules_bind_neon.sh`** - Neon binding functionality
- **`test_capsules_create.sh`** - Capsule creation workflow
- **`test_capsules_list.sh`** - Capsule listing and pagination
- **`test_capsules_read.sh`** - Capsule read operations
- **`test_galleries_create.sh`** - Gallery creation
- **`test_galleries_delete.sh`** - Gallery deletion
- **`test_galleries_list.sh`** - Gallery listing
- **`test_galleries_update.sh`** - Gallery updates
- **`test_memories_ping.sh`** - Memory service health checks
- **`test_store_gallery_forever_with_memories.sh`** - Gallery persistence
- **`test_sync_gallery_memories.sh`** - Gallery-memory synchronization

##### **🎯 Shared Capsule Tests** (`shared-capsule/`)

- **`test_shared_capsule.sh`** - Shared capsule operations
- **`run_all_shared_tests.sh`** - Complete shared capsule test suite

###### **Memory Tests** (`shared-capsule/memories/`)

- **`test_authorization.sh`** - Memory access authorization
- **`test_chunked_upload.sh`** - Large file chunked uploads
- **`test_memories_advanced.sh`** - Advanced memory operations
- **`test_memories_create.sh`** - Memory creation
- **`test_memories_delete.sh`** - Memory deletion
- **`test_memories_list.sh`** - Memory listing
- **`test_memories_read.sh`** - Memory reading
- **`test_memories_update.sh`** - Memory updates
- **`test_memory_crud.sh`** - Complete memory CRUD workflow
- **`run_all_memory_tests.sh`** - Run all memory tests

###### **Gallery Tests** (`shared-capsule/galleries/`)

- **`test_gallery_crud.sh`** - Gallery CRUD operations
- **`test_gallery_upload.sh`** - Gallery file uploads
- **`test_uuid_mapping.sh`** - UUID mapping validation
- **`run_all_gallery_tests.sh`** - Run all gallery tests

#### **Setup Requirements:**

```bash
# 1. Start local ICP network
dfx start --background

# 2. Deploy canisters
dfx deploy

# 3. Configure test settings
# Edit scripts/tests/backend/test_config.sh
export BACKEND_CANISTER_ID="your-backend-canister-id"

# 4. Register test user (if needed)
dfx canister call backend register
```

#### **Running Bash Tests:**

```bash
cd scripts/tests/backend

# Run all shared capsule tests (recommended)
./shared-capsule/run_all_shared_tests.sh

# Run specific test suites
./shared-capsule/memories/run_all_memory_tests.sh
./shared-capsule/galleries/run_all_gallery_tests.sh

# Run individual tests
./general/test_capsules_create.sh
./general/test_capsules_read.sh
./shared-capsule/memories/test_memory_crud.sh

# Run basic canister tests
./canister-capsule/test_canister_capsule.sh
```

#### **Bash Test Features:**

- **Color-coded output** with pass/fail indicators
- **Detailed error reporting** with dfx command outputs
- **Test suite aggregation** with summary statistics
- **Individual test execution** for debugging
- **Configuration-driven** setup (no hardcoded values)

---

### 🔧 **Unit Tests** (Embedded in source files)

#### **Capsule Store Tests** (`src/capsule_store/`)

**Location**: `src/capsule_store/stable.rs`, `src/capsule_store/hash.rs`, `src/capsule_store/store.rs`

##### **Guardrail Tests** (Critical Regression Prevention)

These tests protect against the bugs we discovered during stable memory migration:

1. **`test_memory_manager_uniqueness()`** - Prevents multiple memory managers (memory overlap)
2. **`test_memory_overlap_canary()`** - Early detection of memory corruption
3. **`test_upsert_same_id_overwrites()`** - Ensures proper index cleanup on updates
4. **`test_empty_id_rejection()`** - Prevents empty IDs that corrupt subject indexes
5. **`test_isolation_with_fresh_memory()`** - Validates test isolation

##### **Core Functionality Tests**

- **`test_stable_store_basic_operations()`** - Basic CRUD operations
- **`test_capsule_storable()`** - Data serialization compatibility
- **`test_capsule_size_within_bound()`** - Memory size constraints

#### **Property-Based Tests** (`src/capsule_store/integration_tests.rs`)

**Purpose**: Automated bug hunting through randomized testing.
**Coverage**: Complex operation sequences that reveal edge cases.
**Key Tests**:

- `test_property_based_operations_hash()` - HashMap backend with 10-50 random operations
- `test_property_based_operations_stable()` - StableBTreeMap backend with 10-50 random operations

**Note**: These tests now correctly panic on empty IDs instead of showing false corruption.

#### **Canister Factory Tests** (`src/canister_factory/`)

**Location**: `src/canister_factory/integration_tests/`

1. **`cycles_tests.rs`** - Cycle management and payment logic
2. **`import_session_tests.rs`** - Import session handling
3. **`orchestration_tests.rs`** - Multi-canister orchestration
4. **`registry_tests.rs`** - Canister registry operations

#### **Upload Service Tests** (`src/upload/tests.rs`)

**Purpose**: File upload workflow validation.
**Coverage**:

- Service creation and initialization
- Constants validation (chunk sizes, budgets)
- ID generation (SessionId, BlobId)

#### **Other Module Tests**

- **`metadata.rs`** - Metadata management
- **`memory.rs`** - Memory region management
- **`auth.rs`** - Authentication logic
- **`capsule.rs`** - Core capsule operations

---

## 🚀 Running Tests

### **All Backend Tests**

```bash
cd src/backend
cargo test --lib
```

### **Specific Test Types**

#### **Guardrail Tests Only**

```bash
cd src/backend
cargo test capsule_store::stable::tests --lib -- --nocapture
```

#### **Property-Based Tests Only**

```bash
cd src/backend
cargo test test_property_based_operations_stable --lib -- --nocapture
```

#### **Integration Tests Only**

```bash
cargo test --test capsule_store_integration -- --nocapture
```

#### **Upload Tests Only**

```bash
cd src/backend
cargo test upload::tests --lib
```

#### **Canister Factory Tests**

```bash
cd src/backend
cargo test canister_factory::integration_tests --lib
```

### **Advanced Test Running**

#### **Run with Backtraces** (for debugging panics)

```bash
RUST_BACKTRACE=1 cargo test [test_name]
```

#### **Run Specific Test**

```bash
cargo test test_empty_id_rejection --lib
```

#### **Run Tests in Parallel** (default, faster)

```bash
cargo test --lib -- --test-threads=8
```

#### **Run Tests Sequentially** (for debugging)

```bash
cargo test --lib -- --test-threads=1
```

---

## 🎯 **Testing Pyramid Overview**

Our comprehensive test suite follows a testing pyramid approach with multiple levels:

### **1. 🧪 Unit Tests** (Fast, Isolated)

- **Purpose**: Test individual functions and components in isolation
- **Speed**: Milliseconds per test
- **Coverage**: Core business logic, data structures, algorithms
- **Examples**: Capsule CRUD, memory management, validation logic

### **2. 🔗 Integration Tests** (Medium Speed, Component Interaction)

- **Purpose**: Test component interactions and data flow
- **Speed**: Seconds per test
- **Coverage**: API contracts, backend workflows, error handling
- **Examples**: Store enum delegation, canister factory orchestration

### **3. 🌐 End-to-End Tests** (Slow, Real Environment)

- **Purpose**: Test complete user journeys against deployed canisters
- **Speed**: Minutes per test suite
- **Coverage**: Real ICP network, authentication, file uploads, UI workflows
- **Examples**: Memory uploads, gallery operations, capsule sharing

### **4. 🔒 Property-Based Tests** (Automated Bug Hunting)

- **Purpose**: Find edge cases through randomized testing
- **Speed**: Variable (10-60 seconds)
- **Coverage**: Complex operation sequences, data corruption scenarios
- **Examples**: Upsert sequences, index consistency, memory lifecycle

---

## 🎯 Test Categories by Purpose

### **🔒 Safety & Regression Tests**

- Guardrail tests in `stable.rs` - Prevent memory overlap, empty ID corruption
- Property-based tests - Catch edge cases automatically

### **⚡ Performance & Correctness**

- Property-based tests - Validate complex operation sequences
- Integration tests - End-to-end workflow validation

### **🧩 Component Tests**

- Unit tests in each module - Individual function validation
- Upload service tests - File handling workflows

### **🏗️ Architecture Tests**

- Store enum tests - Runtime polymorphism validation
- Canister factory tests - Multi-canister orchestration

### **🌐 End-to-End Tests**

- Bash integration tests - Real canister deployment testing
- Shared capsule workflows - Complete user journeys
- Memory upload cycles - File handling end-to-end
- Gallery operations - Complete gallery lifecycles

---

## 📊 Test Coverage Areas

| Area                   | Test Type                                 | Files                                                                             | Status      |
| ---------------------- | ----------------------------------------- | --------------------------------------------------------------------------------- | ----------- |
| **Capsule Storage**    | Guardrail + Property + Integration + Bash | `capsule_store/` + `scripts/tests/backend/`                                       | ✅ Complete |
| **Memory Management**  | Unit + Integration + Bash                 | `memory.rs` + `scripts/tests/backend/shared-capsule/memories/`                    | ✅ Complete |
| **Upload Service**     | Unit + Bash                               | `upload/tests.rs` + `scripts/tests/backend/shared-capsule/memories/`              | ✅ Complete |
| **Gallery Operations** | Bash                                      | `scripts/tests/backend/shared-capsule/galleries/`                                 | ✅ Complete |
| **Canister Factory**   | Integration                               | `canister_factory/integration_tests/`                                             | ✅ Complete |
| **Authentication**     | Unit + Bash                               | `auth.rs` + `scripts/tests/backend/shared-capsule/memories/test_authorization.sh` | ✅ Complete |
| **Metadata**           | Unit                                      | `metadata.rs`                                                                     | ✅ Basic    |

---

## 🔧 Test Infrastructure

### **Test Utilities**

- `test_utils.rs` - Shared test helpers and fixtures
- `create_test_capsule()` - Standard test capsule creation
- Memory isolation via `StableStore::new_test()`

### **Test Data**

- Deterministic test data for reproducible results
- Fresh memory per test (no cross-test contamination)
- Realistic capsule structures with owners, subjects, connections

### **Test Assertions**

- Count validation after operations
- Index consistency (subject, owner lookups)
- Memory isolation verification
- Empty ID rejection

---

## 🐛 Known Test Behaviors

### **Expected Panics** (Good!)

- `test_empty_id_rejection` - Should panic on empty CapsuleId
- Property tests - Should panic on invalid operations (catches bugs)

### **Performance Notes**

- Property tests may take 30-60 seconds with 10-50 operations
- Memory isolation adds slight overhead but prevents false positives
- Parallel test execution is recommended for speed

---

## 🎉 Recent Improvements

### **✅ Completed Fixes**

1. **Memory Isolation** - Fresh memory per test prevents accumulation
2. **Empty ID Protection** - Guards against subject index corruption
3. **Guardrail Tests** - 8 comprehensive regression tests
4. **Property Test Fixes** - Now catch real bugs instead of false positives

### **🚀 Current Status**

- All guardrail tests passing ✅
- Property tests working correctly ✅
- Memory isolation implemented ✅
- Test documentation complete ✅

---

## 📝 Contributing

### **Adding New Tests**

1. Place unit tests in the same file as the code they test
2. Add integration tests to `capsule_store_integration.rs`
3. Use `#[test]` attribute for unit tests
4. Use `#[should_panic]` for tests that should fail
5. Add guardrail tests for critical bug fixes

### **Test Naming Conventions**

- `test_[component]_[behavior]()` - Unit tests
- `test_property_based_[component]()` - Property tests
- `test_[bug_name]_rejection()` - Guardrail tests

### **Test Categories**

- **Unit**: Single function/component testing
- **Integration**: Multi-component workflows
- **Property**: Automated edge case discovery
- **Guardrail**: Regression prevention
