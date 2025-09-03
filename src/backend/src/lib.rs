// use crate::types::HttpRequest; // Disabled for now
// use ic_cdk::api::data_certificate; // Disabled for now
use candid::Principal;
use ic_cdk::{api::certified_data_set, *};

// use ic_http_certification::{utils::add_skip_certification_header, HttpResponse}; // Disabled for now

// Import modules
mod admin;
mod auth;
mod canister_factory;
mod capsule;
mod memory;
mod metadata;
mod types;
mod upload;
// memories.rs removed - functionality moved to capsule-based architecture

#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[ic_cdk::query]
pub fn whoami() -> Principal {
    ic_cdk::api::msg_caller()
}

// II integration endpoints (simple registration for II auth flow)
#[ic_cdk::update]
pub fn register() -> bool {
    capsule::register()
}

// Register user and prove nonce in one call (optimized for II auth flow)
#[ic_cdk::update]
pub fn register_with_nonce(nonce: String) -> bool {
    let caller = ic_cdk::api::msg_caller();
    let timestamp = ic_cdk::api::time();

    // Register the user
    capsule::register();

    // Store nonce proof
    memory::store_nonce_proof(nonce, caller, timestamp);

    true
}

// Nonce proof for II authentication (kept for backward compatibility)
#[ic_cdk::update]
pub fn prove_nonce(nonce: String) -> bool {
    // Store the nonce proof with the caller's principal and timestamp
    let caller = ic_cdk::api::msg_caller();
    let timestamp = ic_cdk::api::time();

    // Store in memory (we'll implement this in memory.rs)
    memory::store_nonce_proof(nonce, caller, timestamp)
}

#[ic_cdk::query]
pub fn verify_nonce(nonce: String) -> Option<Principal> {
    // Verify and return the principal who proved this nonce
    memory::get_nonce_proof(nonce)
}

// Admin management endpoints
#[ic_cdk::update]
pub fn add_admin(principal: Principal) -> bool {
    admin::add_admin(principal)
}

#[ic_cdk::update]
pub fn remove_admin(principal: Principal) -> bool {
    admin::remove_admin(principal)
}

#[ic_cdk::query]
pub fn list_admins() -> Vec<Principal> {
    admin::list_admins()
}

#[ic_cdk::query]
pub fn list_superadmins() -> Vec<Principal> {
    admin::list_superadmins()
}

// Flexible resource binding function for Neon database
#[ic_cdk::update]
pub fn capsules_bind_neon(
    resource_type: types::ResourceType,
    resource_id: String,
    bind: bool,
) -> bool {
    capsule::capsules_bind_neon(resource_type, resource_id, bind)
}

// Capsule management endpoints
#[ic_cdk::update]
pub fn capsules_create(subject: Option<types::PersonRef>) -> types::CapsuleCreationResult {
    capsule::capsules_create(subject)
}

#[ic_cdk::query]
pub fn capsules_read_full(capsule_id: Option<String>) -> Option<types::Capsule> {
    match capsule_id {
        Some(id) => capsule::capsules_read(id),
        None => capsule::capsule_read_self(),
    }
}

#[ic_cdk::query]
pub fn capsules_read_basic(capsule_id: Option<String>) -> Option<types::CapsuleInfo> {
    match capsule_id {
        Some(id) => capsule::capsules_read_basic(id),
        None => capsule::capsule_read_self_basic(),
    }
}

#[ic_cdk::query]
pub fn capsules_list() -> Vec<types::CapsuleHeader> {
    capsule::capsules_list()
}

// Gallery storage endpoints
#[ic_cdk::update]
pub async fn galleries_create(gallery_data: types::GalleryData) -> types::StoreGalleryResponse {
    capsule::galleries_create(gallery_data)
}

#[ic_cdk::update]
pub async fn galleries_create_with_memories(
    gallery_data: types::GalleryData,
    sync_memories: bool,
) -> types::StoreGalleryResponse {
    capsule::galleries_create_with_memories(gallery_data, sync_memories)
}

#[ic_cdk::update]
pub fn update_gallery_storage_status(
    gallery_id: String,
    new_status: types::GalleryStorageStatus,
) -> bool {
    capsule::update_gallery_storage_status(gallery_id, new_status)
}

#[ic_cdk::query]
pub fn galleries_list() -> Vec<types::Gallery> {
    capsule::galleries_list()
}

#[ic_cdk::query]
pub fn galleries_read(gallery_id: String) -> Option<types::Gallery> {
    capsule::galleries_read(gallery_id)
}

