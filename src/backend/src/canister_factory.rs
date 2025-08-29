use crate::types;
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Response from migration operations
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct MigrationResponse {
    pub success: bool,
    pub canister_id: Option<Principal>,
    pub message: String,
}

/// Migration status enum tracking the progression through migration states
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum MigrationStatus {
    NotStarted,
    Exporting,
    Creating,
    Installing,
    Importing,
    Verifying,
    Completed,
    Failed,
}

/// Response for migration status queries
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct MigrationStatusResponse {
    pub status: MigrationStatus,
    pub canister_id: Option<Principal>,
    pub message: Option<String>,
}

/// Exported capsule data for migration
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ExportData {
    pub capsule: types::Capsule,
    pub memories: Vec<(String, types::Memory)>,
    pub connections: Vec<(types::PersonRef, types::Connection)>,
    pub metadata: ExportMetadata,
}

/// Metadata about the exported data
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ExportMetadata {
    pub export_timestamp: u64,
    pub original_canister_id: Principal,
    pub data_version: String,
    pub total_size_bytes: u64,
}

/// Migration state for tracking individual user migrations
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct MigrationState {
    pub user: Principal,
    pub status: MigrationStatus,
    pub created_at: u64,
    pub completed_at: Option<u64>,
    pub personal_canister_id: Option<Principal>,
    pub cycles_consumed: u128,
    pub error_message: Option<String>,
}

/// Configuration for migration system
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct MigrationConfig {
    pub enabled: bool,
    pub cycles_reserve: u128,
    pub min_cycles_threshold: u128,
    pub admin_principals: BTreeSet<Principal>,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Start disabled for safety
            cycles_reserve: 0,
            min_cycles_threshold: 1_000_000_000_000, // 1T cycles default threshold
            admin_principals: BTreeSet::new(),
        }
    }
}

/// Registry record for personal canisters
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct PersonalCanisterRecord {
    pub canister_id: Principal,
    pub created_by: Principal,
    pub created_at: u64,
    pub status: MigrationStatus,
    pub cycles_consumed: u128,
}

/// Statistics for migration operations
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Default)]
pub struct MigrationStats {
    pub total_attempts: u64,
    pub total_successes: u64,
    pub total_failures: u64,
    pub total_cycles_consumed: u128,
}

/// Extended state structure with migration fields
#[derive(CandidType, Serialize, Deserialize, Default, Clone, Debug)]
pub struct MigrationStateData {
    pub migration_config: MigrationConfig,
    pub migration_states: BTreeMap<Principal, MigrationState>,
    pub migration_stats: MigrationStats,
    pub personal_canisters: BTreeMap<Principal, PersonalCanisterRecord>,
}

/// Export migration state for upgrade persistence
pub fn export_migration_state_for_upgrade() -> MigrationStateData {
    crate::memory::with_migration_state(|state| state.clone())
}

/// Import migration state from upgrade persistence
pub fn import_migration_state_from_upgrade(migration_data: MigrationStateData) {
    crate::memory::with_migration_state_mut(|state| {
        *state = migration_data;
    })
}

// Personal canister registry management functions

/// Create a new registry entry for a personal canister
pub fn create_registry_entry(
    canister_id: Principal,
    created_by: Principal,
    status: MigrationStatus,
    cycles_consumed: u128,
) -> Result<(), String> {
    let now = ic_cdk::api::time();

    let record = PersonalCanisterRecord {
        canister_id,
        created_by,
        created_at: now,
        status,
        cycles_consumed,
    };

    crate::memory::with_migration_state_mut(|state| {
        state.personal_canisters.insert(canister_id, record);
    });

    Ok(())
}

/// Update the status of a personal canister in the registry
pub fn update_registry_status(
    canister_id: Principal,
    new_status: MigrationStatus,
) -> Result<(), String> {
    crate::memory::with_migration_state_mut(|state| {
        if let Some(record) = state.personal_canisters.get_mut(&canister_id) {
            record.status = new_status;
            Ok(())
        } else {
            Err(format!(
                "Registry entry not found for canister {}",
                canister_id
            ))
        }
    })
}

/// Update cycles consumed for a personal canister in the registry
pub fn update_registry_cycles_consumed(
    canister_id: Principal,
    cycles_consumed: u128,
) -> Result<(), String> {
    crate::memory::with_migration_state_mut(|state| {
        if let Some(record) = state.personal_canisters.get_mut(&canister_id) {
            record.cycles_consumed = cycles_consumed;
            Ok(())
        } else {
            Err(format!(
                "Registry entry not found for canister {}",
                canister_id
            ))
        }
    })
}

/// Get registry entries by user principal (admin function)
pub fn get_registry_entries_by_user(user: Principal) -> Vec<PersonalCanisterRecord> {
    crate::memory::with_migration_state(|state| {
        state
            .personal_canisters
            .values()
            .filter(|record| record.created_by == user)
            .cloned()
            .collect()
    })
}

/// Get registry entries by status (admin function)
pub fn get_registry_entries_by_status(status: MigrationStatus) -> Vec<PersonalCanisterRecord> {
    crate::memory::with_migration_state(|state| {
        state
            .personal_canisters
            .values()
            .filter(|record| record.status == status)
            .cloned()
            .collect()
    })
}

/// Get all registry entries (admin function)
pub fn get_all_registry_entries() -> Vec<PersonalCanisterRecord> {
    crate::memory::with_migration_state(|state| {
        state.personal_canisters.values().cloned().collect()
    })
}

/// Get a specific registry entry by canister ID
pub fn get_registry_entry(canister_id: Principal) -> Option<PersonalCanisterRecord> {
    crate::memory::with_migration_state(|state| state.personal_canisters.get(&canister_id).cloned())
}

/// Remove a registry entry (admin function for cleanup)
pub fn remove_registry_entry(canister_id: Principal) -> Result<(), String> {
    crate::memory::with_migration_state_mut(|state| {
        if state.personal_canisters.remove(&canister_id).is_some() {
            Ok(())
        } else {
            Err(format!(
                "Registry entry not found for canister {}",
                canister_id
            ))
        }
    })
}
