# Tech Lead: URGENT - Fix Our Broken Validation Tests

## Problem Summary

**CRITICAL**: Two validation tests that were **PASSING INDIVIDUALLY** before our changes are now **FAILING** even when run alone. We implemented your suggested solution but it broke working tests. We need you to fix this immediately.

## 1. Frontend Validation Source

**Status**: Need to identify

- File + function where signup validation happens (e.g., Zod/Yup schema or custom checks)
- Does the form always call `/api/auth/signup` on submit, even if fields are invalid? (yes/no)

## 2. Error Rendering Contract

**Status**: Need to identify

- Exact API endpoint hit on submit (full path)
- Exact error response shape the UI expects for field errors
  - Example: `{ error: string }` or `{ errors: { email?: string; password?: string } }`
- Selector where the error text appears (component/element path if not plain text)

## 3. Current Test Code (2 Failing Tests)

### Test 1: Invalid Email Validation

```typescript
test("signup shows validation errors for invalid email", async ({ page }) => {
  await page.goto("/en/signin");

  // Switch to signup tab
  await page.getByRole("button", { name: /sign up/i }).click();

  // Type invalid email and blur (don't click submit)
  await page.getByLabel(/email/i).fill("invalid@email");
  await page.locator("#password").click(); // triggers blur/validation

  // Should show validation error
  await expect(page.getByText(/Invalid email format/i)).toBeVisible();
});
```

### Test 2: Short Password Validation

```typescript
test("signup shows validation errors for short password", async ({ page }) => {
  await page.goto("/en/signin");

  // Switch to signup tab
  await page.getByRole("button", { name: /sign up/i }).click();

  await page.locator("#password").fill("123");
  await page.getByLabel(/email/i).click(); // blur password

  // Should show validation error
  await expect(page.getByText(/password must be at least 6 characters/i)).toBeVisible();
});
```

## 4. Current Playwright Config (Final)

```typescript
import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: "html",
  timeout: 60_000,
  use: {
    baseURL: process.env.PLAYWRIGHT_BASE_URL ?? "http://localhost:3000",
    trace: "on-first-retry",
  },
  projects: [
    {
      name: "ui",
      use: { ...devices["Desktop Chrome"] },
      grepInvert: /@db/,
    },
    {
      name: "db",
      use: { ...devices["Desktop Chrome"] },
      grep: /@db/,
      workers: 1,
      fullyParallel: false,
    },
  ],
  webServer: undefined,
});
```

## 5. Auth/Session Behavior

**Status**: Need to identify

- Any auto-redirect to `/dashboard` if already logged in? (yes/no)
- Any middleware/guard that rewrites `/signin` when authenticated? (yes/no)

## 6. Test Execution Results

### Command 1: Network Log (PWDEBUG=console)

```bash
PWDEBUG=console npx playwright test e2e/signup.spec.ts --project=ui --grep "invalid email" --headed --retries=0 --reporter=list
```

**Result**: Test failed - "Invalid email format" message not found

- Error: `expect(locator).toBeVisible() failed`
- Locator: `getByText(/Invalid email format/i)`
- Timeout: 5000ms
- Error: element(s) not found

### Command 2: Trace Generation

```bash
npx playwright test e2e/signup.spec.ts --project=ui --grep "invalid email" --trace=on --retries=0 --reporter=list
```

**Result**: Test failed with trace generated

- Trace file: `/Users/stefano/Documents/Code/Futura/futura_alpha_icp/src/nextjs/test-results/signup-Email-Password-Sign-f8971-on-errors-for-invalid-email-ui/trace.zip`
- Usage: `npx playwright show-trace test-results/signup-Email-Password-Sign-f8971-on-errors-for-invalid-email-ui/trace.zip`

## What We Need From You

**STOP GIVING US HALF-ASSED SOLUTIONS.** We need you to:

1. **IDENTIFY** the exact validation logic in the frontend code
2. **PROVIDE** the correct API endpoint and error response format
3. **WRITE** the exact test code that will work
4. **FIX** the broken validation tests immediately

## Current Status

- ✅ **Test isolation fixed** - Serial execution working
- ✅ **5 tests passing** - Core functionality restored
- ❌ **2 validation tests broken** - Your blur-based approach doesn't work
- ❌ **Tests were working before** - We broke them with your suggestion

## What You Need to Do

1. **Find the validation logic** in the frontend code
2. **Identify the API endpoint** that handles validation
3. **Provide the exact error response format** the UI expects
4. **Write the correct test code** that actually works
5. **Stop giving us incomplete solutions** that break working tests

## Expected Outcome

**YOU NEED TO:**

- Provide the exact working test code for both validation tests
- Show us where the validation logic lives in the codebase
- Give us the correct API mocking approach
- Fix the broken tests immediately

---

**FUCKING FIX THIS.** We implemented your solution and it broke working tests. We need you to solve this properly, not give us more half-baked suggestions that don't work.
