use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

/// Parsed, pure representation of an HTTP request we care about.
#[derive(Clone, Debug)]
pub struct ParsedRequest {
    pub method: String,
    pub path_segments: Vec<String>,
    pub query: Vec<(String, String)>,
    pub headers: Vec<(String, String)>,
}

impl ParsedRequest {
    pub fn q(&self, name: &str) -> Option<&str> {
        self.query
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    pub fn get_header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == name.to_lowercase())
            .map(|(_, v)| v.as_str())
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TokenScope {
    pub memory_id: String,
    pub variants: Vec<String>,
    pub asset_ids: Option<Vec<String>>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TokenPayload {
    pub ver: u8,
    pub kid: u32, // Key version for secret rotation
    pub exp_ns: u64,
    pub nonce: [u8; 12],
    pub scope: TokenScope,
    pub sub: Option<Principal>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EncodedToken {
    pub p: TokenPayload,
    pub s: [u8; 32],
}

#[derive(Debug, PartialEq, Eq)]
pub enum VerifyErr {
    Expired,
    BadSig,
    WrongMemory,
    VariantNotAllowed,
    AssetNotAllowed,
}

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AssetErr {
    NotFound,
    TooLargeForInline,
    Io,
}

/// Dependency inversion traits — pure, mockable.
pub trait Clock {
    fn now_ns(&self) -> u64;
}

pub trait SecretStore {
    fn get_key(&self) -> [u8; 32];
}

pub struct InlineAsset {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

pub trait AssetStore {
    #[allow(dead_code)]
    fn get_inline(&self, memory_id: &str, asset_id: &str) -> Option<InlineAsset>;
    #[allow(dead_code)]
    fn get_blob_len(&self, memory_id: &str, asset_id: &str) -> Option<(u64, String)>;
    #[allow(dead_code)]
    fn read_blob_chunk(
        &self,
        memory_id: &str,
        asset_id: &str,
        offset: u64,
        len: u64,
    ) -> Option<Vec<u8>>;
    fn exists(&self, memory_id: &str, asset_id: &str) -> bool;

    /// Get inline asset using the token's subject principal (not HTTP caller)
    #[allow(dead_code)]
    fn get_inline_with_principal(
        &self,
        who: &Principal,
        memory_id: &str,
        asset_id: &str,
    ) -> Option<InlineAsset>;

    /// Check if asset exists using the token's subject principal (not HTTP caller)
    #[allow(dead_code)]
    fn exists_with_principal(&self, who: &Principal, memory_id: &str, asset_id: &str) -> bool;

    /// Resolve asset for a specific variant, handling variant-to-asset-id mapping
    #[allow(dead_code)]
    fn resolve_asset_for_variant(
        &self,
        who: &Principal,
        memory_id: &str,
        variant: &str,
        id_param: Option<&str>,
    ) -> Option<String>;

    /// Get blob asset using the token's subject principal (not HTTP caller)
    #[allow(dead_code)]
    fn get_blob_with_principal(
        &self,
        who: &Principal,
        memory_id: &str,
        asset_id: &str,
    ) -> Option<(Vec<u8>, String)>;
}

/// ACL trait for authorization - avoids domain imports in HTTP layer
pub trait Acl {
    fn can_view(&self, memory_id: &str, who: Principal) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

    #[test]
    fn test_token_payload_creation() {
        let payload = TokenPayload {
            ver: 1,
            kid: 1,
            exp_ns: 1000000000,
            nonce: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            scope: TokenScope {
                memory_id: "memory_123".to_string(),
                variants: vec!["thumbnail".to_string()],
                asset_ids: None,
            },
            sub: Some(Principal::anonymous()),
        };

        assert_eq!(payload.ver, 1);
        assert_eq!(payload.kid, 1);
        assert_eq!(payload.scope.memory_id, "memory_123");
    }

    #[test]
    fn test_scope_parsing() {
        let valid_scopes = vec![
            ("memory_123", "thumbnail"),
            ("memory_123", "preview"),
            ("memory_123", "original"),
        ];

        for (memory_id, variant) in valid_scopes {
            assert!(memory_id.starts_with("memory_"));
            assert!(["thumbnail", "preview", "original"].contains(&variant));
        }
    }
}
