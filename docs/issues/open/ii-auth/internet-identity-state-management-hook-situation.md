# Internet Identity State Management - Current Hook Situation Analysis

## 📋 **Issue Summary**

**Status**: 🔍 **ANALYSIS** - Current hook architecture analysis for Phase 2 migration

**Goal**: Analyze existing hooks and plan the migration from complex `useIICoAuth()` to simplified `useIILinks()` hook.

## 🎯 **Current Hook Architecture**

### **Primary Hook: `useIICoAuth()`**

**Location**: `src/nextjs/src/hooks/use-ii-coauth.ts`

**Current Purpose**: Manages II co-auth state, TTL monitoring, and co-auth actions

**Current Interface**:

```typescript
interface IICoAuthState {
  // Account status
  hasLinkedII: boolean;
  isCoAuthActive: boolean;

  // Principal information
  linkedIcPrincipal?: string; // ❌ OLD: Single principal
  activeIcPrincipal?: string; // ❌ OLD: Active principal
  assertedAt?: number; // ❌ OLD: TTL timestamp
  loginProvider?: string;

  // TTL status
  ttlStatus: ReturnType<typeof checkIICoAuthTTL>;
  isExpired: boolean;
  isInGracePeriod: boolean;
  isWarning: boolean;
  requiresReAuth: boolean;

  // UI helpers
  statusMessage: string;
  statusClass: string;
  remainingMinutes: number;

  // Actions
  activateII: (principal: string) => Promise<void>; // ❌ OLD: Activation
  disconnectII: () => Promise<void>; // ❌ OLD: Session disconnect
  refreshTTL: () => void; // ❌ OLD: TTL refresh
}
```

**Current Dependencies**:

- `checkIICoAuthTTL` - TTL calculation utilities
- `requiresIIReAuth` - Re-authentication logic
- `useSession` - NextAuth session management

**Current Logic**:

- Complex TTL monitoring with auto-refresh every minute
- "Active" state management with `activeIcPrincipal`
- Session-based co-auth activation/deactivation
- UI status messages based on TTL state

### **Secondary Hooks**

#### **`useIICoAuthRequired()`**

**Location**: `src/nextjs/src/hooks/use-ii-coauth.ts` (lines 178-192)

**Current Purpose**: Checks if II co-auth is required for specific actions

**Current Logic**:

```typescript
const requiresIICoAuth = ["create-gallery-forever", "upload-to-icp", "sync-to-icp", "icp-storage-operation"].includes(
  action
);

return {
  ...coAuthState,
  requiresIICoAuth,
  canProceed: !requiresIICoAuth || coAuthState.isCoAuthActive, // ❌ OLD: Uses isCoAuthActive
  actionBlocked: requiresIICoAuth && !coAuthState.isCoAuthActive,
};
```

#### **`useIIActivationFlow()`**

**Location**: `src/nextjs/src/hooks/use-ii-coauth.ts` (lines 198-228)

**Current Purpose**: Manages II activation flow with loading states

**Current Features**:

- Loading states (`isActivating`)
- Error handling (`activationError`)
- Uses `useIICoAuth()` internally

### **Supporting Hooks**

#### **`useSession()`** (NextAuth)

**Location**: `next-auth/react`

**Current Usage**: Used by `useIICoAuth()` to access session data

**Current Interface**:

```typescript
const { data: session, update } = useSession();

// Session now has (after Phase 1):
session.user.linkedIcPrincipals?: string[];  // ✅ NEW: Multiple principals
```

#### **`useFileUpload()`**

**Location**: `src/nextjs/src/hooks/use-file-upload.ts`

**Current Usage**: Checks ICP authentication for file uploads

**Current Logic**:

```typescript
// Check ICP authentication if user has ICP in preferences
const userBlobHostingPreferences = preferences?.blobHosting || ["s3"];
if (userBlobHostingPreferences.includes("icp")) {
  try {
    await checkICPAuthentication();
  } catch (_error) {
    // Show authentication required toast
  }
}
```

## 🔄 **Migration Requirements**

### **What Needs to Change**

#### **1. Remove TTL Logic**

- ❌ Remove `checkIICoAuthTTL` dependency
- ❌ Remove `requiresIIReAuth` dependency
- ❌ Remove TTL monitoring (`useEffect` with intervals)
- ❌ Remove TTL-related state (`ttlStatus`, `isExpired`, `isInGracePeriod`, `isWarning`)

#### **2. Remove "Active" State**

- ❌ Remove `activeIcPrincipal` field
- ❌ Remove `icpPrincipalAssertedAt` field
- ❌ Remove `isCoAuthActive` logic
- ❌ Remove `activateII()` action
- ❌ Remove `disconnectII()` action
- ❌ Remove `refreshTTL()` action

