use candid::{CandidType, Principal};
use serde::{Serialize, Deserialize};

/// Parsed, pure representation of an HTTP request we care about.
#[derive(Clone, Debug)]
pub struct ParsedRequest {
    pub method: String,
    pub path_segments: Vec<String>,
    pub query: Vec<(String, String)>,
}

impl ParsedRequest {
    pub fn q(&self, name: &str) -> Option<&str> {
        self.query.iter().find(|(k,_)| k == name).map(|(_,v)| v.as_str())
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
    AssetNotAllowed 
}

#[derive(Debug, PartialEq, Eq)]
pub enum AssetErr { 
    NotFound, 
    TooLargeForInline, 
    Io 
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
    fn get_inline(&self, memory_id: &str, asset_id: &str) -> Option<InlineAsset>;
    fn get_blob_len(&self, memory_id: &str, asset_id: &str) -> Option<(u64, String)>;
    fn read_blob_chunk(&self, memory_id: &str, asset_id: &str, offset: u64, len: u64) -> Option<Vec<u8>>;
}

/// ACL trait for authorization - avoids domain imports in HTTP layer
pub trait Acl {
    fn can_view(&self, memory_id: &str, who: Principal) -> bool;
}
