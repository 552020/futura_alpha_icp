use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use ic_cdk::api::time;
use crate::http::secret::get_current_secret;

type HmacSha256 = Hmac<Sha256>;

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct TokenScope {
    pub memory_id: String,
    pub variants: Vec<String>,             // e.g. ["thumbnail","preview","original"]
    pub asset_ids: Option<Vec<String>>,    // optional narrowing
}

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct TokenPayload {
    pub ver: u8,
    pub exp_ns: u64,                       // expiry in IC time
    pub nonce: [u8; 12],                   // 96-bit random
    pub scope: TokenScope,
    pub sub: Option<Principal>,            // optional: bind to user
}

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct EncodedToken {
    pub p: TokenPayload,
    pub s: [u8; 32],                       // HMAC-SHA256(payload)
}

fn canonical_bytes(p: &TokenPayload) -> Vec<u8> {
    serde_json::to_vec(p).expect("json")
}

pub fn sign_token(p: &TokenPayload) -> EncodedToken {
    let sec = get_current_secret().expect("Secret not initialized");
    let mut mac = HmacSha256::new_from_slice(&sec).unwrap();
    let bytes = canonical_bytes(p);
    mac.update(&bytes);
    let sig = mac.finalize().into_bytes();
    let mut s = [0u8; 32];
    s.copy_from_slice(&sig[..32]);
    EncodedToken { p: p.clone(), s }
}

pub enum VerifyErr { Expired, BadSig, WrongMemory, VariantNotAllowed, AssetNotAllowed }

pub fn verify_token(t: &EncodedToken, now_ns: u64, path_scope: &TokenScope) -> Result<(), VerifyErr> {
    if now_ns > t.p.exp_ns { return Err(VerifyErr::Expired); }
    if t.p.scope.memory_id != path_scope.memory_id { return Err(VerifyErr::WrongMemory); }
    for v in &path_scope.variants {
        if !t.p.scope.variants.iter().any(|vv| vv == v) { return Err(VerifyErr::VariantNotAllowed); }
    }
    if let Some(request_ids) = &path_scope.asset_ids {
        let Some(allowed) = &t.p.scope.asset_ids else { return Err(VerifyErr::AssetNotAllowed); };
        for id in request_ids {
            if !allowed.iter().any(|a| a == id) { return Err(VerifyErr::AssetNotAllowed); }
        }
    }

    let sec = get_current_secret().expect("Secret not initialized");
    let mut mac = HmacSha256::new_from_slice(&sec).unwrap();
    let bytes = canonical_bytes(&t.p);
    mac.update(&bytes);
    mac.verify_slice(&t.s).map_err(|_| VerifyErr::BadSig)
}

pub fn encode_token(t: &EncodedToken) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_vec(t).unwrap())
}
pub fn decode_token(s: &str) -> Result<EncodedToken, ()> {
    let bytes = general_purpose::URL_SAFE_NO_PAD.decode(s).map_err(|_| ())?;
    serde_json::from_slice(&bytes).map_err(|_| ())
}

/// Convenience helper to be used by routes:
pub fn verify_query_token(token_param: Option<&str>, memory_id: &str, variant: &str, asset_id: Option<&str>)
    -> Result<(), VerifyErr>
{
    let tok_str = token_param.ok_or(VerifyErr::BadSig)?; // "missing token" treated as bad sig
    let tok = decode_token(tok_str).map_err(|_| VerifyErr::BadSig)?;
    let path_scope = TokenScope {
        memory_id: memory_id.to_string(),
        variants: vec![variant.to_string()],
        asset_ids: asset_id.map(|a| vec![a.to_string()]),
    };
    verify_token(&tok, time(), &path_scope)
}