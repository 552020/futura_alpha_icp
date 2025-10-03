# Fix NextAuth Redirect Callback UX

## 📋 **Issue Summary**

Currently, the NextAuth `redirect` callback forces every successful login to `/${lang}/dashboard`, which creates a poor user experience by discarding the user's original destination. Users expect to return to the page they were on before signing in.

## 🔍 **Current Problem**

### **Bad UX Pattern**

```typescript
// Current: auth.ts - redirect callback
redirect({ url, baseUrl }) {
  // ❌ Always redirects to dashboard, ignoring user's original destination
  return `${baseUrl}/en/dashboard`;
}
```

### **User Experience Issues**

- **Lost context**: User was on `/user/icp` → signs in → lands on `/dashboard`
- **Frustrating flow**: User has to navigate back to their intended page
- **Poor conversion**: Users abandon the flow due to confusion
- **Inconsistent behavior**: Different from standard web authentication patterns

## 🎯 **Proposed Solution**

### **1. Pass Intended Destination as `callbackUrl`**

```typescript
// Client components - pass current location
await signIn("google", {
  callbackUrl: window.location.href, // Preserve current page
});

// Or specific in-app URL
await signIn("google", {
  callbackUrl: "/user/icp", // Specific destination
});

// For credentials (II authentication)
await signIn("ii", {
  principal: "",
  nonceId: challenge.nonceId,
  nonce: challenge.nonce,
  redirect: true,
  callbackUrl: window.location.href, // ✅ Preserve destination
});
```

### **2. Update Redirect Callback to Preserve Destination**

```typescript
// auth.ts - improved redirect callback
redirect({ url, baseUrl }) {
  try {
    const u = new URL(url);

    // ✅ Same-origin URLs are safe to preserve
    if (u.origin === baseUrl) {
      return u.toString(); // Preserve original destination
    }
  } catch {
    // Invalid URL - fall back to default
  }

  // ✅ Safe fallback for invalid/missing callbackUrl
  return `${baseUrl}/en/dashboard`;
}
```

### **3. Handle Language from Callback URL**

```typescript
// auth.ts - enhanced redirect with language detection
redirect({ url, baseUrl }) {
  try {
    const u = new URL(url);

    if (u.origin === baseUrl) {
      // ✅ Extract language from callback URL
      const lang = u.searchParams.get("lang") ||
                   u.pathname.split("/")[1] ||
                   "en";

      // ✅ Preserve original destination with correct language
      return u.toString();
    }
  } catch {
    // Invalid URL - fall back to default
  }

  // ✅ Safe fallback with default language
  return `${baseUrl}/en/dashboard`;
}
```

## ✅ **Benefits**

### **Improved User Experience**

- **Context preservation**: Users return to their original page
- **Seamless flow**: No navigation confusion
- **Better conversion**: Users complete intended actions
- **Standard behavior**: Matches web authentication expectations

### **Technical Benefits**

- **Flexible routing**: Supports any in-app destination
- **Language support**: Automatically handles internationalization
- **Security**: Only same-origin URLs are preserved
- **Fallback safety**: Invalid URLs default to dashboard

## 📁 **Files to Modify**

### **Modified Files**

- `src/nextjs/auth.ts` - Update `redirect` callback
- `src/nextjs/src/app/[lang]/user/icp/page.tsx` - Add `callbackUrl` to `signIn` calls
- `src/nextjs/src/app/[lang]/sign-ii-only/page.tsx` - Add `callbackUrl` to `signIn` calls
- Any other components using `signIn()` - Add `callbackUrl` parameter

### **Example Updates**

```typescript
// Before: src/app/[lang]/user/icp/page.tsx
await signIn("ii", {
  principal: "",
  nonceId: challenge.nonceId,
  nonce: challenge.nonce,
  redirect: true,
  callbackUrl: safeCallbackUrl, // ❌ This might be external
});

// After: src/app/[lang]/user/icp/page.tsx
await signIn("ii", {
  principal: "",
  nonceId: challenge.nonceId,
  nonce: challenge.nonce,
  redirect: true,
  callbackUrl: window.location.href, // ✅ Preserve current page
});
```

## 🎯 **Implementation Priority**

**High** - This directly impacts user experience and conversion rates. Users expect to return to their original destination after authentication.

## 🔗 **Related Issues**

- Internet Identity authentication flow analysis
- Linked accounts component II authentication sync
- NextAuth JWT explanation
- Optimize nonce verification architecture

## 📝 **Testing Checklist**

- [ ] User on `/user/icp` → signs in → returns to `/user/icp`
- [ ] User on `/dashboard` → signs in → returns to `/dashboard`
- [ ] Invalid `callbackUrl` → falls back to `/en/dashboard`
- [ ] Language detection works correctly
- [ ] External URLs are blocked (security)
- [ ] Works with both Google and II authentication
