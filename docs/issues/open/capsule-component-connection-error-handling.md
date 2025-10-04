# Capsule Component Connection Error Handling Issue

## Problem Description

The CapsuleList component is experiencing ungraceful error handling when the ICP local development environment is not running, resulting in:

1. **Raw stack traces displayed to users** instead of user-friendly error messages
2. **No fallback behavior** when ICP connection fails
3. **Poor user experience** with technical error details exposed
4. **No retry mechanism** or connection status indication

## Current Error Behavior

### What Users See

- Raw JavaScript stack trace with technical details
- "Failed to fetch HTTP request: TypeError: Failed to fetch" error
- Multiple lines of internal file paths and function calls
- No clear indication of what went wrong or how to fix it

### Error Stack Trace Pattern

```
Console TransportError
Failed to fetch HTTP request: TypeError: Failed to fetch
at HttpAgent.requestAndRetryQuery (file:///Users/stefano/Documents/Code/Futura/futura_alpha_icp/src/nextjs/.next/static/chunks/ff37e_%40dfinity_agent_lib_esm_c0b76152._.js:3747:290)
at async HttpAgent.requestAndRetryQuery (file:///Users/stefano/Documents/Code/Futura/futura_alpha_icp/src/nextjs/.next/static/chunks/ff37e_%40dfinity_agent_lib_esm_c0b76152._.js:3770:20)
at async HttpAgent.requestAndRetryQuery (file:///Users/stefano/Documents/Code/Futura/futura_alpha_icp/src/nextjs/.next/static/chunks/ff37e_%40dfinity_agent_lib_esm_c0b76152._.js:3770:20)
at async HttpAgent.requestAndRetryQuery (file:///Users/stefano/Documents/Code/Futura/futura_alpha_icp/src/nextjs/.next/static/chunks/ff37e_%40dfinity_agent_lib_esm_c0b76152._.js:3770:20)
at async makeQuery (file:///Users/stefano/Documents/Code/Futura/futura_alpha_icp/src/nextjs/.next/static/chunks/ff37e_%40dfinity_agent_lib_esm_c0b76152._.js:3181:27)
at async HttpAgent.query (file:///Users/stefano/Documents/Code/Futura/futura_alpha_icp/src/nextjs/.next/static/chunks/ff37e_%40dfinity_agent_lib_esm_c0b76152._.js:3205:54)
at async caller (file:///Users/stefano/Documents/Code/Futura/futura_alpha_icp/src/nextjs/.next/static/chunks/node_modules__pnpm_02ed610b._.js:2617:28)
at async CapsuleList.useCallback[loadCapsules] (src/components/icp/capsule-list.tsx:52:30)
```

### Specific Error Location

- **File**: `src/components/icp/capsule-list.tsx`
- **Line**: 52
- **Code**: `const capsuleHeaders = await actor.capsules_list();`
- **Error Type**: `Console TransportError`
- **Root Cause**: `ERR_CONNECTION_REFUSED` to ICP local development environment

## Root Cause Analysis

### Primary Issues

1. **No Error Boundary**: Component doesn't catch and handle ICP connection errors
2. **Raw Error Display**: Technical errors shown directly to users
3. **No Connection Status**: No indication that ICP environment is required
4. **Missing Fallback**: No graceful degradation when ICP is unavailable

### Technical Details

- **Error Type**: `ERR_CONNECTION_REFUSED` to `127.0.0.1:4943`
- **Component**: `CapsuleList` in `src/nextjs/src/components/icp/capsule-list.tsx`
- **Hook**: `useICPIdentity` and `useAuthenticatedActor`
- **Service**: `capsules_read_basic()` call failing

## Expected Behavior

### When ICP is Available

- ✅ Capsule list loads successfully
- ✅ User sees their capsules
- ✅ No error messages

### When ICP is Not Available

- ✅ Clear, user-friendly error message
- ✅ Instructions on how to fix the issue
- ✅ Retry button or automatic retry
- ✅ Fallback to offline mode or alternative

## Proposed Solutions

### 1. Error Boundary Implementation

```typescript
// Add error boundary around CapsuleList
<ErrorBoundary fallback={<CapsuleErrorFallback />}>
  <CapsuleList />
</ErrorBoundary>
```

### 2. Connection Status Detection

```typescript
// Check ICP connection before making calls
const { isConnected, isConnecting, error } = useICPConnection();
```

### 3. User-Friendly Error Messages

```typescript
// Replace technical errors with user messages
if (error?.message?.includes("ERR_CONNECTION_REFUSED")) {
  return <ICPConnectionError />;
}
```

### 4. Retry Mechanism

```typescript
// Add retry button and automatic retry
const { retry, isRetrying } = useRetryableOperation(loadCapsules);
```

## Immediate Fix Required

### Current Problem

The `CapsuleList` component at line 52 is calling `actor.capsules_list()` without any error handling, causing raw stack traces to be displayed to users when the ICP local development environment is not running.

### Quick Fix

Add try-catch error handling around the ICP calls:

