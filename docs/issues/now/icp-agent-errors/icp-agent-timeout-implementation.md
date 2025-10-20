# ICP Network Availability Detection Issue

## Problem Statement

The ICP agent is still experiencing ProtocolError crashes despite implementing `backendActorSafe`. The error occurs because `HttpAgent.create()` attempts to connect to an unavailable ICP network, causing the app to crash instead of gracefully degrading.

## Current Error

```
Console ProtocolError
HTTP request failed:
  Status: 500 (Internal Server Error)
  Headers: [["connection","keep-alive"],["date","Tue, 14 Oct 2025 19:47:19 GMT"],["keep-alive","timeout=5"],["transfer-encoding","chunked"]]
  Body: Internal Server Error

at ProtocolError.fromCode
at HttpAgent.requestAndRetry
at async HttpAgent.status
at async HttpAgent.fetchRootKey
```

## Current Implementation

### Agent Creation with Timeout

```typescript
// src/ic/agent.ts
export function createAgent(identity?: Identity): Promise<HttpAgent> {
  const key = identity ? identity.getPrincipal().toText() : "anon";
  if (!agentCache.has(key)) {
    const created = (async () => {
      try {
        // Add timeout to prevent hanging
        const timeoutPromise = new Promise<never>((_, reject) => {
          setTimeout(() => reject(new Error("ICP connection timeout")), 8000);
        });

        const agentPromise = HttpAgent.create({ host, identity });
        const agent = await Promise.race([agentPromise, timeoutPromise]);

        if (process.env.NEXT_PUBLIC_DFX_NETWORK !== "ic") {
          try {
            await agent.fetchRootKey();
          } catch (_fetchError) {
            fatLogger.warn("⚠️ ICP replica not available. ICP features will be disabled.", "fe");
            fatLogger.warn("To enable ICP features, run: dfx start", "fe");
            // Don't throw - let the app continue without ICP functionality
          }
        }
        return agent;
      } catch (e) {
        agentCache.delete(key);
        fatLogger.warn("ICP agent creation failed:", "fe", { error: e });
        throw e;
      }
    })();

    agentCache.set(key, created);
  }
  return agentCache.get(key)!;
}
```

### Safe Backend Actor

```typescript
// src/ic/backend.ts
export type IcpInit = { status: "connected"; actor: BackendActor } | { status: "offline"; reason: string };

export async function backendActorSafe(identity?: Identity): Promise<IcpInit> {
  try {
    const agent = await createAgent(identity);
    const actor = makeActor(backendIDL, BACKEND_CANISTER_ID, agent) as BackendActor;
    return { status: "connected", actor };
  } catch (e: unknown) {
    console.warn("[ICP] init failed; Neon-only mode:", e);
    return { status: "offline", reason: e instanceof Error ? e.message : "unknown" };
  }
}
```

### Service Integration

```typescript
// src/services/memories.ts
const fetchMemoriesFromICP = async (page: number): Promise<FetchMemoriesResult> => {
  try {
    const { backendActorSafe } = await import("@/ic/backend");
    const { getAuthClient } = await import("@/ic/ii");

    const authClient = await getAuthClient();
    if (!authClient.isAuthenticated()) {
      throw new Error("Please connect your Internet Identity to fetch ICP memories");
    }

    const identity = authClient.getIdentity();
    const icpResult = await backendActorSafe(identity);

    // Handle ICP connection failure gracefully
    if (icpResult.status === "offline") {
      fatLogger.warn("ICP connection failed:", "be", {
        error: icpResult.reason,
      });

      // Return empty results when ICP is unavailable
      return {
        memories: [],
        hasMore: false,
      };
    }

    const actor = icpResult.actor;
    // ... rest of ICP logic
  } catch (error) {
    fatLogger.warn("ICP connection failed:", "be", {
      error: error instanceof Error ? error.message : String(error),
    });

    return {
      memories: [],
      hasMore: false,
    };
  }
};
```

## Issues Identified

1. **Network Detection**: We're not checking if ICP network is available before attempting to create the agent.

2. **Error Propagation**: `HttpAgent.create()` fails with 500 error from ICP network, which happens before our error handling can catch it.

3. **No Health Check**: We attempt agent creation without verifying ICP network is reachable.

## Questions for ICP Expert

1. **Network Health Check**: What's the best way to detect if ICP network is available before attempting `HttpAgent.create()`?

2. **Health Endpoints**: Are there specific ICP endpoints we can ping to check network status?

3. **Pre-connection Detection**: How can we avoid calling `HttpAgent.create()` when we know ICP is unavailable?

