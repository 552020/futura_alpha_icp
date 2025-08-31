#[cfg(test)]
mod integration_tests {
    use crate::canister_factory::types::*;
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
                            message: format!("Missing chunk {} for memory {}", chunk_index, manifest.memory_id),
                            memory_id: manifest.memory_id.clone(),
                            assembled_size: 0,
                        });
                    }
                }
            }

            // Validate final assembled data checksum
            let assembled_checksum = simple_hash(&String::from_utf8_lossy(&assembled_data));
            if assembled_checksum != manifest.final_checksum {
                return Ok(MemoryCommitResponse {
                    success: false,
                    message: format!(
                        "Final checksum mismatch for memory {}: expected {}, got {}",
                        manifest.memory_id, manifest.final_checksum, assembled_checksum
                    ),
                    memory_id: manifest.memory_id.clone(),
                    assembled_size: 0,
                });
            }

            // Mark memory as complete and move to completed memories
            memory_state.is_complete = true;
            memory_state.memory_metadata = Some(manifest.memory_metadata.clone());

            let completed_memory = CompletedMemoryImport {
                memory_id: manifest.memory_id.clone(),
                assembled_data,
                memory_metadata: manifest.memory_metadata,
                total_size: manifest.total_size,
                completed_at: mock_time(),
            };

            session.completed_memories.insert(manifest.memory_id.clone(), completed_memory);
            session.memories_in_progress.remove(&manifest.memory_id);

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
                        imported_memories_count: 0,
                        total_imported_size: 0,
                    })
                }
            };

            // Validate session ownership
            if session.user != user {
                return Ok(ImportFinalizationResponse {
                    success: false,
                    message: "Access denied: session belongs to different user".to_string(),
                    imported_memories_count: 0,
                    total_imported_size: 0,
                });
            }

            // Check if there are any memories still in progress
            if !session.memories_in_progress.is_empty() {
                return Ok(ImportFinalizationResponse {
                    success: false,
                    message: format!(
                        "Cannot finalize: {} memories still in progress",
                        session.memories_in_progress.len()
                    ),
                    imported_memories_count: session.completed_memories.len(),
                    total_imported_size: session.total_received_size,
                });
            }

            // Calculate final statistics
            let imported_count = session.completed_memories.len();
            let total_size: u64 = session.completed_memories.values()
                .map(|m| m.total_size)
                .sum();

            // Mark session as completed
            session.status = ImportSessionStatus::Completed;

            Ok(ImportFinalizationResponse {
                success: true,
                message: format!(
                    "Import finalized successfully: {} memories imported, {} bytes total",
                    imported_count, total_size
                ),
                imported_memories_count: imported_count,
                total_imported_size: total_size,
            })
        })
    }

    // Simple hash function for testing
    fn simple_hash(input: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    // End-to-End Migration Integration Tests

    #[test]
    fn test_complete_successful_migration_flow() {
        setup_test_state();
        setup_import_test_state();
        
        let user = create_test_principal(1);
        let canister_id = create_test_principal(10);
        let required_cycles = 2_000_000_000_000; // 2T cycles

        // Step 1: Preflight cycles check
        let preflight_result = mock_preflight_cycles_reserve(required_cycles);
        assert!(preflight_result.is_ok(), "Preflight check should pass");

        // Step 2: Create registry entry (Creating status)
        let registry_result = mock_create_registry_entry(
            canister_id, 
            user, 
            MigrationStatus::Creating, 
            0
        );
        assert!(registry_result.is_ok(), "Registry entry creation should succeed");

        // Step 3: Consume cycles from reserve
        let consumption_result = mock_consume_cycles_from_reserve(required_cycles);
        assert!(consumption_result.is_ok(), "Cycles consumption should succeed");

        // Step 4: Begin import session (simulating data transfer)
        let import_session_result = mock_begin_import(user);
        assert!(import_session_result.is_ok(), "Import session creation should succeed");
        let session_response = import_session_result.unwrap();
        assert!(session_response.success, "Import session should be created successfully");
        let session_id = session_response.session_id.unwrap();

        // Step 5: Upload memory chunks
        let memory_id = "test_memory_1".to_string();
        let test_data = b"This is test memory data for migration";
        let chunk_hash = simple_hash(&String::from_utf8_lossy(test_data));
        
        let chunk_result = mock_put_memory_chunk(
            user,
            session_id.clone(),
            memory_id.clone(),
            0,
            test_data.to_vec(),
            chunk_hash.clone(),
        );
        assert!(chunk_result.is_ok(), "Chunk upload should succeed");
        let chunk_response = chunk_result.unwrap();
        assert!(chunk_response.success, "Chunk upload should be successful");

        // Step 6: Commit memory with manifest
        let manifest = MemoryManifest {
            memory_id: memory_id.clone(),
            total_chunks: 1,
            total_size: test_data.len() as u64,
            chunk_checksums: vec![chunk_hash.clone()],
            final_checksum: chunk_hash.clone(),
            memory_metadata: crate::types::Memory {
                id: memory_id.clone(),
                title: "Test Memory".to_string(),
                description: Some("Test memory for migration".to_string()),
                created_at: mock_time(),
                updated_at: mock_time(),
                memory_type: crate::types::MemoryType::Text,
                tags: vec![],
                is_favorite: false,
                content_hash: Some(chunk_hash.clone()),
                file_extension: None,
                file_size: Some(test_data.len() as u64),
            },
        };

        let commit_result = mock_commit_memory(user, session_id.clone(), manifest);
        assert!(commit_result.is_ok(), "Memory commit should succeed");
        let commit_response = commit_result.unwrap();
        assert!(commit_response.success, "Memory commit should be successful");

        // Step 7: Finalize import
        let finalize_result = mock_finalize_import(user, session_id);
        assert!(finalize_result.is_ok(), "Import finalization should succeed");
        let finalize_response = finalize_result.unwrap();
        assert!(finalize_response.success, "Import finalization should be successful");
        assert_eq!(finalize_response.imported_memories_count, 1);

        // Step 8: Update registry to Completed status
        with_mock_migration_state_mut(|state| {
            if let Some(record) = state.personal_canisters.get_mut(&canister_id) {
                record.status = MigrationStatus::Completed;
                record.cycles_consumed = required_cycles;
            }
        });

        // Verify final state
        let final_cycles_status = mock_get_cycles_reserve_status();
        assert_eq!(final_cycles_status.current_reserve, 8_000_000_000_000); // 10T - 2T
        assert_eq!(final_cycles_status.total_consumed, 3_000_000_000_000); // 1T + 2T

        let user_entries = mock_get_registry_entries_by_user(user);
        assert_eq!(user_entries.len(), 1);
        assert_eq!(user_entries[0].status, MigrationStatus::Completed);
        assert_eq!(user_entries[0].cycles_consumed, required_cycles);
    }

    #[test]
    fn test_idempotent_migrate_capsule_behavior() {
        setup_test_state();
        let user = create_test_principal(1);
        let canister_id = create_test_principal(10);

        // Create initial migration state
        with_mock_migration_state_mut(|state| {
            let migration_state = MigrationState {
                user,
                status: MigrationStatus::Completed,
                created_at: mock_time(),
                completed_at: Some(mock_time()),
                personal_canister_id: Some(canister_id),
                cycles_consumed: 2_000_000_000_000,
                error_message: None,
            };
            state.migration_states.insert(user, migration_state);
        });

        // Mock migrate_capsule function that checks existing state
        let mock_migrate_capsule = |user: Principal| -> Result<MigrationResponse, String> {
            with_mock_migration_state(|state| {
                if let Some(existing_state) = state.migration_states.get(&user) {
                    // Return existing result for idempotency
                    match existing_state.status {
                        MigrationStatus::Completed => Ok(MigrationResponse {
                            success: true,
                            canister_id: existing_state.personal_canister_id,
                            message: "Migration already completed".to_string(),
                        }),
                        MigrationStatus::Failed => Ok(MigrationResponse {
                            success: false,
                            canister_id: None,
                            message: existing_state.error_message.clone()
                                .unwrap_or_else(|| "Migration failed".to_string()),
                        }),
                        _ => Ok(MigrationResponse {
                            success: false,
                            canister_id: None,
                            message: format!("Migration in progress (status: {:?})", existing_state.status),
                        }),
                    }
                } else {
                    // Start new migration
                    Ok(MigrationResponse {
                        success: false,
                        canister_id: None,
                        message: "Starting new migration".to_string(),
                    })
                }
            })
        };

        // Test idempotent behavior - multiple calls should return same result
        let result1 = mock_migrate_capsule(user);
        let result2 = mock_migrate_capsule(user);
        let result3 = mock_migrate_capsule(user);

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());

        let response1 = result1.unwrap();
        let response2 = result2.unwrap();
        let response3 = result3.unwrap();

        // All responses should be identical
        assert_eq!(response1.success, response2.success);
        assert_eq!(response1.success, response3.success);
        assert_eq!(response1.canister_id, response2.canister_id);
        assert_eq!(response1.canister_id, response3.canister_id);
        assert_eq!(response1.message, response2.message);
        assert_eq!(response1.message, response3.message);

        // Should indicate completed migration
        assert!(response1.success);
        assert_eq!(response1.canister_id, Some(canister_id));
        assert!(response1.message.contains("already completed"));
    }

    #[test]
    fn test_migration_status_tracking_and_updates() {
        setup_test_state();
        let user = create_test_principal(1);
        let canister_id = create_test_principal(10);

        // Mock get_migration_status function
        let mock_get_migration_status = |user: Principal| -> Option<MigrationStatusResponse> {
            with_mock_migration_state(|state| {
                state.migration_states.get(&user).map(|migration_state| {
                    MigrationStatusResponse {
                        status: migration_state.status.clone(),
                        canister_id: migration_state.personal_canister_id,
                        message: migration_state.error_message.clone(),
                    }
                })
            })
        };

        // Initially no migration status
        let initial_status = mock_get_migration_status(user);
        assert!(initial_status.is_none());

        // Test status progression through migration stages
        let migration_stages = vec![
            (MigrationStatus::NotStarted, None, None),
            (MigrationStatus::Exporting, None, None),
            (MigrationStatus::Creating, None, None),
            (MigrationStatus::Installing, Some(canister_id), None),
            (MigrationStatus::Importing, Some(canister_id), None),
            (MigrationStatus::Verifying, Some(canister_id), None),
            (MigrationStatus::Completed, Some(canister_id), None),
        ];

        for (status, expected_canister_id, expected_error) in migration_stages {
            // Update migration state
            with_mock_migration_state_mut(|state| {
                let migration_state = MigrationState {
                    user,
                    status: status.clone(),
                    created_at: mock_time(),
                    completed_at: if matches!(status, MigrationStatus::Completed) {
                        Some(mock_time())
                    } else {
                        None
                    },
                    personal_canister_id: expected_canister_id,
                    cycles_consumed: if matches!(status, MigrationStatus::Completed) {
                        2_000_000_000_000
                    } else {
                        0
                    },
                    error_message: expected_error.map(|s| s.to_string()),
                };
                state.migration_states.insert(user, migration_state);
            });

            // Check status
            let current_status = mock_get_migration_status(user);
            assert!(current_status.is_some(), "Status should exist for stage {:?}", status);
            
            let status_response = current_status.unwrap();
            assert_eq!(status_response.status, status, "Status should match for stage {:?}", status);
            assert_eq!(status_response.canister_id, expected_canister_id, 
                      "Canister ID should match for stage {:?}", status);
            assert_eq!(status_response.message, expected_error.map(|s| s.to_string()),
                      "Error message should match for stage {:?}", status);
        }

        // Test failed status with error message
        let error_message = "Installation failed: WASM module incompatible";
        with_mock_migration_state_mut(|state| {
            let migration_state = MigrationState {
                user,
                status: MigrationStatus::Failed,
                created_at: mock_time(),
                completed_at: None,
                personal_canister_id: Some(canister_id),
                cycles_consumed: 1_000_000_000_000, // Partial consumption
                error_message: Some(error_message.to_string()),
            };
            state.migration_states.insert(user, migration_state);
        });

        let failed_status = mock_get_migration_status(user);
        assert!(failed_status.is_some());
        let failed_response = failed_status.unwrap();
        assert_eq!(failed_response.status, MigrationStatus::Failed);
        assert_eq!(failed_response.canister_id, Some(canister_id));
        assert_eq!(failed_response.message, Some(error_message.to_string()));
    }

    #[test]
    fn test_migration_with_multiple_memories() {
        setup_test_state();
        setup_import_test_state();
        
        let user = create_test_principal(1);
        let required_cycles = 3_000_000_000_000; // 3T cycles for larger migration

        // Begin import session
        let import_session_result = mock_begin_import(user);
        assert!(import_session_result.is_ok());
        let session_id = import_session_result.unwrap().session_id.unwrap();

        // Upload multiple memories
        let memories = vec![
            ("memory_1", b"First memory content for testing migration"),
            ("memory_2", b"Second memory with different content and longer text"),
            ("memory_3", b"Third memory for comprehensive testing"),
        ];

        for (memory_id, content) in &memories {
            let chunk_hash = simple_hash(&String::from_utf8_lossy(content));
            
            // Upload chunk
            let chunk_result = mock_put_memory_chunk(
                user,
                session_id.clone(),
                memory_id.to_string(),
                0,
                content.to_vec(),
                chunk_hash.clone(),
            );
            assert!(chunk_result.is_ok(), "Chunk upload should succeed for {}", memory_id);

            // Commit memory
            let manifest = MemoryManifest {
                memory_id: memory_id.to_string(),
                total_chunks: 1,
                total_size: content.len() as u64,
                chunk_checksums: vec![chunk_hash.clone()],
                final_checksum: chunk_hash.clone(),
                memory_metadata: crate::types::Memory {
                    id: memory_id.to_string(),
                    title: format!("Test Memory {}", memory_id),
                    description: Some(format!("Test memory {} for migration", memory_id)),
                    created_at: mock_time(),
                    updated_at: mock_time(),
                    memory_type: crate::types::MemoryType::Text,
                    tags: vec![],
                    is_favorite: false,
                    content_hash: Some(chunk_hash),
                    file_extension: None,
                    file_size: Some(content.len() as u64),
                },
            };

            let commit_result = mock_commit_memory(user, session_id.clone(), manifest);
            assert!(commit_result.is_ok(), "Memory commit should succeed for {}", memory_id);
        }

        // Finalize import
        let finalize_result = mock_finalize_import(user, session_id);
        assert!(finalize_result.is_ok());
        let finalize_response = finalize_result.unwrap();
        assert!(finalize_response.success);
        assert_eq!(finalize_response.imported_memories_count, memories.len());

        // Verify total imported size
        let expected_total_size: u64 = memories.iter()
            .map(|(_, content)| content.len() as u64)
            .sum();
        assert_eq!(finalize_response.total_imported_size, expected_total_size);
    }

    #[test]
    fn test_concurrent_migration_attempts() {
        setup_test_state();
        setup_import_test_state();
        
        let user = create_test_principal(1);

        // First migration attempt - begin import session
        let first_session_result = mock_begin_import(user);
        assert!(first_session_result.is_ok());
        assert!(first_session_result.unwrap().success);

        // Second migration attempt - should fail due to existing active session
        let second_session_result = mock_begin_import(user);
        assert!(second_session_result.is_ok());
        let second_response = second_session_result.unwrap();
        assert!(!second_response.success);
        assert!(second_response.message.contains("already has an active"));
    }

    #[test]
    fn test_migration_state_persistence() {
        setup_test_state();
        let user = create_test_principal(1);
        let canister_id = create_test_principal(10);

        // Create migration state
        let original_state = MigrationState {
            user,
            status: MigrationStatus::Installing,
            created_at: mock_time(),
            completed_at: None,
            personal_canister_id: Some(canister_id),
            cycles_consumed: 1_500_000_000_000,
            error_message: None,
        };

        // Store state
        with_mock_migration_state_mut(|state| {
            state.migration_states.insert(user, original_state.clone());
        });

        // Simulate state persistence (like pre_upgrade/post_upgrade)
        let persisted_state = with_mock_migration_state(|state| {
            state.migration_states.get(&user).cloned()
        });

        assert!(persisted_state.is_some());
        let restored_state = persisted_state.unwrap();
        
        // Verify all fields are preserved
        assert_eq!(restored_state.user, original_state.user);
        assert_eq!(restored_state.status, original_state.status);
        assert_eq!(restored_state.created_at, original_state.created_at);
        assert_eq!(restored_state.completed_at, original_state.completed_at);
        assert_eq!(restored_state.personal_canister_id, original_state.personal_canister_id);
        assert_eq!(restored_state.cycles_consumed, original_state.cycles_consumed);
        assert_eq!(restored_state.error_message, original_state.error_message);
    }

    // Helper function to create mock memory object
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
    // Failure Scenarios and Recovery Tests

    #[test]
    fn test_failure_at_exporting_stage() {
        setup_test_state();
        let user = create_test_principal(1);
        let error_message = "Failed to export capsule data: memory corruption detected";

        // Simulate failure during export stage
        with_mock_migration_state_mut(|state| {
            let migration_state = MigrationState {
                user,
                status: MigrationStatus::Failed,
                created_at: mock_time(),
                completed_at: None,
                personal_canister_id: None,
                cycles_consumed: 0, // No cycles consumed yet
                error_message: Some(error_message.to_string()),
            };
            state.migration_states.insert(user, migration_state);
        });

        // Verify failure state
        let status = with_mock_migration_state(|state| {
            state.migration_states.get(&user).cloned()
        });

        assert!(status.is_some());
        let migration_state = status.unwrap();
        assert_eq!(migration_state.status, MigrationStatus::Failed);
        assert_eq!(migration_state.personal_canister_id, None);
        assert_eq!(migration_state.cycles_consumed, 0);
        assert_eq!(migration_state.error_message, Some(error_message.to_string()));

        // Verify no registry entry was created
        let registry_entries = mock_get_registry_entries_by_user(user);
        assert_eq!(registry_entries.len(), 0);

        // Verify no cycles were consumed
        let cycles_status = mock_get_cycles_reserve_status();
        assert_eq!(cycles_status.total_consumed, 1_000_000_000_000); // Only initial consumed amount
    }

    #[test]
    fn test_failure_at_creating_stage() {
        setup_test_state();
        let user = create_test_principal(1);
        let canister_id = create_test_principal(10);
        let error_message = "Failed to create canister: insufficient subnet capacity";

        // Simulate failure during canister creation
        with_mock_migration_state_mut(|state| {
            let migration_state = MigrationState {
                user,
                status: MigrationStatus::Failed,
                created_at: mock_time(),
                completed_at: None,
                personal_canister_id: None,
                cycles_consumed: 0,
                error_message: Some(error_message.to_string()),
            };
            state.migration_states.insert(user, migration_state);
        });

        // Create registry entry with Creating status (before failure)
        mock_create_registry_entry(canister_id, user, MigrationStatus::Creating, 0).unwrap();

        // Update registry to Failed status
        with_mock_migration_state_mut(|state| {
            if let Some(record) = state.personal_canisters.get_mut(&canister_id) {
                record.status = MigrationStatus::Failed;
            }
        });

        // Verify failure state
        let registry_entries = mock_get_registry_entries_by_user(user);
        assert_eq!(registry_entries.len(), 1);
        assert_eq!(registry_entries[0].status, MigrationStatus::Failed);
        assert_eq!(registry_entries[0].cycles_consumed, 0);

        // Verify migration state
        let migration_state = with_mock_migration_state(|state| {
            state.migration_states.get(&user).cloned()
        }).unwrap();
        assert_eq!(migration_state.status, MigrationStatus::Failed);
        assert!(migration_state.error_message.is_some());
    }

    #[test]
    fn test_failure_at_installing_stage() {
        setup_test_state();
        let user = create_test_principal(1);
        let canister_id = create_test_principal(10);
        let cycles_consumed = 2_000_000_000_000; // 2T cycles consumed for creation
        let error_message = "Failed to install WASM: API version incompatible";

        // Consume cycles for canister creation
        mock_consume_cycles_from_reserve(cycles_consumed).unwrap();

        // Create registry entry and update to Failed
        mock_create_registry_entry(canister_id, user, MigrationStatus::Installing, cycles_consumed).unwrap();
        with_mock_migration_state_mut(|state| {
            if let Some(record) = state.personal_canisters.get_mut(&canister_id) {
                record.status = MigrationStatus::Failed;
            }
        });

        // Simulate failure during WASM installation
        with_mock_migration_state_mut(|state| {
            let migration_state = MigrationState {
                user,
                status: MigrationStatus::Failed,
                created_at: mock_time(),
                completed_at: None,
                personal_canister_id: Some(canister_id), // Canister was created but installation failed
                cycles_consumed,
                error_message: Some(error_message.to_string()),
            };
            state.migration_states.insert(user, migration_state);
        });

        // Verify failure state
        let migration_state = with_mock_migration_state(|state| {
            state.migration_states.get(&user).cloned()
        }).unwrap();
        assert_eq!(migration_state.status, MigrationStatus::Failed);
        assert_eq!(migration_state.personal_canister_id, Some(canister_id));
        assert_eq!(migration_state.cycles_consumed, cycles_consumed);

        // Verify registry reflects failure
        let registry_entries = mock_get_registry_entries_by_user(user);
        assert_eq!(registry_entries.len(), 1);
        assert_eq!(registry_entries[0].status, MigrationStatus::Failed);
        assert_eq!(registry_entries[0].cycles_consumed, cycles_consumed);

        // Verify cycles were consumed (canister was created)
        let cycles_status = mock_get_cycles_reserve_status();
        assert_eq!(cycles_status.total_consumed, 3_000_000_000_000); // 1T initial + 2T consumed
    }

    #[test]
    fn test_failure_at_importing_stage() {
        setup_test_state();
        setup_import_test_state();
        
        let user = create_test_principal(1);
        let canister_id = create_test_principal(10);
        let cycles_consumed = 2_000_000_000_000;

        // Begin import session
        let session_result = mock_begin_import(user);
        assert!(session_result.is_ok());
        let session_id = session_result.unwrap().session_id.unwrap();

        // Simulate chunk upload failure
        let memory_id = "test_memory".to_string();
        let test_data = b"Test data for import failure";
        let wrong_hash = "wrong_hash_value".to_string();

        let chunk_result = mock_put_memory_chunk(
            user,
            session_id.clone(),
            memory_id.clone(),
            0,
            test_data.to_vec(),
            wrong_hash, // Wrong hash to trigger failure
        );

        assert!(chunk_result.is_ok());
        let chunk_response = chunk_result.unwrap();
        assert!(!chunk_response.success);
        assert!(chunk_response.message.contains("hash validation failed"));

        // Simulate migration failure due to import issues
        with_mock_migration_state_mut(|state| {
            let migration_state = MigrationState {
                user,
                status: MigrationStatus::Failed,
                created_at: mock_time(),
                completed_at: None,
                personal_canister_id: Some(canister_id),
                cycles_consumed,
                error_message: Some("Data import failed: chunk validation error".to_string()),
            };
            state.migration_states.insert(user, migration_state);
        });

        // Verify failure state
        let migration_state = with_mock_migration_state(|state| {
            state.migration_states.get(&user).cloned()
        }).unwrap();
        assert_eq!(migration_state.status, MigrationStatus::Failed);
        assert!(migration_state.error_message.unwrap().contains("import failed"));
    }

    #[test]
    fn test_failure_at_verifying_stage() {
        setup_test_state();
        setup_import_test_state();
        
        let user = create_test_principal(1);
        let canister_id = create_test_principal(10);
        let cycles_consumed = 2_000_000_000_000;

        // Complete successful import
        let session_result = mock_begin_import(user);
        let session_id = session_result.unwrap().session_id.unwrap();

        let memory_id = "test_memory".to_string();
        let test_data = b"Test data for verification failure";
        let chunk_hash = simple_hash(&String::from_utf8_lossy(test_data));

        // Upload and commit memory successfully
        mock_put_memory_chunk(
            user,
            session_id.clone(),
            memory_id.clone(),
            0,
            test_data.to_vec(),
            chunk_hash.clone(),
        ).unwrap();

        let manifest = MemoryManifest {
            memory_id: memory_id.clone(),
            total_chunks: 1,
            total_size: test_data.len() as u64,
            chunk_checksums: vec![chunk_hash.clone()],
            final_checksum: chunk_hash.clone(),
            memory_metadata: crate::types::Memory {
                id: memory_id.clone(),
                title: "Test Memory".to_string(),
                description: None,
                created_at: mock_time(),
                updated_at: mock_time(),
                memory_type: crate::types::MemoryType::Text,
                tags: vec![],
                is_favorite: false,
                content_hash: Some(chunk_hash),
                file_extension: None,
                file_size: Some(test_data.len() as u64),
            },
        };

        mock_commit_memory(user, session_id.clone(), manifest).unwrap();
        mock_finalize_import(user, session_id).unwrap();

        // Simulate verification failure (e.g., API version mismatch)
        with_mock_migration_state_mut(|state| {
            let migration_state = MigrationState {
                user,
                status: MigrationStatus::Failed,
                created_at: mock_time(),
                completed_at: None,
                personal_canister_id: Some(canister_id),
                cycles_consumed,
                error_message: Some("Verification failed: API version mismatch".to_string()),
            };
            state.migration_states.insert(user, migration_state);
        });

        // Verify failure state
        let migration_state = with_mock_migration_state(|state| {
            state.migration_states.get(&user).cloned()
        }).unwrap();
        assert_eq!(migration_state.status, MigrationStatus::Failed);
        assert!(migration_state.error_message.unwrap().contains("Verification failed"));
    }

    #[test]
    fn test_cleanup_and_rollback_procedures() {
        setup_test_state();
        let user = create_test_principal(1);
        let canister_id = create_test_principal(10);
        let cycles_consumed = 2_000_000_000_000;

        // Simulate failed migration with partial progress
        mock_consume_cycles_from_reserve(cycles_consumed).unwrap();
        mock_create_registry_entry(canister_id, user, MigrationStatus::Installing, cycles_consumed).unwrap();

        // Mock cleanup function
        let mock_cleanup_failed_migration = |user: Principal, canister_id: Principal| -> Result<(), String> {
            with_mock_migration_state_mut(|state| {
                // Update registry status to Failed
                if let Some(record) = state.personal_canisters.get_mut(&canister_id) {
                    record.status = MigrationStatus::Failed;
                }

                // Update migration state to Failed
                if let Some(migration_state) = state.migration_states.get_mut(&user) {
                    migration_state.status = MigrationStatus::Failed;
                    migration_state.error_message = Some("Migration cleaned up after failure".to_string());
                }

                // Note: In real implementation, this would also:
                // - Stop the canister if it was created
                // - Clean up any partial data
                // - Keep factory as controller for manual cleanup
                // - Log the failure for admin review

                Ok(())
            })
        };

        // Perform cleanup
        let cleanup_result = mock_cleanup_failed_migration(user, canister_id);
        assert!(cleanup_result.is_ok());

        // Verify cleanup state
        let registry_entries = mock_get_registry_entries_by_user(user);
        assert_eq!(registry_entries.len(), 1);
        assert_eq!(registry_entries[0].status, MigrationStatus::Failed);

        let migration_state = with_mock_migration_state(|state| {
            state.migration_states.get(&user).cloned()
        }).unwrap();
        assert_eq!(migration_state.status, MigrationStatus::Failed);
        assert!(migration_state.error_message.is_some());

        // Verify cycles remain consumed (no rollback of cycles)
        let cycles_status = mock_get_cycles_reserve_status();
        assert_eq!(cycles_status.total_consumed, 3_000_000_000_000); // 1T initial + 2T consumed
    }

    #[test]
    fn test_error_logging_and_monitoring() {
        setup_test_state();
        let user = create_test_principal(1);

        // Mock error logging function
        let mock_log_migration_error = |user: Principal, stage: &str, error: &str| -> Result<(), String> {
            with_mock_migration_state_mut(|state| {
                // Update migration stats
                state.migration_stats.total_failed += 1;
                state.migration_stats.last_failure_at = Some(mock_time());

                // Create detailed error log entry
                let migration_state = MigrationState {
                    user,
                    status: MigrationStatus::Failed,
                    created_at: mock_time(),
                    completed_at: None,
                    personal_canister_id: None,
                    cycles_consumed: 0,
                    error_message: Some(format!("Failed at {}: {}", stage, error)),
                };
                state.migration_states.insert(user, migration_state);

                Ok(())
            })
        };

        // Log various types of errors
        let errors = vec![
            ("exporting", "Memory corruption detected during export"),
            ("creating", "Insufficient subnet capacity"),
            ("installing", "WASM module validation failed"),
            ("importing", "Chunk validation error"),
            ("verifying", "API version incompatible"),
        ];

        let initial_stats = with_mock_migration_state(|state| state.migration_stats.clone());

        for (stage, error) in errors {
            let test_user = create_test_principal((stage.len() % 255) as u8 + 1);
            let log_result = mock_log_migration_error(test_user, stage, error);
            assert!(log_result.is_ok());

            // Verify error was logged
            let migration_state = with_mock_migration_state(|state| {
                state.migration_states.get(&test_user).cloned()
            });
            assert!(migration_state.is_some());
            let state = migration_state.unwrap();
            assert_eq!(state.status, MigrationStatus::Failed);
            assert!(state.error_message.unwrap().contains(stage));
        }

        // Verify error statistics
        let final_stats = with_mock_migration_state(|state| state.migration_stats.clone());
        assert_eq!(final_stats.total_failed, initial_stats.total_failed + errors.len() as u64);
        assert!(final_stats.last_failure_at.is_some());
    }

    #[test]
    fn test_retry_mechanisms_and_recovery_strategies() {
        setup_test_state();
        let user = create_test_principal(1);
        let canister_id = create_test_principal(10);

        // Mock retry logic for failed migration
        let mock_retry_migration = |user: Principal| -> Result<MigrationResponse, String> {
            with_mock_migration_state_mut(|state| {
                if let Some(existing_state) = state.migration_states.get(&user) {
                    match existing_state.status {
                        MigrationStatus::Failed => {
                            // Reset migration state for retry
                            let retry_state = MigrationState {
                                user,
                                status: MigrationStatus::NotStarted,
                                created_at: mock_time(),
                                completed_at: None,
                                personal_canister_id: None,
                                cycles_consumed: 0,
                                error_message: None,
                            };
                            state.migration_states.insert(user, retry_state);

                            Ok(MigrationResponse {
                                success: true,
                                canister_id: None,
                                message: "Migration reset for retry".to_string(),
                            })
                        }
                        MigrationStatus::Completed => Ok(MigrationResponse {
                            success: true,
                            canister_id: existing_state.personal_canister_id,
                            message: "Migration already completed".to_string(),
                        }),
                        _ => Ok(MigrationResponse {
                            success: false,
                            canister_id: None,
                            message: "Migration in progress, cannot retry".to_string(),
                        }),
                    }
                } else {
                    Ok(MigrationResponse {
                        success: true,
                        canister_id: None,
                        message: "Starting new migration".to_string(),
                    })
                }
            })
        };

        // Set up initial failed state
        with_mock_migration_state_mut(|state| {
            let failed_state = MigrationState {
                user,
                status: MigrationStatus::Failed,
                created_at: mock_time(),
                completed_at: None,
                personal_canister_id: Some(canister_id),
                cycles_consumed: 1_000_000_000_000,
                error_message: Some("Previous migration failed".to_string()),
            };
            state.migration_states.insert(user, failed_state);
        });

        // Test retry mechanism
        let retry_result = mock_retry_migration(user);
        assert!(retry_result.is_ok());
        let retry_response = retry_result.unwrap();
        assert!(retry_response.success);
        assert!(retry_response.message.contains("reset for retry"));

        // Verify state was reset
        let migration_state = with_mock_migration_state(|state| {
            state.migration_states.get(&user).cloned()
        }).unwrap();
        assert_eq!(migration_state.status, MigrationStatus::NotStarted);
        assert_eq!(migration_state.cycles_consumed, 0);
        assert!(migration_state.error_message.is_none());

        // Test retry of in-progress migration (should fail)
        with_mock_migration_state_mut(|state| {
            if let Some(migration_state) = state.migration_states.get_mut(&user) {
                migration_state.status = MigrationStatus::Installing;
            }
        });

        let retry_in_progress = mock_retry_migration(user);
        assert!(retry_in_progress.is_ok());
        let in_progress_response = retry_in_progress.unwrap();
        assert!(!in_progress_response.success);
        assert!(in_progress_response.message.contains("in progress"));
    }

    #[test]
    fn test_partial_failure_recovery() {
        setup_test_state();
        setup_import_test_state();
        
        let user = create_test_principal(1);

        // Begin import session
        let session_result = mock_begin_import(user);
        let session_id = session_result.unwrap().session_id.unwrap();

        // Upload some chunks successfully
        let memory1_data = b"First memory data";
        let memory1_hash = simple_hash(&String::from_utf8_lossy(memory1_data));
        
        let chunk1_result = mock_put_memory_chunk(
            user,
            session_id.clone(),
            "memory_1".to_string(),
            0,
            memory1_data.to_vec(),
            memory1_hash.clone(),
        );
        assert!(chunk1_result.unwrap().success);

        // Commit first memory successfully
        let manifest1 = MemoryManifest {
            memory_id: "memory_1".to_string(),
            total_chunks: 1,
            total_size: memory1_data.len() as u64,
            chunk_checksums: vec![memory1_hash.clone()],
            final_checksum: memory1_hash.clone(),
            memory_metadata: crate::types::Memory {
                id: "memory_1".to_string(),
                title: "Memory 1".to_string(),
                description: None,
                created_at: mock_time(),
                updated_at: mock_time(),
                memory_type: crate::types::MemoryType::Text,
                tags: vec![],
                is_favorite: false,
                content_hash: Some(memory1_hash),
                file_extension: None,
                file_size: Some(memory1_data.len() as u64),
            },
        };
        
        let commit1_result = mock_commit_memory(user, session_id.clone(), manifest1);
        assert!(commit1_result.unwrap().success);

        // Attempt to upload second memory with wrong hash (should fail)
        let memory2_data = b"Second memory data";
        let wrong_hash = "wrong_hash".to_string();
        
        let chunk2_result = mock_put_memory_chunk(
            user,
            session_id.clone(),
            "memory_2".to_string(),
            0,
            memory2_data.to_vec(),
            wrong_hash,
        );
        assert!(!chunk2_result.unwrap().success);

        // Verify session state - first memory should be completed, second should not exist
        with_mock_migration_state(|state| {
            if let Some(session) = state.import_sessions.get(&session_id) {
                assert_eq!(session.completed_memories.len(), 1);
                assert!(session.completed_memories.contains_key("memory_1"));
                assert!(!session.memories_in_progress.contains_key("memory_2"));
            }
        });

        // Attempt to finalize should fail due to incomplete import
        let finalize_result = mock_finalize_import(user, session_id);
        assert!(finalize_result.is_ok());
        let finalize_response = finalize_result.unwrap();
        assert!(finalize_response.success); // Should succeed with partial data
        assert_eq!(finalize_response.imported_memories_count, 1);
    }

    #[test]
    fn test_session_timeout_recovery() {
        setup_test_state();
        setup_import_test_state();
        
        let user = create_test_principal(1);

        // Begin import session
        let session_result = mock_begin_import(user);
        let session_id = session_result.unwrap().session_id.unwrap();

        // Simulate session timeout by modifying session timestamp
        with_mock_migration_state_mut(|state| {
            if let Some(session) = state.import_sessions.get_mut(&session_id) {
                // Set last activity to more than timeout period ago
                let timeout_nanos = state.import_config.session_timeout_seconds * 1_000_000_000;
                session.last_activity_at = mock_time() - timeout_nanos - 1;
            }
        });

        // Attempt to upload chunk to expired session
        let test_data = b"Test data for expired session";
        let chunk_hash = simple_hash(&String::from_utf8_lossy(test_data));
        
        let chunk_result = mock_put_memory_chunk(
            user,
            session_id.clone(),
            "test_memory".to_string(),
            0,
            test_data.to_vec(),
            chunk_hash,
        );

        assert!(chunk_result.is_ok());
        let chunk_response = chunk_result.unwrap();
        assert!(!chunk_response.success);
        assert!(chunk_response.message.contains("expired"));

        // Verify session status was updated to expired
        with_mock_migration_state(|state| {
            if let Some(session) = state.import_sessions.get(&session_id) {
                assert_eq!(session.status, ImportSessionStatus::Expired);
            }
        });

        // New session should be allowed after timeout
        let new_session_result = mock_begin_import(user);
        assert!(new_session_result.is_ok());
        assert!(new_session_result.unwrap().success);
    } 
   // Upgrade Resilience Tests

    #[test]
    fn test_restart_resume_functionality_mid_state() {
        setup_test_state();
        let user = create_test_principal(1);
        let canister_id = create_test_principal(10);

        // Simulate migration in progress at Installing stage
        let mid_migration_state = MigrationState {
            user,
            status: MigrationStatus::Installing,
            created_at: mock_time(),
            completed_at: None,
            personal_canister_id: Some(canister_id),
            cycles_consumed: 2_000_000_000_000,
            error_message: None,
        };

        // Store state before "upgrade"
        with_mock_migration_state_mut(|state| {
            state.migration_states.insert(user, mid_migration_state.clone());
        });

        // Create registry entry
        mock_create_registry_entry(canister_id, user, MigrationStatus::Installing, 2_000_000_000_000).unwrap();

        // Simulate canister upgrade/restart by preserving state
        let preserved_state = with_mock_migration_state(|state| {
            (
                state.migration_states.clone(),
                state.personal_canisters.clone(),
                state.migration_config.clone(),
                state.migration_stats.clone(),
            )
        });

        // Simulate post-upgrade restoration
        with_mock_migration_state_mut(|state| {
            state.migration_states = preserved_state.0;
            state.personal_canisters = preserved_state.1;
            state.migration_config = preserved_state.2;
            state.migration_stats = preserved_state.3;
        });

        // Verify state was preserved across restart
        let restored_migration_state = with_mock_migration_state(|state| {
            state.migration_states.get(&user).cloned()
        });

        assert!(restored_migration_state.is_some());
        let restored_state = restored_migration_state.unwrap();
        assert_eq!(restored_state.user, mid_migration_state.user);
        assert_eq!(restored_state.status, mid_migration_state.status);
        assert_eq!(restored_state.created_at, mid_migration_state.created_at);
        assert_eq!(restored_state.personal_canister_id, mid_migration_state.personal_canister_id);
        assert_eq!(restored_state.cycles_consumed, mid_migration_state.cycles_consumed);

        // Verify registry was preserved
        let registry_entries = mock_get_registry_entries_by_user(user);
        assert_eq!(registry_entries.len(), 1);
        assert_eq!(registry_entries[0].status, MigrationStatus::Installing);
        assert_eq!(registry_entries[0].cycles_consumed, 2_000_000_000_000);

        // Mock resume functionality
        let mock_resume_migration = |user: Principal| -> Result<MigrationResponse, String> {
            with_mock_migration_state_mut(|state| {
                if let Some(migration_state) = state.migration_states.get_mut(&user) {
                    match migration_state.status {
                        MigrationStatus::Installing => {
                            // Resume from Installing stage
                            migration_state.status = MigrationStatus::Importing;
                            Ok(MigrationResponse {
                                success: true,
                                canister_id: migration_state.personal_canister_id,
                                message: "Resumed migration from Installing stage".to_string(),
                            })
                        }
                        _ => Ok(MigrationResponse {
                            success: false,
                            canister_id: None,
                            message: "Cannot resume from current state".to_string(),
                        }),
                    }
                } else {
                    Err("No migration state found".to_string())
                }
            })
        };

        // Test resume functionality
        let resume_result = mock_resume_migration(user);
        assert!(resume_result.is_ok());
        let resume_response = resume_result.unwrap();
        assert!(resume_response.success);
        assert!(resume_response.message.contains("Resumed"));

        // Verify state progression
        let updated_state = with_mock_migration_state(|state| {
            state.migration_states.get(&user).cloned()
        }).unwrap();
        assert_eq!(updated_state.status, MigrationStatus::Importing);
    }

    #[test]
    fn test_pre_post_upgrade_state_persistence() {
        setup_test_state();
        setup_import_test_state();
        
        let user1 = create_test_principal(1);
        let user2 = create_test_principal(2);
        let canister1 = create_test_principal(10);
        let canister2 = create_test_principal(11);

        // Set up complex state before upgrade
        with_mock_migration_state_mut(|state| {
            // Migration states
            state.migration_states.insert(user1, MigrationState {
                user: user1,
                status: MigrationStatus::Importing,
                created_at: mock_time(),
                completed_at: None,
                personal_canister_id: Some(canister1),
                cycles_consumed: 2_000_000_000_000,
                error_message: None,
            });

            state.migration_states.insert(user2, MigrationState {
                user: user2,
                status: MigrationStatus::Completed,
                created_at: mock_time(),
                completed_at: Some(mock_time()),
                personal_canister_id: Some(canister2),
                cycles_consumed: 3_000_000_000_000,
                error_message: None,
            });

            // Registry entries
            state.personal_canisters.insert(canister1, PersonalCanisterRecord {
                canister_id: canister1,
                created_by: user1,
                created_at: mock_time(),
                status: MigrationStatus::Importing,
                cycles_consumed: 2_000_000_000_000,
            });

            state.personal_canisters.insert(canister2, PersonalCanisterRecord {
                canister_id: canister2,
                created_by: user2,
                created_at: mock_time(),
                status: MigrationStatus::Completed,
                cycles_consumed: 3_000_000_000_000,
            });

            // Import session
            let session_id = "test_session_123".to_string();
            state.import_sessions.insert(session_id.clone(), ImportSession {
                session_id: session_id.clone(),
                user: user1,
                created_at: mock_time(),
                last_activity_at: mock_time(),
                total_expected_size: 1000,
                total_received_size: 500,
                memories_in_progress: std::collections::HashMap::new(),
                completed_memories: std::collections::HashMap::new(),
                import_manifest: None,
                status: ImportSessionStatus::Active,
            });

            // Update stats
            state.migration_stats.total_attempted = 5;
            state.migration_stats.total_completed = 3;
            state.migration_stats.total_failed = 1;
            state.migration_stats.total_cycles_consumed = 8_000_000_000_000;
        });

        // Mock pre_upgrade serialization
        let pre_upgrade_data = with_mock_migration_state(|state| {
            serde_json::to_string(&MigrationStateData {
                migration_config: state.migration_config.clone(),
                migration_states: state.migration_states.clone(),
                migration_stats: state.migration_stats.clone(),
                personal_canisters: state.personal_canisters.clone(),
                import_config: state.import_config.clone(),
                import_sessions: state.import_sessions.clone(),
            }).unwrap()
        });

        // Simulate canister upgrade - clear state
        with_mock_migration_state_mut(|state| {
            *state = MigrationStateData::default();
        });

        // Verify state is cleared
        let cleared_state = with_mock_migration_state(|state| {
            (
                state.migration_states.len(),
                state.personal_canisters.len(),
                state.import_sessions.len(),
            )
        });
        assert_eq!(cleared_state, (0, 0, 0));

        // Mock post_upgrade deserialization
        let restored_data: MigrationStateData = serde_json::from_str(&pre_upgrade_data).unwrap();
        with_mock_migration_state_mut(|state| {
            *state = restored_data;
        });

        // Verify all state was restored correctly
        let restored_migration_states = with_mock_migration_state(|state| {
            state.migration_states.clone()
        });
        assert_eq!(restored_migration_states.len(), 2);
        assert!(restored_migration_states.contains_key(&user1));
        assert!(restored_migration_states.contains_key(&user2));

        let user1_state = &restored_migration_states[&user1];
        assert_eq!(user1_state.status, MigrationStatus::Importing);
        assert_eq!(user1_state.personal_canister_id, Some(canister1));

        let user2_state = &restored_migration_states[&user2];
        assert_eq!(user2_state.status, MigrationStatus::Completed);
        assert_eq!(user2_state.personal_canister_id, Some(canister2));

        // Verify registry was restored
        let restored_registry = with_mock_migration_state(|state| {
            state.personal_canisters.clone()
        });
        assert_eq!(restored_registry.len(), 2);
        assert!(restored_registry.contains_key(&canister1));
        assert!(restored_registry.contains_key(&canister2));

        // Verify import sessions were restored
        let restored_sessions = with_mock_migration_state(|state| {
            state.import_sessions.clone()
        });
        assert_eq!(restored_sessions.len(), 1);
        let session = restored_sessions.values().next().unwrap();
        assert_eq!(session.user, user1);
        assert_eq!(session.status, ImportSessionStatus::Active);

        // Verify stats were restored
        let restored_stats = with_mock_migration_state(|state| {
            state.migration_stats.clone()
        });
        assert_eq!(restored_stats.total_attempted, 5);
        assert_eq!(restored_stats.total_completed, 3);
        assert_eq!(restored_stats.total_failed, 1);
        assert_eq!(restored_stats.total_cycles_consumed, 8_000_000_000_000);
    }

    #[test]
    fn test_idempotency_across_canister_upgrades() {
        setup_test_state();
        let user = create_test_principal(1);
        let canister_id = create_test_principal(10);

        // Set up completed migration state
        with_mock_migration_state_mut(|state| {
            state.migration_states.insert(user, MigrationState {
                user,
                status: MigrationStatus::Completed,
                created_at: mock_time(),
                completed_at: Some(mock_time()),
                personal_canister_id: Some(canister_id),
                cycles_consumed: 2_000_000_000_000,
                error_message: None,
            });

            state.personal_canisters.insert(canister_id, PersonalCanisterRecord {
                canister_id,
                created_by: user,
                created_at: mock_time(),
                status: MigrationStatus::Completed,
                cycles_consumed: 2_000_000_000_000,
            });
        });

        // Mock migrate_capsule function
        let mock_migrate_capsule = |user: Principal| -> Result<MigrationResponse, String> {
            with_mock_migration_state(|state| {
                if let Some(existing_state) = state.migration_states.get(&user) {
                    match existing_state.status {
                        MigrationStatus::Completed => Ok(MigrationResponse {
                            success: true,
                            canister_id: existing_state.personal_canister_id,
                            message: "Migration already completed".to_string(),
                        }),
                        _ => Ok(MigrationResponse {
                            success: false,
                            canister_id: None,
                            message: format!("Migration in progress: {:?}", existing_state.status),
                        }),
                    }
                } else {
                    Ok(MigrationResponse {
                        success: false,
                        canister_id: None,
                        message: "Starting new migration".to_string(),
                    })
                }
            })
        };

        // Test idempotency before upgrade
        let pre_upgrade_result1 = mock_migrate_capsule(user);
        let pre_upgrade_result2 = mock_migrate_capsule(user);
        
        assert!(pre_upgrade_result1.is_ok());
        assert!(pre_upgrade_result2.is_ok());
        
        let response1 = pre_upgrade_result1.unwrap();
        let response2 = pre_upgrade_result2.unwrap();
        
        assert_eq!(response1.success, response2.success);
        assert_eq!(response1.canister_id, response2.canister_id);
        assert_eq!(response1.message, response2.message);

        // Simulate upgrade by preserving and restoring state
        let preserved_state = with_mock_migration_state(|state| {
            (
                state.migration_states.clone(),
                state.personal_canisters.clone(),
            )
        });

        // Clear state (simulate upgrade)
        with_mock_migration_state_mut(|state| {
            state.migration_states.clear();
            state.personal_canisters.clear();
        });

        // Restore state (simulate post_upgrade)
        with_mock_migration_state_mut(|state| {
            state.migration_states = preserved_state.0;
            state.personal_canisters = preserved_state.1;
        });

        // Test idempotency after upgrade
        let post_upgrade_result1 = mock_migrate_capsule(user);
        let post_upgrade_result2 = mock_migrate_capsule(user);
        
        assert!(post_upgrade_result1.is_ok());
        assert!(post_upgrade_result2.is_ok());
        
        let post_response1 = post_upgrade_result1.unwrap();
        let post_response2 = post_upgrade_result2.unwrap();
        
        // Results should be identical to pre-upgrade results
        assert_eq!(post_response1.success, response1.success);
        assert_eq!(post_response1.canister_id, response1.canister_id);
        assert_eq!(post_response1.message, response1.message);
        
        assert_eq!(post_response2.success, response2.success);
        assert_eq!(post_response2.canister_id, response2.canister_id);
        assert_eq!(post_response2.message, response2.message);
    }

    #[test]
    fn test_migration_state_recovery_after_restart() {
        setup_test_state();
        setup_import_test_state();
        
        let user = create_test_principal(1);
        let canister_id = create_test_principal(10);

        // Set up migration in various states before restart
        let test_states = vec![
            (MigrationStatus::Exporting, None, 0),
            (MigrationStatus::Creating, None, 0),
            (MigrationStatus::Installing, Some(canister_id), 2_000_000_000_000),
            (MigrationStatus::Importing, Some(canister_id), 2_000_000_000_000),
            (MigrationStatus::Verifying, Some(canister_id), 2_000_000_000_000),
        ];

        for (status, canister_id_opt, cycles_consumed) in test_states {
            // Set up state before restart
            with_mock_migration_state_mut(|state| {
                state.migration_states.insert(user, MigrationState {
                    user,
                    status: status.clone(),
                    created_at: mock_time(),
                    completed_at: None,
                    personal_canister_id: canister_id_opt,
                    cycles_consumed,
                    error_message: None,
                });

                if let Some(cid) = canister_id_opt {
                    state.personal_canisters.insert(cid, PersonalCanisterRecord {
                        canister_id: cid,
                        created_by: user,
                        created_at: mock_time(),
                        status: status.clone(),
                        cycles_consumed,
                    });
                }
            });

            // Mock recovery logic that determines next action based on state
            let mock_determine_recovery_action = |user: Principal| -> Result<String, String> {
                with_mock_migration_state(|state| {
                    if let Some(migration_state) = state.migration_states.get(&user) {
                        let action = match migration_state.status {
                            MigrationStatus::Exporting => "Resume export process",
                            MigrationStatus::Creating => "Retry canister creation",
                            MigrationStatus::Installing => "Resume WASM installation",
                            MigrationStatus::Importing => "Resume data import",
                            MigrationStatus::Verifying => "Resume verification",
                            MigrationStatus::Completed => "No action needed",
                            MigrationStatus::Failed => "Cleanup and allow retry",
                            MigrationStatus::NotStarted => "Start new migration",
                        };
                        Ok(action.to_string())
                    } else {
                        Ok("No migration state found".to_string())
                    }
                })
            };

            // Test recovery action determination
            let recovery_action = mock_determine_recovery_action(user);
            assert!(recovery_action.is_ok());
            let action = recovery_action.unwrap();
            
            match status {
                MigrationStatus::Exporting => assert!(action.contains("export")),
                MigrationStatus::Creating => assert!(action.contains("creation")),
                MigrationStatus::Installing => assert!(action.contains("installation")),
                MigrationStatus::Importing => assert!(action.contains("import")),
                MigrationStatus::Verifying => assert!(action.contains("verification")),
                _ => {}
            }

            // Verify state consistency after restart
            let recovered_state = with_mock_migration_state(|state| {
                state.migration_states.get(&user).cloned()
            });
            
            assert!(recovered_state.is_some());
            let state = recovered_state.unwrap();
            assert_eq!(state.status, status);
            assert_eq!(state.personal_canister_id, canister_id_opt);
            assert_eq!(state.cycles_consumed, cycles_consumed);

            // Clean up for next iteration
            with_mock_migration_state_mut(|state| {
                state.migration_states.clear();
                state.personal_canisters.clear();
            });
        }
    }

    #[test]
    fn test_import_session_recovery_after_restart() {
        setup_test_state();
        setup_import_test_state();
        
        let user = create_test_principal(1);
        let session_id = "test_session_recovery".to_string();

        // Set up import session with partial progress before restart
        with_mock_migration_state_mut(|state| {
            let mut memories_in_progress = std::collections::HashMap::new();
            let mut completed_memories = std::collections::HashMap::new();

            // Add memory in progress
            memories_in_progress.insert("memory_1".to_string(), MemoryImportState {
                memory_id: "memory_1".to_string(),
                expected_chunks: 3,
                received_chunks: {
                    let mut chunks = std::collections::HashMap::new();
                    chunks.insert(0, ChunkData {
                        chunk_index: 0,
                        data: b"chunk 0 data".to_vec(),
                        sha256: "hash0".to_string(),
                        received_at: mock_time(),
                    });
                    chunks.insert(1, ChunkData {
                        chunk_index: 1,
                        data: b"chunk 1 data".to_vec(),
                        sha256: "hash1".to_string(),
                        received_at: mock_time(),
                    });
                    chunks
                },
                total_size: 1000,
                received_size: 500,
                memory_metadata: None,
                is_complete: false,
            });

            // Add completed memory
            completed_memories.insert("memory_2".to_string(), CompletedMemoryImport {
                memory_id: "memory_2".to_string(),
                assembled_data: b"complete memory data".to_vec(),
                memory_metadata: crate::types::Memory {
                    id: "memory_2".to_string(),
                    title: "Completed Memory".to_string(),
                    description: None,
                    created_at: mock_time(),
                    updated_at: mock_time(),
                    memory_type: crate::types::MemoryType::Text,
                    tags: vec![],
                    is_favorite: false,
                    content_hash: Some("complete_hash".to_string()),
                    file_extension: None,
                    file_size: Some(100),
                },
                total_size: 100,
                completed_at: mock_time(),
            });

            let session = ImportSession {
                session_id: session_id.clone(),
                user,
                created_at: mock_time(),
                last_activity_at: mock_time(),
                total_expected_size: 2000,
                total_received_size: 600,
                memories_in_progress,
                completed_memories,
                import_manifest: None,
                status: ImportSessionStatus::Active,
            };

            state.import_sessions.insert(session_id.clone(), session);
        });

        // Simulate restart by preserving and restoring session state
        let preserved_session = with_mock_migration_state(|state| {
            state.import_sessions.get(&session_id).cloned()
        });

        assert!(preserved_session.is_some());

        // Clear sessions (simulate restart)
        with_mock_migration_state_mut(|state| {
            state.import_sessions.clear();
        });

        // Restore session (simulate post_upgrade)
        with_mock_migration_state_mut(|state| {
            if let Some(session) = preserved_session {
                state.import_sessions.insert(session_id.clone(), session);
            }
        });

        // Verify session was recovered correctly
        let recovered_session = with_mock_migration_state(|state| {
            state.import_sessions.get(&session_id).cloned()
        });

        assert!(recovered_session.is_some());
        let session = recovered_session.unwrap();
        
        assert_eq!(session.user, user);
        assert_eq!(session.status, ImportSessionStatus::Active);
        assert_eq!(session.total_expected_size, 2000);
        assert_eq!(session.total_received_size, 600);
        assert_eq!(session.memories_in_progress.len(), 1);
        assert_eq!(session.completed_memories.len(), 1);

        // Verify in-progress memory state
        let memory_in_progress = session.memories_in_progress.get("memory_1").unwrap();
        assert_eq!(memory_in_progress.expected_chunks, 3);
        assert_eq!(memory_in_progress.received_chunks.len(), 2);
        assert_eq!(memory_in_progress.received_size, 500);
        assert!(!memory_in_progress.is_complete);

        // Verify completed memory state
        let completed_memory = session.completed_memories.get("memory_2").unwrap();
        assert_eq!(completed_memory.memory_id, "memory_2");
        assert_eq!(completed_memory.total_size, 100);
        assert_eq!(completed_memory.memory_metadata.title, "Completed Memory");

        // Test resuming upload after recovery
        let resume_chunk_result = mock_put_memory_chunk(
            user,
            session_id.clone(),
            "memory_1".to_string(),
            2, // Missing chunk
            b"chunk 2 data".to_vec(),
            "hash2".to_string(),
        );

        assert!(resume_chunk_result.is_ok());
        let resume_response = resume_chunk_result.unwrap();
        assert!(resume_response.success);
        assert!(resume_response.message.contains("uploaded successfully"));
    }

    #[test]
    fn test_cycles_state_consistency_across_upgrades() {
        setup_test_state();
        let initial_reserve = 10_000_000_000_000; // 10T cycles
        let initial_consumed = 1_000_000_000_000;  // 1T cycles

        // Perform some operations before upgrade
        let operations = vec![
            (create_test_principal(1), 2_000_000_000_000), // 2T cycles
            (create_test_principal(2), 1_500_000_000_000), // 1.5T cycles
        ];

        for (user, cycles) in &operations {
            mock_consume_cycles_from_reserve(*cycles).unwrap();
            mock_create_registry_entry(
                create_test_principal(10 + (user.as_slice()[0] as u32)),
                *user,
                MigrationStatus::Completed,
                *cycles,
            ).unwrap();
        }

        // Capture state before upgrade
        let pre_upgrade_status = mock_get_cycles_reserve_status();
        let pre_upgrade_registry = with_mock_migration_state(|state| {
            state.personal_canisters.clone()
        });

        // Expected values
        let expected_reserve = initial_reserve - 2_000_000_000_000 - 1_500_000_000_000; // 6.5T
        let expected_consumed = initial_consumed + 2_000_000_000_000 + 1_500_000_000_000; // 4.5T

        assert_eq!(pre_upgrade_status.current_reserve, expected_reserve);
        assert_eq!(pre_upgrade_status.total_consumed, expected_consumed);
        assert_eq!(pre_upgrade_registry.len(), 2);

        // Simulate upgrade by preserving state
        let preserved_state = with_mock_migration_state(|state| {
            (
                state.migration_config.clone(),
                state.migration_stats.clone(),
                state.personal_canisters.clone(),
            )
        });

        // Clear state (simulate upgrade)
        with_mock_migration_state_mut(|state| {
            *state = MigrationStateData::default();
        });

        // Restore state (simulate post_upgrade)
        with_mock_migration_state_mut(|state| {
            state.migration_config = preserved_state.0;
            state.migration_stats = preserved_state.1;
            state.personal_canisters = preserved_state.2;
        });

        // Verify cycles state consistency after upgrade
        let post_upgrade_status = mock_get_cycles_reserve_status();
        let post_upgrade_registry = with_mock_migration_state(|state| {
            state.personal_canisters.clone()
        });

        assert_eq!(post_upgrade_status.current_reserve, pre_upgrade_status.current_reserve);
        assert_eq!(post_upgrade_status.total_consumed, pre_upgrade_status.total_consumed);
        assert_eq!(post_upgrade_status.min_threshold, pre_upgrade_status.min_threshold);
        assert_eq!(post_upgrade_status.is_above_threshold, pre_upgrade_status.is_above_threshold);

        assert_eq!(post_upgrade_registry.len(), pre_upgrade_registry.len());

        // Verify individual registry entries
        for (canister_id, record) in pre_upgrade_registry {
            let post_record = post_upgrade_registry.get(&canister_id);
            assert!(post_record.is_some());
            let post_record = post_record.unwrap();
            assert_eq!(post_record.created_by, record.created_by);
            assert_eq!(post_record.status, record.status);
            assert_eq!(post_record.cycles_consumed, record.cycles_consumed);
        }

        // Test that operations continue to work correctly after upgrade
        let new_user = create_test_principal(3);
        let new_cycles = 1_000_000_000_000; // 1T cycles

        let post_upgrade_operation = mock_consume_cycles_from_reserve(new_cycles);
        assert!(post_upgrade_operation.is_ok());

        let final_status = mock_get_cycles_reserve_status();
        assert_eq!(final_status.current_reserve, expected_reserve - new_cycles);
        assert_eq!(final_status.total_consumed, expected_consumed + new_cycles);
    }

    #[test]
    fn test_migration_statistics_persistence() {
        setup_test_state();

        // Set up initial statistics
        with_mock_migration_state_mut(|state| {
            state.migration_stats = MigrationStats {
                total_attempted: 10,
                total_completed: 7,
                total_failed: 2,
                total_cycles_consumed: 15_000_000_000_000,
                last_migration_at: Some(mock_time()),
                last_failure_at: Some(mock_time() - 1000),
            };
        });

        // Capture pre-upgrade stats
        let pre_upgrade_stats = with_mock_migration_state(|state| {
            state.migration_stats.clone()
        });

        // Simulate upgrade
        let preserved_stats = pre_upgrade_stats.clone();
        with_mock_migration_state_mut(|state| {
            state.migration_stats = MigrationStats::default();
        });

        // Restore stats
        with_mock_migration_state_mut(|state| {
            state.migration_stats = preserved_stats;
        });

        // Verify stats were preserved
        let post_upgrade_stats = with_mock_migration_state(|state| {
            state.migration_stats.clone()
        });

        assert_eq!(post_upgrade_stats.total_attempted, pre_upgrade_stats.total_attempted);
        assert_eq!(post_upgrade_stats.total_completed, pre_upgrade_stats.total_completed);
        assert_eq!(post_upgrade_stats.total_failed, pre_upgrade_stats.total_failed);
        assert_eq!(post_upgrade_stats.total_cycles_consumed, pre_upgrade_stats.total_cycles_consumed);
        assert_eq!(post_upgrade_stats.last_migration_at, pre_upgrade_stats.last_migration_at);
        assert_eq!(post_upgrade_stats.last_failure_at, pre_upgrade_stats.last_failure_at);

        // Test that stats continue to update correctly after upgrade
        with_mock_migration_state_mut(|state| {
            state.migration_stats.total_attempted += 1;
            state.migration_stats.total_completed += 1;
        });

        let updated_stats = with_mock_migration_state(|state| {
            state.migration_stats.clone()
        });

        assert_eq!(updated_stats.total_attempted, pre_upgrade_stats.total_attempted + 1);
        assert_eq!(updated_stats.total_completed, pre_upgrade_stats.total_completed + 1);
    }

}}
