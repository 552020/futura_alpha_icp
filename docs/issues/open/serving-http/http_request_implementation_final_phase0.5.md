gotcha — no `mod.rs`. Here’s a **modern Rust** layout that avoids `mod.rs` while still keeping the HTTP stack nicely isolated.

# 📁 File layout (no `mod.rs`)

```
src/
  http.rs                      // root http module (declares submodules via #[path])
  http/
    request.rs                 // parse path/query/headers
    response.rs                // helpers to build HttpResponse
    auth.rs                    // HMAC token: types, sign, verify, scope checks
    secret.rs                  // stable secret init/rotation/get
    streaming.rs               // streaming strategy + callback token/handler
    routes/
      assets.rs                // /asset/{memory}/{variant}[?id=...] (touches storage)
      health.rs                // /health
```

## 1) `src/http.rs` (root module that wires submodules)

```rust
// src/http.rs
#![allow(clippy::needless_return)]

#[path = "http/request.rs"]   pub mod request;
#[path = "http/response.rs"]  pub mod response;
#[path = "http/auth.rs"]      pub mod auth;
#[path = "http/secret.rs"]    pub mod secret;
#[path = "http/streaming.rs"] pub mod streaming;

pub mod routes {
    #[path = "http/routes/assets.rs"] pub mod assets;
    #[path = "http/routes/health.rs"] pub mod health;
}

use ic_http_certification::{HttpRequest, HttpResponse};
use request::ParsedRequest;
use routes::{assets, health};

/// Main HTTP entrypoint router
pub fn handle(req: HttpRequest) -> HttpResponse {
    let parsed = match ParsedRequest::try_from(req) {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    match (parsed.method.as_str(), parsed.path_segments.as_slice()) {
        ("GET", ["health"]) => health::get(&parsed),
        // /asset/{memory_id}/{variant}
        ("GET", ["asset", mem, var]) => assets::get(mem, var, &parsed),
        _ => response::not_found(),
    }
}
```

## 2) Wire it in `src/lib.rs`

```rust
mod http; // uses src/http.rs

#[ic_cdk::init]
fn init() {
    http::secret::init_secret();       // stable HMAC key (Phase 1)
    // ...your existing init
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    http::secret::post_upgrade_secret();// rotate on upgrade
    // ...your existing post_upgrade
}

#[ic_cdk::query]
fn http_request(req: ic_http_certification::HttpRequest) -> ic_http_certification::HttpResponse {
    http::handle(req)
}

#[ic_cdk::query]
fn http_request_streaming_callback(token: http::streaming::CallbackToken)
    -> http::streaming::CallbackResponse
{
    http::streaming::callback(token)
}
```

## 3) Minimal skeletons (drop-in)

### `src/http/request.rs`

```rust
use ic_http_certification::HttpRequest;
use crate::http::response;

pub struct ParsedRequest {
    pub method: String,
    pub path_segments: Vec<String>,
    pub query: Vec<(String, String)>, // simple k=v pairs
}

impl TryFrom<HttpRequest> for ParsedRequest {
    type Error = ic_http_certification::HttpResponse;
    fn try_from(req: HttpRequest) -> Result<Self, Self::Error> {
        let method = req.method.to_uppercase();
        // split "/a/b?x=1" into segments & query
        let (path, query_str) = req.url.split_once('?').unwrap_or((&req.url[..], ""));
        let path_segments = path.trim_start_matches('/').split('/')
            .filter(|s| !s.is_empty()).map(|s| s.to_string()).collect::<Vec<_>>();
        let query = query_str.split('&').filter(|s| !s.is_empty())
            .filter_map(|kv| kv.split_once('=').map(|(k,v)| (k.to_string(), v.to_string())))
            .collect::<Vec<_>>();
        if method.is_empty() { return Err(response::bad_request("invalid method")); }
        Ok(Self { method, path_segments, query })
    }
}

impl ParsedRequest {
    pub fn q(&self, name: &str) -> Option<&str> {
        self.query.iter().find(|(k,_)| k == name).map(|(_,v)| v.as_str())
    }
}
```

### `src/http/response.rs`

```rust
use ic_http_certification::{HttpResponse, HttpStreamingStrategy};

pub fn ok(bytes: Vec<u8>, ct: &str) -> HttpResponse {
    HttpResponse {
        status_code: 200,
        headers: vec![
            ("Content-Type".into(), ct.into()),
            ("Cache-Control".into(), "private, no-store".into()),
        ],
        body: bytes,
        upgrade: None,
        streaming_strategy: None,
    }
}

pub fn stream(headers: Vec<(String,String)>, strategy: HttpStreamingStrategy) -> HttpResponse {
    HttpResponse {
        status_code: 200,
        headers,
        body: vec![],
        upgrade: None,
        streaming_strategy: Some(strategy),
    }
}

pub fn bad_request(msg: &str) -> HttpResponse {
    HttpResponse { status_code: 400, headers: vec![("Content-Type".into(), "text/plain".into())], body: msg.as_bytes().to_vec(), upgrade: None, streaming_strategy: None }
}
pub fn unauthorized() -> HttpResponse { status(401, "Unauthorized") }
pub fn forbidden()    -> HttpResponse { status(403, "Forbidden") }
pub fn not_found()    -> HttpResponse { status(404, "Not Found") }

fn status(code: u16, msg: &str) -> HttpResponse {
    HttpResponse { status_code: code, headers: vec![("Content-Type".into(), "text/plain".into())], body: msg.as_bytes().to_vec(), upgrade: None, streaming_strategy: None }
}
```

### `src/http/auth.rs` (Phase-1 HMAC token helpers)

> Use the Phase-1 structs you already approved (HMAC secret in `secret.rs`, payload, encode/decode, verify). Keep it **stateless**.

### `src/http/secret.rs` (Phase-1 secret mgmt)

> Your stable-secret init/rotation code from Phase-1.

### `src/http/streaming.rs`

> Define `CallbackToken` (include memory_id, asset_id, byte range, and a server session id) and a `callback()` that re-checks auth and pulls next chunk. Keep it small for now; you can flesh it out in Phase-2.

### `src/http/routes/health.rs`

```rust
use crate::http::{request::ParsedRequest, response};

pub fn get(_: &ParsedRequest) -> ic_http_certification::HttpResponse {
    response::ok(b"OK".to_vec(), "text/plain")
}
```

### `src/http/routes/assets.rs` (outline)

```rust
use crate::http::{request::ParsedRequest, response, auth};
use ic_http_certification::HttpResponse;

pub fn get(memory_id: &str, variant: &str, req: &ParsedRequest) -> HttpResponse {
    // 1) verify token against path scope
    let token = req.q("token");
    let asset_id = req.q("id");
    if let Err(_) = auth::verify_http_query_token(token, memory_id, variant, asset_id) {
        return response::forbidden();
    }

    // 2) resolve asset from your storage (memories + upload::blob_store)
    //    -> choose inline vs streaming based on size
    // TODO: call your existing adapters and return response::ok(...) or response::stream(...)

    response::not_found() // placeholder
}
```

---

## Why this is “modern”

- No `mod.rs`; one **`http.rs`** root module declares submodules explicitly with `#[path = "..."]`.
- All HTTP surface (routing, auth, secret, streaming) is **isolated** behind `http::*`.
- Your domain (`memories`, `upload`, etc.) stays unchanged; only `routes/assets.rs` imports them.

If you want, I can tailor the `assets.rs` to your exact storage APIs (`memories::core::read`, `upload::blob_store::read_chunk`, etc.) next.
