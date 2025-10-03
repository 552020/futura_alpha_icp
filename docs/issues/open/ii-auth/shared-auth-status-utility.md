# Shared Authentication Status Utility

## 📋 **Issue Summary**

**Status**: 🔄 **PENDING** - Create shared utility for checking user authentication status

**Goal**: Extract the authentication status checking logic into a reusable utility function.

## 🎯 **Current Problem**

The logic for checking which service a user is logged in with is currently duplicated in components:

```typescript
// Currently in ii-coauth-controls.tsx
const appLoginProvider = session?.user?.loginProvider;
const isSignedInWithIIInApp = appLoginProvider === "internet-identity";
const isSignedInWithGoogleInApp = appLoginProvider === "google";
```

This logic will be needed in multiple components and should be centralized.

## 📚 **Proposed Solution**

### **Create Shared Utility Function**

**Location**: `src/lib/utils/auth-status.ts`

```typescript
import { Session } from "next-auth";

export interface AuthStatus {
  isSignedIn: boolean;
  loginProvider: string | null;
  isSignedInWithGoogle: boolean;
  isSignedInWithII: boolean;
  isSignedInWithEmail: boolean;
  providerDisplayName: string;
}

export function getAuthStatus(session: Session | null): AuthStatus {
  const isSignedIn = !!session?.user;
  const loginProvider = session?.user?.loginProvider || null;

  return {
    isSignedIn,
    loginProvider,
    isSignedInWithGoogle: loginProvider === "google",
    isSignedInWithII: loginProvider === "internet-identity",
    isSignedInWithEmail: loginProvider === "email",
    providerDisplayName: getProviderDisplayName(loginProvider),
  };
}

function getProviderDisplayName(provider: string | null): string {
  switch (provider) {
    case "google":
      return "Google";
    case "internet-identity":
      return "Internet Identity";
    case "email":
      return "Email";
    default:
      return "Not signed in";
  }
}
```

### **Usage in Components**

**Before** (duplicated logic):

```typescript
const appLoginProvider = session?.user?.loginProvider;
const isSignedInWithIIInApp = appLoginProvider === "internet-identity";
const isSignedInWithGoogleInApp = appLoginProvider === "google";
```

**After** (shared utility):

```typescript
import { getAuthStatus } from "@/lib/utils/auth-status";

const authStatus = getAuthStatus(session);
// authStatus.isSignedIn
// authStatus.isSignedInWithGoogle
// authStatus.isSignedInWithII
// authStatus.providerDisplayName
```

## 🎯 **Implementation Tasks**

### **1. Create Utility File**

- [ ] Create `src/lib/utils/auth-status.ts`
- [ ] Implement `getAuthStatus()` function
- [ ] Add proper TypeScript types
- [ ] Add JSDoc documentation

### **2. Update Components**

- [ ] Update `ii-coauth-controls.tsx` to use shared utility
- [ ] Update `linked-accounts.tsx` to use shared utility
- [ ] Update `internet-identity-management.tsx` to use shared utility
- [ ] Update any other components that check auth status

### **3. Add Tests**

- [ ] Create unit tests for `getAuthStatus()`
- [ ] Test all provider types
- [ ] Test edge cases (null session, unknown provider)

## 🎯 **Benefits**

✅ **No duplication**: Single source of truth for auth status logic  
✅ **Consistent**: Same logic everywhere  
✅ **Maintainable**: Easy to update provider logic  
✅ **Type-safe**: Proper TypeScript support  
✅ **Testable**: Easy to unit test

## 📊 **Estimated Effort**

- **Create utility**: 30 minutes
- **Update components**: 1 hour
- **Add tests**: 30 minutes
- **Total**: 2 hours

---

**Priority**: 🟡 **MEDIUM** - Simple refactoring for better code organization.

**Dependencies**: None - can be done immediately.
