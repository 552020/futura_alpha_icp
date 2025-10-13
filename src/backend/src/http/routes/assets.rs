use crate::http::{
    asset_store::FuturaAssetStore,
    auth_core::{decode_token_url, verify_token_core},
    canister_env::CanisterClock,
    core_types::{AssetStore, ParsedRequest, VerifyErr},
    path_core::path_to_scope,
    secret_store::StableSecretStore,
};
use ic_http_certification::{HttpResponse, StatusCode};

pub fn get(memory_id: &str, variant: &str, req: &ParsedRequest) -> HttpResponse<'static> {
    // 1) verify token
    let token_param = match req.q("token") {
        Some(t) => t,
        None => return status(401, "Missing token"),
    };
    let token = match decode_token_url(token_param) {
        Some(t) => t,
        None => return status(403, "Bad token"),
    };
    let want = path_to_scope(req, memory_id, variant);
    let clock = CanisterClock;
    let secret = StableSecretStore;
    // Enhanced error handling with structured enums
    if let Err(e) = verify_token_core(&clock, &secret, &token, &want) {
        return match e {
            VerifyErr::Expired => status(401, "Token expired"),
            VerifyErr::BadSig => status(403, "Invalid token signature"),
            VerifyErr::WrongMemory => status(403, "Token not valid for this memory"),
            VerifyErr::VariantNotAllowed => status(403, "Token not valid for this variant"),
            VerifyErr::AssetNotAllowed => status(403, "Token not valid for this asset"),
        };
    }

    // 2) load asset (inline preferred; else prepare for streaming in Phase 2)
    let store = FuturaAssetStore;
    let asset_id = req.q("id").unwrap_or("");
    if let Some(inline) = store.get_inline(memory_id, asset_id) {
        let content_length = inline.bytes.len().to_string();
        return HttpResponse::ok(
            inline.bytes,
            vec![
                ("Content-Type".into(), inline.content_type),
                ("Cache-Control".into(), "private, no-store".into()),
                ("Content-Length".into(), content_length),
            ],
        )
        .build();
    }

    // 3) fallback: not found (streaming to be added in Phase 2)
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