4. **Circuit Breaker**: Should we implement a pattern that stops attempting ICP connections after repeated failures?

5. **Fallback Strategy**: What's the best practice for completely disabling ICP features when the network is unavailable?

## Proposed Solutions

### Option 1: Network Health Check Before Agent Creation

```typescript
const isICPAvailable = async (): Promise<boolean> => {
  try {
    // Check if ICP network is reachable
    const response = await fetch(`${host}/api/v2/status`, {
      method: "GET",
      signal: AbortSignal.timeout(5000),
    });
    return response.ok;
  } catch {
    return false;
  }
};

const createAgentWithHealthCheck = async (identity?: Identity): Promise<HttpAgent> => {
  // Check network availability BEFORE attempting agent creation
  const isAvailable = await isICPAvailable();
  if (!isAvailable) {
    throw new Error("ICP network is not available");
  }

  // Only create agent if network is confirmed available
  return HttpAgent.create({ host, identity });
};
```

### Option 2: Circuit Breaker Pattern

```typescript
class ICPCircuitBreaker {
  private failures = 0;
  private lastFailureTime = 0;
  private readonly threshold = 3;
  private readonly timeout = 60000; // 1 minute

  async call<T>(operation: () => Promise<T>): Promise<T> {
    if (this.isOpen()) {
      throw new Error("ICP circuit breaker is open");
    }

    try {
      const result = await operation();
      this.onSuccess();
      return result;
    } catch (error) {
      this.onFailure();
      throw error;
    }
  }

  private isOpen(): boolean {
    return this.failures >= this.threshold && Date.now() - this.lastFailureTime < this.timeout;
  }

  private onSuccess(): void {
    this.failures = 0;
    this.lastFailureTime = 0;
  }

  private onFailure(): void {
    this.failures++;
    this.lastFailureTime = Date.now();
  }
}
```

### Option 3: Combined Approach

```typescript
const createAgentSafely = async (identity?: Identity): Promise<HttpAgent> => {
  // 1. Check network health first
  const isHealthy = await isICPAvailable();
  if (!isHealthy) {
    throw new Error("ICP network is not available");
  }

  // 2. Use circuit breaker to prevent repeated failures
  const circuitBreaker = new ICPCircuitBreaker();

  return circuitBreaker.call(async () => {
    // 3. Only create agent if network is healthy and circuit is closed
    return HttpAgent.create({ host, identity });
  });
};
```

## Expected Outcome

- App should never crash due to ICP connection failures
- Users should see graceful degradation to Neon-only mode
- ICP features should be disabled when network is unavailable
- Proper error logging for debugging
- Automatic recovery when ICP becomes available

## Priority

**High** - This is blocking user experience and causing app crashes in production.

## Related Issues

- Client-server boundary violations
- Database connection issues
- Upload service failures
- Authentication problems

## Expert Analysis & Solution

The expert identified the **root cause**: The 500 error is coming from `fetchRootKey()` (which internally calls `status`), not from `HttpAgent.create()`. The issue is that we're calling `fetchRootKey()` against mainnet hosts when we shouldn't.

### Root Cause Analysis

1. **Wrong Detection**: Our guard was only `DFX_NETWORK !== 'ic'` but if `host` points to a mainnet boundary, we still call `fetchRootKey()` against mainnet
2. **Network Call**: `fetchRootKey()` calls `status` endpoint which returns 500 when ICP is unavailable
3. **Cache Poisoning**: Failed promises stay cached forever

### Expert's 4-Part Solution

#### 1. Strong Mainnet/Local Detection

```typescript
// src/ic/env.ts
export const HOST =
  process.env.NEXT_PUBLIC_IC_HOST ??
  (process.env.NEXT_PUBLIC_DFX_NETWORK === "ic" ? "https://icp-api.io" : "http://127.0.0.1:4943");

export const IS_MAINNET =
  process.env.NEXT_PUBLIC_DFX_NETWORK === "ic" || /(^https:\/\/)?(icp-api\.io|ic0\.app|icp0\.io)\b/.test(HOST);

export const IS_LOCAL = /^https?:\/\/(127\.0\.0\.1|localhost)(:\d+)?$/.test(HOST);
```

**⚠️ Issue with Expert's Detection Logic**: The mainnet detection is redundant because:

- `IS_MAINNET` checks both `NEXT_PUBLIC_DFX_NETWORK === "ic"` AND host pattern
- But `HOST` is already determined by `NEXT_PUBLIC_DFX_NETWORK === "ic"`
- So if `NEXT_PUBLIC_DFX_NETWORK === "ic"`, then `HOST` will be `"https://icp-api.io"`
- The host pattern check `/(^https:\/\/)?(icp-api\.io|ic0\.app|icp0\.io)\b/.test(HOST)` is redundant

