use ic_http_certification::{HttpResponse, StatusCode};
use crate::http::core_types::ParsedRequest;

pub fn get(_: &ParsedRequest) -> HttpResponse<'static> {
    HttpResponse::ok(
        b"OK",
        vec![("Content-Type".into(), "text/plain".into())]
    ).build()
}
