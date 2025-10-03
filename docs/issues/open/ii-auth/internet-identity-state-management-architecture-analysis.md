# Internet Identity State Management Architecture Analysis

## 📋 **Issue Summary**

**Status**: 🔍 **ANALYSIS** - Comprehensive analysis of II state management architecture

**Problem**: The current Internet Identity authentication state is managed through multiple, disconnected systems creating complexity, inconsistency, and potential race conditions. We need to understand and potentially consolidate the state management approach.

## 🎯 **Core Problem Statement**

The application currently uses **three separate state management systems** for Internet Identity authentication, each with different persistence, scope, and synchronization characteristics:

1. **NextAuth JWT Session State** (Global, Server-side)
2. **Local React State** (Component-level, Client-side)
3. **ICP Auth Client State** (Browser-based, Client-side)

This creates a complex state synchronization problem where components may show different authentication states depending on which system they read from.

## 🔍 **Current State Management Architecture**

### **1. NextAuth JWT Session State (Global)**

**Location**: `src/nextjs/auth.ts` - JWT callback and session callback
**Persistence**: Encrypted JWT tokens stored in HTTP-only cookies
**Scope**: Global application state
**Data Stored**:

```typescript
// JWT Token Fields
interface JWT {
  loginProvider?: string; // Base session provider (e.g., "google")
  activeIcPrincipal?: string; // Currently active II principal
  activeIcPrincipalAssertedAt?: number; // Timestamp of II proof verification
  linkedIcPrincipal?: string; // Permanently linked II principal
}

// Session Fields (exposed to components)
interface Session {
  user: {
    icpPrincipal?: string; // Active II principal (when co-auth active)
    linkedIcPrincipal?: string; // Linked II principal (always available)
    loginProvider?: string; // Base session provider
    icpPrincipalAssertedAt?: number; // TTL timestamp
  };
}
```

**How it works**:

- State is managed in the JWT callback (`auth.ts:342-437`)
- Session callback exposes data to components (`auth.ts:440-477`)
- Components access via `useSession()` hook
- Updates via `session.update()` calls

**Pros**:

- ✅ Global state accessible everywhere
- ✅ Server-side validation and security
- ✅ Automatic persistence across page reloads
- ✅ Type-safe with TypeScript interfaces

**Cons**:

- ❌ Complex JWT callback logic (100+ lines)
- ❌ Database lookups on every session read
- ❌ Race conditions between fresh sign-in and updates
- ❌ Difficult to debug state changes

### **2. Local React State (Component-level)**

**Location**: `src/nextjs/src/app/[lang]/user/icp/page.tsx`
**Persistence**: Component state only (lost on page reload)
**Scope**: Single component
**Data Stored**:

```typescript
// ICP Page Local State
const [isAuthenticated, setIsAuthenticated] = useState(false);
const [principalId, setPrincipalId] = useState<string>("");
const [greeting, setGreeting] = useState("");
const [isRehydrating, setIsRehydrating] = useState(true);
```

**How it works**:

- State managed with `useState()` hooks
- Updated via `handleLogin()` function
- Persisted across page reloads via `useEffect()` rehydration
- Uses `getAuthClient()` to check authentication state

**Pros**:

- ✅ Simple and direct
- ✅ Fast updates without server round-trips
- ✅ Component-specific state isolation

**Cons**:

- ❌ Not shared between components
- ❌ Lost on page navigation
- ❌ No server-side validation
- ❌ Duplicates authentication logic

### **3. ICP Auth Client State (Browser-based)**

**Location**: `@dfinity/auth-client` library
**Persistence**: Browser localStorage/sessionStorage
**Scope**: Browser tab/window
**Data Stored**:

```typescript
// ICP Auth Client manages:
- Identity object (cryptographic keys)
- Principal object (unique identifier)
- Authentication state (isAuthenticated)
- Session persistence across browser sessions
```

**How it works**:

- Managed by `@dfinity/auth-client` library
- Accessed via `getAuthClient()` function
- Persists across browser sessions
- Provides `Identity` object for canister calls

**Pros**:

- ✅ Official ICP authentication library
- ✅ Handles cryptographic operations
- ✅ Persistent across browser sessions
- ✅ Required for canister interactions

**Cons**:

- ❌ Not integrated with NextAuth
- ❌ No server-side access
- ❌ Separate from application session state

## 🔄 **State Synchronization Flows**

### **Flow 1: II-Only Signin (NextAuth Integration)**