**Simplified Detection**:

```typescript
// src/ic/env.ts
export const HOST =
  process.env.NEXT_PUBLIC_IC_HOST ??
  (process.env.NEXT_PUBLIC_DFX_NETWORK === "ic" ? "https://icp-api.io" : "http://127.0.0.1:4943");

export const IS_MAINNET = process.env.NEXT_PUBLIC_DFX_NETWORK === "ic";
export const IS_LOCAL = /^https?:\/\/(127\.0\.0\.1|localhost)(:\d+)?$/.test(HOST);
```

**Why the expert's version exists**: It's defensive programming for cases where:

- Environment variables are misconfigured
- Someone manually sets `NEXT_PUBLIC_IC_HOST` to a mainnet URL
- The host is overridden to point to mainnet

**Recommendation**: Use the simplified version unless you need the extra safety checks.

#### 2. Preflight Health Check

```typescript
// src/ic/health.ts
"use client";

import { HOST } from "./env";

const HEALTH_TIMEOUT_MS = 4_000;

// Minimal GET to boundary node status
export async function isIcpAvailable(): Promise<boolean> {
  const ctrl = new AbortController();
  const id = setTimeout(() => ctrl.abort(), HEALTH_TIMEOUT_MS);
  try {
    const res = await fetch(`${HOST}/api/v2/status`, {
      method: "GET",
      signal: ctrl.signal,
      cache: "no-store",
    });
    return res.ok; // 200 expected
  } catch {
    return false;
  } finally {
    clearTimeout(id);
  }
}
```

#### 3. Safe Agent Creation

```typescript
// src/ic/agent.ts
"use client";

import { HttpAgent, type Identity } from "@dfinity/agent";
import { HOST, IS_MAINNET, IS_LOCAL } from "./env";
import { fatLogger } from "@/lib/logger";

const AGENT_TIMEOUT_MS = 8_000;
const agentCache = new Map<string, Promise<HttpAgent>>();

function withTimeout<T>(p: Promise<T>, ms = AGENT_TIMEOUT_MS) {
  return new Promise<T>((resolve, reject) => {
    const t = setTimeout(() => reject(new Error("ICP connection timeout")), ms);
    p.then(
      (v) => {
        clearTimeout(t);
        resolve(v);
      },
      (e) => {
        clearTimeout(t);
        reject(e);
      }
    );
  });
}

export function createAgent(identity?: Identity): Promise<HttpAgent> {
  const key = identity ? identity.getPrincipal().toText() : "anon";
  const existing = agentCache.get(key);
  if (existing) return existing;

  const created = (async () => {
    try {
      const agent = await withTimeout(HttpAgent.create({ host: HOST, identity }));
      // Only on a real local replica try fetchRootKey — and swallow any error.
      if (IS_LOCAL && !IS_MAINNET) {
        try {
          await withTimeout(agent.fetchRootKey());
        } catch (e) {
          fatLogger.warn("⚠️ Local replica not available; ICP features disabled.", "fe");
        }
      }
      return agent;
    } catch (e) {
      // prevent cache poisoning on failure
      agentCache.delete(key);
      throw e;
    }
  })();

  agentCache.set(key, created);
  return created;
}

export function clearAgentCache() {
  agentCache.clear();
}
```

#### 4. Circuit Breaker Pattern

```typescript
// src/ic/circuit.ts
let failures = 0;
let lastFail = 0;
const THRESHOLD = 3;
const COOLDOWN = 60_000;

export function circuitOpen() {
  return failures >= THRESHOLD && Date.now() - lastFail < COOLDOWN;
}
export function noteSuccess() {
  failures = 0;
  lastFail = 0;
}
export function noteFailure() {
  failures++;
  lastFail = Date.now();
}
```

#### 5. Updated backendActorSafe

```typescript
// src/ic/backend.ts
"use client";

import { isIcpAvailable } from "./health";
import { createAgent } from "./agent";
import { circuitOpen, noteFailure, noteSuccess } from "./circuit";
// ... other imports

export async function backendActorSafe(identity?: Identity): Promise<IcpInit> {
  if (circuitOpen()) return { status: "offline", reason: "circuit-open" };

  const ok = await isIcpAvailable();
  if (!ok) {
    noteFailure();
    return { status: "offline", reason: "boundary-unavailable" };
  }

  try {
    const agent = await createAgent(identity);
    const actor = makeActor(backendIDL, BACKEND_CANISTER_ID, agent) as BackendActor;
    noteSuccess();
    return { status: "connected", actor };
  } catch (e: any) {
    noteFailure();
    console.warn("[ICP] init failed; Neon-only mode:", e?.message ?? e);
    return { status: "offline", reason: e?.message ?? "unknown" };
  }
}
```

