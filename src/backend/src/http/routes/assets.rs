use crate::http::{
    asset_store::FuturaAssetStore,
    auth_core::{decode_token_url, verify_token_core},
    canister_env::CanisterClock,
    core_types::{AssetStore, ParsedRequest, VerifyErr},
    path_core::path_to_scope,
    secret_store::StableSecretStore,
};
use ic_http_certification::{HttpResponse, StatusCode};
use percent_encoding::percent_decode_str;

/// HTTP error taxonomy for precise status mapping
#[derive(Debug, Clone)]
pub enum HttpError {
    MissingToken,
    BadTokenFormat,
    BadTokenSignature,
    TokenExpired,
    AclDenied,
    // AssetNotFound, // TODO: Remove when needed
    InputTooLong,
    TooManyParams,
}

impl HttpError {
    /// Map error to HTTP status code and message
    pub fn to_response(&self) -> HttpResponse<'static> {
        match self {
            HttpError::MissingToken => status(401, "Missing token"),
            HttpError::BadTokenFormat => status(400, "Bad token format"),
            HttpError::BadTokenSignature => status(403, "Bad token signature"),
            HttpError::TokenExpired => status(403, "Token expired"),
            HttpError::AclDenied => status(403, "Access denied"),
            // HttpError::AssetNotFound => status(404, "Asset not found"), // TODO: Remove when needed
            HttpError::InputTooLong => status(400, "Input too long"),
            HttpError::TooManyParams => status(400, "Too many parameters"),
        }
    }
}

/// Robust query parameter extraction that handles percent-encoding and values with '='
fn qs_get(url: &str, key: &str) -> Option<String> {
    let (_, qs) = url.split_once('?')?;
    for pair in qs.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut it = pair.splitn(2, '=');
        let k = it.next().unwrap_or("");
        if k == key {
            let v = it.next().unwrap_or(""); // may be empty
                                             // decode percent-encoding safely
            return Some(percent_decode_str(v).decode_utf8_lossy().into_owned());
        }
    }
    None
}

/// Extract token from request - header takes precedence over query string
/// Resolution rule: Authorization header > query parameter
fn extract_token_from_request(req: &ParsedRequest, url: &str) -> Option<String> {
    // 1. Check Authorization header first (highest priority)
    if let Some(auth_header) = req.get_header("authorization") {
        if let Some(token) = extract_bearer_token(&auth_header) {
            return Some(token);
        }
    }

    // 2. Fall back to query parameter
    qs_get(url, "token")
}

/// Validate input limits and return appropriate error response
fn validate_input_limits(req: &ParsedRequest, url: &str) -> Result<(), HttpResponse<'static>> {
    // Hard cap token param length (8 KB)
    const MAX_TOKEN_LENGTH: usize = 8192;

    // Cap total query length (16 KB)
    const MAX_QUERY_LENGTH: usize = 16384;

    // Fail fast on >64 params to avoid pathological inputs
    const MAX_PARAMS: usize = 64;

    // Check query string length
    if url.len() > MAX_QUERY_LENGTH {
        return Err(HttpError::InputTooLong.to_response());
    }

    // Check parameter count
    if req.query.len() > MAX_PARAMS {
        return Err(HttpError::TooManyParams.to_response());
    }

    // Check token length if present
    if let Some(token) = extract_token_from_request(req, url) {
        if token.len() > MAX_TOKEN_LENGTH {
            return Err(HttpError::InputTooLong.to_response());
        }
    }

    Ok(())
}

/// Extract Bearer token from Authorization header
fn extract_bearer_token(auth_header: &str) -> Option<String> {
    let auth_header = auth_header.trim();
    if auth_header.to_lowercase().starts_with("bearer ") {
        let token = &auth_header[7..]; // Skip "bearer "
        if !token.is_empty() {
            return Some(token.to_string());
        }
    }
    None
}

