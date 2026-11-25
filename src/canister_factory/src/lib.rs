use candid::{CandidType, Principal};
use ic_cdk::api::msg_caller;
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

mod personal;
mod top_up;
#[allow(unused_imports)]
use personal::{PersonalCanister, ReleaseInfo};
use top_up::{TopUpConfig, TopUpReport};

/// ===== Configuration Constants =====
const MAX_CHUNK_SIZE: usize = 2_000_000; // 2MB per chunk
const DEFAULT_UPLOAD_TTL_NS: u64 = 24 * 60 * 60 * 1_000_000_000; // 24 hours in nanoseconds
const DEFAULT_MAX_UPLOAD_SIZE: u64 = 50_000_000; // 50MB max WASM size

/// ===== Types exposed over Candid =====

#[derive(CandidType, Deserialize)]
pub struct InitArg {
    /// Optional minimal cycles balance to keep in factory (default 5T).
    pub min_factory_cycles: Option<u128>,
    /// Optional allowed callers; if present, only these may use the factory.
    pub allowlist: Option<Vec<Principal>>,
    /// Optional admin principals who can modify settings.
    pub admins: Option<Vec<Principal>>,
    /// Optional max upload size in bytes (default 50MB).
    pub max_upload_size: Option<u64>,
    /// Optional upload TTL in seconds (default 24 hours).
    pub upload_ttl_seconds: Option<u64>,
}

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct UploadInfo {
    pub owner: Principal,
    pub chunks: Vec<Vec<u8>>,
    pub total_len: u64,
    pub committed_hash: Option<[u8; 32]>,
    pub created_at_time_ns: u64,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct UploadCommit {
    /// Expected SHA-256 of the concatenated wasm.
    pub expected_sha256_hex: String,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct FactoryStats {
    pub total_canisters_created: u64,
    pub total_uploads: u64,
    pub active_uploads: u64,
    pub factory_cycles_balance: u128,
}

/// ===== Internal stable state =====

#[derive(CandidType, Serialize, Deserialize, Clone)]
struct Config {
    min_factory_cycles: u128,
    max_upload_size: u64,
    upload_ttl_ns: u64,
    allowlist: Option<BTreeSet<Principal>>,
    admins: BTreeSet<Principal>,
    next_upload_id: u64,
    emergency_stop: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            min_factory_cycles: 5_000_000_000_000, // 5T cycles
            max_upload_size: DEFAULT_MAX_UPLOAD_SIZE,
            upload_ttl_ns: DEFAULT_UPLOAD_TTL_NS,
            allowlist: None,
            admins: BTreeSet::new(),
            next_upload_id: 1,
            emergency_stop: false,
        }
    }
}

#[derive(CandidType, Serialize, Deserialize, Default, Clone)]
struct State {
    cfg: Config,
    uploads: BTreeMap<u64, UploadInfo>,
    total_canisters_created: u64,
    /// Releases and the per-user registry (personal.rs). Optional so that
    /// state saved before it existed still restores.
    personal: Option<personal::PersonalState>,
}

thread_local! {
    static STATE: std::cell::RefCell<State> = std::cell::RefCell::new(State::default());
}

/// ===== Helpers =====
fn must_allowed(caller: Principal) -> Result<(), String> {
    STATE.with(|s| {
        let st = s.borrow();
        if st.cfg.emergency_stop {
            return Err("Factory is in emergency stop mode".into());
        }
        if let Some(allow) = &st.cfg.allowlist {
            if !allow.contains(&caller) {
                return Err("Caller not in allowlist".into());
            }
        }
        Ok(())
    })
}

fn must_be_admin(caller: Principal) -> Result<(), String> {
    STATE.with(|s| {
        let st = s.borrow();
        if !st.cfg.admins.contains(&caller) {
            return Err("Admin access required".into());
        }
        Ok(())
    })
}

