# Backend Connectivity Detection - Technical Question

## Context

We're implementing error handling for the ICP capsule service and need to understand how to properly detect backend connectivity issues.

## Current Situation

- Frontend calls ICP backend through actors
- We have basic error handling but no specific connectivity detection
- Need to distinguish between:
  - Backend is down/unreachable
  - Authentication issues
  - Business logic errors
  - Network timeouts

## Questions for Tech Lead

### 1. Backend Connectivity Detection

**How can we detect if the ICP backend is unreachable vs other types of errors?**

- Are there specific error patterns from the ICP agent when the backend is down?
- Should we use a simple health check endpoint?
- What error types does the ICP agent throw for connectivity issues?

### 2. Error Classification

**What are the actual error types we should handle?**

From the backend code, we see these error variants:

```rust
pub enum Error {
    Unauthorized,
    NotFound,
    InvalidArgument(String),
    Conflict(String),
    ResourceExhausted,
    Internal(String),
    NotImplemented(String),
}
```

But what about:

- Network timeouts
- Connection refused
- DNS resolution failures
- ICP agent errors

### 3. Recommended Approach

**What's the best practice for detecting backend connectivity?**

Options we're considering:

1. **Health Check Endpoint**: Add a simple `health_check()` method to the backend
2. **Error Pattern Matching**: Detect specific error messages from the ICP agent
3. **Timeout Detection**: Use request timeouts to detect connectivity issues
4. **Actor Creation Failure**: Check if actor creation itself fails

### 4. Concrete Example

**Current situation - what happens when backend is unreachable:**

```typescript
// What we're doing now:
const handleGetCapsuleInfo = async () => {
  try {
    const authenticatedActor = await getActor();
    const capsuleResult = await authenticatedActor.capsules_read_basic([]);
    // ... handle result
  } catch (error) {
    // What type of error do we get when backend is down?
    console.log("Error:", error);
  }
};
```

**Behind the scenes:**

- `getActor()` creates an ICP actor using `Actor.createActor()` with `HttpAgent`
- `capsules_read_basic([])` makes the actual call to the ICP backend
- We need to know what errors come from the ICP agent/HttpAgent layer vs the backend business logic

**What we need to know:**

- What error object do we get when ICP backend is unreachable?
- What error object do we get when ICP backend is slow/timing out?
- What error object do we get when ICP backend returns business logic errors?

**Example error scenarios:**

1. **Backend completely down** - What error?
2. **Network timeout** - What error?
3. **Authentication expired** - What error?
4. **Capsule not found** - What error?
5. **Invalid arguments** - What error?

### 5. Error Handling Strategy

**What should the frontend do when backend is unreachable?**

- Show specific "Backend Unavailable" message?
- Retry automatically?
- Fallback to cached data?
- Disable certain features?

## Current Frontend Error Handling

```typescript
// We currently handle these error types:
- AuthenticationExpiredError
- CapsuleNotFoundError
- CapsuleUnauthorizedError
- CapsuleServiceError (generic)

// But we need:
- BackendConnectionError (when backend is unreachable)
```

## Request

Please provide guidance on:

1. How to detect backend connectivity issues
2. Recommended error handling strategy
3. Whether to implement a health check endpoint
4. Best practices for ICP backend error detection

## Files Involved

- `src/nextjs/src/services/capsule.ts` - Service layer
- `src/nextjs/src/components/icp/capsule-info.tsx` - UI component
- `src/backend/src/` - Backend error types

---

**Priority**: Medium  
**Assignee**: Tech Lead  
**Labels**: backend, connectivity, error-handling, icp
