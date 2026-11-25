//! Personal canisters, the way Juno's console creates satellites:
//!
//! 1. An admin uploads the backend wasm (chunked upload API) and releases it.
//! 2. A user calls `create_personal_canister`. The factory creates a canister
//!    with itself and the user as controllers, installs the released wasm with
//!    `init(owner = user)`, then removes itself: the user is the only
//!    controller. After that the factory cannot touch the canister.
//! 3. The registry remembers which canister belongs to which user.
//!
//! Upgrades are pulled, not pushed: the owner fetches a release
//! (`release_wasm`) and installs it with their own identity.
//!
//! A failed step is resumed by calling `create_personal_canister` again: the
//! registry keeps the canister id and how far it got, so no second canister is
//! created and no cycles are spent twice.

use candid::{CandidType, Principal};
use ic_cdk::api::msg_caller;
use ic_cdk::call::Call;
use ic_cdk::management_canister::{
    install_code, update_settings, CanisterInstallMode, CanisterSettings, CreateCanisterArgs,
    CreateCanisterResult, InstallCodeArgs, UpdateSettingsArgs,
};
use ic_cdk_macros::{query, update};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{must_allowed, must_be_admin, must_have_cycles_left, sha256_hex, STATE};

/// Cycles given to a new personal canister (creation fee included)
pub const DEFAULT_PERSONAL_CYCLES: u128 = 2_000_000_000_000; // 2T
const MAX_VERSION_LEN: usize = 32;

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct Release {
    pub version: String,
    pub sha256_hex: String,
    pub wasm: Vec<u8>,
    pub released_at: u64,
    pub released_by: Principal,
}

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct ReleaseInfo {
    pub version: String,
    pub sha256_hex: String,
    pub size: u64,
    pub released_at: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PersonalStatus {
    /// Registered, nothing created yet
    Pending,
    /// Canister exists; factory and user are controllers
    Created,
    /// Code installed; factory is still a controller
    Installed,
    /// Handed off: the user is the only controller
    Ready,
}

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct PersonalCanister {
    pub owner: Principal,
    pub canister_id: Option<Principal>,
    pub status: PersonalStatus,
    pub version: Option<String>,
    pub module_sha256_hex: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
    /// A create call is running for this owner
    pub busy: bool,
    pub last_error: Option<String>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Default)]
pub struct PersonalState {
    pub release: Option<Release>,
    pub canisters: BTreeMap<Principal, PersonalCanister>,
    pub cycles_per_canister: Option<u128>,
}

/// Init argument of the backend wasm (`personal::BackendInitArg` there)
#[derive(CandidType, Serialize, Deserialize)]
struct BackendInitArg {
    owner: Option<Principal>,
}

fn with_personal<R>(f: impl FnOnce(&mut PersonalState) -> R) -> R {
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        f(st.personal.get_or_insert_with(PersonalState::default))
    })
}

fn is_wasm_or_gzip(bytes: &[u8]) -> bool {
    bytes.starts_with(b"\0asm") || bytes.starts_with(&[0x1f, 0x8b])
}

// ---- releases ---------------------------------------------------------------

/// Admin: make a committed upload the release that new personal canisters get.
#[update]
fn release_set(upload_id: u64, version: String) -> Result<ReleaseInfo, String> {
    let caller = msg_caller();
    must_be_admin(caller)?;
    let version = version.trim().to_string();
    if version.is_empty() || version.len() > MAX_VERSION_LEN {
        return Err(format!("version must have 1 to {MAX_VERSION_LEN} characters"));
    }
    let wasm = STATE.with(|s| {
        let st = s.borrow();
        let up = st.uploads.get(&upload_id).ok_or("upload_id not found")?;
        if up.owner != caller {
            return Err("not owner of upload".to_string());
        }
        if up.committed_hash.is_none() {
            return Err("upload not committed".to_string());
        }
        Ok(up.chunks.concat())
    })?;
    if !is_wasm_or_gzip(&wasm) {
        return Err("not a wasm module (plain or gzipped)".into());
    }
    let release = Release {
        version,
        sha256_hex: sha256_hex(&wasm),
        wasm,
        released_at: ic_cdk::api::time(),
        released_by: caller,
    };
    let info = info_of(&release);
    STATE.with(|s| s.borrow_mut().uploads.remove(&upload_id));
    with_personal(|p| p.release = Some(release));
    Ok(info)
}

fn info_of(r: &Release) -> ReleaseInfo {
    ReleaseInfo {
        version: r.version.clone(),
        sha256_hex: r.sha256_hex.clone(),
        size: r.wasm.len() as u64,
        released_at: r.released_at,
    }
}

#[query]
fn release_info() -> Option<ReleaseInfo> {
    with_personal(|p| p.release.as_ref().map(info_of))
}

/// The released wasm, for owners who upgrade their canister themselves.
/// Check it against `release_info().sha256_hex` before installing.
#[query]
fn release_wasm() -> Result<Vec<u8>, String> {
    with_personal(|p| p.release.as_ref().map(|r| r.wasm.clone())).ok_or("no release".into())
}

/// Admin: cycles a new personal canister starts with.
#[update]
fn set_personal_canister_cycles(cycles: u128) -> Result<(), String> {
    must_be_admin(msg_caller())?;
    if cycles < 500_000_000_000 {
        return Err("at least 0.5T cycles: creation alone costs that".into());
    }
    with_personal(|p| p.cycles_per_canister = Some(cycles));
    Ok(())
}

// ---- registry ---------------------------------------------------------------

#[query]
fn my_personal_canister() -> Option<PersonalCanister> {
    let caller = msg_caller();
    with_personal(|p| p.canisters.get(&caller).cloned())
}