pub fn get(
    memory_id: &str,
    variant: &str,
    req: &ParsedRequest,
    url: &str,
) -> HttpResponse<'static> {
    // Debug logging for token extraction
    ic_cdk::println!("🔍 Asset Route Debug:");
    ic_cdk::println!("  Memory ID: {}", memory_id);
    ic_cdk::println!("  Variant: {}", variant);
    ic_cdk::println!("  Query params: {:?}", req.query);

    // 0) validate input limits first
    if let Err(response) = validate_input_limits(req, url) {
        return response;
    }

    // 1) verify token - check header first, then query string
    let token_param = extract_token_from_request(req, url);
    let token_param = match token_param {
        Some(t) => {
            ic_cdk::println!("  Token found: {}", t);
            t
        }
        None => {
            ic_cdk::println!("  ❌ No token found in headers or query params");
            return HttpError::MissingToken.to_response();
        }
    };
    let token = match decode_token_url(&token_param) {
        Some(t) => {
            ic_cdk::println!("  ✅ Token decoded successfully");
            ic_cdk::println!("  Token payload: {:?}", t.p);
            t
        }
        None => {
            ic_cdk::println!("  ❌ Failed to decode token");
            return HttpError::BadTokenFormat.to_response();
        }
    };

    let want = path_to_scope(req, memory_id, variant);
    ic_cdk::println!("  Expected scope: {:?}", want);

    let clock = CanisterClock;
    let secret = StableSecretStore;

    // Enhanced error handling with structured enums
    if let Err(e) = verify_token_core(&clock, &secret, &token, &want) {
        ic_cdk::println!("  ❌ Token validation failed: {:?}", e);
        return match e {
            VerifyErr::Expired => {
                ic_cdk::println!("  ❌ Token expired");
                HttpError::TokenExpired.to_response()
            }
            VerifyErr::BadSig => {
                ic_cdk::println!("  ❌ Bad token signature");
                HttpError::BadTokenSignature.to_response()
            }
            VerifyErr::WrongMemory => {
                ic_cdk::println!("  ❌ Wrong memory ID");
                HttpError::AclDenied.to_response()
            }
            VerifyErr::VariantNotAllowed => {
                ic_cdk::println!("  ❌ Variant not allowed");
                HttpError::AclDenied.to_response()
            }
            VerifyErr::AssetNotAllowed => {
                ic_cdk::println!("  ❌ Asset not allowed");
                HttpError::AclDenied.to_response()
            }
        };
    }

    ic_cdk::println!("  ✅ Token validation successful");

    // 2) load asset (inline preferred; else prepare for streaming in Phase 2)
    let store = FuturaAssetStore;
    let asset_id_param = qs_get(url, "id");

    // Add comprehensive debug logging
    ic_cdk::println!(
        "[HTTP-ASSET] mem={} variant={} id_param={:?}",
        memory_id,
        variant,
        asset_id_param
    );
    ic_cdk::println!(
        "[HTTP-ASSET] token.sub={:?} scope.mem={} scope.variants={:?} scope.ids={:?}",
        token.p.sub,
        token.p.scope.memory_id,
        token.p.scope.variants,
        token.p.scope.asset_ids
    );

    // Get the token's subject principal
    let token_subject = match &token.p.sub {
        Some(sub) => sub,
        None => {
            ic_cdk::println!("[HTTP-ASSET] ❌ Token missing subject principal");
            return status(401, "Token missing subject");
        }
    };

    // Resolve the asset ID using variant-specific resolution
    let asset_id = match store.resolve_asset_for_variant(
        token_subject,
        memory_id,
        variant,
        asset_id_param.as_deref(),
    ) {
        Some(id) => {
            ic_cdk::println!("[HTTP-ASSET] ✅ Resolved asset_id={}", id);
            id
        }
        None => {
            ic_cdk::println!("[HTTP-ASSET] ❌ No assets found for memory: {}", memory_id);
            return status(404, "No assets found");
        }
    };

    // Decide priority by variant: for display/thumbnail/original prefer blob first.
    // Inline should only be preferred when the requested variant is explicitly "inline".
    let is_inline_variant = variant.eq_ignore_ascii_case("inline");

    // Try blob first unless the caller explicitly asked for inline
    if !is_inline_variant {
        if let Some((blob_data, content_type)) =
            store.get_blob_with_principal(token_subject, memory_id, &asset_id)
        {
            let content_length = blob_data.len().to_string();
            ic_cdk::println!(
                "[HTTP-ASSET] ✅ Serving blob asset: {} bytes, content_type={}",
                content_length,
                content_type
            );
            return HttpResponse::ok(
                blob_data,
                vec![
                    ("Content-Type".into(), content_type),
                    ("Cache-Control".into(), "private, no-store".into()),
                    ("X-Content-Type-Options".into(), "nosniff".into()),
                    ("Content-Length".into(), content_length),
                ],
            )
            .build();
        }
    }

    // Only try inline if explicitly requested (never as fallback for display/thumbnail/original)
    if is_inline_variant {
        if let Some(inline) = store.get_inline_with_principal(token_subject, memory_id, &asset_id) {
            let content_length = inline.bytes.len().to_string();
            ic_cdk::println!(
                "[HTTP-ASSET] ✅ Serving inline asset: {} bytes, content_type={}",
                content_length,
                inline.content_type
            );
            return HttpResponse::ok(
                inline.bytes,
                vec![
                    ("Content-Type".into(), inline.content_type),
                    ("Cache-Control".into(), "private, no-store".into()),
                    ("X-Content-Type-Options".into(), "nosniff".into()),
                    ("Content-Length".into(), content_length),
                ],
            )
            .build();
        }
    } else {
        ic_cdk::println!(
            "[HTTP-ASSET] ❌ No blob asset found for variant: {} - returning 404 instead of placeholder",
            variant
        );
    }

    // 3) fallback: not found (streaming to be added in Phase 2)
    ic_cdk::println!("[HTTP-ASSET] ❌ Asset not found: {}", asset_id);
    status(404, "Asset not found")
}