```typescript
// 1. User authenticates with II
const { identity, principal } = await loginWithII();

// 2. NextAuth session created
await signIn("ii", { callbackUrl });

// 3. JWT callback processes authentication
// - Sets loginProvider = 'internet-identity'
// - Sets activeIcPrincipal = principal
// - Sets linkedIcPrincipal = principal

// 4. Session callback exposes to components
// - session.user.icpPrincipal = principal
// - session.user.linkedIcPrincipal = principal
```

### **Flow 2: ICP Page Direct Authentication (Local State)**

```typescript
// 1. User authenticates with II
const { identity, principal } = await loginWithII();

// 2. Local state updated
setIsAuthenticated(true);
setPrincipalId(principal.toString());

// 3. Actor created for canister calls
const actor = await backendActor(identity);
authenticatedActorRef.current = actor;

// 4. NextAuth session NOT updated
// - Components using useSession() don't see authentication
// - LinkedAccounts shows "not linked yet"
```

### **Flow 3: Co-Authentication (Session Update)**

```typescript
// 1. User already has NextAuth session (Google/GitHub)
// 2. User authenticates with II
const { identity, principal } = await loginWithII();

// 3. Session updated via update() call
await update({
  activeIcPrincipal: principal,
  icpPrincipalAssertedAt: Date.now(),
});

// 4. JWT callback processes update
// - Sets activeIcPrincipal = principal
// - Sets activeIcPrincipalAssertedAt = timestamp
// - Keeps existing linkedIcPrincipal

// 5. Session callback exposes to components
// - session.user.icpPrincipal = principal (temporary)
// - session.user.linkedIcPrincipal = existing (permanent)
```

## 🚨 **Current Problems**

### **1. State Inconsistency**

Different components read from different state sources:

```typescript
// LinkedAccounts component (NextAuth session)
const { hasLinkedII, linkedIcPrincipal } = useIICoAuth();
// Reads from: session.user.linkedIcPrincipal

// ICP Page component (Local state)
const [isAuthenticated, setIsAuthenticated] = useState(false);
// Reads from: Local React state

// IICoAuthControls component (NextAuth session)
const { isCoAuthActive, activeIcPrincipal } = useIICoAuth();
// Reads from: session.user.icpPrincipal
```

### **2. Race Conditions**

JWT callback has complex logic for handling different scenarios:

```typescript
// Fresh sign-in detection
if (trigger === "signIn" && account) {
  // Set base session provider
  token.loginProvider = account.provider;

  // Clear II co-auth on non-II sign-in
  if (account.provider !== "internet-identity") {
    delete token.activeIcPrincipal;
    delete token.activeIcPrincipalAssertedAt;
  }
}

// Session update handling
if (trigger === "update" && session) {
  // Handle II co-auth activation
  if (session.activeIcPrincipal) {
    token.activeIcPrincipal = session.activeIcPrincipal;
    token.activeIcPrincipalAssertedAt = session.icpPrincipalAssertedAt;
  }
}
```

### **3. Database Lookups on Every Session Read**

```typescript
// JWT callback does database lookup for linkedIcPrincipal
if (!token.linkedIcPrincipal) {
  const iiAccount = await db.query.accounts.findFirst({
    where: (a, { and, eq }) => and(eq(a.userId, token.sub), eq(a.provider, "internet-identity")),
  });
  if (iiAccount) {
    token.linkedIcPrincipal = iiAccount.providerAccountId;
  }
}
```

### **4. TTL Management Complexity**

```typescript
// TTL checking in useIICoAuth hook
const checkIICoAuthTTL = (assertedAt?: number) => {
  if (!assertedAt) return { status: "inactive", remainingMinutes: 0 };

  const now = Date.now();
  const elapsed = now - assertedAt;
  const remaining = Math.max(0, TTL_DURATION - elapsed);

  if (remaining <= 0) return { status: "expired", remainingMinutes: 0 };
  if (remaining <= WARNING_THRESHOLD) return { status: "warning", remainingMinutes: Math.ceil(remaining / 60000) };
  return { status: "active", remainingMinutes: Math.ceil(remaining / 60000) };
};
```

## 💡 **Agreed Solution (Tech Lead + Team)**

### **Simplified NextAuth Architecture (Approved)**

**Approach**: Use NextAuth as the single source of truth with simplified schema and basic TTL

**Key Changes**:

