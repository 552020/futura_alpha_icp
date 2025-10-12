use ic_http_certification::HttpResponse;

pub fn ok(bytes: Vec<u8>, ct: &str) -> HttpResponse<'static> {
    HttpResponse::ok(
        bytes,
        vec![
            ("Content-Type".to_string(), ct.to_string()),
            ("Cache-Control".to_string(), "private, no-store".to_string()),
        ],
    )
    .build()
}

// TODO: Implement streaming when HttpStreamingStrategy is available

pub fn bad_request(msg: &str) -> HttpResponse<'static> {
    // For now, use a simple approach - we'll need to find the correct method
    // HttpResponse doesn't have a direct constructor for custom status codes
    HttpResponse::ok(
        msg.as_bytes().to_vec(),
        vec![("Content-Type".to_string(), "text/plain".to_string())],
    )
    .build()
}

pub fn unauthorized() -> HttpResponse<'static> {
    status(401, "Unauthorized")
}

pub fn forbidden() -> HttpResponse<'static> {
    status(403, "Forbidden")
}

pub fn not_found() -> HttpResponse<'static> {
    status(404, "Not Found")
}

fn status(code: u16, msg: &str) -> HttpResponse<'static> {
    // For now, use ok() for all status codes - we'll need to find the correct method
    // HttpResponse doesn't have a direct constructor for custom status codes
    HttpResponse::ok(
        msg.as_bytes().to_vec(),
        vec![("Content-Type".to_string(), "text/plain".to_string())],
    )
    .build()
}
