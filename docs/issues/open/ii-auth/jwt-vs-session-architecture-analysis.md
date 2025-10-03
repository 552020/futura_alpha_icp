# Complete State Management Architecture Analysis

## 📋 **Issue Summary**

**Status**: 🔍 **ANALYSIS** - Comprehensive analysis of JWT vs Session architecture for ALL state management

**Question**: What should be the ultimate source of truth for state management? How should we organize JWT token vs Session responsibilities? What information do we have/need from ICP libraries vs our own state?

## 🎯 **Core Questions**

For the complete state management architecture, we need to understand:

1. **JWT Token responsibilities** - What should live in the token?
2. **Session responsibilities** - What should live in the session?
3. **ICP Library state** - What do we get from `@dfinity/auth-client`?
4. **Database state** - What's the source of truth in the database?
5. **State synchronization** - How do all these sources stay in sync?

## 🎯 **Tech Lead's Definitive Answer**

**❌ NO `activeIcPrincipal` in JWT/Session** - This is the wrong approach for fundamental architectural reasons.

### **Why NOT to track "active II principal" in JWT/Session:**

#### **1. Wrong Source of Truth**

- The active principal is chosen by `@dfinity/auth-client` at call time
- Your app doesn't own that state
- Mirroring it elsewhere will drift

#### **2. Guaranteed Inconsistency**

- User switches identity in one tab/device → JWT/session in another tab/device is stale
- JWT is cached by design; "active" there will be wrong until next refresh

#### **3. Race Conditions You Can't Fix**

- CSR vs SSR: server renders with "active=A" while browser calls canisters with "B"
- Multi-tab: each tab would need cross-tab sync; still lags behind auth-client

#### **4. No Authorization Value**

- "Active" in JWT/session is not a proof of possession
- For protected ops you must verify a fresh signature from current identity anyway
- So the field can't replace verification

#### **5. Security Foot-gun**

- Teams eventually start trusting the "active" field for access checks
- That's susceptible to manipulation (e.g., client-side `session.update`)
- Unless you build extra server guards. Easier: don't have the field

#### **6. Extra Complexity, Zero Benefit**

- You'd need TTLs, heartbeats, event listeners, conflict resolution, and migrations
- All to approximate what the auth-client already knows precisely

#### **7. Privacy/Minimization**

- Keep tokens minimal
- Don't stuff transient identifiers in a long-lived cookie if you don't need them

#### **8. Clear Ownership Boundaries**

- **ICP Auth Client**: selects/signs (runtime crypto)
- **DB**: durable links (accounts)
- **JWT/session**: display/cache of links only
- **Server**: verifies proofs when it matters

### **What to Store Instead:**

```typescript
// JWT Token (Server-side)
interface JWT {
  loginProvider?: string;
  linkedIcPrincipals?: string[]; // Mirror of DB only
  role?: string;
  businessUserId?: string;
  // NO activeIcPrincipal
  // NO activeSince
  // NO TTL fields
}

// Session (Client-side)
interface SessionUser {
  loginProvider?: string;
  linkedIcPrincipals: string[]; // Mirror of DB only
  // NO activeIcPrincipal
  // NO activeSince
  // NO TTL fields
}
```

### **Key Principles from Tech Lead:**

1. **JWT/Session**: Only `linkedIcPrincipals: string[]` (mirror of DB) + base login provider/roles
2. **At call time**: Read actual principal from `authClient.getIdentity()` and send signed proof
3. **Server-side validation**: Optionally enforce "principal ∈ linked set" by reading DB
4. **If you need "active" concept**: Keep it purely local (`localStorage`/in-memory) as UX hint only
5. **Never use for authorization**: Always verify with fresh signature
6. **Never persist in JWT/session**: Keep it local only

### **How to Handle "Active" Concept:**

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

### **If You Ever Truly Need an "Active" Concept:**

#### **Keep it Purely Local (UX Hint Only):**

