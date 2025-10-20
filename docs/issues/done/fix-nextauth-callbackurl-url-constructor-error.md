# Fix NextAuth callbackUrl URL Constructor Error

## 🎯 **Problem**

The email/password signup flow was failing with a `TypeError: Failed to construct 'URL': Invalid URL` error, even though the user was being created successfully in the database.

## 🐛 **Root Cause**

The issue was caused by passing a **relative URL path** (e.g., `/en/dashboard`) to NextAuth's `signIn` function's `callbackUrl` parameter. NextAuth internally tries to construct a full URL from this parameter, but the URL constructor fails when given a relative path.

### **Error Details:**

```typescript
// ❌ This caused the error:
const res = await signIn("credentials", {
  email,
  password,
  redirect: false,
  callbackUrl: "/en/dashboard", // ← Relative path causes URL constructor to fail
});
```

### **Error Stack:**

```
TypeError: Failed to construct 'URL': Invalid URL
at async handleSignUp (page.tsx:106:19)
```

## 💡 **Solution Implemented**

**Removed the `callbackUrl` parameter** from the `signIn` call and handled navigation manually:

```typescript
// ✅ Fixed version:
const res = await signIn("credentials", {
  email,
  password,
  redirect: false,
  // Removed callbackUrl - let NextAuth handle authentication only
});

// Manual navigation after successful sign-in
if (!res?.error) {
  router.push(safeCallbackUrl);
}
```

## 🔧 **Technical Details**

### **Why This Happened:**

1. **NextAuth expects full URLs** for `callbackUrl` (e.g., `http://localhost:3000/en/dashboard`)
2. **Relative paths cause URL constructor to fail** when NextAuth tries to process them
3. **The error was caught by the catch block**, showing "Sign up failed" to users

### **Alternative Solutions Considered:**

1. **Convert to full URL**: `callbackUrl: window.location.origin + safeCallbackUrl`
2. **Remove callbackUrl entirely**: Let NextAuth handle navigation (chosen)
3. **Use NextAuth redirect**: Set `redirect: true` and handle via NextAuth config

## ✅ **Resolution**

- ✅ **User creation works** - API returns 200 status
- ✅ **Automatic sign-in works** - NextAuth authenticates successfully
- ✅ **Navigation works** - Manual `router.push()` handles redirect
- ✅ **No more URL constructor errors** - Removed problematic parameter

## 🧪 **Testing**

The fix has been tested and confirmed working:

- User signup completes successfully
- Automatic sign-in works without errors
- User is redirected to the dashboard
- No more "Sign up failed" error messages

## 📝 **Lessons Learned**

1. **NextAuth `callbackUrl` requires full URLs** - not relative paths
2. **Manual navigation is more reliable** for complex routing scenarios
3. **Error handling should distinguish** between authentication failures and navigation issues

## 🔄 **Future Considerations**

Consider implementing a utility function to handle callback URLs consistently across the application:

```typescript
function getFullCallbackUrl(path: string): string {
  return typeof window !== "undefined" ? `${window.location.origin}${path}` : path;
}
```

This would prevent similar issues in other parts of the application that use NextAuth with custom callback URLs.
