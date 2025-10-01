// External imports
use candid::Principal;
use hex;
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::BTreeMap;

// Internal imports
use crate::capsule_store::{types::PaginationOrder as Order, CapsuleStore};
use crate::memory::{with_capsule_store, with_capsule_store_mut};
use crate::types::{Error, Result_13, Result_14, Result_15, UploadFinishResult};

// Rolling hash storage for upload verification
thread_local! {
    static UPLOAD_HASH: RefCell<BTreeMap<u64, Sha256>> = RefCell::new(BTreeMap::new());
}

// Import modules
mod admin;
mod auth;
mod canister_factory;
mod capsule;
mod capsule_acl;
mod capsule_store;
mod gallery;
mod memories;
mod memory;
mod person;
mod session;
mod state;
mod types;
mod unified_types;
mod upload;
mod user;

// ============================================================================
// CORE SYSTEM & UTILITY FUNCTIONS (3 functions)
// ============================================================================

#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {name}!")
}

#[ic_cdk::query]
fn whoami() -> Principal {
    ic_cdk::api::msg_caller()
}

#[ic_cdk::query]
fn get_canister_size_stats() -> state::CanisterSizeStats {
    state::get_canister_size_stats()
}

// ============================================================================
// AUTHENTICATION & USER MANAGEMENT (6 functions)
// ============================================================================
// Register user and prove nonce in one call (optimized for II auth flow)
#[ic_cdk::update]
fn register_with_nonce(nonce: String) -> std::result::Result<(), Error> {
    // Delegate to user module (frontend adapter)
    user::register_user_with_nonce(nonce)
}

#[ic_cdk::query]
fn verify_nonce(nonce: String) -> Result_14 {
    // Verify and return the principal who proved this nonce
    match auth::get_nonce_proof(nonce) {
        Some(principal) => Result_14::Ok(principal),
        None => Result_14::Err(types::Error::NotFound),
    }
}

// ============================================================================
// ADMINISTRATIVE FUNCTIONS (4 functions)
// ============================================================================
#[ic_cdk::update]
fn add_admin(principal: Principal) -> std::result::Result<(), Error> {
    admin::add_admin(principal)
}

#[ic_cdk::update]
fn remove_admin(principal: Principal) -> std::result::Result<(), Error> {
    admin::remove_admin(principal)
}

#[ic_cdk::query]
fn list_admins() -> Vec<Principal> {
    admin::list_admins()
}

#[ic_cdk::query]
fn list_superadmins() -> Vec<Principal> {
    admin::list_superadmins()
}

// ============================================================================
// CAPSULE MANAGEMENT (5 functions)
// ============================================================================

// Capsule management endpoints
#[ic_cdk::update]
fn capsules_create(
    subject: Option<types::PersonRef>,
) -> std::result::Result<types::Capsule, Error> {
    // Delegate to capsule module (thin facade)
    capsule::capsules_create(subject)
}

#[ic_cdk::query]
fn capsules_read_basic(
    capsule_id: Option<String>,
) -> std::result::Result<types::CapsuleInfo, Error> {
    // Delegate to capsule module (thin facade)
    match capsule_id {
        Some(id) => capsule::capsules_read_basic(id),
        None => capsule::capsule_read_self_basic(),
    }
}

#[ic_cdk::query]
fn capsules_read_full(capsule_id: Option<String>) -> std::result::Result<types::Capsule, Error> {
    // Delegate to capsule module (thin facade)
    match capsule_id {
        Some(id) => capsule::capsules_read(id),
        None => capsule::capsule_read_self(),
    }
}

#[ic_cdk::update]
fn capsules_update(
    capsule_id: String,
    updates: types::CapsuleUpdateData,
) -> std::result::Result<types::Capsule, Error> {
    // Delegate to capsule module (thin facade)
    capsule::capsules_update(capsule_id, updates)
}

#[ic_cdk::update]
fn capsules_delete(capsule_id: String) -> std::result::Result<(), Error> {
    // Delegate to capsule module (thin facade)
    capsule::capsules_delete(capsule_id)
}

