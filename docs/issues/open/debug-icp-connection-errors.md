# Debug ICP Connection Errors Report

## Error Summary

**Primary Issue**: ICP local development environment is not running, causing `ERR_CONNECTION_REFUSED` errors.

## Error Analysis

### 1. Connection Refused Errors

```
POST http://127.0.0.1:4943/api/v3/canister/uxrrr-q7777-77774-qaaaq-cai/call net::ERR_CONNECTION_REFUSED
POST http://127.0.0.1:4943/api/v2/canister/uxrrr-q7777-77774-qaaaq-cai/query net::ERR_CONNECTION_REFUSED
```

**Root Cause**: The ICP local replica is not running on port 4943.

### 2. Upload Flow Analysis

**What Worked:**

- ✅ Hardcoded ICP preferences activated
- ✅ `uploadFileToICPWithProgress` function called
- ✅ Authentication successful (capsule found: `capsule_1759589492075271000`)
- ✅ Upload configuration determined (1 chunk, chunked upload)
- ✅ Upload session started

**What Failed:**

- ❌ `uploads_begin()` call failed with connection refused
- ❌ All subsequent ICP canister calls failed
- ❌ Both original file and derivative uploads failed

### 3. Error Pattern

The errors follow this sequence:

1. **Query calls succeed** (capsule lookup, status checks)
2. **Call operations fail** (uploads_begin, uploads_put_chunk)
3. **Retry mechanism activates** (multiple retry attempts)
4. **Final failure** with "Failed to fetch" error

### 4. Affected Operations

**Failed Operations:**

- `uploads_begin()` - Starting upload session
- `uploads_put_chunk()` - Uploading file chunks
- `uploads_finish()` - Completing upload
- Derivative uploads (display, thumb, placeholder)

**Successful Operations:**

- `capsules_read_basic()` - Reading existing capsule
- `status()` - ICP replica status check
- Authentication and actor creation

## Technical Details

### Error Types

1. **Network Errors**: `ERR_CONNECTION_REFUSED`
2. **HTTP Errors**: `Failed to fetch`
3. **Retry Exhaustion**: Multiple retry attempts failed

### Retry Pattern

The ICP agent attempts multiple retries:

- Initial call fails
- Retry 1 fails
- Retry 2 fails
- Retry 3 fails
- Final failure

### Canister ID

- **Canister**: `uxrrr-q7777-77774-qaaaq-cai`
- **Port**: `127.0.0.1:4943`
- **Protocol**: HTTP/HTTPS

## Root Cause Analysis

### Primary Cause

**ICP Local Development Environment Not Running**

The application is configured to use local ICP development:

```typescript
NEXT_PUBLIC_DFX_NETWORK=local
NEXT_PUBLIC_IC_HOST=http://127.0.0.1:4943
```

But the local ICP replica is not running on port 4943.

### Secondary Issues

1. **No Fallback**: No error handling for missing local environment
2. **Silent Failures**: Upload appears to start but fails silently
3. **User Experience**: No clear error message to user

## Solutions

### Immediate Fix

1. **Start ICP Local Environment**:

   ```bash
   dfx start --clean
   dfx deploy
   ```

2. **Verify Canister Deployment**:
   ```bash
   dfx canister status uxrrr-q7777-77774-qaaaq-cai
   ```

### Long-term Improvements

1. **Environment Detection**: Check if local ICP is running
2. **Better Error Messages**: Show user-friendly error messages
3. **Fallback Options**: Switch to production ICP if local fails
4. **Health Checks**: Verify canister availability before upload

## Debug Steps

### 1. Check ICP Status

```bash
# Check if dfx is running
dfx ping

# Check canister status
dfx canister status uxrrr-q7777-77774-qaaaq-cai

# Check replica status
curl http://127.0.0.1:4943/api/v2/status
```

### 2. Verify Canister

```bash
# Deploy canister if needed
dfx deploy uxrrr-q7777-77774-qaaaq-cai

# Check canister methods
dfx canister call uxrrr-q7777-77774-qaaaq-cai capsules_read_basic
```

### 3. Test Connection

```bash
# Test basic connectivity
curl -X POST http://127.0.0.1:4943/api/v2/status

# Test canister query
curl -X POST http://127.0.0.1:4943/api/v2/canister/uxrrr-q7777-77774-qaaaq-cai/query
```

## Expected Behavior

### When ICP is Running

- ✅ Capsule lookup succeeds
- ✅ Upload session starts
- ✅ File chunks upload successfully
- ✅ Upload completes
- ✅ Database record created

### When ICP is Not Running

- ❌ Connection refused errors
- ❌ Upload fails immediately
- ❌ No user feedback
- ❌ Silent failure

## Recommendations

### 1. Environment Setup

- Document ICP local development setup
- Add environment validation
- Provide clear setup instructions

### 2. Error Handling

- Add connection health checks
- Show user-friendly error messages
- Implement retry with backoff

### 3. Development Workflow

- Add pre-upload environment checks
- Provide fallback to production ICP
- Add development mode indicators

## Files Affected

- `src/nextjs/src/services/upload/icp-with-processing.ts` (main upload logic)
- `src/nextjs/src/ic/backend.ts` (ICP actor creation)
- `src/nextjs/src/ic/ii.ts` (Internet Identity integration)
- Environment configuration (`.env.local`)

## Priority

**High** - This blocks all ICP upload functionality in development.

## Next Steps

1. **Start ICP local environment** (`dfx start --clean`)
2. **Deploy canister** (`dfx deploy`)
3. **Test upload** (should work after environment is running)
4. **Add environment validation** (prevent future issues)


