// use crate::types::HttpRequest; // Disabled for now
// use ic_cdk::api::data_certificate; // Disabled for now
use candid::Principal;
use ic_cdk::{api::certified_data_set, *};
use ic_http_certification::utils::skip_certification_certified_data;
// use ic_http_certification::{utils::add_skip_certification_header, HttpResponse}; // Disabled for now

// Import modules
mod admin;
mod canister_factory;
mod capsule;
mod memory;
mod types;
// mod memories; // Disabled for now

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

#[ic_cdk::update]
pub fn mark_bound() -> bool {
    capsule::mark_bound()
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

#[ic_cdk::query]
pub fn get_user() -> Option<types::CapsuleInfo> {
    // Redirect to capsule info
    capsule::get_capsule_info()
}

#[ic_cdk::query]
pub fn list_users() -> Vec<types::CapsuleHeader> {
    // Redirect to capsule listing
    capsule::list_my_capsules()
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

// Capsule registration (replaces user registration)
#[ic_cdk::update]
pub fn register_capsule() -> types::CapsuleRegistrationResult {
    capsule::register_capsule()
}

// Mark capsule as bound to Web2 (replaces mark_bound)
#[ic_cdk::update]
pub fn mark_capsule_bound_to_web2() -> bool {
    capsule::mark_capsule_bound_to_web2()
}

// Capsule management endpoints
#[ic_cdk::update]
pub fn create_capsule(subject: types::PersonRef) -> types::CapsuleCreationResult {
    capsule::create_capsule(subject)
}

#[ic_cdk::query]
pub fn get_capsule(capsule_id: String) -> Option<types::Capsule> {
    capsule::get_capsule(capsule_id)
}

#[ic_cdk::query]
pub fn list_my_capsules() -> Vec<types::CapsuleHeader> {
    capsule::list_my_capsules()
}

// Gallery storage endpoints
#[ic_cdk::update]
pub async fn store_gallery_forever(
    gallery_data: types::GalleryData,
) -> types::StoreGalleryResponse {
    capsule::store_gallery_forever(gallery_data)
}

#[ic_cdk::query]
pub fn get_user_galleries(user_principal: Principal) -> Vec<types::Gallery> {
    capsule::get_user_galleries(user_principal)
}

#[ic_cdk::query]
pub fn get_my_galleries() -> Vec<types::Gallery> {
    let caller = ic_cdk::api::msg_caller();
    capsule::get_user_galleries(caller)
}

#[ic_cdk::query]
pub fn get_gallery_by_id(gallery_id: String) -> Option<types::Gallery> {
    capsule::get_gallery_by_id(gallery_id)
}

#[ic_cdk::update]
pub async fn update_gallery(
    gallery_id: String,
    update_data: types::GalleryUpdateData,
) -> types::UpdateGalleryResponse {
    capsule::update_gallery(gallery_id, update_data)
}

#[ic_cdk::update]
pub async fn delete_gallery(gallery_id: String) -> types::DeleteGalleryResponse {
    capsule::delete_gallery(gallery_id)
}

// Memory management endpoints
#[ic_cdk::update]
pub async fn add_memory_to_capsule(
    memory_data: types::MemoryData,
) -> types::MemoryOperationResponse {
    capsule::add_memory_to_capsule(memory_data)
}

#[ic_cdk::query]
pub fn get_memory_from_capsule(memory_id: String) -> Option<types::Memory> {
    capsule::get_memory_from_capsule(memory_id)
}

#[ic_cdk::update]
pub async fn update_memory_in_capsule(
    memory_id: String,
    updates: types::MemoryUpdateData,
) -> types::MemoryOperationResponse {
    capsule::update_memory_in_capsule(memory_id, updates)
}

#[ic_cdk::update]
pub async fn delete_memory_from_capsule(memory_id: String) -> types::MemoryOperationResponse {
    capsule::delete_memory_from_capsule(memory_id)
}

#[ic_cdk::query]
pub fn list_capsule_memories() -> types::MemoryListResponse {
    capsule::list_capsule_memories()
}

// Note: User principal management is handled through capsule registration
// The existing register() and mark_bound() functions handle user principal management

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
pub fn get_api_version() -> String {
    canister_factory::get_api_version()
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

#[init]
fn init() {
    certified_data_set(&skip_certification_certified_data());
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
    // Serialize capsules and admins before upgrade
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
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
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
}

// Export the interface for the smart contract.
ic_cdk::export_candid!();
