# Frontend ICP HTTP Asset Serving Implementation Plan

**Status**: 🔴 **CRITICAL** - Frontend changes needed to resolve 503 errors  
**Priority**: **HIGH** - Blocking ICP asset serving via HTTP gateway  
**Date**: 2025-01-27  
**Reporter**: Development Team

## 🚨 **Problem Summary**

The backend HTTP module is correctly implemented with skip certification for private assets, but the backend is not generating proper HTTP URLs for ICP assets. This causes 503 "response verification error" because:

1. **Backend returns `icp://` protocol URLs** instead of HTTP URLs with tokens
2. **No HTTP URL generation in backend** - Backend doesn't create authenticated HTTP URLs
3. **No HTTP gateway integration** - Assets not served through ICP HTTP gateway
4. **Frontend receives unusable URLs** - Frontend gets `icp://` URLs it can't use

## 📋 **Current Status Analysis**

### ✅ **Backend is Ready**

- **Skip certification implemented** in `src/backend/src/lib.rs` (lines 1444-1462)
- **Token minting API exists** (`mint_http_token` function available)
- **HTTP module complete** with proper authentication and ACL
- **503 error should be resolved** once frontend generates proper URLs

### ❌ **Backend Gaps Identified**

1. **No HTTP URL generation** - Backend returns `icp://` URLs instead of HTTP URLs
2. **No token integration in asset URLs** - Backend doesn't embed tokens in asset URLs
3. **No HTTP gateway URL construction** - Backend doesn't create proper HTTP gateway URLs
4. **Frontend receives unusable URLs** - Frontend gets protocol URLs it can't use in browsers

---

## 🎯 **Implementation Plan**

### **Phase 1: Backend HTTP URL Generation** ⭐ **CRITICAL**

#### **1.1 Create Backend HTTP URL Generator**

**File**: `src/backend/src/http/url_generator.rs` (NEW)

```rust
/**
 * Backend HTTP URL Generation
 *
 * Generates authenticated HTTP URLs for ICP assets using token-based authentication.
 * This replaces the current icp:// protocol URLs with proper HTTP gateway URLs.
 */

use ic_cdk::api::canister_self;
use std::collections::HashMap;
use std::sync::Mutex;

// Cache for tokens to avoid repeated minting
static TOKEN_CACHE: Mutex<HashMap<String, (String, u64)>> = Mutex::new(HashMap::new());

pub struct HttpUrlGenerator;

impl HttpUrlGenerator {
    /// Generate HTTP URL for ICP asset with authentication token
    pub async fn generate_asset_url(
        memory_id: &str,
        variant: &str,
        asset_id: Option<&str>,
    ) -> Result<String, String> {
        // Get authentication token
        let token = Self::get_asset_token(memory_id, variant, asset_id).await?;

        // Get canister ID and network URL
        let canister_id = canister_self().to_text();
        let network_url = Self::get_network_url();

        // Construct HTTP URL
        let base_url = format!("{}/?canisterId={}", network_url, canister_id);
        let path = format!("/asset/{}/{}", memory_id, variant);

        let mut params = vec![("token".to_string(), token)];
        if let Some(id) = asset_id {
            params.push(("id".to_string(), id.to_string()));
        }

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let full_url = format!("{}{}?{}", base_url, path, query_string);

        ic_cdk::println!("Generated ICP HTTP asset URL: {}", full_url);
        Ok(full_url)
    }

    /// Get or mint authentication token for ICP asset access
    async fn get_asset_token(
        memory_id: &str,
        variant: &str,
        asset_id: Option<&str>,
    ) -> Result<String, String> {
        let cache_key = format!("{}-{}-{}", memory_id, variant, asset_id.unwrap_or("all"));

        // Check cache first
        if let Ok(cache) = TOKEN_CACHE.lock() {
            if let Some((token, expires_at)) = cache.get(&cache_key) {
                if *expires_at > ic_cdk::api::time() {
                    return Ok(token.clone());
                }
            }
        }

        // Mint new token
        let ttl_seconds = 180u32;
        let variants = vec![variant.to_string()];
        let asset_ids = asset_id.map(|id| vec![id.to_string()]);

        ic_cdk::println!("Minting ICP HTTP token for memory: {}", memory_id);

        let token = crate::mint_http_token(
            memory_id.to_string(),
            variants,
            asset_ids,
            ttl_seconds,
        ).await?;

        // Cache token with 2-minute expiry (1 minute before actual expiry)
        let expires_at = ic_cdk::api::time() + (2 * 60 * 1_000_000_000); // 2 minutes in nanoseconds
        if let Ok(mut cache) = TOKEN_CACHE.lock() {
            cache.insert(cache_key, (token.clone(), expires_at));
        }

        ic_cdk::println!("Successfully minted ICP HTTP token");
        Ok(token)
    }

    /// Get the appropriate network URL based on environment
    fn get_network_url() -> String {
        // In production, use ic0.app
        // In development, use localhost
        if cfg!(feature = "production") {
            "https://ic0.app".to_string()
        } else {
            "http://localhost:4943".to_string()
        }
    }

    /// Clear token cache (useful for testing or memory management)
    pub fn clear_token_cache() {
        if let Ok(mut cache) = TOKEN_CACHE.lock() {
            cache.clear();
            ic_cdk::println!("Cleared ICP token cache");
        }
    }
}
```