- ✅ Simple TTL system (6 hours, no user control for MVP)
- ✅ Support multiple linked principals (`linkedIcPrincipals: string[]`) - users may lose/forget principals
- ✅ DB reads on signup/signin only (not in callbacks)
- ✅ Thin `useIICoAuth()` hook over `useSession()`
- ✅ No React Context needed

### **New Canonical Schema**

```typescript
// JWT Token (Server-side)
type JWT = {
  loginProvider?: "google" | "github" | "internet-identity";
  linkedIcPrincipals?: string[]; // 0..n linked II principals
  activeIcPrincipal?: string | null; // currently active principal (co-auth)
  activeSince?: number; // timestamp when co-auth was activated (for 6h TTL)
};

// Session User (Client-side, read-only mirror)
type SessionUser = {
  loginProvider?: string;
  linkedIcPrincipals?: string[];
  icpPrincipal?: string | null; // mirrors activeIcPrincipal
  activeSince?: number; // for TTL calculation
};
```

### **Simplified JWT Callback**

```typescript
// auth.ts - Simplified callback with DB reads on signin only
async jwt({ token, trigger, account, session }) {
  // On sign-in, set base provider and check DB for linked principals
  if (trigger === 'signIn' && account) {
    token.loginProvider = account.provider;

    // Always read DB on signin to get current linked principals
    const linkedPrincipals = await getLinkedPrincipalsFromDB(token.sub);
    token.linkedIcPrincipals = linkedPrincipals;

    // If II sign-in, set active to that principal
    if (account.provider === 'internet-identity') {
      const principal = account.providerAccountId;
      token.activeIcPrincipal = principal;
      token.activeSince = Date.now();
    } else {
      // Non-II sign-in clears only active co-auth
      token.activeIcPrincipal = null;
      token.activeSince = undefined;
    }
  }

  // Client-initiated mutations via session.update()
  if (trigger === 'update' && session) {
    if ('activeIcPrincipal' in session) {
      token.activeIcPrincipal = session.activeIcPrincipal ?? null;
      token.activeSince = session.activeIcPrincipal ? Date.now() : undefined;
    }
    if ('linkedIcPrincipals' in session && Array.isArray(session.linkedIcPrincipals)) {
      token.linkedIcPrincipals = session.linkedIcPrincipals;
    }
  }

  return token;
}
```

### **Minimal Client Hook**

```typescript
// useIICoAuth.ts - Thin wrapper over useSession with TTL
export function useIICoAuth() {
  const { data: session, update, status } = useSession();
  const linked = session?.user.linkedIcPrincipals ?? [];
  const active = session?.user.icpPrincipal ?? null;
  const activeSince = session?.user.activeSince;

  // Simple TTL check (6 hours)
  const isExpired = activeSince && Date.now() - activeSince > 6 * 60 * 60 * 1000;
  const isCoAuthActive = !!active && !isExpired;

  return {
    status, // 'loading' | 'authenticated' | 'unauthenticated'
    linkedIcPrincipals: linked,
    activeIcPrincipal: isCoAuthActive ? active : null,
    hasLinkedII: linked.length > 0,
    isCoAuthActive,
    isExpired,
    // helpers
    setActive: (p: string | null) => update({ activeIcPrincipal: p }),
    setLinked: (arr: string[]) => update({ linkedIcPrincipals: arr }),
  };
}
```

### **Key Benefits of This Approach**

- ✅ **Simple TTL system** - 6 hours automatic expiry, no user control for MVP
- ✅ **DB reads on signin only** - Fresh data on authentication, cached in JWT
- ✅ **Multiple linked principals** - Users may lose/forget principals, need multiple options
- ✅ **Simplified state management** - Single source of truth in NextAuth
- ✅ **No React Context needed** - `useSession()` already provides global state
- ✅ **Clear separation** - NextAuth for auth state, ICP Auth Client for crypto operations

## 🔐 **Our Internet Identity Verification System**

### **How II Authentication Works in Our Web2 App**

The tech lead may not be familiar with our specific implementation, so here's how our II verification system works:

#### **1. Challenge/Nonce System**

```typescript
// Step 1: User requests II authentication
// GET /api/ii/challenge
const challenge = await generateNonce(); // Store in DB with expiration
return { challenge };

// Step 2: User signs challenge with II
const signature = await identity.sign(challenge);

// Step 3: Verify signature server-side
// POST /api/ii/verify-nonce
const isValid = await verifySignature(challenge, signature, principal);
if (isValid) {
  // Link principal to user account in DB
  await linkPrincipalToUser(userId, principal);
}
```

