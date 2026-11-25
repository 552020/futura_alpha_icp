use crate::http::{
    acl::FuturaAclAdapter,
    asset_store::FuturaAssetStore,
    auth_core::{encode_token_url, sign_token_core},
    core_types::{Acl, AssetStore, TokenPayload, TokenScope},
    secret_store::StableSecretStore,
};
use ic_cdk::api::{msg_caller, time};

/// Token service for HTTP authentication
pub struct TokenService;

impl TokenService {
    /// Mint a single HTTP token for a memory
    pub fn mint_token(
        memory_id: String,
        variants: Vec<String>,
        asset_ids: Option<Vec<String>>,
        ttl_secs: u32,
    ) -> String {
        let caller = msg_caller();
        let acl = FuturaAclAdapter;

        // ✅ Enhanced: Use ACL adapter for authorization (no domain imports in HTTP layer)
        assert!(acl.can_view(&memory_id, caller), "forbidden");

        // ✅ Enhanced: Validate asset existence if asset_ids are specified
        if let Some(ids) = &asset_ids {
            let store = FuturaAssetStore;
            for id in ids {
                assert!(store.exists(&memory_id, id), "asset not found");
            }
        }

        Self::sign_for_caller(memory_id, variants, asset_ids, ttl_secs, caller)
    }

    /// Mint a token for a memory the caller already holds.
    ///
    /// For callers that loaded the memory from a capsule they verified read
    /// access to (memories_list / memories_list_by_capsule). It applies the
    /// same checks as `mint_token` -- VIEW permission through
    /// `effective_perm_mask` and asset existence -- but on the memory in hand.
    /// `mint_token` re-finds the memory through FuturaAclAdapter::can_view and
    /// FuturaAssetStore::exists, each of which decodes every capsule the
    /// caller can access; per token that made listing ~100 memories exceed the
    /// 5B instruction limit.
    pub fn mint_token_for_memory(
        memory: &crate::memories::types::Memory,
        variants: Vec<String>,
        asset_ids: Option<Vec<String>>,
        ttl_secs: u32,
    ) -> String {
        use crate::capsule::domain::{effective_perm_mask, Perm, PrincipalContext};

        let caller = msg_caller();
        let ctx = PrincipalContext {
            principal: caller,
            groups: vec![],
            link: None,
            now_ns: time(),
        };
        assert!(
            (effective_perm_mask(memory, &ctx) & Perm::VIEW.bits()) != 0,
            "forbidden"
        );

        if let Some(ids) = &asset_ids {
            for id in ids {
                let exists = memory.inline_assets.iter().any(|a| &a.asset_id == id)
                    || memory.blob_internal_assets.iter().any(|a| &a.asset_id == id)
                    || memory.blob_external_assets.iter().any(|a| &a.asset_id == id);
                assert!(exists, "asset not found");
            }
        }

        Self::sign_for_caller(memory.id.clone(), variants, asset_ids, ttl_secs, caller)
    }

    /// Build, sign and encode a token for `caller` (authorization done by caller)
    fn sign_for_caller(
        memory_id: String,
        variants: Vec<String>,
        asset_ids: Option<Vec<String>>,
        ttl_secs: u32,
        caller: candid::Principal,
    ) -> String {
        let ttl = effective_ttl(ttl_secs);

        // build payload
        let mut nonce = [0u8; 12];
        // Use deterministic nonce based on time and caller for query functions
        let time_bytes = time().to_le_bytes();
        let caller_bytes = caller.as_slice();
        for i in 0..12 {
            nonce[i] = time_bytes[i % 8] ^ caller_bytes[i % caller_bytes.len()];
        }

        let payload = TokenPayload {
            ver: 1,
            kid: 1, // ✅ Enhanced: Key version for secret rotation
            exp_ns: time() + (ttl as u64) * 1_000_000_000,
            nonce,
            scope: TokenScope {
                memory_id,
                variants,
                asset_ids,
            },
            sub: Some(caller), // ✅ Enhanced: Bind token to caller by default
        };

        let token = sign_token_core(&StableSecretStore, &payload);
        encode_token_url(&token)
    }

    /// Bulk token minting for efficient dashboard loading
    /// Returns a vector of (memory_id, token) pairs
    pub fn mint_tokens_bulk(
        memory_ids: Vec<String>,
        variants: Vec<String>,
        asset_ids: Option<Vec<String>>,
        ttl_secs: u32,
    ) -> Vec<(String, String)> {
        let caller = msg_caller();
        let acl = FuturaAclAdapter;

        // ✅ Enhanced: Default TTL to 180 seconds if not specified, cap at 180s
        let ttl = if ttl_secs == 0 {
            180
        } else {
            ttl_secs.min(180)
        };

        let mut tokens = Vec::new();

        for memory_id in memory_ids {
            // ✅ Enhanced: Use ACL adapter for authorization
            if !acl.can_view(&memory_id, caller) {
                // Skip memories the user doesn't have access to
                continue;
            }

            // ✅ Enhanced: Validate asset existence if asset_ids are specified
            if let Some(ids) = &asset_ids {
                let store = FuturaAssetStore;
                let mut all_assets_exist = true;
                for id in ids {
                    if !store.exists(&memory_id, id) {
                        all_assets_exist = false;
                        break;
                    }
                }
                if !all_assets_exist {
                    // Skip memories with missing assets
                    continue;
                }
            }

            // Build payload for this memory
            let mut nonce = [0u8; 12];
            // Use deterministic nonce based on time, caller, and memory_id for uniqueness
            let time_bytes = time().to_le_bytes();
            let caller_bytes = caller.as_slice();
            let memory_bytes = memory_id.as_bytes();
            for i in 0..12 {
                nonce[i] = time_bytes[i % 8]
                    ^ caller_bytes[i % caller_bytes.len()]
                    ^ memory_bytes[i % memory_bytes.len()];
            }

            let payload = TokenPayload {
                ver: 1,
                kid: 1,
                exp_ns: time() + (ttl as u64) * 1_000_000_000,
                nonce,
                scope: TokenScope {
                    memory_id: memory_id.clone(),
                    variants: variants.clone(),
                    asset_ids: asset_ids.clone(),
                },
                sub: Some(caller),
            };

            let token = sign_token_core(&StableSecretStore, &payload);
            let encoded_token = encode_token_url(&token);

            tokens.push((memory_id, encoded_token));
        }

        tokens
    }
}

/// A link asked for without a lifetime lives this long
const DEFAULT_TTL_SECS: u32 = 180;
/// The longest a link may live, whatever the caller asks for
const MAX_TTL_SECS: u32 = 1800;

/// How long a link lives: what was asked for, at most `MAX_TTL_SECS`;
/// 0 means "the default".
fn effective_ttl(ttl_secs: u32) -> u32 {
    if ttl_secs == 0 {
        DEFAULT_TTL_SECS
    } else {
        ttl_secs.min(MAX_TTL_SECS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_link_lives_as_long_as_asked_within_the_maximum() {
        assert_eq!(effective_ttl(0), DEFAULT_TTL_SECS);
        assert_eq!(effective_ttl(1), 1);
        assert_eq!(effective_ttl(600), 600);
        assert_eq!(effective_ttl(MAX_TTL_SECS), MAX_TTL_SECS);
        assert_eq!(effective_ttl(MAX_TTL_SECS + 1), MAX_TTL_SECS);
        assert_eq!(effective_ttl(u32::MAX), MAX_TTL_SECS);
    }
}
