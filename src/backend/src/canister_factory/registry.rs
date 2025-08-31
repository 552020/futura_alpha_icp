use crate::canister_factory::types::*;
use candid::Principal;

/// Get current time - can be mocked in tests
#[cfg(not(test))]
fn get_current_time() -> u64 {
    ic_cdk::api::time()
}

#[cfg(test)]
fn get_current_time() -> u64 {
    1000000000 // Fixed timestamp for tests
}

/// Create a new registry entry for a personal canister
pub fn create_registry_entry(
    canister_id: Principal,
    created_by: Principal,
    status: CreationStatus,
    cycles_consumed: u128,
) -> Result<(), String> {
    let now = get_current_time();

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
    new_status: CreationStatus,
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
pub fn get_registry_entries_by_status(status: CreationStatus) -> Vec<PersonalCanisterRecord> {
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

/// Finalize registry after successful personal canister creation
/// This function updates the registry with final status and cycles consumed
pub fn finalize_registry_after_creation(
    canister_id: Principal,
    cycles_consumed: u128,
) -> Result<(), String> {
    crate::memory::with_migration_state_mut(|state| {
        if let Some(record) = state.personal_canisters.get_mut(&canister_id) {
            record.status = CreationStatus::Completed;
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

/// Legacy function for backward compatibility
pub fn finalize_registry_after_migration(
    canister_id: Principal,
    cycles_consumed: u128,
) -> Result<(), String> {
    finalize_registry_after_creation(canister_id, cycles_consumed)
}

#[cfg(test)]
mod tests {

    use crate::canister_factory::types::{CreationStatus, PersonalCanisterRecord};
    use candid::Principal;
    use std::collections::BTreeMap;

    // Mock data for testing
    fn create_test_principal(id: u8) -> Principal {
        let mut bytes = [0u8; 29];
        bytes[0] = id;
        Principal::from_slice(&bytes)
    }

    fn create_test_record(
        canister_id: Principal,
        created_by: Principal,
        status: CreationStatus,
        cycles_consumed: u128,
    ) -> PersonalCanisterRecord {
        PersonalCanisterRecord {
            canister_id,
            created_by,
            created_at: 1234567890000, // Mock timestamp
            status,
            cycles_consumed,
        }
    }

    fn setup_test_registry() -> BTreeMap<Principal, PersonalCanisterRecord> {
        let mut registry = BTreeMap::new();

        let user1 = create_test_principal(1);
        let user2 = create_test_principal(2);
        let user3 = create_test_principal(3);

        let canister1 = create_test_principal(10);
        let canister2 = create_test_principal(11);
        let canister3 = create_test_principal(12);
        let canister4 = create_test_principal(13);
        let canister5 = create_test_principal(14);

        // User1 has 2 canisters - one completed, one failed
        registry.insert(
            canister1,
            create_test_record(
                canister1,
                user1,
                CreationStatus::Completed,
                2_000_000_000_000,
            ),
        );
        registry.insert(
            canister2,
            create_test_record(canister2, user1, CreationStatus::Failed, 500_000_000_000),
        );

        // User2 has 2 canisters - one creating, one installing
        registry.insert(
            canister3,
            create_test_record(canister3, user2, CreationStatus::Creating, 0),
        );
        registry.insert(
            canister4,
            create_test_record(
                canister4,
                user2,
                CreationStatus::Installing,
                1_000_000_000_000,
            ),
        );

        // User3 has 1 canister - completed
        registry.insert(
            canister5,
            create_test_record(
                canister5,
                user3,
                CreationStatus::Completed,
                3_000_000_000_000,
            ),
        );

        registry
    }

    #[test]
    fn test_registry_entry_creation() {
        let canister_id = create_test_principal(10);
        let created_by = create_test_principal(1);
        let status = CreationStatus::Creating;
        let cycles_consumed = 0;

        let record = create_test_record(canister_id, created_by, status.clone(), cycles_consumed);

        // Verify record fields
        assert_eq!(record.canister_id, canister_id);
        assert_eq!(record.created_by, created_by);
        assert_eq!(record.status, status);
        assert_eq!(record.cycles_consumed, cycles_consumed);
        assert!(record.created_at > 0);
    }

    #[test]
    fn test_registry_entry_status_update() {
        let mut registry = setup_test_registry();
        let canister_id = create_test_principal(10);
        let new_status = CreationStatus::Installing;

        // Update status
        if let Some(record) = registry.get_mut(&canister_id) {
            record.status = new_status.clone();
        }

        // Verify update
        let updated_record = registry.get(&canister_id).unwrap();
        assert_eq!(updated_record.status, new_status);
    }

    #[test]
    fn test_registry_cycles_consumed_update() {
        let mut registry = setup_test_registry();
        let canister_id = create_test_principal(10);
        let new_cycles_consumed = 5_000_000_000_000;

        // Update cycles consumed
        if let Some(record) = registry.get_mut(&canister_id) {
            record.cycles_consumed = new_cycles_consumed;
        }

        // Verify update
        let updated_record = registry.get(&canister_id).unwrap();
        assert_eq!(updated_record.cycles_consumed, new_cycles_consumed);
    }

    #[test]
    fn test_registry_query_by_user() {
        let registry = setup_test_registry();
        let user1 = create_test_principal(1);
        let user2 = create_test_principal(2);
        let user3 = create_test_principal(3);
        let nonexistent_user = create_test_principal(99);

        // Query by user1 (should have 2 entries)
        let user1_entries: Vec<PersonalCanisterRecord> = registry
            .values()
            .filter(|record| record.created_by == user1)
            .cloned()
            .collect();
        assert_eq!(user1_entries.len(), 2);

        // Query by user2 (should have 2 entries)
        let user2_entries: Vec<PersonalCanisterRecord> = registry
            .values()
            .filter(|record| record.created_by == user2)
            .cloned()
            .collect();
        assert_eq!(user2_entries.len(), 2);

        // Query by user3 (should have 1 entry)
        let user3_entries: Vec<PersonalCanisterRecord> = registry
            .values()
            .filter(|record| record.created_by == user3)
            .cloned()
            .collect();
        assert_eq!(user3_entries.len(), 1);

        // Query by nonexistent user (should have 0 entries)
        let nonexistent_entries: Vec<PersonalCanisterRecord> = registry
            .values()
            .filter(|record| record.created_by == nonexistent_user)
            .cloned()
            .collect();
        assert_eq!(nonexistent_entries.len(), 0);
    }

    #[test]
    fn test_registry_query_by_status() {
        let registry = setup_test_registry();

        // Query by Completed status (should have 2 entries)
        let completed_entries: Vec<PersonalCanisterRecord> = registry
            .values()
            .filter(|record| record.status == MigrationStatus::Completed)
            .cloned()
            .collect();
        assert_eq!(completed_entries.len(), 2);

        // Query by Failed status (should have 1 entry)
        let failed_entries: Vec<PersonalCanisterRecord> = registry
            .values()
            .filter(|record| record.status == MigrationStatus::Failed)
            .cloned()
            .collect();
        assert_eq!(failed_entries.len(), 1);

        // Query by Creating status (should have 1 entry)
        let creating_entries: Vec<PersonalCanisterRecord> = registry
            .values()
            .filter(|record| record.status == MigrationStatus::Creating)
            .cloned()
            .collect();
        assert_eq!(creating_entries.len(), 1);

        // Query by Installing status (should have 1 entry)
        let installing_entries: Vec<PersonalCanisterRecord> = registry
            .values()
            .filter(|record| record.status == MigrationStatus::Installing)
            .cloned()
            .collect();
        assert_eq!(installing_entries.len(), 1);

        // Query by NotStarted status (should have 0 entries)
        let not_started_entries: Vec<PersonalCanisterRecord> = registry
            .values()
            .filter(|record| record.status == MigrationStatus::NotStarted)
            .cloned()
            .collect();
        assert_eq!(not_started_entries.len(), 0);
    }

    #[test]
    fn test_registry_get_specific_entry() {
        let registry = setup_test_registry();
        let canister_id = create_test_principal(10);
        let nonexistent_canister = create_test_principal(99);

        // Get existing entry
        let entry = registry.get(&canister_id);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().canister_id, canister_id);

        // Get nonexistent entry
        let nonexistent_entry = registry.get(&nonexistent_canister);
        assert!(nonexistent_entry.is_none());
    }

    #[test]
    fn test_registry_remove_entry() {
        let mut registry = setup_test_registry();
        let canister_id = create_test_principal(10);
        let initial_count = registry.len();

        // Remove existing entry
        let removed_entry = registry.remove(&canister_id);
        assert!(removed_entry.is_some());
        assert_eq!(registry.len(), initial_count - 1);

        // Try to remove the same entry again
        let removed_again = registry.remove(&canister_id);
        assert!(removed_again.is_none());
        assert_eq!(registry.len(), initial_count - 1);
    }

    #[test]
    fn test_registry_finalization() {
        let mut registry = setup_test_registry();
        let canister_id = create_test_principal(11); // This one has Failed status initially
        let final_cycles_consumed = 2_500_000_000_000;

        // Finalize the registry entry
        if let Some(record) = registry.get_mut(&canister_id) {
            record.status = MigrationStatus::Completed;
            record.cycles_consumed = final_cycles_consumed;
        }

        // Verify finalization
        let finalized_record = registry.get(&canister_id).unwrap();
        assert_eq!(finalized_record.status, MigrationStatus::Completed);
        assert_eq!(finalized_record.cycles_consumed, final_cycles_consumed);
    }

    #[test]
    fn test_registry_all_entries() {
        let registry = setup_test_registry();

        let all_entries: Vec<PersonalCanisterRecord> = registry.values().cloned().collect();
        assert_eq!(all_entries.len(), 5); // Based on setup_test_registry

        // Verify all entries have valid data
        for entry in all_entries {
            assert_ne!(entry.canister_id, Principal::anonymous());
            assert_ne!(entry.created_by, Principal::anonymous());
            assert!(entry.created_at > 0);
            // cycles_consumed can be 0 for entries in Creating status
        }
    }

    #[test]
    fn test_registry_status_transitions() {
        let mut registry = BTreeMap::new();
        let canister_id = create_test_principal(10);
        let created_by = create_test_principal(1);

        // Start with Creating status
        let record = create_test_record(canister_id, created_by, MigrationStatus::Creating, 0);
        registry.insert(canister_id, record);

        // Transition through states
        let status_transitions = vec![
            MigrationStatus::Installing,
            MigrationStatus::Importing,
            MigrationStatus::Verifying,
            MigrationStatus::Completed,
        ];

        for status in status_transitions {
            if let Some(entry) = registry.get_mut(&canister_id) {
                entry.status = status.clone();
            }

            let updated_entry = registry.get(&canister_id).unwrap();
            assert_eq!(updated_entry.status, status);
        }
    }

    #[test]
    fn test_registry_cycles_tracking() {
        let registry = setup_test_registry();

        // Calculate total cycles consumed across all entries
        let total_cycles: u128 = registry.values().map(|record| record.cycles_consumed).sum();

        // Based on setup_test_registry: 2T + 0.5T + 0 + 1T + 3T = 6.5T
        let expected_total = 6_500_000_000_000;
        assert_eq!(total_cycles, expected_total);

        // Test cycles consumed by status
        let completed_cycles: u128 = registry
            .values()
            .filter(|record| record.status == MigrationStatus::Completed)
            .map(|record| record.cycles_consumed)
            .sum();

        // Completed entries: 2T + 3T = 5T
        let expected_completed = 5_000_000_000_000;
        assert_eq!(completed_cycles, expected_completed);
    }

    #[test]
    fn test_registry_user_statistics() {
        let registry = setup_test_registry();
        let user1 = create_test_principal(1);

        let user1_entries: Vec<PersonalCanisterRecord> = registry
            .values()
            .filter(|record| record.created_by == user1)
            .cloned()
            .collect();

        // User1 statistics
        let total_canisters = user1_entries.len();
        let completed_canisters = user1_entries
            .iter()
            .filter(|record| record.status == MigrationStatus::Completed)
            .count();
        let failed_canisters = user1_entries
            .iter()
            .filter(|record| record.status == MigrationStatus::Failed)
            .count();
        let total_cycles_consumed: u128 = user1_entries
            .iter()
            .map(|record| record.cycles_consumed)
            .sum();

        assert_eq!(total_canisters, 2);
        assert_eq!(completed_canisters, 1);
        assert_eq!(failed_canisters, 1);
        assert_eq!(total_cycles_consumed, 2_500_000_000_000); // 2T + 0.5T
    }

    #[test]
    fn test_registry_status_distribution() {
        let registry = setup_test_registry();

        let mut status_counts = std::collections::HashMap::new();
        for record in registry.values() {
            *status_counts.entry(record.status.clone()).or_insert(0) += 1;
        }

        // Based on setup_test_registry
        assert_eq!(
            status_counts.get(&MigrationStatus::Completed).unwrap_or(&0),
            &2
        );
        assert_eq!(
            status_counts.get(&MigrationStatus::Failed).unwrap_or(&0),
            &1
        );
        assert_eq!(
            status_counts.get(&MigrationStatus::Creating).unwrap_or(&0),
            &1
        );
        assert_eq!(
            status_counts
                .get(&MigrationStatus::Installing)
                .unwrap_or(&0),
            &1
        );
        assert_eq!(
            status_counts
                .get(&MigrationStatus::NotStarted)
                .unwrap_or(&0),
            &0
        );
    }

    #[test]
    fn test_registry_timestamp_validation() {
        let registry = setup_test_registry();

        // All entries should have valid timestamps
        for record in registry.values() {
            assert!(record.created_at > 0);
            // In a real system, you might want to check that timestamps are reasonable
            // e.g., not in the future, not too old, etc.
        }
    }

    #[test]
    fn test_registry_principal_validation() {
        let registry = setup_test_registry();

        // All entries should have valid principals
        for record in registry.values() {
            assert_ne!(record.canister_id, Principal::anonymous());
            assert_ne!(record.created_by, Principal::anonymous());

            // Verify principals are different (canister_id != created_by)
            assert_ne!(record.canister_id, record.created_by);
        }
    }

    #[test]
    fn test_registry_concurrent_operations() {
        let mut registry = setup_test_registry();
        let canister_id = create_test_principal(12); // This one has Creating status initially

        // Simulate concurrent status update and cycles update
        if let Some(record) = registry.get_mut(&canister_id) {
            let old_status = record.status.clone();
            let old_cycles = record.cycles_consumed;

            // Update both fields
            record.status = MigrationStatus::Completed;
            record.cycles_consumed = 10_000_000_000_000;

            // Verify both updates took effect
            assert_ne!(record.status, old_status);
            assert_ne!(record.cycles_consumed, old_cycles);
        }
    }

    #[test]
    fn test_registry_edge_cases() {
        let mut registry = BTreeMap::new();

        // Test with maximum cycles value
        let max_cycles_canister = create_test_principal(100);
        let max_cycles_record = create_test_record(
            max_cycles_canister,
            create_test_principal(1),
            MigrationStatus::Completed,
            u128::MAX,
        );
        registry.insert(max_cycles_canister, max_cycles_record);

        // Test with zero cycles
        let zero_cycles_canister = create_test_principal(101);
        let zero_cycles_record = create_test_record(
            zero_cycles_canister,
            create_test_principal(2),
            MigrationStatus::Creating,
            0,
        );
        registry.insert(zero_cycles_canister, zero_cycles_record);

        // Verify both edge cases
        let max_entry = registry.get(&max_cycles_canister).unwrap();
        assert_eq!(max_entry.cycles_consumed, u128::MAX);

        let zero_entry = registry.get(&zero_cycles_canister).unwrap();
        assert_eq!(zero_entry.cycles_consumed, 0);
    }
}