fn must_have_cycles_left() -> Result<(), String> {
    STATE.with(|s| {
        let st = s.borrow();
        let bal = ic_cdk::api::canister_cycle_balance();
        if bal <= st.cfg.min_factory_cycles {
            Err(format!(
                "Factory low on cycles: {bal} (min {})",
                st.cfg.min_factory_cycles
            ))
        } else {
            Ok(())
        }
    })
}

fn bump_upload_id() -> u64 {
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        let id = st.cfg.next_upload_id;
        st.cfg.next_upload_id += 1;
        id
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

fn cleanup_expired_uploads() -> u32 {
    let now = ic_cdk::api::time();
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        let expired: Vec<u64> = st
            .uploads
            .iter()
            .filter(|(_, upload)| {
                now.saturating_sub(upload.created_at_time_ns) > st.cfg.upload_ttl_ns
            })
            .map(|(id, _)| *id)
            .collect();

        for id in &expired {
            st.uploads.remove(id);
        }
        expired.len() as u32
    })
}

/// ===== Init/Upgrade =====

#[init]
fn init(arg: Option<InitArg>) {
    let mut st = STATE.with(|s| s.borrow().clone());
    if let Some(a) = arg {
        if let Some(m) = a.min_factory_cycles {
            st.cfg.min_factory_cycles = m;
        }
        if let Some(size) = a.max_upload_size {
            st.cfg.max_upload_size = size;
        }
        if let Some(ttl_sec) = a.upload_ttl_seconds {
            st.cfg.upload_ttl_ns = ttl_sec * 1_000_000_000; // Convert to nanoseconds
        }
        if let Some(list) = a.allowlist {
            st.cfg.allowlist = Some(list.into_iter().collect());
        }
        if let Some(admins) = a.admins {
            st.cfg.admins = admins.into_iter().collect();
        }
    }

    // Add the deployer as an admin if no admins were specified
    if st.cfg.admins.is_empty() {
        st.cfg.admins.insert(msg_caller());
    }

    STATE.with(|s| *s.borrow_mut() = st);
    top_up::start_timer();
}

#[pre_upgrade]
fn pre_upgrade() {
    let st = STATE.with(|s| s.borrow().clone());
    ic_cdk::storage::stable_save((st,)).expect("pre_upgrade: stable_save failed");
}

#[post_upgrade]
fn post_upgrade() {
    // Trap instead of starting empty: a failed upgrade is rolled back by the
    // network, an empty state would forget every canister and the release.
    let (st,): (State,) = ic_cdk::storage::stable_restore()
        .expect("post_upgrade: cannot read the saved state; upgrade rolled back");
    STATE.with(|s| *s.borrow_mut() = st);
    // Timers are not kept across upgrades
    top_up::start_timer();
}

/// ===== Upload API (chunked) =====

#[update]
async fn create_upload() -> Result<u64, String> {
    let caller = msg_caller();
    // Uploads exist to release a wasm (personal.rs): admins only. An open
    // upload API lets anybody fill the factory's memory.
    must_be_admin(caller)?;
    must_have_cycles_left()?;

    // Clean up expired uploads opportunistically
    cleanup_expired_uploads();

    let id = bump_upload_id();
    STATE.with(|s| {
        s.borrow_mut().uploads.insert(
            id,
            UploadInfo {
                owner: caller,
                chunks: Vec::new(),
                total_len: 0,
                committed_hash: None,
                created_at_time_ns: ic_cdk::api::time(),
            },
        );
    });
    Ok(id)
}

