// External imports
use candid::Principal;
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::BTreeMap;

// Internal imports
use crate::capsule::domain::Capsule;
use crate::capsule_store::CapsuleStore;
use crate::memory::{with_capsule_store, with_capsule_store_mut};
use crate::types::{
    CapsuleHeader, CapsuleId, CapsuleInfo, CapsuleUpdateData, Error, Result13, Result14,
};
use crate::upload::types::{Result15, UploadFinishResult};

// Rolling hash storage for upload verification
thread_local! {
    static UPLOAD_HASH: RefCell<BTreeMap<u64, Sha256>> = const { RefCell::new(BTreeMap::new()) };
}

// Import modules
mod admin;
mod auth;
pub mod capsule;
pub mod capsule_acl;
pub mod capsule_store;
mod folder;
mod gallery;
mod http;
pub mod legacy;
pub mod memories;
mod memory;
mod person;
pub mod personal;
mod session;
mod state;
pub mod types;
mod unified_types;
mod upgrade;
mod upload;
mod user;
mod utils;

#[cfg(test)]
mod test_utils;

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

#[ic_cdk::update]
fn capsules_create(subject: Option<types::PersonRef>) -> std::result::Result<Capsule, Error> {
    crate::capsule::commands::capsules_create(subject)
}

#[ic_cdk::query]
fn capsules_read_basic(capsule_id: Option<String>) -> std::result::Result<CapsuleInfo, Error> {
    match capsule_id {
        Some(id) => crate::capsule::query::capsules_read_basic(id),
        None => crate::capsule::query::capsule_read_self_basic(),
    }
}

#[ic_cdk::query]
fn capsules_read_full(capsule_id: Option<String>) -> std::result::Result<Capsule, Error> {
    match capsule_id {
        Some(id) => crate::capsule::query::capsules_read(id),
        None => crate::capsule::query::capsule_read_self(),
    }
}

#[ic_cdk::update]
fn capsules_update(
    capsule_id: String,
    updates: CapsuleUpdateData,
) -> std::result::Result<Capsule, Error> {
    crate::capsule::commands::capsules_update(capsule_id, updates)
}

#[ic_cdk::update]
fn capsules_delete(capsule_id: String) -> std::result::Result<(), Error> {
    crate::capsule::commands::capsules_delete(capsule_id)
}

#[ic_cdk::query]
fn capsules_list() -> Vec<CapsuleHeader> {
    crate::capsule::query::capsules_list()
}

#[ic_cdk::update]
fn capsules_bind_neon(
    resource_type: types::ResourceType,
    resource_id: String,
    bind: bool,
) -> std::result::Result<(), Error> {
    crate::capsule::commands::resources_bind_neon(resource_type, resource_id, bind)
}

#[ic_cdk::query]
fn get_user_settings() -> std::result::Result<types::UserSettingsResponse, Error> {
    crate::capsule::query::get_user_settings()
}

#[ic_cdk::update]
fn update_user_settings(
    updates: types::UserSettingsUpdateData,
) -> std::result::Result<types::UserSettingsResponse, Error> {
    crate::capsule::commands::update_user_settings(updates)
}

// ============================================================================
// GALLERY MANAGEMENT (7 functions)
// ============================================================================
#[ic_cdk::update]
async fn galleries_create(
    gallery_data: types::GalleryData,
) -> std::result::Result<types::Gallery, Error> {
    // TESTING: Using gallery.rs implementation
    gallery::commands::galleries_create(gallery_data)
}

#[ic_cdk::update]
async fn galleries_create_with_memories(
    gallery_data: types::GalleryData,
    sync_memories: bool,
) -> std::result::Result<types::Gallery, Error> {
    // TESTING: Using gallery.rs implementation
    gallery::commands::galleries_create_with_memories(gallery_data, sync_memories)
}