#[ic_cdk::query]
fn capsules_list() -> Vec<types::CapsuleHeader> {
    // Delegate to capsule module (thin facade)
    capsule::capsules_list()
}

#[ic_cdk::update]
fn capsules_bind_neon(
    resource_type: types::ResourceType,
    resource_id: String,
    bind: bool,
) -> std::result::Result<(), Error> {
    // Delegate to capsule module (thin facade)
    capsule::resources_bind_neon(resource_type, resource_id, bind)
}

// ============================================================================
// GALLERY MANAGEMENT (7 functions)
// ============================================================================
#[ic_cdk::update]
async fn galleries_create(
    gallery_data: types::GalleryData,
) -> std::result::Result<types::Gallery, Error> {
    // TESTING: Using gallery.rs implementation
    gallery::galleries_create(gallery_data)
}

#[ic_cdk::update]
async fn galleries_create_with_memories(
    gallery_data: types::GalleryData,
    sync_memories: bool,
) -> std::result::Result<types::Gallery, Error> {
    // TESTING: Using gallery.rs implementation
    gallery::galleries_create_with_memories(gallery_data, sync_memories)
}

#[ic_cdk::update]
fn update_gallery_storage_location(
    gallery_id: String,
    new_location: types::GalleryStorageLocation,
) -> std::result::Result<(), Error> {
    // Delegate to gallery module (thin facade)
    gallery::update_gallery_storage_location(gallery_id, new_location)
}

#[ic_cdk::query]
fn galleries_list() -> Vec<types::GalleryHeader> {
    // Delegate to gallery module (thin facade)
    gallery::galleries_list()
}

#[ic_cdk::query]
fn galleries_read(gallery_id: String) -> std::result::Result<types::Gallery, Error> {
    use crate::capsule_store::types::PaginationOrder as Order;
    use crate::memory::with_capsule_store;
    use crate::types::PersonRef;

    let caller = PersonRef::from_caller();

    // Find gallery in caller's self-capsule
    with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller))
            .and_then(|capsule| capsule.galleries.get(&gallery_id).cloned())
            .ok_or(types::Error::NotFound)
    })
}

#[ic_cdk::update]
async fn galleries_update(
    gallery_id: String,
    update_data: types::GalleryUpdateData,
) -> std::result::Result<types::Gallery, Error> {
    // Delegate to gallery module (thin facade)
    gallery::galleries_update(gallery_id, update_data)
}

#[ic_cdk::update]
async fn galleries_delete(gallery_id: String) -> std::result::Result<(), Error> {
    // Delegate to gallery module (thin facade)
    gallery::galleries_delete(gallery_id)
}

// ============================================================================
// GALLERY UTILITY ENDPOINTS
// ============================================================================

/// Get gallery size information for debugging stable memory limits
#[ic_cdk::query]
fn get_gallery_size_info(gallery: types::Gallery) -> String {
    gallery::get_gallery_size_report(&gallery)
}

/// Get detailed gallery size breakdown
#[ic_cdk::query]
fn get_gallery_size_breakdown(gallery: types::Gallery) -> gallery::GallerySizeInfo {
    gallery::get_gallery_size_breakdown(&gallery)
}

/// Calculate just the gallery size (without capsule overhead)
#[ic_cdk::query]
fn calculate_gallery_size(gallery: types::Gallery) -> u64 {
    gallery::estimate_gallery_size(&gallery)
}

/// Calculate gallery size when stored in capsule context
#[ic_cdk::query]
fn calculate_gallery_capsule_size(gallery: types::Gallery) -> u64 {
    gallery::estimate_gallery_capsule_size(&gallery)
}

// ============================================================================
// MEMORIES
// ============================================================================

// === Core ===
#[ic_cdk::update]
fn memories_create(
    capsule_id: types::CapsuleId,
    bytes: Option<Vec<u8>>,
    blob_ref: Option<types::BlobRef>,
    external_location: Option<types::StorageEdgeBlobType>,
    external_storage_key: Option<String>,
    external_url: Option<String>,
    external_size: Option<u64>,
    external_hash: Option<Vec<u8>>,
    asset_metadata: types::AssetMetadata,
    idem: String,
) -> std::result::Result<types::MemoryId, Error> {
    use crate::memories::core::memories_create_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    memories_create_core(
        &env,
        &mut store,
        capsule_id,
        bytes,
        blob_ref,
        external_location,
        external_storage_key,
        external_url,
        external_size,
        external_hash,
        asset_metadata,
        idem,
    )
}