#### **3. Simplify to "Linked" State**

- ✅ Keep `hasLinkedII` (derived from `linkedIcPrincipals.length > 0`)
- ✅ Change `linkedIcPrincipal` to `linkedIcPrincipals: string[]`
- ✅ Add `linkII(principal)` action
- ✅ Add `unlinkII(principal)` action
- ✅ Add `refreshLinks()` action

### **New Hook Interface**

#### **`useIILinks()` - Simplified Architecture**

```typescript
interface IILinksState {
  // Account status
  hasLinkedII: boolean;

  // Principal information
  linkedIcPrincipals: string[];
  loginProvider?: string;

  // Actions
  linkII: (principal: string) => Promise<void>;
  unlinkII: (principal: string) => Promise<void>;
  refreshLinks: () => Promise<void>;
}
```

#### **Implementation Strategy**

```typescript
export function useIILinks(): IILinksState {
  const { data: session, update } = useSession();

  // Extract from session (simplified)
  const linkedIcPrincipals = (session?.user as ExtendedSessionUser)?.linkedIcPrincipals || [];
  const loginProvider = (session?.user as ExtendedSessionUser)?.loginProvider;

  // Compute derived state
  const hasLinkedII = linkedIcPrincipals.length > 0;

  // Actions
  const linkII = useCallback(
    async (principal: string) => {
      // Call API to link principal
      // Update session with new linkedIcPrincipals array
    },
    [update]
  );

  const unlinkII = useCallback(
    async (principal: string) => {
      // Call API to unlink principal
      // Update session with updated linkedIcPrincipals array
    },
    [update]
  );

  const refreshLinks = useCallback(async () => {
    // Refresh linkedIcPrincipals from session
  }, [update]);

  return {
    hasLinkedII,
    linkedIcPrincipals,
    loginProvider,
    linkII,
    unlinkII,
    refreshLinks,
  };
}
```

## 📊 **Component Usage Analysis**

### **Current Components Using `useIICoAuth()`**

#### **1. `IICoAuthControls` Component**

**Location**: `src/nextjs/src/components/user/ii-coauth-controls.tsx`

**Current Usage**:

```typescript
const {
  hasLinkedII,
  isCoAuthActive, // ❌ OLD: Will be removed
  activeIcPrincipal, // ❌ OLD: Will be removed
  ttlStatus, // ❌ OLD: Will be removed
  isExpired, // ❌ OLD: Will be removed
  requiresReAuth, // ❌ OLD: Will be removed
  remainingMinutes, // ❌ OLD: Will be removed
  statusMessage, // ❌ OLD: Will be removed
  statusClass, // ❌ OLD: Will be removed
  disconnectII, // ❌ OLD: Will be removed
  refreshTTL, // ❌ OLD: Will be removed
} = useIICoAuth();
```

**Migration Required**:

- Remove all TTL-related UI elements
- Remove "Activate" button functionality
- Remove "Extend Session" button functionality
- Remove "Disconnect for This Session" button functionality
- Update to show linked principals list only
- Add "Link new" button functionality
- Add "Unlink" button functionality for each principal

#### **2. `LinkedAccounts` Component**

**Location**: `src/nextjs/src/components/user/linked-accounts.tsx`

**Current Usage**: Likely uses `useIICoAuth()` for II account management

**Migration Required**:

- Update to use `useIILinks()` hook
- Remove `activeIcPrincipal` display logic
- Remove `isCoAuthActive` state logic
- Update to show list of linked principals only

#### **3. ICP Page Component**

**Location**: `src/nextjs/src/app/[lang]/user/icp/page.tsx`

**Current Usage**: Uses local authentication state and `handleLogin()` function

**Migration Required**:

- Remove local authentication state (`isAuthenticated`, `principalId`)
- Remove `handleLogin()` function
- Remove `handleLogout()` function
- Update to use `useIILinks()` hook for linked principals display
- Update canister calls to use ICP Auth Client directly

### **Supporting Hooks Migration**

#### **`useIICoAuthRequired()` → `useIILinksRequired()`**

**Current Logic**:

```typescript
const canProceed = !requiresIICoAuth || coAuthState.isCoAuthActive; // ❌ OLD
```

**New Logic**:

```typescript
const canProceed = !requiresIICoAuth || linksState.hasLinkedII; // ✅ NEW
```

#### **`useIIActivationFlow()` → `useIILinksFlow()`**

**Current Features**:

- Loading states for activation
- Error handling for activation

**New Features**:

- Loading states for linking/unlinking
- Error handling for linking/unlinking

## 🎯 **Migration Plan**

### **Phase 2.1: Create New Hook**

