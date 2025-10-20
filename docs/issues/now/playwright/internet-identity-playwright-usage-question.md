# Internet Identity Playwright Plugin Usage Question

## Question for ICP Expert

**Subject:** How to properly use `@dfinity/internet-identity-playwright` plugin for local testing

**Priority:** High  
**Status:** NEEDS EXPERT GUIDANCE

## Current Situation

We have successfully:

- ✅ Fixed ES Module compatibility issues
- ✅ Set up local Internet Computer replica with `dfx start --clean`
- ✅ Deployed Internet Identity with `dfx deploy internet_identity`
- ✅ Plugin imports and loads correctly

## Current Issue

The plugin gets stuck waiting for `#userNumber` element and times out after 60 seconds. We're not sure about the correct configuration.

## Current Configuration

```typescript
// Test setup
testWithII.beforeEach(async ({ iiPage }) => {
  const canisterId = execSync("dfx canister id internet_identity", { encoding: "utf8" }).trim();

  await iiPage.waitReady({
    url: `http://127.0.0.1:4943/?canisterId=${canisterId}`,
    canisterId,
  });
});

// Test execution
await iiPage.signInWithNewIdentity({
  selector: 'button:has-text("Sign in with Internet Identity")',
});
```

## Questions

1. **URL Configuration:** What's the correct URL format for local Internet Identity service?
2. **Plugin Usage:** How should we properly use `signInWithNewIdentity()`?
3. **Element Selection:** What selector should we use for the Internet Identity button?
4. **Setup Requirements:** Are there any specific dfx or Internet Identity configuration requirements?

## Error Details

### Original Error (Fixed)

```
Error: locator.textContent: Test timeout of 60000ms exceeded.
Call log:
  - waiting for locator('#userNumber')
```

### New Error After Tech Lead's Fixes

```
Error: expect(page).toHaveURL(expected) failed

Expected pattern: /\/en\/sign-ii-only/
Received string:  "http://localhost:3000/en/signin"
Timeout: 5000ms

Call log:
  - Expect "toHaveURL" with timeout 5000ms
  9 × unexpected value "http://localhost:3000/en/signin"
```

**Issue:** The "Sign in with Internet Identity" button in the modal (`/en/signin`) doesn't navigate to `/en/sign-ii-only` as expected. It stays on the signin page.

**Current Test Flow:**

1. ✅ Go to homepage (`/en`)
2. ✅ Click "Sign In" button in header (opens modal)
3. ✅ Wait for modal with "Sign in with Internet Identity" button
4. ✅ Click "Sign in with Internet Identity" button
5. ❌ **Expected:** Navigate to `/en/sign-ii-only`
6. ❌ **Actual:** Stays on `/en/signin`

**Question:** Should the modal button navigate to `/sign-ii-only`, or should we use a different approach for the Internet Identity flow?

### Latest Error After Tech Lead's Fixes + Correct Flow

```
Test timeout of 60000ms exceeded.

Error: locator.textContent: Test timeout of 60000ms exceeded.
Call log:
  - waiting for locator('#userNumber')
```

**Issue:** Even with the correct flow and tech lead's fixes, the plugin is still timing out waiting for `#userNumber` element. This suggests the Internet Identity service isn't loading properly or there's still a configuration issue.

**Current Test Flow (Correct):**

1. ✅ Go to homepage (`/en`)
2. ✅ Click "Sign In" button in header → Navigates to `/en/signin` (modal)
3. ✅ Wait for modal with Internet Identity button
4. ✅ Use plugin with `[data-testid="ii-signin-button"]` selector
5. ❌ **Plugin times out** waiting for `#userNumber` element

**Question:** Is there still a configuration issue with the Internet Identity service setup, or are we missing something in the plugin usage?

## Direct Sign-In Flow Analysis

We have **two different Internet Identity flows** in our application:

### Flow 1: Direct Sign-In from Header (What we want to test)

- **Path:** Header "Sign In" → `/en/signin` modal → "Sign in with Internet Identity" button
- **Behavior:** Button calls `handleInternetIdentity()` which starts II flow directly using `loginWithII()`
- **Does NOT navigate** to `/sign-ii-only` page
- **Handles everything in place** - no page navigation

### Flow 2: ICP Management Page Flow

- **Path:** `/en/user/icp` → "Connect Internet Identity" button → `/en/sign-ii-only` page
- **Behavior:** Navigates to dedicated `/sign-ii-only` page with `[data-testid="ii-start"]` button
- **Uses separate page** for II flow

### The Problem

The **direct sign-in flow** (Flow 1) doesn't use the `/sign-ii-only` page at all. The modal button starts the II flow directly via `handleInternetIdentity()` function. But the plugin expects to handle the flow itself.

**Question:** How should we test the direct sign-in flow when our button already handles the II flow internally? Should we:

1. **Modify the button** to let the plugin handle the flow instead of our code?
2. **Use a different approach** for testing the direct flow?
3. **Test the ICP flow instead** (which uses `/sign-ii-only` page)?

## Environment

- **dfx:** Latest version
- **Internet Identity:** Deployed locally
- **Canister ID:** `uzt4z-lp777-77774-qaabq-cai`
- **Replica Port:** 4943
- **Plugin Version:** 2.0.0

## Request

Could you provide guidance on the correct setup and usage of the Internet Identity Playwright plugin for local testing?