#[ic_cdk::update]
pub async fn galleries_update(
    gallery_id: String,
    update_data: types::GalleryUpdateData,
) -> types::UpdateGalleryResponse {
    capsule::galleries_update(gallery_id, update_data)
}

#[ic_cdk::update]
pub async fn galleries_delete(gallery_id: String) -> types::DeleteGalleryResponse {
    capsule::galleries_delete(gallery_id)
}

// Memory management endpoints
#[ic_cdk::update]
pub async fn memories_create(
    capsule_id: String,
    memory_data: types::MemoryData,
) -> types::MemoryOperationResponse {
    capsule::memories_create(capsule_id, memory_data)
}

#[ic_cdk::query]
pub fn memories_read(memory_id: String) -> Option<types::Memory> {
    capsule::memories_read(memory_id)
}

#[ic_cdk::update]
pub async fn memories_update(
    memory_id: String,
    updates: types::MemoryUpdateData,
) -> types::MemoryOperationResponse {
    capsule::memories_update(memory_id, updates)
}

#[ic_cdk::update]
pub async fn memories_delete(memory_id: String) -> types::MemoryOperationResponse {
    capsule::memories_delete(memory_id)
}

#[ic_cdk::query]
pub fn memories_list(capsule_id: String) -> types::MemoryListResponse {
    capsule::memories_list(capsule_id)
}

// Note: User principal management is handled through capsule registration
// The existing register() function handles user principal management

// Migration endpoints (only available with migration feature)
#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::update]
pub async fn create_personal_canister() -> canister_factory::PersonalCanisterCreationResponse {
    match canister_factory::create_personal_canister().await {
        Ok(response) => response,
        Err(error) => canister_factory::PersonalCanisterCreationResponse {
            success: false,
            canister_id: None,
            message: format!("Personal canister creation failed: {}", error),
        },
    }
}

#[ic_cdk::query]
pub fn get_creation_status() -> Option<canister_factory::CreationStatusResponse> {
    canister_factory::get_creation_status()
}

#[ic_cdk::query]
pub fn get_personal_canister_id(user: Principal) -> Option<Principal> {
    canister_factory::get_personal_canister_id(user)
}

#[ic_cdk::query]
pub fn get_my_personal_canister_id() -> Option<Principal> {
    canister_factory::get_my_personal_canister_id()
}

#[ic_cdk::query]
pub fn get_detailed_creation_status() -> Option<canister_factory::DetailedCreationStatus> {
    canister_factory::get_detailed_creation_status()
}

// Admin personal canister creation functions
#[ic_cdk::query]
pub fn get_user_creation_status(
    user: Principal,
) -> Result<Option<canister_factory::DetailedCreationStatus>, String> {
    canister_factory::get_user_creation_status(user)
}

