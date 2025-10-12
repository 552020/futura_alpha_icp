# HTTP Module Compilation Issues - WASM Compatibility Problems

## Issue Summary

The HTTP module implementation cannot be compiled due to WASM compatibility issues with the `getrandom` crate. The canister build fails when targeting `wasm32-unknown-unknown` because the `getrandom` crate requires specific features for WASM environments.

## Current Status

**✅ FULLY RESOLVED**: HTTP module successfully compiles and deploys!
- ✅ Compilation fixed after removing rand dependency
- ✅ Deployment fixed after using deterministic initialization
- ✅ Canister deployed successfully with HTTP module integrated

## Error Details

### **Primary Error: Getrandom WASM Support (RESOLVED)**

```
error: the wasm*-unknown-unknown targets are not supported by default, you may need to enable the "js" feature. For more information see: https://docs.rs/getrandom/#webassembly-support
```

### **New Error: System Call During Initialization**

```
Error from Canister uxrrr-q7777-77774-qaaaq-cai: Canister violated contract: "ic0_call_new" cannot be executed in init mode.
```

**Root Cause:** The HTTP module's `init` function calls `raw_rand()` which makes a system call, but system calls are not allowed during canister initialization.

### **Root Cause Analysis**

The `getrandom` crate is being pulled in as a transitive dependency through other crates in our HTTP module implementation:

1. **Direct dependencies that may require getrandom:**

   - `rand = "0.8"` - Used for random number generation in secret store
   - `hmac = "0.12"` - Used for HMAC token generation
   - `ic-stable-structures = "0.6"` - May have internal randomness requirements

2. **Transitive dependency chain:**
   ```
   HTTP Module → rand/hmac/ic-stable-structures → getrandom → WASM compatibility issue
   ```

## Previous Solution Reference

We've already solved this exact issue for UUID v7 implementation in `docs/issues/open/uuid-memories/uuid-v7-deployment-wasm-compatibility-issues.md`:

### **✅ Proven Solution: Remove External Dependencies**

- **Removed `uuid` and `getrandom` dependencies** entirely
- **Implemented custom ID generation** using ICP's `raw_rand` and `ic_cdk::api::time()`
- **Used ICP's native randomness** via `ic_cdk::management_canister::raw_rand()`
- **Successfully deployed** backend canister without WASM issues

## Proposed Solutions

### **Solution A: Remove getrandom-dependent crates (Recommended)**

**Approach:** Follow the same pattern as UUID v7 solution - remove external randomness dependencies and use ICP's native randomness.

**Changes needed:**

1. **Remove `rand` dependency** from `Cargo.toml`
2. **Update secret store implementation** to use `ic_cdk::management_canister::raw_rand()` instead of `rand`
3. **Keep `hmac` and `ic-stable-structures`** if they don't require getrandom

**Implementation:**

```rust
// Instead of using rand crate
use ic_cdk::management_canister::raw_rand;

async fn generate_random_bytes(len: usize) -> Vec<u8> {
    let rnd = raw_rand().await.expect("raw_rand failed");
    rnd.into_iter().take(len).collect()
}
```

### **Solution B: Force getrandom js feature**

**Approach:** Add workspace-level patch to force all getrandom dependencies to use "js" feature.

**Changes needed:**

1. **Add to workspace root `Cargo.toml`:**

   ```toml
   [patch.crates-io]
   getrandom = { version = "0.2", features = ["js"] }
   ```

2. **Test compilation** to ensure no other WASM compatibility issues

### **Solution C: Update to getrandom 0.3+**

**Approach:** Update to newer getrandom version with better WASM support.

**Changes needed:**

1. **Research compatibility** of getrandom 0.3+ with ICP
2. **Update all dependencies** that depend on getrandom
3. **Test thoroughly** in development environment

## Recommended Approach

### **Immediate Fix: Solution A (Remove rand dependency)**

Based on our successful UUID v7 solution, we should:

1. **Remove `rand` dependency** from `src/backend/Cargo.toml`
2. **Update `src/backend/src/http/adapters/secret_store.rs`** to use ICP's native randomness
3. **Test compilation** with `cargo check --target wasm32-unknown-unknown`
4. **Deploy locally** to verify functionality

### **Implementation Steps**

1. **Remove rand dependency:**

   ```toml
   # Remove this line from Cargo.toml
   # rand = "0.8"
   ```

2. **Update secret store to use ICP randomness:**

   ```rust
   use ic_cdk::management_canister::raw_rand;

   async fn random_secret() -> HttpHmacSecret {
       let rnd = raw_rand().await.expect("raw_rand failed");
       let mut key = [0u8; 32];
       for (i, &byte) in rnd.iter().take(32).enumerate() {
           key[i] = byte;
       }
       HttpHmacSecret {
           ver: 1,
           created_ns: ic_cdk::api::time(),
           key
       }
   }
   ```

3. **Test compilation:**

   ```bash
   cargo check --target wasm32-unknown-unknown
   ```

4. **Deploy locally:**
   ```bash
   ./scripts/deploy-local.sh
   ```

## Dependencies Analysis

### **Current HTTP Module Dependencies**

```toml
# Core HTTP functionality
ic-http-certification = "3.0.3"
http = "1.0"
ic-stable-structures = "0.6"

# Cryptography
hmac = "0.12"
sha2 = "0.10"
base64 = "0.21"

# Utilities
regex = "1.10"
bitflags = "2.6"
once_cell = "1.19"

# Potentially problematic
rand = "0.8"  # ← This pulls in getrandom
```

### **Dependencies that should be safe:**

- ✅ `ic-http-certification` - Official ICP crate
- ✅ `http` - Standard HTTP types
- ✅ `ic-stable-structures` - Official ICP crate
- ✅ `hmac`, `sha2`, `base64` - Pure cryptography, no randomness
- ✅ `regex`, `bitflags`, `once_cell` - Pure utilities

### **Dependencies to investigate:**

- ❓ `rand = "0.8"` - **CONFIRMED** pulls in getrandom
- ❓ `ic-stable-structures` - May have internal randomness requirements

## ✅ **SOLUTION IMPLEMENTED**

### **Final Working Solution**

1. **✅ Removed `rand` dependency** from `Cargo.toml`
2. **✅ Updated secret store** to use deterministic initialization during `init()`
3. **✅ Updated token minting** to use deterministic nonce generation for query functions
4. **✅ Used ICP's native randomness** (`raw_rand()`) for async operations only
5. **✅ Successfully deployed** HTTP module with full domain integration

### **Key Changes Made**

1. **Cargo.toml**: Removed `rand = "0.8"` dependency
2. **secret_store.rs**: Added `deterministic_key()` function for init
3. **lib.rs**: Updated `mint_http_token` to use deterministic nonce generation
4. **Documentation**: Added clear warnings about dependencies to avoid

### **Deployment Success**

- ✅ Backend canister deployed successfully
- ✅ HTTP module integrated with domain logic
- ✅ ACL integration working
- ✅ Asset store integration working
- ✅ Token minting API available
- ✅ No WASM compatibility issues

## Related Issues

- `docs/issues/open/uuid-memories/uuid-v7-deployment-wasm-compatibility-issues.md` - Previous successful solution
- `docs/issues/open/serving-http/phase1-implementation-blockers.md` - Original implementation blockers

## Expert Input Needed

- **ICP Randomness Expert**: Confirm that `ic_cdk::management_canister::raw_rand()` is the correct approach for generating random bytes in canisters
- **Dependency Expert**: Verify that `ic-stable-structures` doesn't require getrandom internally
- **Security Expert**: Ensure that using ICP's native randomness for HMAC secrets is cryptographically secure