#[ic_cdk::query]
fn memories_read(memory_id: String) -> std::result::Result<types::Memory, Error> {
    use crate::memories::core::memories_read_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let store = StoreAdapter;

    // Get the full memory but strip blob data for performance
    let memory = memories_read_core(&env, &store, memory_id)?;

    // Create a new memory with empty blob data but preserved metadata
    let mut stripped_memory = memory.clone();
    for asset in &mut stripped_memory.inline_assets {
        asset.bytes.clear(); // Remove blob data, keep metadata
    }
    // blob_internal_assets and blob_external_assets keep their references (no blob data to clear)

    Ok(stripped_memory)
}

#[ic_cdk::query]
fn memories_read_with_assets(memory_id: String) -> std::result::Result<types::Memory, Error> {
    use crate::memories::core::memories_read_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let store = StoreAdapter;

    // Return full memory with all blob data (original behavior)
    memories_read_core(&env, &store, memory_id)
}

#[ic_cdk::query]
fn memories_read_asset(memory_id: String, asset_index: u32) -> std::result::Result<Vec<u8>, Error> {
    use crate::memories::core::memories_read_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let store = StoreAdapter;

    // Get the full memory first
    let memory = memories_read_core(&env, &store, memory_id)?;

    // Find the asset by index
    let asset_index = asset_index as usize;

    // Check inline assets first
    if asset_index < memory.inline_assets.len() {
        return Ok(memory.inline_assets[asset_index].bytes.clone());
    }

    // Check blob internal assets
    let inline_count = memory.inline_assets.len();
    if asset_index < inline_count + memory.blob_internal_assets.len() {
        let _blob_index = asset_index - inline_count;
        // For blob internal assets, we need to fetch from blob store
        // This would require implementing blob retrieval logic
        return Err(Error::NotImplemented(
            "Blob internal asset retrieval not yet implemented".to_string(),
        ));
    }

    // Check blob external assets
    let blob_internal_count = memory.blob_internal_assets.len();
    if asset_index < inline_count + blob_internal_count + memory.blob_external_assets.len() {
        let _external_index = asset_index - inline_count - blob_internal_count;
        // For external assets, we would need to fetch from external storage
        return Err(Error::NotImplemented(
            "External asset retrieval not yet implemented".to_string(),
        ));
    }

    Err(Error::InvalidArgument(format!(
        "Asset index {} out of range",
        asset_index
    )))
}

#[ic_cdk::update]
fn memories_update(
    memory_id: String,
    updates: types::MemoryUpdateData,
) -> types::MemoryOperationResponse {
    use crate::memories::core::memories_update_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    match memories_update_core(&env, &mut store, memory_id, updates) {
        Ok(()) => types::MemoryOperationResponse {
            memory_id: None,
            message: "Memory updated successfully".to_string(),
            success: true,
        },
        Err(e) => types::MemoryOperationResponse {
            memory_id: None,
            message: format!("Failed to update memory: {:?}", e),
            success: false,
        },
    }
}

#[ic_cdk::update]
fn memories_delete(memory_id: String) -> types::MemoryOperationResponse {
    use crate::memories::core::memories_delete_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    match memories_delete_core(&env, &mut store, memory_id) {
        Ok(()) => types::MemoryOperationResponse {
            memory_id: None,
            message: "Memory deleted successfully".to_string(),
            success: true,
        },
        Err(e) => types::MemoryOperationResponse {
            memory_id: None,
            message: format!("Failed to delete memory: {:?}", e),
            success: false,
        },
    }
}

#[ic_cdk::query]
fn memories_list(capsule_id: String) -> types::MemoryListResponse {
    crate::memories::list(capsule_id)
}

// === Presence ===

/// Check presence for multiple memories on ICP (consolidated from get_memory_presence_icp and get_memory_list_presence_icp)
#[ic_cdk::query]
fn memories_ping(
    memory_ids: Vec<String>,
) -> std::result::Result<Vec<types::MemoryPresenceResult>, Error> {
    crate::memories::ping(memory_ids)
}

