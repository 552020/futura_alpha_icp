use crate::http::response;
use ic_http_certification::HttpRequest;

pub struct ParsedRequest {
    pub method: String,
    pub path_segments: Vec<String>,
    pub query: Vec<(String, String)>, // simple k=v pairs
}

impl TryFrom<HttpRequest<'_>> for ParsedRequest {
    type Error = ic_http_certification::HttpResponse<'static>;
    fn try_from(req: HttpRequest) -> Result<Self, Self::Error> {
        let method = req.method().to_string().to_uppercase();
        // split "/a/b?x=1" into segments & query
        let (path, query_str) = req.url().split_once('?').unwrap_or((&req.url()[..], ""));
        let path_segments = path
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let query = query_str
            .split('&')
            .filter(|s| !s.is_empty())
            .filter_map(|kv| {
                kv.split_once('=')
                    .map(|(k, v)| (k.to_string(), v.to_string()))
            })
            .collect::<Vec<_>>();
        if method.is_empty() {
            return Err(response::bad_request("invalid method"));
        }
        Ok(Self {
            method,
            path_segments,
            query,
        })
    }
}

impl ParsedRequest {
    pub fn q(&self, name: &str) -> Option<&str> {
        self.query
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}
