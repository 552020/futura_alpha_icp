# Remove Default NextAuth Sign-in Page

## Issue Description

The application currently has two different sign-in interfaces:

1. **Custom sign-in page** at `/[lang]/signin` (our preferred UI)
2. **Default NextAuth page** at `/api/auth/signin` (unwanted)

Users can access the default NextAuth page by clicking "Use default sign-in page" link at the bottom of our custom signin page.

## Problem

- **Inconsistent UX**: Two different sign-in experiences
- **Confusing for users**: Default NextAuth page doesn't match our design
- **Security concern**: Default page might expose different authentication options
- **Maintenance burden**: Two different authentication flows to maintain

## Current Behavior

1. User visits `/[lang]/signin`
2. Sees our custom sign-in page with:
   - Google OAuth
   - Internet Identity
   - Email/Password form
3. At bottom, sees "Use default sign-in page" link
4. Clicking link redirects to `/api/auth/signin?callbackUrl=...`
5. Shows NextAuth default page with different UI/options

## Expected Behavior

- Only our custom sign-in page should be accessible
- No link to default NextAuth page
- Consistent authentication experience

## Solution Options

### Option 1: Remove the Link

Remove the "Use default sign-in page" link from our custom signin page.

### Option 2: Disable Default NextAuth Page

Configure NextAuth to not generate the default sign-in page.

### Option 3: Redirect Default Page

Redirect `/api/auth/signin` to our custom signin page.

## Recommended Approach

**Option 1 + Option 3**: Remove the link and redirect the default page to our custom signin.

## Implementation

1. Remove the link from `/[lang]/signin/page.tsx`:

   ```tsx
   // Remove this section:
   <div className="mt-4 text-center text-xs text-muted-foreground">
     <Link href={`/api/auth/signin?callbackUrl=${encodeURIComponent(callbackUrl)}`}>Use default sign-in page</Link>
   </div>
   ```

2. Add redirect in NextAuth configuration or middleware to redirect `/api/auth/signin` to `/[lang]/signin`.

## Priority

**Medium** - UX improvement, not critical functionality.

## Acceptance Criteria

- [ ] "Use default sign-in page" link removed
- [ ] `/api/auth/signin` redirects to custom signin page
- [ ] Only one sign-in interface accessible to users
- [ ] All authentication flows work through custom page
