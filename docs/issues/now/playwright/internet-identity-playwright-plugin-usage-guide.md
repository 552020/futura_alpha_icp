# Internet Identity Playwright Plugin Usage Guide

## Issue Summary

**Status:** NEEDS EXPERT GUIDANCE  
**Priority:** High  
**Assigned To:** ICP Expert  
**Created:** 2024-12-19

## Problem Description

We are trying to use the `@dfinity/internet-identity-playwright` plugin for end-to-end testing of Internet Identity authentication flows, but we're encountering issues with the plugin configuration and usage. The plugin appears to be working (ES Module compatibility is resolved), but we need guidance on proper setup and configuration.

## Current Setup

### ✅ What's Working

1. **ES Module Compatibility:** Fixed with `"type": "module"` in package.json
2. **Plugin Import:** Successfully importing `@dfinity/internet-identity-playwright`
3. **Local Internet Computer Replica:** Running with `dfx start --clean`
4. **Internet Identity Deployment:** Successfully deployed with `dfx deploy internet_identity`
5. **Canister ID Detection:** Dynamically getting canister ID with `dfx canister id internet_identity`

### ❌ Current Issues

1. **Plugin Navigation:** The plugin is getting stuck waiting for `#userNumber` element
2. **URL Configuration:** Uncertain about correct URL format for local Internet Identity service
3. **Test Flow:** Plugin navigation to Internet Identity service is not working as expected

## Current Configuration

### Playwright Configuration

```typescript
// playwright.config.ts
import { defineConfig, devices } from "@playwright/test";
import { execSync } from "child_process";

// Check if dev server is running in local development
function checkDevServer() {
  if (process.env.CI) return; // Skip check in CI

  try {
    execSync("curl -s http://localhost:3000 > /dev/null", { stdio: "ignore" });
    console.log("✅ Dev server is running on http://localhost:3000");
  } catch (_error) {
    console.error("❌ Dev server is not running on http://localhost:3000");
    console.error("Please start the dev server first:");
    console.error("  pnpm dev:nextjs");
    console.error("Then run the tests again.");
    process.exit(1);
  }
}

// Check dev server before running tests
checkDevServer();
```

### Internet Identity Test Configuration

```typescript
// e2e/auth.internet-identity.spec.ts
import { testWithII, expect } from "@dfinity/internet-identity-playwright";
import { execSync } from "child_process";

testWithII.describe("Internet Identity Authentication", () => {
  // Check if Internet Computer replica is running before tests
  testWithII.beforeAll(async () => {
    try {
      execSync("dfx ping", { stdio: "ignore" });
      console.log("✅ Internet Computer replica is running");
    } catch (_error) {
      console.error("❌ Internet Computer replica is not running");
      console.error("Please start the local replica first:");
      console.error("  dfx start --clean");
      console.error("  dfx deploy internet_identity");
      console.error("Then run the tests again.");
      process.exit(1);
    }
  });

  // Configure Internet Identity service URL
  testWithII.beforeEach(async ({ iiPage }) => {
    // Get the actual canister ID from dfx
    let canisterId: string;
    try {
      canisterId = execSync("dfx canister id internet_identity", { encoding: "utf8" }).trim();
      console.log(`✅ Using Internet Identity canister ID: ${canisterId}`);
    } catch (_error) {
      console.error("❌ Failed to get Internet Identity canister ID");
      console.error("Make sure Internet Identity is deployed:");
      console.error("  dfx deploy internet_identity");
      process.exit(1);
    }

    // Use local Internet Identity service (requires dfx to be running)
    await iiPage.waitReady({
      url: `http://127.0.0.1:4943/?canisterId=${canisterId}`,
      canisterId,
    });
  });

  testWithII("II sign-in from header avatar when not authenticated", async ({ page, iiPage }) => {
    // 1) Go to homepage (not authenticated)
    await page.goto("/en");

    // 2) Click sign in button in header - this opens a modal
    await page.getByRole("button", { name: "Sign In" }).click();

    // 3) Wait for modal to appear and look for Internet Identity button
    await expect(page.getByText("Sign in with Internet Identity")).toBeVisible();

    // 4) Start II flow directly - the button is already visible in the modal
    await iiPage.signInWithNewIdentity({ selector: 'button:has-text("Sign in with Internet Identity")' });

    // 5) Assert: we are authenticated (avatar visible in header)
    await expect(page.getByTestId("user-avatar")).toBeVisible();
  });
});
```

## Questions for ICP Expert

### 1. Plugin Configuration

- **URL Format:** What is the correct URL format for local Internet Identity service?
  - Current: `http://127.0.0.1:4943/?canisterId=${canisterId}`
  - Alternative: `http://127.0.0.1:4943` with separate canisterId parameter?

### 2. Plugin Usage

- **Navigation:** How should the plugin navigate to Internet Identity service?
- **Element Selection:** What selector should be used for the Internet Identity button?
- **Timing:** Are there specific timing requirements or wait conditions?

### 3. Local Development Setup

- **dfx Configuration:** Are there specific dfx configuration requirements?
- **Internet Identity Setup:** Any specific setup steps for local Internet Identity?
- **Port Configuration:** Should we use different ports or configurations?

### 4. Test Flow

- **Modal Interaction:** How should the plugin interact with modals in the application?
- **Button Selection:** What's the correct way to select the Internet Identity button?
- **Navigation Flow:** Should the plugin handle navigation automatically or manually?

## Current Error Details

```
Error: locator.textContent: Test timeout of 60000ms exceeded.
Call log:
  - waiting for locator('#userNumber')

Test timeout of 60000ms exceeded.
Error: locator.click: Test timeout of 60000ms exceeded.
Call log:
  - waiting for getByTestId('ii-connect')
```

## Environment Details

- **Node.js:** v22+
- **Playwright:** Latest
- **Internet Identity Playwright Plugin:** v2.0.0
- **dfx:** Latest
- **Local Replica:** Running on port 4943
- **Internet Identity Canister ID:** `uzt4z-lp777-77774-qaabq-cai`

## Expected Behavior

The plugin should:

1. Navigate to the Internet Identity service
2. Create a new identity
3. Complete the authentication flow
4. Return to the application with the user authenticated

## Current Behavior

The plugin:

1. ✅ Successfully imports and loads
2. ✅ Detects local Internet Computer replica
3. ✅ Gets correct canister ID
4. ❌ Gets stuck waiting for `#userNumber` element
5. ❌ Times out after 60 seconds

## Request for Expert Guidance

We need guidance on:

1. **Correct plugin configuration** for local Internet Identity service
2. **Proper URL format** and parameters
3. **Element selection strategies** for Internet Identity buttons
4. **Navigation flow** and timing requirements
5. **Best practices** for Internet Identity testing with Playwright

## Related Documentation

- [Internet Identity Playwright Plugin](https://github.com/dfinity/internet-identity-playwright)
- [Internet Identity Documentation](https://internetcomputer.org/docs/current/developer-docs/integrations/internet-identity/)
- [Playwright Configuration](../architecture/playwright-configuration.md)

## Priority

**High** - This is blocking automated testing of Internet Identity authentication flows, which is a core feature of the application.

## Assigned To

**ICP Expert** - Requires knowledge of:

- Internet Identity Playwright plugin configuration
- Local Internet Computer replica setup
- Internet Identity service configuration
- Playwright testing best practices for Internet Computer applications