// === Upload ===

/// Get upload configuration for TypeScript client discoverability
#[ic_cdk::query]
fn upload_config() -> types::UploadConfig {
    use crate::upload::types::{CAPSULE_INLINE_BUDGET, CHUNK_SIZE, INLINE_MAX};

    types::UploadConfig {
        inline_max: INLINE_MAX as u32,
        chunk_size: CHUNK_SIZE as u32,
        inline_budget_per_capsule: CAPSULE_INLINE_BUDGET as u32,
    }
}

/// Begin chunked upload for large files
#[ic_cdk::update]
fn uploads_begin(
    capsule_id: types::CapsuleId,
    asset_metadata: types::AssetMetadata,
    expected_chunks: u32,
    idem: String,
) -> Result_13 {
    match with_capsule_store_mut(|store| {
        let mut upload_service = upload::service::UploadService::new();
        upload_service.begin_upload(store, capsule_id, asset_metadata, expected_chunks, idem)
    }) {
        Ok(session_id) => {
            let sid = session_id.0;
            // Initialize rolling hash for this session
            UPLOAD_HASH.with(|m| {
                m.borrow_mut().insert(sid, Sha256::new());
            });
            ic_cdk::println!("UPLOAD_HASH_INIT sid={}", sid);
            Result_13::Ok(sid)
        }
        Err(error) => Result_13::Err(error),
    }
}

/// Upload a chunk for an active session
#[ic_cdk::update]
async fn uploads_put_chunk(
    session_id: u64,
    chunk_idx: u32,
    bytes: Vec<u8>,
) -> std::result::Result<(), Error> {
    // Breadcrumb logging: log what we receive from Candid
    let hex = bytes
        .iter()
        .take(8)
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ");
    ic_cdk::println!(
        "PUT_CHUNK_RECV: session_id={}, chunk_idx={}, data_len={}, prefix=[{}]",
        session_id,
        chunk_idx,
        bytes.len(),
        hex
    );

    // Update rolling hash FIRST (before writing)
    match UPLOAD_HASH.with(|m| {
        if let Some(hasher) = m.borrow_mut().get_mut(&session_id) {
            hasher.update(&bytes);
            Ok(())
        } else {
            Err(Error::NotFound)
        }
    }) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    // Then write chunk to storage
    memory::with_capsule_store_mut(|store| {
        let mut upload_service = upload::service::UploadService::new();
        let session_id = upload::types::SessionId(session_id);
        match upload_service.put_chunk(store, &session_id, chunk_idx, bytes) {
            Ok(()) => Ok(()),
            Err(err) => Err(err),
        }
    })
}

