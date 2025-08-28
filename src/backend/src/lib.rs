// use crate::types::HttpRequest; // Disabled for now
// use ic_cdk::api::data_certificate; // Disabled for now
use candid::Principal;
use ic_cdk::{api::certified_data_set, *};
use ic_http_certification::utils::skip_certification_certified_data;
// use ic_http_certification::{utils::add_skip_certification_header, HttpResponse}; // Disabled for now

// Import modules
mod admin;
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
    ic_cdk::storage::stable_save((capsule_data, admin_data))
        .expect("Failed to save data to stable storage");
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    // Restore capsules and admins after upgrade
    if let Ok((capsule_data, admin_data)) =
        ic_cdk::storage::stable_restore::<(Vec<(String, types::Capsule)>, Vec<Principal>)>()
    {
        capsule::import_capsules_from_upgrade(capsule_data);
        admin::import_admins_from_upgrade(admin_data);
    }
    // If restore fails, start with empty state (no panic)
}

// Export the interface for the smart contract.
ic_cdk::export_candid!();