```typescript
// In CapsuleList component, around line 52
try {
  const actor = await getActor();
  const capsuleHeaders = await actor.capsules_list();
  // ... rest of the logic
} catch (error) {
  console.error("Failed to load capsules:", error);
  // Show user-friendly error message instead of raw stack trace
  setError("Unable to connect to ICP. Please check if the local development environment is running.");
}
```

## Implementation Plan

### Phase 1: Error Boundary (Immediate)

1. **Add Error Boundary**: Wrap CapsuleList in error boundary
2. **Create Error Fallback**: Design user-friendly error component
3. **Test Error Scenarios**: Verify error handling works

### Phase 2: Connection Status (Short-term)

1. **Add Connection Check**: Detect ICP availability
2. **Show Status Indicator**: Display connection status
3. **Prevent Failed Calls**: Don't attempt calls when disconnected

### Phase 3: Retry & Recovery (Medium-term)

1. **Add Retry Button**: Allow manual retry
2. **Auto-retry Logic**: Automatic retry with backoff
3. **Connection Recovery**: Detect when ICP comes back online

### Phase 4: Offline Mode (Long-term)

1. **Cached Data**: Show cached capsule data when offline
2. **Sync Indicators**: Show sync status when online
3. **Offline Actions**: Queue actions for when online

## Error Message Design

### Current (Bad)

```
Failed to fetch HTTP request: TypeError: Failed to fetch
at HttpAgent.requestAndRetryQuery (http://localhost:3000/_next/static/chunks/ff37e_%40dfinity_agent_lib_esm_c0b76152._.js:3747:290)
at async HttpAgent.requestAndRetryQuery (http://localhost:3000/_next/static/chunks/ff37e_%40dfinity_agent_lib_esm_c0b76152._.js:3770:20)
```

### Proposed (Good)

```
🔌 ICP Connection Error

Unable to connect to the ICP network. This might be because:

• The ICP local development environment is not running
• There's a network connectivity issue
• The ICP canister is not deployed

[Retry Connection] [Check Status] [Get Help]
```

## Technical Implementation

### 1. Error Boundary Component

```typescript
interface CapsuleErrorFallbackProps {
  error: Error;
  resetError: () => void;
}

export function CapsuleErrorFallback({ error, resetError }: CapsuleErrorFallbackProps) {
  const isConnectionError = error.message.includes("ERR_CONNECTION_REFUSED");

  if (isConnectionError) {
    return <ICPConnectionError onRetry={resetError} />;
  }

  return <GenericErrorFallback error={error} onRetry={resetError} />;
}
```

### 2. Connection Status Hook

```typescript
export function useICPConnection() {
  const [status, setStatus] = useState<"checking" | "connected" | "disconnected">("checking");

  const checkConnection = useCallback(async () => {
    try {
      await fetch("http://127.0.0.1:4943/api/v2/status");
      setStatus("connected");
    } catch {
      setStatus("disconnected");
    }
  }, []);

  return { status, checkConnection };
}
```

### 3. Retry Logic

```typescript
export function useRetryableOperation<T>(operation: () => Promise<T>, maxRetries: number = 3) {
  const [retryCount, setRetryCount] = useState(0);
  const [isRetrying, setIsRetrying] = useState(false);

  const retry = useCallback(async () => {
    if (retryCount >= maxRetries) return;

    setIsRetrying(true);
    try {
      await operation();
      setRetryCount(0);
    } catch (error) {
      setRetryCount((prev) => prev + 1);
      throw error;
    } finally {
      setIsRetrying(false);
    }
  }, [operation, retryCount, maxRetries]);

  return { retry, isRetrying, retryCount };
}
```

## User Experience Improvements

### 1. Loading States

- Show loading spinner while checking connection
- Display "Connecting to ICP..." message
- Indicate when retrying connection

### 2. Error Recovery

- Clear error messages with actionable steps
- Retry buttons that actually work
- Status indicators for connection state

### 3. Offline Support

- Cache last known capsule data
- Show "Last updated" timestamps
- Queue actions for when online

## Testing Scenarios

### 1. ICP Not Running

- Should show connection error
- Should provide retry option
- Should not show stack traces

### 2. Network Issues

- Should detect network problems
- Should offer retry mechanism
- Should show appropriate error messages

### 3. Canister Not Deployed

- Should detect missing canister
- Should provide deployment instructions
- Should show helpful error messages

## Priority

**High** - This affects user experience and makes the app look unprofessional.

## Files to Modify

- `src/nextjs/src/components/icp/capsule-list.tsx` (main component)
- `src/nextjs/src/hooks/use-icp-connection.ts` (new hook)
- `src/nextjs/src/components/icp/capsule-error-fallback.tsx` (new component)
- `src/nextjs/src/components/icp/icp-connection-error.tsx` (new component)

## Success Criteria

- ✅ No raw stack traces shown to users
- ✅ Clear, actionable error messages
- ✅ Retry functionality works
- ✅ Connection status is visible
- ✅ Graceful degradation when offline
- ✅ Professional error handling

## Related Issues

- [Debug ICP Connection Errors](./debug-icp-connection-errors.md) - Root cause analysis
- [ICP Upload Logging Broken](./icp-upload-logging-broken.md) - Similar connection issues
- [Capsule State Management Refactoring](./capsule-state-management-refactoring.md) - Component architecture