### Key Insights from Expert

1. **Health Endpoint**: `/api/v2/status` is the canonical boundary status route
2. **Preflight Check**: Do health check BEFORE creating agent to avoid `fetchRootKey()` calls
3. **Strong Detection**: Use both env AND host-based detection to prevent mainnet `fetchRootKey()` calls
4. **Cache Poisoning**: Remove failed promises from cache to prevent permanent failures
5. **Circuit Breaker**: Prevent hammering sick boundaries with cooldown semantics

### Expected Outcome

With this solution:

- **No more ProtocolError crashes** - health check prevents agent creation when ICP is down
- **No mainnet fetchRootKey calls** - strong detection prevents calls to mainnet
- **Graceful degradation** - app continues with Neon-only features
- **Automatic recovery** - circuit breaker allows retry after cooldown
- **Performance** - preflight check is fast (4s timeout) and prevents expensive operations

## Implementation Plan

### Phase 1: Core Infrastructure (Priority: High)

- [ ] **Step 1**: Create `src/ic/env.ts` with simplified mainnet/local detection

#### Step 1 Discussion: Simplified Environment Detection

**What we're changing from the expert's version:**

- **Remove redundant host pattern checking** - since `HOST` is already determined by `NEXT_PUBLIC_DFX_NETWORK`
- **Keep it simple** - just use environment variable for mainnet detection
- **Add local detection** - for `fetchRootKey()` safety

**Implementation:**

```typescript
// src/ic/env.ts
export const HOST =
  process.env.NEXT_PUBLIC_IC_HOST ??
  (process.env.NEXT_PUBLIC_DFX_NETWORK === "ic" ? "https://icp-api.io" : "http://127.0.0.1:4943");

export const IS_MAINNET = process.env.NEXT_PUBLIC_DFX_NETWORK === "ic";
export const IS_LOCAL = /^https?:\/\/(127\.0\.0\.1|localhost)(:\d+)?$/.test(HOST);
```

**Why this is better:**

- **Simpler logic** - no redundant checks
- **Still safe** - `IS_LOCAL` prevents `fetchRootKey()` on mainnet
- **Maintainable** - easier to understand and debug
- **Reliable** - based on environment variables we control

**Key insight:** The expert's host pattern checking was defensive programming, but since we control the environment variables, the simplified version is sufficient and cleaner.

- [ ] **Step 2**: Create `src/ic/health.ts` with `/api/v2/status` preflight check

#### Step 2 Discussion: Health Check with Community Status Endpoint

**What we're implementing:**

- **Preflight health check** using `/api/v2/status` endpoint
- **Timeout protection** with AbortController (4 seconds)
- **Fast failure** - if status check fails, skip all ICP operations

**Implementation:**

```typescript
// src/ic/health.ts
"use client";
import { HOST } from "./env";

export async function isIcpAvailable(timeoutMs = 4000): Promise<boolean> {
  const ctrl = new AbortController();
  const id = setTimeout(() => ctrl.abort(), timeoutMs);
  try {
    const res = await fetch(`${HOST}/api/v2/status`, {
      method: "GET",
      signal: ctrl.signal,
      cache: "no-store",
    });
    return res.ok;
  } catch {
    return false;
  } finally {
    clearTimeout(id);
  }
}
```

**Why this approach:**

- **Community standard** - `/api/v2/status` is widely used in ICP community
- **Fast and lightweight** - just a simple GET request
- **Timeout protection** - prevents hanging on unresponsive networks
- **Prevents crashes** - gates all ICP operations behind this check

**Key insight:** Even though it's not "official", the `/api/v2/status` endpoint is the community-standard way to check ICP boundary health. It's battle-tested and reliable in practice.

**How it prevents the crash:**

- **Before any agent creation** - we check if ICP is available
- **If status check fails** - we never call `createAgent()` or `fetchRootKey()`
- **Immediate fallback** - services switch to Neon-only mode without crashes
- [ ] **Step 3**: Update `src/ic/agent.ts` to remove network calls (simplified)

#### Step 3 Discussion: Simplified Agent (No Network Calls)

**What we're changing:**

- **Remove all network calls** from `createAgent()` function
- **Remove `fetchRootKey()`** from agent creation
- **Remove timeout logic** - no network calls means no timeouts needed
- **Keep caching** - but make it simpler

**Current problematic code:**

