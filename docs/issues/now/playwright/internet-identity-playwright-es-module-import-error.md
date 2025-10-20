# Internet Identity Playwright Plugin ES Module Import Error

## Issue Summary

The `@dfinity/internet-identity-playwright` plugin cannot be imported in our Playwright tests due to an ES Module compatibility issue. The plugin is an ES Module but Playwright is trying to import it using CommonJS `require()`.

## Error Details

```
Error: require() of ES Module /Users/stefano/Documents/Code/Futura/futura_alpha_icp/src/nextjs/node_modules/.pnpm/@dfinity+internet-identity-playwright@2.0.0_@playwright+test@1.56.0/node_modules/@dfinity/internet-identity-playwright/dist/index.js from /Users/stefano/Documents/Code/Futura/futura_alpha_icp/src/nextjs/e2e/auth.internet-identity.spec.ts not supported.
Instead change the require of index.js in /Users/stefano/Documents/Code/Futura/futura_alpha_icp/src/nextjs/e2e/auth.internet-identity-playwright/dist/index.js to a dynamic import() which is available in all CommonJS modules.
```

## Current Test File

**File:** `src/nextjs/e2e/auth.internet-identity.spec.ts`

```typescript
import { testWithII, expect } from "@dfinity/internet-identity-playwright";

testWithII.describe("Internet Identity Authentication", () => {
  testWithII("II sign-in from header avatar when not authenticated", async ({ page, iiPage }) => {
    // Test implementation
  });
});
```

## Environment Details

- **Package:** `@dfinity/internet-identity-playwright@2.0.0`
- **Playwright Version:** `1.56.0`
- **Node.js:** ES Module system
- **Package Manager:** pnpm
- **Project:** Next.js with TypeScript

## What Works

✅ **Modal Flow is Correct:** Our debug tests confirm the Internet Identity sign-in flow works perfectly:

- Header "Sign In" button → Opens modal
- Modal contains "Sign in with Internet Identity" button
- Button is visible and clickable
- No console errors

✅ **Manual Testing:** The Internet Identity authentication flow works when tested manually in the browser.

## What Doesn't Work

❌ **Playwright Plugin Import:** Cannot import the `@dfinity/internet-identity-playwright` plugin due to ES Module compatibility.

## Root Cause Analysis

The issue is a **module system mismatch**:

1. **Plugin is ES Module:** The `@dfinity/internet-identity-playwright` package is built as an ES Module
2. **Playwright expects CommonJS:** Playwright's test runner is trying to import it using `require()`
3. **Node.js restriction:** Node.js doesn't allow `require()` of ES Modules in CommonJS context

## Potential Solutions

### Option 1: Dynamic Import (Recommended)

Change the import to use dynamic import syntax:

```typescript
// Instead of:
import { testWithII, expect } from "@dfinity/internet-identity-playwright";

// Use:
const { testWithII, expect } = await import("@dfinity/internet-identity-playwright");
```

### Option 2: Playwright Configuration

Check if there's a Playwright configuration option to handle ES Modules:

```typescript
// playwright.config.ts
export default defineConfig({
  // Add ES Module support
  use: {
    // Configuration for ES Modules
  },
});
```

### Option 3: Plugin Configuration

The plugin might need to be configured differently in the Playwright config:

```typescript
// playwright.config.ts
export default defineConfig({
  projects: [
    {
      name: "chromium",
      use: {
        // Internet Identity plugin configuration
        // ...devices['Desktop Chrome'],
      },
    },
  ],
});
```

### Option 4: Alternative Testing Approach

If the plugin continues to have issues, we could:

1. Use regular Playwright tests without the plugin
2. Mock the Internet Identity authentication
3. Use a different testing strategy for II flows

## Test Implementation Status

### ✅ Completed

- [x] Modal flow analysis and documentation
- [x] Debug test confirming modal functionality
- [x] Internet Identity button detection
- [x] Modal interaction testing

### ❌ Blocked

- [ ] Internet Identity Playwright plugin integration
- [ ] Automated II authentication testing
- [ ] II flow end-to-end testing

## Files Involved

| File                                 | Status     | Notes                      |
| ------------------------------------ | ---------- | -------------------------- |
| `e2e/auth.internet-identity.spec.ts` | ❌ Blocked | Cannot import plugin       |
| `e2e/debug-signin.spec.ts`           | ✅ Working | Tests modal functionality  |
| `playwright.config.ts`               | ✅ Working | Server configuration fixed |

## Next Steps

1. **Investigate ES Module solutions** for the Internet Identity Playwright plugin
2. **Test dynamic import approach** if recommended
3. **Check plugin documentation** for ES Module configuration
4. **Consider alternative testing strategies** if plugin issues persist

## Related Documentation

- [Internet Identity Direct Sign-In Flow Analysis](../analysis/internet-identity-direct-signin-flow.md)
- [Playwright Configuration](../architecture/playwright-configuration.md)

## ✅ SOLUTION IMPLEMENTED

**Status:** RESOLVED - ES Module compatibility issue fixed

### What Was Fixed

1. **Added ES Module support to project:**

   ```json
   // package.json
   {
     "type": "module"
   }
   ```

2. **Updated TypeScript configuration:**

   ```json
   // tsconfig.json
   {
     "compilerOptions": {
       "module": "ESNext"
     }
   }
   ```

3. **Result:** Internet Identity Playwright plugin now imports successfully without ES Module errors.

### Current Status

- ✅ **ES Module Import:** Fixed - plugin loads successfully
- ✅ **Modal Flow:** Working - confirmed by debug tests
- ❌ **Navigation Issue:** New issue - plugin expects "Internet Identity" page title but gets empty title

### Next Steps

The plugin is now working but has a navigation issue where it expects the page title to be "Internet Identity" but receives an empty title. This may be due to:

1. Internet Identity service accessibility in test environment
2. Network configuration issues
3. Plugin configuration needs adjustment

## Priority

**Medium** - ES Module issue resolved, but navigation issue remains for full Internet Identity testing.

## Assigned To

**ICP Expert** - Requires knowledge of:

- Internet Identity service configuration for testing
- Playwright plugin navigation setup
- Test environment Internet Identity accessibility