#[update]
async fn put_chunk(upload_id: u64, chunk: Vec<u8>) -> Result<u64, String> {
    let caller = msg_caller();

    if chunk.len() > MAX_CHUNK_SIZE {
        return Err(format!(
            "Chunk too large: {} bytes (max {})",
            chunk.len(),
            MAX_CHUNK_SIZE
        ));
    }

    STATE.with(|s| {
        let mut st = s.borrow_mut();
        let max_size = st.cfg.max_upload_size;
        let up = st
            .uploads
            .get_mut(&upload_id)
            .ok_or_else(|| "upload_id not found".to_string())?;
        if up.owner != caller {
            return Err("not owner".into());
        }
        if up.committed_hash.is_some() {
            return Err("upload already committed".into());
        }

        let new_total = up
            .total_len
            .checked_add(chunk.len() as u64)
            .ok_or("size overflow")?;

        if new_total > max_size {
            return Err(format!(
                "Upload too large: {new_total} bytes (max {max_size})"
            ));
        }

        up.total_len = new_total;
        up.chunks.push(chunk);
        Ok(up.total_len)
    })
}

#[update]
async fn commit_upload(upload_id: u64, commit: UploadCommit) -> Result<String, String> {
    let caller = msg_caller();
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        let up = st
            .uploads
            .get_mut(&upload_id)
            .ok_or_else(|| "upload_id not found".to_string())?;
        if up.owner != caller {
            return Err("not owner".into());
        }
        if up.committed_hash.is_some() {
            return Err("already committed".into());
        }
        // compute hash over concatenation
        let mut hasher = Sha256::new();
        for c in &up.chunks {
            hasher.update(c);
        }
        let sum: [u8; 32] = hasher.finalize().into();
        let hex_now = hex::encode(sum);
        if hex_now != commit.expected_sha256_hex {
            return Err(format!(
                "hash mismatch: expected {}, got {}",
                commit.expected_sha256_hex, hex_now
            ));
        }
        up.committed_hash = Some(sum);
        Ok(hex_now)
    })
}

#[update]
async fn clear_upload(upload_id: u64) -> Result<(), String> {
    let caller = msg_caller();
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        let up = st
            .uploads
            .get(&upload_id)
            .ok_or_else(|| "upload_id not found".to_string())?;
        if up.owner != caller {
            return Err("not owner".into());
        }
        st.uploads.remove(&upload_id);
        Ok(())
    })
}

/// ===== Admin endpoints =====

#[update]
fn set_emergency_stop(stop: bool) -> Result<(), String> {
    let caller = msg_caller();
    must_be_admin(caller)?;

    STATE.with(|s| {
        s.borrow_mut().cfg.emergency_stop = stop;
    });
    Ok(())
}

#[update]
fn set_allowlist(list: Option<Vec<Principal>>) -> Result<(), String> {
    let caller = msg_caller();
    must_be_admin(caller)?;

    STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.cfg.allowlist = list.map(|v| v.into_iter().collect());
    });
    Ok(())
}

#[update]
fn add_admin(admin: Principal) -> Result<(), String> {
    let caller = msg_caller();
    must_be_admin(caller)?;

    STATE.with(|s| {
        s.borrow_mut().cfg.admins.insert(admin);
    });
    Ok(())
}

#[update]
fn remove_admin(admin: Principal) -> Result<(), String> {
    let caller = msg_caller();
    must_be_admin(caller)?;

    STATE.with(|s| {
        let mut st = s.borrow_mut();
        // Prevent removing the last admin
        if st.cfg.admins.len() <= 1 {
            return Err("Cannot remove the last admin".into());
        }
        st.cfg.admins.remove(&admin);
        Ok(())
    })
}

#[update]
fn cleanup_expired_uploads_manual() -> Result<u32, String> {
    let caller = msg_caller();
    must_be_admin(caller)?;

    Ok(cleanup_expired_uploads())
}

/// ===== Query endpoints =====

#[query]
fn get_config() -> Config {
    STATE.with(|s| s.borrow().cfg.clone())
}

#[query]
fn get_factory_stats() -> FactoryStats {
    STATE.with(|s| {
        let st = s.borrow();
        FactoryStats {
            total_canisters_created: st.total_canisters_created,
            total_uploads: st.cfg.next_upload_id - 1,
            active_uploads: st.uploads.len() as u64,
            factory_cycles_balance: ic_cdk::api::canister_cycle_balance(),
        }
    })
}