```typescript
// OLD - causes crashes
export function createAgent(identity?: Identity): Promise<HttpAgent> {
  // ... complex timeout logic
  const agent = await HttpAgent.create({ host, identity });
  if (process.env.NEXT_PUBLIC_DFX_NETWORK !== "ic") {
    await agent.fetchRootKey(); // 💥 CRASH POINT
  }
  return agent;
}
```

**New simplified code:**

```typescript
// NEW - no network calls
export function createAgent(identity?: Identity): Promise<HttpAgent> {
  const key = identity ? identity.getPrincipal().toText() : "anon";
  const cached = agentCache.get(key);
  if (cached) return cached;

  const p = HttpAgent.create({ host: HOST, identity }).catch((e) => {
    agentCache.delete(key);
    throw e;
  });

  agentCache.set(key, p);
  return p;
}
```

**Why this fixes the crash:**

- **`HttpAgent.create()` doesn't touch the network** - it just creates the agent object
- **No `fetchRootKey()` call** - this was the source of the 500 error
- **No timeout needed** - since there are no network calls
- **Still caches properly** - but removes failed entries to prevent cache poisoning

**Key insight:** The tech lead confirmed that `HttpAgent.create()` itself doesn't hit the network. The first network touch is `fetchRootKey()` or a canister call. By removing `fetchRootKey()` from agent creation, we eliminate the crash point entirely.

**What happens next:**

- **Agent creation is now safe** - no network calls, no crashes
- **`fetchRootKey()` will be called later** - in `backendActorSafe()` after health check
- **Only on local networks** - with proper error handling
- [ ] **Step 4**: Update `src/ic/backend.ts` with lazy `fetchRootKey()` and preflight gate
- [ ] **Step 5**: Update `src/services/memories.ts` to use new `backendActorSafe`

### Phase 2: Integration & Testing (Priority: High)

- [ ] **Step 6**: Update `src/services/memories.ts` to use new `backendActorSafe`
- [ ] **Step 7**: Test with ICP network down scenarios
- [ ] **Step 8**: Test with local ICP replica down
- [ ] **Step 9**: Test circuit breaker behavior
- [ ] **Step 10**: Verify graceful degradation to Neon-only mode

### Phase 3: User Experience (Priority: Medium)

- [ ] **Step 11**: Add ICP status indicator in UI
- [ ] **Step 12**: Show toast notifications for ICP unavailability
- [ ] **Step 13**: Add retry mechanism for ICP recovery
- [ ] **Step 14**: Update error messages to be user-friendly

### Phase 4: Advanced Features (Priority: Low)

- [ ] **Step 15**: Add server-side health proxy (`/api/health/icp`) to avoid CORS

#### Step 15 Discussion: Server-side Health Proxy

**What we're implementing:**

- **Next.js API route** that checks ICP health server-side
- **Avoids CORS issues** - server-side fetch doesn't have CORS restrictions
- **Centralized failover logic** - all health checks go through one endpoint

**Implementation:**

```typescript
// /api/health/icp - Next.js API route
export async function GET() {
  try {
    const res = await fetch(`${HOST}/api/v2/status`, {
      method: "GET",
      signal: AbortSignal.timeout(5000),
    });
    return Response.json({ available: res.ok });
  } catch {
    return Response.json({ available: false });
  }
}
```

- [ ] **Step 16**: Implement boundary failover list (try multiple hosts)

#### Step 16 Discussion: Boundary Failover

**What we're implementing:**

- **Multiple boundary hosts** - try different ICP endpoints
- **Automatic failover** - if one host fails, try the next
- **Better availability** - multiple fallback options

**Implementation:**

```typescript
const BOUNDARY_HOSTS = ["https://icp-api.io", "https://ic0.app", "https://icp0.io"];

export async function isIcpAvailable(): Promise<boolean> {
  for (const host of BOUNDARY_HOSTS) {
    try {
      const res = await fetch(`${host}/api/v2/status`, {
        method: "GET",
        signal: AbortSignal.timeout(4000),
      });
      if (res.ok) return true;
    } catch {
      continue; // Try next host
    }
  }
  return false;
}
```

- [ ] **Step 17**: Add circuit breaker pattern to prevent hammering sick boundaries

#### Step 17 Discussion: Circuit Breaker Pattern

**What we're implementing:**

- **Failure tracking** - count consecutive failures
- **Cooldown period** - stop trying after threshold reached
- **Automatic recovery** - retry after cooldown expires

**Implementation:**

