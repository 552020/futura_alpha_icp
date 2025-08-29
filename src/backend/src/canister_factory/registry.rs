use crate::canister_factory::types::*;
use candid::Principal;

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

/// Finalize registry after successful migration
/// This function updates the registry with final status and cycles consumed
pub fn finalize_registry_after_migration(
    canister_id: Principal,
    cycles_consumed: u128,
) -> Result<(), String> {
    crate::memory::with_migration_state_mut(|state| {
        if let Some(record) = state.personal_canisters.get_mut(&canister_id) {
            record.status = MigrationStatus::Completed;
            record.cycles_consumed = cycles_consumed;

            ic_cdk::println!(
                "Finalized registry for canister {}: status=Completed, cycles_consumed={}",
                canister_id,
                cycles_consumed
            );

            Ok(())
        } else {
            Err(format!(
                "Registry entry not found for canister {}",
                canister_id
            ))
        }
    })
}