#[query]
fn get_upload_info(upload_id: u64) -> Result<UploadInfo, String> {
    let caller = msg_caller();
    STATE.with(|s| {
        let st = s.borrow();
        let upload = st
            .uploads
            .get(&upload_id)
            .ok_or_else(|| "Upload not found".to_string())?;

        if upload.owner != caller {
            return Err("Not the owner of this upload".into());
        }

        Ok(upload.clone())
    })
}

/// ===== Utility functions =====

#[query]
fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[query]
fn health_check() -> String {
    let balance = ic_cdk::api::canister_cycle_balance();
    let min_balance = STATE.with(|s| s.borrow().cfg.min_factory_cycles);

    if balance > min_balance {
        "Healthy".to_string()
    } else {
        format!("Low cycles: {balance} (min {min_balance})")
    }
}

// Export Candid
#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    use candid::export_service;
    export_service!();
    __export_service()
}

#[cfg(test)]

mod tests {
    use super::*;

    /// State as the factory saved it while it still had the generic
    /// create-and-install endpoint: two fields more than today.
    #[test]
    fn state_saved_by_the_older_factory_still_loads() {
        #[derive(CandidType, Serialize)]
        struct OldCallerStats {
            created_count: u32,
            last_created_at: Option<u64>,
        }
        #[derive(CandidType, Serialize)]
        struct OldConfig {
            max_canisters_per_caller: u32,
            min_factory_cycles: u128,
            max_upload_size: u64,
            upload_ttl_ns: u64,
            allowlist: Option<BTreeSet<Principal>>,
            admins: BTreeSet<Principal>,
            next_upload_id: u64,
            emergency_stop: bool,
        }
        #[derive(CandidType, Serialize)]
        struct OldState {
            cfg: OldConfig,
            uploads: BTreeMap<u64, UploadInfo>,
            caller_stats: BTreeMap<Principal, OldCallerStats>,
            total_canisters_created: u64,
            personal: Option<personal::PersonalState>,
        }
        let admin = Principal::from_slice(&[1; 29]);
        let old = OldState {
            cfg: OldConfig {
                max_canisters_per_caller: 100,
                min_factory_cycles: 7,
                max_upload_size: 8,
                upload_ttl_ns: 9,
                allowlist: None,
                admins: [admin].into(),
                next_upload_id: 4,
                emergency_stop: true,
            },
            uploads: BTreeMap::new(),
            caller_stats: [(
                admin,
                OldCallerStats {
                    created_count: 2,
                    last_created_at: Some(5),
                },
            )]
            .into(),
            total_canisters_created: 3,
            personal: Some(personal::PersonalState::default()),
        };
        // The same encoding stable_save uses: a tuple of one
        let bytes = candid::encode_args((old,)).unwrap();
        let (new,): (State,) = candid::decode_args(&bytes).unwrap();
        assert_eq!(new.cfg.min_factory_cycles, 7);
        assert_eq!(new.cfg.next_upload_id, 4);
        assert!(new.cfg.emergency_stop);
        assert!(new.cfg.admins.contains(&admin));
        assert_eq!(new.total_canisters_created, 3);
        assert!(new.personal.is_some());
    }

    /// Run with `-- --ignored --nocapture` to print the interface the code
    /// exports, for comparing with the hand-kept canister_factory.did
    #[test]
    #[ignore]
    fn print_candid() {
        println!("{}", export_candid());
    }

    #[test]
    fn sha256_hex_works() {
        assert_eq!(
            sha256_hex(&[]),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn config_defaults() {
        let config = Config::default();
        assert_eq!(config.min_factory_cycles, 5_000_000_000_000);
        assert_eq!(config.max_upload_size, DEFAULT_MAX_UPLOAD_SIZE);
        assert!(!config.emergency_stop);
    }
}
