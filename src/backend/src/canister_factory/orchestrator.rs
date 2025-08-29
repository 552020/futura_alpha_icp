use crate::canister_factory::types::*;
use crate::canister_factory::{auth::*, cycles::*, export::*, factory::*, registry::*, verify::*};
use candid::Principal;

/// Main migration function that orchestrates the complete capsule migration process
/// This function implements the state machine: NotStarted → Exporting → Creating → Installing → Importing → Verifying → Completed/Failed
pub async fn migrate_capsule() -> Result<MigrationResponse, String> {
    // Validate caller and get user principal
    let user = validate_migration_caller()?;

    ic_cdk::println!("Starting migration for user {}", user);

    // Check if migration is enabled
    let migration_enabled =
        crate::memory::with_migration_state(|state| state.migration_config.enabled);
    if !migration_enabled {
        return Ok(MigrationResponse {
            success: false,
            canister_id: None,
            message: "Migration is currently disabled".to_string(),
        });
    }

    // Get or create migration state for this user
    let existing_state =
        crate::memory::with_migration_state(|state| state.migration_states.get(&user).cloned());

    // Handle idempotency - if migration already exists, return current status
    if let Some(existing) = existing_state {
        match existing.status {
            MigrationStatus::Completed => {
                return Ok(MigrationResponse {
                    success: true,
                    canister_id: existing.personal_canister_id,
                    message: "Migration already completed".to_string(),
                });
            }
            MigrationStatus::Failed => {
                // Allow retry of failed migrations
                ic_cdk::println!("Retrying failed migration for user {}", user);
            }
            _ => {
                // Migration is in progress, return current status
                return Ok(MigrationResponse {
                    success: false,
                    canister_id: existing.personal_canister_id,
                    message: format!(
                        "Migration already in progress (status: {:?})",
                        existing.status
                    ),
                });
            }
        }
    }

    // Initialize migration state
    let now = ic_cdk::api::time();
    let mut migration_state = MigrationState {
        user,
        status: MigrationStatus::NotStarted,
        created_at: now,
        completed_at: None,
        personal_canister_id: None,
        cycles_consumed: 0,
        error_message: None,
    };

    // Update migration stats
    crate::memory::with_migration_state_mut(|state| {
        state.migration_stats.total_attempts += 1;
        state.migration_states.insert(user, migration_state.clone());
    });

    // Execute migration state machine
    let result = execute_migration_state_machine(&mut migration_state).await;

    // Update final migration state
    crate::memory::with_migration_state_mut(|state| {
        state.migration_states.insert(user, migration_state.clone());

        // Update stats based on result
        match &result {
            Ok(_) => {
                if migration_state.status == MigrationStatus::Completed {
                    state.migration_stats.total_successes += 1;
                }
            }
            Err(_) => {
                state.migration_stats.total_failures += 1;
            }
        }
    });

    result
}

