use candid::{CandidType, Principal};
use ic_cdk::api::msg_caller;
use ic_cdk::call::Call;
use ic_cdk::management_canister::{
    install_code,
    update_settings,
    CanisterId, // type alias
    CanisterInstallMode,
    CanisterSettings,
    CreateCanisterArgs,
    InstallCodeArgs,
    UpdateSettingsArgs,
};
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

/// ===== Configuration Constants =====
const MAX_CHUNK_SIZE: usize = 2_000_000; // 2MB per chunk
const DEFAULT_UPLOAD_TTL_NS: u64 = 24 * 60 * 60 * 1_000_000_000; // 24 hours in nanoseconds
const DEFAULT_MAX_UPLOAD_SIZE: u64 = 50_000_000; // 50MB max WASM size

/// ===== Types exposed over Candid =====

#[derive(CandidType, Deserialize)]
pub struct InitArg {
    /// Optional per-caller cap (default 100).
    pub max_canisters_per_caller: Option<u32>,
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

#[derive(CandidType, Serialize, Deserialize, Default, Clone)]
pub struct CallerStats {
    pub created_count: u32,
    pub last_created_at: Option<u64>,
}

#[derive(CandidType, Serialize, Deserialize)]
pub enum Mode {
    Install,
    Reinstall,
    Upgrade,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct CreateInstallRequest {
    /// Upload id previously created by `create_upload`.
    pub upload_id: u64,
    /// Optional extra controllers to add along with the caller.
    pub extra_controllers: Option<Vec<Principal>>,
    /// Candid-encoded init arg expected by the target canister (may be empty).
    pub init_arg: Vec<u8>,
    /// Install mode.
    pub mode: Mode,
    /// Attach this many cycles to `create_canister`.
    pub cycles: u128,
    /// If true, hand off control exclusively to caller (+extras) after install.
    pub handoff: bool,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct CreateInstallResponse {
    pub canister_id: Principal,
    pub module_hash_hex: String,
    pub cycles_used: u128,
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
    pub unique_callers: u64,
}

/// ===== Internal stable state =====

#[derive(CandidType, Serialize, Deserialize, Clone)]
struct Config {
    max_canisters_per_caller: u32,
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
            max_canisters_per_caller: 100,
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
    caller_stats: BTreeMap<Principal, CallerStats>,
    total_canisters_created: u64,
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

fn caller_stats_ok(caller: Principal) -> Result<(), String> {
    STATE.with(|s| {
        let st = s.borrow();
        let used = st
            .caller_stats
            .get(&caller)
            .map(|x| x.created_count)
            .unwrap_or(0);
        if used >= st.cfg.max_canisters_per_caller {
            Err(format!(
                "Per-caller canister limit reached: {}/{}",
                used, st.cfg.max_canisters_per_caller
            ))
        } else {
            Ok(())
        }
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
        if let Some(m) = a.max_canisters_per_caller {
            st.cfg.max_canisters_per_caller = m;
        }
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
}

#[pre_upgrade]
fn pre_upgrade() {
    let st = STATE.with(|s| s.borrow().clone());
    ic_cdk::storage::stable_save((st,)).expect("pre_upgrade: stable_save failed");
}

#[post_upgrade]
fn post_upgrade() {
    let (st,): (State,) = ic_cdk::storage::stable_restore().unwrap_or_default();
    STATE.with(|s| *s.borrow_mut() = st);
}

/// ===== Upload API (chunked) =====

#[update]
async fn create_upload() -> Result<u64, String> {
    let caller = msg_caller();
    must_allowed(caller)?;
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

/// ===== Factory: create + install (supports Install/Reinstall/Upgrade) =====

#[update]
async fn create_and_install_with(
    req: CreateInstallRequest,
) -> Result<CreateInstallResponse, String> {
    let caller = msg_caller();
    must_allowed(caller)?;
    must_have_cycles_left()?;
    caller_stats_ok(caller)?;

    let initial_balance = ic_cdk::api::canister_cycle_balance();

    // Retrieve and assemble wasm
    let (wasm_bytes, module_hash_hex) = STATE.with(|s| {
        let st = s.borrow();
        let up = st
            .uploads
            .get(&req.upload_id)
            .ok_or_else(|| "upload_id not found".to_string())?;
        if up.owner != caller {
            return Err("not owner of upload".into());
        }
        if up.committed_hash.is_none() {
            return Err("upload not committed".into());
        }
        let mut buf = Vec::with_capacity(up.total_len as usize);
        for c in &up.chunks {
            buf.extend_from_slice(c);
        }
        let hex = sha256_hex(&buf);
        Ok::<(Vec<u8>, String), String>((buf, hex))
    })?;

    // Create the canister with caller as controller (+ optional extras)
    let mut controllers = vec![caller];
    if let Some(mut extras) = req.extra_controllers.clone() {
        controllers.append(&mut extras);
    }

    let create_args = CreateCanisterArgs {
        settings: Some(CanisterSettings {
            controllers: Some(controllers.clone()),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
            reserved_cycles_limit: None,
            log_visibility: None,
            wasm_memory_limit: None,
            wasm_memory_threshold: None,
        }),
    };

    let res = Call::unbounded_wait(Principal::management_canister(), "create_canister")
        .with_arg(&create_args)
        .with_cycles(req.cycles)
        .await
        .map_err(|e| format!("create_canister failed: {e:?}"))?;

    // The response is decoded using .candid::<T>()
    let (canister_id,): (CanisterId,) = res
        .candid()
        .map_err(|e| format!("Candid decode failed: {e:?}"))?;

    // Install/reinstall/upgrade
    let mode = match req.mode {
        Mode::Install => CanisterInstallMode::Install,
        Mode::Reinstall => CanisterInstallMode::Reinstall,
        Mode::Upgrade => CanisterInstallMode::Upgrade(None),
    };

    let install_args = InstallCodeArgs {
        mode,
        canister_id,
        wasm_module: wasm_bytes,
        arg: req.init_arg.clone(),
    };

    // install_code(install_args)
    //     .await
    //     .map_err(|(code, msg)| format!("install_code failed: {code:?}: {msg}"))?;

    // install_code(&install_args)
    //     .await
    //     .map_err(|(code, msg)| format!("install_code failed: {code:?}: {msg}"))?;

    install_code(&install_args)
        .await
        .map_err(|e| format!("install_code failed: {e:?}"))?;

    // Optional handoff: ensure only caller (+extras) remain as controllers.
    if req.handoff {
        let upd = UpdateSettingsArgs {
            canister_id,
            settings: CanisterSettings {
                controllers: Some(controllers),
                compute_allocation: None,
                memory_allocation: None,
                freezing_threshold: None,
                reserved_cycles_limit: None,
                log_visibility: None,
                wasm_memory_limit: None,
                wasm_memory_threshold: None,
            },
        };
        update_settings(&upd)
            .await
            .map_err(|e| format!("update_settings failed: {e:?}"))?;
    }

    let final_balance = ic_cdk::api::canister_cycle_balance();
    let cycles_used = initial_balance.saturating_sub(final_balance);

    // Bump caller stats, clean upload (to free memory)
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.caller_stats
            .entry(caller)
            .and_modify(|c| {
                c.created_count += 1;
                c.last_created_at = Some(ic_cdk::api::time());
            })
            .or_insert_with(|| CallerStats {
                created_count: 1,
                last_created_at: Some(ic_cdk::api::time()),
            });
        st.uploads.remove(&req.upload_id);
        st.total_canisters_created += 1;
    });

    Ok(CreateInstallResponse {
        canister_id,
        module_hash_hex,
        cycles_used,
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
fn my_stats() -> CallerStats {
    let caller = msg_caller();
    STATE.with(|s| {
        s.borrow()
            .caller_stats
            .get(&caller)
            .cloned()
            .unwrap_or_default()
    })
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
            unique_callers: st.caller_stats.len() as u64,
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
        assert_eq!(config.max_canisters_per_caller, 100);
        assert_eq!(config.min_factory_cycles, 5_000_000_000_000);
        assert_eq!(config.max_upload_size, DEFAULT_MAX_UPLOAD_SIZE);
        assert!(!config.emergency_stop);
    }
}