#### **1.2 Update Backend Asset Response Functions**

**File**: `src/backend/src/memories/core/assets.rs`

Update asset response functions to return HTTP URLs instead of `icp://` URLs:

```rust
// Update the asset response functions to use HTTP URLs
pub async fn asset_get_by_id_core<E: Env, S: Store>(
    env: &E,
    store: &S,
    memory_id: String,
    asset_id: String,
) -> Result<MemoryAssetData, Error> {
    // ... existing logic to get asset data ...

    // Instead of returning icp:// URLs, generate HTTP URLs
    match asset_data {
        MemoryAssetData::Inline { bytes, content_type, .. } => {
            // For inline assets, we can serve them directly via HTTP
            Ok(MemoryAssetData::Inline { bytes, content_type, .. })
        }
        MemoryAssetData::InternalBlob { blob_id, size, .. } => {
            // Generate HTTP URL for blob assets
            let http_url = crate::http::url_generator::HttpUrlGenerator::generate_asset_url(
                &memory_id,
                "original", // or determine variant based on asset_id
                Some(&asset_id),
            ).await.map_err(|e| Error::Internal(e))?;

            // Return as external URL pointing to our HTTP endpoint
            Ok(MemoryAssetData::ExternalUrl {
                url: http_url,
                size: Some(size),
                sha256: None, // Could be calculated if needed
            })
        }
        MemoryAssetData::ExternalUrl { url, .. } => {
            // If it's already an external URL, return as-is
            Ok(asset_data)
        }
    }
}
```

#### **1.3 Update Memory List Functions**

**File**: `src/backend/src/memories/core/list.rs`

Update memory list functions to return HTTP URLs for ICP assets:

```rust
// Update memory list functions to generate HTTP URLs for ICP assets
pub async fn memories_list_by_capsule_core<E: Env, S: Store>(
    env: &E,
    store: &S,
    capsule_id: String,
    cursor: Option<String>,
    limit: Option<u32>,
) -> Result<MemoryListResponse, Error> {
    // ... existing logic to get memories ...

    // For each memory, update asset URLs to use HTTP
    for memory in &mut memories {
        // Update inline assets
        for asset in &mut memory.inline_assets {
            // Inline assets can be served directly, no URL change needed
        }

        // Update blob internal assets
        for asset in &mut memory.blob_internal_assets {
            // Generate HTTP URL for blob assets
            if let Ok(http_url) = crate::http::url_generator::HttpUrlGenerator::generate_asset_url(
                &memory.id,
                "original", // or determine variant based on asset type
                Some(&asset.asset_id),
            ).await {
                // Update the asset to use HTTP URL
                asset.url = Some(http_url);
            }
        }

        // Update blob external assets
        for asset in &mut memory.blob_external_assets {
            // Generate HTTP URL for blob assets
            if let Ok(http_url) = crate::http::url_generator::HttpUrlGenerator::generate_asset_url(
                &memory.id,
                "original", // or determine variant based on asset type
                Some(&asset.asset_id),
            ).await {
                // Update the asset to use HTTP URL
                asset.url = Some(http_url);
            }
        }
    }

    Ok(MemoryListResponse { memories, cursor: next_cursor })
}
```

### **Phase 2: Frontend Configuration Updates** ⭐ **MEDIUM PRIORITY**

#### **2.1 Update Next.js Configuration**

**File**: `src/nextjs/next.config.ts`

Add ICP canister domains to the image configuration:

```typescript
// Update the images.remotePatterns section
remotePatterns: [
  // ... existing patterns ...

  // ICP canister domains for HTTP asset serving
  {
    protocol: 'https',
    hostname: '*.ic0.app',
    pathname: '/**',
  },
  {
    protocol: 'https',
    hostname: '*.icp0.io',
    pathname: '/**',
  },
  // Local development ICP
  {
    protocol: 'http',
    hostname: 'localhost',
    port: '4943',
    pathname: '/**',
  },
],
```

#### **2.2 No Frontend Code Changes Needed**

Since the backend will now return proper HTTP URLs with tokens, the frontend doesn't need any special handling for ICP assets. The existing image components will work automatically with the HTTP URLs returned by the backend.

### **Phase 3: No Frontend Changes Required** ⭐ **COMPLETED**

