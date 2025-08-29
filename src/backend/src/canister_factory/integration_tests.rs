#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::canister_factory::{cycles, registry, types::*};
    use candid::Principal;
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    // Mock state for testing
    thread_local! {
        static MOCK_STATE: RefCell<MigrationStateData> = RefCell::new(MigrationStateData::default());
    }

    // Mock implementation of memory functions for testing
    pub fn with_mock_migration_state<R>(f: impl FnOnce(&MigrationStateData) -> R) -> R {
        MOCK_STATE.with(|state| f(&state.borrow()))
    }

    pub fn with_mock_migration_state_mut<R>(f: impl FnOnce(&mut MigrationStateData) -> R) -> R {
        MOCK_STATE.with(|state| f(&mut state.borrow_mut()))
    }

    fn setup_test_state() {
        with_mock_migration_state_mut(|state| {
            *state = MigrationStateData {
                migration_config: MigrationConfig {
                    enabled: true,
                    cycles_reserve: 10_000_000_000_000, // 10T cycles
                    min_cycles_threshold: 2_000_000_000_000, // 2T cycles
                    admin_principals: std::collections::BTreeSet::new(),
                },
                migration_stats: MigrationStats {
                    total_cycles_consumed: 1_000_000_000_000, // 1T cycles consumed
                    ..Default::default()
                },
                personal_canisters: BTreeMap::new(),
                ..Default::default()
            };
        });
    }

    fn setup_low_reserve_state() {
        with_mock_migration_state_mut(|state| {
            *state = MigrationStateData {
                migration_config: MigrationConfig {
                    enabled: true,
                    cycles_reserve: 1_000_000_000_000, // 1T cycles (below threshold)
                    min_cycles_threshold: 2_000_000_000_000, // 2T cycles
                    admin_principals: std::collections::BTreeSet::new(),
                },
                migration_stats: MigrationStats {
                    total_cycles_consumed: 5_000_000_000_000, // 5T cycles consumed
                    ..Default::default()
                },
                personal_canisters: BTreeMap::new(),
                ..Default::default()
            };
        });
    }

    fn create_test_principal(id: u8) -> Principal {
        let mut bytes = [0u8; 29];
        bytes[0] = id;
        Principal::from_slice(&bytes)
    }

    // Mock cycles functions that use our mock state
    fn mock_preflight_cycles_reserve(required_cycles: u128) -> Result<(), String> {
        with_mock_migration_state(|state| {
            let config = &state.migration_config;

            if config.cycles_reserve < config.min_cycles_threshold {
                return Err(format!(
                    "Factory cycles reserve ({}) is below minimum threshold ({})",
                    config.cycles_reserve, config.min_cycles_threshold
                ));
            }

            if config.cycles_reserve < required_cycles {
                return Err(format!(
                    "Insufficient cycles in factory reserve. Required: {}, Available: {}",
                    required_cycles, config.cycles_reserve
                ));
            }

            Ok(())
        })
    }

    fn mock_consume_cycles_from_reserve(cycles_to_consume: u128) -> Result<(), String> {
        with_mock_migration_state_mut(|state| {
            let config = &mut state.migration_config;

            if config.cycles_reserve < cycles_to_consume {
                return Err(format!(
                    "Cannot consume {} cycles, only {} available in reserve",
                    cycles_to_consume, config.cycles_reserve
                ));
            }

            config.cycles_reserve = config.cycles_reserve.saturating_sub(cycles_to_consume);
            state.migration_stats.total_cycles_consumed = state
                .migration_stats
                .total_cycles_consumed
                .saturating_add(cycles_to_consume);

            Ok(())
        })
    }

    fn mock_get_cycles_reserve_status() -> CyclesReserveStatus {
        with_mock_migration_state(|state| {
            let config = &state.migration_config;
            CyclesReserveStatus {
                current_reserve: config.cycles_reserve,
                min_threshold: config.min_cycles_threshold,
                is_above_threshold: config.cycles_reserve >= config.min_cycles_threshold,
                total_consumed: state.migration_stats.total_cycles_consumed,
            }
        })
    }

    // Mock registry functions
    fn mock_create_registry_entry(
        canister_id: Principal,
        created_by: Principal,
        status: MigrationStatus,
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

        with_mock_migration_state_mut(|state| {
            state.personal_canisters.insert(canister_id, record);
        });

        Ok(())
    }

    fn mock_get_registry_entries_by_user(user: Principal) -> Vec<PersonalCanisterRecord> {
        with_mock_migration_state(|state| {
            state
                .personal_canisters
                .values()
                .filter(|record| record.created_by == user)
                .cloned()
                .collect()
        })
    }

    fn mock_get_registry_entries_by_status(status: MigrationStatus) -> Vec<PersonalCanisterRecord> {
        with_mock_migration_state(|state| {
            state
                .personal_canisters
                .values()
                .filter(|record| record.status == status)
                .cloned()
                .collect()
        })
    }

    #[test]
    fn test_cycles_preflight_sufficient_reserve() {
        setup_test_state();
        let required_cycles = 1_000_000_000_000; // 1T cycles

        let result = mock_preflight_cycles_reserve(required_cycles);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cycles_preflight_below_threshold() {
        setup_low_reserve_state();
        let required_cycles = 500_000_000_000; // 0.5T cycles

        let result = mock_preflight_cycles_reserve(required_cycles);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("below minimum threshold"));
    }

    #[test]
    fn test_cycles_preflight_insufficient_reserve() {
        setup_test_state();
        let required_cycles = 15_000_000_000_000; // 15T cycles (more than available)

        let result = mock_preflight_cycles_reserve(required_cycles);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient cycles"));
    }

    #[test]
    fn test_cycles_consumption_success() {
        setup_test_state();
        let cycles_to_consume = 3_000_000_000_000; // 3T cycles

        let initial_status = mock_get_cycles_reserve_status();
        let result = mock_consume_cycles_from_reserve(cycles_to_consume);
        assert!(result.is_ok());

        let final_status = mock_get_cycles_reserve_status();
        assert_eq!(
            final_status.current_reserve,
            initial_status.current_reserve - cycles_to_consume
        );
        assert_eq!(
            final_status.total_consumed,
            initial_status.total_consumed + cycles_to_consume
        );
    }

    #[test]
    fn test_cycles_consumption_insufficient() {
        setup_test_state();
        let cycles_to_consume = 15_000_000_000_000; // 15T cycles (more than available)

        let result = mock_consume_cycles_from_reserve(cycles_to_consume);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot consume"));
    }

    #[test]
    fn test_cycles_reserve_status() {
        setup_test_state();

        let status = mock_get_cycles_reserve_status();
        assert_eq!(status.current_reserve, 10_000_000_000_000);
        assert_eq!(status.min_threshold, 2_000_000_000_000);
        assert!(status.is_above_threshold);
        assert_eq!(status.total_consumed, 1_000_000_000_000);
    }

    #[test]
    fn test_cycles_threshold_monitoring() {
        // Test normal state
        setup_test_state();
        let normal_status = mock_get_cycles_reserve_status();
        assert!(normal_status.is_above_threshold);

        // Test low reserve state
        setup_low_reserve_state();
        let low_status = mock_get_cycles_reserve_status();
        assert!(!low_status.is_above_threshold);
    }

    #[test]
    fn test_registry_entry_creation() {
        setup_test_state();
        let canister_id = create_test_principal(10);
        let created_by = create_test_principal(1);
        let status = MigrationStatus::Creating;
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
            MigrationStatus::Completed,
            2_000_000_000_000,
        )
        .unwrap();
        mock_create_registry_entry(canister2, user1, MigrationStatus::Failed, 500_000_000_000)
            .unwrap();
        mock_create_registry_entry(canister3, user2, MigrationStatus::Creating, 0).unwrap();

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
            MigrationStatus::Completed,
            2_000_000_000_000,
        )
        .unwrap();
        mock_create_registry_entry(canister2, user1, MigrationStatus::Failed, 500_000_000_000)
            .unwrap();
        mock_create_registry_entry(
            canister3,
            user2,
            MigrationStatus::Completed,
            3_000_000_000_000,
        )
        .unwrap();

        // Query by Completed status
        let completed_entries = mock_get_registry_entries_by_status(MigrationStatus::Completed);
        assert_eq!(completed_entries.len(), 2);

        // Query by Failed status
        let failed_entries = mock_get_registry_entries_by_status(MigrationStatus::Failed);
        assert_eq!(failed_entries.len(), 1);

        // Query by Creating status
        let creating_entries = mock_get_registry_entries_by_status(MigrationStatus::Creating);
        assert_eq!(creating_entries.len(), 0);
    }

    #[test]
    fn test_cycles_and_registry_integration() {
        setup_test_state();
        let user = create_test_principal(1);
        let canister_id = create_test_principal(10);
        let required_cycles = 2_000_000_000_000; // 2T cycles

        // Step 1: Preflight check
        let preflight_result = mock_preflight_cycles_reserve(required_cycles);
        assert!(preflight_result.is_ok());

        // Step 2: Create registry entry
        let registry_result =
            mock_create_registry_entry(canister_id, user, MigrationStatus::Creating, 0);
        assert!(registry_result.is_ok());

        // Step 3: Consume cycles
        let consumption_result = mock_consume_cycles_from_reserve(required_cycles);
        assert!(consumption_result.is_ok());

        // Step 4: Verify state changes
        let final_status = mock_get_cycles_reserve_status();
        assert_eq!(final_status.current_reserve, 8_000_000_000_000); // 10T - 2T
        assert_eq!(final_status.total_consumed, 3_000_000_000_000); // 1T + 2T

        let user_entries = mock_get_registry_entries_by_user(user);
        assert_eq!(user_entries.len(), 1);
        assert_eq!(user_entries[0].canister_id, canister_id);
    }

    #[test]
    fn test_cycles_alert_levels() {
        // Test normal alert level
        setup_test_state();
        let normal_status = mock_get_cycles_reserve_status();
        assert!(normal_status.is_above_threshold);

        // Test warning alert level
        setup_low_reserve_state();
        let warning_status = mock_get_cycles_reserve_status();
        assert!(!warning_status.is_above_threshold);

        // Test critical alert level (below 50% of threshold)
        with_mock_migration_state_mut(|state| {
            state.migration_config.cycles_reserve = 500_000_000_000; // 0.5T cycles
        });
        let critical_status = mock_get_cycles_reserve_status();
        assert!(!critical_status.is_above_threshold);
        let critical_threshold = critical_status.min_threshold / 2;
        assert!(critical_status.current_reserve <= critical_threshold);
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
            MigrationStatus::Completed,
            2_000_000_000_000,
        )
        .unwrap();
        mock_create_registry_entry(
            canister2,
            user2,
            MigrationStatus::Completed,
            3_000_000_000_000,
        )
        .unwrap();

        // Calculate total cycles from registry
        let all_completed = mock_get_registry_entries_by_status(MigrationStatus::Completed);
        let total_registry_cycles: u128 = all_completed
            .iter()
            .map(|record| record.cycles_consumed)
            .sum();

        assert_eq!(total_registry_cycles, 5_000_000_000_000); // 2T + 3T
    }

    #[test]
    fn test_multiple_operations_sequence() {
        setup_test_state();
        let user = create_test_principal(1);
        let operations = vec![
            (create_test_principal(10), 1_000_000_000_000), // 1T cycles
            (create_test_principal(11), 2_000_000_000_000), // 2T cycles
            (create_test_principal(12), 1_500_000_000_000), // 1.5T cycles
        ];

        let initial_status = mock_get_cycles_reserve_status();
        let mut expected_consumed = initial_status.total_consumed;
        let mut expected_reserve = initial_status.current_reserve;

        for (canister_id, cycles) in operations {
            // Preflight check
            let preflight_result = mock_preflight_cycles_reserve(cycles);
            assert!(preflight_result.is_ok());

            // Create registry entry
            let registry_result =
                mock_create_registry_entry(canister_id, user, MigrationStatus::Creating, 0);
            assert!(registry_result.is_ok());

            // Consume cycles
            let consumption_result = mock_consume_cycles_from_reserve(cycles);
            assert!(consumption_result.is_ok());

            // Update expected values
            expected_consumed += cycles;
            expected_reserve -= cycles;
        }

        // Verify final state
        let final_status = mock_get_cycles_reserve_status();
        assert_eq!(final_status.current_reserve, expected_reserve);
        assert_eq!(final_status.total_consumed, expected_consumed);

        // Verify registry entries
        let user_entries = mock_get_registry_entries_by_user(user);
        assert_eq!(user_entries.len(), 3);
    }

    // Import Session Management Tests

    fn setup_import_test_state() {
        with_mock_migration_state_mut(|state| {
            *state = MigrationStateData {
                import_config: ImportConfig {
                    max_chunk_size: 1_000_000,          // 1MB max chunk size
                    max_total_import_size: 100_000_000, // 100MB max total import size
                    session_timeout_seconds: 3600,      // 1 hour session timeout
                },
                import_sessions: std::collections::HashMap::new(),
                ..Default::default()
            };
        });
    }

    fn mock_time() -> u64 {
        1234567890000000000 // Mock timestamp in nanoseconds
    }

    fn mock_begin_import(user: Principal) -> Result<ImportSessionResponse, String> {
        let session_id = format!("import_{}", simple_hash(&user.to_text()));
        let now = mock_time();

        with_mock_migration_state_mut(|state| {
            // Clean up expired sessions
            let timeout_nanos = state.import_config.session_timeout_seconds * 1_000_000_000;
            let expired_sessions: Vec<String> = state
                .import_sessions
                .iter()
                .filter(|(_, session)| {
                    let session_age = now - session.last_activity_at;
                    session_age > timeout_nanos && session.status == ImportSessionStatus::Active
                })
                .map(|(id, _)| id.clone())
                .collect();

            for session_id in expired_sessions {
                state.import_sessions.remove(&session_id);
            }

            // Check if user already has an active session
            let existing_active = state
                .import_sessions
                .values()
                .any(|s| s.user == user && s.status == ImportSessionStatus::Active);

            if existing_active {
                return Ok(ImportSessionResponse {
                    success: false,
                    session_id: None,
                    message: "User already has an active import session".to_string(),
                });
            }

            // Create new import session
            let session = ImportSession {
                session_id: session_id.clone(),
                user,
                created_at: now,
                last_activity_at: now,
                total_expected_size: 0,
                total_received_size: 0,
                memories_in_progress: std::collections::HashMap::new(),
                completed_memories: std::collections::HashMap::new(),
                import_manifest: None,
                status: ImportSessionStatus::Active,
            };

            state.import_sessions.insert(session_id.clone(), session);

            Ok(ImportSessionResponse {
                success: true,
                session_id: Some(session_id),
                message: "Import session created successfully".to_string(),
            })
        })
    }

    fn mock_put_memory_chunk(
        user: Principal,
        session_id: String,
        memory_id: String,
        chunk_index: u32,
        bytes: Vec<u8>,
        sha256: String,
    ) -> Result<ChunkUploadResponse, String> {
        with_mock_migration_state_mut(|state| {
            let config = &state.import_config;

            // Validate chunk size
            if bytes.len() as u64 > config.max_chunk_size {
                return Ok(ChunkUploadResponse {
                    success: false,
                    message: format!(
                        "Chunk size {} exceeds maximum allowed size {}",
                        bytes.len(),
                        config.max_chunk_size
                    ),
                    received_size: 0,
                    total_expected_size: 0,
                });
            }

            // Get and validate session
            let session = match state.import_sessions.get_mut(&session_id) {
                Some(s) => s,
                None => {
                    return Ok(ChunkUploadResponse {
                        success: false,
                        message: "Import session not found".to_string(),
                        received_size: 0,
                        total_expected_size: 0,
                    })
                }
            };

            // Validate session ownership
            if session.user != user {
                return Ok(ChunkUploadResponse {
                    success: false,
                    message: "Access denied: session belongs to different user".to_string(),
                    received_size: 0,
                    total_expected_size: 0,
                });
            }

            // Check session status
            if session.status != ImportSessionStatus::Active {
                return Ok(ChunkUploadResponse {
                    success: false,
                    message: format!("Session is not active (status: {:?})", session.status),
                    received_size: 0,
                    total_expected_size: 0,
                });
            }

            // Check session timeout
            let now = mock_time();
            let session_age = (now - session.last_activity_at) / 1_000_000_000;
            if session_age > config.session_timeout_seconds {
                session.status = ImportSessionStatus::Expired;
                return Ok(ChunkUploadResponse {
                    success: false,
                    message: "Session has expired".to_string(),
                    received_size: 0,
                    total_expected_size: 0,
                });
            }

            // Validate total import size
            let new_total_size = session.total_received_size + bytes.len() as u64;
            if new_total_size > config.max_total_import_size {
                return Ok(ChunkUploadResponse {
                    success: false,
                    message: format!(
                        "Total import size would exceed maximum allowed size {}",
                        config.max_total_import_size
                    ),
                    received_size: session.total_received_size,
                    total_expected_size: session.total_expected_size,
                });
            }

            // Validate chunk hash
            let calculated_hash = simple_hash(&String::from_utf8_lossy(&bytes));
            if calculated_hash != sha256 {
                return Ok(ChunkUploadResponse {
                    success: false,
                    message: "Chunk hash validation failed".to_string(),
                    received_size: session.total_received_size,
                    total_expected_size: session.total_expected_size,
                });
            }

            // Get or create memory import state
            let memory_state = session
                .memories_in_progress
                .entry(memory_id.clone())
                .or_insert_with(|| MemoryImportState {
                    memory_id: memory_id.clone(),
                    expected_chunks: 0,
                    received_chunks: std::collections::HashMap::new(),
                    total_size: 0,
                    received_size: 0,
                    memory_metadata: None,
                    is_complete: false,
                });

            // Check if chunk already exists
            if memory_state.received_chunks.contains_key(&chunk_index) {
                return Ok(ChunkUploadResponse {
                    success: false,
                    message: format!(
                        "Chunk {} already received for memory {}",
                        chunk_index, memory_id
                    ),
                    received_size: session.total_received_size,
                    total_expected_size: session.total_expected_size,
                });
            }

            // Store the chunk
            let chunk = ChunkData {
                chunk_index,
                data: bytes.clone(),
                sha256,
                received_at: now,
            };

            memory_state.received_chunks.insert(chunk_index, chunk);
            memory_state.received_size += bytes.len() as u64;

            // Update session totals
            session.total_received_size += bytes.len() as u64;
            session.last_activity_at = now;

            Ok(ChunkUploadResponse {
                success: true,
                message: format!("Chunk {} uploaded successfully", chunk_index),
                received_size: session.total_received_size,
                total_expected_size: session.total_expected_size,
            })
        })
    }

    fn mock_commit_memory(
        user: Principal,
        session_id: String,
        manifest: MemoryManifest,
    ) -> Result<MemoryCommitResponse, String> {
        with_mock_migration_state_mut(|state| {
            // Get and validate session
            let session = match state.import_sessions.get_mut(&session_id) {
                Some(s) => s,
                None => {
                    return Ok(MemoryCommitResponse {
                        success: false,
                        message: "Import session not found".to_string(),
                        memory_id: manifest.memory_id.clone(),
                        assembled_size: 0,
                    })
                }
            };

            // Validate session ownership
            if session.user != user {
                return Ok(MemoryCommitResponse {
                    success: false,
                    message: "Access denied: session belongs to different user".to_string(),
                    memory_id: manifest.memory_id.clone(),
                    assembled_size: 0,
                });
            }

            // Check session status
            if session.status != ImportSessionStatus::Active {
                return Ok(MemoryCommitResponse {
                    success: false,
                    message: format!("Session is not active (status: {:?})", session.status),
                    memory_id: manifest.memory_id.clone(),
                    assembled_size: 0,
                });
            }

            // Get memory import state
            let memory_state = match session.memories_in_progress.get_mut(&manifest.memory_id) {
                Some(state) => state,
                None => {
                    return Ok(MemoryCommitResponse {
                        success: false,
                        message: format!("Memory {} not found in session", manifest.memory_id),
                        memory_id: manifest.memory_id.clone(),
                        assembled_size: 0,
                    })
                }
            };

            // Validate chunk count
            if memory_state.received_chunks.len() as u32 != manifest.total_chunks {
                return Ok(MemoryCommitResponse {
                    success: false,
                    message: format!(
                        "Chunk count mismatch: received {}, expected {}",
                        memory_state.received_chunks.len(),
                        manifest.total_chunks
                    ),
                    memory_id: manifest.memory_id.clone(),
                    assembled_size: 0,
                });
            }

            // Validate total size
            if memory_state.received_size != manifest.total_size {
                return Ok(MemoryCommitResponse {
                    success: false,
                    message: format!(
                        "Size mismatch: received {}, expected {}",
                        memory_state.received_size, manifest.total_size
                    ),
                    memory_id: manifest.memory_id.clone(),
                    assembled_size: 0,
                });
            }

            // Assemble chunks in order and validate checksums
            let mut assembled_data = Vec::new();
            for chunk_index in 0..manifest.total_chunks {
                match memory_state.received_chunks.get(&chunk_index) {
                    Some(chunk) => {
                        // Validate chunk checksum against manifest
                        if chunk_index < manifest.chunk_checksums.len() as u32 {
                            let expected_checksum = &manifest.chunk_checksums[chunk_index as usize];
                            if chunk.sha256 != *expected_checksum {
                                return Ok(MemoryCommitResponse {
                                    success: false,
                                    message: format!(
                                        "Chunk {} checksum mismatch for memory {}",
                                        chunk_index, manifest.memory_id
                                    ),
                                    memory_id: manifest.memory_id.clone(),
                                    assembled_size: 0,
                                });
                            }
                        }
                        assembled_data.extend_from_slice(&chunk.data);
                    }
                    None => {
                        return Ok(MemoryCommitResponse {
                            success: false,
                            message: format!(
                                "Missing chunk {} for memory {}",
                                chunk_index, manifest.memory_id
                            ),
                            memory_id: manifest.memory_id.clone(),
                            assembled_size: 0,
                        })
                    }
                }
            }

            // Validate final assembled data checksum
            let final_checksum = simple_hash(&String::from_utf8_lossy(&assembled_data));
            if final_checksum != manifest.final_checksum {
                return Ok(MemoryCommitResponse {
                    success: false,
                    message: format!(
                        "Final checksum mismatch for memory {}: calculated {}, expected {}",
                        manifest.memory_id, final_checksum, manifest.final_checksum
                    ),
                    memory_id: manifest.memory_id.clone(),
                    assembled_size: 0,
                });
            }

            // Create mock memory object
            let memory = create_mock_memory(&manifest.memory_id, assembled_data);

            // Move memory from in-progress to completed
            session
                .completed_memories
                .insert(manifest.memory_id.clone(), memory);
            session.memories_in_progress.remove(&manifest.memory_id);

            // Update session activity
            session.last_activity_at = mock_time();

            Ok(MemoryCommitResponse {
                success: true,
                message: format!("Memory {} committed successfully", manifest.memory_id),
                memory_id: manifest.memory_id,
                assembled_size: manifest.total_size,
            })
        })
    }

    fn mock_finalize_import(
        user: Principal,
        session_id: String,
    ) -> Result<ImportFinalizationResponse, String> {
        with_mock_migration_state_mut(|state| {
            // Get and validate session
            let session = match state.import_sessions.get_mut(&session_id) {
                Some(s) => s,
                None => {
                    return Ok(ImportFinalizationResponse {
                        success: false,
                        message: "Import session not found".to_string(),
                        total_memories_imported: 0,
                        total_size_imported: 0,
                    })
                }
            };

            // Validate session ownership
            if session.user != user {
                return Ok(ImportFinalizationResponse {
                    success: false,
                    message: "Access denied: session belongs to different user".to_string(),
                    total_memories_imported: 0,
                    total_size_imported: 0,
                });
            }

            // Check session status
            if session.status != ImportSessionStatus::Active {
                return Ok(ImportFinalizationResponse {
                    success: false,
                    message: format!("Session is not active (status: {:?})", session.status),
                    total_memories_imported: 0,
                    total_size_imported: 0,
                });
            }

            // Check that all memories in progress have been committed
            if !session.memories_in_progress.is_empty() {
                return Ok(ImportFinalizationResponse {
                    success: false,
                    message: format!(
                        "Cannot finalize: {} memories still in progress",
                        session.memories_in_progress.len()
                    ),
                    total_memories_imported: 0,
                    total_size_imported: 0,
                });
            }

            // Update session status
            session.status = ImportSessionStatus::Finalizing;
            session.last_activity_at = mock_time();

            let total_memories = session.completed_memories.len() as u32;
            let total_size = session.total_received_size;

            // Mark session as completed
            session.status = ImportSessionStatus::Completed;

            Ok(ImportFinalizationResponse {
                success: true,
                message: format!(
                    "Import finalized successfully: {} memories imported",
                    total_memories
                ),
                total_memories_imported: total_memories,
                total_size_imported: total_size,
            })
        })
    }

    fn create_mock_memory(memory_id: &str, data: Vec<u8>) -> crate::types::Memory {
        let now = mock_time();
        let data_size = data.len() as u64;

        crate::types::Memory {
            id: memory_id.to_string(),
            info: crate::types::MemoryInfo {
                memory_type: crate::types::MemoryType::Document,
                name: format!("Test Memory {}", memory_id),
                content_type: "application/octet-stream".to_string(),
                created_at: now,
                updated_at: now,
                uploaded_at: now,
                date_of_memory: Some(now),
            },
            data: crate::types::MemoryData {
                blob_ref: crate::types::BlobRef {
                    kind: crate::types::MemoryBlobKind::ICPCapsule,
                    locator: format!("test:{}", memory_id),
                    hash: None,
                },
                data: Some(data),
            },
            access: crate::types::MemoryAccess::Private,
            metadata: crate::types::MemoryMetadata::Document(crate::types::DocumentMetadata {
                base: crate::types::MemoryMetadataBase {
                    size: data_size,
                    mime_type: "application/octet-stream".to_string(),
                    original_name: format!("test_{}.bin", memory_id),
                    uploaded_at: now.to_string(),
                    date_of_memory: Some(now.to_string()),
                    people_in_memory: None,
                    format: Some("binary".to_string()),
                },
            }),
        }
    }

    fn simple_hash(data: &str) -> String {
        let mut hash: u64 = 5381;
        for byte in data.bytes() {
            hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u64);
        }
        format!("{:016x}", hash)
    }

    // Test import session creation and lifecycle
    #[test]
    fn test_import_session_creation_success() {
        setup_import_test_state();
        let user = create_test_principal(1);

        let result = mock_begin_import(user);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.success);
        assert!(response.session_id.is_some());
        assert_eq!(response.message, "Import session created successfully");

        // Verify session was stored
        with_mock_migration_state(|state| {
            let session_id = response.session_id.unwrap();
            let session = state.import_sessions.get(&session_id).unwrap();
            assert_eq!(session.user, user);
            assert_eq!(session.status, ImportSessionStatus::Active);
            assert_eq!(session.total_received_size, 0);
            assert!(session.memories_in_progress.is_empty());
            assert!(session.completed_memories.is_empty());
        });
    }

    #[test]
    fn test_import_session_duplicate_prevention() {
        setup_import_test_state();
        let user = create_test_principal(1);

        // Create first session
        let result1 = mock_begin_import(user);
        assert!(result1.is_ok());
        assert!(result1.unwrap().success);

        // Try to create second session for same user
        let result2 = mock_begin_import(user);
        assert!(result2.is_ok());

        let response2 = result2.unwrap();
        assert!(!response2.success);
        assert!(response2.session_id.is_none());
        assert!(response2
            .message
            .contains("already has an active import session"));
    }

    #[test]
    fn test_import_session_cleanup_expired() {
        setup_import_test_state();
        let user1 = create_test_principal(1);
        let user2 = create_test_principal(2);

        // Create session for user1
        let result1 = mock_begin_import(user1);
        assert!(result1.is_ok());

        // Manually expire the session by modifying the timestamp
        with_mock_migration_state_mut(|state| {
            let session_id = result1.unwrap().session_id.unwrap();
            if let Some(session) = state.import_sessions.get_mut(&session_id) {
                // Set last activity to 2 hours ago (beyond 1 hour timeout)
                session.last_activity_at = mock_time() - (2 * 3600 * 1_000_000_000);
            }
        });

        // Try to create session for user2, which should trigger cleanup
        let result2 = mock_begin_import(user2);
        assert!(result2.is_ok());
        assert!(result2.unwrap().success);

        // Verify expired session was cleaned up
        with_mock_migration_state(|state| {
            let active_sessions: Vec<_> = state
                .import_sessions
                .values()
                .filter(|s| s.status == ImportSessionStatus::Active)
                .collect();
            assert_eq!(active_sessions.len(), 1);
            assert_eq!(active_sessions[0].user, user2);
        });
    }

    // Test chunk upload and assembly
    #[test]
    fn test_chunk_upload_success() {
        setup_import_test_state();
        let user = create_test_principal(1);

        // Create session
        let session_result = mock_begin_import(user);
        let session_id = session_result.unwrap().session_id.unwrap();

        // Upload chunk
        let memory_id = "test_memory_1".to_string();
        let chunk_data = b"Hello, World!".to_vec();
        let chunk_hash = simple_hash(&String::from_utf8_lossy(&chunk_data));

        let result = mock_put_memory_chunk(
            user,
            session_id.clone(),
            memory_id.clone(),
            0,
            chunk_data.clone(),
            chunk_hash,
        );
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.success);
        assert_eq!(response.received_size, chunk_data.len() as u64);
        assert!(response.message.contains("uploaded successfully"));

        // Verify chunk was stored
        with_mock_migration_state(|state| {
            let session = state.import_sessions.get(&session_id).unwrap();
            assert_eq!(session.total_received_size, chunk_data.len() as u64);

            let memory_state = session.memories_in_progress.get(&memory_id).unwrap();
            assert_eq!(memory_state.received_chunks.len(), 1);
            assert!(memory_state.received_chunks.contains_key(&0));

            let chunk = memory_state.received_chunks.get(&0).unwrap();
            assert_eq!(chunk.data, chunk_data);
            assert_eq!(chunk.chunk_index, 0);
        });
    }

    #[test]
    fn test_chunk_upload_size_validation() {
        setup_import_test_state();
        let user = create_test_principal(1);

        let session_result = mock_begin_import(user);
        let session_id = session_result.unwrap().session_id.unwrap();

        // Try to upload chunk that exceeds max size
        let memory_id = "test_memory_1".to_string();
        let oversized_chunk = vec![0u8; 2_000_000]; // 2MB, exceeds 1MB limit
        let chunk_hash = simple_hash(&String::from_utf8_lossy(&oversized_chunk));

        let result =
            mock_put_memory_chunk(user, session_id, memory_id, 0, oversized_chunk, chunk_hash);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.success);
        assert!(response.message.contains("exceeds maximum allowed size"));
    }

    #[test]
    fn test_chunk_upload_hash_validation() {
        setup_import_test_state();
        let user = create_test_principal(1);

        let session_result = mock_begin_import(user);
        let session_id = session_result.unwrap().session_id.unwrap();

        // Upload chunk with incorrect hash
        let memory_id = "test_memory_1".to_string();
        let chunk_data = b"Hello, World!".to_vec();
        let wrong_hash = "incorrect_hash".to_string();

        let result = mock_put_memory_chunk(user, session_id, memory_id, 0, chunk_data, wrong_hash);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.success);
        assert!(response.message.contains("hash validation failed"));
    }

    #[test]
    fn test_chunk_upload_duplicate_prevention() {
        setup_import_test_state();
        let user = create_test_principal(1);

        let session_result = mock_begin_import(user);
        let session_id = session_result.unwrap().session_id.unwrap();

        let memory_id = "test_memory_1".to_string();
        let chunk_data = b"Hello, World!".to_vec();
        let chunk_hash = simple_hash(&String::from_utf8_lossy(&chunk_data));

        // Upload chunk first time
        let result1 = mock_put_memory_chunk(
            user,
            session_id.clone(),
            memory_id.clone(),
            0,
            chunk_data.clone(),
            chunk_hash.clone(),
        );
        assert!(result1.is_ok());
        assert!(result1.unwrap().success);

        // Try to upload same chunk again
        let result2 = mock_put_memory_chunk(user, session_id, memory_id, 0, chunk_data, chunk_hash);
        assert!(result2.is_ok());

        let response2 = result2.unwrap();
        assert!(!response2.success);
        assert!(response2.message.contains("already received"));
    }

    #[test]
    fn test_chunk_upload_session_access_control() {
        setup_import_test_state();
        let user1 = create_test_principal(1);
        let user2 = create_test_principal(2);

        // Create session for user1
        let session_result = mock_begin_import(user1);
        let session_id = session_result.unwrap().session_id.unwrap();

        // Try to upload chunk as user2
        let memory_id = "test_memory_1".to_string();
        let chunk_data = b"Hello, World!".to_vec();
        let chunk_hash = simple_hash(&String::from_utf8_lossy(&chunk_data));

        let result = mock_put_memory_chunk(user2, session_id, memory_id, 0, chunk_data, chunk_hash);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.success);
        assert!(response.message.contains("Access denied"));
    }

    // Test memory commit and finalization
    #[test]
    fn test_memory_commit_success() {
        setup_import_test_state();
        let user = create_test_principal(1);

        let session_result = mock_begin_import(user);
        let session_id = session_result.unwrap().session_id.unwrap();

        let memory_id = "test_memory_1".to_string();

        // Upload chunks
        let chunk1_data = b"Hello, ".to_vec();
        let chunk2_data = b"World!".to_vec();
        let chunk1_hash = simple_hash(&String::from_utf8_lossy(&chunk1_data));
        let chunk2_hash = simple_hash(&String::from_utf8_lossy(&chunk2_data));

        mock_put_memory_chunk(
            user,
            session_id.clone(),
            memory_id.clone(),
            0,
            chunk1_data.clone(),
            chunk1_hash.clone(),
        )
        .unwrap();
        mock_put_memory_chunk(
            user,
            session_id.clone(),
            memory_id.clone(),
            1,
            chunk2_data.clone(),
            chunk2_hash.clone(),
        )
        .unwrap();

        // Create manifest
        let mut full_data = chunk1_data.clone();
        full_data.extend_from_slice(&chunk2_data);
        let final_checksum = simple_hash(&String::from_utf8_lossy(&full_data));

        let manifest = MemoryManifest {
            memory_id: memory_id.clone(),
            total_chunks: 2,
            total_size: full_data.len() as u64,
            chunk_checksums: vec![chunk1_hash, chunk2_hash],
            final_checksum,
        };

        // Commit memory
        let result = mock_commit_memory(user, session_id.clone(), manifest);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.success);
        assert_eq!(response.memory_id, memory_id);
        assert_eq!(response.assembled_size, full_data.len() as u64);

        // Verify memory was moved to completed
        with_mock_migration_state(|state| {
            let session = state.import_sessions.get(&session_id).unwrap();
            assert!(!session.memories_in_progress.contains_key(&memory_id));
            assert!(session.completed_memories.contains_key(&memory_id));

            let memory = session.completed_memories.get(&memory_id).unwrap();
            assert_eq!(memory.id, memory_id);
            assert_eq!(memory.data.data.as_ref().unwrap(), &full_data);
        });
    }

    #[test]
    fn test_memory_commit_chunk_count_mismatch() {
        setup_import_test_state();
        let user = create_test_principal(1);

        let session_result = mock_begin_import(user);
        let session_id = session_result.unwrap().session_id.unwrap();

        let memory_id = "test_memory_1".to_string();

        // Upload only one chunk
        let chunk_data = b"Hello, World!".to_vec();
        let chunk_hash = simple_hash(&String::from_utf8_lossy(&chunk_data));
        mock_put_memory_chunk(
            user,
            session_id.clone(),
            memory_id.clone(),
            0,
            chunk_data.clone(),
            chunk_hash.clone(),
        )
        .unwrap();

        // Create manifest expecting 2 chunks
        let manifest = MemoryManifest {
            memory_id: memory_id.clone(),
            total_chunks: 2,
            total_size: chunk_data.len() as u64,
            chunk_checksums: vec![chunk_hash.clone(), chunk_hash],
            final_checksum: simple_hash(&String::from_utf8_lossy(&chunk_data)),
        };

        let result = mock_commit_memory(user, session_id, manifest);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.success);
        assert!(response.message.contains("Chunk count mismatch"));
    }

    #[test]
    fn test_memory_commit_size_mismatch() {
        setup_import_test_state();
        let user = create_test_principal(1);

        let session_result = mock_begin_import(user);
        let session_id = session_result.unwrap().session_id.unwrap();

        let memory_id = "test_memory_1".to_string();

        let chunk_data = b"Hello, World!".to_vec();
        let chunk_hash = simple_hash(&String::from_utf8_lossy(&chunk_data));
        mock_put_memory_chunk(
            user,
            session_id.clone(),
            memory_id.clone(),
            0,
            chunk_data.clone(),
            chunk_hash.clone(),
        )
        .unwrap();

        // Create manifest with wrong total size
        let manifest = MemoryManifest {
            memory_id: memory_id.clone(),
            total_chunks: 1,
            total_size: 999, // Wrong size
            chunk_checksums: vec![chunk_hash],
            final_checksum: simple_hash(&String::from_utf8_lossy(&chunk_data)),
        };

        let result = mock_commit_memory(user, session_id, manifest);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.success);
        assert!(response.message.contains("Size mismatch"));
    }

    #[test]
    fn test_memory_commit_checksum_validation() {
        setup_import_test_state();
        let user = create_test_principal(1);

        let session_result = mock_begin_import(user);
        let session_id = session_result.unwrap().session_id.unwrap();

        let memory_id = "test_memory_1".to_string();

        let chunk_data = b"Hello, World!".to_vec();
        let chunk_hash = simple_hash(&String::from_utf8_lossy(&chunk_data));
        mock_put_memory_chunk(
            user,
            session_id.clone(),
            memory_id.clone(),
            0,
            chunk_data.clone(),
            chunk_hash.clone(),
        )
        .unwrap();

        // Create manifest with wrong final checksum
        let manifest = MemoryManifest {
            memory_id: memory_id.clone(),
            total_chunks: 1,
            total_size: chunk_data.len() as u64,
            chunk_checksums: vec![chunk_hash],
            final_checksum: "wrong_checksum".to_string(),
        };

        let result = mock_commit_memory(user, session_id, manifest);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.success);
        assert!(response.message.contains("Final checksum mismatch"));
    }

    #[test]
    fn test_import_finalization_success() {
        setup_import_test_state();
        let user = create_test_principal(1);

        let session_result = mock_begin_import(user);
        let session_id = session_result.unwrap().session_id.unwrap();

        let memory_id = "test_memory_1".to_string();

        // Upload and commit a memory
        let chunk_data = b"Hello, World!".to_vec();
        let chunk_hash = simple_hash(&String::from_utf8_lossy(&chunk_data));
        mock_put_memory_chunk(
            user,
            session_id.clone(),
            memory_id.clone(),
            0,
            chunk_data.clone(),
            chunk_hash.clone(),
        )
        .unwrap();

        let manifest = MemoryManifest {
            memory_id: memory_id.clone(),
            total_chunks: 1,
            total_size: chunk_data.len() as u64,
            chunk_checksums: vec![chunk_hash],
            final_checksum: simple_hash(&String::from_utf8_lossy(&chunk_data)),
        };

        mock_commit_memory(user, session_id.clone(), manifest).unwrap();

        // Finalize import
        let result = mock_finalize_import(user, session_id.clone());
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.success);
        assert_eq!(response.total_memories_imported, 1);
        assert_eq!(response.total_size_imported, chunk_data.len() as u64);

        // Verify session status
        with_mock_migration_state(|state| {
            let session = state.import_sessions.get(&session_id).unwrap();
            assert_eq!(session.status, ImportSessionStatus::Completed);
        });
    }

    #[test]
    fn test_import_finalization_with_pending_memories() {
        setup_import_test_state();
        let user = create_test_principal(1);

        let session_result = mock_begin_import(user);
        let session_id = session_result.unwrap().session_id.unwrap();

        let memory_id = "test_memory_1".to_string();

        // Upload chunk but don't commit
        let chunk_data = b"Hello, World!".to_vec();
        let chunk_hash = simple_hash(&String::from_utf8_lossy(&chunk_data));
        mock_put_memory_chunk(
            user,
            session_id.clone(),
            memory_id,
            0,
            chunk_data,
            chunk_hash,
        )
        .unwrap();

        // Try to finalize with pending memory
        let result = mock_finalize_import(user, session_id);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.success);
        assert!(response.message.contains("memories still in progress"));
    }

    // Test session cleanup and error handling
    #[test]
    fn test_session_timeout_handling() {
        setup_import_test_state();
        let user = create_test_principal(1);

        let session_result = mock_begin_import(user);
        let session_id = session_result.unwrap().session_id.unwrap();

        // Manually set session to expired time
        with_mock_migration_state_mut(|state| {
            if let Some(session) = state.import_sessions.get_mut(&session_id) {
                // Set last activity to 2 hours ago (beyond 1 hour timeout)
                session.last_activity_at = mock_time() - (2 * 3600 * 1_000_000_000);
            }
        });

        // Try to upload chunk to expired session
        let memory_id = "test_memory_1".to_string();
        let chunk_data = b"Hello, World!".to_vec();
        let chunk_hash = simple_hash(&String::from_utf8_lossy(&chunk_data));

        let result = mock_put_memory_chunk(
            user,
            session_id.clone(),
            memory_id,
            0,
            chunk_data,
            chunk_hash,
        );
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.success);
        assert!(response.message.contains("expired"));

        // Verify session status was updated
        with_mock_migration_state(|state| {
            let session = state.import_sessions.get(&session_id).unwrap();
            assert_eq!(session.status, ImportSessionStatus::Expired);
        });
    }

    #[test]
    fn test_total_import_size_limit() {
        setup_import_test_state();
        let user = create_test_principal(1);

        let session_result = mock_begin_import(user);
        let session_id = session_result.unwrap().session_id.unwrap();

        // Upload chunks that would exceed total import size limit (100MB)
        // Use 1MB chunks (max allowed size) to test total size limit
        let memory_id = "test_memory_1".to_string();
        let large_chunk = vec![0u8; 1_000_000]; // 1MB chunk (max allowed)
        let chunk_hash = simple_hash(&String::from_utf8_lossy(&large_chunk));

        // Upload 100 chunks of 1MB each (total 100MB - at the limit)
        for i in 0..100 {
            let result = mock_put_memory_chunk(
                user,
                session_id.clone(),
                memory_id.clone(),
                i,
                large_chunk.clone(),
                chunk_hash.clone(),
            );
            assert!(result.is_ok());
            assert!(result.unwrap().success);
        }

        // Next chunk should fail (would exceed 100MB limit)
        let small_chunk = vec![0u8; 1000]; // 1KB chunk
        let small_hash = simple_hash(&String::from_utf8_lossy(&small_chunk));
        let result_fail =
            mock_put_memory_chunk(user, session_id, memory_id, 100, small_chunk, small_hash);
        assert!(result_fail.is_ok());

        let response_fail = result_fail.unwrap();
        assert!(!response_fail.success);
        assert!(response_fail
            .message
            .contains("exceed maximum allowed size"));
    }

    #[test]
    fn test_session_error_handling_nonexistent_session() {
        setup_import_test_state();
        let user = create_test_principal(1);

        let fake_session_id = "nonexistent_session".to_string();
        let memory_id = "test_memory_1".to_string();
        let chunk_data = b"Hello, World!".to_vec();
        let chunk_hash = simple_hash(&String::from_utf8_lossy(&chunk_data));

        let result =
            mock_put_memory_chunk(user, fake_session_id, memory_id, 0, chunk_data, chunk_hash);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.success);
        assert!(response.message.contains("session not found"));
    }

    #[test]
    fn test_multiple_memory_import_workflow() {
        setup_import_test_state();
        let user = create_test_principal(1);

        let session_result = mock_begin_import(user);
        let session_id = session_result.unwrap().session_id.unwrap();

        // Import first memory
        let memory1_id = "memory_1".to_string();
        let memory1_data = b"First memory content".to_vec();
        let memory1_hash = simple_hash(&String::from_utf8_lossy(&memory1_data));

        mock_put_memory_chunk(
            user,
            session_id.clone(),
            memory1_id.clone(),
            0,
            memory1_data.clone(),
            memory1_hash.clone(),
        )
        .unwrap();

        let manifest1 = MemoryManifest {
            memory_id: memory1_id.clone(),
            total_chunks: 1,
            total_size: memory1_data.len() as u64,
            chunk_checksums: vec![memory1_hash],
            final_checksum: simple_hash(&String::from_utf8_lossy(&memory1_data)),
        };

        let commit1_result = mock_commit_memory(user, session_id.clone(), manifest1);
        assert!(commit1_result.is_ok());
        assert!(commit1_result.unwrap().success);

        // Import second memory
        let memory2_id = "memory_2".to_string();
        let memory2_data = b"Second memory content".to_vec();
        let memory2_hash = simple_hash(&String::from_utf8_lossy(&memory2_data));

        mock_put_memory_chunk(
            user,
            session_id.clone(),
            memory2_id.clone(),
            0,
            memory2_data.clone(),
            memory2_hash.clone(),
        )
        .unwrap();

        let manifest2 = MemoryManifest {
            memory_id: memory2_id.clone(),
            total_chunks: 1,
            total_size: memory2_data.len() as u64,
            chunk_checksums: vec![memory2_hash],
            final_checksum: simple_hash(&String::from_utf8_lossy(&memory2_data)),
        };

        let commit2_result = mock_commit_memory(user, session_id.clone(), manifest2);
        assert!(commit2_result.is_ok());
        assert!(commit2_result.unwrap().success);

        // Finalize import
        let finalize_result = mock_finalize_import(user, session_id.clone());
        assert!(finalize_result.is_ok());

        let finalize_response = finalize_result.unwrap();
        assert!(finalize_response.success);
        assert_eq!(finalize_response.total_memories_imported, 2);
        assert_eq!(
            finalize_response.total_size_imported,
            (memory1_data.len() + memory2_data.len()) as u64
        );

        // Verify final session state
        with_mock_migration_state(|state| {
            let session = state.import_sessions.get(&session_id).unwrap();
            assert_eq!(session.status, ImportSessionStatus::Completed);
            assert_eq!(session.completed_memories.len(), 2);
            assert!(session.completed_memories.contains_key(&memory1_id));
            assert!(session.completed_memories.contains_key(&memory2_id));
            assert!(session.memories_in_progress.is_empty());
        });
    }
}
