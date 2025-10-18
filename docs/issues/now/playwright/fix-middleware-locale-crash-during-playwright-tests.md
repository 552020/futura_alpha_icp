# Fix: Middleware Locale Crash During Playwright Tests

## 🎯 **Problem**

The Next.js middleware is crashing with a `RangeError: Incorrect locale information provided` when Playwright tests make requests to the server. This prevents any E2E testing from working.

## 🐛 **Error Details**

```
⨯ Error [RangeError]: Incorrect locale information provided
    at Intl.getCanonicalLocales (<anonymous>)
    at getLocale (src/middleware.ts:16:29)
    at middleware (src/middleware.ts:84:20)
```

## 🔍 **Root Cause**

The middleware (`src/middleware.ts`) is receiving **invalid locale information** from Playwright's HTTP requests, causing `Intl.getCanonicalLocales()` to throw a `RangeError`.

### **Why This Happens:**

1. **Playwright requests** don't include proper locale headers
2. **The middleware** tries to process locale information from these requests
3. **Invalid locale data** is passed to `Intl.getCanonicalLocales()`
4. **The server crashes** on every request

## 💡 **Proposed Solutions**

### **Option 1: Add Error Handling to Middleware (Recommended)**

```typescript
// src/middleware.ts
function getLocale(request: NextRequest) {
  try {
    const negotiatorHeaders: Record<string, string> = {};
    request.headers.forEach((value, key) => {
      negotiatorHeaders[key] = value;
    });
    const languages = new Negotiator({ headers: negotiatorHeaders }).languages();
    const locale = matchLocale(languages, locales, defaultLocale);
    return locale;
  } catch (error) {
    // Fallback to default locale if locale detection fails
    console.warn("Locale detection failed, using default:", error);
    return defaultLocale;
  }
}
```

### **Option 2: Skip Middleware for Test Environment**

```typescript
// src/middleware.ts
export function middleware(request: NextRequest) {
  // Skip middleware during testing
  if (process.env.NODE_ENV === "test" || process.env.PLAYWRIGHT === "true") {
    return NextResponse.next();
  }

  // ... existing middleware logic
}
```

### **Option 3: Add Test-Specific Headers**

```typescript
// In Playwright tests
await page.goto("http://localhost:3000/en/signin", {
  headers: {
    "Accept-Language": "en-US,en;q=0.9",
  },
});
```

## 🚀 **Recommended Fix**

**Option 1** is the most robust solution as it:

- ✅ Handles edge cases gracefully
- ✅ Doesn't break existing functionality
- ✅ Provides fallback behavior
- ✅ Logs warnings for debugging

## 🧪 **Testing the Fix**

After implementing the fix, test with:

```bash
# Run Playwright tests
pnpm exec playwright test simple-signup.spec.ts

# Should no longer see locale errors in server logs
```

## 📊 **Impact**

- **Current State:** E2E testing completely broken
- **After Fix:** E2E testing works properly
- **Risk:** Low - only adds error handling, doesn't change core logic

## 🔧 **Files to Modify**

- `src/middleware.ts` - Add try/catch around locale detection
- `playwright.config.ts` - Ensure proper test configuration

## 📝 **Additional Notes**

This issue affects all E2E testing, not just signup tests. The middleware crash prevents any Playwright test from working properly.
