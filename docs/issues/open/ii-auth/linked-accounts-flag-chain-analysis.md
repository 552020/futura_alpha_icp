# Linked Accounts Flag Chain Analysis

## 📋 **Issue Summary**

**Status**: 🔍 **ANALYSIS** - Complete flag chain analysis for LinkedAccounts component display logic

**Purpose**: Trace the exact flag chain that determines whether the LinkedAccounts component shows "not linked yet" or displays the linked II account information.

## 🎯 **The Flag Chain**

The LinkedAccounts component's display is controlled by a single boolean flag that flows through multiple layers:

```
Database → JWT Token → Session → Hook → UI Component
```

## 🔍 **Complete Flag Chain Analysis**

### **1. UI Decision Point**

```typescript
// src/nextjs/src/components/user/linked-accounts.tsx:80
if (!hasLinkedII) {
  return (
    <Card>
      <CardContent>
        <div className="text-center py-6">
          <p className="text-sm">No Internet Identity account linked yet</p>
          <p className="text-xs mt-1">Link your II account to enable ICP operations</p>
        </div>
      </CardContent>
    </Card>
  );
}
```

**Key Point**: The entire "not linked yet" UI is controlled by the `hasLinkedII` boolean flag.

### **2. Hook Computation**

```typescript
// src/nextjs/src/hooks/use-ii-coauth.ts:68
const hasLinkedII = !!linkedIcPrincipal;
```

**Key Point**: `hasLinkedII` is simply a boolean conversion of `linkedIcPrincipal`. If `linkedIcPrincipal` is truthy, `hasLinkedII` is `true`.

### **3. Session Extraction**

```typescript
// src/nextjs/src/hooks/use-ii-coauth.ts:62
const linkedIcPrincipal = (session?.user as ExtendedSessionUser)?.linkedIcPrincipal;
```

**Key Point**: The hook extracts `linkedIcPrincipal` from the NextAuth session user object.

### **4. Session Callback**

```typescript
// src/nextjs/auth.ts:459-461
if (token.linkedIcPrincipal) {
  (session.user as { linkedIcPrincipal?: string }).linkedIcPrincipal = token.linkedIcPrincipal;
}
```

**Key Point**: The session callback copies `linkedIcPrincipal` from the JWT token to the session user object.

### **5. JWT Token Source**

```typescript
// src/nextjs/auth.ts:150-163 (JWT callback)
if (trigger === "signIn" && account) {
  if (!token.linkedIcPrincipal) {
    const uid = (user?.id as string | undefined) ?? (token.sub as string | undefined);
    if (uid) {
      try {
        const iiAccount = await db.query.accounts.findFirst({
          where: (a, { and, eq }) => and(eq(a.userId, uid), eq(a.provider, "internet-identity")),
          columns: { providerAccountId: true },
        });
        if (iiAccount?.providerAccountId) {
          token.linkedIcPrincipal = iiAccount.providerAccountId;
        }
      } catch (error) {
        console.warn("⚠️ [JWT] Failed to fetch linkedIcPrincipal:", error);
      }
    }
  }
}
```

**Key Point**: The JWT token gets `linkedIcPrincipal` from a database lookup in the `accounts` table, looking for a row with `provider = 'internet-identity'` and the current user's ID.

## 🗄️ **Database Schema**

The flag chain starts with the `accounts` table:

```sql
-- accounts table structure
CREATE TABLE accounts (
  id TEXT PRIMARY KEY,
  userId TEXT NOT NULL,
  provider TEXT NOT NULL,
  providerAccountId TEXT NOT NULL,
  -- ... other fields
);

-- The specific row that controls the flag
SELECT providerAccountId
FROM accounts
WHERE userId = ? AND provider = 'internet-identity';
```

## 🔄 **Complete Data Flow**

### **Forward Flow (Database → UI)**