#### **2. Database Integration**

```typescript
// Our accounts table structure
interface Account {
  userId: string;
  provider: "internet-identity";
  providerAccountId: string; // This is the II principal
  type: "oauth";
  // ... other fields
}

// When user links II principal
await db.insert(accounts).values({
  userId: session.user.id,
  provider: "internet-identity",
  providerAccountId: principal, // The II principal
  type: "oauth",
});
```

#### **3. Co-Authentication Flow**

```typescript
// User already has Google/GitHub session
// User wants to activate II co-auth

// 1. User authenticates with II (challenge/verify)
const { principal } = await authenticateWithII();

// 2. Check if principal is linked to user
const isLinked = await checkIfPrincipalLinked(userId, principal);

// 3. If linked, activate co-auth
if (isLinked) {
  await session.update({
    activeIcPrincipal: principal,
    activeSince: Date.now(),
  });
}
```

#### **4. Canister Calls with II**

```typescript
// When user makes ICP canister calls
const identity = await getAuthClient().getIdentity();
const actor = await backendActor(identity);
const result = await actor.someMethod();
```

### **Key Points for Tech Lead**

1. **We already have challenge/verification system** - No changes needed to crypto flow
2. **Database is source of truth for linked principals** - JWT is just cache
3. **Co-auth is temporary** - Only active during session, not permanent
4. **Multiple principals needed** - Users lose/forget principals, need backup options
5. **Simple TTL sufficient** - 6 hours auto-expiry, no complex user controls

### **Implementation Flows**

#### **Link Another II (MVP)**

```typescript
// After server verifies challenge/verification
// Update both DB (accounts table) and session
await unstable_update({
  linkedIcPrincipals: newArray,
});
// Do not read DB in callbacks
```

#### **Activate Co-Auth**

```typescript
// Verify proof server-side (existing system)
// Then update session
await unstable_update({
  activeIcPrincipal: principal,
});
```

#### **Clear Co-Auth (Manual)**

```typescript
await unstable_update({
  activeIcPrincipal: null,
});
```

#### **Logout (Authoritative)**

```typescript
// NextAuth signOut() + ICP Auth Client logout()
await signOut();
await authClient.logout(); // Clear browser-held identity
// No TTL to manage
```

### **UI Usage Pattern**

```typescript
const { isCoAuthActive, activeIcPrincipal, linkedIcPrincipals, setActive } = useIICoAuth();

<button onClick={() => setActive(null)}>Disconnect II</button>
<ul>
  {linkedIcPrincipals.map(p => (
    <li key={p}>
      {p}
      {activeIcPrincipal === p
        ? <em>(active)</em>
        : <button onClick={() => setActive(p)}>Use this</button>}
    </li>
  ))}
</ul>
```

### **Edge Cases / Guardrails**

- **Data drift handling**: If `activeIcPrincipal` not in `linkedIcPrincipals`, auto-add to linked on session.update
- **Duplicate prevention**: Always use `Array.from(new Set([...]))` when linking
- **Multi-tab sync**: NextAuth's cookie session is fine; optional `BroadcastChannel` for instant UI sync

### **DB Alignment (MVP)**

- Keep `accounts` table as durable source of linked principals
- On link/unlink API, update DB + session in same handler
- No per-request DB hydration in callbacks

### **Proposed API Handlers**

The tech lead can provide exact implementations for:

- `/api/auth/ii/activate` - Activate co-auth with principal
- `/api/auth/ii/link` - Link new II principal
- `/api/auth/ii/unlink` - Unlink II principal

These will integrate with existing challenge/verification system without changes to crypto flow.

## 🎯 **Implementation Plan (Tech Lead Approved)**

### **Phase 1: Schema Migration**

1. **Update JWT Interface**: Replace current schema with new simplified schema

   - Remove `activeIcPrincipalAssertedAt` (TTL field)
   - Change `linkedIcPrincipal` to `linkedIcPrincipals: string[]`
   - Keep `activeIcPrincipal` as `string | null`

2. **Update Session Interface**: Mirror JWT changes in session user interface

   - Update `SessionUser` type definitions
   - Update session callback to map new fields

3. **Simplify JWT Callback**: Remove database reads and TTL logic
   - Remove complex TTL management
   - Remove database lookups for `linkedIcPrincipal`
   - Implement simplified logic per tech lead's specification

