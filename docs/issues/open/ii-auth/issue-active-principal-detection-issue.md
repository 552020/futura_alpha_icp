# Active Principal Detection Issue

## 📋 **Issue Summary**

**Status**: 🔴 **CRITICAL** - How to detect if user is currently signed in with Internet Identity (not just linked)

**Problem**: The `user-button-client-with-ii` component needs to show the **active principal** only when a user is **currently signed in** with Internet Identity, not just when they have linked accounts.

## 🎯 **Current State Analysis**

### **What We Have Now (BROKEN)**

```typescript
// Current logic in user-button-client-with-ii.tsx
const { isCoAuthActive, activeIcPrincipal, statusMessage, statusClass } = useIICoAuth();

// Only show Principal when II co-auth is active
const principal = isCoAuthActive ? activeIcPrincipal : undefined;
```

**Problem**: `isCoAuthActive` and `activeIcPrincipal` come from **NextAuth session state**, which is **stale** and doesn't reflect actual II auth status.

### **What We Need (SOLUTION)**

```typescript
// New logic needed
const { principal, isAuthenticated } = useICPIdentity();

// Show principal only when actually signed in with II
const showPrincipal = isAuthenticated && principal;
```

**Solution**: `useICPIdentity()` calls `authClient.isAuthenticated()` and `authClient.getIdentity()` to get **real-time** II auth status.

## 🔍 **Technical Analysis**

### **Current Detection Method (BROKEN)**

- **`isCoAuthActive`**: Based on TTL/session state in NextAuth
- **`activeIcPrincipal`**: Stored in NextAuth session
- **Problem**: This is **stale state** - doesn't reflect actual II auth status

### **Required Detection Method (SOLUTION)**

- **`authClient.isAuthenticated()`**: Check actual II auth client status
- **`authClient.getIdentity().getPrincipal()`**: Get current active principal
- **Solution**: This provides **real-time state** from `@dfinity/auth-client`

## 🚨 **Critical UX Issue**

### **Current Behavior (BROKEN)**

1. User signs in with Google → `isCoAuthActive = false` → No principal shown ✅
2. User links II account → `isCoAuthActive = false` → No principal shown ✅
3. User activates II co-auth → `isCoAuthActive = true` → Principal shown ✅
4. **User signs out of II in another tab** → `isCoAuthActive = true` → **Principal still shown** ❌
5. **User's II session expires** → `isCoAuthActive = true` → **Principal still shown** ❌

### **After Migration (SOLUTION)**

1. User signs in with Google → No principal shown ✅
2. User links II account → No principal shown ✅
3. User activates II co-auth → Principal shown ✅
4. **User signs out of II in another tab** → **Principal disappears** ✅
5. **User's II session expires** → **Principal disappears** ✅

**Result**: The migration **SOLVES** the problem by using real-time II auth status instead of stale NextAuth state.

## ✅ **Does the Migration Solve the Problem?**

### **YES - Here's Why:**

| **Scenario**           | **Current (BROKEN)**     | **After Migration (FIXED)** |
| ---------------------- | ------------------------ | --------------------------- |
| **Cross-tab sign out** | Principal still shows ❌ | Principal disappears ✅     |
| **Session expiry**     | Principal still shows ❌ | Principal disappears ✅     |
| **Identity switch**    | Old principal shows ❌   | New principal shows ✅      |
| **Network issues**     | Stale state ❌           | Real-time state ✅          |

### **Key Difference:**

- **Current**: Uses `isCoAuthActive` from NextAuth session (stale)
- **Migration**: Uses `authClient.isAuthenticated()` (real-time)

## 💡 **Production-Ready Solution Architecture**

### **1. Auth-client singleton + events**

