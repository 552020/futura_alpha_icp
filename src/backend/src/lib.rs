// External imports
use candid::Principal;
use hex;
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::BTreeMap;

// Internal imports
use crate::capsule_store::{types::PaginationOrder as Order, CapsuleStore};
use crate::memory::{with_capsule_store, with_capsule_store_mut};
use crate::types::{Error, Result13, Result14};
use crate::upload::types::{Result15, UploadFinishResult};

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
mod util;

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
fn verify_nonce(nonce: String) -> Result14 {
    // Verify and return the principal who proved this nonce
    match auth::get_nonce_proof(nonce) {
        Some(principal) => Result14::Ok(principal),
        None => Result14::Err(types::Error::NotFound),
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

#[ic_cdk::query]
fn get_user_settings() -> std::result::Result<types::UserSettingsResponse, Error> {
    // Delegate to capsule module (thin facade)
    capsule::get_user_settings()
}

#[ic_cdk::update]
fn update_user_settings(
    updates: types::UserSettingsUpdateData,
) -> std::result::Result<types::UserSettingsResponse, Error> {
    // Delegate to capsule module (thin facade)
    capsule::update_user_settings(updates)
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
) -> types::Result20 {
    use crate::memories::core::memories_create_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    match memories_create_core(
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
    ) {
        Ok(memory_id) => types::Result20::Ok(memory_id),
        Err(error) => types::Result20::Err(error),
    }
}

#[ic_cdk::update]
fn memories_create_with_internal_blobs(
    capsule_id: types::CapsuleId,
    memory_metadata: crate::memories::types::MemoryMetadata,
    internal_blob_assets: Vec<crate::memories::types::InternalBlobAssetInput>,
    idem: String,
) -> types::Result20 {
    use crate::memories::core::create::memories_create_with_internal_blobs_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    match memories_create_with_internal_blobs_core(
        &env,
        &mut store,
        capsule_id,
        memory_metadata,
        internal_blob_assets,
        idem,
    ) {
        Ok(memory_id) => types::Result20::Ok(memory_id),
        Err(error) => types::Result20::Err(error),
    }
}

#[ic_cdk::query]
fn memories_read(memory_id: String) -> std::result::Result<types::Memory, Error> {
    use crate::memories::core::memories_read_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let store = StoreAdapter;

    // Get the full memory with all content
    memories_read_core(&env, &store, memory_id)
}

#[ic_cdk::query]
fn memories_read_asset(
    memory_id: String,
    asset_index: u32,
) -> std::result::Result<types::MemoryAssetData, Error> {
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
        let asset = &memory.inline_assets[asset_index];
        return Ok(types::MemoryAssetData::Inline {
            bytes: asset.bytes.clone(),
            content_type: asset.metadata.get_base().mime_type.clone(),
            size: asset.bytes.len() as u64,
            sha256: asset.metadata.get_base().sha256.map(|h| h.to_vec()),
        });
    }

    // Check blob internal assets
    let inline_count = memory.inline_assets.len();
    if asset_index < inline_count + memory.blob_internal_assets.len() {
        let blob_index = asset_index - inline_count;
        let asset = &memory.blob_internal_assets[blob_index];
        return Ok(types::MemoryAssetData::InternalBlob {
            blob_id: asset.blob_ref.locator.clone(),
            size: asset.blob_ref.len,
            sha256: asset.blob_ref.hash.map(|h| h.to_vec()),
        });
    }

    // Check blob external assets
    let blob_internal_count = memory.blob_internal_assets.len();
    if asset_index < inline_count + blob_internal_count + memory.blob_external_assets.len() {
        let external_index = asset_index - inline_count - blob_internal_count;
        let asset = &memory.blob_external_assets[external_index];
        return Ok(types::MemoryAssetData::ExternalUrl {
            url: asset.url.clone().unwrap_or_default(),
            size: Some(asset.metadata.get_base().bytes),
            sha256: asset.metadata.get_base().sha256.map(|h| h.to_vec()),
        });
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
) -> std::result::Result<types::Memory, Error> {
    use crate::memories::core::memories_update_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    memories_update_core(&env, &mut store, memory_id, updates)
}

#[ic_cdk::update]
fn memories_delete(memory_id: String, delete_assets: bool) -> std::result::Result<(), Error> {
    use crate::memories::core::memories_delete_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    memories_delete_core(&env, &mut store, memory_id, delete_assets)
}

#[ic_cdk::query]
fn memories_list(
    capsule_id: String,
    cursor: Option<String>,
    limit: Option<u32>,
) -> std::result::Result<crate::capsule_store::types::Page<types::MemoryHeader>, Error> {
    use crate::capsule_store::CapsuleStore;
    use crate::memory::with_capsule_store;
    use crate::types::PersonRef;

    let caller = PersonRef::from_caller();
    let limit = limit.unwrap_or(50).min(100); // Default 50, max 100

    with_capsule_store(|store| {
        store
            .get(&capsule_id)
            .and_then(|capsule| {
                // Check if caller has read access
                if capsule.has_read_access(&caller) {
                    // Get memories with pagination
                    let memories: Vec<types::MemoryHeader> = capsule
                        .memories
                        .values()
                        .map(|memory| memory.to_header())
                        .collect();

                    // Simple pagination implementation
                    let start_idx = cursor.and_then(|c| c.parse::<usize>().ok()).unwrap_or(0);

                    let end_idx = (start_idx + limit as usize).min(memories.len());
                    let page_items = memories[start_idx..end_idx].to_vec();

                    let next_cursor = if end_idx < memories.len() {
                        Some(end_idx.to_string())
                    } else {
                        None
                    };

                    Some(crate::capsule_store::types::Page {
                        items: page_items,
                        next_cursor,
                    })
                } else {
                    None
                }
            })
            .ok_or(Error::NotFound)
    })
}

/// List memories filtered by capsule_id field (for UUID v7 implementation)
#[ic_cdk::query]
fn memories_list_by_capsule(
    capsule_id: String,
    cursor: Option<String>,
    limit: Option<u32>,
) -> std::result::Result<crate::capsule_store::types::Page<types::MemoryHeader>, Error> {
    use crate::capsule_store::CapsuleStore;
    use crate::memory::with_capsule_store;
    use crate::types::PersonRef;

    let caller = PersonRef::from_caller();
    let limit = limit.unwrap_or(50).min(100); // Default 50, max 100

    with_capsule_store(|store| {
        store
            .get(&capsule_id)
            .and_then(|capsule| {
                if capsule.has_read_access(&caller) {
                    // Filter memories by capsule_id field
                    let memories: Vec<types::MemoryHeader> = capsule
                        .memories
                        .values()
                        .filter(|memory| memory.capsule_id == capsule_id)
                        .map(|memory| memory.to_header())
                        .collect();

                    // Pagination logic
                    let start_idx = cursor.and_then(|c| c.parse::<usize>().ok()).unwrap_or(0);
                    let end_idx = (start_idx + limit as usize).min(memories.len());
                    let page_items = memories[start_idx..end_idx].to_vec();

                    let next_cursor = if end_idx < memories.len() {
                        Some(end_idx.to_string())
                    } else {
                        None
                    };

                    Some(crate::capsule_store::types::Page {
                        items: page_items,
                        next_cursor,
                    })
                } else {
                    None
                }
            })
            .ok_or(Error::NotFound)
    })
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
fn uploads_begin(capsule_id: types::CapsuleId, expected_chunks: u32, idem: String) -> Result13 {
    match with_capsule_store_mut(|store| {
        upload::service::begin_upload(store, capsule_id, expected_chunks, idem)
    }) {
        Ok(session_id) => {
            let sid = session_id.0;
            // Initialize rolling hash for this session
            UPLOAD_HASH.with(|m| {
                m.borrow_mut().insert(sid, Sha256::new());
            });
            ic_cdk::println!("UPLOAD_HASH_INIT sid={}", sid);
            Result13::Ok(sid)
        }
        Err(error) => Result13::Err(error),
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
        let session_id = upload::types::SessionId(session_id);
        upload::service::put_chunk(store, &session_id, chunk_idx, bytes)
    })
}

/// Commit chunks to create final memory
#[ic_cdk::update]
async fn uploads_finish(session_id: u64, expected_sha256: Vec<u8>, total_len: u64) -> Result15 {
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
            return Result15::Err(e);
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
        return Result15::Err(Error::InvalidArgument(format!(
            "checksum_mismatch: computed={}, expected={}",
            hex::encode(&computed_hash),
            hex::encode(&expected_sha256)
        )));
    }

    ic_cdk::println!("FINISH_HASH_OK sid={} len={}", session_id, total_len);

    // Use functional upload service
    let hash: [u8; 32] = match expected_sha256.clone().try_into() {
        Ok(h) => h,
        Err(_) => {
            ic_cdk::println!(
                "FINISH_ERROR sid={} err=invalid_hash_length got={}",
                session_id,
                expected_sha256.len()
            );
            return Result15::Err(types::Error::InvalidArgument(format!(
                "invalid_hash_length: expected 32 bytes, got {}",
                expected_sha256.len()
            )));
        }
    };

    memory::with_capsule_store_mut(|store| {
        let session_id = upload::types::SessionId(session_id);
        match upload::service::commit(store, session_id, hash, total_len) {
            Ok(blob_id) => {
                ic_cdk::println!(
                    "FINISH_BLOB_COMMITTED sid={} blob={}",
                    session_id.0,
                    blob_id
                );

                let result = UploadFinishResult {
                    memory_id: "".to_string(), // No memory created - separate concern
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
                Result15::Ok(result)
            }
            Err(err) => {
                ic_cdk::println!("FINISH_ERROR sid={} err={:?}", session_id.0, err);
                Result15::Err(err)
            }
        }
    })
}

/// Abort upload session and cleanup
#[ic_cdk::update]
async fn uploads_abort(session_id: u64) -> std::result::Result<(), Error> {
    // Use functional upload service
    memory::with_capsule_store_mut(|store| {
        let session_id = upload::types::SessionId(session_id);
        upload::service::abort(store, session_id)
    })
}

// ============================================================================
// SESSION MANAGEMENT ENDPOINTS (Development/Debug)
// ============================================================================

/// Clear all upload sessions (development/debugging only)
#[ic_cdk::update]
fn sessions_clear_all() -> std::result::Result<String, Error> {
    memory::with_capsule_store_mut(|_store| {
        upload::service::clear_all_sessions();
        Ok("All sessions cleared".to_string())
    })
}

/// Get session statistics for monitoring
#[ic_cdk::query]
fn sessions_stats() -> std::result::Result<String, Error> {
    memory::with_capsule_store_mut(|_store| {
        let total = upload::service::total_session_count();
        let (pending, committed) = upload::service::session_count_by_status();

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
        let sessions = upload::service::list_upload_sessions();

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
        const SESSION_EXPIRY_MS: u64 = 30 * 60 * 1000; // 30 minutes
        upload::service::cleanup_expired_sessions(SESSION_EXPIRY_MS);
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

/// Delete blob by ID (unified endpoint for all blob types)
#[ic_cdk::update]
fn blob_delete(blob_id: String) -> types::Result6 {
    // Determine blob type and handle accordingly
    if blob_id.starts_with("blob_") {
        // Internal blob (ICP blob store)
        match upload::blob_store::blob_delete(blob_id) {
            Ok(()) => types::Result6::Ok("Internal blob deleted successfully".to_string()),
            Err(error) => types::Result6::Err(error),
        }
    } else if blob_id.starts_with("inline_") {
        // Inline asset (stored in memory)
        // For inline assets, we can't delete the blob directly since it's part of the memory
        // This would require deleting the entire memory or the specific asset
        types::Result6::Err(Error::InvalidArgument(
            "Inline assets cannot be deleted directly. Delete the memory or use asset removal endpoints.".to_string(),
        ))
    } else if blob_id.starts_with("external_") {
        // External blob (S3, IPFS, etc.)
        // External blobs are managed by external systems
        types::Result6::Err(Error::InvalidArgument(
            "External blobs cannot be deleted via this endpoint. Use external storage management."
                .to_string(),
        ))
    } else {
        // Unknown blob type
        types::Result6::Err(Error::InvalidArgument(format!(
            "Unknown blob type for ID: {}",
            blob_id
        )))
    }
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
        let session_id = upload::types::SessionId(session_id);
        upload::service::put_chunk(store, &session_id, chunk_idx, bytes).map_err(types::Error::from)
    })
}

/// Debug endpoint to finish upload with hex hash (dev only)
#[ic_cdk::update]
async fn debug_finish_hex(
    session_id: u64,
    sha256_hex: String,
    total_len: u64,
) -> std::result::Result<String, Error> {
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
        let session_id = upload::types::SessionId(session_id);
        upload::service::commit(store, session_id, hash_array, total_len)
            .map_err(types::Error::from)
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

// ============================================================================
// BULK MEMORY OPERATIONS API
// ============================================================================

/// Bulk delete multiple memories in a single operation
#[ic_cdk::update]
fn memories_delete_bulk(
    capsule_id: String,
    memory_ids: Vec<String>,
    delete_assets: bool,
) -> Result<crate::memories::types::BulkDeleteResult, Error> {
    use crate::memories::core::memories_delete_bulk_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    memories_delete_bulk_core(&env, &mut store, capsule_id, memory_ids, delete_assets)
}

/// Delete ALL memories in a capsule (high-risk operation)
#[ic_cdk::update]
fn memories_delete_all(
    capsule_id: String,
    delete_assets: bool,
) -> Result<crate::memories::types::BulkDeleteResult, Error> {
    use crate::memories::core::memories_delete_all_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    memories_delete_all_core(&env, &mut store, capsule_id, delete_assets)
}

/// Clean up all assets from a memory while preserving the memory record
#[ic_cdk::update]
fn memories_cleanup_assets_all(
    memory_id: String,
) -> Result<crate::memories::types::AssetCleanupResult, Error> {
    use crate::memories::core::memories_cleanup_assets_all_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    memories_cleanup_assets_all_core(&env, &mut store, memory_id)
}

/// Bulk cleanup assets from multiple memories
#[ic_cdk::update]
fn memories_cleanup_assets_bulk(
    memory_ids: Vec<String>,
) -> Result<types::BulkResult<String>, Error> {
    use crate::memories::core::memories_cleanup_assets_bulk_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    match memories_cleanup_assets_bulk_core(&env, &mut store, memory_ids.clone()) {
        Ok(_result) => {
            // Convert BulkAssetCleanupResult to BulkResult<String>
            // Since the core function doesn't provide per-item tracking yet,
            // we'll simulate the results based on the aggregate counts
            let mut ok = Vec::new();
            let failed = Vec::new();

            // For now, we'll treat all input memory_ids as successful
            // since the core function doesn't provide per-item failure tracking
            // TODO: Update core function to provide per-item results
            for memory_id in memory_ids {
                ok.push(memory_id);
            }

            Ok(types::BulkResult { ok, failed })
        }
        Err(e) => Err(e),
    }
}

// ============================================================================
// ASSET OPERATIONS API
// ============================================================================

/// Remove a specific asset from a memory by asset reference
#[ic_cdk::update]
fn asset_remove(
    memory_id: String,
    asset_ref: String,
) -> Result<crate::memories::types::AssetRemovalResult, Error> {
    use crate::memories::core::asset_remove_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    asset_remove_core(&env, &mut store, memory_id, asset_ref)
}

/// Remove specific inline asset by index
#[ic_cdk::update]
fn asset_remove_inline(
    memory_id: String,
    asset_index: u32,
) -> Result<crate::memories::types::AssetRemovalResult, Error> {
    use crate::memories::core::asset_remove_inline_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    asset_remove_inline_core(&env, &mut store, memory_id, asset_index)
}

/// Remove specific ICP blob asset by blob reference
#[ic_cdk::update]
fn asset_remove_internal(
    memory_id: String,
    blob_ref: String,
) -> Result<crate::memories::types::AssetRemovalResult, Error> {
    use crate::memories::core::asset_remove_internal_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    asset_remove_internal_core(&env, &mut store, memory_id, blob_ref)
}

/// Remove specific external storage asset by storage key
#[ic_cdk::update]
fn asset_remove_external(
    memory_id: String,
    storage_key: String,
) -> Result<crate::memories::types::AssetRemovalResult, Error> {
    use crate::memories::core::asset_remove_external_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    asset_remove_external_core(&env, &mut store, memory_id, storage_key)
}

/// Remove a specific asset from a memory by asset_id
#[ic_cdk::update]
fn asset_remove_by_id(
    memory_id: String,
    asset_id: String,
) -> Result<crate::memories::types::AssetRemovalResult, Error> {
    use crate::memories::core::asset_remove_by_id_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    asset_remove_by_id_core(&env, &mut store, memory_id, asset_id)
}

/// Get a specific asset from a memory by asset_id
#[ic_cdk::query]
fn asset_get_by_id(memory_id: String, asset_id: String) -> Result<types::MemoryAssetData, Error> {
    use crate::memories::core::asset_get_by_id_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let store = StoreAdapter;

    asset_get_by_id_core(&env, &store, memory_id, asset_id)
}

/// List all assets in a memory
#[ic_cdk::query]
fn memories_list_assets(
    memory_id: String,
) -> Result<crate::memories::types::MemoryAssetsList, Error> {
    use crate::memories::core::memories_list_assets_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    memories_list_assets_core(&env, &mut store, memory_id)
}

// Export the interface for the smart contract.
ic_cdk::export_candid!();