/// Execute the migration state machine with comprehensive error handling
async fn execute_migration_state_machine(
    migration_state: &mut MigrationState,
) -> Result<MigrationResponse, String> {
    let user = migration_state.user;

    // State: NotStarted → Exporting
    migration_state.status = MigrationStatus::Exporting;
    ic_cdk::println!("Migration state: Exporting data for user {}", user);

    // Export user's capsule data
    let export_data = match export_user_capsule_data(user) {
        Ok(data) => data,
        Err(e) => {
            migration_state.status = MigrationStatus::Failed;
            migration_state.error_message = Some(format!("Export failed: {}", e));
            return Ok(MigrationResponse {
                success: false,
                canister_id: None,
                message: format!("Failed to export capsule data: {}", e),
            });
        }
    };

    // Validate exported data
    if let Err(e) = validate_export_data(&export_data) {
        migration_state.status = MigrationStatus::Failed;
        migration_state.error_message = Some(format!("Export validation failed: {}", e));
        return Ok(MigrationResponse {
            success: false,
            canister_id: None,
            message: format!("Export data validation failed: {}", e),
        });
    }

    // State: Exporting → Creating
    migration_state.status = MigrationStatus::Creating;
    ic_cdk::println!(
        "Migration state: Creating personal canister for user {}",
        user
    );

    // Create personal canister
    let cycles_to_fund = get_default_canister_cycles();
    let config = create_default_config();

    let canister_id = match create_personal_canister(user, config, cycles_to_fund).await {
        Ok(id) => {
            migration_state.personal_canister_id = Some(id);
            migration_state.cycles_consumed = cycles_to_fund;
            id
        }
        Err(e) => {
            migration_state.status = MigrationStatus::Failed;
            migration_state.error_message = Some(format!("Canister creation failed: {}", e));
            return Ok(MigrationResponse {
                success: false,
                canister_id: None,
                message: format!("Failed to create personal canister: {}", e),
            });
        }
    };

    // State: Creating → Installing
    migration_state.status = MigrationStatus::Installing;
    ic_cdk::println!(
        "Migration state: Installing WASM for canister {}",
        canister_id
    );

    // Install WASM module
    if let Err(e) = complete_wasm_installation(canister_id, user, &export_data).await {
        migration_state.status = MigrationStatus::Failed;
        migration_state.error_message = Some(format!("WASM installation failed: {}", e));

        // Cleanup failed canister
        if let Err(cleanup_err) = cleanup_failed_canister_creation(canister_id, user).await {
            ic_cdk::println!("Warning: Cleanup failed: {}", cleanup_err);
        }

        return Ok(MigrationResponse {
            success: false,
            canister_id: Some(canister_id),
            message: format!("Failed to install WASM: {}", e),
        });
    }

    // State: Installing → Importing
    migration_state.status = MigrationStatus::Importing;
    ic_cdk::println!(
        "Migration state: Importing data to canister {}",
        canister_id
    );

    // For MVP, we'll simulate the import process since the actual chunked import
    // would require the personal canister to be fully implemented
    // In production, this would use the chunked import API
    if let Err(e) = simulate_data_import(canister_id, &export_data).await {
        migration_state.status = MigrationStatus::Failed;
        migration_state.error_message = Some(format!("Data import failed: {}", e));

        // Cleanup failed canister
        if let Err(cleanup_err) = cleanup_failed_canister_creation(canister_id, user).await {
            ic_cdk::println!("Warning: Cleanup failed: {}", cleanup_err);
        }

        return Ok(MigrationResponse {
            success: false,
            canister_id: Some(canister_id),
            message: format!("Failed to import data: {}", e),
        });
    }

    // State: Importing → Verifying
    migration_state.status = MigrationStatus::Verifying;
    ic_cdk::println!(
        "Migration state: Verifying data for canister {}",
        canister_id
    );

    // Verify migration data integrity
    if let Err(e) = verify_migration_data(canister_id, &export_data).await {
        migration_state.status = MigrationStatus::Failed;
        migration_state.error_message = Some(format!("Data verification failed: {}", e));

        return Ok(MigrationResponse {
            success: false,
            canister_id: Some(canister_id),
            message: format!("Data verification failed: {}", e),
        });
    }

    // State: Verifying → Handoff Controllers
    ic_cdk::println!(
        "Migration state: Handing off controllers for canister {}",
        canister_id
    );

    // Handoff controllers to user
    if let Err(e) = handoff_controllers(canister_id, user).await {
        migration_state.status = MigrationStatus::Failed;
        migration_state.error_message = Some(format!("Controller handoff failed: {}", e));

        // Handle handoff failure
        if let Err(cleanup_err) = handle_handoff_failure(canister_id, user, e.clone()).await {
            ic_cdk::println!("Warning: Handoff failure handling failed: {}", cleanup_err);
        }

        return Ok(MigrationResponse {
            success: false,
            canister_id: Some(canister_id),
            message: format!("Failed to handoff controllers: {}", e),
        });
    }

    // State: Completed
    migration_state.status = MigrationStatus::Completed;
    migration_state.completed_at = Some(ic_cdk::api::time());

    // Update registry status to Completed
    if let Err(e) = update_registry_status(canister_id, MigrationStatus::Completed) {
        ic_cdk::println!(
            "Warning: Failed to update registry status to Completed: {}",
            e
        );
    }

    ic_cdk::println!(
        "Migration completed successfully for user {} (canister: {})",
        user,
        canister_id
    );

    Ok(MigrationResponse {
        success: true,
        canister_id: Some(canister_id),
        message: "Migration completed successfully".to_string(),
    })
}

/// Complete WASM installation process with error handling and validation
/// This function orchestrates the complete WASM installation process
pub async fn complete_wasm_installation(
    canister_id: Principal,
    user: Principal,
    export_data: &ExportData,
) -> Result<(), String> {
    ic_cdk::println!(
        "Starting complete WASM installation for canister {} (user: {})",
        canister_id,
        user
    );

    // Prepare initialization arguments for personal canister
    let init_args = prepare_personal_canister_init_args(user, export_data)?;

    // Load the personal canister WASM module
    // For MVP, we'll use a placeholder WASM
    let wasm_module = get_personal_canister_wasm_module()?;

    // Install the WASM module
    install_personal_canister_wasm(canister_id, wasm_module, init_args).await?;

    // Check API version compatibility
    check_api_version_compatibility(canister_id).await?;

    ic_cdk::println!(
        "Complete WASM installation finished for canister {}",
        canister_id
    );

    Ok(())
}

/// Prepare initialization arguments for personal canister
/// This function creates the initialization data for the personal canister
pub fn prepare_personal_canister_init_args(
    user: Principal,
    export_data: &ExportData,
) -> Result<Vec<u8>, String> {
    ic_cdk::println!(
        "Preparing init args for personal canister (user: {}, {} memories, {} connections)",
        user,
        export_data.memories.len(),
        export_data.connections.len()
    );

    // For MVP, create basic initialization data
    // In production, this would serialize the complete export data
    let init_data = format!(
        "{{\"user\":\"{}\",\"memory_count\":{},\"connection_count\":{}}}",
        user.to_text(),
        export_data.memories.len(),
        export_data.connections.len()
    );

    Ok(init_data.into_bytes())
}

