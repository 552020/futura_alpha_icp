# Capsule State Management Refactoring Request

## Problem Statement

The current `CapsuleInfo` component has messy and confusing state management that makes the code hard to maintain and understand. We need a clean, unified solution for managing capsule-related state.

## Current State Analysis

### Current State Variables (Messy)

```typescript
const [capsuleInfo, setCapsuleInfo] = useState<CapsuleInfo | null>(null); // Basic capsule info
const [capsuleReadResult, setCapsuleReadResult] = useState<Capsule | null>(null); // Full capsule data
const [capsuleIdInput, setCapsuleIdInput] = useState(""); // Input field
const [busy, setBusy] = useState(false); // Loading state
```

### Current Issues

1. **Confusing Naming**: `capsuleInfo` vs `capsuleReadResult` - unclear distinction
2. **Duplicate Data**: Both store capsule data but in different formats
3. **Manual Synchronization**: `setCapsuleState()` manually syncs between the two
4. **Inconsistent Updates**: Some functions update one, some update both
5. **Type Confusion**: `CapsuleInfo` vs `Capsule` - unclear when to use which
6. **State Fragmentation**: Related data scattered across multiple state variables

### Current Usage Patterns

```typescript
// Pattern 1: Only basic info (getCapsuleInfo)
setCapsuleInfo(result);

// Pattern 2: Full data (createCapsule, readCapsule)
setCapsuleState(result); // Updates both capsuleInfo AND capsuleReadResult

// Pattern 3: Clear state
setCapsuleInfo(null);
setCapsuleReadResult(null);
```

## Backend Type Definitions

### Core Capsule Types (from backend.did.d.ts)

```typescript
// Main capsule type - full capsule data
export interface Capsule {
  id: string;
  updated_at: bigint;
  controllers: Array<[PersonRef, ControllerState]>;
  subject: PersonRef;
  owners: Array<[PersonRef, OwnerState]>;
  inline_bytes_used: bigint;
  created_at: bigint;
  connection_groups: Array<[string, ConnectionGroup]>;
  connections: Array<[PersonRef, Connection]>;
  memories: Array<[string, Memory]>;
  bound_to_neon: boolean;
  galleries: Array<[string, Gallery]>;
}

// Basic capsule info - summary data
export interface CapsuleInfo {
  updated_at: bigint;
  gallery_count: bigint;
  subject: PersonRef;
  capsule_id: string;
  is_owner: boolean;
  created_at: bigint;
  bound_to_neon: boolean;
  memory_count: bigint;
  connection_count: bigint;
  is_self_capsule: boolean;
  is_controller: boolean;
}

// Person reference - can be Principal or Opaque string
export type PersonRef = { Opaque: string } | { Principal: Principal };

// Owner state - tracks ownership details
export interface OwnerState {
  last_activity_at: bigint;
  since: bigint;
}

// Controller state - tracks controller permissions
export interface ControllerState {
  granted_at: bigint;
  granted_by: PersonRef;
}

// Capsule update data - for partial updates
export interface CapsuleUpdateData {
  bound_to_neon: [] | [boolean];
}
```

### Key Differences Between Types

- **`Capsule`**: Full capsule data with all relationships (owners, controllers, memories, galleries)
- **`CapsuleInfo`**: Summary data for quick display and listing
- **`PersonRef`**: Union type for different person types (ICP Principal vs external string)
- **`OwnerState`/`ControllerState`**: Track relationship metadata and permissions

### Type Usage Patterns

```typescript
// ✅ CORRECT: Use Capsule for full operations
const createCapsule = (): Promise<Capsule> => { /* ... */ };
const readCapsule = (id: string): Promise<Capsule> => { /* ... */ };

// ✅ CORRECT: Use CapsuleInfo for listings and summaries
const getCapsuleInfo = (): Promise<CapsuleInfo> => { /* ... */ };
const listCapsules = (): Promise<CapsuleInfo[]> => { /* ... */ };

// ✅ CORRECT: Derive CapsuleInfo from Capsule
const capsuleInfo = deriveFromCapsule(capsule: Capsule): CapsuleInfo => {
  return {
    capsule_id: capsule.id,
    subject: capsule.subject,
    // ... compute other fields
  };
};
```