```typescript
// src/ic/circuit.ts
let failures = 0;
let lastFail = 0;
const THRESHOLD = 3;
const COOLDOWN = 60_000; // 1 minute

export function circuitOpen(): boolean {
  return failures >= THRESHOLD && Date.now() - lastFail < COOLDOWN;
}

export function noteSuccess(): void {
  failures = 0;
  lastFail = 0;
}

export function noteFailure(): void {
  failures++;
  lastFail = Date.now();
}
```

- [ ] **Step 18**: Add logging and metrics for ICP health checks
- [ ] **Step 19**: Add alerting for repeated ICP failures
- [ ] **Step 20**: Optimize health check frequency and caching

## Implementation Checklist

### ✅ **Ready to Implement**

- [ ] Create environment detection module
- [ ] Create health check module
- [ ] Create circuit breaker module
- [ ] Update agent creation logic
- [ ] Update backend actor logic
- [ ] Update service integration
- [ ] Test all scenarios
- [ ] Add user feedback

### 🔄 **Dependencies**

- Each step builds on the previous one
- Phase 1 must be completed before Phase 2
- Testing can be done incrementally

### ⏱️ **Estimated Timeline**

- **Phase 1**: 2-3 hours (core infrastructure)
- **Phase 2**: 1-2 hours (integration & testing)
- **Phase 3**: 1 hour (user experience)
- **Phase 4**: 1 hour (monitoring)

**Total**: 5-7 hours for complete implementation

## Final Implementation (Tech Lead's Refined Solution)

The tech lead confirmed our approach and provided the final implementation details:

### Key Insights from Tech Lead:

1. **`HttpAgent.create()` doesn't touch the network** - the crash is from the first network call (`fetchRootKey()` or canister query)
2. **No "official" health check** - but `/api/v2/status` is the community-standard preflight
3. **Move `fetchRootKey()` out of `createAgent`** - call it lazily only after successful preflight
4. **Preflight before any agent work** - gate ICP usage with `/api/v2/status` check

### Final Implementation:

#### 1. Simplified Agent (No Network Calls)

```typescript
// src/ic/agent.ts
"use client";
import { HttpAgent, type Identity } from "@dfinity/agent";
import { HOST } from "./env";

const agentCache = new Map<string, Promise<HttpAgent>>();

export function createAgent(identity?: Identity): Promise<HttpAgent> {
  const key = identity ? identity.getPrincipal().toText() : "anon";
  const cached = agentCache.get(key);
  if (cached) return cached;

  const p = HttpAgent.create({ host: HOST, identity }).catch((e) => {
    agentCache.delete(key);
    throw e;
  });

  agentCache.set(key, p);
  return p;
}

export function clearAgentCache() {
  agentCache.clear();
}
```

#### 2. Health Check (Preflight Gate)

```typescript
// src/ic/health.ts
"use client";
import { HOST } from "./env";

export async function isIcpAvailable(timeoutMs = 4000): Promise<boolean> {
  const ctrl = new AbortController();
  const id = setTimeout(() => ctrl.abort(), timeoutMs);
  try {
    const res = await fetch(`${HOST}/api/v2/status`, {
      method: "GET",
      signal: ctrl.signal,
      cache: "no-store",
    });
    return res.ok;
  } catch {
    return false;
  } finally {
    clearTimeout(id);
  }
}
```

#### 3. Safe Backend Actor (Lazy fetchRootKey)

```typescript
// src/ic/backend.ts
"use client";
import { isIcpAvailable } from "./health";
import { createAgent } from "./agent";
import { IS_LOCAL } from "./env";
import { idlFactory as backendIDL } from "@/ic/declarations/backend/backend.did.js";
import { canisterId as BACKEND_CANISTER_ID } from "@/ic/declarations/backend";
import { makeActor } from "./actor-factory";
import type { Identity, ActorSubclass } from "@dfinity/agent";
import type { _SERVICE as Backend } from "@/ic/declarations/backend/backend.did";

export type BackendActor = ActorSubclass<Backend>;
export type IcpInit = { status: "connected"; actor: BackendActor } | { status: "offline"; reason: string };

export async function backendActorSafe(identity?: Identity): Promise<IcpInit> {
  // 1) Preflight
  if (!(await isIcpAvailable())) return { status: "offline", reason: "boundary-unavailable" };

  try {
    // 2) Build agent (no network yet)
    const agent = await createAgent(identity);

    // 3) Local-only root key, guarded by preflight
    if (IS_LOCAL) {
      try {
        await agent.fetchRootKey();
      } catch {
        /* swallow → degrade later */
      }
    }

    // 4) Actor
    const actor = makeActor(backendIDL, BACKEND_CANISTER_ID, agent) as BackendActor;
    return { status: "connected", actor };
  } catch (e: any) {
    return { status: "offline", reason: e?.message ?? "unknown" };
  }
}
```

