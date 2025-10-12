use crate::http::{request::ParsedRequest, response};

pub fn get(_: &ParsedRequest) -> ic_http_certification::HttpResponse<'static> {
    response::ok(b"OK".to_vec(), "text/plain")
}