```typescript
// src/ic/ii.ts
import { AuthClient } from "@dfinity/auth-client";

let clientPromise: Promise<AuthClient> | null = null;
export async function getAuthClient(): Promise<AuthClient> {
  if (!clientPromise) clientPromise = AuthClient.create();
  return clientPromise;
}

// Broadcast identity changes across tabs
const CHANNEL = "icp-auth";
export function notifyIdentityChanged() {
  try {
    new BroadcastChannel(CHANNEL).postMessage({ type: "identity-changed" });
  } catch {}
}

// Wrap login/logout so we always broadcast
export async function loginWithII(opts?: Parameters<AuthClient["login"]>[0]) {
  const c = await getAuthClient();
  await c.login(opts);
  notifyIdentityChanged();
}
export async function logoutII() {
  const c = await getAuthClient();
  await c.logout();
  notifyIdentityChanged();
}
```

### **2. Ephemeral identity hook (truth for avatar)**

```typescript
// src/hooks/use-icp-identity.ts
"use client";
import { useEffect, useState, useCallback } from "react";
import { getAuthClient } from "@/ic/ii";

const ANON = "2vxsx-fae";
const CHANNEL = "icp-auth";

export function useICPIdentity() {
  const [principal, setPrincipal] = useState<string | null>(null);
  const [isAuthenticated, setIsAuth] = useState(false);
  const [isLoading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const c = await getAuthClient();
      const authed = await c.isAuthenticated();
      if (!authed) {
        setPrincipal(null);
        setIsAuth(false);
      } else {
        const p = c.getIdentity().getPrincipal().toText();
        setPrincipal(p === ANON ? null : p);
        setIsAuth(p !== ANON);
      }
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    // initial
    refresh();
    // focus/visibility
    const onFocus = () => refresh();
    const onVis = () => document.visibilityState === "visible" && refresh();
    window.addEventListener("focus", onFocus);
    document.addEventListener("visibilitychange", onVis);
    // cross-tab via BroadcastChannel
    let bc: BroadcastChannel | null = null;
    try {
      bc = new BroadcastChannel(CHANNEL);
      bc.onmessage = (e) => e.data?.type === "identity-changed" && refresh();
    } catch {
      // Fallback: storage ping
      const key = "__icp_identity_ping__";
      const onStorage = (ev: StorageEvent) => ev.key === key && refresh();
      window.addEventListener("storage", onStorage);
      return () => window.removeEventListener("storage", onStorage);
    }
    return () => {
      window.removeEventListener("focus", onFocus);
      document.removeEventListener("visibilitychange", onVis);
      bc?.close();
    };
  }, [refresh]);

  return { principal, isAuthenticated, isLoading, refresh };
}
```

### **3. Avatar usage (show only when really signed in)**

```tsx
// src/components/auth/user-button-client-with-ii.tsx
"use client";
import { useICPIdentity } from "@/hooks/use-icp-identity";
import { useIILinks } from "@/hooks/use-ii-links"; // your simplified links hook

function shorten(p?: string | null) {
  return p ? `${p.slice(0, 5)}…${p.slice(-5)}` : "";
}

export function UserButtonClientWithII() {
  const { principal, isAuthenticated, isLoading } = useICPIdentity();
  const { linkedIcPrincipals } = useIILinks();

  const showPrincipal = isAuthenticated && !!principal;
  const isLinked = showPrincipal && linkedIcPrincipals.includes(principal!);

  return (
    <div className="flex items-center gap-2">
      {/* …your normal user button … */}
      {!isLoading && showPrincipal && (
        <div className="px-2 py-1 rounded bg-gray-100 text-xs">
          {shorten(principal)}{" "}
          <span className={isLinked ? "text-green-600" : "text-amber-600"}>{isLinked ? "linked" : "unlinked"}</span>
        </div>
      )}
    </div>
  );
}
```

## 🎯 **Where This Fixes Your CRITICAL Issues**

- **Cross-tab sign-out / identity switch** → BroadcastChannel (or storage fallback) triggers `refresh()` → avatar updates instantly.
- **II session expiry** → `refresh()` on focus/visibility removes the principal.
- **No stale NextAuth state involved**; the display is real-time from auth-client.