### Current Type Confusion Issues

1. **Mixed Usage**: Component uses both `Capsule` and `CapsuleInfo` inconsistently
2. **Manual Conversion**: `setCapsuleState()` manually converts between types
3. **Type Duplication**: Both store similar data but in different formats
4. **Unclear Semantics**: When to use which type is not clear

## Proposed Clean Solution

### Option A: Single Unified State (Recommended)

```typescript
interface CapsuleState {
  // Core capsule data (using actual backend types)
  capsule: Capsule | null;

  // UI state
  isLoading: boolean;
  error?: CapsuleError;

  // Input state (keep local unless multiple actions use it)
  capsuleIdInput: string;
}

type CapsuleError =
  | { kind: "connection"; message: string }
  | { kind: "authExpired"; message: string }
  | { kind: "unauthorized"; message: string }
  | { kind: "notFound"; message: string }
  | { kind: "invalid"; message: string }
  | { kind: "internal"; message: string };
```

### Option A+: Capsules Collection State (Extended)

```typescript
interface CapsulesState {
  // Current capsule (for detailed view)
  currentCapsule: Capsule | null;

  // All user's capsules (for listing/selection)
  capsules: CapsuleInfo[];

  // UI state
  loading: boolean;
  error: string | null;

  // Input state
  capsuleIdInput: string;

  // Selection state
  selectedCapsuleId: string | null;
}

const [capsuleState, setCapsuleState] = useState<CapsuleState>({
  capsule: null,
  loading: false,
  error: null,
  capsuleIdInput: "",
});

// Derived values (computed from Capsule to CapsuleInfo)
const capsuleInfo = useMemo((): CapsuleInfo | null => {
  if (!capsuleState.capsule) return null;

  const capsule = capsuleState.capsule;

  return {
    capsule_id: capsule.id,
    subject: capsule.subject,
    is_owner: true, // TODO: Calculate from capsule.owners
    is_controller: true, // TODO: Calculate from capsule.controllers
    updated_at: capsule.updated_at,
    created_at: capsule.created_at,
    bound_to_neon: capsule.bound_to_neon,
    gallery_count: BigInt(capsule.galleries.length),
    memory_count: BigInt(capsule.memories.length),
    connection_count: BigInt(capsule.connections.length),
    is_self_capsule: true, // TODO: Calculate based on subject vs caller
  };
}, [capsuleState.capsule]);
```

### Capsules Collection State Usage Patterns

```typescript
// ✅ Load all user's capsules
const loadCapsules = async () => {
  setCapsulesState((prev) => ({ ...prev, loading: true }));
  try {
    const capsules = await listCapsules(); // Returns CapsuleInfo[]
    setCapsulesState((prev) => ({
      ...prev,
      capsules,
      loading: false,
    }));
  } catch (error) {
    setCapsulesState((prev) => ({
      ...prev,
      error: error.message,
      loading: false,
    }));
  }
};

// ✅ Select a capsule for detailed view
const selectCapsule = async (capsuleId: string) => {
  setCapsulesState((prev) => ({
    ...prev,
    selectedCapsuleId: capsuleId,
    loading: true,
  }));
  try {
    const capsule = await readCapsule(capsuleId); // Returns Capsule
    setCapsulesState((prev) => ({
      ...prev,
      currentCapsule: capsule,
      loading: false,
    }));
  } catch (error) {
    setCapsulesState((prev) => ({
      ...prev,
      error: error.message,
      loading: false,
    }));
  }
};

// ✅ Create new capsule and add to collection
const createCapsule = async () => {
  try {
    const newCapsule = await createCapsule();
    setCapsulesState((prev) => ({
      ...prev,
      currentCapsule: newCapsule,
      capsules: [...prev.capsules, deriveCapsuleInfo(newCapsule)],
      selectedCapsuleId: newCapsule.id,
    }));
  } catch (error) {
    // Handle error
  }
};
```

### Benefits of Capsules Collection State

1. **Complete User Context**: Shows all user's capsules, not just one
2. **Efficient Navigation**: Switch between capsules without re-fetching
3. **Consistent State**: All capsule data in one place
4. **Better UX**: Users can see their capsule collection
5. **Optimistic Updates**: Add new capsules to list immediately

