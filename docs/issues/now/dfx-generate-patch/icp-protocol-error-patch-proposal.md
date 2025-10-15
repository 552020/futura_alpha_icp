# ICP ProtocolError Patch Proposal for DFX Generate

## Issue Summary

### Problem

The `dfx generate` command creates declaration files that include unsafe `fetchRootKey()` calls, which cause `ProtocolError` warnings in Next.js development mode when the ICP network is unavailable. This creates a poor developer experience with scary red warnings that make developers think the application is broken.

### Error Details

```
Console ProtocolError
HTTP request failed:
  Status: 500 (Internal Server Error)
  Headers: [["connection","keep-alive"],["date","Tue, 14 Oct 2025 20:43:40 GMT"],["keep-alive","timeout=5"],["transfer-encoding","chunked"]]
  Body: Internal Server Error

at ProtocolError.fromCode
at HttpAgent.requestAndRetry
at HttpAgent.status
at HttpAgent.fetchRootKey
```

### Root Cause

The generated declaration files in `src/ic/declarations/*/index.js` contain:

```javascript
// Fetch root key for certificate validation during development
if (process.env.NEXT_PUBLIC_DFX_NETWORK !== "ic") {
  agent.fetchRootKey().catch((err) => {
    console.warn("Unable to fetch root key. Check to ensure that your local replica is running");
    console.error(err);
  });
}
```

This code:

1. **Always calls `fetchRootKey()`** when not on mainnet
2. **No health check** before making the call
3. **Triggers Next.js development warnings** (red "N" indicator) when ICP network is unavailable
4. **Creates poor developer experience** - scary warnings that look like errors
5. **Application continues to work** but developers think something is broken

## Current Workarounds

### 1. Manual Declaration Patching

We manually edited the declaration files to add health checks:

```javascript
// Health check function
const isIcpAvailable = async () => {
  try {
    const host = process.env.NEXT_PUBLIC_IC_HOST || "http://127.0.0.1:4943";
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 4000);

    const res = await fetch(`${host}/api/v2/status`, {
      method: "GET",
      signal: controller.signal,
      cache: "no-store",
    });

    clearTimeout(timeoutId);
    return res.ok;
  } catch {
    return false;
  }
};

// Safe fetchRootKey with health check
if (process.env.NEXT_PUBLIC_DFX_NETWORK !== "ic") {
  isIcpAvailable().then((available) => {
    if (available) {
      agent.fetchRootKey().catch((err) => {
        console.warn("Unable to fetch root key. ICP may be unavailable");
        console.error(err);
      });
    } else {
      console.warn("ICP network unavailable, skipping fetchRootKey to prevent crashes");
    }
  });
}
```

### 2. Automated Patching Script

Created `scripts/patch-declarations.js` to automatically patch declarations after `dfx generate`:

```bash
# After dfx generate
node scripts/patch-declarations.js
```

### 3. Custom Backend Actor Wrapper

Implemented `backendActorSafe` function with health checks, but this creates redundant safety layers.

## Proposed DFX Generate Fix

### Solution: Patch DFX Generate Source Code

The ideal solution is to modify the DFX generate command to produce safer declaration files by default.

#### Core Principle: Auto-Generated Library Code Should Be Developer-Friendly

**Auto-generated library code should provide a clean developer experience.** It should:

- ✅ **Never trigger scary warnings in development**
- ✅ **Handle network unavailability gracefully**
- ✅ **Provide clear, non-alarming error messages**
- ✅ **Let developers focus on their code, not library warnings**

The current generated code violates this principle by triggering Next.js development warnings that make developers think the application is broken.

#### Location

- **File**: `secretus/sdk/src/dfx/assets/language_bindings/canister.js`
- **Template**: The Handlebars template that generates `index.js` files
- **Target**: The `fetchRootKey()` call in the template

#### Proposed Changes

1. **Make the template developer-friendly** - prevent scary warnings in development
2. **Add silent error handling** around `fetchRootKey()` calls to prevent Next.js warnings
3. **Use console.info instead of console.error** for non-critical network issues
4. **Implement graceful degradation** when ICP is unavailable without triggering warnings

#### Minimal Solution: Silent Error Handling

The **least invasive** fix is to prevent Next.js warnings by handling errors silently:

```javascript
// Current problematic code:
agent.fetchRootKey().catch((err) => {
  console.warn("Unable to fetch root key. Check to ensure that your local replica is running");
  console.error(err); // This triggers Next.js development warnings
});

// Minimal fix - silent error handling:
agent.fetchRootKey().catch((err) => {
  // Silent handling - no console.error that triggers Next.js warnings
  console.info("ICP network unavailable - continuing without root key validation");
  // Don't log the error to prevent scary warnings
});
```

#### Implementation Details

```rust
// In the canister generation logic, modify the JavaScript template
let js_template = r#"
import { Actor, HttpAgent } from "@dfinity/agent";

// Health check function to prevent crashes when ICP is unavailable
const isIcpAvailable = async () => {
  try {
    const host = process.env.NEXT_PUBLIC_IC_HOST || 'http://127.0.0.1:4943';
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 4000);

    const res = await fetch(`${host}/api/v2/status`, {
      method: 'GET',
      signal: controller.signal,
      cache: 'no-store',
    });

    clearTimeout(timeoutId);
    return res.ok;
  } catch {
    return false;
  }
};

export const createActor = (canisterId, options = {}) => {
  const agent = options.agent || new HttpAgent({ ...options.agentOptions });

  // SAFE fetchRootKey with health check - only call if ICP is available
  if (process.env.NEXT_PUBLIC_DFX_NETWORK !== "ic") {
    isIcpAvailable().then(available => {
      if (available) {
        agent.fetchRootKey().catch(err => {
          console.warn('Unable to fetch root key. ICP may be unavailable');
          console.error(err);
        });
      } else {
        console.warn('ICP network unavailable, skipping fetchRootKey to prevent crashes');
      }
    });
  }

  return Actor.createActor(idlFactory, {
    agent,
    canisterId,
    ...options.actorOptions
  });
};
"#;
```

## Benefits of DFX Generate Fix

### 1. **Universal Solution**

- All ICP applications benefit automatically
- No manual patching required
- No custom wrapper functions needed

### 2. **Better Developer Experience**

- Declarations work out-of-the-box
- No scary Next.js warnings when ICP is unavailable
- Clean development experience by default

### 3. **Maintainability**

- Single source of truth in DFX
- No need for post-generation scripts
- Consistent behavior across all projects

### 4. **Production Ready**

- Applications work in all network conditions
- No scary warnings in development
- Better developer experience

### 5. **Library Design Principle**

- **Auto-generated code provides clean developer experience**
- **No scary warnings for non-critical network issues**
- **Applications can focus on their code, not library warnings**
- **Library code doesn't trigger development warnings unnecessarily**

## Implementation Plan

### Phase 1: Research

1. **Locate the JavaScript template** in DFX source code
2. **Identify where `fetchRootKey()` is added** to declarations
3. **Understand the generation pipeline**

### Phase 2: Development

1. **Modify the template** to include health checks
2. **Test with various network conditions**
3. **Ensure backward compatibility**

### Phase 3: Testing

1. **Test with local replica down**
2. **Test with mainnet unavailable**
3. **Test with network timeouts**
4. **Verify graceful degradation**

### Phase 4: Contribution

1. **Submit PR to DFX repository**
2. **Document the changes**
3. **Get community feedback**

## Alternative Approaches Considered

### 1. **Runtime Wrapping**

- ❌ Requires changes in every application
- ❌ Redundant safety layers
- ❌ Maintenance overhead

### 2. **Post-Generation Scripts**

- ❌ Must be run after every `dfx generate`
- ❌ Easy to forget
- ❌ Not universal

### 3. **Custom Declaration Templates**

- ❌ Requires DFX configuration
- ❌ Not standard across projects
- ❌ Maintenance complexity

## Conclusion

The ideal solution is to patch the DFX generate command at the source to produce cleaner declaration files by default. This would:

- ✅ **Fix the root cause** of Next.js development warnings
- ✅ **Benefit all ICP applications** automatically
- ✅ **Improve developer experience** significantly
- ✅ **Make ICP development more pleasant**

The current workarounds (manual patching, scripts, wrapper functions) are temporary solutions that should be replaced by a proper fix in the DFX generate command itself.