## 🚀 **Performance & Sync Answers**

- **Cost**: `AuthClient.create()` is cached; `isAuthenticated()` + `getIdentity()` are lightweight. We call on mount, tab focus, visibility change, and event pings—not every render.
- **Cross-tab**: BroadcastChannel covers modern browsers; storage event fallback covers the rest.
- **Loading**: use `isLoading` to avoid flicker. Show nothing (or a subtle spinner) until the first `refresh()` completes.

## 🔒 **Policy Line (Authorization)**

- **Keep avatar purely decorative/informative**. For protected server actions, require a fresh II proof regardless of what the avatar shows.

## 🔄 **State Management Strategy**

### **Ephemeral State (Runtime Only)**

- **Source**: `@dfinity/auth-client`
- **Hook**: `useICPIdentity()`
- **Purpose**: Show current active principal
- **Lifecycle**: Updates when II auth state changes

### **Persistent State (Database)**

- **Source**: NextAuth session
- **Hook**: `useIILinks()`
- **Purpose**: Show linked principals count
- **Lifecycle**: Updates when accounts are linked/unlinked

### **Combined Display Logic**

```typescript
const { principal, isAuthenticated } = useICPIdentity();
const { linkedIcPrincipals } = useIILinks();

// Show active principal if signed in with II
const showActivePrincipal = isAuthenticated && principal;

// Show linked count in tooltip or secondary info
const linkedCount = linkedIcPrincipals.length;
```

## 🎯 **Implementation Plan**

### **Phase 1: Create Hook**

1. **Create `useICPIdentity()` hook** - Detect actual II auth status
2. **Test hook** - Verify it detects auth state changes

### **Phase 2: Update Component**

3. **Update `user-button-client-with-ii.tsx`** - Use new hook
4. **Test component** - Verify principal shows/hides correctly

### **Phase 3: Clean Up**

5. **Remove old logic** - Remove `isCoAuthActive` usage
6. **Update other components** - Apply same pattern elsewhere

## 🚨 **Critical Questions**

### **1. Performance Impact**

- **Question**: Is `authClient.isAuthenticated()` expensive to call frequently?
- **Answer**: Need to test - may need caching/debouncing

### **2. State Synchronization**

- **Question**: How to handle cross-tab auth state changes?
- **Answer**: May need `storage` event listeners or polling

### **3. Loading States**

- **Question**: How to handle loading states during auth checks?
- **Answer**: Need proper loading indicators

## 📊 **Testing Scenarios**

### **Scenario 1: Fresh Login**

1. User signs in with Google → No principal shown ✅
2. User links II account → No principal shown ✅
3. User activates II → Principal appears ✅

### **Scenario 2: Cross-Tab Sign Out**

1. User has active II in Tab A → Principal shown ✅
2. User signs out of II in Tab B → Principal disappears in Tab A ✅

### **Scenario 3: Session Expiry**

1. User has active II → Principal shown ✅
2. II session expires → Principal disappears ✅

### **Scenario 4: Network Issues**

1. User has active II → Principal shown ✅
2. Network disconnects → Principal shown (cached) ✅
3. Network reconnects → Principal updates if changed ✅

## 🎯 **Next Steps**

1. **Create `useICPIdentity()` hook** - Implement ephemeral identity detection
2. **Test hook thoroughly** - Verify all scenarios work
3. **Update user button** - Use new hook instead of old logic
4. **Test cross-tab behavior** - Ensure state syncs correctly
5. **Performance optimization** - Add caching if needed

---

**Priority**: 🔴 **CRITICAL** - This is a fundamental UX issue that affects user understanding of their authentication state.

**Estimated Effort**: 1-2 days for implementation and testing.

**Dependencies**: Need to understand `@dfinity/auth-client` behavior and performance characteristics.