### Backend API Support for Capsules Collection

```typescript
// Backend methods that support capsules collection state
interface CapsuleAPI {
  // Get all user's capsules (returns CapsuleInfo[])
  listCapsules(): Promise<CapsuleInfo[]>;

  // Get specific capsule by ID (returns Capsule)
  readCapsule(id: string): Promise<Capsule>;

  // Create new capsule (returns Capsule)
  createCapsule(subject?: PersonRef): Promise<Capsule>;

  // Update capsule (returns updated Capsule)
  updateCapsule(id: string, updates: CapsuleUpdateData): Promise<Capsule>;

  // Delete capsule
  deleteCapsule(id: string): Promise<void>;
}
```

### Current Backend API Analysis

From the existing backend, we have:

- ✅ `capsules_create()` - Create capsule
- ✅ `capsules_read_basic()` - Get basic info (CapsuleInfo)
- ✅ `capsules_read()` - Get full capsule (Capsule)
- ❓ `capsules_list_by_owner()` - List user's capsules (needs verification)
- ❓ `capsules_list_by_subject()` - List capsules by subject (needs verification)

### When to Use Each Approach

- **Option A (Single State)**: Simple components that work with one capsule
- **Option A+ (Collection State)**: Complex components that need to manage multiple capsules
- **Option B (Reducer)**: Very complex state with many actions and transitions

### Option B: Reducer Pattern (For Complex State)

```typescript
type CapsuleAction =
  | { type: "SET_LOADING"; payload: boolean }
  | { type: "SET_CAPSULE"; payload: Capsule | null }
  | { type: "SET_ERROR"; payload: string | null }
  | { type: "SET_INPUT"; payload: string }
  | { type: "CLEAR_CAPSULE" };

const capsuleReducer = (state: CapsuleState, action: CapsuleAction): CapsuleState => {
  switch (action.type) {
    case "SET_LOADING":
      return { ...state, loading: action.payload };
    case "SET_CAPSULE":
      return { ...state, capsule: action.payload, error: null };
    case "SET_ERROR":
      return { ...state, error: action.payload, loading: false };
    case "SET_INPUT":
      return { ...state, capsuleIdInput: action.payload };
    case "CLEAR_CAPSULE":
      return { ...state, capsule: null, error: null };
    default:
      return state;
  }
};

const [capsuleState, dispatch] = useReducer(capsuleReducer, {
  capsule: null,
  loading: false,
  error: null,
  capsuleIdInput: "",
});
```

## Benefits of Clean Solution

### 1. **Single Source of Truth**

- One state object contains all capsule-related data
- No more manual synchronization between multiple states
- Clear data flow and updates

### 2. **Type Safety**

- Single `Capsule` type for all operations
- Derived `CapsuleInfo` computed from `Capsule`
- No type confusion or duplication

### 3. **Simplified Logic**

```typescript
// Before (messy)
if (result) {
  setCapsuleReadResult(result);
  const basicInfo = {
    /* complex object */
  };
  setCapsuleInfo(basicInfo);
}

// After (clean)
if (result) {
  setCapsuleState((prev) => ({ ...prev, capsule: result }));
}
```

### 4. **Better Performance**

- No unnecessary re-renders from multiple state updates
- Computed values only recalculate when capsule changes
- Single state update per operation

### 5. **Easier Testing**

- Single state object to mock/test
- Clear action patterns
- Predictable state transitions

## Implementation Plan

### Phase 1: Create New State Structure

1. Define `CapsuleState` interface
2. Create `useCapsuleState` custom hook
3. Implement derived values with `useMemo`

### Phase 2: Refactor Component

1. Replace multiple `useState` with single state
2. Update all state setters to use new pattern
3. Update UI to use derived values

### Phase 3: Clean Up

1. Remove old state variables
2. Remove `setCapsuleState` helper function
3. Update tests and documentation

## Questions for Tech Lead

1. **Which approach do you prefer?**

   - Option A: Single unified state (simpler)
   - Option B: Reducer pattern (more scalable)

2. **Should we create a custom hook?**

   - `useCapsuleState()` for reusability
   - Or keep it component-specific?

