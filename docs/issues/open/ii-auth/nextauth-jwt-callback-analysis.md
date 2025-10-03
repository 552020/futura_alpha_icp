# NextAuth JWT Callback Analysis

## 📋 **Overview**

Our NextAuth.js JWT callback is a **complex state management system** that handles multiple authentication scenarios, session synchronization, and database integration. The JWT callback is the **core of our dual Web2/Web3 authentication system**, managing token state across Google OAuth, Internet Identity, and account linking flows.

## 🔧 **JWT Callback Configuration**

### **Available Parameters**

```typescript
async jwt({ token, user, account, profile, session, trigger }) {
  // Our implementation uses:
  // ✅ token - JWT token state management
  // ✅ user - User data from authentication
  // ✅ account - Provider account information
  // ✅ session - Session data for updates
  // ✅ trigger - NextAuth trigger type
  // ❌ profile - Not used (OAuth profile data)
}
```

### **Why We Don't Use All Parameters**

- **`profile`**: Not used because we handle profile data in provider-specific `profile()` functions
- **`user`**: Used for initial user data and business user ID lookup
- **`account`**: Critical for provider detection and fresh sign-in logic
- **`token`**: Core JWT state management
- **`session`**: Essential for session updates and II co-auth
- **`trigger`**: Key for distinguishing between sign-in and update operations

## 🔍 **JWT Callback Logic Flow**

### **1. Fresh Sign-In Detection**

```typescript
if (trigger === "signIn" && account) {
  // Set base session provider (authoritative on each fresh sign-in)
  token.loginProvider = account.provider;

  // Clear any existing II co-auth flags on fresh sign-in
  if (account.provider !== "internet-identity") {
    delete token.activeIcPrincipal;
    delete token.activeIcPrincipalAssertedAt;
  }

  // One-time fetch of linkedIcPrincipal from DB (avoid first-pass race)
  if (!token.linkedIcPrincipal) {
    // Database lookup for existing II account
  }
}
```

### **2. Session Update Handling**

```typescript
if (trigger === "update" && session) {
  // Set co-auth when provided
  if ((session as any).activeIcPrincipal) {
    token.activeIcPrincipal = (session as any).activeIcPrincipal as string;
    token.activeIcPrincipalAssertedAt = Date.now();
  }

  // Clear co-auth when explicitly requested
  if ((session as any).clearActiveIc === true) {
    delete token.activeIcPrincipal;
    delete token.activeIcPrincipalAssertedAt;
  }
}
```

### **3. Standard Token Updates**

```typescript
if (account?.access_token) {
  token.accessToken = account.access_token;
}

if (user?.role) {
  token.role = user.role;
}
```

### **4. Business User ID Resolution**

```typescript
if (user?.id && !token.businessUserId) {
  // Database lookup for business user ID
  const allUser = await db.query.allUsers.findFirst({
    where: (allUsers, { eq }) => eq(allUsers.userId, user.id!),
    columns: { id: true },
  });
  if (allUser?.id) {
    token.businessUserId = allUser.id;
  }
}
```

## 🎯 **Key JWT Callback Branches**

### **Primary Conditional Branches**

1. **`if (trigger === 'signIn' && account)`** - Fresh sign-in detection
2. **`if (trigger === 'update' && session)`** - Session update handling
3. **`if (account?.access_token)`** - OAuth access token management
4. **`if (user?.role)`** - User role assignment
5. **`if (user?.id && !token.businessUserId)`** - Business user ID resolution

### **Nested Conditional Branches**

#### **Within Fresh Sign-In:**

- **`if (account.provider !== 'internet-identity')`** - Clear II co-auth for non-II providers
- **`if (!token.linkedIcPrincipal)`** - Database lookup for existing II account

#### **Within Session Update:**

- **`if ((session as any).activeIcPrincipal)`** - Set II co-auth state
- **`if ((session as any).clearActiveIc === true)`** - Clear II co-auth state

#### **Within Business User ID:**

- **`if (allUser?.id)`** - Set business user ID from database

## 🔄 **State Management Flow**

### **Token State Evolution**

**Initial State:**

```typescript
token = {
  sub: "user-id",
  role: undefined,
  businessUserId: undefined,
  loginProvider: undefined,
  activeIcPrincipal: undefined,
  activeIcPrincipalAssertedAt: undefined,
  linkedIcPrincipal: undefined,
};
```

**After Google Sign-In:**

```typescript
token = {
  sub: "user-id",
  role: "user",
  businessUserId: "business-id",
  loginProvider: "google",
  activeIcPrincipal: undefined,
  activeIcPrincipalAssertedAt: undefined,
  linkedIcPrincipal: undefined,
};
```

**After II Co-Auth:**

```typescript
token = {
  sub: "user-id",
  role: "user",
  businessUserId: "business-id",
  loginProvider: "google",
  activeIcPrincipal: "ii-principal",
  activeIcPrincipalAssertedAt: 1699123456,
  linkedIcPrincipal: "ii-principal",
};
```

## 🎯 **Critical Design Decisions**

### **1. Provider Detection**

- **`account.provider`** determines authentication method
- **`'google'`** vs **`'internet-identity'`** triggers different logic paths

### **2. Session Update Mechanism**

- **`trigger === 'update'`** handles II co-auth activation
- **`session.activeIcPrincipal`** enables II co-authentication
- **`session.clearActiveIc`** disables II co-authentication

### **3. Database Integration**

- **`linkedIcPrincipal`** fetched from database on fresh sign-in
- **`businessUserId`** resolved from `allUsers` table
- **Race condition prevention** with `!token.linkedIcPrincipal` check

### **4. State Synchronization**

- **JWT token** serves as single source of truth
- **Session callback** exposes token data to components
- **Database consistency** maintained through account linking

## 📋 **Summary**

Our JWT callback is a **sophisticated state machine** that:

1. **Manages dual authentication** (Google + Internet Identity)
2. **Handles session synchronization** across Web2/Web3 systems
3. **Integrates database state** with JWT token state
4. **Provides co-authentication** capabilities
5. **Maintains security** through proper state management

The complexity reflects the **real-world requirements** of bridging traditional Web2 authentication with Web3 identity systems while maintaining session consistency and security.