1. **Database**: `accounts.providerAccountId` contains the II principal
2. **JWT Token**: `token.linkedIcPrincipal` gets set from database lookup
3. **Session**: `session.user.linkedIcPrincipal` gets copied from token
4. **Hook**: `linkedIcPrincipal` extracted from session
5. **UI Flag**: `hasLinkedII = !!linkedIcPrincipal`
6. **Component**: Shows linked account info if `hasLinkedII` is true

### **Backward Flow (UI → Database)**

1. **Component**: Shows "not linked yet" if `hasLinkedII` is false
2. **Hook**: `hasLinkedII` is false if `linkedIcPrincipal` is falsy
3. **Session**: `linkedIcPrincipal` is undefined if not in session
4. **JWT Token**: `linkedIcPrincipal` is undefined if not in token
5. **Database**: No row exists with `provider = 'internet-identity'` for this user

## 🚨 **Failure Points**

The "not linked yet" message appears when any of these conditions are met:

### **1. Database Level**

- No row exists in `accounts` table with `provider = 'internet-identity'`
- Database query fails
- User ID mismatch

### **2. JWT Token Level**

- Database lookup fails in JWT callback
- Token doesn't get updated with `linkedIcPrincipal`
- JWT callback doesn't execute (not a fresh sign-in)

### **3. Session Level**

- Session callback doesn't copy `linkedIcPrincipal` from token
- Session update fails
- Session expires

### **4. Hook Level**

- `useSession()` returns null/undefined
- Session user object is malformed
- Type casting fails

### **5. Component Level**

- Hook returns incorrect values
- Component re-renders with stale data

## 🔧 **Debugging the Flag Chain**

### **Check Database**

```sql
SELECT * FROM accounts
WHERE userId = 'USER_ID' AND provider = 'internet-identity';
```

### **Check JWT Token**

```typescript
// In JWT callback, add logging
console.log("🔍 [JWT] linkedIcPrincipal:", token.linkedIcPrincipal);
```

### **Check Session**

```typescript
// In session callback, add logging
console.log("🔍 [Session] linkedIcPrincipal:", session.user.linkedIcPrincipal);
```

### **Check Hook**

```typescript
// In useIICoAuth hook, add logging
console.log("🔍 [Hook] linkedIcPrincipal:", linkedIcPrincipal);
console.log("🔍 [Hook] hasLinkedII:", hasLinkedII);
```

### **Check Component**

```typescript
// In LinkedAccounts component, add logging
console.log("🔍 [Component] hasLinkedII:", hasLinkedII);
```

## 📊 **Flag Chain Summary**

| Layer         | Variable                         | Source             | Controls                    |
| ------------- | -------------------------------- | ------------------ | --------------------------- |
| **Database**  | `accounts.providerAccountId`     | II authentication  | Whether user has linked II  |
| **JWT Token** | `token.linkedIcPrincipal`        | Database lookup    | Token state                 |
| **Session**   | `session.user.linkedIcPrincipal` | Token copy         | Session state               |
| **Hook**      | `linkedIcPrincipal`              | Session extraction | Hook state                  |
| **UI Flag**   | `hasLinkedII`                    | Boolean conversion | Component display           |
| **Component** | `if (!hasLinkedII)`              | Flag check         | "Not linked" vs "Linked" UI |

## 🎯 **Key Insights**

1. **Single Source of Truth**: The database `accounts` table is the ultimate source
2. **Cascading Failure**: If any layer fails, the entire chain breaks
3. **Boolean Conversion**: The final decision is a simple truthy/falsy check
4. **Session Dependency**: The component is completely dependent on NextAuth session state
5. **Database Lookup**: The flag chain requires a successful database query on every fresh sign-in

## 🔍 **Related Documents**

- `internet-identity-authentication-flow-analysis.md` - Overall II authentication flows
- `linked-accounts-component-ii-authentication-sync.md` - Session synchronization issues
- `session-synchronization-problem.md` - ICP page vs LinkedAccounts disconnect
