# ICP Connection Error - Next.js Development Warnings

**Priority:** High  
**Type:** Bug  
**Component:** ICP Integration  
**Created:** 2025-01-14  
**Status:** Open

## Problem Description

The application shows scary Next.js development warnings (red "N" indicator) when the ICP network is unavailable or returns a 500 error. The app continues to work, but developers see alarming warnings that make them think the application is broken.

## Error Details

```
## Error Type
Console ProtocolError

## Error Message
HTTP request failed:
  Status: 500 (Internal Server Error)
  Headers: [["connection","keep-alive"],["date","Tue, 14 Oct 2025 18:43:41 GMT"],["keep-alive","timeout=5"],["transfer-encoding","chunked"]]
  Body: Internal Server Error

    at ProtocolError.fromCode (file:///Users/stefano/Documents/Code/Futura/futura_alpha_icp/src/nextjs/.next/static/chunks/ff37e_%40dfinity_agent_lib_esm_c0b76152._.js:172:16)
    at HttpAgent.requestAndRetry (file:///Users/stefano/Documents/Code/Futura/futura_alpha_icp/src/nextjs/.next/static/chunks/ff37e_%40dfinity_agent_lib_esm_c0b76152._.js:3864:403)
    at async HttpAgent.requestAndRetry (file:///Users/stefano/Documents/Code/Futura/futura_alpha_icp/src/nextjs/.next/static/chunks/ff37e_%40dfinity_agent_lib_esm_c0b76152._.js:3858:16)
    at async HttpAgent.requestAndRetry (file:///Users/stefano/Documents/Code/Futura/futura_alpha_icp/src/nextjs/.next/static/chunks/ff37e_%40dfinity_agent_lib_esm_c0b76152._.js:3858:16)
    at async HttpAgent.requestAndRetry (file:///Users/stefano/Documents/Code/Futura/futura_alpha_icp/src/nextjs/.next/static/chunks/ff37e_%40dfinity_agent_lib_esm_c0b76152._.js:3858:16)
    at async HttpAgent.status (file:///Users/stefano/Documents/Code/Futura/futura_alpha_icp/src/nextjs/.next/static/chunks/ff37e_%40dfinity_agent_lib_esm_c0b76152._.js:3418:39)
    at async (file:///Users/stefano/Documents/Code/Futura/futura_alpha_icp/src/nextjs/.next/static/chunks/ff37e_%40dfinity_agent_lib_esm_c0b76152._.js:3432:27)
    at async HttpAgent.fetchRootKey (file:///Users/stefano/Documents/Code/Futura/futura_alpha_icp/src/nextjs/.next/static/chunks/ff37e_%40dfinity_agent_lib_esm_c0b76152._.js:3438:16)
```

## Root Cause Analysis

### The Problem

1. **ICP Agent Initialization**: The `@dfinity/agent` library is trying to connect to the ICP network
2. **Network Failure**: The ICP network returns a 500 Internal Server Error
3. **Next.js Development Warnings**: The error triggers scary red warnings in Next.js development mode
4. **Poor Developer Experience**: Developers think the app is broken when it's actually working fine

### Why This Happens

1. **ICP Network Issues**: The Internet Computer network is experiencing problems
2. **Configuration Issues**: Wrong endpoint URL or network configuration
3. **No Error Handling**: The ICP agent initialization is not wrapped in try-catch
4. **Next.js Development Mode**: Errors trigger scary warnings that alarm developers

### Call Stack Analysis

The error occurs in this specific call chain:

```
Dashboard Component (dataSource: 'icp')
  ↓
fetchMemoriesFromICP() in src/services/memories.ts
  ↓
backendActor() in src/ic/backend.ts
  ↓
createAgent() in src/ic/agent.ts
  ↓
agent.fetchRootKey() ← FAILS HERE with 500 error
```

**Specific Code Path:**

1. **Dashboard**: `src/app/[lang]/dashboard/page.tsx` calls `fetchMemories` with `dataSource: 'icp'`
2. **Service**: `src/services/memories.ts` → `fetchMemoriesFromICP()`
3. **Backend**: `src/ic/backend.ts` → `backendActor(identity)`
4. **Agent**: `src/ic/agent.ts` → `createAgent(identity)` → `agent.fetchRootKey()`

**The Issue**: The `fetchRootKey()` call in `createAgent()` is not wrapped in proper error handling for production ICP network failures. The current error handling only covers local development scenarios, and errors trigger scary Next.js development warnings.

### Current backendActor Implementation

```typescript
"use client";

import { idlFactory as backendIDL } from "@/ic/declarations/backend/backend.did.js";
import { canisterId as BACKEND_CANISTER_ID } from "@/ic/declarations/backend";
import { createAgent } from "./agent";
import { makeActor } from "./actor-factory";
import { Identity } from "@dfinity/agent";
import type { _SERVICE as Backend } from "@/ic/declarations/backend/backend.did";
import type { ActorSubclass } from "@dfinity/agent";

export type BackendActor = ActorSubclass<Backend>;

export async function backendActor(identity?: Identity): Promise<BackendActor> {
  const agent = await createAgent(identity);
  return makeActor(backendIDL, BACKEND_CANISTER_ID, agent);
}
```

