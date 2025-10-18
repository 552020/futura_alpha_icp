# 🐛 Playwright Test Isolation Issue

## Problem

Signup validation tests fail when run with database-dependent tests in parallel.

## Symptoms

- ✅ Tests 1-4 pass when run together
- ❌ Tests 3-4 fail when Test 5 runs in parallel
- ❌ Tests 3-4 fail when Test 6 runs in parallel

## Failing Tests

- **Test 3**: `signup shows validation errors for invalid email`
- **Test 4**: `signup shows validation errors for short password`

## Error Details

```
Error: expect(locator).toBeVisible() failed
Locator: getByText(/Invalid email format/i)
Expected: visible
Timeout: 5000ms
Error: element(s) not found
```

## Root Cause

**Most likely cause**: Tests that create/log in real users leave behind authenticated state (via shared `storageState` file or reused context). When validation tests hit `/signin`, the app auto-redirects to `/dashboard` (already logged in), so the "Invalid email format" text never renders → timeout.

## Test Classification

- **Safe Tests**: 1, 2, 3, 4, 6, 7 (validation/UI only)
- **Database Tests**: 5 (creates real users)

## Impact

- Tests pass individually but fail in parallel
- CI/CD pipeline likely affected
- Test reliability compromised

## Solution (Apply All 3 Fixes)

### 1) Hard-isolate auth state between tests

Do **not** share a global `storageState` across the whole project. Split projects in `playwright.config.ts`:

```ts
import { defineConfig } from "@playwright/test";

export default defineConfig({
  projects: [
    {
      name: "ui",
      use: {
        // force empty state per test; no shared storageState file
        storageState: { cookies: [], origins: [] },
      },
      grepInvert: /@db/, // exclude DB tests
      fullyParallel: true,
    },
    {
      name: "db",
      use: {
        // DB tests can login, but keep it local to each test/context
        storageState: { cookies: [], origins: [] },
      },
      grep: /@db/,
      workers: 1, // serialize DB tests
      fullyParallel: false, // belt-and-suspenders
    },
  ],
});
```

In DB-creating specs add tag:

```ts
test.describe("@db create user flow", () => {
  // ...
});
```

### 2) Make validation tests network-independent

Block auth/API calls in validation-only tests:

```ts
test.describe("Signup validation", () => {
  test.beforeEach(async ({ page }) => {
    await page.route("**/api/**", (route) => route.abort()); // prevent backend from changing UI logic
  });

  test("signup shows validation errors for invalid email", async ({ page }) => {
    await page.goto("/en/signin");
    await page.getByRole("button", { name: /sign up/i }).click();
    await page.getByLabel(/email/i).fill("foo@"); // invalid
    await page.getByRole("button", { name: /sign up with email/i }).click();
    await expect(page.getByText(/invalid email format/i)).toBeVisible();
  });
});
```

### 3) Keep DB data clean and unique

- Generate unique emails in DB tests (already doing `test-${Date.now()}@example.com`)
- Clean up persisted data in `afterEach` if needed
- Consider transaction rollback if backend supports it

## Expected Result

- Tests 3–4 no longer fail when 5–6 run in parallel
- CI becomes reliable: `npx playwright test -p ui` (fast, parallel) and `npx playwright test -p db` (serialized)

## Priority

High - Test reliability issue blocking development workflow
