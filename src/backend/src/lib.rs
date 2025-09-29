// use crate::types::HttpRequest; // Disabled for now
// use ic_cdk::api::data_certificate; // Disabled for now
use candid::Principal;

// use ic_http_certification::{utils::add_skip_certification_header, HttpResponse}; // Disabled for now

// Import modules
mod admin;
mod auth;
mod canister_factory;
mod capsule;
pub mod capsule_store;
mod gallery;
mod memories;
mod memory;
mod person;
mod state;
pub mod types;
pub mod upload;
mod user;

// Import CapsuleStore trait for direct access to store methods
use crate::capsule_store::{types::PaginationOrder as Order, CapsuleStore};
use crate::memory::{with_capsule_store, with_capsule_store_mut};
// memories.rs removed - functionality moved to capsule-based architecture

// ============================================================================
// CORE SYSTEM & UTILITY FUNCTIONS (2 functions)
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
fn register_with_nonce(nonce: String) -> types::Result<()> {
    // Delegate to user module (frontend adapter)
    user::register_user_with_nonce(nonce)
}

#[ic_cdk::query]
fn verify_nonce(nonce: String) -> types::Result<Principal> {
    // Verify and return the principal who proved this nonce
    match auth::get_nonce_proof(nonce) {
        Some(principal) => Ok(principal),
        None => Err(types::Error::NotFound),
    }
}

// ============================================================================
// ADMINISTRATIVE FUNCTIONS (4 functions)
// ============================================================================
#[ic_cdk::update]
fn add_admin(principal: Principal) -> types::Result<()> {
    admin::add_admin(principal)
}