3. **How should we handle the derived `CapsuleInfo`?**

   - Computed in component with `useMemo`
   - Or computed in the custom hook?

4. **Should we apply this pattern to other components?**
   - Memory components
   - Gallery components
   - Other ICP-related components

## Current Component Structure

```typescript
// Current messy state
const [capsuleInfo, setCapsuleInfo] = useState<CapsuleInfo | null>(null);
const [capsuleReadResult, setCapsuleReadResult] = useState<Capsule | null>(null);
const [capsuleIdInput, setCapsuleIdInput] = useState("");
const [busy, setBusy] = useState(false);

// Current helper function
const setCapsuleState = (capsule: Capsule) => {
  setCapsuleReadResult(capsule);
  const basicInfo = {
    /* complex object */
  };
  setCapsuleInfo(basicInfo);
};
```

## Proposed Clean Structure

```typescript
// Clean unified state
const [capsuleState, setCapsuleState] = useState<CapsuleState>({
  capsule: null,
  loading: false,
  error: null,
  capsuleIdInput: "",
});

// Derived values
const capsuleInfo = useMemo(() => {
  // Compute from capsuleState.capsule
}, [capsuleState.capsule]);
```

## Conclusion

The current state management is confusing and error-prone. A clean, unified solution will:

- Reduce bugs and inconsistencies
- Improve code maintainability
- Make the component easier to understand
- Provide better performance
- Enable easier testing

**Recommendation**: Implement Option A (Single Unified State) as it's simpler and meets our current needs while being easy to extend later.

## Tech Lead Response & Refinements

### ✅ **Approved: Option A (Single Unified State)**

The tech lead has approved **Option A** with several important refinements for scalability and maintainability.

### 🔧 **Key Refinements from Tech Lead**

1. **Name state by intent**

   - `loading` → `isLoading`
   - `error` → typed union, not `string | null`
   - Keep `capsuleIdInput` local unless multiple actions use it

2. **Keep domain → view mapping in one place**

   - Create `capsuleToInfo(c: Capsule): CapsuleInfo` adapter
   - Avoid sprinkling conversion logic across UI

3. **Type errors properly**

   ```typescript
   type CapsuleError =
     | { kind: "connection"; message: string }
     | { kind: "authExpired"; message: string }
     | { kind: "unauthorized"; message: string }
     | { kind: "notFound"; message: string }
     | { kind: "invalid"; message: string }
     | { kind: "internal"; message: string };
   ```

4. **Guard against stale updates**

   - Track `requestId` for overlapping requests
   - Only apply latest result

5. **Don't store derived data**

   - Keep only `capsule` in state
   - Compute `capsuleInfo` via `useMemo`

6. **Tiny service layer**
   - Wrap actor calls in `capsuleService`
   - Returns `Capsule` and throws typed errors
   - Component stays dumb

### 📋 **Final Implementation Shape**

**Key Insight**: The backend already provides the perfect API design:

- **`CapsuleInfo`** - Lightweight summaries (counts, permissions)
- **`Capsule`** - Structure metadata (memory/gallery headers, no content)
- **`Memory`** - Full content (fetched on-demand)

**No adapter needed** - backend types are exactly what we need!

```typescript
// Frontend state structure
interface CapsulesState {
  // High-level capsule management
  capsules: CapsuleInfo[]; // Array of capsule summaries
  currentCapsule: Capsule | null; // Selected capsule structure

  // UI state
  isLoading: boolean;
  error: string | null;

  // Navigation state
  selectedCapsuleId: string | null;
}

// Usage patterns:
// 1. Load capsule summaries for dashboard
const capsules = await actor.capsules_list(); // Returns CapsuleInfo[]

// 2. Load capsule structure when selected
const capsule = await actor.capsules_read_full(capsuleId); // Returns Capsule

// 3. Load memory content when needed
const memory = await actor.memories_read(memoryId); // Returns Memory
```