```typescript
// Optional: UX hint for pre-selecting in modal
const preferredPrincipal = localStorage.getItem("preferredPrincipal");

// Never use for authorization
// Never persist in JWT/session
// Never trust for security decisions
```

#### **Key Rules:**

- ✅ **Local only** - `localStorage` or in-memory
- ✅ **UX hint only** - Pre-select in modal, remember user preference
- ❌ **Never for authorization** - Always verify with fresh signature
- ❌ **Never persist in JWT/session** - Keep it local only
- ❌ **Never trust for security** - Always verify at call time

### **Benefits of This Approach:**

1. **One owner per concern** - Each component has single responsibility
2. **Eliminates drift/races** - No synchronization issues
3. **Forces proof-of-possession** - Only thing that matters for security
4. **Simplified architecture** - No TTL, no active state, no complexity
5. **Clear boundaries** - Each component knows its role

## 📊 **Complete State Inventory**

### **Current State Sources (What We Have)**

#### **1. Database State (Durable)**

```typescript
// accounts table
interface Account {
  userId: string;
  provider: "internet-identity" | "google" | "github";
  providerAccountId: string; // II principal or OAuth ID
  type: "oauth";
}

// users table
interface User {
  id: string;
  email: string;
  name: string;
  role: string;
}

// iiNonces table
interface IINonce {
  id: string;
  nonce: string;
  expiresAt: Date;
  usedAt?: Date;
}
```

#### **2. ICP Auth Client State (Browser)**

```typescript
// From @dfinity/auth-client
interface AuthClientState {
  isAuthenticated: boolean;
  identity: Identity; // Current identity object
  principal: Principal; // Current principal
  // No persistence control - managed by ICP library
}
```

#### **3. Current JWT Token State**

```typescript
interface JWT {
  role?: string;
  businessUserId?: string;
  loginProvider?: string;
  activeIcPrincipal?: string;
  activeIcPrincipalAssertedAt?: number;
  linkedIcPrincipal?: string;
}
```

#### **4. Current Session State**

```typescript
interface SessionUser {
  id: string;
  businessUserId?: string;
  icpPrincipal?: string;
  linkedIcPrincipal?: string;
  loginProvider?: string;
  icpPrincipalAssertedAt?: number;
}
```

#### **5. Local React State (Should be eliminated)**

```typescript
// ICP Page component
const [isAuthenticated, setIsAuthenticated] = useState(false);
const [principalId, setPrincipalId] = useState<string>("");
const [greeting, setGreeting] = useState("");
const [isRehydrating, setIsRehydrating] = useState(true);
```

### **Required State (What We Need)**

#### **1. User Authentication State**

- ✅ **Who is the user** - User ID, email, name, role
- ✅ **How did they authenticate** - Google, GitHub, II, credentials
- ✅ **Business user linkage** - Link to business user ID

#### **2. Internet Identity State**

- ✅ **Linked principals** - Which II principals are linked to this user
- ✅ **Active principal** - Which II principal is currently active (if any)
- ✅ **Authentication status** - Is user authenticated with II
- ✅ **Session timing** - When was II last authenticated (for TTL)

#### **3. UI State**

- ✅ **Loading states** - Is authentication in progress
- ✅ **Error states** - Authentication errors, network errors
- ✅ **Display state** - What to show in components

#### **4. ICP Operations State**

- ✅ **Canister calls** - Which identity to use for canister calls
- ✅ **Actor creation** - How to create authenticated actors
- ✅ **Principal validation** - Is current principal linked

## 🔍 **JWT Token vs Session: Purpose & Characteristics**

### **JWT Token (Server-side)**

**Purpose:**

- **Authentication state** - Who the user is
- **Authorization data** - What they can do
- **Persistent data** - Survives across requests
- **Server-side validation** - Can be trusted for security decisions

**Characteristics:**