#[ic_cdk::query]
pub fn get_user_migration_status(
    user: Principal,
) -> Result<Option<canister_factory::DetailedCreationStatus>, String> {
    get_user_creation_status(user)
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
pub fn list_all_creation_states(
) -> Result<Vec<(Principal, canister_factory::DetailedCreationStatus)>, String> {
    canister_factory::list_all_creation_states()
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
pub fn list_all_migration_states(
) -> Result<Vec<(Principal, canister_factory::DetailedCreationStatus)>, String> {
    list_all_creation_states()
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
pub fn get_creation_states_by_status(
    status: canister_factory::CreationStatus,
) -> Result<Vec<(Principal, canister_factory::DetailedCreationStatus)>, String> {
    canister_factory::get_creation_states_by_status(status)
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
pub fn get_migration_states_by_status(
    status: canister_factory::CreationStatus,
) -> Result<Vec<(Principal, canister_factory::DetailedCreationStatus)>, String> {
    get_creation_states_by_status(status)
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::update]
pub fn clear_creation_state(user: Principal) -> Result<bool, String> {
    canister_factory::clear_creation_state(user)
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::update]
pub fn clear_migration_state(user: Principal) -> Result<bool, String> {
    clear_creation_state(user)
}

// Admin controls for migration (only available with migration feature)
#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::update]
pub fn set_personal_canister_creation_enabled(enabled: bool) -> Result<(), String> {
    canister_factory::set_personal_canister_creation_enabled(enabled)
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
pub fn get_personal_canister_creation_stats(
) -> Result<canister_factory::PersonalCanisterCreationStats, String> {
    canister_factory::get_personal_canister_creation_stats()
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
pub fn is_personal_canister_creation_enabled() -> bool {
    canister_factory::is_personal_canister_creation_enabled()
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
pub fn is_migration_enabled() -> bool {
    is_personal_canister_creation_enabled()
}

// Legacy function names for backward compatibility
#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::update]
pub async fn migrate_capsule() -> canister_factory::PersonalCanisterCreationResponse {
    create_personal_canister().await
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
pub fn get_migration_status() -> Option<canister_factory::CreationStatusResponse> {
    get_creation_status()
}

#[cfg(feature = "migration")]
#[ic_cdk::query]
pub fn get_detailed_migration_status() -> Option<canister_factory::DetailedCreationStatus> {
    get_detailed_creation_status()
}

#[cfg(feature = "migration")]
#[ic_cdk::update]
pub fn set_migration_enabled(enabled: bool) -> Result<(), String> {
    set_personal_canister_creation_enabled(enabled)
}

#[cfg(feature = "migration")]
#[ic_cdk::query]
pub fn get_migration_stats() -> Result<canister_factory::PersonalCanisterCreationStats, String> {
    get_personal_canister_creation_stats()
}

// pub use memories::*; // Disabled for now

// HTTP request handler for serving memories (disabled for now)
// #[ic_cdk::query]
// pub fn http_request(request: HttpRequest) -> HttpResponse<'static> {
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
    let capsule_data = capsule::export_capsules_for_upgrade();
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
            capsule::import_capsules_from_upgrade(capsule_data);
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
            capsule::import_capsules_from_upgrade(capsule_data);
            admin::import_admins_from_upgrade(admin_data);
        }
    }
    // If restore fails, start with empty state (no panic)

    ic_cdk::println!("Post-upgrade: stable memory structures restored automatically");
}

// ============================================================================
// MEMORY METADATA OPERATIONS - ICP Canister Endpoints
// ============================================================================

/// Store memory metadata on ICP with idempotency support
#[ic_cdk::update]
pub fn upsert_metadata(
    memory_id: String,
    memory_type: types::MemoryType,
    metadata: types::SimpleMemoryMetadata,
    idempotency_key: String,
) -> types::ICPResult<types::MetadataResponse> {
    metadata::upsert_metadata(memory_id, memory_type, metadata, idempotency_key)
}

/// Check presence for multiple memories on ICP (consolidated from get_memory_presence_icp and get_memory_list_presence_icp)
#[ic_cdk::query]
pub fn memories_ping(
    memory_ids: Vec<String>,
) -> types::ICPResult<Vec<types::MemoryPresenceResult>> {
    metadata::memories_ping(memory_ids)
}

// ============================================================================
// CHUNKED ASSET UPLOAD ENDPOINTS - ICP Canister API
// ============================================================================

/// Begin chunked upload session for large files
#[ic_cdk::update]
pub fn begin_asset_upload(
    memory_id: String,
    memory_type: types::MemoryType,
    expected_hash: String,
    chunk_count: u32,
    total_size: u64,
) -> types::ICPResult<types::UploadSessionResponse> {
    upload::begin_asset_upload(
        memory_id,
        memory_type,
        expected_hash,
        chunk_count,
        total_size,
    )
}

/// Upload individual file chunk
#[ic_cdk::update]
pub fn put_chunk(
    session_id: String,
    chunk_index: u32,
    chunk_data: Vec<u8>,
) -> types::ICPResult<types::ChunkResponse> {
    upload::put_chunk(session_id, chunk_index, chunk_data)
}

/// Finalize upload after all chunks received
#[ic_cdk::update]
pub fn commit_asset(
    session_id: String,
    final_hash: String,
) -> types::ICPResult<types::CommitResponse> {
    upload::commit_asset(session_id, final_hash)
}

/// Cancel upload and cleanup resources
#[ic_cdk::update]
pub fn cancel_upload(session_id: String) -> types::ICPResult<()> {
    upload::cancel_upload(session_id)
}

/// Batch sync multiple memories for a gallery to ICP
#[ic_cdk::update]
pub async fn sync_gallery_memories(
    gallery_id: String,
    memory_sync_requests: Vec<types::MemorySyncRequest>,
) -> types::ICPResult<types::BatchMemorySyncResponse> {
    upload::sync_gallery_memories(gallery_id, memory_sync_requests).await
}

/// Clean up expired upload sessions (admin function)
#[ic_cdk::update]
pub fn cleanup_expired_sessions() -> u32 {
    upload::cleanup_expired_sessions()
}

/// Clean up orphaned chunks (admin function)
#[ic_cdk::update]
pub fn cleanup_orphaned_chunks() -> u32 {
    upload::cleanup_orphaned_chunks()
}

/// Get upload session statistics for monitoring
#[ic_cdk::query]
pub fn get_upload_session_stats() -> (u32, u32, u64) {
    upload::get_upload_session_stats()
}

// Export the interface for the smart contract.
ic_cdk::export_candid!();