```typescript
// Component implementation
const [capsulesState, setCapsulesState] = useState<CapsulesState>({
  capsules: [],
  currentCapsule: null,
  isLoading: false,
  error: null,
  selectedCapsuleId: null,
});

// Load all user's capsules (summaries only)
const loadCapsules = async () => {
  setCapsulesState((prev) => ({ ...prev, isLoading: true }));
  try {
    const capsules = await actor.capsules_list(); // Returns CapsuleInfo[]
    setCapsulesState((prev) => ({ ...prev, capsules, isLoading: false }));
  } catch (error) {
    setCapsulesState((prev) => ({ ...prev, error: error.message, isLoading: false }));
  }
};

// Select capsule for detailed view
const selectCapsule = async (capsuleId: string) => {
  setCapsulesState((prev) => ({ ...prev, selectedCapsuleId: capsuleId, isLoading: true }));
  try {
    const capsule = await actor.capsules_read_full(capsuleId); // Returns Capsule
    setCapsulesState((prev) => ({ ...prev, currentCapsule: capsule, isLoading: false }));
  } catch (error) {
    setCapsulesState((prev) => ({ ...prev, error: error.message, isLoading: false }));
  }
};
```

### ✅ **Tech Lead Answers**

- **Option A vs reducer?** Start with **Option A**; if actions multiply, drop in a reducer later.
- **Custom hook?** If ≥2 components need this, yes—extract `useCapsuleState`. Otherwise keep local.
- **Where to compute `CapsuleInfo`?** In the component via `useMemo`, calling the shared adapter.

## 🎯 **Final Solution: No Adapter Needed**

### **Why We Don't Need an Adapter**

1. **Backend API is Already Perfect**: The backend provides exactly the right types for different use cases:

   - `CapsuleInfo` → Lightweight summaries (counts, permissions)
   - `Capsule` → Structure metadata (memory/gallery headers, no content)
   - `Memory` → Full content (fetched on-demand)

2. **No Type Mismatch**: The backend types match our frontend needs perfectly:

   - `capsules_list()` returns `CapsuleInfo[]` for dashboards
   - `capsules_read_full()` returns `Capsule` for structure views
   - `memories_read()` returns `Memory` for content views

3. **Performance Optimized**: The backend already handles the heavy lifting:
   - Capsules never contain full memory content
   - Memory assets are stored separately
   - Content is fetched only when needed

### **Our Solution: Direct Backend Integration**

```typescript
// ✅ CORRECT: Use backend types directly
interface CapsulesState {
  capsules: CapsuleInfo[]; // From capsules_list()
  currentCapsule: Capsule | null; // From capsules_read_full()
  isLoading: boolean;
  error: string | null;
}

// ✅ CORRECT: Call backend APIs directly
const capsules = await actor.capsules_list(); // Returns CapsuleInfo[]
const capsule = await actor.capsules_read_full(id); // Returns Capsule
const memory = await actor.memories_read(memoryId); // Returns Memory
```

### **What We're NOT Doing (No Adapter)**

```typescript
// ❌ WRONG: Don't create unnecessary adapters
export function capsuleToInfo(c: Capsule): CapsuleInfo {
  /* ... */
}

// ❌ WRONG: Don't convert between types that are already perfect
const capsuleInfo = useMemo(() => (state.capsule ? capsuleToInfo(state.capsule) : null), [state.capsule]);
```

### **The Result**

- **Simpler code**: No adapter layer to maintain
- **Better performance**: No unnecessary conversions
- **Type safety**: Direct use of backend types
- **Maintainability**: Fewer moving parts, clearer data flow

### **Single-Source-of-Truth Example**

```typescript
// ✅ Single source of truth
const [state, setState] = useState<CapsuleState>({
  capsule: null,
  isLoading: false,
});

// ✅ Derive CapsuleInfo from Capsule when needed
const capsuleInfo = useMemo<CapsuleInfo | null>(() => {
  if (!state.capsule) return null;

  const c = state.capsule; // Single source of truth

  // Derive CapsuleInfo from Capsule
  return {
    capsule_id: c.id,
    subject: c.subject,
    is_owner: c.owners.length > 0,
    is_controller: c.controllers.length > 0,
    is_self_capsule: true, // Compare subject to caller principal
    updated_at: c.updated_at,
    created_at: c.created_at,
    bound_to_neon: c.bound_to_neon,
    gallery_count: BigInt(c.galleries.length),
    memory_count: BigInt(c.memories.length),
    connection_count: BigInt(c.connections.length),
  };
}, [state.capsule]); // Only recompute when capsule changes
```

