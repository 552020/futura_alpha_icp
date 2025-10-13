# HTTP Asset Lookup 404 Errors - Asset Not Found Issue

## Issue Summary

The HTTP module is successfully parsing URLs, validating tokens, and handling all edge cases, but is failing to locate assets that were created during memory creation. This results in **404 "Asset not found"** errors even when valid tokens are provided.

## Current Status

- ✅ **URL Parsing**: Robust query parsing handles all edge cases
- ✅ **Token Validation**: Tokens are correctly validated and accepted
- ✅ **HTTP Module**: Compiles, deploys, and responds correctly
- ❌ **Asset Lookup**: `FuturaAssetStore.get_inline()` fails to find assets (404 errors)

## Problem Description

### Expected Behavior

When a valid token is provided for a memory with assets, the HTTP module should:

1. Parse the URL and extract the token ✅ (working)
2. Validate the token ✅ (working)
3. Look up the asset by ID ✅ (working)
4. Return the asset data with proper headers ❌ (failing with 404)

### Actual Behavior

```bash
curl "http://canister.localhost:4943/asset/memory_id/thumbnail?token=VALID_TOKEN&id=asset_id"
# Returns: HTTP/1.1 404 Not Found
# Body: "Asset not found"
```

### Debug Information

The HTTP module correctly:

- ✅ Parses multiple query parameters
- ✅ Extracts token and asset ID
- ✅ Validates the token (returns 403 "Bad token" for invalid tokens)
- ❌ Fails to find the asset in the store

## Root Cause Analysis

### Potential Issues

1. **Asset ID Mismatch**: The asset ID format between creation and lookup might be different
2. **Memory Context**: The asset lookup might be searching in the wrong memory context
3. **Store Implementation**: The `FuturaAssetStore.get_inline()` method might have bugs
4. **Asset Storage**: Assets might not be stored correctly during memory creation
5. **ACL Context**: The asset lookup might be using the wrong principal context

### Investigation Areas

#### 1. Asset ID Format Consistency

**Question**: Are asset IDs generated and stored in the same format as they're being looked up?

**Investigation**:

- Check how asset IDs are generated during memory creation
- Verify the format used in `FuturaAssetStore.get_inline()`
- Compare asset IDs in memory creation vs. HTTP lookup

#### 2. Memory Context in Asset Lookup

**Question**: Is the asset lookup searching in the correct memory?

**Investigation**:

- Verify that `get_first_asset_id()` finds the correct memory
- Check if the memory ID format matches between creation and lookup
- Ensure the memory is accessible to the token's subject principal

#### 3. Store Implementation Issues

**Question**: Are there bugs in the `FuturaAssetStore` implementation?

**Investigation**:

- Review `src/backend/src/http/adapters/asset_store.rs`
- Check the `get_inline()` method implementation
- Verify the asset retrieval logic

#### 4. Asset Storage During Creation

**Question**: Are assets actually being stored correctly during memory creation?

**Investigation**:

- Verify that `memories_create` properly stores inline assets
- Check if asset metadata is correctly associated with assets
- Ensure assets are stored in the expected data structures

#### 5. ACL Context Mismatch

**Question**: Is the asset lookup using the correct principal context?

**Investigation**:

- Verify that `get_first_asset_id()` uses the token's subject principal
- Check if the memory is accessible to that principal
- Ensure ACL permissions are correctly set up

## Debugging Strategy

### Step 1: Add Debug Logging

Add comprehensive logging to the asset lookup process:

```rust
// In FuturaAssetStore.get_inline()
ic_cdk::println!("🔍 Asset Lookup Debug:");
ic_cdk::println!("  Memory ID: {}", memory_id);
ic_cdk::println!("  Asset ID: {}", asset_id);
ic_cdk::println!("  Principal: {:?}", principal);

// Log accessible capsules
let accessible_capsules = store.get_accessible_capsules(&PersonRef::Principal(*principal));
ic_cdk::println!("  Accessible capsules: {:?}", accessible_capsules);

// Log memory search results
for capsule_id in accessible_capsules {
    if let Some(memory) = store.get_memory(&capsule_id, &memory_id.to_string()) {
        ic_cdk::println!("  Found memory in capsule: {}", capsule_id);
        ic_cdk::println!("  Inline assets: {:?}", memory.inline_assets.len());
        ic_cdk::println!("  Blob internal assets: {:?}", memory.blob_internal_assets.len());
        ic_cdk::println!("  Blob external assets: {:?}", memory.blob_external_assets.len());

        // Log asset IDs for comparison
        for asset in &memory.inline_assets {
            ic_cdk::println!("    Inline asset ID: {}", asset.asset_id);
        }
    }
}
```

### Step 2: Test Asset Creation and Retrieval

Create a test that:

1. Creates a memory with an inline asset
2. Logs the generated asset ID
3. Attempts to retrieve the asset via HTTP
4. Compares the asset IDs

### Step 3: Verify Memory Accessibility

Test that:

1. The memory is created by the same principal that mints the token
2. The memory is accessible to that principal
3. The asset lookup uses the correct principal context

### Step 4: Check Asset Storage Format

Verify that:

1. Assets are stored in the expected data structures
2. Asset metadata is correctly associated
3. The asset ID format is consistent

## Test Cases to Implement

### Test 1: Asset ID Format Consistency

```javascript
// Create memory with known asset
const memoryId = await createTestImageMemory(actor, capsuleId);
const memory = await actor.memories_get(capsuleId, memoryId);
const assetId = memory.inline_assets[0].asset_id;

// Try to retrieve via HTTP
const token = await mintHttpToken(memoryId, ["thumbnail"], [assetId]);
const response = await fetch(`http://canister.localhost:4943/asset/${memoryId}/thumbnail?token=${token}&id=${assetId}`);

// Should return 200, not 404
assert(response.status === 200);
```

### Test 2: Memory Context Verification

```javascript
// Create memory and verify it's accessible
const memoryId = await createTestImageMemory(actor, capsuleId);
const accessibleMemories = await actor.memories_list(capsuleId);

// Verify memory is in the list
const foundMemory = accessibleMemories.find((m) => m.id === memoryId);
assert(foundMemory !== undefined);
```

### Test 3: Asset Storage Verification

```javascript
// Create memory and verify asset storage
const memoryId = await createTestImageMemory(actor, capsuleId);
const memory = await actor.memories_get(capsuleId, memoryId);

// Verify asset is stored correctly
assert(memory.inline_assets.length > 0);
assert(memory.inline_assets[0].asset_id !== undefined);
assert(memory.inline_assets[0].bytes.length > 0);
```

## Expected Resolution

Once the root cause is identified and fixed, the HTTP module should:

1. ✅ Parse URLs correctly (already working)
2. ✅ Validate tokens correctly (already working)
3. ✅ Find assets in the store (currently failing)
4. ✅ Return asset data with proper headers (currently failing)

## Success Criteria

- [ ] HTTP requests with valid tokens return 200 with asset data
- [ ] Asset lookup finds assets created during memory creation
- [ ] Debug logging shows successful asset retrieval
- [ ] Integration tests pass for asset serving

## Related Files

- `src/backend/src/http/adapters/asset_store.rs` - Asset store implementation
- `src/backend/src/http/routes/assets.rs` - Asset route handler
- `tests/backend/http/test_*.mjs` - Integration tests
- `tests/backend/utils/helpers/memory-creation.js` - Memory creation utilities

## Priority

**HIGH** - This is the final blocker preventing the HTTP module from being fully functional for asset serving.
