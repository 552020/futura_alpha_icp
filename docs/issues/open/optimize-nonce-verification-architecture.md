# Optimize Nonce Verification Architecture

## 📋 **Issue Summary**

Currently, the Internet Identity authentication flow makes an extra HTTP API call to `/api/ii/verify-nonce` even when running on the server side. This creates unnecessary overhead and can be optimized by calling the canister directly from the `authorize` function in `auth.ts`.

## 🔍 **Current Problem**

### **Inefficient Flow**

```typescript
// Current: auth.ts → /api/ii/verify-nonce → canister
async authorize(credentials) {
  // ... validation logic ...

  // ❌ Extra HTTP call to our own API
  const response = await fetch('/api/ii/verify-nonce', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ nonce })
  });

  if (!response.ok) {
    throw new Error('Nonce verification failed');
  }
}
```

### **Performance Impact**

- **Extra HTTP overhead** for server-side operations
- **Unnecessary network round-trip** to our own API
- **Potential bottleneck** during high authentication load
- **Code duplication** between `authorize` and API endpoint

## 🎯 **Proposed Solution**

### **Extract Shared Verification Logic**

Create `lib/ii/verifyNonce.ts` with shared functions:

```typescript
import { createServerSideActor } from "@/lib/server-actor";
import { db } from "@/lib/db";
import { eq } from "drizzle-orm";
import { iiNonces } from "@/lib/schema";

export async function verifyNonceWithCanister(nonce: string) {
  const actor = await createServerSideActor();
  const nonceResult = (await actor.verify_nonce(nonce)) as { Ok: any } | { Err: any };

  if ("Err" in nonceResult) {
    return { success: false, error: JSON.stringify(nonceResult.Err) };
  }

  return { success: true, principal: nonceResult.Ok.toString() };
}

export async function validateNonceRecord(nonceId: string) {
  const record = await db.query.iiNonces.findFirst({
    where: eq(iiNonces.id, nonceId),
  });

  if (!record) return { ok: false, error: "not-found" };
  if (record.usedAt) return { ok: false, error: "already-used" };
  if (record.expiresAt < new Date()) return { ok: false, error: "expired" };

  return { ok: true, record };
}
```

### **Optimize `authorize` Function**

```typescript
import { validateNonceRecord, verifyNonceWithCanister } from "@/lib/ii/verifyNonce"

async authorize(credentials) {
  const { principal, nonceId, nonce } = credentials ?? {}

  // Local DB validation
  const nonceCheck = await validateNonceRecord(nonceId)
  if (!nonceCheck.ok) throw new Error(`Nonce invalid: ${nonceCheck.error}`)

  // ✅ Direct canister call (no HTTP overhead)
  const proof = await verifyNonceWithCanister(nonce!)
  if (!proof.success) throw new Error("Nonce proof verification failed")

  if (proof.principal !== principal) {
    throw new Error("Principal mismatch")
  }

  // ✅ Nonce is valid and principal matches
  // continue with account/user logic...
}
```

### **Keep API Endpoint for Debugging**

```typescript
import { NextRequest, NextResponse } from "next/server";
import { verifyNonceWithCanister } from "@/lib/ii/verifyNonce";

export async function POST(req: NextRequest) {
  const { nonce } = await req.json();
  if (!nonce || typeof nonce !== "string") {
    return NextResponse.json({ success: false, error: "invalid nonce" }, { status: 400 });
  }

  const result = await verifyNonceWithCanister(nonce);
  return NextResponse.json(result, { status: result.success ? 200 : 401 });
}
```

## ✅ **Benefits**

1. **Performance**: Direct canister calls eliminate HTTP overhead
2. **Code Reuse**: Single source of truth for verification logic
3. **Maintainability**: Changes only need to be made in one place
4. **Debugging**: API endpoint still available for testing
5. **Scalability**: Better performance under high authentication load

## 📁 **Files to Modify**

### **New Files**

- `src/nextjs/src/lib/ii/verifyNonce.ts` - Shared verification logic

### **Modified Files**

- `src/nextjs/auth.ts` - Update `authorize` function to use direct calls
- `src/nextjs/src/app/api/ii/verify-nonce/route.ts` - Refactor to use shared logic

## 🎯 **Implementation Priority**

**High** - This optimization will improve authentication performance and reduce server load, especially important for production environments with high user traffic.

## 🔗 **Related Issues**

- Internet Identity authentication flow analysis
- Linked accounts component II authentication sync
- NextAuth JWT explanation