/// Commit chunks to create final memory
#[ic_cdk::update]
async fn uploads_finish(session_id: u64, expected_sha256: Vec<u8>, total_len: u64) -> Result_15 {
    ic_cdk::println!("FINISH_START sid={} expected_len={}", session_id, total_len);

    // Verify rolling hash FIRST (before any other operations)
    let computed_hash = match UPLOAD_HASH.with(|m| {
        if let Some(hasher) = m.borrow_mut().remove(&session_id) {
            Ok(hasher.finalize().to_vec())
        } else {
            Err(Error::NotFound)
        }
    }) {
        Ok(hash) => hash,
        Err(e) => {
            ic_cdk::println!("FINISH_ERROR sid={} err=hash_not_found", session_id);
            return Result_15::Err(e);
        }
    };

    // Compare with client's expected hash
    if computed_hash != expected_sha256 {
        ic_cdk::println!(
            "FINISH_ERROR sid={} err=checksum_mismatch computed={:?} expected={:?}",
            session_id,
            &computed_hash[..8],
            &expected_sha256[..8]
        );
        return Result_15::Err(Error::InvalidArgument(format!(
            "checksum_mismatch: computed={}, expected={}",
            hex::encode(&computed_hash),
            hex::encode(&expected_sha256)
        )));
    }

    ic_cdk::println!("FINISH_HASH_OK sid={} len={}", session_id, total_len);

    // Use real UploadService with actual store integration
    let hash: [u8; 32] = match expected_sha256.clone().try_into() {
        Ok(h) => h,
        Err(_) => {
            ic_cdk::println!(
                "FINISH_ERROR sid={} err=invalid_hash_length got={}",
                session_id,
                expected_sha256.len()
            );
            return Result_15::Err(types::Error::InvalidArgument(format!(
                "invalid_hash_length: expected 32 bytes, got {}",
                expected_sha256.len()
            )));
        }
    };

    memory::with_capsule_store_mut(|store| {
        let mut upload_service = upload::service::UploadService::new();
        let session_id = upload::types::SessionId(session_id);
        match upload_service.commit(store, session_id, hash, total_len) {
            Ok((blob_id, memory_id)) => {
                ic_cdk::println!(
                    "FINISH_INDEX_COMMITTED sid={} blob={} mem={}",
                    session_id.0,
                    blob_id,
                    memory_id
                );

                let result = UploadFinishResult {
                    memory_id: memory_id.clone(),
                    blob_id: blob_id.clone(),
                    remote_id: None,
                    size: total_len,
                    checksum_sha256: Some(hash),
                    storage_backend: upload::types::StorageBackend::Icp,
                    storage_location: format!("icp://blob/{}", blob_id),
                    uploaded_at: ic_cdk::api::time(),
                    expires_at: None,
                };

                ic_cdk::println!("FINISH_OK sid={}", session_id.0);
                Result_15::Ok(result)
            }
            Err(err) => {
                ic_cdk::println!("FINISH_ERROR sid={} err={:?}", session_id.0, err);
                Result_15::Err(err)
            }
        }
    })
}

/// Abort upload session and cleanup
#[ic_cdk::update]
async fn uploads_abort(session_id: u64) -> std::result::Result<(), Error> {
    // Use real UploadService with actual store integration
    memory::with_capsule_store_mut(|store| {
        let mut upload_service = upload::service::UploadService::new();
        let session_id = upload::types::SessionId(session_id);
        match upload_service.abort(store, session_id) {
            Ok(()) => Ok(()),
            Err(err) => Err(err),
        }
    })
}

// ============================================================================
// SESSION MANAGEMENT ENDPOINTS (Development/Debug)
// ============================================================================

/// Clear all upload sessions (development/debugging only)
#[ic_cdk::update]
fn sessions_clear_all() -> std::result::Result<String, Error> {
    memory::with_capsule_store_mut(|_store| {
        let upload_service = upload::service::UploadService::new();
        upload_service.clear_all_sessions();
        Ok("All sessions cleared".to_string())
    })
}

/// Get session statistics for monitoring
#[ic_cdk::query]
fn sessions_stats() -> std::result::Result<String, Error> {
    memory::with_capsule_store_mut(|_store| {
        let upload_service = upload::service::UploadService::new();
        let total = upload_service.total_session_count();
        let (pending, committed) = upload_service.session_count_by_status();

        let stats = format!(
            "Total sessions: {}, Pending: {}, Committed: {}",
            total, pending, committed
        );
        Ok(stats)
    })
}

/// List all sessions for debugging
#[ic_cdk::query]
fn sessions_list() -> std::result::Result<String, Error> {
    memory::with_capsule_store_mut(|_store| {
        let upload_service = upload::service::UploadService::new();
        let sessions = upload_service.list_upload_sessions();

        let mut result = String::new();
        result.push_str(&format!("Found {} sessions:\n", sessions.len()));

        for (id, session) in sessions {
            let status = match session.status {
                session::types::SessionStatus::Pending => "Pending",
                session::types::SessionStatus::Committed { .. } => "Committed",
            };

            result.push_str(&format!(
                "Session {}: caller={}, capsule={}, status={}, created={}\n",
                id, session.caller, session.capsule_id, status, session.created_at
            ));
        }

        Ok(result)
    })
}

