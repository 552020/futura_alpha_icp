// src/http.rs
#![allow(clippy::needless_return)]

// Core modules (pure business logic)
#[path = "http/core/auth_core.rs"]
pub mod auth_core;
#[path = "http/core/types.rs"]
pub mod core_types;
#[path = "http/core/path_core.rs"]
pub mod path_core;

// Adapter modules (ICP integration layer)
#[path = "http/adapters/acl.rs"]
pub mod acl;
#[path = "http/adapters/asset_store.rs"]
pub mod asset_store;
#[path = "http/adapters/canister_env.rs"]
pub mod canister_env;
#[path = "http/adapters/secret_store.rs"]
pub mod secret_store;

// Service modules (business logic)
#[path = "http/services/token_service.rs"]
pub mod token_service;

// Route modules (thin HTTP handlers)
#[path = "http/routes/assets.rs"]
pub mod assets_route;
#[path = "http/routes/health.rs"]
pub mod health_route;

/// Generate relative asset path (no base URL)
#[allow(dead_code)]
pub fn asset_path(memory_id: &str, kind: &str) -> String {
    format!("/asset/{}/{}", memory_id, kind)
}

use core_types::ParsedRequest;
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use percent_encoding::percent_decode_str;

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

// Tiny parser (pure) to keep routes simple
fn parse(req: HttpRequest) -> Result<ParsedRequest, HttpResponse<'static>> {
    let method = req.method().to_string().to_uppercase();
    // keep the string alive
    let url = req.url().to_string();

    // Debug logging to see what URL we're actually receiving
    ic_cdk::println!("🔍 HTTP Request Debug:");
    ic_cdk::println!("  Method: {}", method);
    ic_cdk::println!("  Full URL: {}", url);
    ic_cdk::println!("  Token present: {}", qs_get(&url, "token").is_some());
    ic_cdk::println!("  ID present: {}", qs_get(&url, "id").is_some());

    // path segments
    let (path, _qs) = url.split_once('?').unwrap_or((url.as_str(), ""));
    let path_segments: Vec<String> = path
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    // Build query params for backward compatibility with existing code
    let query = _qs
        .split('&')
        .filter(|s| !s.is_empty())
        .filter_map(|kv| {
            kv.split_once('=')
                .map(|(k, v)| (k.to_string(), v.to_string()))
        })
        .collect::<Vec<_>>();

    // Extract headers (case-insensitive)
    let headers = req
        .headers()
        .iter()
        .map(|(k, v)| (k.to_lowercase(), v.clone()))
        .collect::<Vec<_>>();

    ic_cdk::println!("  Path Segments: {:?}", path_segments);
    ic_cdk::println!("  Query Params: {:?}", query);
    ic_cdk::println!("  Headers: {:?}", headers.len());

    if method.is_empty() {
        return Err(HttpResponse::builder()
            .with_status_code(StatusCode::BAD_REQUEST)
            .with_headers(vec![("Content-Type".into(), "text/plain".into())])
            .with_body(b"invalid method")
            .build());
    }
    Ok(ParsedRequest {
        method,
        path_segments,
        query,
        headers,
    })
}

/// Main HTTP entrypoint router
pub fn handle(req: HttpRequest) -> HttpResponse<'static> {
    let url = req.url().to_string();
    let parsed = match parse(req) {
        Ok(p) => p,
        Err(r) => return r,
    };

    // Debug logging for HTTP routing
    ic_cdk::println!(
        "[HTTP-ROUTER] method={} path_segments={:?} url={}",
        parsed.method,
        parsed.path_segments,
        url
    );

    match (parsed.method.as_str(), parsed.path_segments.as_slice()) {
        ("GET", [health]) if health == "health" => health_route::get(&parsed),
        // /asset/{memory_id}/{variant}
        ("GET", [asset, mem, var]) if asset == "asset" => {
            ic_cdk::println!(
                "[HTTP-ROUTER] ✅ Matched asset route: mem={} var={}",
                mem,
                var
            );
            assets_route::get(mem, var, &parsed, &url)
        }
        _ => {
            ic_cdk::println!("[HTTP-ROUTER] ❌ No route matched, returning 404");
            HttpResponse::builder()
                .with_status_code(StatusCode::NOT_FOUND)
                .with_headers(vec![("Content-Type".into(), "text/plain".into())])
                .with_body(b"Not Found")
                .build()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qs_get_handles_multiple_params_and_equals() {
        let url = "/asset/mid/preview?token=a==&id=xyz";
        assert_eq!(qs_get(url, "token").as_deref(), Some("a=="));
        assert_eq!(qs_get(url, "id").as_deref(), Some("xyz"));
        assert_eq!(qs_get(url, "missing"), None);
    }

    #[test]
    fn qs_get_percent_decodes_values() {
        let url = "/x?token=abc%2Bdef%2Fghi";
        assert_eq!(qs_get(url, "token").as_deref(), Some("abc+def/ghi"));
    }

    #[test]
    fn qs_get_handles_empty_values() {
        let url = "/x?token=&id=xyz";
        assert_eq!(qs_get(url, "token").as_deref(), Some(""));
        assert_eq!(qs_get(url, "id").as_deref(), Some("xyz"));
    }

    #[test]
    fn qs_get_handles_no_query_string() {
        let url = "/asset/mid/preview";
        assert_eq!(qs_get(url, "token"), None);
    }

    #[test]
    fn qs_get_handles_parameter_order() {
        let url1 = "/x?token=abc&id=xyz";
        let url2 = "/x?id=xyz&token=abc";
        assert_eq!(qs_get(url1, "token").as_deref(), Some("abc"));
        assert_eq!(qs_get(url2, "token").as_deref(), Some("abc"));
        assert_eq!(qs_get(url1, "id").as_deref(), Some("xyz"));
        assert_eq!(qs_get(url2, "id").as_deref(), Some("xyz"));
    }
}