### **Phase 2: Hook Migration**

1. **Update `useIICoAuth()` Hook**: Implement new thin wrapper

   - Remove TTL-related state and logic
   - Add support for multiple linked principals
   - Add helper functions `setActive()` and `setLinked()`

2. **Update Components**: Migrate all components to new hook
   - Remove local authentication state from ICP page
   - Update `LinkedAccounts` component for multiple principals
   - Update `IICoAuthControls` component for new schema

### **Phase 3: API Implementation**

1. **Create API Handlers**: Implement new API endpoints

   - `/api/auth/ii/activate` - Activate co-auth
   - `/api/auth/ii/link` - Link new II principal
   - `/api/auth/ii/unlink` - Unlink II principal

2. **Update Existing APIs**: Modify existing challenge/verification APIs
   - Integrate with new session update pattern
   - Remove TTL-related logic
   - Add support for multiple principals

### **Phase 4: Testing & Validation**

1. **State Consistency Testing**: Ensure all components show same state
2. **Multi-Principal Testing**: Test linking/unlinking multiple II accounts
3. **Session Persistence Testing**: Verify state survives page reloads
4. **Error Handling Testing**: Test edge cases and error scenarios

### **Key Benefits of New Architecture**

- ✅ **Simplified State Management**: Single source of truth in NextAuth
- ✅ **No TTL Complexity**: Manual co-auth management only
- ✅ **Multiple Principal Support**: Users can link multiple II accounts
- ✅ **Better Performance**: No database reads in JWT callbacks
- ✅ **Cleaner Code**: Removed complex state synchronization logic
- ✅ **Future-Proof**: Easy to extend with additional features

## 🔧 **Technical Implementation**

### **State Management Hierarchy**

```
1. NextAuth JWT Session (Global, Server-side)
   ├── loginProvider (base authentication method)
   ├── linkedIcPrincipals (array of linked II principals)
   ├── activeIcPrincipal (current II session)
   └── activeSince (TTL timestamp - if TTL enabled)

2. ICP Auth Client (Browser, Cryptographic)
   ├── Identity object (signing keys)
   ├── Principal object (unique identifier)
   └── Authentication state (isAuthenticated)

3. React Context (UI State, Optional)
   ├── Loading states
   ├── Error states
   └── UI-specific state
```

### **Component State Reading**

```typescript
// All components should use this pattern:
export function MyComponent() {
  const { data: session } = useSession();
  const { hasLinkedII, isCoAuthActive, activeIcPrincipal, linkedIcPrincipal, statusMessage, remainingMinutes } =
    useIICoAuth();

  // No local authentication state
  // No direct ICP Auth Client access
  // All state from NextAuth session
}
```

## 📊 **Current State Map**

| State Source        | Location  | Persistence     | Scope     | Used By                              |
| ------------------- | --------- | --------------- | --------- | ------------------------------------ |
| **NextAuth JWT**    | `auth.ts` | HTTP cookies    | Global    | `LinkedAccounts`, `IICoAuthControls` |
| **Local React**     | ICP page  | Component state | Component | ICP page only                        |
| **ICP Auth Client** | Browser   | localStorage    | Browser   | ICP page, canister calls             |

## 🎯 **Next Steps**

1. **Create State Audit**: Document all current state usage
2. **Design Migration Plan**: How to consolidate state sources
3. **Implement Centralized State**: Use NextAuth as single source
4. **Remove Redundant State**: Eliminate local authentication state
5. **Test State Consistency**: Ensure all components show same state
6. **Document New Architecture**: Clear state management guidelines

---

**Priority**: 🔴 **HIGH** - State management complexity is causing user experience issues and development confusion.

**Estimated Effort**: 2-3 days for audit and consolidation, 1-2 days for testing and documentation.

## ❓ **Tech Lead's Answers**

### **1. Database Consistency** ✅ **ANSWERED**

**Question**: We're reading DB on signin but using JWT as cache. What happens if:

