# 404 Fixes Implementation Summary

## Overview

This document summarizes the implementation of fixes for 404 errors in HTTP asset serving. The fixes address the root causes identified in the analysis and implement proper diagnostic logging.

## Root Causes Identified and Fixed

### 1. Token Subject Principal Issue ✅ FIXED

**Problem**: The `FuturaAssetStore` was using `ic_cdk::caller()` instead of the token's subject principal for ACL/store queries.

**Solution**:

- Added new methods `get_inline_with_principal()`, `exists_with_principal()`, and `get_blob_with_principal()` that use the token's subject principal
- Updated the asset route to extract and use `token.p.sub` for all asset lookups
- This ensures that asset lookups use the correct principal for ACL checks

### 2. Variant-to-Asset-ID Resolution ✅ FIXED

**Problem**: No proper mapping between variants and asset IDs, causing mismatches when requesting `/asset/{mem}/thumbnail?id=<original_id>`.

**Solution**:

- Implemented `resolve_asset_for_variant()` method that handles variant-specific asset resolution
- Supports both exact ID matching and automatic selection of first available asset for a variant
- Priority order: inline → blob_internal → blob_external
- Provides foundation for future base_id → variant_id mapping

### 3. Incomplete Asset Type Handling ✅ FIXED

**Problem**: Only inline assets were handled; blob assets were ignored.

**Solution**:

- Added `get_blob_with_principal()` method to handle blob internal assets
- For Phase 1, only blobs ≤ 2MB are served inline (larger blobs will be handled in Phase 2 with streaming)
- External blob URLs are not supported in Phase 1
- Fallback chain: inline → blob_internal → 404

### 4. Missing Diagnostic Logging ✅ FIXED

**Problem**: No visibility into why 404s were occurring.

**Solution**:

- Added comprehensive logging throughout the asset resolution pipeline
- Logs include: token subject, memory ID, variant, asset IDs, accessible capsules, asset counts
- Log prefixes: `[HTTP-ASSET]`, `[ASSET-LOOKUP]`, `[VARIANT-RESOLVE]`, `[BLOB-LOOKUP]`
- All logs exclude sensitive token data

## Implementation Details

### New AssetStore Methods

```rust
// Use token's subject principal instead of HTTP caller
fn get_inline_with_principal(&self, who: &Principal, memory_id: &str, asset_id: &str) -> Option<InlineAsset>;
fn exists_with_principal(&self, who: &Principal, memory_id: &str, asset_id: &str) -> bool;
fn get_blob_with_principal(&self, who: &Principal, memory_id: &str, asset_id: &str) -> Option<(Vec<u8>, String)>;

// Handle variant-specific asset resolution
fn resolve_asset_for_variant(
    &self,
    who: &Principal,
    memory_id: &str,
    variant: &str,
    id_param: Option<&str>,
) -> Option<String>;
```

### Updated Asset Route Flow

1. **Token Validation**: Extract and validate token, ensuring `token.p.sub` is present
2. **Asset Resolution**: Use `resolve_asset_for_variant()` to find the correct asset ID
3. **Asset Retrieval**: Try inline assets first, then blob assets as fallback
4. **Response**: Return asset with proper headers or 404 with diagnostic info

### Diagnostic Logging Examples

```
[HTTP-ASSET] mem=mem-123 variant=thumbnail id_param=Some("asset-456")
[HTTP-ASSET] token.sub=Some(rdmx6-jaaaa-aaaah-qcaiq-cai) scope.mem=mem-123 scope.variants=["thumbnail"] scope.ids=None
[VARIANT-RESOLVE] principal=rdmx6-jaaaa-aaaah-qcaiq-cai mem=mem-123 variant=thumbnail id_param=Some("asset-456")
[ASSET-LOOKUP] accessible_capsules=[capsule-789]
[ASSET-LOOKUP] found memory in cap=capsule-789
[ASSET-LOOKUP] counts inline=2 blob_int=1 blob_ext=0
[ASSET-LOOKUP] inline.id=asset-456 ct=image/png
[ASSET-LOOKUP] ✅ Found matching inline asset
[HTTP-ASSET] ✅ Resolved asset_id=asset-456
```

## Testing

### Integration Tests

Created comprehensive integration tests in `tests/backend/http/test_404_fixes.mjs`:

- **Token Subject Principal Test**: Verifies correct principal usage
- **Variant Resolution Test**: Tests asset ID resolution logic
- **Diagnostic Logging Test**: Confirms logging is working
- **Authorization Header Test**: Tests Bearer token support

### Running Tests

```bash
cd tests/backend/http
./run_404_fixes_test.sh
```

### Manual Testing Checklist

1. **Variant Correctness**:

   - Create memory with original + thumbnail assets
   - Mint token for `["thumbnail"]` variant
   - Call `/asset/{mem}/thumbnail?id=<thumbnail_id>` → expect 200
   - Call `/asset/{mem}/thumbnail` (no id) → expect 200 (first asset)

2. **Principal Correctness**:

   - Mint token as user A
   - Ensure user B's memory cannot be resolved → expect 403/404

3. **Asset Type Handling**:
   - Test inline asset (<2MB) → expect 200
   - Test blob asset (≤2MB) → expect 200
   - Test blob asset (>2MB) → expect 404 (Phase 1 limitation)

## Phase 2 Considerations

The current implementation sets up the foundation for Phase 2 improvements:

1. **Streaming Support**: Blob assets >2MB will need streaming implementation
2. **External URL Support**: Blob external assets need HTTP range request support
3. **Base ID Mapping**: Implement `base_id → variant_id` mapping for frontend compatibility
4. **Performance Optimization**: Cache accessible capsules and memory lookups

## Files Modified

- `src/backend/src/http/adapters/asset_store.rs` - Added new methods with proper principal usage
- `src/backend/src/http/core/types.rs` - Extended AssetStore trait
- `src/backend/src/http/routes/assets.rs` - Updated route logic to use new methods
- `tests/backend/http/test_404_fixes.mjs` - Integration tests
- `tests/backend/http/run_404_fixes_test.sh` - Test runner script

## Verification

To verify the fixes are working:

1. **Check Logs**: Look for diagnostic log entries in canister logs
2. **Test Requests**: Make HTTP requests and verify proper responses
3. **Monitor 404s**: 404 errors should now be rare and have clear diagnostic info
4. **Performance**: Asset serving should be faster due to correct principal usage

## Next Steps

1. Deploy the fixes to your local environment
2. Run the integration tests
3. Monitor canister logs for diagnostic output
4. Test with real assets and tokens
5. Prepare for Phase 2 streaming implementation

The 404 fixes are now implemented and ready for testing. The diagnostic logging will help identify any remaining issues quickly.
