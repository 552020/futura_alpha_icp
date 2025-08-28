// use crate::types::HttpRequest; // Disabled for now
// use ic_cdk::api::data_certificate; // Disabled for now
use candid::Principal;
use ic_cdk::{api::certified_data_set, *};
use ic_http_certification::utils::skip_certification_certified_data;
// use ic_http_certification::{utils::add_skip_certification_header, HttpResponse}; // Disabled for now

// Import modules
mod types;
mod users;
// mod memories; // Disabled for now
// mod memory; // Disabled for now - conflicts with current types

#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[ic_cdk::query]
pub fn whoami() -> Principal {
    ic_cdk::api::msg_caller()
}

// User management endpoints for Internet Identity integration
#[ic_cdk::update]
pub fn register() -> types::UserRegistrationResult {
    users::register_user()
}

#[ic_cdk::update]
pub fn mark_bound() -> bool {
    users::mark_user_bound()
}

#[ic_cdk::query]
pub fn get_user() -> Option<types::User> {
    users::get_user()
}

#[ic_cdk::query]
pub fn get_user_by_principal(principal: Principal) -> Option<types::User> {
    users::get_user_by_principal(principal)
}

#[ic_cdk::query]
pub fn list_users() -> Vec<types::User> {
    users::list_all_users()
}

#[ic_cdk::query]
pub fn user_stats() -> std::collections::HashMap<String, u64> {
    users::get_user_stats()
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
    // Serialize user data before upgrade
    let user_data = users::export_users_for_upgrade();
    ic_cdk::storage::stable_save((user_data,)).expect("Failed to save user data to stable storage");
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    // Restore user data after upgrade
    if let Ok((user_data,)) = ic_cdk::storage::stable_restore::<(Vec<(Principal, types::User)>,)>()
    {
        users::import_users_from_upgrade(user_data);
    }
    // If restore fails, start with empty state (no panic)
}

// Export the interface for the smart contract.
ic_cdk::export_candid!();