- ✅ **Server-side only** - Not accessible to client JavaScript
- ✅ **Persistent** - Survives page reloads, browser restarts
- ✅ **Secure** - Encrypted, tamper-proof
- ✅ **Stateless** - No server-side storage needed
- ❌ **Not real-time** - Updates require token refresh
- ❌ **Limited size** - Should be kept small
- ❌ **Not directly accessible** - Requires server-side processing

### **Session (Client-side)**

**Purpose:**

- **UI state** - What to display to user
- **Current context** - What's happening now
- **User experience** - How the app should behave
- **Temporary state** - Current session information

**Characteristics:**

- ✅ **Client-accessible** - Available to React components
- ✅ **Real-time updates** - Can be updated immediately
- ✅ **UI-friendly** - Perfect for display logic
- ✅ **Flexible** - Can contain any data structure
- ❌ **Not persistent** - Lost on page reload (unless synced with token)
- ❌ **Client-side only** - Not available server-side
- ❌ **Not secure** - Can be modified by client

## 🏗️ **Architecture Options Analysis**

### **Option 1: JWT Token Only**

```typescript
// JWT Token
interface JWT {
  linkedIcPrincipals?: string[];
  activeIcPrincipal?: string; // Only in token
}

// Session
interface SessionUser {
  linkedIcPrincipals: string[];
  // No activeIcPrincipal in session
}
```

**Pros:**

- ✅ **Single source of truth** - No duplication
- ✅ **Persistent** - Survives page reloads
- ✅ **Server-side available** - Can be used for API validation
- ✅ **Secure** - Cannot be tampered with by client

**Cons:**

- ❌ **Not real-time** - Updates require token refresh
- ❌ **Server-side only** - Components can't access directly
- ❌ **Complex updates** - Need to update token to change active principal
- ❌ **Performance** - Token updates trigger full session refresh

**Use Case:** When active principal needs to be persistent and server-side validation is required.

### **Option 2: Session Only**

```typescript
// JWT Token
interface JWT {
  linkedIcPrincipals?: string[];
  // No activeIcPrincipal in token
}

// Session
interface SessionUser {
  linkedIcPrincipals: string[];
  activeIcPrincipal?: string; // Only in session
  activeSince?: number; // For TTL
}
```

**Pros:**

- ✅ **Real-time updates** - Immediate UI updates
- ✅ **Client-accessible** - Components can read directly
- ✅ **Simple updates** - Just call `session.update()`
- ✅ **UI-focused** - Perfect for display logic
- ✅ **Flexible** - Easy to add TTL, timestamps, etc.

**Cons:**

- ❌ **Not persistent** - Lost on page reload
- ❌ **Not server-side** - Cannot be used for API validation
- ❌ **Client-side only** - Not available for server-side logic

**Use Case:** When active principal is purely for UI display and doesn't need server-side validation.

### **Option 3: Both JWT and Session**

```typescript
// JWT Token
interface JWT {
  linkedIcPrincipals?: string[];
  activeIcPrincipal?: string; // In token
}

// Session
interface SessionUser {
  linkedIcPrincipals: string[];
  activeIcPrincipal?: string; // In session too
  activeSince?: number; // For TTL
}
```

**Pros:**

- ✅ **Best of both worlds** - Persistent + real-time
- ✅ **Server-side validation** - Can validate against token
- ✅ **Client-side display** - Components can access session
- ✅ **Redundant safety** - Two sources of truth

**Cons:**

- ❌ **Complexity** - Need to keep both in sync
- ❌ **Duplication** - Same data in two places
- ❌ **Sync issues** - Risk of inconsistency
- ❌ **More code** - Need to update both

**Use Case:** When you need both persistence and real-time updates.

## 🤔 **Our Specific Use Case Analysis**

### **What do we need `activeIcPrincipal` for?**

1. **UI Display** - Show which principal is currently active
2. **User Experience** - Indicate current state to user
3. **Policy Enforcement** - Check if current principal is linked
4. **Session Management** - Track current authentication state

### **Key Questions:**

1. **Do we need server-side validation of active principal?**

   - For API endpoints that require II authentication
   - For security-sensitive operations
   - For canister calls validation