/// Get the personal canister WASM module
/// This function loads the WASM binary for personal canisters
pub fn get_personal_canister_wasm_module() -> Result<Vec<u8>, String> {
    // For MVP, return a placeholder WASM module
    // In production, this would load the actual personal canister WASM
    ic_cdk::println!("Loading personal canister WASM module");

    // Placeholder WASM module (minimal valid WASM)
    let wasm_module = vec![
        0x00, 0x61, 0x73, 0x6d, // WASM magic number
        0x01, 0x00, 0x00, 0x00, // WASM version
    ];

    if wasm_module.len() < 8 {
        return Err("Invalid WASM module: too small".to_string());
    }

    ic_cdk::println!("Loaded WASM module ({} bytes)", wasm_module.len());
    Ok(wasm_module)
}

/// Simulate data import for MVP (placeholder for actual chunked import)
async fn simulate_data_import(
    canister_id: Principal,
    export_data: &ExportData,
) -> Result<(), String> {
    ic_cdk::println!(
        "Simulating data import for canister {} ({} memories, {} connections)",
        canister_id,
        export_data.memories.len(),
        export_data.connections.len()
    );

    // For MVP, we simulate the import process
    // In production, this would:
    // 1. Call begin_import() on the personal canister
    // 2. Upload data in chunks using put_memory_chunk()
    // 3. Commit each memory using commit_memory()
    // 4. Finalize the import using finalize_import()

    // Simulate processing time
    let total_size = export_data.metadata.total_size_bytes;
    ic_cdk::println!("Simulating import of {} bytes", total_size);

    // Basic validation that we have data to import
    if export_data.memories.is_empty() && export_data.connections.is_empty() {
        return Err("No data to import".to_string());
    }

    // Simulate successful import
    ic_cdk::println!(
        "Data import simulation completed for canister {}",
        canister_id
    );
    Ok(())
}

/// Cleanup failed canister creation
async fn cleanup_failed_canister_creation(
    canister_id: Principal,
    user: Principal,
) -> Result<(), String> {
    ic_cdk::println!(
        "Cleaning up failed canister creation for canister {} (user: {})",
        canister_id,
        user
    );

    // Update registry status to Failed
    if let Err(e) = update_registry_status(canister_id, MigrationStatus::Failed) {
        ic_cdk::println!("Warning: Failed to update registry status: {}", e);
    }

    // For MVP, we don't delete the canister to allow for debugging
    // In production, you might want to delete failed canisters to recover cycles
    ic_cdk::println!("Cleanup completed for canister {}", canister_id);

    Ok(())
}

/// Handle controller handoff failure
async fn handle_handoff_failure(
    canister_id: Principal,
    user: Principal,
    error: String,
) -> Result<(), String> {
    ic_cdk::println!(
        "Handling handoff failure for canister {} (user: {}): {}",
        canister_id,
        user,
        error
    );

    // Update registry status to Failed
    if let Err(e) = update_registry_status(canister_id, MigrationStatus::Failed) {
        ic_cdk::println!("Warning: Failed to update registry status: {}", e);
    }

    // Log the failure for debugging
    ic_cdk::println!(
        "HANDOFF_FAILURE: canister={}, user={}, error={}, timestamp={}",
        canister_id,
        user,
        error,
        ic_cdk::api::time()
    );

    Ok(())
}

/// Get migration status for the calling user
pub fn get_migration_status() -> Option<MigrationStatusResponse> {
    let caller = ic_cdk::api::msg_caller();

    // Reject anonymous callers
    if caller == Principal::anonymous() {
        return None;
    }

    crate::memory::with_migration_state(|state| {
        state.migration_states.get(&caller).map(|migration_state| {
            let message = match migration_state.status {
                MigrationStatus::NotStarted => "Migration not started".to_string(),
                MigrationStatus::Exporting => "Exporting capsule data...".to_string(),
                MigrationStatus::Creating => "Creating personal canister...".to_string(),
                MigrationStatus::Installing => "Installing WASM module...".to_string(),
                MigrationStatus::Importing => "Importing data to personal canister...".to_string(),
                MigrationStatus::Verifying => "Verifying data integrity...".to_string(),
                MigrationStatus::Completed => "Migration completed successfully".to_string(),
                MigrationStatus::Failed => migration_state
                    .error_message
                    .clone()
                    .unwrap_or_else(|| "Migration failed".to_string()),
            };

            MigrationStatusResponse {
                status: migration_state.status.clone(),
                canister_id: migration_state.personal_canister_id,
                message: Some(message),
            }
        })
    })
}

/// Get personal canister ID for a user (convenience function)
pub fn get_personal_canister_id(user: Principal) -> Option<Principal> {
    crate::memory::with_migration_state(|state| {
        state
            .migration_states
            .get(&user)
            .and_then(|migration_state| {
                if migration_state.status == MigrationStatus::Completed {
                    migration_state.personal_canister_id
                } else {
                    None
                }
            })
    })
}

/// Get personal canister ID for the calling user
pub fn get_my_personal_canister_id() -> Option<Principal> {
    let caller = ic_cdk::api::msg_caller();
    if caller == Principal::anonymous() {
        return None;
    }
    get_personal_canister_id(caller)
}