1. **Create `useIILinks()` hook**

   - Implement simplified interface
   - Remove all TTL logic
   - Remove all "active" state logic
   - Add link/unlink actions

2. **Create supporting hooks**
   - `useIILinksRequired()` for action checks
   - `useIILinksFlow()` for link/unlink flows

### **Phase 2.2: Update Components**

1. **Update `IICoAuthControls`**

   - Remove TTL-related UI elements
   - Remove activation/deactivation buttons
   - Add link/unlink functionality
   - Update to show linked principals list

2. **Update `LinkedAccounts`**

   - Remove active principal display
   - Update to use new hook
   - Add link/unlink functionality

3. **Update ICP Page**
   - Remove local authentication state
   - Update to use new hook
   - Update canister calls

### **Phase 2.3: Remove Old Hook**

1. **Delete `useIICoAuth()` hook**
2. **Remove TTL utilities** (`checkIICoAuthTTL`, `requiresIIReAuth`)
3. **Update all imports** to use new hook
4. **Clean up unused code**

## 🚀 **Benefits of New Architecture**

### **Simplified State Management**

- ✅ No complex TTL monitoring
- ✅ No "active" state synchronization
- ✅ Clear ownership boundaries
- ✅ Reduced complexity

### **Better User Experience**

- ✅ No confusion about "active" vs "linked" states
- ✅ User has full control over which II to use
- ✅ Clear UI showing linked principals
- ✅ Easy link/unlink functionality

### **Maintainable Code**

- ✅ Single source of truth for linked principals
- ✅ No race conditions between systems
- ✅ Clear separation of concerns
- ✅ Easier testing and debugging

## 📋 **Next Steps**

1. **Create `useIILinks()` hook** with simplified interface
2. **Update components** to use new hook
3. **Create API endpoints** for link/unlink operations
4. **Test thoroughly** to ensure all functionality works
5. **Remove old hook** and clean up code

---

**Priority**: 🔴 **HIGH** - Core hook migration affecting multiple components and user experience.

**Estimated Effort**: 2-3 days for complete hook migration and testing.

**Dependencies**: Phase 1 completion (Schema Migration) and API endpoint implementation.

## 🤔 **What Can We Keep? - Questions for Tech Lead**

### **Potential Reusable Components**

#### **1. Session Management Logic**

**Current**: `useSession()` integration with `update()` function
**Question**: Can we keep the basic session update pattern for `linkedIcPrincipals`?

```typescript
// Current pattern that works well
const { data: session, update } = useSession();
await update({ linkedIcPrincipals: newArray });
```

#### **2. Error Handling Patterns**

**Current**: Comprehensive error handling in `useIIActivationFlow()`
**Question**: Can we reuse the error handling patterns for link/unlink operations?

```typescript
// Current error handling pattern
const [isActivating, setIsActivating] = useState(false);
const [activationError, setActivationError] = useState<string | null>(null);
```

#### **3. Loading State Management**

**Current**: Loading states for activation flow
**Question**: Can we reuse loading state patterns for link/unlink operations?

```typescript
// Current loading pattern
const [isActivating, setIsActivating] = useState(false);
```

#### **4. Action-Specific Logic**

**Current**: `useIICoAuthRequired()` for action checks
**Question**: Can we keep the action-specific logic but simplify the check?

```typescript
// Current action list
const requiresIICoAuth = ["create-gallery-forever", "upload-to-icp", "sync-to-icp", "icp-storage-operation"].includes(
  action
);

// New simplified check
const canProceed = !requiresIICoAuth || linksState.hasLinkedII;
```

#### **5. UI Helper Functions**

**Current**: Status messages and CSS classes for UI
**Question**: Can we keep some UI helpers but simplify them?

```typescript
// Current UI helpers (complex)
statusMessage: ttlStatus.status === 'active' ? `II Active (${ttlStatus.remainingMinutes}m remaining)` : ...
statusClass: ttlStatus.status === 'active' ? 'text-green-600' : ...

// New simplified helpers
statusMessage: hasLinkedII ? `Linked (${linkedIcPrincipals.length} principals)` : 'No II linked';
statusClass: hasLinkedII ? 'text-green-600' : 'text-gray-500';
```

### **Questions for Tech Lead**

1. **Session Update Pattern**: Should we keep the current `update()` pattern for managing `linkedIcPrincipals`?

2. **Error Handling**: Can we reuse the current error handling patterns for link/unlink operations?

3. **Loading States**: Should we keep the current loading state management for link/unlink operations?

4. **Action Checks**: Can we keep the current action-specific logic but simplify the `canProceed` check?

5. **UI Helpers**: Should we keep some UI helper functions but simplify them for the new architecture?