#[ic_cdk::update]
fn remove_admin(principal: Principal) -> types::Result<()> {
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
fn capsules_bind_neon(
    resource_type: types::ResourceType,
    resource_id: String,
    bind: bool,
) -> types::Result<()> {
    // Delegate to capsule module (thin facade)
    capsule::capsules_bind_neon(resource_type, resource_id, bind)
}

// Capsule management endpoints
#[ic_cdk::update]
fn capsules_create(subject: Option<types::PersonRef>) -> types::Result<types::Capsule> {
    // Delegate to capsule module (thin facade)
    capsule::capsules_create(subject)
}

#[ic_cdk::query]
fn capsules_read_full(capsule_id: Option<String>) -> types::Result<types::Capsule> {
    // Delegate to capsule module (thin facade)
    match capsule_id {
        Some(id) => capsule::capsules_read(id),
        None => capsule::capsule_read_self(),
    }
}

#[ic_cdk::query]
fn capsules_read_basic(capsule_id: Option<String>) -> types::Result<types::CapsuleInfo> {
    // Delegate to capsule module (thin facade)
    match capsule_id {
        Some(id) => capsule::capsules_read_basic(id),
        None => capsule::capsule_read_self_basic(),
    }
}

#[ic_cdk::query]
fn capsules_list() -> Vec<types::CapsuleHeader> {
    // Delegate to capsule module (thin facade)
    capsule::capsules_list()
}

#[ic_cdk::update]
fn capsules_update(
    capsule_id: String,
    updates: types::CapsuleUpdateData,
) -> types::Result<types::Capsule> {
    // Delegate to capsule module (thin facade)
    capsule::capsules_update(capsule_id, updates)
}

#[ic_cdk::update]
fn capsules_delete(capsule_id: String) -> types::Result<()> {
    // Delegate to capsule module (thin facade)
    capsule::capsules_delete(capsule_id)
}

// ============================================================================
// GALLERY MANAGEMENT (7 functions)
// ============================================================================
#[ic_cdk::update]
async fn galleries_create(gallery_data: types::GalleryData) -> types::Result<types::Gallery> {
    // TESTING: Using gallery.rs implementation
    gallery::galleries_create(gallery_data)
}

#[ic_cdk::update]
async fn galleries_create_with_memories(
    gallery_data: types::GalleryData,
    sync_memories: bool,
) -> types::Result<types::Gallery> {
    // TESTING: Using gallery.rs implementation
    gallery::galleries_create_with_memories(gallery_data, sync_memories)
}

#[ic_cdk::update]
fn update_gallery_storage_location(
    gallery_id: String,
    new_location: types::GalleryStorageLocation,
) -> types::Result<()> {
    // Delegate to gallery module (thin facade)
    gallery::update_gallery_storage_location(gallery_id, new_location)
}

#[ic_cdk::query]
fn galleries_list() -> Vec<types::GalleryHeader> {
    // Delegate to gallery module (thin facade)
    gallery::galleries_list()
}

#[ic_cdk::query]
fn galleries_read(gallery_id: String) -> types::Result<types::Gallery> {
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
) -> types::Result<types::Gallery> {
    // Delegate to gallery module (thin facade)
    gallery::galleries_update(gallery_id, update_data)
}

#[ic_cdk::update]
async fn galleries_delete(gallery_id: String) -> types::Result<()> {
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
async fn memories_create(
    capsule_id: types::CapsuleId,
    memory_assets: types::MemoryAssetInline,
    idem: String,
) -> types::Result<types::MemoryId> {
    crate::memories::create_inline(capsule_id, memory_assets, idem)
}

#[ic_cdk::query]
fn memories_read(memory_id: String) -> types::Result<types::Memory> {
    // Delegate to memories module (thin facade)
    crate::memories::read(memory_id)
}

#[ic_cdk::update]
async fn memories_update(
    memory_id: String,
    updates: types::MemoryUpdateData,
) -> types::MemoryOperationResponse {
    crate::memories::update(memory_id, updates)
}

#[ic_cdk::update]
async fn memories_delete(memory_id: String) -> types::MemoryOperationResponse {
    crate::memories::delete(memory_id)
}

#[ic_cdk::query]
fn memories_list(capsule_id: String) -> types::MemoryListResponse {
    crate::memories::list(capsule_id)
}

// === Presence ===

/// Check presence for multiple memories on ICP (consolidated from get_memory_presence_icp and get_memory_list_presence_icp)
#[ic_cdk::query]
fn memories_ping(memory_ids: Vec<String>) -> types::Result<Vec<types::MemoryPresenceResult>> {
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
    meta: types::MemoryMeta,
    expected_chunks: u32,
    idem: String,
) -> types::Result<upload::types::SessionId> {
    with_capsule_store_mut(|store| {
        let mut upload_service = upload::service::UploadService::new();
        upload_service.begin_upload(store, capsule_id, meta, expected_chunks, idem)
    })
}

/// Upload a chunk for an active session
#[ic_cdk::update]
async fn uploads_put_chunk(session_id: u64, chunk_idx: u32, bytes: Vec<u8>) -> types::Result<()> {
    // Use real UploadService with actual store integration
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
async fn uploads_finish(
    session_id: u64,
    expected_sha256: Vec<u8>,
    total_len: u64,
) -> types::Result<types::MemoryId> {
    // Use real UploadService with actual store integration
    let hash: [u8; 32] = match expected_sha256.clone().try_into() {
        Ok(h) => h,
        Err(_) => {
            return Err(types::Error::InvalidArgument(format!(
                "invalid_hash_length: expected 32 bytes, got {}",
                expected_sha256.len()
            )))
        }
    };

    memory::with_capsule_store_mut(|store| {
        let mut upload_service = upload::service::UploadService::new();
        let session_id = upload::types::SessionId(session_id);
        match upload_service.commit(store, session_id, hash, total_len) {
            Ok(memory_id) => Ok(memory_id),
            Err(err) => Err(err),
        }
    })
}

/// Abort upload session and cleanup
#[ic_cdk::update]
async fn uploads_abort(session_id: u64) -> types::Result<()> {
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
// EMERGENCY RECOVERY ENDPOINTS (Admin Only)
// ============================================================================

/// Emergency function to clear all stable memory data
/// WARNING: This will delete all stored data and should only be used for recovery
#[ic_cdk::update]
fn clear_all_stable_memory() -> types::Result<()> {
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
fn blob_read(locator: String) -> types::Result<Vec<u8>> {
    upload::blob_read(locator)
}

/// Debug endpoint to upload chunk with base64 data (dev only)
#[ic_cdk::update]
async fn debug_put_chunk_b64(session_id: u64, chunk_idx: u32, b64: String) -> types::Result<()> {
    let bytes =
        base64::decode(b64).map_err(|_| types::Error::InvalidArgument("bad base64".into()))?;
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
) -> types::Result<types::MemoryId> {
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
            Ok(memory_id) => Ok(memory_id),
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
) -> types::Result<Option<canister_factory::DetailedCreationStatus>> {
    canister_factory::get_user_creation_status(user)
}

#[ic_cdk::query]
fn get_user_migration_status(
    user: Principal,
) -> types::Result<Option<canister_factory::DetailedCreationStatus>> {
    get_user_creation_status(user)
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn list_all_creation_states(
) -> types::Result<Vec<(Principal, canister_factory::DetailedCreationStatus)>> {
    canister_factory::list_all_creation_states()
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn list_all_migration_states(
) -> types::Result<Vec<(Principal, canister_factory::DetailedCreationStatus)>> {
    list_all_creation_states()
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn get_creation_states_by_status(
    status: canister_factory::CreationStatus,
) -> types::Result<Vec<(Principal, canister_factory::DetailedCreationStatus)>> {
    canister_factory::get_creation_states_by_status(status)
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn get_migration_states_by_status(
    status: canister_factory::CreationStatus,
) -> types::Result<Vec<(Principal, canister_factory::DetailedCreationStatus)>> {
    get_creation_states_by_status(status)
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::update]
fn clear_creation_state(user: Principal) -> types::Result<bool> {
    canister_factory::clear_creation_state(user)
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::update]
fn clear_migration_state(user: Principal) -> types::Result<bool> {
    clear_creation_state(user)
}

// Admin controls for migration (only available with migration feature)
#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::update]
fn set_personal_canister_creation_enabled(enabled: bool) -> types::Result<()> {
    canister_factory::set_personal_canister_creation_enabled(enabled)
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn get_personal_canister_creation_stats(
) -> types::Result<canister_factory::PersonalCanisterCreationStats> {
    canister_factory::get_personal_canister_creation_stats()
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn is_personal_canister_creation_enabled() -> types::Result<bool> {
    canister_factory::is_personal_canister_creation_enabled()
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn is_migration_enabled() -> types::Result<bool> {
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
fn set_migration_enabled(enabled: bool) -> types::Result<()> {
    set_personal_canister_creation_enabled(enabled)
}

#[cfg(feature = "migration")]
#[ic_cdk::query]
fn get_migration_stats() -> types::Result<canister_factory::PersonalCanisterCreationStats> {
    get_personal_canister_creation_stats()
}

// pub use memories::*; // Disabled for now

// HTTP request handler for serving memories (disabled for now)
// #[ic_cdk::query]
// fn http_request(request: HttpRequest) -> HttpResponse<'static> {
//     use crate::memory::{create_memory_response, create_not_found_response, load_memory};
//
//     let mut response = match load_memory(&request.url) {
//         Some((data, content_type)) => create_memory_response(data, content_type),
//         None => create_not_found_response(),
//     };
//
//     add_skip_certification_header(data_certificate().unwrap(), &mut response);
//     response
// }

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

// Export the interface for the smart contract.
ic_cdk::export_candid!();