/// Clean up expired sessions
#[ic_cdk::update]
fn sessions_cleanup_expired() -> std::result::Result<String, Error> {
    memory::with_capsule_store_mut(|_store| {
        let upload_service = upload::service::UploadService::new();
        const SESSION_EXPIRY_MS: u64 = 30 * 60 * 1000; // 30 minutes
        upload_service.cleanup_expired_sessions(SESSION_EXPIRY_MS);
        Ok("Expired sessions cleaned up".to_string())
    })
}

// ============================================================================
// EMERGENCY RECOVERY ENDPOINTS (Admin Only)
// ============================================================================

/// Emergency function to clear all stable memory data
/// WARNING: This will delete all stored data and should only be used for recovery
#[ic_cdk::update]
fn clear_all_stable_memory() -> std::result::Result<(), Error> {
    // Only allow admin to call this
    let caller = ic_cdk::api::msg_caller();
    if !admin::is_admin(&caller) {
        return Err(types::Error::Unauthorized);
    }

    memory::clear_all_stable_memory().map_err(types::Error::Internal)
}

// ============================================================================
// CHUNKED ASSET UPLOAD ENDPOINTS - ICP Canister API
// ============================================================================

// OLD UPLOAD FUNCTIONS REMOVED - Migration to new hybrid architecture complete
// All old upload functions have been removed and replaced with the new workflow:
// - memories_create_inline (≤32KB files)
// - uploads_begin + uploads_put_chunk + uploads_finish (large files)
// - uploads_abort (cancel uploads)
// All upload endpoints are now organized under the MEMORIES section above.

// ============================================================================
// DEBUG ENDPOINTS (dev only)
// ============================================================================

/// Debug endpoint to compute SHA256 hash of provided bytes
#[ic_cdk::query]
fn debug_sha256(bytes: Vec<u8>) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(&bytes);
    hex::encode(hash)
}

/// Read blob data by locator (for asset retrieval)
#[ic_cdk::query]
fn blob_read(locator: String) -> std::result::Result<Vec<u8>, Error> {
    upload::blob_read(locator)
}

/// Read blob data by locator in chunks (for large files)
/// Returns individual chunks to avoid IC message size limits
#[ic_cdk::query]
fn blob_read_chunk(locator: String, chunk_index: u32) -> std::result::Result<Vec<u8>, Error> {
    upload::blob_store::blob_read_chunk(locator, chunk_index)
}

/// Get blob metadata including total chunk count
#[ic_cdk::query]
fn blob_get_meta(locator: String) -> std::result::Result<types::BlobMeta, Error> {
    upload::blob_store::blob_get_meta(locator)
}

/// Debug endpoint to upload chunk with base64 data (dev only)
#[ic_cdk::update]
async fn debug_put_chunk_b64(
    session_id: u64,
    chunk_idx: u32,
    b64: String,
) -> std::result::Result<(), Error> {
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64)
        .map_err(|_| types::Error::InvalidArgument("bad base64".into()))?;
    memory::with_capsule_store_mut(|store| {
        let mut upload_service = upload::service::UploadService::new();
        let session_id = upload::types::SessionId(session_id);
        match upload_service.put_chunk(store, &session_id, chunk_idx, bytes) {
            Ok(()) => Ok(()),
            Err(e) => Err(types::Error::from(e)),
        }
    })
}

/// Debug endpoint to finish upload with hex hash (dev only)
#[ic_cdk::update]
async fn debug_finish_hex(
    session_id: u64,
    sha256_hex: String,
    total_len: u64,
) -> std::result::Result<(String, types::MemoryId), Error> {
    let bytes =
        hex::decode(sha256_hex).map_err(|_| types::Error::InvalidArgument("bad hex".into()))?;
    if bytes.len() != 32 {
        return Err(types::Error::InvalidArgument(
            "hash must be 32 bytes".into(),
        ));
    }
    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&bytes);

    memory::with_capsule_store_mut(|store| {
        let mut upload_service = upload::service::UploadService::new();
        let session_id = upload::types::SessionId(session_id);
        match upload_service.commit(store, session_id, hash_array, total_len) {
            Ok((blob_id, memory_id)) => Ok((blob_id, memory_id)),
            Err(e) => Err(types::Error::from(e)),
        }
    })
}