6. **Hook Structure**: Should we keep the current hook structure (main hook + supporting hooks) or create a single simplified hook?

7. **Component Integration**: Are there any current component integration patterns we should preserve?

8. **Testing**: Should we keep the current test structure and adapt it for the new hook?

### **Proposed Hybrid Approach**

Instead of completely replacing everything, we could:

1. **Keep the hook structure** but simplify the interface
2. **Keep error handling** but adapt it for link/unlink operations
3. **Keep loading states** but simplify them
4. **Keep action checks** but update the logic
5. **Keep UI helpers** but simplify them
6. **Keep testing patterns** but update the tests

This would provide a smoother migration path and preserve working patterns while implementing the simplified architecture.

## ✅ **Tech Lead's Decision - Final Architecture**

### **Decision**

Keep the working patterns, drop the "active/TTL" bits. Ship a single primary hook `useIILinks()` plus two tiny helpers. **No compatibility layer needed** - we're doing greenfield development.

### **Keep vs. Drop**

**✅ Keep:**

- `useSession().update()` pattern
- Loading/error patterns
- Action checks (but simpler)
- UI helpers (simplified)
- Existing test style

**❌ Drop:**

- `activeIcPrincipal`
- TTL logic
- "activate/refresh/disconnect" actions
- Any local auth state

### **Final Hook Architecture**

#### **1. `useIILinks()` - Primary Hook**

```typescript
// src/hooks/use-ii-links.ts
"use client";
import { useCallback } from "react";
import { useSession } from "next-auth/react";

export function useIILinks() {
  const { data: session, update, status } = useSession();
  const linked = session?.user?.linkedIcPrincipals ?? [];
  const hasLinkedII = linked.length > 0;

  const linkII = useCallback(
    async (principal: string) => {
      const res = await fetch("/api/auth/ii/link", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ principal }),
      });
      if (!res.ok) throw new Error("Link failed");
      const { linkedIcPrincipals } = await res.json();
      await update({ linkedIcPrincipals });
    },
    [update]
  );

  const unlinkII = useCallback(
    async (principal: string) => {
      const res = await fetch("/api/auth/ii/unlink", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ principal }),
      });
      if (!res.ok) throw new Error("Unlink failed");
      const { linkedIcPrincipals } = await res.json();
      await update({ linkedIcPrincipals });
    },
    [update]
  );

  const refreshLinks = useCallback(async () => {
    const res = await fetch("/api/auth/ii/linked");
    if (!res.ok) throw new Error("Refresh failed");
    const { linkedIcPrincipals } = await res.json();
    await update({ linkedIcPrincipals });
  }, [update]);

  return { status, hasLinkedII, linkedIcPrincipals: linked, linkII, unlinkII, refreshLinks };
}
```

#### **2. `useIILinksRequired(action: string)` - Action Gate**

```typescript
export function useIILinksRequired(action: string) {
  const { hasLinkedII } = useIILinks();
  const requires = ["create-gallery-forever", "upload-to-icp", "sync-to-icp", "icp-storage-operation"].includes(action);
  return { requires, canProceed: !requires || hasLinkedII, blocked: requires && !hasLinkedII };
}
```

#### **3. `useIILinksFlow()` - Loading/Error Flow (Optional)**

```typescript
export function useIILinksFlow() {
  const { linkII, unlinkII } = useIILinks();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  // wrap linkII/unlinkII with setLoading/setError; return handlers
}
```

### **Component Migration Strategy**

#### **`IICoAuthControls`**

- Remove TTL/activate UI
- Show list of `linkedIcPrincipals` + "Link new" + per-item "Unlink"

#### **`LinkedAccounts`**

- Switch to `useIILinks()`
- No "active" badge
- If displaying runtime principal in avatar, read it live from `@dfinity/auth-client` (ephemeral)

#### **ICP Page**

- Delete local auth state
- Rely on auth-client for actors, `useIILinks()` for links

### **API Contract**

- `POST /api/auth/ii/link { principal }` → verifies with nonce/proof → returns `{ linkedIcPrincipals: string[] }`
- `POST /api/auth/ii/unlink { principal }` → returns `{ linkedIcPrincipals }`
- `GET /api/auth/ii/linked` → returns `{ linkedIcPrincipals }`

### **Implementation Checklist**

- ✅ Add `useIILinks.ts` + helpers
- ✅ Implement `/api/auth/ii/{link,unlink,linked}` returning arrays
- ✅ Remove TTL utilities and any `activeIcPrincipal` references
- ✅ Update tests; snapshot UI for linked/unlinked states
- ✅ Docs: short "II Links" page (what's durable vs runtime)

**Result**: Keeps the ergonomics we already have, kills the risky parts, and gives us a clean seam for further evolution.