#[ic_cdk::update]
fn update_gallery_storage_location(
    gallery_id: String,
    new_location: Vec<types::BlobHosting>,
) -> std::result::Result<(), Error> {
    // Delegate to gallery module (thin facade)
    gallery::commands::update_gallery_storage_location(gallery_id, new_location)
}

#[ic_cdk::query]
fn galleries_list() -> Vec<types::GalleryHeader> {
    // Delegate to gallery module (thin facade)
    gallery::query::galleries_list()
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
    gallery::commands::galleries_update(gallery_id, update_data)
}

#[ic_cdk::update]
async fn galleries_delete(gallery_id: String) -> std::result::Result<(), Error> {
    // Delegate to gallery module (thin facade)
    gallery::commands::galleries_delete(gallery_id)
}

// ============================================================================
// GALLERY UTILITY ENDPOINTS
// ============================================================================

/// Get gallery size information for debugging stable memory limits
#[ic_cdk::query]
fn get_gallery_size_info(gallery: types::Gallery) -> String {
    gallery::util::get_gallery_size_report(&gallery)
}

/// Get detailed gallery size breakdown
#[ic_cdk::query]
fn get_gallery_size_breakdown(gallery: types::Gallery) -> gallery::util::GallerySizeInfo {
    gallery::util::get_gallery_size_breakdown(&gallery)
}

/// Calculate just the gallery size (without capsule overhead)
#[ic_cdk::query]
fn calculate_gallery_size(gallery: types::Gallery) -> u64 {
    gallery::util::estimate_gallery_size(&gallery)
}

/// Calculate gallery size when stored in capsule context
#[ic_cdk::query]
fn calculate_gallery_capsule_size(gallery: types::Gallery) -> u64 {
    gallery::util::estimate_gallery_capsule_size(&gallery)
}

// ============================================================================
// FOLDERS
// ============================================================================

#[ic_cdk::update]
fn folders_create(folder_data: types::FolderData) -> std::result::Result<types::Folder, Error> {
    folder::commands::folders_create(folder_data)
}

#[ic_cdk::update]
fn folders_update(
    folder_id: String,
    update_data: types::FolderUpdateData,
) -> std::result::Result<types::Folder, Error> {
    folder::commands::folders_update(folder_id, update_data)
}

#[ic_cdk::update]
fn folders_delete(folder_id: String) -> std::result::Result<(), Error> {
    folder::commands::folders_delete(folder_id)
}

#[ic_cdk::query]
fn folders_list() -> Vec<types::FolderHeader> {
    folder::query::folders_list()
}

// ============================================================================
// MEMORIES
// ============================================================================

