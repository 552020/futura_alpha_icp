use super::test_utils::*;
use crate::canister_factory::types::*;

// Mock registry functions
fn mock_create_registry_entry(
    canister_id: candid::Principal,
    created_by: candid::Principal,
    status: CreationStatus,
    cycles_consumed: u128,
) -> Result<(), String> {
    let now = 1234567890000; // Mock timestamp

    let record = PersonalCanisterRecord {
        canister_id,
        created_by,
        created_at: now,
        status,
        cycles_consumed,
    };

    with_mock_creation_state_mut(|state| {
        state.personal_canisters.insert(canister_id, record);
    });

    Ok(())
}

fn mock_get_registry_entries_by_user(user: candid::Principal) -> Vec<PersonalCanisterRecord> {
    with_mock_creation_state(|state| {
        state
            .personal_canisters
            .values()
            .filter(|record| record.created_by == user)
            .cloned()
            .collect()
    })
}

fn mock_get_registry_entries_by_status(status: CreationStatus) -> Vec<PersonalCanisterRecord> {
    with_mock_creation_state(|state| {
        state
            .personal_canisters
            .values()
            .filter(|record| record.status == status)
            .cloned()
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_entry_creation() {
        setup_test_state();
        let canister_id = create_test_principal(10);
        let created_by = create_test_principal(1);
        let status = CreationStatus::Creating;
        let cycles_consumed = 0;

        let result =
            mock_create_registry_entry(canister_id, created_by, status.clone(), cycles_consumed);
        assert!(result.is_ok());

        // Verify entry was created
        let entries = mock_get_registry_entries_by_user(created_by);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].canister_id, canister_id);
        assert_eq!(entries[0].status, status);
    }

    #[test]
    fn test_registry_query_by_user() {
        setup_test_state();
        let user1 = create_test_principal(1);
        let user2 = create_test_principal(2);
        let canister1 = create_test_principal(10);
        let canister2 = create_test_principal(11);
        let canister3 = create_test_principal(12);

        // Create entries for different users
        mock_create_registry_entry(
            canister1,
            user1,
            CreationStatus::Completed,
            2_000_000_000_000,
        )
        .unwrap();
        mock_create_registry_entry(canister2, user1, CreationStatus::Failed, 500_000_000_000)
            .unwrap();
        mock_create_registry_entry(canister3, user2, CreationStatus::Creating, 0).unwrap();

        // Query by user1
        let user1_entries = mock_get_registry_entries_by_user(user1);
        assert_eq!(user1_entries.len(), 2);

        // Query by user2
        let user2_entries = mock_get_registry_entries_by_user(user2);
        assert_eq!(user2_entries.len(), 1);

        // Query by nonexistent user
        let nonexistent_user = create_test_principal(99);
        let nonexistent_entries = mock_get_registry_entries_by_user(nonexistent_user);
        assert_eq!(nonexistent_entries.len(), 0);
    }

    #[test]
    fn test_registry_query_by_status() {
        setup_test_state();
        let user1 = create_test_principal(1);
        let user2 = create_test_principal(2);
        let canister1 = create_test_principal(10);
        let canister2 = create_test_principal(11);
        let canister3 = create_test_principal(12);

        // Create entries with different statuses
        mock_create_registry_entry(
            canister1,
            user1,
            CreationStatus::Completed,
            2_000_000_000_000,
        )
        .unwrap();
        mock_create_registry_entry(canister2, user1, CreationStatus::Failed, 500_000_000_000)
            .unwrap();
        mock_create_registry_entry(
            canister3,
            user2,
            CreationStatus::Completed,
            3_000_000_000_000,
        )
        .unwrap();

        // Query by Completed status
        let completed_entries = mock_get_registry_entries_by_status(CreationStatus::Completed);
        assert_eq!(completed_entries.len(), 2);

        // Query by Failed status
        let failed_entries = mock_get_registry_entries_by_status(CreationStatus::Failed);
        assert_eq!(failed_entries.len(), 1);

        // Query by Creating status
        let creating_entries = mock_get_registry_entries_by_status(CreationStatus::Creating);
        assert_eq!(creating_entries.len(), 0);
    }

    #[test]
    fn test_registry_cycles_tracking() {
        setup_test_state();
        let user1 = create_test_principal(1);
        let user2 = create_test_principal(2);
        let canister1 = create_test_principal(10);
        let canister2 = create_test_principal(11);

        // Create entries with different cycles consumed
        mock_create_registry_entry(
            canister1,
            user1,
            CreationStatus::Completed,
            2_000_000_000_000,
        )
        .unwrap();
        mock_create_registry_entry(
            canister2,
            user2,
            CreationStatus::Completed,
            3_000_000_000_000,
        )
        .unwrap();

        // Calculate total cycles from registry
        let all_completed = mock_get_registry_entries_by_status(CreationStatus::Completed);
        let total_registry_cycles: u128 = all_completed
            .iter()
            .map(|record| record.cycles_consumed)
            .sum();

        assert_eq!(total_registry_cycles, 5_000_000_000_000); // 2T + 3T
    }
}