2. **Do we need persistence across page reloads?**

   - User refreshes page - should active principal persist?
   - User closes browser and reopens - should active principal persist?
   - User switches devices - should active principal sync?

3. **Do we need real-time updates?**

   - User switches II identity - should UI update immediately?
   - User activates different principal - should display change instantly?

4. **Do we need TTL for active principal?**
   - Should active principal expire after some time?
   - Should we show warnings before expiry?
   - Should we auto-clear expired principals?

## 🎯 **Recommended Approach**

### **For Our Use Case: Session Only**

**Reasoning:**

1. **UI-focused** - We need it primarily for display
2. **Real-time updates** - User should see changes immediately
3. **No server-side validation** - ICP Auth Client handles the actual authentication
4. **Simple TTL** - Easy to implement in session
5. **No persistence needed** - User can re-activate II after page reload

### **Implementation:**

```typescript
// JWT Token (durable data only)
interface JWT {
  loginProvider?: string;
  linkedIcPrincipals?: string[]; // Durable - survives page reloads
}

// Session (current state only)
interface SessionUser {
  loginProvider?: string;
  linkedIcPrincipals: string[]; // Mirrored from token
  activeIcPrincipal?: string; // Current active principal
  activeSince?: number; // For TTL (6 hours)
}
```

### **Flow:**

1. **User activates II** → ICP Auth Client
2. **We detect active principal** → Update session only
3. **UI displays current state** → Read from session
4. **TTL check** → Client-side check in session
5. **Page reload** → Active principal lost, user can re-activate

## 🎯 **State Management Responsibilities**

### **JWT Token Responsibilities (Server-side)**

**Should contain:**

- ✅ **Durable authentication data** - User ID, role, business user ID
- ✅ **Base session provider** - How user originally authenticated
- ✅ **Linked principals** - Which II principals are linked (from DB)
- ✅ **Server-side validation data** - What APIs need to validate

**Should NOT contain:**

- ❌ **UI state** - Loading, error states
- ❌ **Temporary state** - Current active principal (if not persistent)
- ❌ **Client-specific data** - Browser-specific information

### **Session Responsibilities (Client-side)**

**Should contain:**

- ✅ **UI state** - What to display to user
- ✅ **Current context** - Active principal, loading states
- ✅ **Real-time data** - Current authentication status
- ✅ **Display data** - Linked principals for UI

**Should NOT contain:**

- ❌ **Server-side validation data** - Security-sensitive information
- ❌ **Durable data** - Should be in JWT token
- ❌ **Database data** - Should be fetched from DB

### **ICP Auth Client Responsibilities (Browser)**

**Should handle:**

- ✅ **Cryptographic operations** - Signing, verification
- ✅ **Identity management** - Creating, storing identities
- ✅ **Authentication flow** - II login/logout
- ✅ **Principal generation** - Creating principal objects

**Should NOT handle:**

- ❌ **State management** - Not our source of truth
- ❌ **UI state** - Not for display logic
- ❌ **Persistence** - Not for long-term storage

## ❓ **Critical Questions for Tech Lead**

### **1. Complete State Architecture**

**Question**: What should be the complete state management architecture?

- Which state should live in JWT token?
- Which state should live in session?
- How should ICP Auth Client state integrate?
- What's the synchronization strategy?

### **2. Active Principal Management**

**Question**: How should we handle the active principal?

- Should it be in JWT token, session, or both?
- Should it persist across page reloads?
- Should it be server-side validated?
- How should it sync with ICP Auth Client?

### **3. State Synchronization**

**Question**: How should different state sources stay in sync?

- Database ↔ JWT token synchronization
- JWT token ↔ Session synchronization
- ICP Auth Client ↔ Our state synchronization
- Update conflicts and resolution

### **4. Performance vs Complexity**

**Question**: What's the right balance between performance and complexity?

- Real-time updates vs persistence
- Server-side validation vs client-side state
- Single source of truth vs multiple sources
- Simple architecture vs feature completeness

