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
    get_migration_status, get_my_personal_canister_id, get_personal_canister_id, migrate_capsule,
};

// use crate::types as crate_types; // Will be used when implementing actual functions
use candid::Principal;
// Imports will be added as needed when we implement the actual functions

// Export migration state management functions
pub fn export_migration_state_for_upgrade() -> MigrationStateData {
    crate::memory::with_migration_state(|state| state.clone())
}

pub fn import_migration_state_from_upgrade(migration_data: MigrationStateData) {
    crate::memory::with_migration_state_mut(|state| {
        *state = migration_data;
    })
}

// Main migration functions are now implemented in orchestrator module
// These are re-exported above, but we can add convenience functions here if needed

pub fn get_api_version() -> String {
    API_VERSION.to_string()
}

pub fn get_detailed_migration_status() -> Option<DetailedMigrationStatus> {
    None
}

pub fn get_user_migration_status(
    _user: Principal,
) -> Result<Option<DetailedMigrationStatus>, String> {
    Ok(None)
}

pub fn list_all_migration_states() -> Result<Vec<(Principal, DetailedMigrationStatus)>, String> {
    Ok(vec![])
}

pub fn get_migration_states_by_status(
    _status: MigrationStatus,
) -> Result<Vec<(Principal, DetailedMigrationStatus)>, String> {
    Ok(vec![])
}

pub fn clear_migration_state(_user: Principal) -> Result<bool, String> {
    Ok(false)
}

pub fn set_migration_enabled(_enabled: bool) -> Result<(), String> {
    Ok(())
}

pub fn get_migration_stats() -> Result<MigrationStats, String> {
    Ok(MigrationStats::default())
}

pub fn is_migration_enabled() -> bool {
    false
}