// ============================================================================
// PERSONAL CANISTER MANAGEMENT (22 functions)
// ============================================================================
#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::update]
async fn create_personal_canister() -> canister_factory::PersonalCanisterCreationResponse {
    match canister_factory::create_personal_canister().await {
        Ok(response) => response,
        Err(error) => canister_factory::PersonalCanisterCreationResponse {
            success: false,
            canister_id: None,
            message: format!("Personal canister creation failed: {error}"),
        },
    }
}

#[ic_cdk::query]
fn get_creation_status() -> Option<canister_factory::CreationStatusResponse> {
    canister_factory::get_creation_status()
}

#[ic_cdk::query]
fn get_personal_canister_id(user: Principal) -> Option<Principal> {
    canister_factory::get_personal_canister_id(user)
}

#[ic_cdk::query]
fn get_my_personal_canister_id() -> Option<Principal> {
    canister_factory::get_my_personal_canister_id()
}

#[ic_cdk::query]
fn get_detailed_creation_status() -> Option<canister_factory::DetailedCreationStatus> {
    canister_factory::get_detailed_creation_status()
}

// Admin personal canister creation functions
#[ic_cdk::query]
fn get_user_creation_status(
    user: Principal,
) -> std::result::Result<Option<canister_factory::DetailedCreationStatus>, Error> {
    canister_factory::get_user_creation_status(user)
}