/// Admin: every personal canister the factory created.
#[query]
fn personal_canisters_list() -> Result<Vec<PersonalCanister>, String> {
    must_be_admin(msg_caller())?;
    Ok(with_personal(|p| p.canisters.values().cloned().collect()))
}

// ---- creation ---------------------------------------------------------------

/// Create (or finish creating) the caller's personal canister.
/// Idempotent: once Ready it returns the same canister.
#[update]
async fn create_personal_canister() -> Result<PersonalCanister, String> {
    let caller = msg_caller();
    if caller == Principal::anonymous() {
        return Err("sign in first: the anonymous principal cannot own a canister".into());
    }
    must_allowed(caller)?;

    // Take the per-owner lock before the first await
    let now = ic_cdk::api::time();
    let entry = with_personal(|p| {
        let e = p.canisters.entry(caller).or_insert_with(|| PersonalCanister {
            owner: caller,
            canister_id: None,
            status: PersonalStatus::Pending,
            version: None,
            module_sha256_hex: None,
            created_at: now,
            updated_at: now,
            busy: false,
            last_error: None,
        });
        if e.status == PersonalStatus::Ready {
            return Ok(Some(e.clone()));
        }
        if e.busy {
            return Err("creation already in progress".to_string());
        }
        e.busy = true;
        Ok(None)
    })?;
    if let Some(ready) = entry {
        return Ok(ready);
    }

    let result = run_creation(caller).await;

    with_personal(|p| {
        let e = p.canisters.get_mut(&caller).expect("entry exists");
        e.busy = false;
        e.updated_at = ic_cdk::api::time();
        e.last_error = result.as_ref().err().cloned();
        result.map(|_| e.clone())
    })
}

fn update_entry(owner: Principal, f: impl FnOnce(&mut PersonalCanister)) {
    with_personal(|p| {
        if let Some(e) = p.canisters.get_mut(&owner) {
            f(e);
        }
    });
}

async fn run_creation(owner: Principal) -> Result<(), String> {
    let (release, cycles, entry) = with_personal(|p| {
        (
            p.release.clone(),
            p.cycles_per_canister.unwrap_or(DEFAULT_PERSONAL_CYCLES),
            p.canisters.get(&owner).cloned().expect("entry exists"),
        )
    });
    let release = release.ok_or("no release: an admin must call release_set first")?;
    let me = ic_cdk::api::canister_self();

    // 1) create: factory + owner as controllers
    let canister_id = match entry.canister_id {
        Some(id) => id,
        None => {
            must_have_cycles_left()?;
            let args = CreateCanisterArgs {
                settings: Some(settings_with(vec![me, owner])),
            };
            let res = Call::unbounded_wait(Principal::management_canister(), "create_canister")
                .with_arg(&args)
                .with_cycles(cycles)
                .await
                .map_err(|e| format!("create_canister failed: {e:?}"))?;
            // The reply is `record { canister_id : principal }`
            let id = res
                .candid::<CreateCanisterResult>()
                .map_err(|e| format!("create_canister reply: {e:?}"))?
                .canister_id;
            update_entry(owner, |e| {
                e.canister_id = Some(id);
                e.status = PersonalStatus::Created;
            });
            STATE.with(|s| s.borrow_mut().total_canisters_created += 1);
            id
        }
    };

    // 2) install the release with the owner baked in. Reinstall so that a
    //    retry after a half-finished install starts clean (no data yet).
    let status = with_personal(|p| p.canisters[&owner].status);
    if status == PersonalStatus::Created {
        let arg = candid::encode_one(Some(BackendInitArg { owner: Some(owner) }))
            .map_err(|e| format!("encode init arg: {e}"))?;
        install_code(&InstallCodeArgs {
            mode: CanisterInstallMode::Reinstall,
            canister_id,
            wasm_module: release.wasm.clone(),
            arg,
        })
        .await
        .map_err(|e| format!("install_code failed: {e:?}"))?;
        update_entry(owner, |e| {
            e.status = PersonalStatus::Installed;
            e.version = Some(release.version.clone());
            e.module_sha256_hex = Some(release.sha256_hex.clone());
        });
    }

    // 3) hand off: the owner is the only controller
    update_settings(&UpdateSettingsArgs {
        canister_id,
        settings: settings_with(vec![owner]),
    })
    .await
    .map_err(|e| format!("handoff (update_settings) failed: {e:?}"))?;
    update_entry(owner, |e| e.status = PersonalStatus::Ready);
    Ok(())
}

fn settings_with(controllers: Vec<Principal>) -> CanisterSettings {
    CanisterSettings {
        controllers: Some(controllers),
        compute_allocation: None,
        memory_allocation: None,
        freezing_threshold: None,
        reserved_cycles_limit: None,
        log_visibility: None,
        wasm_memory_limit: None,
        wasm_memory_threshold: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_wasm_and_gzip() {
        assert!(is_wasm_or_gzip(b"\0asm\x01\0\0\0"));
        assert!(is_wasm_or_gzip(&[0x1f, 0x8b, 8, 0]));
        assert!(!is_wasm_or_gzip(b"hello"));
        assert!(!is_wasm_or_gzip(b""));
    }

    #[test]
    fn init_arg_matches_the_backend_candid() {
        // (opt record { owner : opt principal })
        let owner = Principal::from_slice(&[7; 29]);
        let bytes = candid::encode_one(Some(BackendInitArg { owner: Some(owner) })).unwrap();
        let back: Option<BackendInitArg> = candid::decode_one(&bytes).unwrap();
        assert_eq!(back.unwrap().owner, Some(owner));
    }
}
