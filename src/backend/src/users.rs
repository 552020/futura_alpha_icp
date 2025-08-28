use crate::types::*;
use candid::Principal;
use ic_cdk::api::{msg_caller, time};
use std::collections::HashMap;

// User storage - thread-local for canister state
thread_local! {
    static USERS: std::cell::RefCell<HashMap<Principal, User>> = std::cell::RefCell::new(HashMap::new());
}

/// Register a user (idempotent operation)
/// This is the simplified, independent canister registration
/// No nonce verification - just register the principal
pub fn register_user() -> UserRegistrationResult {
    let principal = msg_caller();
    let now = time();

    USERS.with(|users| {
        let mut users_map = users.borrow_mut();

        match users_map.get_mut(&principal) {
            Some(existing_user) => {
                // User already exists - update last activity
                existing_user.last_activity_at = now;

                UserRegistrationResult {
                    success: true,
                    user: Some(existing_user.clone()),
                    message: "User already registered, updated activity".to_string(),
                }
            }
            None => {
                // Create new user
                let new_user = User {
                    principal,
                    registered_at: now,
                    last_activity_at: now,
                    bound: false, // Not bound to Web2 initially
                };

                users_map.insert(principal, new_user.clone());

                UserRegistrationResult {
                    success: true,
                    user: Some(new_user),
                    message: "User registered successfully".to_string(),
                }
            }
        }
    })
}

/// Mark user as bound to Web2 (optional convenience method)
/// Called after successful Web2 authentication
pub fn mark_user_bound() -> bool {
    let principal = msg_caller();

    USERS.with(|users| {
        let mut users_map = users.borrow_mut();

        match users_map.get_mut(&principal) {
            Some(user) => {
                user.bound = true;
                user.last_activity_at = time();
                true
            }
            None => false, // User must register first
        }
    })
}

/// Get user information
pub fn get_user() -> Option<User> {
    let principal = msg_caller();

    USERS.with(|users| users.borrow().get(&principal).cloned())
}

/// Get user by principal (for admin/debugging)
pub fn get_user_by_principal(principal: Principal) -> Option<User> {
    USERS.with(|users| users.borrow().get(&principal).cloned())
}

/// List all users (for admin/debugging)
pub fn list_all_users() -> Vec<User> {
    USERS.with(|users| users.borrow().values().cloned().collect())
}

/// Get user statistics
pub fn get_user_stats() -> HashMap<String, u64> {
    USERS.with(|users| {
        let users_map = users.borrow();
        let total_users = users_map.len() as u64;
        let bound_users = users_map.values().filter(|u| u.bound).count() as u64;
        let unbound_users = total_users - bound_users;

        let mut stats = HashMap::new();
        stats.insert("total_users".to_string(), total_users);
        stats.insert("bound_users".to_string(), bound_users);
        stats.insert("unbound_users".to_string(), unbound_users);
        stats
    })
}

/// Update user activity (called on any user action)
pub fn update_user_activity() -> bool {
    let principal = msg_caller();

    USERS.with(|users| {
        let mut users_map = users.borrow_mut();

        match users_map.get_mut(&principal) {
            Some(user) => {
                user.last_activity_at = time();
                true
            }
            None => false,
        }
    })
}