**Benefits:**

- **Single Source**: Only `state.capsule` contains the data
- **No Duplication**: `capsuleInfo` is computed from `capsule`
- **Always in Sync**: If `capsule` changes, `capsuleInfo` automatically updates
- **Simpler State**: Only one state variable to manage

### **Service Call Pattern (Prevents Duplication)**

```typescript
// ✅ Service call pattern - prevents duplication
async function loadCapsule(id: string) {
  setState(s => ({ ...s, isLoading: true, error: undefined }));
  try {
    const capsule = await capsuleService.read(id); // returns Capsule or throws typed error
    setState(s => ({ ...s, capsule, isLoading: false }));
  } catch (e) {
    setState(s => ({ ...s, error: e as CapsuleError, isLoading: false }));
  }
}

// ✅ Create capsule pattern
async function createCapsule() {
  setState(s => ({ ...s, isLoading: true, error: undefined }));
  try {
    const capsule = await capsuleService.create(); // returns Capsule or throws typed error
    setState(s => ({ ...s, capsule, isLoading: false }));
  } catch (e) {
    setState(s => ({ ...s, error: e as CapsuleError, isLoading: false }));
  }

// ✅ Update capsule pattern
async function updateCapsule(id: string, updates: CapsuleUpdateData) {
  setState(s => ({ ...s, isLoading: true, error: undefined }));
  try {
    const capsule = await capsuleService.update(id, updates);
    setState(s => ({ ...s, capsule, isLoading: false }));
  } catch (e) {
    setState(s => ({ ...s, error: e as CapsuleError, isLoading: false }));
  }
}

// ✅ Delete capsule pattern
async function deleteCapsule(id: string) {
  setState(s => ({ ...s, isLoading: true, error: undefined }));
  try {
    await capsuleService.delete(id);
    setState(s => ({ ...s, capsule: null, isLoading: false }));
  } catch (e) {
    setState(s => ({ ...s, error: e as CapsuleError, isLoading: false }));
  }
}
```

**Benefits:**

- **Consistent Error Handling**: All service calls use the same pattern
- **No Duplication**: Same pattern for all operations (create, read, update, delete)
- **Type Safety**: Service returns `Capsule` or throws `CapsuleError`
- **Loading States**: Automatic loading state management

### **BigInt Handling Note**

**Important**: All time/count fields are `bigint`; don't stringify in state. Convert at render boundary.

```typescript
// ✅ CORRECT: Keep BigInt in state
const [state, setState] = useState<CapsuleState>({
  capsule: {
    id: "capsule_123",
    updated_at: 1704067200000000000n, // ← Keep as BigInt
    created_at: 1704067200000000000n, // ← Keep as BigInt
    inline_bytes_used: 1024000n, // ← Keep as BigInt
  },
});

// ✅ Convert only when displaying
const formatDate = (timestamp: bigint) => {
  return new Date(Number(timestamp / 1000000n)).toLocaleString();
};

const formatFileSize = (bytes: bigint) => {
  return `${Number(bytes / 1024n)} KB`;
};

// ✅ Use in JSX
<div>
  <p>Created: {formatDate(state.capsule.created_at)}</p>
  <p>Size: {formatFileSize(state.capsule.inline_bytes_used)}</p>
</div>;
```

**Why BigInt is Required:**

- **Backend Precision**: ICP uses nanosecond precision timestamps
- **Large Numbers**: File sizes and counts can be very large
- **Type Safety**: Frontend types must match backend exactly
- **No Data Loss**: Preserves all precision from backend

### 🚀 **Implementation Todo List**

#### **Phase 1: Foundation (Core Infrastructure)**

- [x] **1.1** Define `CapsuleState` interface with `capsule: Capsule | null` and `isLoading: boolean`
- [x] **1.2** Define `CapsuleError` type in `src/nextjs/src/types/capsule.ts`
- [x] ~~**1.3** Create `src/nextjs/src/hooks/useCapsulesState.ts` custom hook (optional)~~ **SKIPPED - Not needed for MVP**
- [x] **1.4** Update `src/nextjs/src/services/capsule.ts` to use backend types directly
- [ ] **1.5** Add request ID tracking utilities for stale update prevention

