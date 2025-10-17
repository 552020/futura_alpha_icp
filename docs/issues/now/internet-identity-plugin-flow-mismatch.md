# Internet Identity Plugin Flow Mismatch Issue

## Problem Summary

**Status:** CRITICAL BLOCKER  
**Priority:** High  
**Assigned To:** ICP Expert  
**Created:** 2024-12-19

## Core Problem

The `@dfinity/internet-identity-playwright` plugin expects a **specific flow** that our application's **direct sign-in modal** doesn't follow. This creates an incompatibility that prevents testing the direct sign-in flow.

## What the Plugin Expects

The plugin's `signInWithNewIdentity()` method expects this flow:

1. **Click a button** that triggers Internet Identity flow
2. **Navigate to Internet Identity service** (external URL)
3. **Plugin takes control** of the II service page
4. **Plugin handles** the entire authentication process
5. **Plugin returns** to the application

## What Our Direct Sign-In Modal Does

Our modal button (`handleInternetIdentity()`) follows this flow:

1. **Click button** → Calls `handleInternetIdentity()` function
2. **Function calls `loginWithII()`** → Opens II service directly
3. **Our code handles** the entire authentication process
4. **Our code manages** the return to application
5. **No navigation** - everything happens in place

## The Mismatch

| **Plugin Expects**                    | **Our Modal Does**                    |
| ------------------------------------- | ------------------------------------- |
| Click button → Navigate to II service | Click button → Start II flow directly |
| Plugin controls II service page       | Our code controls II service page     |
| Plugin handles authentication         | Our code handles authentication       |
| Plugin manages return                 | Our code manages return               |

## Visual Flow Comparison

### Plugin's Expected Flow:

```
User clicks button → Navigate to II service → Plugin takes over → Authentication → Return
```

### Our Modal's Actual Flow:

```
User clicks button → Our code starts II → Our code handles everything → Return
```

## The Conflict

**Two Different Drivers:**

- **Plugin wants to drive** the Internet Identity flow
- **Our code already drives** the Internet Identity flow

**Result:** They interfere with each other, causing timeouts and failures.

## Why This Matters

1. **Testing Gap:** We can't test the direct sign-in flow (most common user path)
2. **User Experience:** Direct sign-in is the primary authentication method
3. **Test Coverage:** Missing critical user journey testing
4. **Plugin Limitation:** Plugin assumes a specific flow pattern

## Possible Solutions

### Option 1: Modify Our Code for Tests

- Add test-specific behavior to route to `/sign-ii-only`
- **Problem:** Changes user experience for testing
- **Problem:** Extra navigation step (bad UX)

### Option 2: Align URLs, Keep Direct Flow

- Make our `loginWithII()` use local II service
- Let plugin click button but take over the II popup
- **Problem:** Complex coordination between our code and plugin

### Option 3: Test Different Flow

- Test the ICP management flow instead of direct sign-in
- **Problem:** Doesn't test the actual user experience

### Option 4: Plugin Modification

- Modify plugin to work with direct flows
- **Problem:** Requires plugin changes (not our control)

## Questions for ICP Expert

1. **Is there a way to make the plugin work with direct flows** where the application code already handles the II flow?

2. **Should we modify our application** to support the plugin's expected flow, or is there a better approach?

3. **Are there alternative testing strategies** for Internet Identity that don't require the specific flow the plugin expects?

4. **Is the plugin designed only for specific flow patterns**, or can it be adapted to work with direct authentication flows?

## Current Status

- ✅ **Plugin loads** and imports successfully
- ✅ **Local II service** is running and accessible
- ✅ **URL configuration** is correct
- ❌ **Flow mismatch** prevents successful testing
- ❌ **Direct sign-in flow** cannot be tested

## Impact

- **Cannot test primary user authentication flow**
- **Missing critical test coverage**
- **Plugin limitations block testing**
- **Need expert guidance on resolution**

## Code Analysis

### `loginWithII()` function (in `/src/ic/ii.ts`):

```typescript
export async function loginWithII(): Promise<{ identity: Identity; principal: string }> {
  const provider = process.env.NEXT_PUBLIC_II_URL || process.env.NEXT_PUBLIC_II_URL_FALLBACK;
  if (!provider) throw new Error("II URL not configured");

  const authClient = await getAuthClient();
  const maxTimeToLive = getSessionTtlNs();

  await new Promise<void>((resolve, reject) =>
    authClient.login({
      identityProvider: provider,
      ...(maxTimeToLive ? { maxTimeToLive } : {}),
      onSuccess: resolve,
      onError: reject,
    })
  );

  const identity = authClient.getIdentity();
  const principal = identity.getPrincipal().toString();
  return { identity, principal };
}
```

### `handleInternetIdentity()` function (in `/src/app/[lang]/signin/page.tsx`):

```typescript
async function handleInternetIdentity() {
  if (iiBusy || busy) return;
  setError(null);
  setIiBusy(true);
  try {
    // 1. Ensure II identity with AuthClient.login
    const { loginWithII } = await import("@/ic/ii");
    const { principal, identity } = await loginWithII();

    // Fetch challenge → get { nonceId, nonce }
    const { fetchChallenge } = await import("@/lib/ii-client");
    const challenge = await fetchChallenge(safeCallbackUrl);

    // Register user and prove nonce in one call
    const { registerWithNonce } = await import("@/lib/ii-client");
    await registerWithNonce(challenge.nonce, identity);

    // Call signIn with principal + nonceId + actual nonce
    const signInResult = await signIn("ii", {
      principal,
      nonceId: challenge.nonceId,
      nonce: challenge.nonce,
      redirect: false,
    });

    // Handle success/error and redirect
    if (signInResult?.ok) {
      // Optional: call capsules_bind_neon() on canister
      try {
        const { markBoundOnCanister } = await import("@/lib/ii-client");
        await markBoundOnCanister(identity);
      } catch (error) {
        // Don't fail the auth flow if this optional step fails
      }
      router.push(safeCallbackUrl);
    } else {
      setError(`Authentication failed: ${signInResult?.error || "Unknown error"}`);
    }
  } catch (e) {
    setError("Internet Identity authentication failed");
  } finally {
    setIiBusy(false);
  }
}
```

### The Complete Flow:

1. **Button click** → `handleInternetIdentity()`
2. **`handleInternetIdentity()`** → calls `loginWithII()`
3. **`loginWithII()`** → opens II service via `authClient.login()`
4. **Our code handles** the entire flow from there

This confirms the conflict - our code is already handling the II flow completely, so the plugin can't take over.

## Request

We need guidance on how to test the direct sign-in flow with the Internet Identity Playwright plugin, or alternative approaches that don't require changing our user experience.