// === Core ===
#[ic_cdk::update]
fn memories_create(
    capsule_id: CapsuleId,
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
    capsule_id: CapsuleId,
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
fn memories_add_asset(
    memory_id: String,
    asset: crate::memories::types::InternalBlobAssetInput,
    idem: String,
) -> types::Result20 {
    use crate::memories::core::memories_add_asset_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    match memories_add_asset_core(&env, &mut store, memory_id, asset, idem) {
        Ok(asset_id) => types::Result20::Ok(asset_id),
        Err(error) => types::Result20::Err(error),
    }
}

#[ic_cdk::update]
fn memories_add_inline_asset(
    memory_id: String,
    asset: crate::memories::types::InlineAssetInput,
    idem: String,
) -> types::Result20 {
    use crate::memories::core::memories_add_inline_asset_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    match memories_add_inline_asset_core(&env, &mut store, memory_id, asset, idem) {
        Ok(asset_id) => types::Result20::Ok(asset_id),
        Err(error) => types::Result20::Err(error),
    }
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
    use crate::memories::utils::generate_asset_links_for_memory_header;
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
                    // Paginate first, then build headers + asset links for the
                    // requested page only. Building links for every memory
                    // before slicing made the cost grow with the whole capsule
                    // and exceeded the 5B instruction limit at ~100 memories.
                    let total = capsule.memories.len();
                    let start_idx = cursor.and_then(|c| c.parse::<usize>().ok()).unwrap_or(0);

                    let page_items: Vec<types::MemoryHeader> = capsule
                        .memories
                        .values()
                        .skip(start_idx)
                        .take(limit as usize)
                        .map(|memory| {
                            let header = memory.to_header();
                            generate_asset_links_for_memory_header(header, memory)
                        })
                        .collect();

                    let end_idx = start_idx.saturating_add(page_items.len());
                    let next_cursor = if end_idx < total {
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
    use crate::memories::utils::generate_asset_links_for_memory_header;
    use crate::memory::with_capsule_store;
    use crate::types::PersonRef;

    let caller = PersonRef::from_caller();
    let limit = limit.unwrap_or(50).min(100); // Default 50, max 100

    with_capsule_store(|store| {
        store
            .get(&capsule_id)
            .and_then(|capsule| {
                if capsule.has_read_access(&caller) {
                    // Filter memories by capsule_id field and generate asset links
                    let memories_with_asset_links: Vec<types::MemoryHeader> = capsule
                        .memories
                        .values()
                        .filter(|memory| memory.capsule_id == capsule_id)
                        .map(|memory| {
                            let header = memory.to_header();
                            generate_asset_links_for_memory_header(header, memory)
                        })
                        .collect();

                    // Pagination logic
                    let start_idx = cursor.and_then(|c| c.parse::<usize>().ok()).unwrap_or(0);
                    let end_idx = (start_idx + limit as usize).min(memories_with_asset_links.len());
                    let page_items = memories_with_asset_links[start_idx..end_idx].to_vec();

                    let next_cursor = if end_idx < memories_with_asset_links.len() {
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
fn uploads_begin(capsule_id: CapsuleId, expected_chunks: u32, idem: String) -> Result13 {
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

/// Only the caller who began an upload session may feed or finish it. Checked
/// before the rolling hash is touched, so a stranger who knows a session id
/// cannot corrupt or cancel somebody else's upload.
fn require_session_owner(session_id: u64) -> std::result::Result<(), Error> {
    let session = upload::service::session_caller(session_id).ok_or(Error::NotFound)?;
    if session != ic_cdk::api::msg_caller() {
        return Err(Error::NotFound);
    }
    Ok(())
}

/// Upload a chunk for an active session
#[ic_cdk::update]
async fn uploads_put_chunk(
    session_id: u64,
    chunk_idx: u32,
    bytes: Vec<u8>,
) -> std::result::Result<(), Error> {
    require_session_owner(session_id)?;
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
    if let Err(e) = require_session_owner(session_id) {
        return Result15::Err(e);
    }
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
    // Shows who uploads into which capsule: admins only
    if !admin::is_admin(&ic_cdk::api::msg_caller()) {
        return Err(Error::Unauthorized);
    }
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

/// Does a capsule the caller may write to hold a memory that uses this blob?
fn caller_owns_blob(blob_id: &str) -> bool {
    let caller = types::PersonRef::from_caller();
    with_capsule_store(|store| {
        store
            .paginate(None, u32::MAX, capsule_store::types::PaginationOrder::Asc)
            .items
            .iter()
            .filter(|c| c.has_write_access(&caller))
            .any(|c| {
                c.memories.values().any(|m| {
                    m.blob_internal_assets
                        .iter()
                        .any(|a| a.blob_ref.locator == blob_id)
                })
            })
    })
}

/// Delete blob by ID (unified endpoint for all blob types)
#[ic_cdk::update]
fn blob_delete(blob_id: String) -> types::Result6 {
    // Determine blob type and handle accordingly
    if blob_id.starts_with("blob_") {
        // Only for a blob that one of the caller's own capsules refers to
        if !caller_owns_blob(&blob_id) {
            return types::Result6::Err(Error::NotFound);
        }
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

// Persistence hooks for canister upgrades
#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    // Nothing to do: everything lives in stable structures, which persist by
    // themselves. Do NOT stable_save here: it overwrites the memory manager
    // and loses the blob store (see upgrade.rs).
}

#[ic_cdk::post_upgrade]
async fn post_upgrade() {
    // Must come first, before any stable structure is touched (see upgrade.rs)
    if upgrade::legacy_snapshot_present() {
        // Upgrading from a version that snapshotted in pre_upgrade: restore
        // once. The third value of that snapshot was the state of the old
        // in-backend canister factory; it is read and dropped.
        if let Ok((capsule_data, admin_data, _old_factory_state)) =
            ic_cdk::storage::stable_restore::<(
                Vec<(String, Capsule)>,
                Vec<Principal>,
                candid::Reserved,
            )>()
        {
            with_capsule_store_mut(|store| {
                for (id, capsule) in capsule_data {
                    store.upsert(id, capsule);
                }
            });
            admin::import_admins_from_upgrade(admin_data);
        } else {
            // Do not continue: the memory manager would start empty over the
            // old data. Trapping rolls the upgrade back and keeps everything.
            ic_cdk::trap("post_upgrade: cannot read the snapshot of the previous version");
        }
    }

    ic_cdk::println!("Post-upgrade: stable memory structures restored automatically");

    // Initialize HTTP secret store
    http::secret_store::post_upgrade();
    http::secret_store::schedule_random_key();

    // Set skip certification for private assets
    ic_cdk::api::certified_data_set(
        ic_http_certification::utils::skip_certification_certified_data(),
    );
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

/// TEMPORARY DEV METHOD: Clear all memories in a capsule
///
/// WARNING: This is a developer method that bypasses normal ACL checks
/// and uses atomic operations. It should be refined or removed before
/// production use.
///
/// TODO: Implement proper ACL checks and individual memory deletion
/// TODO: Add proper error handling and rollback mechanisms
/// TODO: Consider if this should be a user-facing feature
#[ic_cdk::update]
fn dev_clear_all_memories_in_capsule(
    capsule_id: String,
    delete_assets: bool,
) -> Result<crate::memories::types::BulkDeleteResult, Error> {
    // Wipes a capsule: only for someone who may write to it
    let caller = types::PersonRef::from_caller();
    let allowed = with_capsule_store(|store| {
        store
            .get(&capsule_id)
            .map(|c| c.has_write_access(&caller))
            .unwrap_or(false)
    });
    if !allowed {
        return Err(Error::NotFound);
    }

    use crate::memories::core::_dev_clear_all_memories_in_capsule_core;
    use crate::memories::{CanisterEnv, StoreAdapter};

    let env = CanisterEnv;
    let mut store = StoreAdapter;

    _dev_clear_all_memories_in_capsule_core(&env, &mut store, capsule_id, delete_assets)
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
    let store = StoreAdapter;

    memories_list_assets_core(&env, &store, memory_id)
}

// ============================================================================
// HTTP REQUEST HANDLERS
// ============================================================================

#[ic_cdk::init]
async fn init(arg: Option<personal::BackendInitArg>) {
    // Personal mode if the factory passed an owner; shared mode otherwise
    personal::init(arg, ic_cdk::api::msg_caller());

    // Initialize HTTP secret store
    http::secret_store::init().await;
    http::secret_store::schedule_random_key();

    // Set skip certification for private assets
    ic_cdk::api::certified_data_set(
        ic_http_certification::utils::skip_certification_certified_data(),
    );
}

#[ic_cdk::query]
fn http_request(
    req: ic_http_certification::HttpRequest,
) -> ic_http_certification::HttpResponse<'static> {
    let mut response = http::handle(req);

    // Add skip certification headers for private assets
    ic_http_certification::utils::add_skip_certification_header(
        ic_cdk::api::data_certificate().expect("no data cert"),
        &mut response,
    );

    response
}

// TODO: Enable when streaming is needed
// #[ic_cdk::query]
// fn http_request_streaming_callback(token: http::streaming::CallbackToken)
//     -> http::streaming::CallbackResponse
// {
//     // Note: This will need to be made async in the actual implementation
//     // For now, we'll return an error since we can't use async in query methods
//     Err("Streaming callback not yet implemented".to_string())
// }

// HTTP token functions are now handled by the TokenService

#[ic_cdk::query]
fn mint_http_token(
    memory_id: String,
    variants: Vec<String>,
    asset_ids: Option<Vec<String>>,
    ttl_secs: u32,
) -> String {
    crate::http::token_service::TokenService::mint_token(memory_id, variants, asset_ids, ttl_secs)
}

/// Bulk token minting for efficient dashboard loading
/// Returns a vector of (memory_id, token) pairs
#[ic_cdk::query]
fn mint_http_tokens_bulk(
    memory_ids: Vec<String>,
    variants: Vec<String>,
    asset_ids: Option<Vec<String>>,
    ttl_secs: u32,
) -> Vec<(String, String)> {
    crate::http::token_service::TokenService::mint_tokens_bulk(
        memory_ids, variants, asset_ids, ttl_secs,
    )
}

// Export the interface for the smart contract.
/// Runs before an update call from outside is accepted (and before the canister
/// pays for it). In a personal canister only the owner may send updates,
/// except the few that invited people need. Calls from other canisters do not
/// pass through here: every endpoint still checks access itself.
#[ic_cdk::inspect_message]
fn inspect_message() {
    let method = ic_cdk::api::msg_method_name();
    if personal::accepts_update(&ic_cdk::api::msg_caller(), &method) {
        ic_cdk::api::accept_message();
    } else {
        ic_cdk::trap("this personal canister accepts updates from its owner only");
    }
}

/// Shared canister (many users) or the personal canister of one owner
#[ic_cdk::query]
fn canister_mode() -> personal::CanisterMode {
    personal::mode()
}

/// The factory that may read this personal canister's cycles and tops it up
#[ic_cdk::query]
fn personal_factory() -> Option<Principal> {
    personal::factory()
}

/// Owner only: name the factory that may read this canister's cycles
/// (`null`: nobody but the owner, which also ends top-ups).
#[ic_cdk::update]
fn personal_factory_set(factory: Option<Principal>) -> std::result::Result<(), Error> {
    personal::set_factory_by_owner(&ic_cdk::api::msg_caller(), factory)
}

fn must_read_cycles() {
    let caller = ic_cdk::api::msg_caller();
    if !personal::may_read_cycles(&caller, ic_cdk::api::is_controller(&caller)) {
        ic_cdk::trap("cycles are shown to the owner and the factory only");
    }
}

/// All cycles this canister holds, including the part the network keeps
/// back as the freezing reserve. For the owner and the factory.
#[ic_cdk::query]
fn cycles_balance() -> u128 {
    must_read_cycles();
    ic_cdk::api::canister_cycle_balance()
}

/// The cycles this canister can still spend before it freezes: the balance
/// above the freezing reserve. This is what the factory tops up against.
#[ic_cdk::query]
fn cycles_liquid_balance() -> u128 {
    must_read_cycles();
    ic_cdk::api::canister_liquid_cycle_balance()
}

// ============================================================================
// INHERITANCE ("legacy"): trustees, death confirmation, events, shared reads
// See legacy.rs for the rules.
// ============================================================================

fn legacy_update<F>(capsule_id: String, f: F) -> std::result::Result<legacy::LegacyStatus, Error>
where
    F: FnOnce(&mut Capsule, Principal, u64) -> std::result::Result<legacy::LegacyStatus, Error>,
{
    let caller = ic_cdk::api::msg_caller();
    let now = ic_cdk::api::time();
    with_capsule_store_mut(|store| {
        // A missing capsule and a capsule the caller has no role in look the same
        store
            .update_with(&capsule_id, |capsule| f(capsule, caller, now))
            .map_err(|e| match e {
                Error::InvalidArgument(_) | Error::Conflict(_) | Error::ResourceExhausted => e,
                Error::Unauthorized => Error::NotFound,
                other => other,
            })
    })
}

/// Owner: set trustees, how many must confirm a death, and the waiting period
#[ic_cdk::update]
fn legacy_settings_set(
    capsule_id: String,
    trustees: Vec<Principal>,
    required_confirmations: u32,
    waiting_period_secs: u64,
) -> std::result::Result<legacy::LegacyStatus, Error> {
    legacy_update(capsule_id, |capsule, caller, now| {
        legacy::settings_set(
            capsule,
            caller,
            trustees,
            required_confirmations,
            waiting_period_secs,
            now,
        )
    })
}

/// Owner, controller or trustee: the inheritance state of a capsule
#[ic_cdk::query]
fn legacy_status(capsule_id: String) -> std::result::Result<legacy::LegacyStatus, Error> {
    let caller = ic_cdk::api::msg_caller();
    let now = ic_cdk::api::time();
    with_capsule_store(|store| {
        let capsule = store.get(&capsule_id).ok_or(Error::NotFound)?;
        legacy::status(&capsule, caller, now)
    })
}

/// Trustee: confirm the owner's death (starts the waiting period once enough confirmed)
#[ic_cdk::update]
fn legacy_confirm_death(capsule_id: String) -> std::result::Result<legacy::LegacyStatus, Error> {
    legacy_update(capsule_id, legacy::confirm_death)
}

/// Trustee: take the confirmation back (only before the death is in effect)
#[ic_cdk::update]
fn legacy_withdraw_confirmation(
    capsule_id: String,
) -> std::result::Result<legacy::LegacyStatus, Error> {
    legacy_update(capsule_id, legacy::withdraw_confirmation)
}

/// Owner: "I am alive" - drops every death confirmation, at any time
#[ic_cdk::update]
fn legacy_cancel_death(capsule_id: String) -> std::result::Result<legacy::LegacyStatus, Error> {
    legacy_update(capsule_id, legacy::cancel_death)
}

/// Owner: record that an event happened (wedding, graduation, custom, ...)
#[ic_cdk::update]
fn legacy_event_record(
    capsule_id: String,
    event: capsule::domain::AccessEvent,
) -> std::result::Result<legacy::LegacyStatus, Error> {
    legacy_update(capsule_id, |capsule, caller, now| {
        legacy::event_record(capsule, caller, event, now)
    })
}

/// Owner: take a recorded event back
#[ic_cdk::update]
fn legacy_event_revoke(
    capsule_id: String,
    event: capsule::domain::AccessEvent,
) -> std::result::Result<legacy::LegacyStatus, Error> {
    legacy_update(capsule_id, |capsule, caller, now| {
        legacy::event_revoke(capsule, caller, event, now)
    })
}

/// Anyone granted access: the memories of a capsule they may view right now
#[ic_cdk::query]
fn memories_list_shared(
    capsule_id: String,
    cursor: Option<String>,
    limit: Option<u32>,
) -> std::result::Result<crate::capsule_store::types::Page<types::MemoryHeader>, Error> {
    let caller = ic_cdk::api::msg_caller();
    let now = ic_cdk::api::time();
    with_capsule_store(|store| {
        let capsule = store.get(&capsule_id).ok_or(Error::NotFound)?;
        legacy::list_shared(&capsule, caller, cursor, limit, now)
    })
}

/// Anyone granted access: one memory they may view right now
#[ic_cdk::query]
fn memories_read_shared(
    capsule_id: String,
    memory_id: String,
) -> std::result::Result<types::Memory, Error> {
    let caller = ic_cdk::api::msg_caller();
    let now = ic_cdk::api::time();
    with_capsule_store(|store| {
        let capsule = store.get(&capsule_id).ok_or(Error::NotFound)?;
        legacy::read_shared(&capsule, caller, &memory_id, now)
    })
}

ic_cdk::export_candid!();