#### **Phase 2: Component Refactoring**

- [x] **2.1** Replace multiple `useState` with single `CapsuleState` in `capsule-info.tsx`
- [x] **2.2** Update all state setters to use new pattern (`setCapsule`, `setIsLoading`, `setError`)
- [x] **2.3** Implement proper API calls: `capsules_read_full()` → `Capsule`
- [x] **2.4** Update error handling to use typed `CapsuleError` instead of string errors
- [x] **2.5** Remove old state variables (`capsuleInfo`, `capsuleReadResult`, `busy`)
- [x] **2.6** Remove `setCapsuleState` helper function

#### **Phase 3: Service Layer Enhancement** ✅ **ALREADY COMPLETE**

- [x] **3.1** Update service functions to use backend API directly (no adapter needed) ✅ **Already Done**
- [x] **3.2** Update `getCapsulesList()` to call `actor.capsules_list()` → `CapsuleInfo[]` ✅ **Already Done**
- [x] **3.3** Update `getCapsuleStructure()` to call `actor.capsules_read_full()` → `Capsule` ✅ **Already Done**
- [x] **3.4** Update `createCapsule()` to use `actor.capsules_create()` ✅ **Already Done**
- [x] **3.5** Update `updateCapsule()` to use `actor.capsules_update()` ✅ **Already Done**
- [x] **3.6** Update `deleteCapsule()` to use `actor.capsules_delete()` ✅ **Already Done**

**Note**: Phase 3 was already complete - our service layer was already using backend APIs directly with proper error handling.

#### **Phase 4: Testing & Validation**

- [ ] **4.1** Test all capsule operations (create, read, update, delete)
- [ ] **4.2** Test error handling for each `CapsuleError` type
- [ ] **4.3** Test stale update prevention with overlapping requests
- [ ] **4.4** Verify BigInt serialization still works correctly
- [ ] **4.5** Test component re-renders and performance

#### **Phase 5: Cleanup & Documentation**

- [ ] **5.1** Remove unused imports and dead code
- [ ] **5.2** Update component documentation
- [ ] **5.3** Add JSDoc comments to new functions
- [ ] **5.4** Update any related tests
- [ ] **5.5** Verify build passes without errors

#### **Phase 6: Future Enhancements (Optional)**

- [ ] **6.1** Consider extracting `useCapsuleState` if used in ≥2 components
- [ ] **6.2** Add optimistic updates for better UX
- [ ] **6.3** Implement capsule collection state if needed
- [ ] **6.4** Add caching layer for frequently accessed capsules
- [ ] **6.5** Consider reducer pattern if state becomes complex

### 📋 **Implementation Priority**

**High Priority (Must Do):**

- ✅ Phase 1: Foundation (1.1, 1.2, 1.3, 1.4, 1.5)
- ✅ Phase 2: Component Refactoring (2.1, 2.2, 2.3, 2.4, 2.5, 2.6)
- ✅ Phase 3: Service Layer Enhancement (3.1, 3.2, 3.3, 3.4, 3.5, 3.6)

**Medium Priority (Should Do):**

- ✅ Phase 4: Testing & Validation (4.1, 4.2, 4.3, 4.4, 4.5)
- ✅ Phase 5: Cleanup & Documentation (5.1, 5.2, 5.3, 5.4, 5.5)

**Low Priority (Nice to Have):**

- ✅ Phase 6: Future Enhancements (6.1, 6.2, 6.3, 6.4, 6.5)

### 🎯 **Success Criteria**

- [ ] **Proper state structure**: `capsules: CapsuleInfo[]` for summaries, `currentCapsule: Capsule | null` for structure
- [ ] **Backend API alignment**: Use `capsules_list()` → `CapsuleInfo[]`, `capsules_read_full()` → `Capsule`
- [ ] **No unnecessary adapters**: Use backend types directly (they're already perfect)
- [ ] **Performance**: Load summaries first, structure on-demand, content only when needed
- [ ] **Type safety**: All errors properly typed, no `any` types
- [ ] **Clean separation**: Service layer handles IO, component handles UI
- [ ] **Maintainability**: Clear code structure, easy to understand and modify