Since the backend will now return proper HTTP URLs with authentication tokens, the frontend will automatically work with ICP assets without any code changes. The existing image components and asset URL handling will work seamlessly with the HTTP URLs returned by the backend.

### **Phase 4: Backend Module Integration** ⭐ **CRITICAL**

#### **4.1 Add HTTP URL Generator to Backend Module**

**File**: `src/backend/src/lib.rs`

Add the HTTP URL generator module:

```rust
// Add to the existing module declarations
pub mod http {
    // ... existing http modules ...
    pub mod url_generator;
}

// Make the url_generator module public
pub use http::url_generator::HttpUrlGenerator;
```

#### **4.2 Update Backend Cargo.toml**

**File**: `src/backend/Cargo.toml`

Ensure the HTTP module dependencies are properly configured:

```toml
[dependencies]
# ... existing dependencies ...
ic-http-certification = "3.0.3"
ic-cdk = "0.18.0"
```

### **Phase 5: Backend Error Handling** ⭐ **MEDIUM PRIORITY**

#### **5.1 Add Error Handling to URL Generator**

**File**: `src/backend/src/http/url_generator.rs`

Add retry logic and error handling:

```rust
impl HttpUrlGenerator {
    /// Generate HTTP URL with retry logic
    pub async fn generate_asset_url_with_retry(
        memory_id: &str,
        variant: &str,
        asset_id: Option<&str>,
        max_retries: u32,
    ) -> Result<String, String> {
        let mut last_error = String::new();

        for attempt in 1..=max_retries {
            match Self::generate_asset_url(memory_id, variant, asset_id).await {
                Ok(url) => return Ok(url),
                Err(e) => {
                    last_error = e.clone();
                    if attempt < max_retries {
                        ic_cdk::println!("ICP asset URL generation attempt {} failed, retrying...", attempt);
                        // Note: In a real implementation, you'd need to use a different approach for delays
                        // since ic_cdk doesn't have sleep functionality
                    }
                }
            }
        }

        Err(format!("All {} retry attempts failed. Last error: {}", max_retries, last_error))
    }
}
```

---

## 🧪 **Testing Strategy**

### **Phase 1: Backend Unit Tests**

- Test HTTP URL generation with various asset types
- Test token minting and caching
- Test error handling and retry logic

### **Phase 2: Backend Integration Tests**

- Test with real ICP canister
- Test HTTP gateway integration
- Test token expiration and refresh

### **Phase 3: End-to-End Tests**

- Test complete asset serving flow via HTTP gateway
- Test with different asset variants
- Test error scenarios and fallbacks

---

## 📊 **Success Criteria**

### **Immediate (Phase 1-2)**

- [ ] Backend generates HTTP URLs instead of `icp://` protocol URLs
- [ ] Token-based authentication is working in backend
- [ ] 503 errors are resolved
- [ ] Frontend receives proper HTTP URLs from backend

### **Complete (All Phases)**

- [ ] All ICP assets serve via HTTP with proper authentication
- [ ] Backend error handling provides graceful fallbacks
- [ ] Performance is optimized with token caching
- [ ] All existing functionality remains intact
- [ ] Frontend works seamlessly without code changes
- [ ] Error handling provides good user experience

---

## 🚨 **Critical Dependencies**

1. **Backend HTTP module must be deployed** with skip certification
2. **HTTP URL generator must be implemented** in backend
3. **Asset response functions must be updated** to return HTTP URLs
4. **Token minting must work** with proper authentication

---

## 📝 **Implementation Order**

1. **Start with Phase 1** - Backend HTTP URL generation (most critical)
2. **Add Phase 2** - Frontend configuration updates (minimal)
3. **Implement Phase 3** - No frontend changes needed (completed)
4. **Configure Phase 4** - Backend module integration (required)
5. **Add Phase 5** - Backend error handling (polish)

---

## 🔗 **Related Documentation**

- [HTTP Certification 503 Error Analysis](./http-certification-503-error-analysis.md)
- [HTTP Certification Requirement Clarification](./http-certification-requirement-clarification.md)
- [Private Asset Serving Implementation Report](./private-asset-serving-implementation-report.md)
- [Tech Lead Feedback Implementation Plan](./tech-lead-feedback-implementation-plan.md)

---

## 📋 **Next Steps**

1. **Review and approve** this backend-focused implementation plan
2. **Implement Phase 1** - Backend HTTP URL generation (most critical)
3. **Update asset response functions** to return HTTP URLs instead of `icp://` URLs
4. **Test with real ICP canister** to verify 503 error resolution
5. **Implement remaining phases** based on testing results

---

**Priority**: 🔴 **CRITICAL**  
**Estimated Effort**: 1-2 days for Phase 1, 3-5 days for complete implementation  
**Dependencies**: Backend HTTP module deployment, HTTP URL generator implementation  
**Blocking**: ICP asset serving, 503 error resolution  
**Architecture**: Backend-centric approach - frontend requires no changes