**The Problem**: The `backendActor` function calls `createAgent(identity)` without any error handling. When `createAgent` fails (due to ICP network issues), the error triggers scary Next.js development warnings that make developers think the app is broken.

**The Solution**: Add network availability check and error handling in `backendActor` before calling `createAgent` to prevent scary warnings.

## Expert Analysis

Based on ICP expert feedback, the recommended approach is:

### 1. **Wrap Agent Initialization and Calls in Try-Catch**

The ICP JavaScript agent (`@dfinity/agent`) will throw errors (including network and protocol errors) if the network is unavailable or returns a 500 error. Always wrap agent initialization and network calls in `try-catch` blocks to prevent scary Next.js development warnings.

**Key Points:**

- Always wrap `fetchRootKey()` calls in try-catch
- Avoid calling `fetchRootKey()` on mainnet (security risk)
- Only use `fetchRootKey()` in local development with error handling
- Catch errors from agent initialization and canister interactions

### 2. **Timeouts and Retry Logic**

Implement timeouts using `Promise.race` and retry logic to handle transient network issues. This is compatible with the agent's async API.

### 3. **Graceful Degradation and User Feedback**

If the agent fails to connect:

- Inform the user with clear messaging
- Disable ICP-dependent features
- Allow the rest of the app to function
- Avoid triggering scary development warnings

### 4. **Check Network/Endpoint Before Calls (Optional)**

Perform a lightweight call (query to canister or `readState` call) and catch errors to determine network availability before making critical calls.

### 5. **Implementation Strategy**

**Core Principle**: Always expect and handle network errors in ICP integration code to prevent scary development warnings.

**Recommended Pattern:**

```typescript
try {
  const agent = new HttpAgent({ host: "https://icp-api.io" });
  if (process.env.DFX_NETWORK !== "ic") {
    await agent.fetchRootKey();
  }
  // Proceed with actor creation or calls
} catch (error) {
  // Handle error gracefully without triggering Next.js warnings
  console.info("ICP connection failed:", error);
  // Show user-friendly message or fallback
}
```

## Impact Assessment

### Current State

- ❌ **Scary Development Warnings**: Red "N" indicator in Next.js development mode
- ❌ **Poor Developer Experience**: Developers think the app is broken
- ❌ **No Fallback**: Users can't use the app when ICP is down
- ❌ **No Recovery**: App doesn't attempt to reconnect

### Business Impact

- **Developer Experience**: Scary warnings make development unpleasant
- **Reliability**: Single point of failure (ICP dependency)
- **User Retention**: Users abandon the app when it crashes
- **Support Burden**: Developers report scary warnings instead of understanding the issue

## Proposed Solutions

### Option 1: Graceful Degradation (Recommended)

```typescript
// ICP Connection State
let icpState = {
  isConnected: false,
  retryCount: 0,
  maxRetries: 3,
  lastError: null as Error | null,
};

// Initialize ICP with timeout and error handling
export const initializeICP = async (): Promise<boolean> => {
  try {
    const agent = await createAgentWithTimeout();
    icpState.isConnected = true;
    icpState.lastError = null;
    return true;
  } catch (error) {
    console.warn("ICP connection failed:", error);
    handleICPError(error);
    return false;
  }
};

// Create agent with timeout
const createAgentWithTimeout = async (): Promise<HttpAgent> => {
  return Promise.race([
    createAgent(),
    new Promise((_, reject) => setTimeout(() => reject(new Error("ICP connection timeout")), 10000)),
  ]);
};

// Handle ICP connection errors
const handleICPError = (error: Error): void => {
  icpState.lastError = error;
  console.warn("ICP unavailable, falling back to Neon-only mode");

  // Show user-friendly message
  showICPOfflineMessage();

  // Attempt reconnection in background
  scheduleReconnection();
};

// Show offline message to user
const showICPOfflineMessage = (): void => {
  // Show toast or banner that ICP is offline
  // App continues to work with Neon-only features
};

// Schedule reconnection attempt
const scheduleReconnection = (): void => {
  setTimeout(() => {
    if (icpState.retryCount < icpState.maxRetries) {
      icpState.retryCount++;
      initializeICP();
    }
  }, 5000); // Retry after 5 seconds
};
```

### Option 2: Circuit Breaker Pattern

```typescript
// Circuit breaker state
let circuitBreakerState = {
  failures: 0,
  lastFailureTime: 0,
  threshold: 5,
  timeout: 60000, // 1 minute
};

// Call ICP operation with circuit breaker
export const callICP = async <T>(operation: () => Promise<T>): Promise<T | null> => {
  if (isCircuitOpen()) {
    console.warn("ICP circuit breaker is open, skipping ICP operation");
    return null;
  }

  try {
    const result = await operation();
    onSuccess();
    return result;
  } catch (error) {
    onFailure();
    throw error;
  }
};

// Check if circuit breaker is open
const isCircuitOpen = (): boolean => {
  return (
    circuitBreakerState.failures >= circuitBreakerState.threshold &&
    Date.now() - circuitBreakerState.lastFailureTime < circuitBreakerState.timeout
  );
};

// Handle successful operation
const onSuccess = (): void => {
  circuitBreakerState.failures = 0;
  circuitBreakerState.lastFailureTime = 0;
};

// Handle failed operation
const onFailure = (): void => {
  circuitBreakerState.failures++;
  circuitBreakerState.lastFailureTime = Date.now();
};
```

