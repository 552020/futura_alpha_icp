# Linked Accounts Component II Authentication Sync Issue

## 📋 **Issue Summary**

**Status**: ✅ **RESOLVED** - Fixed ICP page authentication sync with NextAuth session

**Problem**: The `LinkedAccounts` component showed "not linked yet" even when users were authenticated with Internet Identity on the ICP page. This occurred because the ICP page used direct II authentication that didn't properly sync with the NextAuth session system.

### **What "Direct II Authentication" Means**

The ICP page uses a **direct authentication flow** that bypasses the NextAuth session system:

```typescript
// ICP page authentication (DIRECT)
async function handleLogin() {
  const { identity, principal } = await loginWithII(); // Direct II auth
  const backend = await backendActor(identity); // Direct canister access
  setPrincipalId(principal.toString()); // Local state only
  setIsAuthenticated(true); // Local state only
  // ❌ NO NextAuth session update
}
```

This creates **two separate authentication states**:

1. **ICP Page State**: Local React state (`isAuthenticated`, `principalId`)
2. **NextAuth Session State**: Global session state (`linkedIcPrincipal`, `activeIcPrincipal`)

The `LinkedAccounts` component only sees the **NextAuth session state**, not the ICP page's local state. So even though you're authenticated with II on the ICP page, the LinkedAccounts component doesn't know about it because the authentication never reached the NextAuth session system.

**Analogy**: It's like having two separate login systems - you're logged into System A (ICP page) but System B (LinkedAccounts) doesn't know about it because they don't communicate.

## 🔍 **Root Cause Analysis**

### **Two Different Authentication Flows**

The application has two separate authentication flows that weren't properly integrated:

1. **ICP Page Flow**: Direct Internet Identity authentication using `loginWithII()` and `backendActor()`
2. **LinkedAccounts Flow**: NextAuth session-based authentication expecting `linkedIcPrincipal` in the session

### **The Disconnect**

```typescript
// ❌ PROBLEM: ICP page authentication didn't update NextAuth session
async function handleLogin() {
  const { identity, principal } = await loginWithII();
  // Direct II authentication - no session update
  setPrincipalId(principal.toString());
  setIsAuthenticated(true);
  // Missing: Session sync for LinkedAccounts component
}
```

```typescript
// ❌ PROBLEM: LinkedAccounts component expected session data
export function LinkedAccounts() {
  const { hasLinkedII, linkedIcPrincipal } = useIICoAuth();
  // This checks for linkedIcPrincipal in NextAuth session
  // But ICP page didn't update the session
}
```

### **Session vs Token Architecture**

The NextAuth system uses a **JWT token-based architecture**:

- **Session Callback**: Reads from JWT token, not session updates
- **JWT Callback**: Handles token updates during sign-in and session updates
- **Database Linking**: Required for persistent account linking

```typescript
// auth.ts - Session callback reads from token
session({ session, token }) {
  if (token.linkedIcPrincipal) {
    session.user.linkedIcPrincipal = token.linkedIcPrincipal;
  }
  // Session updates don't directly affect this
}
```

## 🛠️ **Solution Implemented**

### **Proper Account Linking Flow**

Updated the ICP page to use the complete account linking flow:

```typescript
async function handleLogin() {
  const { identity, principal } = await loginWithII();

  // 1. Create nonce and register with canister
  const { fetchChallenge, registerWithNonce } = await import("@/lib/ii-client");
  const challenge = await fetchChallenge(window.location.href);
  await registerWithNonce(challenge.nonce, identity);

  // 2. Link II account to existing session via API
  const linkResponse = await fetch("/api/auth/link-ii", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ nonce: challenge.nonce }),
  });

  // 3. Activate II co-auth in session
  await update({
    activeIcPrincipal: principal.toString(),
    icpPrincipalAssertedAt: Date.now(),
  });
}
```

### **Database Integration**

The `/api/auth/link-ii` API route properly links the II account:

```typescript
// /api/auth/link-ii/route.ts
export async function POST(request: NextRequest) {
  // 1. Verify nonce with canister
  const nonceResult = await actor.verify_nonce(nonce);

  // 2. Check for principal conflicts
  const existingAccount = await db.query.accounts.findFirst({
    where: (a, { and, eq }) => and(eq(a.provider, "internet-identity"), eq(a.providerAccountId, principal)),
  });

  // 3. Upsert account link in database
  await db
    .insert(accounts)
    .values({
      userId: session.user.id,
      provider: "internet-identity",
      providerAccountId: principal,
      type: "oidc",
    })
    .onConflictDoUpdate({
      target: [accounts.provider, accounts.providerAccountId],
      set: { userId: session.user.id },
    });
}
```