### Why This Fixes the Crash:

- **`createAgent` can no longer be the first place that triggers the network**
- **`fetchRootKey()` is never called unless the boundary just said "I'm OK" and you're on localhost**
- **If preflight fails, you never create the actor at all** - services immediately switch to Neon-only mode

### Optional Extras (Nice but not required):

- Server-side health proxy (`/api/health/icp`) to avoid client CORS
- Boundary failover list (try `icp-api.io`, `ic0.app`, `icp0.io` in order)
- Circuit breaker to skip preflight for 60s after N failures

## Next Steps

1. **Immediate**: Implement the final solution above
2. **Short-term**: Test with ICP network down scenarios
3. **Long-term**: Add optional extras (server-side proxy, failover, circuit breaker)

You’re very close. The stack you pasted (“…at HttpAgent.status → …fetchRootKey”) shows the 500 is coming from `fetchRootKey()` (which internally calls `status`), not from `HttpAgent.create()`. In current agent versions, `HttpAgent.create()` itself doesn’t hit the network; the first network touch is `fetchRootKey()` or your first canister call.

So the fix is two-part:

1. never run `fetchRootKey()` unless you’re on a real local replica, and
2. short-circuit ICP usage before you even try to build an agent when the boundary is sick.

Below is a compact, battle-tested setup that addresses your three issues (detection, propagation, health check) without overcomplicating the agent.

---

## 0) Strong mainnet/local detection

Your guard was only `DFX_NETWORK !== 'ic'`. Add a host check so you never call `fetchRootKey()` against mainnet-like hosts even if envs are mis-set.

```ts
// src/ic/env.ts
export const HOST =
  process.env.NEXT_PUBLIC_IC_HOST ??
  (process.env.NEXT_PUBLIC_DFX_NETWORK === "ic" ? "https://icp-api.io" : "http://127.0.0.1:4943");

export const IS_MAINNET =
  process.env.NEXT_PUBLIC_DFX_NETWORK === "ic" || /(^https:\/\/)?(icp-api\.io|ic0\.app|icp0\.io)\b/.test(HOST);

export const IS_LOCAL = /^https?:\/\/(127\.0\.0\.1|localhost)(:\d+)?$/.test(HOST);
```

---

## 1) Preflight health check (cheap, time-bounded)

Do this before creating any agent. If unhealthy, return “offline” and skip `createAgent()` entirely.

```ts
// src/ic/health.ts
"use client";

import { HOST } from "./env";

const HEALTH_TIMEOUT_MS = 4_000;

// Minimal GET to boundary node status
export async function isIcpAvailable(): Promise<boolean> {
  const ctrl = new AbortController();
  const id = setTimeout(() => ctrl.abort(), HEALTH_TIMEOUT_MS);
  try {
    const res = await fetch(`${HOST}/api/v2/status`, {
      method: "GET",
      signal: ctrl.signal,
      cache: "no-store",
    });
    return res.ok; // 200 expected
  } catch {
    return false;
  } finally {
    clearTimeout(id);
  }
}
```

(Yes, `/api/v2/status` is the right lightweight probe. If you ever host behind a custom gateway that blocks CORS, you can mirror a tiny health endpoint in your Next API and let the server do the probe.)

---

## 2) Agent that can’t poison the cache and won’t call root key on mainnet

```ts
// src/ic/agent.ts
"use client";

import { HttpAgent, type Identity } from "@dfinity/agent";
import { HOST, IS_MAINNET, IS_LOCAL } from "./env";
import { fatLogger } from "@/lib/logger";

const AGENT_TIMEOUT_MS = 8_000;
const agentCache = new Map<string, Promise<HttpAgent>>();

function withTimeout<T>(p: Promise<T>, ms = AGENT_TIMEOUT_MS) {
  return new Promise<T>((resolve, reject) => {
    const t = setTimeout(() => reject(new Error("ICP connection timeout")), ms);
    p.then(
      (v) => {
        clearTimeout(t);
        resolve(v);
      },
      (e) => {
        clearTimeout(t);
        reject(e);
      }
    );
  });
}

export function createAgent(identity?: Identity): Promise<HttpAgent> {
  const key = identity ? identity.getPrincipal().toText() : "anon";
  const existing = agentCache.get(key);
  if (existing) return existing;

  const created = (async () => {
    try {
      const agent = await withTimeout(HttpAgent.create({ host: HOST, identity }));
      // Only on a real local replica try fetchRootKey — and swallow any error.
      if (IS_LOCAL && !IS_MAINNET) {
        try {
          await withTimeout(agent.fetchRootKey());
        } catch (e) {
          fatLogger.warn("⚠️ Local replica not available; ICP features disabled.", "fe");
        }
      }
      return agent;
    } catch (e) {
      // prevent cache poisoning on failure
      agentCache.delete(key);
      throw e;
    }
  })();

  agentCache.set(key, created);
  return created;
}

export function clearAgentCache() {
  agentCache.clear();
}
```