## Performance Optimizations

### Lazy Loading (Future Enhancement)

```typescript
// Only initialize ICP when needed
export const useICP = () => {
  const [icpStatus, setIcpStatus] = useState<"loading" | "connected" | "offline">("loading");

  useEffect(() => {
    // Initialize ICP only when user tries to use ICP features
    initializeICPWhenNeeded();
  }, []);

  const initializeICPWhenNeeded = async () => {
    try {
      await initializeICP();
      setIcpStatus("connected");
    } catch (error) {
      setIcpStatus("offline");
      // App continues to work without ICP
    }
  };
};
```

**Note**: Lazy loading is a performance optimization, not a solution to the crash problem. It postpones the error but doesn't prevent it. The core issue still requires proper error handling (Option 1).

## Recommended Solution

Based on the analysis, **Option 1 (Graceful Degradation)** is the recommended solution for this use case because:

- **Low user volume** and simple ICP usage
- **Two main scenarios**: Production ICP network down, or local development without ICP
- **Simple error handling** is sufficient
- **Quick implementation** to fix the immediate crash

### Implementation

**Primary Fix: Update `backendActor` function**

```typescript
export async function backendActor(identity?: Identity): Promise<BackendActor> {
  try {
    const agent = await createAgent(identity);
    return makeActor(backendIDL, BACKEND_CANISTER_ID, agent);
  } catch (error) {
    console.warn("ICP unavailable");
    throw new Error("ICP service unavailable. Please try again later.");
  }
}
```

**Secondary Fix: Update service layer**

```typescript
// In fetchMemoriesFromICP function
const fetchMemoriesFromICP = async (page: number): Promise<FetchMemoriesResult> => {
  try {
    const { backendActor } = await import("@/ic/backend");
    const { getAuthClient } = await import("@/ic/ii");

    // ... existing code ...
  } catch (error) {
    console.warn("ICP connection failed:", error);
    // Show user-friendly toast message
    showToast({
      title: "ICP Unavailable",
      description: "Internet Computer network is currently unavailable",
      variant: "default",
    });

    // Return empty results when ICP is unavailable
    return { data: [], hasMore: false, total: 0 };
  }
};
```

### Benefits of This Solution

- ✅ **No more scary warnings** when ICP is down
- ✅ **User-friendly error messages**
- ✅ **App continues to work** without scary warnings
- ✅ **Works in local development** without ICP running
- ✅ **Simple implementation** - just add try-catch blocks
- ✅ **Perfect for low-volume usage** - no complex state management needed

## Implementation Plan

### Phase 1: Error Handling (Immediate)

1. **Wrap ICP initialization in try-catch**
2. **Add timeout to ICP connections**
3. **Show user-friendly error messages**
4. **Prevent scary Next.js development warnings**

### Phase 2: Graceful Degradation

1. **Implement fallback modes**
2. **Add ICP status indicators**
3. **Create offline-first features**
4. **Add retry mechanisms**

### Phase 3: Resilience

1. **Implement circuit breaker pattern**
2. **Add health checks**
3. **Create monitoring and alerting**
4. **Add automatic recovery**

## Files to Modify

### Core ICP Files

- `src/services/icp/` - ICP service files
- `src/hooks/use-icp.ts` - ICP React hook
- `src/components/icp/` - ICP components
- `src/lib/icp-agent.ts` - ICP agent configuration

### Error Handling

- `src/lib/error-handler.ts` - Global error handling
- `src/components/error-boundary.tsx` - React error boundary
- `src/hooks/use-error-handler.ts` - Error handling hook

## Acceptance Criteria

- [ ] No scary Next.js development warnings when ICP is unavailable
- [ ] User sees clear message about ICP status
- [ ] App continues to work with available features
- [ ] ICP connection attempts are retried automatically
- [ ] Error messages are user-friendly
- [ ] Performance is not impacted by ICP failures
- [ ] Monitoring and logging for ICP issues

## Technical Details

### Error Types to Handle

1. **Network Errors**: Connection timeouts, DNS failures
2. **HTTP Errors**: 500, 502, 503, 504 status codes
3. **Protocol Errors**: ICP-specific protocol failures
4. **Authentication Errors**: Invalid credentials or tokens

### Monitoring

- Track ICP connection success/failure rates
- Monitor response times
- Alert on repeated failures
- Log error details for debugging

## Related Issues

- Client-server boundary violations
- Database connection issues
- Upload service failures
- Authentication problems

## Next Steps

1. **Immediate**: Add try-catch around ICP initialization to prevent scary warnings
2. **Short-term**: Implement graceful degradation
3. **Long-term**: Add resilience patterns and monitoring