### **JWT Token Integration**

The JWT callback now properly handles the linked principal:

```typescript
// auth.ts - JWT callback
async jwt({ token, account, user, trigger, session }) {
  // Fresh sign-in: fetch linkedIcPrincipal from database
  if (trigger === 'signIn' && account) {
    if (!token.linkedIcPrincipal) {
      const iiAccount = await db.query.accounts.findFirst({
        where: (a, { and, eq }) => and(
          eq(a.userId, uid),
          eq(a.provider, 'internet-identity')
        ),
      });
      if (iiAccount?.providerAccountId) {
        token.linkedIcPrincipal = iiAccount.providerAccountId;
      }
    }
  }

  // Session update: handle II co-auth activation
  if (trigger === 'update' && session) {
    if (session.activeIcPrincipal) {
      token.activeIcPrincipal = session.activeIcPrincipal;
      token.activeIcPrincipalAssertedAt = Date.now();
    }
  }
}
```

## 🎯 **Result**

### **Before Fix**

- ❌ LinkedAccounts showed "not linked yet" despite II authentication
- ❌ II Co-Auth controls didn't display proper status
- ❌ Session and authentication state were disconnected

### **After Fix**

- ✅ LinkedAccounts correctly shows II authentication status
- ✅ II Co-Auth controls display proper status and TTL information
- ✅ Seamless integration between direct II auth and session-based components
- ✅ Proper database persistence of account linking
- ✅ Consistent authentication state across the application

## 📁 **Files Modified**

### **Core Changes**

1. **`src/app/[lang]/user/icp/page.tsx`**
   - Added NextAuth session integration
   - Implemented proper account linking flow
   - Added error handling for linking failures
   - Updated sign out to clear session properly

### **Supporting Infrastructure**

2. **`/api/auth/link-ii/route.ts`** (existing)

   - Handles II account linking to existing sessions
   - Prevents principal conflicts
   - Updates database with account relationships

3. **`src/lib/ii-client.ts`** (existing)

   - Provides nonce creation and registration utilities
   - Handles canister communication for account linking

4. **`auth.ts`** (existing)
   - JWT callback handles token updates
   - Session callback exposes linked principal to components

## 🔧 **Technical Details**

### **Authentication Flow Sequence**

1. **User signs in with Google** → NextAuth session created
2. **User navigates to ICP page** → Direct II authentication
3. **II authentication triggers linking** → Nonce created and registered
4. **Account linked via API** → Database updated with II principal
5. **Session activated** → II co-auth enabled in session
6. **LinkedAccounts component** → Now shows proper II status

### **Error Handling**

- **Principal Conflict**: Handled gracefully with user-friendly error messages
- **Linking Failure**: User remains authenticated with II, but gets warning toast
- **Session Update Failure**: Non-blocking, logged for debugging

### **Backward Compatibility**

- Existing II-only signin flow continues to work
- No breaking changes to existing authentication patterns
- Graceful degradation if linking fails

## 🧪 **Testing Scenarios**

### **Test Case 1: Google → II Linking**

1. Sign in with Google
2. Navigate to ICP page
3. Authenticate with Internet Identity
4. **Expected**: LinkedAccounts shows II as linked and active

### **Test Case 2: Direct II Signin**

1. Use II-only signin page
2. **Expected**: LinkedAccounts shows II as linked and active

### **Test Case 3: Principal Conflict**

1. Try to link II that's already linked to another account
2. **Expected**: Clear error message about principal conflict

### **Test Case 4: Sign Out**

1. Sign out from ICP page
2. **Expected**: LinkedAccounts shows "not linked yet"

## 📚 **Related Documentation**

- **II Integration Flow**: `docs/issues/open/frontend-icp-upload.md`
- **Account Linking API**: `/api/auth/link-ii/route.ts`
- **Session Management**: `src/hooks/use-ii-coauth.ts`
- **LinkedAccounts Component**: `src/components/user/linked-accounts.tsx`

## 🎉 **Resolution Status**

✅ **COMPLETE** - The LinkedAccounts component now properly syncs with ICP page authentication, providing a seamless user experience across both authentication flows.