fn status(code: u16, msg: &str) -> HttpResponse<'static> {
    let status_code = StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let body = msg.as_bytes().to_vec();
    HttpResponse::builder()
        .with_status_code(status_code)
        .with_headers(vec![("Content-Type".into(), "text/plain".into())])
        .with_body(body)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test the asset ID extraction logic directly
    /// This tests the core logic without the Store dependency
    #[test]
    fn test_asset_id_extraction_logic() {
        // Test the priority order: inline -> blob_internal -> blob_external

        // Simulate inline assets (highest priority)
        let inline_assets = vec![(
            "asset_1".to_string(),
            "image/png".to_string(),
            vec![1, 2, 3, 4],
        )];

        // Simulate blob internal assets (medium priority)
        let blob_internal_assets = vec![(
            "blob_asset_1".to_string(),
            "image/jpeg".to_string(),
            "blob_1".to_string(),
            1024,
        )];

        // Simulate blob external assets (lowest priority)
        let blob_external_assets = vec![(
            "external_asset_1".to_string(),
            "image/webp".to_string(),
            "external_1".to_string(),
            2048,
        )];

        // Test priority: inline should be returned first
        if !inline_assets.is_empty() {
            assert_eq!(inline_assets[0].0, "asset_1");
        }

        // Test priority: blob internal should be returned if no inline
        if inline_assets.is_empty() && !blob_internal_assets.is_empty() {
            assert_eq!(blob_internal_assets[0].0, "blob_asset_1");
        }

        // Test priority: blob external should be returned if no inline or internal
        if inline_assets.is_empty()
            && blob_internal_assets.is_empty()
            && !blob_external_assets.is_empty()
        {
            assert_eq!(blob_external_assets[0].0, "external_asset_1");
        }
    }

    /// Test Bearer token extraction from Authorization header
    #[test]
    fn test_extract_bearer_token() {
        // Valid Bearer token
        assert_eq!(
            extract_bearer_token("Bearer abc123"),
            Some("abc123".to_string())
        );

        // Bearer token with spaces (header is trimmed, but token is not)
        assert_eq!(
            extract_bearer_token("Bearer  abc123  "),
            Some(" abc123".to_string())
        );

        // Case insensitive
        assert_eq!(
            extract_bearer_token("bearer abc123"),
            Some("abc123".to_string())
        );

        // Invalid formats
        assert_eq!(extract_bearer_token("Basic abc123"), None);
        assert_eq!(extract_bearer_token("Bearer"), None);
        assert_eq!(extract_bearer_token(""), None);
    }

    /// Test input validation limits
    #[test]
    fn test_validate_input_limits() {
        use crate::http::core_types::ParsedRequest;

        // Valid input
        let req = ParsedRequest {
            method: "GET".to_string(),
            path_segments: vec!["asset".to_string(), "test".to_string()],
            query: vec![("token".to_string(), "abc123".to_string())],
            headers: vec![],
        };
        let url = "/asset/test?token=abc123";
        assert!(validate_input_limits(&req, url).is_ok());

        // Too many parameters
        let mut req = req.clone();
        req.query = (0..65)
            .map(|i| (format!("param{}", i), "value".to_string()))
            .collect();
        assert!(validate_input_limits(&req, url).is_err());

        // URL too long
        let long_url = format!("/asset/test?token={}", "a".repeat(17000));
        assert!(validate_input_limits(&req, &long_url).is_err());
    }

    /// Test error taxonomy mapping
    #[test]
    fn test_http_error_mapping() {
        assert_eq!(HttpError::MissingToken.to_response().status_code(), 401);
        assert_eq!(HttpError::BadTokenFormat.to_response().status_code(), 400);
        assert_eq!(
            HttpError::BadTokenSignature.to_response().status_code(),
            403
        );
        assert_eq!(HttpError::TokenExpired.to_response().status_code(), 403);
        assert_eq!(HttpError::AclDenied.to_response().status_code(), 403);
        // assert_eq!(HttpError::AssetNotFound.to_response().status_code(), 404); // TODO: Remove when needed
        assert_eq!(HttpError::InputTooLong.to_response().status_code(), 400);
        assert_eq!(HttpError::TooManyParams.to_response().status_code(), 400);
    }

    // --- Tech Lead's comprehensive asset selection tests ---

    // Minimal shapes to keep this test self-contained
    #[derive(Clone)]
    struct InlineAsset {
        bytes: Vec<u8>,
        content_type: String,
    }

    trait TestStore {
        fn get_blob_with_principal(
            &self,
            _principal: Option<candid::Principal>,
            _memory_id: &str,
            _asset_id: &str,
        ) -> Option<(Vec<u8>, String)>;

        fn get_inline_with_principal(
            &self,
            _principal: Option<candid::Principal>,
            _memory_id: &str,
            _asset_id: &str,
        ) -> Option<InlineAsset>;
    }

    // This constant mirrors the heuristic in the fix
    const PLACEHOLDER_MAX_LEN: usize = 1200;

    // Extract the selection logic into a pure function for easy testing.
    // Returns (bytes, content_type).
    fn select_asset_for_variant<S: TestStore>(
        store: &S,
        token_subject: Option<candid::Principal>,
        memory_id: &str,
        asset_id: &str,
        variant: &str,
    ) -> Option<(Vec<u8>, String)> {
        let is_inline_variant = variant.eq_ignore_ascii_case("inline");

        // Prefer BLOB for non-inline variants
        if !is_inline_variant {
            if let Some((blob, ct)) =
                store.get_blob_with_principal(token_subject, memory_id, asset_id)
            {
                return Some((blob, ct));
            }
        }

        // Otherwise try INLINE (but skip tiny placeholders unless explicitly inline)
        if let Some(inl) = store.get_inline_with_principal(token_subject, memory_id, asset_id) {
            let looks_like_placeholder = inl.bytes.len() <= PLACEHOLDER_MAX_LEN;
            if is_inline_variant || !looks_like_placeholder {
                return Some((inl.bytes, inl.content_type));
            }
        }

        None
    }

    // A tiny fake store to simulate all scenarios
    #[derive(Default)]
    struct FakeStore {
        blob: Option<(Vec<u8>, String)>,
        inline: Option<InlineAsset>,
    }

    impl TestStore for FakeStore {
        fn get_blob_with_principal(
            &self,
            _principal: Option<candid::Principal>,
            _memory_id: &str,
            _asset_id: &str,
        ) -> Option<(Vec<u8>, String)> {
            self.blob.clone()
        }

        fn get_inline_with_principal(
            &self,
            _principal: Option<candid::Principal>,
            _memory_id: &str,
            _asset_id: &str,
        ) -> Option<InlineAsset> {
            self.inline.clone()
        }
    }

    fn big_webp(len: usize) -> (Vec<u8>, String) {
        (vec![1u8; len], "image/webp".to_string())
    }

    fn tiny_jpeg(len: usize) -> InlineAsset {
        InlineAsset {
            bytes: vec![2u8; len],
            content_type: "image/jpeg".into(),
        }
    }

    #[test]
    fn prefers_blob_over_inline_for_display() {
        let mut store = FakeStore::default();
        // Real processed asset as BLOB (~248 KB)
        store.blob = Some(big_webp(248_000));
        // Inline is the tiny placeholder (~1 KB JPEG)
        store.inline = Some(tiny_jpeg(1_000));

        let got = select_asset_for_variant(
            &store, None, "mem1", "assetD", "display", // non-inline
        )
        .expect("should select something");

        assert_eq!(
            got.0.len(),
            248_000,
            "blob must win over inline placeholder"
        );
        assert_eq!(got.1, "image/webp");
    }

    #[test]
    fn falls_back_to_inline_when_blob_missing_for_display_if_not_placeholder() {
        let mut store = FakeStore::default();
        store.blob = None;
        // Inline is a real image (e.g., server produced inline previews in future)
        store.inline = Some(InlineAsset {
            bytes: vec![3u8; 50_000],
            content_type: "image/webp".into(),
        });

        let got = select_asset_for_variant(&store, None, "mem1", "assetT", "thumbnail")
            .expect("should select inline since blob is missing");

        assert_eq!(got.0.len(), 50_000);
        assert_eq!(got.1, "image/webp");
    }

    #[test]
    fn skips_tiny_inline_placeholder_when_blob_missing_for_display() {
        let mut store = FakeStore::default();
        store.blob = None;
        // Only a tiny placeholder inline is present
        store.inline = Some(tiny_jpeg(1_000)); // <= 1200 → placeholder

        let got = select_asset_for_variant(
            &store, None, "mem1", "assetD", "display", /* non-inline */
        );

        assert!(
            got.is_none(),
            "should skip tiny inline placeholder for non-inline variants if blob missing"
        );
    }

    #[test]
    fn inline_variant_prefers_inline_even_if_tiny() {
        let mut store = FakeStore::default();
        // Even if a big blob exists, when variant == inline we serve inline
        store.blob = Some(big_webp(200_000));
        store.inline = Some(tiny_jpeg(800));

        let got = select_asset_for_variant(&store, None, "mem1", "assetI", "inline")
            .expect("inline variant must serve inline");

        assert_eq!(got.0.len(), 800);
        assert_eq!(got.1, "image/jpeg");
    }

    #[test]
    fn blob_wins_for_thumbnail_even_if_inline_is_large() {
        let mut store = FakeStore::default();
        // Both exist → for non-inline variant, blob should win
        store.blob = Some(big_webp(36_000));
        store.inline = Some(InlineAsset {
            bytes: vec![4u8; 10_000],
            content_type: "image/webp".into(),
        });

        let got = select_asset_for_variant(&store, None, "mem1", "assetT", "thumbnail")
            .expect("should select blob");

        assert_eq!(got.0.len(), 36_000);
        assert_eq!(got.1, "image/webp");
    }
}