### **5. Migration Strategy**

**Question**: How should we migrate from current state to new architecture?

- Existing user data migration
- Backward compatibility
- Gradual rollout strategy
- Rollback plan

### **6. Server-side Validation**

**Question**: Do we need server-side validation of the active principal?

- For API endpoints that require II authentication?
- For security-sensitive operations?
- For canister calls validation?

**If YES**: We need `activeIcPrincipal` in JWT token
**If NO**: Session only is sufficient

### **2. Persistence Requirements**

**Question**: Should the active principal persist across:

- Page reloads?
- Browser restarts?
- Device switches?

**If YES**: We need `activeIcPrincipal` in JWT token
**If NO**: Session only is sufficient

### **3. Real-time Updates**

**Question**: Do we need immediate UI updates when:

- User switches II identity?
- User activates different principal?
- User disconnects II?

**If YES**: We need `activeIcPrincipal` in session
**If NO**: JWT token only is sufficient

### **4. TTL Implementation**

**Question**: For the 6-hour TTL:

- Should it be enforced server-side or client-side?
- Should expired principals be auto-cleared?
- Should we show warnings before expiry?

**Server-side TTL**: Need `activeSince` in JWT token
**Client-side TTL**: Need `activeSince` in session only

### **5. Architecture Complexity**

**Question**: Are we willing to accept the complexity of:

- Keeping JWT and session in sync?
- Handling update conflicts?
- Managing two sources of truth?

**If YES**: Both JWT and session
**If NO**: Choose one (JWT or session)

## 🎯 **Final Architecture (Tech Lead Approved)**

### **What We Will Implement:**

```typescript
// JWT Token (Server-side)
interface JWT {
  loginProvider?: string;
  linkedIcPrincipals?: string[]; // Mirror of DB only
  role?: string;
  businessUserId?: string;
  // NO activeIcPrincipal
  // NO activeSince
  // NO TTL fields
}

// Session (Client-side)
interface SessionUser {
  loginProvider?: string;
  linkedIcPrincipals: string[]; // Mirror of DB only
  // NO activeIcPrincipal
  // NO activeSince
  // NO TTL fields
}
```

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

### **Key Principles:**

1. **ICP Auth Client owns active state** - We don't mirror it
2. **JWT/Session for linked principals only** - Display/cache of DB
3. **Fresh verification at call time** - Always verify with auth-client
4. **No TTL complexity** - Let auth-client handle session management
5. **Clear ownership boundaries** - Each component has single responsibility

## 📊 **Decision Matrix**

| Requirement                 | JWT Only   | Session Only | Both       |
| --------------------------- | ---------- | ------------ | ---------- |
| **UI Display**              | ❌ Complex | ✅ Simple    | ✅ Simple  |
| **Real-time Updates**       | ❌ Slow    | ✅ Fast      | ✅ Fast    |
| **Persistence**             | ✅ Yes     | ❌ No        | ✅ Yes     |
| **Server-side Access**      | ✅ Yes     | ❌ No        | ✅ Yes     |
| **Architecture Complexity** | ✅ Simple  | ✅ Simple    | ❌ Complex |
| **Performance**             | ❌ Slow    | ✅ Fast      | ❌ Medium  |

## 🎯 **Next Steps**

1. **✅ Tech lead's answer received** - Clear guidance on architecture
2. **🔄 Update JWT interface** - Remove `activeIcPrincipal` and TTL fields
3. **🔄 Update Session interface** - Remove `activeIcPrincipal` and TTL fields
4. **🔄 Update callbacks** - Remove active principal logic from JWT/session callbacks
5. **🔄 Update components** - Remove active principal display logic
6. **🔄 Update canister calls** - Use fresh auth-client identity at call time
7. **🔄 Test implementation** - Verify no active principal in JWT/session

---

**Priority**: 🟢 **READY TO IMPLEMENT** - Tech lead has provided definitive guidance.

**Estimated Effort**: 2-3 days to implement the simplified architecture.