Key points:

- `IS_LOCAL` ensures `fetchRootKey()` is never called against mainnet hosts even if `DFX_NETWORK` is wrong.
- Rejected promise is removed from the cache.

---

## 3) Short-circuit in your safe init (don’t even create an agent if health is down)

```ts
// src/ic/backend.ts
"use client";

import { isIcpAvailable } from "./health";
import { createAgent } from "./agent";
import { idlFactory as backendIDL } from "@/ic/declarations/backend/backend.did.js";
import { canisterId as BACKEND_CANISTER_ID } from "@/ic/declarations/backend";
import { makeActor } from "./actor-factory";
import type { ActorSubclass, Identity } from "@dfinity/agent";
import type { _SERVICE as Backend } from "@/ic/declarations/backend/backend.did";

export type BackendActor = ActorSubclass<Backend>;
export type IcpInit = { status: "connected"; actor: BackendActor } | { status: "offline"; reason: string };

export async function backendActorSafe(identity?: Identity): Promise<IcpInit> {
  // Preflight gate: avoid touching the agent if boundary is down
  const ok = await isIcpAvailable();
  if (!ok) return { status: "offline", reason: "boundary-unavailable" };

  try {
    const agent = await createAgent(identity);
    const actor = makeActor(backendIDL, BACKEND_CANISTER_ID, agent) as BackendActor;
    return { status: "connected", actor };
  } catch (e: any) {
    console.warn("[ICP] init failed; Neon-only mode:", e?.message ?? e);
    return { status: "offline", reason: e?.message ?? "unknown" };
  }
}
```

With this, if `/api/v2/status` returns 500/timeout, you never call `createAgent()` → no `fetchRootKey()` → no crash.

---

## 4) Optional circuit breaker (global)

Prevents hammering a sick boundary and gives you cool-down semantics.

```ts
// src/ic/circuit.ts
let failures = 0;
let lastFail = 0;
const THRESHOLD = 3;
const COOLDOWN = 60_000;

export function circuitOpen() {
  return failures >= THRESHOLD && Date.now() - lastFail < COOLDOWN;
}
export function noteSuccess() {
  failures = 0;
  lastFail = 0;
}
export function noteFailure() {
  failures++;
  lastFail = Date.now();
}
```

Use it inside `backendActorSafe`:

```ts
import { circuitOpen, noteFailure, noteSuccess } from "./circuit";

export async function backendActorSafe(identity?: Identity): Promise<IcpInit> {
  if (circuitOpen()) return { status: "offline", reason: "circuit-open" };

  const ok = await isIcpAvailable();
  if (!ok) {
    noteFailure();
    return { status: "offline", reason: "boundary-unavailable" };
  }

  try {
    const agent = await createAgent(identity);
    const actor = makeActor(backendIDL, BACKEND_CANISTER_ID, agent) as BackendActor;
    noteSuccess();
    return { status: "connected", actor };
  } catch (e: any) {
    noteFailure();
    console.warn("[ICP] init failed; Neon-only mode:", e?.message ?? e);
    return { status: "offline", reason: e?.message ?? "unknown" };
  }
}
```

---

## Answers to your specific questions

1. Best pre-create detection: a fast GET to `${host}/api/v2/status` with a 3–5s timeout (as above).
2. Health endpoint: `/api/v2/status` is the canonical boundary status route.
3. Avoid calling `createAgent()` when down: do the preflight status check; if `false`, bail out before agent creation.
4. Circuit breaker: yes, add a simple one (above) to avoid repeated work during outages.
5. Disabling features: mount a small ICP mode flag in context (e.g., `icp: "connected" | "offline"`). When offline, hide/disable ICP UI and route to Neon logic; show a banner/toast.

---

## Likely root cause in your current code

Your `fetchRootKey()` gate was only env-based. If `NEXT_PUBLIC_DFX_NETWORK !== 'ic'` but `host` points to a mainnet boundary, you’ll call `fetchRootKey()` against mainnet, which calls `status` and can 500 → exactly the stack you’re seeing. The strong `IS_LOCAL`/`IS_MAINNET` checks plus the preflight status gate remove that failure path entirely.

Apply the four blocks above and the app will degrade to Neon-only mode without ever throwing a ProtocolError into React.
