// src/http.rs
#![allow(clippy::needless_return)]

#[path = "http/request.rs"]   pub mod request;
#[path = "http/response.rs"]  pub mod response;
#[path = "http/auth.rs"]      pub mod auth;
#[path = "http/secret.rs"]    pub mod secret;
// #[path = "http/streaming.rs"] pub mod streaming; // TODO: Enable when streaming is needed

#[path = "http/routes/assets.rs"] pub mod assets;
#[path = "http/routes/health.rs"] pub mod health;

use ic_http_certification::{HttpRequest, HttpResponse};
use request::ParsedRequest;

/// Main HTTP entrypoint router
pub fn handle(req: HttpRequest) -> HttpResponse<'static> {
    let parsed = match ParsedRequest::try_from(req) {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    match (parsed.method.as_str(), parsed.path_segments.as_slice()) {
        ("GET", [health]) if health == "health" => health::get(&parsed),
        // /asset/{memory_id}/{variant}
        ("GET", [asset, mem, var]) if asset == "asset" => assets::get(mem, var, &parsed),
        _ => response::not_found(),
    }
}
