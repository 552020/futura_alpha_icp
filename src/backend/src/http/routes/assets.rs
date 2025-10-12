use crate::http::{request::ParsedRequest, response};
use ic_http_certification::HttpResponse;

pub fn get(memory_id: &str, variant: &str, req: &ParsedRequest) -> HttpResponse<'static> {
    // 1) verify token against path scope
    let token = req.q("token");
    let asset_id = req.q("id");
    
    // TODO: Make this async when we implement the actual auth verification
    // For now, we'll return a placeholder response
    if token.is_none() {
        return response::unauthorized();
    }

    // 2) resolve asset from your storage (memories + upload::blob_store)
    //    -> choose inline vs streaming based on size
    // TODO: call your existing adapters and return response::ok(...) or response::stream(...)

    response::not_found() // placeholder
}