#[ic_cdk::query]
fn get_user_migration_status(
    user: Principal,
) -> std::result::Result<Option<canister_factory::DetailedCreationStatus>, Error> {
    get_user_creation_status(user)
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn list_all_creation_states(
) -> std::result::Result<Vec<(Principal, canister_factory::DetailedCreationStatus)>, Error> {
    canister_factory::list_all_creation_states()
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn list_all_migration_states(
) -> std::result::Result<Vec<(Principal, canister_factory::DetailedCreationStatus)>, Error> {
    list_all_creation_states()
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn get_creation_states_by_status(
    status: canister_factory::CreationStatus,
) -> std::result::Result<Vec<(Principal, canister_factory::DetailedCreationStatus)>, Error> {
    canister_factory::get_creation_states_by_status(status)
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn get_migration_states_by_status(
    status: canister_factory::CreationStatus,
) -> std::result::Result<Vec<(Principal, canister_factory::DetailedCreationStatus)>, Error> {
    get_creation_states_by_status(status)
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::update]
fn clear_creation_state(user: Principal) -> std::result::Result<bool, Error> {
    canister_factory::clear_creation_state(user)
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::update]
fn clear_migration_state(user: Principal) -> std::result::Result<bool, Error> {
    clear_creation_state(user)
}

// Admin controls for migration (only available with migration feature)
#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::update]
fn set_personal_canister_creation_enabled(enabled: bool) -> std::result::Result<(), Error> {
    canister_factory::set_personal_canister_creation_enabled(enabled)
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn get_personal_canister_creation_stats(
) -> std::result::Result<canister_factory::PersonalCanisterCreationStats, Error> {
    canister_factory::get_personal_canister_creation_stats()
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn is_personal_canister_creation_enabled() -> std::result::Result<bool, Error> {
    canister_factory::is_personal_canister_creation_enabled()
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn is_migration_enabled() -> std::result::Result<bool, Error> {
    canister_factory::is_migration_enabled()
}

// Legacy function names for backward compatibility
#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::update]
async fn migrate_capsule() -> canister_factory::PersonalCanisterCreationResponse {
    create_personal_canister().await
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn get_migration_status() -> Option<canister_factory::CreationStatusResponse> {
    get_creation_status()
}

#[cfg(feature = "migration")]
#[ic_cdk::query]
fn get_detailed_migration_status() -> Option<canister_factory::DetailedCreationStatus> {
    get_detailed_creation_status()
}

#[cfg(feature = "migration")]
#[ic_cdk::update]
fn set_migration_enabled(enabled: bool) -> std::result::Result<(), Error> {
    set_personal_canister_creation_enabled(enabled)
}

#[cfg(feature = "migration")]
#[ic_cdk::query]
fn get_migration_stats(
) -> std::result::Result<canister_factory::PersonalCanisterCreationStats, Error> {
    get_personal_canister_creation_stats()
}

// pub use memories::*; // Disabled for now

// Persistence hooks for canister upgrades
#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    // Stable memory structures (StableBTreeMap) automatically persist their data
    // No explicit action needed for stable memory - ic-stable-structures handles this

    // For backward compatibility, also serialize thread_local data using the old approach
    let capsule_data = with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .map(|capsule| (capsule.id.clone(), capsule))
            .collect::<Vec<(String, types::Capsule)>>()
    });
    let admin_data = admin::export_admins_for_upgrade();

    #[cfg(feature = "migration")]
    {
        // Also serialize migration state if migration feature is enabled
        let migration_data = canister_factory::export_migration_state_for_upgrade();
        ic_cdk::storage::stable_save((capsule_data, admin_data, migration_data))
            .expect("Failed to save data to stable storage");
    }

    #[cfg(not(feature = "migration"))]
    {
        // Save without migration data if migration feature is disabled
        ic_cdk::storage::stable_save((capsule_data, admin_data))
            .expect("Failed to save data to stable storage");
    }

    ic_cdk::println!("Pre-upgrade: stable memory structures will persist automatically");
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    // Stable memory structures (StableBTreeMap) automatically restore their data
    // No explicit action needed for stable memory - ic-stable-structures handles this

    // For backward compatibility, restore thread_local data using the old approach
    #[cfg(feature = "migration")]
    {
        // Restore capsules, admins, and migration state after upgrade
        if let Ok((capsule_data, admin_data, migration_data)) = ic_cdk::storage::stable_restore::<(
            Vec<(String, types::Capsule)>,
            Vec<Principal>,
            canister_factory::PersonalCanisterCreationStateData,
        )>() {
            with_capsule_store_mut(|store| {
                for (id, capsule) in capsule_data {
                    store.upsert(id, capsule);
                }
            });
            admin::import_admins_from_upgrade(admin_data);
            canister_factory::import_migration_state_from_upgrade(migration_data);
        }
    }

    #[cfg(not(feature = "migration"))]
    {
        // Restore capsules and admins only if migration feature is disabled
        if let Ok((capsule_data, admin_data)) =
            ic_cdk::storage::stable_restore::<(Vec<(String, types::Capsule)>, Vec<Principal>)>()
        {
            with_capsule_store_mut(|store| {
                for (id, capsule) in capsule_data {
                    store.upsert(id, capsule);
                }
            });
            admin::import_admins_from_upgrade(admin_data);
        }
    }
    // If restore fails, start with empty state (no panic)

    ic_cdk::println!("Post-upgrade: stable memory structures restored automatically");
}

// DEBUG: Cross-call canary to test StableBTreeMap persistence
#[ic_cdk::update]
fn debug_blob_write_canary(pmid: String, idx: u32, n: u32) {
    use crate::upload::blob_store::pmid_hash32;
    let stem = pmid_hash32(&pmid);
    let payload = vec![0xAA; n as usize];
    ic_cdk::println!("CANARY_WRITE pmid={} idx={} len={}", pmid, idx, n);
    upload::blob_store::STABLE_BLOB_STORE.with(|s| {
        s.borrow_mut().insert((stem, idx), payload);
    });
}

#[ic_cdk::query]
fn debug_blob_read_canary(pmid: String, idx: u32) -> Option<u32> {
    use crate::upload::blob_store::pmid_hash32;
    let stem = pmid_hash32(&pmid);
    let result = upload::blob_store::STABLE_BLOB_STORE
        .with(|s| s.borrow().get(&(stem, idx)).map(|v| v.len() as u32));
    ic_cdk::println!("CANARY_READ pmid={} idx={} result={:?}", pmid, idx, result);
    result
}

// Temporary diagnostic endpoint to probe inline bytes length
#[ic_cdk::update]
fn _probe_inline_len(content: Option<Vec<u8>>) -> (u64, Vec<u8>) {
    match content {
        Some(b) => {
            let len = b.len() as u64;
            let mut p = b.clone();
            p.truncate(8);
            (len, p)
        }
        None => (0, vec![]),
    }
}

// Export the interface for the smart contract.
ic_cdk::export_candid!();
