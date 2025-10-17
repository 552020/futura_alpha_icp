# Internet Identity Sign-In Flow Analysis

## Overview

This document traces the complete Internet Identity (II) sign-in flow starting from the ICP page (`/user/icp`) through all the components and API calls involved.

## Flow Diagram

```
ICP Page (/user/icp)
    ↓
InternetIdentityManagement Component
    ↓ (if not authenticated)
"Connect Internet Identity" Button
    ↓ (onClick: handleSignInII)
Redirect to /sign-ii-only page
    ↓
Sign-II-Only Page
    ↓ (onClick: handleInternetIdentity)
Internet Identity Authentication Flow
    ↓
Backend Integration & Session Management
```

## Detailed Flow Analysis

### 1. Entry Point: ICP Page

**File:** `src/nextjs/src/app/[lang]/user/icp/page.tsx`

- **Component:** `ICPPage`
- **Authentication Check:** Uses `useAuthGuard()` hook
- **Renders:** `InternetIdentityManagement` component (line 38)

### 2. Internet Identity Management Component

**File:** `src/nextjs/src/components/user/internet-identity-management.tsx`

- **Component:** `InternetIdentityManagement`
- **Key Function:** `handleSignInII()` (lines 47-73)
- **Button Location:** Lines 196-199
- **Button Text:** "Connect Internet Identity"
- **Button Icon:** `<User className="h-4 w-4 mr-2" />`

#### Button Click Handler Logic:

```typescript
const handleSignInII = () => {
  // Extract current URL and locale
  const currentUrl = window.location.href;
  const locale = window.location.pathname.split("/")[1];

  // Construct sign-in URL with callback
  const signinUrl = `/${locale}/sign-ii-only?callbackUrl=${encodeURIComponent(currentUrl)}`;

  // Redirect to sign-in page
  router.push(signinUrl);
};
```

### 3. Sign-II-Only Page

**File:** `src/nextjs/src/app/[lang]/sign-ii-only/page.tsx`

- **Component:** `SignIIOnlyPage` (wrapper with Suspense)
- **Main Component:** `SignIIOnlyContent`
- **Button Location:** Lines 159-168
- **Button Text:** "Sign in with Internet Identity"
- **Button Handler:** `handleInternetIdentity()` (lines 42-116)

#### Sign-In Button Handler Logic:

```typescript
async function handleInternetIdentity() {
  // 1. Login with Internet Identity
  const { loginWithII } = await import('@/ic/ii');
  const { identity } = await loginWithII();

  // 2. Auto-create capsule (non-blocking)
  const { ensureSelfCapsuleWithIdentity } = await import('@/services/capsule');
  ensureSelfCapsuleWithIdentity(identity);

  // 3. Fetch challenge and register
  const { fetchChallenge, registerWithNonce } = await import('@/lib/ii-client');
  const challenge = await fetchChallenge(safeCallbackUrl);
  await registerWithNonce(challenge.nonce, identity);

  // 4. Handle session linking or standalone sign-in
  if (hasActiveSession) {
    // Link II to existing session
    await fetch('/api/auth/link-ii', { ... });
    await update({ activeIcPrincipal: principal, linkedIcPrincipals });
  } else {
    // Standalone II sign-in
    await signIn('ii', { principal, nonceId, nonce, redirect: true });
  }
}
```

### 4. Internet Identity Authentication

**File:** `src/nextjs/src/ic/ii.ts`

- **Function:** `loginWithII()` (lines 23-46)
- **Uses:** `@dfinity/auth-client` AuthClient
- **Process:**
  1. Creates AuthClient instance
  2. Calls `authClient.login()` with identity provider
  3. Returns identity and principal

### 5. Backend Integration

**File:** `src/nextjs/src/lib/ii-client.ts`

#### Challenge Fetching:

- **Function:** `fetchChallenge()` (lines 18-38)
- **API Endpoint:** `/api/ii/challenge`
- **Purpose:** Get nonce for authentication

#### Registration:

- **Function:** `registerWithNonce()` (lines 44-62)
- **Backend Call:** `actor.register_with_nonce(nonce)`
- **Purpose:** Register user and prove nonce

### 6. Session Management

#### For Existing Sessions (Linking):

- **API Route:** `/api/auth/link-ii`
- **Process:** Verify nonce server-side and upsert account
- **Session Update:** Add `activeIcPrincipal` and `linkedIcPrincipals`

#### For New Sessions:

- **NextAuth Provider:** `'ii'`
- **Parameters:** `principal`, `nonceId`, `nonce`
- **Redirect:** To callback URL

## Component Hierarchy

```
ICPPage
└── InternetIdentityManagement
    ├── Status Display (Connection, Principal, App Login, Linked Principals)
    ├── LinkedAccounts (if linked principals exist)
    └── Action Buttons
        ├── "Connect Internet Identity" (when not authenticated)
        └── "Sign Out from Internet Identity" (when authenticated)
```

## Key Files and Their Roles

| File                               | Role                | Key Functions                         |
| ---------------------------------- | ------------------- | ------------------------------------- |
| `page.tsx` (ICP)                   | Entry point         | Renders InternetIdentityManagement    |
| `internet-identity-management.tsx` | Main UI component   | handleSignInII(), status display      |
| `sign-ii-only/page.tsx`            | Sign-in modal       | handleInternetIdentity()              |
| `ic/ii.ts`                         | II authentication   | loginWithII(), clearIiSession()       |
| `lib/ii-client.ts`                 | Backend integration | fetchChallenge(), registerWithNonce() |

## Button Locations Summary

1. **Primary Button:** `InternetIdentityManagement` component, line 196

   - Text: "Connect Internet Identity"
   - Icon: User icon
   - Condition: `!isAuthenticated`

2. **Secondary Button:** `SignIIOnlyContent` component, line 159
   - Text: "Sign in with Internet Identity"
   - Condition: Always visible (modal)

## Authentication States

- **Not Authenticated:** Shows "Connect Internet Identity" button
- **Authenticated:** Shows "Sign Out from Internet Identity" button
- **Loading:** Shows loading spinner with "Checking..." text

## Error Handling

- **Redirect Failures:** Toast notification with "Redirect Failed"
- **II Linking Failures:** Alert with error message
- **Principal Conflicts:** Specific error for already linked principals
- **Session Update Failures:** Warning logs (non-blocking)

## Callback URL Flow

1. User clicks "Connect Internet Identity" on ICP page
2. Current URL is captured and encoded
3. Redirect to `/{locale}/sign-ii-only?callbackUrl={encodedUrl}`
4. After successful authentication, redirect back to original URL

This flow ensures users return to their original location after completing Internet Identity authentication.
