// Public facade for the canister factory module
// This module provides a clean interface to the migration functionality

#![allow(dead_code)] // Many functions will be used in upcoming tasks

pub mod auth;
pub mod cycles;
pub mod export;
pub mod factory;
pub mod import;
pub mod orchestrator;
pub mod registry;
pub mod types;
pub mod verify;

#[cfg(test)]
pub mod integration_tests;

// Re-export commonly used types
pub use types::*;

// Re-export only the functions that are actually used
pub use orchestrator::{
    create_personal_canister, get_creation_status, get_my_personal_canister_id,
    get_personal_canister_id,
};

// use crate::types as crate_types; // Will be used when implementing actual functions
use candid::Principal;
// Imports will be added as needed when we implement the actual functions

// Export personal canister creation state management functions
pub fn export_creation_state_for_upgrade() -> PersonalCanisterCreationStateData {
    crate::memory::with_migration_state(|state| state.clone())
}

pub fn import_creation_state_from_upgrade(creation_data: PersonalCanisterCreationStateData) {
    crate::memory::with_migration_state_mut(|state| {
        *state = creation_data;
    })
}

// Legacy functions for backward compatibility
pub fn export_migration_state_for_upgrade() -> PersonalCanisterCreationStateData {
    export_creation_state_for_upgrade()
}

pub fn import_migration_state_from_upgrade(migration_data: PersonalCanisterCreationStateData) {
    import_creation_state_from_upgrade(migration_data)
}

// Main migration functions are now implemented in orchestrator module
// These are re-exported above, but we can add convenience functions here if needed

pub fn get_api_version() -> String {
    API_VERSION.to_string()
}

pub fn get_detailed_creation_status() -> Option<DetailedCreationStatus> {
    None
}

pub fn get_user_creation_status(
    _user: Principal,
) -> Result<Option<DetailedCreationStatus>, String> {
    Ok(None)
}

// Legacy function for backward compatibility
pub fn get_user_migration_status(
    user: Principal,
) -> Result<Option<DetailedCreationStatus>, String> {
    get_user_creation_status(user)
}

pub fn list_all_creation_states() -> Result<Vec<(Principal, DetailedCreationStatus)>, String> {
    Ok(vec![])
}

// Legacy function for backward compatibility
pub fn list_all_migration_states() -> Result<Vec<(Principal, DetailedCreationStatus)>, String> {
    list_all_creation_states()
}

pub fn get_creation_states_by_status(
    _status: CreationStatus,
) -> Result<Vec<(Principal, DetailedCreationStatus)>, String> {
    Ok(vec![])
}

// Legacy function for backward compatibility
pub fn get_migration_states_by_status(
    status: CreationStatus,
) -> Result<Vec<(Principal, DetailedCreationStatus)>, String> {
    get_creation_states_by_status(status)
}

pub fn clear_creation_state(_user: Principal) -> Result<bool, String> {
    Ok(false)
}

// Legacy function for backward compatibility
pub fn clear_migration_state(user: Principal) -> Result<bool, String> {
    clear_creation_state(user)
}

pub fn set_personal_canister_creation_enabled(_enabled: bool) -> Result<(), String> {
    Ok(())
}

pub fn get_personal_canister_creation_stats() -> Result<PersonalCanisterCreationStats, String> {
    Ok(PersonalCanisterCreationStats::default())
}

pub fn is_personal_canister_creation_enabled() -> bool {
    false
}

// Legacy function for backward compatibility
pub fn is_migration_enabled() -> bool {
    is_personal_canister_creation_enabled()
}
