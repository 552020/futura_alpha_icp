# NextAuth JWT Explanation

## 📋 **What is JWT?**

**JWT (JSON Web Token)** is a compact, URL-safe token format for securely transmitting information between parties. It consists of three parts separated by dots:

```
header.payload.signature
```

## 🔧 **NextAuth JWT Implementation**

### **JWT Flavor**: `jose` library

NextAuth uses the `jose` library for JWT handling, which provides:

- **ES256** (ECDSA using P-256 and SHA-256) as the default algorithm
- **RS256** (RSA with SHA-256) as fallback
- **HS256** (HMAC with SHA-256) for development

### **Token Structure**

```typescript
// NextAuth JWT payload example
{
  "sub": "user-id-123",           // Subject (user ID)
  "name": "John Doe",             // User name
  "email": "john@example.com",    // User email
  "role": "user",                 // User role
  "loginProvider": "google",      // How they signed in
  "linkedIcPrincipal": "abc123",  // Linked II principal
  "activeIcPrincipal": "def456",  // Active II principal
  "iat": 1699123456,             // Issued at
  "exp": 1699209856,             // Expires at
  "jti": "token-id-789"           // JWT ID
}
```

## 🔄 **How NextAuth JWT Works**

### **1. Token Creation**

```typescript
// When user signs in
const token = await jwt({
  token: {},           // Empty initial token
  account: { ... },    // OAuth account data
  user: { ... }        // User data from database
});
```

### **2. Token Storage**

- **Development**: Stored in cookies (encrypted)
- **Production**: Can use database sessions or JWT-only mode

### **3. Token Validation**

```typescript
// On every request
const session = await getServerSession(authOptions);
// NextAuth automatically validates JWT signature and expiration
```

## 🔐 **Security Features**

### **Token Signing**

- Uses **private key** to sign tokens
- **Public key** used for verification
- **No secret sharing** required

### **Token Expiration**

- **Default**: 30 days
- **Configurable**: Set `maxAge` in auth options
- **Automatic refresh**: Tokens renewed on activity

### **Token Encryption**

- **Cookies encrypted** with `NEXTAUTH_SECRET`
- **Sensitive data protected** in transit
- **CSRF protection** built-in

## 📝 **Key Points**

1. **Stateless**: No server-side session storage needed
2. **Self-contained**: All user data in the token
3. **Secure**: Cryptographically signed and verified
4. **Flexible**: Custom claims and data can be added
5. **Standards-based**: Uses RFC 7519 JWT specification

## 🔗 **Related Files**

- **JWT Callback**: `src/nextjs/auth.ts` (lines 200-300)
- **Session Callback**: `src/nextjs/auth.ts` (lines 300-400)
- **Configuration**: `src/nextjs/auth.ts` (lines 1-50)