- User links II principal on Device A
- User signs in on Device B (won't see the new principal until next signin)
- Is this acceptable for MVP, or do we need real-time sync?

**Answer**: MVP: acceptable to show on Device B after next sign-in/page refresh. To reduce surprise without adding complexity:

- On any view that needs fresh links (e.g., "Choose II principal"), call a lightweight `GET /api/auth/ii/linked` to read DB directly and then `session.update({ linkedIcPrincipals })`. No callback DB reads.

### **2. Multiple Principals UX** ✅ **ANSWERED**

**Question**: For multiple linked principals, what's the UX?

**Answer**:

- **UI**: List with "Activate" button per principal; mark "(active)".
- **Default**: Auto-activate last used after successful II proof.
- **"Forgot which one"**: Offer "Try II sign-in" path; if the proved principal isn't linked, show "Link this principal" CTA.
- **Optional labels**: Allow nicknames per principal to avoid confusion.

### **3. TTL Implementation** ✅ **ANSWERED**

**Question**: For the 6-hour TTL:

**Answer**: (Only if you keep TTL)

- **Check expiry**: In the hook for UX; also enforce server-side on sensitive routes (idempotent).
- **On expiry during use**: Flip `isCoAuthActive=false`, toast "II session expired—re-verify to continue".
- **Warning**: Optional 5-minute client timer; easy, but skip for strict MVP if you want less code.

### **4. Error Handling** ✅ **ANSWERED**

**Question**: What should happen when:

**Answer**:

- **`session.update()` failure**: Optimistic UI off; show toast and refetch `getSession()`; keep previous state.
- **Corrupted JWT**: Force sign-out, clear ICP auth client, redirect to login.
- **DB ↔ JWT drift**: APIs that mutate links must update DB first; on success, call `unstable_update()`; add idempotency keys to avoid double-links.

### **5. Migration Strategy** ✅ **ANSWERED**

**Question**: For existing users with current TTL system:

**Answer**: If any legacy users: map `linkedIcPrincipal` → `linkedIcPrincipals=[value]`; map `activeIcPrincipalAssertedAt` → `activeSince`. Do this lazily in `jwt()` on first touch; no separate migration script needed.

### **6. API Endpoints** ✅ **ANSWERED**

**Question**: You mentioned providing exact implementations for:

**Answer**: (Contracts; plugs into your existing challenge/verify)

```typescript
// POST /api/auth/ii/activate
// body: { principal, proof }
// 1) verifyProof(principal, proof) using your system
// 2) ensure principal ∈ user's linked set (or auto-link if you allow)
// 3) unstable_update({ activeIcPrincipal: principal, ...(TTL? { activeSince: Date.now() } : {}) })

// POST /api/auth/ii/link
// body: { principal, proof }
// 1) verifyProof()
// 2) upsert into accounts(provider='internet-identity', providerAccountId=principal)
// 3) read all linked principals from DB; unstable_update({ linkedIcPrincipals })

// POST /api/auth/ii/unlink
// body: { principal }
// 1) delete from accounts for that principal
// 2) if principal === activeIcPrincipal -> also clear active
// 3) read all linked principals; unstable_update({ linkedIcPrincipals, ...(cleared active if needed) })

// GET /api/auth/ii/linked
// returns latest array from DB; client then calls session.update({ linkedIcPrincipals })
```

### **7. Performance** ✅ **ANSWERED**

**Question**: With DB reads on every signin:

**Answer**: Fine. Add a tiny in-memory cache on the API (per user, 1–5 min) if you expect bursts. If DB is down during `/link`: fail fast; don't update session. If down during `/linked`: return current JWT state as fallback.

### **8. Security** ✅ **ANSWERED**

**Question**: For the 6-hour TTL:

**Answer**: 6h TTL is okay for medium-risk. For anything sensitive:

- Require fresh II proof within N minutes (step-up auth) regardless of JWT state.
- Different TTLs per operation are easy to implement at the API boundary (policy table).
- Multiple linked principals don't weaken auth if you always verify ownership on link and activation.

## 🎯 **Final Decision Required**

### **TTL Decision**

The tech lead noted we need to decide between:

**Option A: No-TTL MVP** (Simplest)

- End co-auth only on logout/clear
- No `activeSince` field needed
- Simplest implementation

**Option B: TTL-MVP (6h)** (Security hardening)

- 6-hour automatic expiry
- Add `activeSince` field
- Client check + server enforcement on sensitive endpoints

**Recommendation**: Choose **Option A (No-TTL)** for MVP since:

- We're not responsible for II management
- Users can manually disconnect when needed
- Simpler implementation
- Can add TTL later if needed

### **Tech Lead's Final Architecture**

**Single source of truth**: NextAuth session
**Fields**: `linkedIcPrincipals[]`, `activeIcPrincipal`, ~~optional `activeSince`~~ (if no TTL)
**No DB reads in callbacks**: Mutate via APIs then `unstable_update`
**Thin `useIICoAuth()` only**: No React Context for auth
**ICP Auth Client**: Crypto only (build actors), never UI truth

### **Implementation Ready**

The tech lead confirmed they can provide:

- Exact NextAuth callbacks
- Three route handlers ready to paste
- Once we confirm TTL on/off decision

---

## 🎯 **Final Architecture (Tech Lead Approved)**

### **Tech Lead's Definitive Answer: NO `activeIcPrincipal` in JWT/Session**

**❌ NO `activeIcPrincipal` tracking** - This is the wrong approach for fundamental architectural reasons.

**Key Insight**: "If 'activate' isn't part of your product responsibility, let's make II selection entirely the user's job (via the ICP Auth Client) and keep our app purely informative."

### **Why NOT to track "active II principal" in JWT/Session:**

1. **Wrong Source of Truth** - The active principal is chosen by `@dfinity/auth-client` at call time
2. **Guaranteed Inconsistency** - User switches identity in one tab/device → JWT/session in another tab/device is stale
3. **Race Conditions You Can't Fix** - CSR vs SSR: server renders with "active=A" while browser calls canisters with "B"
4. **No Authorization Value** - "Active" in JWT/session is not a proof of possession
5. **Security Foot-gun** - Teams eventually start trusting the "active" field for access checks
6. **Extra Complexity, Zero Benefit** - You'd need TTLs, heartbeats, event listeners, conflict resolution, and migrations
7. **Privacy/Minimization** - Keep tokens minimal
8. **Clear Ownership Boundaries** - ICP Auth Client: selects/signs (runtime crypto), DB: durable links (accounts), JWT/session: display/cache of links only

### **Final Schema (No Active, No TTL)**

```typescript
// JWT Token (Server-side)
type JWT = {
  loginProvider?: "google" | "github" | "internet-identity";
  linkedIcPrincipals?: string[]; // durable, for display only
  // NO activeIcPrincipal
  // NO activeSince
  // NO TTL fields
};

// Session User (Client-side)
type SessionUser = {
  loginProvider?: string;
  linkedIcPrincipals: string[];
  // NO icpPrincipal
  // NO icpPrincipalAssertedAt
  // NO TTL fields
};
```

### **What to Store Instead:**

- **JWT/Session**: Only `linkedIcPrincipals: string[]` (mirror of DB) + base login provider/roles
- **At call time**: Read actual principal from `authClient.getIdentity()` and send signed proof
- **Server-side validation**: Optionally enforce "principal ∈ linked set" by reading DB
- **If you need "active" concept**: Keep it purely local (`localStorage`/in-memory) as UX hint only
- **Never use for authorization**: Always verify with fresh signature
- **Never persist in JWT/session**: Keep it local only

### **How to Handle "Active" Principal:**

#### **At Call Time (Correct Approach):**

```typescript
// Read actual principal from auth-client at call time
const authClient = await getAuthClient();
const identity = authClient.getIdentity();
const principal = identity.getPrincipal().toText();

// Send signed proof for any protected server action
const actor = await backendActor(identity);
const result = await actor.someMethod();
```

#### **For UX Only (Optional):**

```typescript
// Keep it purely local as UX hint
const preferredPrincipal = localStorage.getItem("preferredPrincipal");
// Never use for authorization
// Never persist in JWT/session
```

### **Concrete Failure Cases (Why This Approach Fails):**

1. **Tab A**: JWT/session says active=A. User switches auth-client to B in Tab B. Server/UI still show A; canister calls go out as B.

2. **Edge/SSR**: Renders using stale "active" and prefetches data for A, but user interacts as B.

3. **Security Bug**: Someone adds "require active == X" check on API. It passes without fresh signature → confused-deputy bug.

### **Simplified NextAuth Callbacks**

```typescript
// auth.ts - Lean callbacks
callbacks: {
  async jwt({ token, trigger, account, session }) {
    if (trigger === 'signIn' && account) {
      token.loginProvider = account.provider;

      // On any sign-in, (re)load linked principals once from DB
      token.linkedIcPrincipals = await getLinkedPrincipalsFromDB(token.sub);
    }

    if (trigger === 'update' && session?.linkedIcPrincipals) {
      token.linkedIcPrincipals = session.linkedIcPrincipals; // only via link/unlink APIs
    }

    return token;
  },

  async session({ session, token }) {
    session.user.loginProvider = token.loginProvider;
    session.user.linkedIcPrincipals = token.linkedIcPrincipals ?? [];
    return session;
  },
}
```

### **Simplified Client Hook**

```typescript
// useIILinks.ts - Thin wrapper over useSession
export function useIILinks() {
  const { data: session, update, status } = useSession();
  const linked = session?.user.linkedIcPrincipals ?? [];

  return {
    status,
    linkedIcPrincipals: linked,
    setLinked: (arr: string[]) => update({ linkedIcPrincipals: arr }),
  };
}
```

### **Boundary Policy (Clear Ownership)**

- **Web2 routes**: Trust NextAuth session only
- **ICP/canister calls**: Always take principal from ICP Auth Client's current identity at call time
- **No server-side "active" state, no TTL**
- **If current principal not in `linkedIcPrincipals`**: Either block and ask user to link, or allow with warning

### **Simplified APIs (No `/activate`)**

```typescript
// POST /api/auth/ii/link   { principal, proof } -> verify, upsert DB, read-all, unstable_update({ linkedIcPrincipals })
// POST /api/auth/ii/unlink { principal }        -> delete DB, read-all, unstable_update({ linkedIcPrincipals })
// GET  /api/auth/ii/linked                        -> read DB; client may call setLinked() to refresh UI cache
```

### **UI Design**

- **"Linked Internet Identities"**: List principals, add "Link new", and "Unlink"
- **No "Activate" button**: No session-held selection
- **Optional**: Store purely-local "preferred principal" in `localStorage` for UX only

### **Canister Usage (Always Stateless)**

```typescript
const authClient = await getAuthClient();
const identity = authClient.getIdentity();            // user-chosen, outside our control
const principal = identity.getPrincipal().toText();   // compare to linked if policy requires
const actor = await backendActor(identity);
const res = await actor.doThing(...);
```

### **Why This Works Better**

- ✅ **Removes cross-store consistency problems**: NextAuth stores only links; ICP Auth Client controls live identity
- ✅ **No "activate" state**: No race conditions or expiry logic
- ✅ **Clear ownership**: User selects identity; app only verifies/links and enforces policy
- ✅ **Simpler implementation**: No TTL, no active state, no complex state management
- ✅ **Better UX**: User has full control over which II to use

## 📚 **Related Documents**

### **Related Documents**

### **Deep Analysis Documents**

- **[JWT vs Session Architecture Analysis](jwt-vs-session-architecture-analysis.md)** - Comprehensive analysis of JWT vs Session responsibilities and tech lead's definitive answer
- **[Multiple II Authentication Buttons Analysis](multiple-ii-authentication-buttons-analysis.md)** - Analysis of multiple authentication buttons and their inconsistencies
- **[Why Separate Sign-II-Only Page Created](why-separate-sign-ii-only-page-created.md)** - Explanation of separate II authentication page rationale

### **Implementation Documents**

- **[Internet Identity State Management To-Do](internet-identity-state-management-to-do.md)** - Complete implementation task list with 87 specific tasks organized into 7 phases
- **[Internet Identity State Management Architecture Analysis](internet-identity-state-management-architecture-analysis.md)** - This document (comprehensive architecture analysis)

### **Key Insights from Related Analysis**

#### **From JWT vs Session Analysis:**

- ✅ **NO `activeIcPrincipal` in JWT/Session** - Tech lead's definitive answer
- ✅ **8 detailed reasons** why this approach fails
- ✅ **Concrete failure cases** showing the problems
- ✅ **Clear ownership boundaries** for each component

#### **From To-Do List:**

- ✅ **87 specific tasks** organized into 7 phases
- ✅ **Phase 1**: Schema Migration (16 tasks)
- ✅ **Phase 2**: Hook Migration (12 tasks)
- ✅ **Phase 3**: API Implementation (16 tasks)
- ✅ **Phase 4**: Component Updates (18 tasks)
- ✅ **Phase 5**: Database Migration (8 tasks)
- ✅ **Phase 6**: Testing & Validation (20 tasks)
- ✅ **Phase 7**: Documentation & Cleanup (12 tasks)

---

**Status**: ✅ **FINAL ARCHITECTURE APPROVED** - Simplified approach with no active state, no TTL, clear ownership boundaries. Ready for implementation with tech lead's final code.

**Next Steps**: See [Internet Identity State Management To-Do](internet-identity-state-management-to-do.md) for complete implementation plan.
