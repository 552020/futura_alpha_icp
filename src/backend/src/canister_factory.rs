// Public facade for the canister factory module
// This module provides a clean interface to the migration functionality

#![allow(dead_code)] // Many functions will be used in upcoming tasks

use crate::types::Error;

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
) -> std::result::Result<Option<DetailedCreationStatus>, Error> {
    Ok(None)
}

// Legacy function for backward compatibility
pub fn get_user_migration_status(
    user: Principal,
) -> std::result::Result<Option<DetailedCreationStatus>, Error> {
    get_user_creation_status(user)
}

pub fn list_all_creation_states(
) -> std::result::Result<Vec<(Principal, DetailedCreationStatus)>, Error> {
    Ok(vec![])
}

// Legacy function for backward compatibility
pub fn list_all_migration_states(
) -> std::result::Result<Vec<(Principal, DetailedCreationStatus)>, Error> {
    list_all_creation_states()
}

pub fn get_creation_states_by_status(
    _status: CreationStatus,
) -> std::result::Result<Vec<(Principal, DetailedCreationStatus)>, Error> {
    Ok(vec![])
}

// Legacy function for backward compatibility
pub fn get_migration_states_by_status(
    status: CreationStatus,
) -> std::result::Result<Vec<(Principal, DetailedCreationStatus)>, Error> {
    get_creation_states_by_status(status)
}

pub fn clear_creation_state(_user: Principal) -> std::result::Result<bool, Error> {
    Ok(false)
}

// Legacy function for backward compatibility
pub fn clear_migration_state(user: Principal) -> std::result::Result<bool, Error> {
    clear_creation_state(user)
}

pub fn set_personal_canister_creation_enabled(_enabled: bool) -> std::result::Result<(), Error> {
    Ok(())
}

pub fn get_personal_canister_creation_stats(
) -> std::result::Result<PersonalCanisterCreationStats, Error> {
    Ok(PersonalCanisterCreationStats::default())
}

pub fn is_personal_canister_creation_enabled() -> std::result::Result<bool, Error> {
    Ok(false)
}

// Legacy function for backward compatibility
pub fn is_migration_enabled() -> std::result::Result<bool, Error> {
    is_personal_canister_creation_enabled()
}
