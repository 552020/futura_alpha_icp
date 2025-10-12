use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::http::core_types::{TokenPayload, EncodedToken, TokenScope, VerifyErr, SecretStore, Clock};

type HmacSha256 = Hmac<Sha256>;

fn canonical_bytes(p: &TokenPayload) -> Vec<u8> {
    serde_json::to_vec(p).expect("json")
}

pub fn sign_token_core(secret: &dyn SecretStore, payload: &TokenPayload) -> EncodedToken {
    let mut mac = HmacSha256::new_from_slice(&secret.get_key()).unwrap();
    let bytes = canonical_bytes(payload);
    mac.update(&bytes);
    let sig = mac.finalize().into_bytes();
    let mut s = [0u8; 32];
    s.copy_from_slice(&sig[..32]);
    EncodedToken { p: payload.clone(), s }
}

pub fn verify_token_core(clock: &dyn Clock, secret: &dyn SecretStore, t: &EncodedToken, want: &TokenScope)
    -> Result<(), VerifyErr>
{
    if clock.now_ns() > t.p.exp_ns { return Err(VerifyErr::Expired); }
    if t.p.scope.memory_id != want.memory_id { return Err(VerifyErr::WrongMemory); }
    for v in &want.variants {
        if !t.p.scope.variants.iter().any(|vv| vv == v) { return Err(VerifyErr::VariantNotAllowed); }
    }
    if let Some(req_ids) = &want.asset_ids {
        let Some(allow) = &t.p.scope.asset_ids else { return Err(VerifyErr::AssetNotAllowed); };
        for id in req_ids {
            if !allow.iter().any(|a| a == id) { return Err(VerifyErr::AssetNotAllowed); }
        }
    }
    let mut mac = HmacSha256::new_from_slice(&secret.get_key()).unwrap();
    mac.update(&canonical_bytes(&t.p));
    mac.verify_slice(&t.s).map_err(|_| VerifyErr::BadSig)
}

/// For URL param usage
pub fn encode_token_url(t: &EncodedToken) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_vec(t).unwrap())
}

pub fn decode_token_url(s: &str) -> Option<EncodedToken> {
    let bytes = general_purpose::URL_SAFE_NO_PAD.decode(s).ok()?;
    serde_json::from_slice(&bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::core_types::{TokenScope, TokenPayload, SecretStore, Clock};
    use candid::Principal;

    struct MockClock { now: u64 }
    impl Clock for MockClock { fn now_ns(&self) -> u64 { self.now } }

    struct MockSecret { key: [u8; 32] }
    impl SecretStore for MockSecret { fn get_key(&self) -> [u8; 32] { self.key } }

    fn key(bytes: u8) -> [u8;32] { [bytes; 32] }

    #[test]
    fn verify_roundtrip_ok() {
        let clock = MockClock { now: 1_000_000_000 };
        let secret = MockSecret { key: key(7) };

        let payload = TokenPayload {
            ver: 1,
            exp_ns: clock.now_ns() + 10_000_000, // +10ms
            nonce: [1u8; 12],
            scope: TokenScope {
                memory_id: "mem-123".into(),
                variants: vec!["thumbnail".into()],
                asset_ids: Some(vec!["asset-1".into()]),
            },
            sub: Some(Principal::anonymous())
        };
        let tok = sign_token_core(&secret, &payload);

        let want = TokenScope {
            memory_id: "mem-123".into(),
            variants: vec!["thumbnail".into()],
            asset_ids: Some(vec!["asset-1".into()]),
        };

        assert!(verify_token_core(&clock, &secret, &tok, &want).is_ok());
    }

    #[test]
    fn verify_expired_fails() {
        let clock = MockClock { now: 2_000_000_000 };
        let secret = MockSecret { key: key(3) };

        let payload = TokenPayload {
            ver: 1,
            exp_ns: clock.now_ns() - 1, // already expired
            nonce: [0u8; 12],
            scope: TokenScope { memory_id: "m".into(), variants: vec!["preview".into()], asset_ids: None },
            sub: None
        };
        let tok = sign_token_core(&secret, &payload);
        let want = TokenScope { memory_id: "m".into(), variants: vec!["preview".into()], asset_ids: None };

        assert_eq!(verify_token_core(&clock, &secret, &tok, &want), Err(VerifyErr::Expired));
    }

    #[test]
    fn verify_wrong_memory_fails() {
        let clock = MockClock { now: 1 };
        let secret = MockSecret { key: key(9) };

        let payload = TokenPayload {
            ver: 1, exp_ns: 10, nonce: [0;12],
            scope: TokenScope { memory_id: "A".into(), variants: vec!["thumbnail".into()], asset_ids: None },
            sub: None
        };
        let tok = sign_token_core(&secret, &payload);
        let want = TokenScope { memory_id: "B".into(), variants: vec!["thumbnail".into()], asset_ids: None };

        assert_eq!(verify_token_core(&clock, &secret, &tok, &want), Err(VerifyErr::WrongMemory));
    }

    #[test]
    fn verify_variant_not_allowed_fails() {
        let clock = MockClock { now: 1 };
        let secret = MockSecret { key: key(1) };

        let payload = TokenPayload {
            ver: 1, exp_ns: 10, nonce: [0;12],
            scope: TokenScope { memory_id: "M".into(), variants: vec!["preview".into()], asset_ids: None },
            sub: None
        };
        let tok = sign_token_core(&secret, &payload);
        let want = TokenScope { memory_id: "M".into(), variants: vec!["thumbnail".into()], asset_ids: None };

        assert_eq!(verify_token_core(&clock, &secret, &tok, &want), Err(VerifyErr::VariantNotAllowed));
    }

    #[test]
    fn verify_asset_id_not_allowed_fails() {
        let clock = MockClock { now: 1 };
        let secret = MockSecret { key: key(4) };

        let payload = TokenPayload {
            ver: 1, exp_ns: 10, nonce: [0;12],
            scope: TokenScope { memory_id: "M".into(), variants: vec!["thumbnail".into()], asset_ids: Some(vec!["id-1".into()]) },
            sub: None
        };
        let tok = sign_token_core(&secret, &payload);
        let want = TokenScope { memory_id: "M".into(), variants: vec!["thumbnail".into()], asset_ids: Some(vec!["id-2".into()]) };

        assert_eq!(verify_token_core(&clock, &secret, &tok, &want), Err(VerifyErr::AssetNotAllowed));
    }

    #[test]
    fn tamper_signature_fails() {
        let clock = MockClock { now: 1 };
        let secret = MockSecret { key: key(8) };

        let payload = TokenPayload {
            ver: 1, exp_ns: 10, nonce: [9;12],
            scope: TokenScope { memory_id: "M".into(), variants: vec!["preview".into()], asset_ids: None },
            sub: None
        };
        let mut tok = sign_token_core(&secret, &payload);
        // flip one bit in signature
        tok.s[0] ^= 0x01;

        let want = TokenScope { memory_id: "M".into(), variants: vec!["preview".into()], asset_ids: None };
        assert_eq!(verify_token_core(&clock, &secret, &tok, &want), Err(VerifyErr::BadSig));
    }
}
