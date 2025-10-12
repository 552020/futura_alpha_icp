// src/http.rs
#![allow(clippy::needless_return)]

// Core modules (pure business logic)
#[path = "http/core/types.rs"]      pub mod core_types;
#[path = "http/core/auth_core.rs"]  pub mod auth_core;
#[path = "http/core/path_core.rs"]  pub mod path_core;

// Adapter modules (ICP integration layer)
#[path = "http/adapters/canister_env.rs"]  pub mod canister_env;
#[path = "http/adapters/secret_store.rs"]  pub mod secret_store;
#[path = "http/adapters/asset_store.rs"]   pub mod asset_store;
#[path = "http/adapters/acl.rs"]           pub mod acl;

// Route modules (thin HTTP handlers)
#[path = "http/routes/health.rs"]   pub mod health_route;
#[path = "http/routes/assets.rs"]   pub mod assets_route;

use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use core_types::ParsedRequest;

// Tiny parser (pure) to keep routes simple
fn parse(req: HttpRequest) -> Result<ParsedRequest, HttpResponse<'static>> {
    let method = req.method().to_string().to_uppercase();
    let (path, qs) = req.url().split_once('?').unwrap_or((&req.url()[..], ""));
    let path_segments = path.trim_start_matches('/').split('/')
        .filter(|s| !s.is_empty()).map(|s| s.to_string()).collect::<Vec<_>>();
    let query = qs.split('&').filter(|s| !s.is_empty())
        .filter_map(|kv| kv.split_once('=').map(|(k,v)| (k.to_string(), v.to_string())))
        .collect::<Vec<_>>();
    if method.is_empty() {
        return Err(HttpResponse::builder()
            .with_status_code(StatusCode::BAD_REQUEST)
            .with_headers(vec![("Content-Type".into(), "text/plain".into())])
            .with_body(b"invalid method")
            .build());
    }
    Ok(ParsedRequest { method, path_segments, query })
}

/// Main HTTP entrypoint router
pub fn handle(req: HttpRequest) -> HttpResponse<'static> {
    let parsed = match parse(req) {
        Ok(p) => p,
        Err(r) => return r,
    };

    match (parsed.method.as_str(), parsed.path_segments.as_slice()) {
        ("GET", [health]) if health == "health" => health_route::get(&parsed),
        // /asset/{memory_id}/{variant}
        ("GET", [asset, mem, var]) if asset == "asset" => assets_route::get(mem, var, &parsed),
        _ => HttpResponse::builder()
            .with_status_code(StatusCode::NOT_FOUND)
            .with_headers(vec![("Content-Type".into(), "text/plain".into())])
            .with_body(b"Not Found")
            .build(),
    }
}
