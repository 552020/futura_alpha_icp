use crate::http::core_types::{ParsedRequest, TokenScope};

/// Map a parsed request to a path scope we expect the token to authorize.
pub fn path_to_scope(parsed: &ParsedRequest, memory_id: &str, variant: &str) -> TokenScope {
    let asset_id = parsed.q("id").map(|s| s.to_string());
    TokenScope {
        memory_id: memory_id.to_string(),
        variants: vec![variant.to_string()],
        asset_ids: asset_id.map(|a| vec![a]),
    }
}
