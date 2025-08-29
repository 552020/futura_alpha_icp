use crate::canister_factory::types::*;
use crate::canister_factory::{cycles::*, registry::*};
use candid::Principal;
use ic_cdk::api::management_canister::main::{
    create_canister, install_code, update_settings, CanisterInstallMode, CanisterSettings,
    CreateCanisterArgument, InstallCodeArgument, UpdateSettingsArgument,
};

/// Create a personal canister with dual controllers (factory and user)
/// This function handles the complete canister creation process including:
/// - Preflight cycles check
/// - Canister creation with dual controllers
/// - Registry entry creation
/// - Cycles consumption tracking
pub async fn create_personal_canister(
    user: Principal,
    _config: CreatePersonalCanisterConfig,
    cycles_to_fund: u128,
) -> Result<Principal, String> {
    // Preflight check for cycles reserve
    preflight_cycles_reserve(cycles_to_fund)?;

    // Prepare canister settings with dual controllers
    let factory_principal = ic_cdk::api::canister_self();
    let controllers = vec![factory_principal, user];

    let canister_settings = CanisterSettings {
        controllers: Some(controllers),
        compute_allocation: None,
        memory_allocation: None,
        freezing_threshold: None,
        reserved_cycles_limit: None,
        log_visibility: None,
        wasm_memory_limit: None,
    };

    // Create canister creation arguments
    let create_args = CreateCanisterArgument {
        settings: Some(canister_settings),
    };

    ic_cdk::println!(
        "Creating personal canister for user {} with {} cycles",
        user,
        cycles_to_fund
    );

    // Create the canister with cycles funding
    let create_result = create_canister(create_args, cycles_to_fund).await;

    match create_result {
        Ok((canister_record,)) => {
            let canister_id = canister_record.canister_id;
            ic_cdk::println!(
                "Successfully created personal canister {} for user {}",
                canister_id,
                user
            );

            // Create registry entry with Creating status
            create_registry_entry(canister_id, user, MigrationStatus::Creating, cycles_to_fund)?;

            // Consume cycles from reserve
            consume_cycles_from_reserve(cycles_to_fund)?;

            // Log cycles consumption
            log_cycles_consumption(
                "create_canister",
                cycles_to_fund,
                Some(user),
                Some(canister_id),
            );

            Ok(canister_id)
        }
        Err((rejection_code, message)) => {
            let error_msg = format!(
                "Failed to create personal canister for user {}: {:?} - {}",
                user, rejection_code, message
            );

            ic_cdk::println!("{}", error_msg);

            // Don't consume cycles on failure
            Err(error_msg)
        }
    }
}

/// Install WASM module on a personal canister
/// This function handles WASM installation with proper error handling and validation
pub async fn install_personal_canister_wasm(
    canister_id: Principal,
    wasm_module: Vec<u8>,
    init_arg: Vec<u8>,
) -> Result<(), String> {
    ic_cdk::println!(
        "Installing WASM module on personal canister {} ({} bytes)",
        canister_id,
        wasm_module.len()
    );

    // Prepare installation arguments
    let install_args = InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id,
        wasm_module,
        arg: init_arg,
    };

    // Install the WASM module
    let install_result = install_code(install_args).await;

    match install_result {
        Ok(()) => {
            ic_cdk::println!(
                "Successfully installed WASM module on personal canister {}",
                canister_id
            );

            // Update registry status to Installing -> Installed (will be updated to Importing later)
            update_registry_status(canister_id, MigrationStatus::Installing)?;

            Ok(())
        }
        Err((rejection_code, message)) => {
            let error_msg = format!(
                "Failed to install WASM on personal canister {}: {:?} - {}",
                canister_id, rejection_code, message
            );

            ic_cdk::println!("{}", error_msg);

            // Update registry status to Failed
            update_registry_status(canister_id, MigrationStatus::Failed)?;

            Err(error_msg)
        }
    }
}

/// Handoff controllers from {factory, user} to {user} only
/// This function performs the controller transition after successful verification
pub async fn handoff_controllers(canister_id: Principal, user: Principal) -> Result<(), String> {
    ic_cdk::println!(
        "Starting controller handoff for canister {} to user {}",
        canister_id,
        user
    );

    // Verify preconditions before handoff
    verify_handoff_preconditions(canister_id, user).await?;

    // Perform the controller update
    let settings = CanisterSettings {
        controllers: Some(vec![user]),
        compute_allocation: None,
        memory_allocation: None,
        freezing_threshold: None,
        reserved_cycles_limit: None,
        log_visibility: None,
        wasm_memory_limit: None,
    };

    let update_args = UpdateSettingsArgument {
        canister_id,
        settings,
    };

    match update_settings(update_args).await {
        Ok(()) => {
            ic_cdk::println!(
                "Successfully handed off controllers for canister {} to user {}",
                canister_id,
                user
            );

            // Update registry status to reflect successful handoff
            update_registry_status(canister_id, MigrationStatus::Completed)?;

            // Log the successful handoff
            log_controller_handoff_success(canister_id, user);

            Ok(())
        }
        Err((rejection_code, msg)) => {
            let error_msg = format!(
                "Failed to update canister settings for handoff: {:?} - {}",
                rejection_code, msg
            );

            ic_cdk::println!("Controller handoff failed: {}", error_msg);

            // Update registry to reflect handoff failure
            update_registry_status(canister_id, MigrationStatus::Failed)?;

            Err(error_msg)
        }
    }
}

/// Verify preconditions before attempting controller handoff
async fn verify_handoff_preconditions(
    canister_id: Principal,
    user: Principal,
) -> Result<(), String> {
    ic_cdk::println!(
        "Verifying handoff preconditions for canister {} and user {}",
        canister_id,
        user
    );

    // Check that the registry entry exists and is in the right state
    let registry_entry = get_registry_entry(canister_id)
        .ok_or_else(|| format!("No registry entry found for canister {}", canister_id))?;

    // Verify the user matches the registry
    if registry_entry.created_by != user {
        return Err(format!(
            "User mismatch: registry shows canister {} was created by {}, but handoff requested for {}",
            canister_id, registry_entry.created_by, user
        ));
    }

    // Verify the canister is in a state ready for handoff
    match registry_entry.status {
        MigrationStatus::Verifying => {
            // This is the expected state for handoff
            ic_cdk::println!(
                "Canister {} is in Verifying state, ready for handoff",
                canister_id
            );
        }
        MigrationStatus::Completed => {
            // Already completed, this might be a retry
            ic_cdk::println!("Canister {} is already in Completed state", canister_id);
            return Ok(()); // Allow retry of completed handoff
        }
        other_status => {
            return Err(format!(
                "Canister {} is in {:?} state, not ready for handoff",
                canister_id, other_status
            ));
        }
    }

    Ok(())
}

/// Log successful controller handoff
fn log_controller_handoff_success(canister_id: Principal, user: Principal) {
    ic_cdk::println!(
        "CONTROLLER_HANDOFF_SUCCESS: canister={}, user={}, timestamp={}",
        canister_id,
        user,
        ic_cdk::api::time()
    );
}

/// Apply configuration defaults for personal canister creation
/// This function fills in default values for optional configuration fields
pub fn apply_config_defaults(config: CreatePersonalCanisterConfig) -> CreatePersonalCanisterConfig {
    let mut config_with_defaults = config;

    // Apply default name if not provided
    if config_with_defaults.name.is_none() {
        config_with_defaults.name = Some("Personal Capsule".to_string());
    }

    // subnet_id remains None by default (let IC choose)

    ic_cdk::println!(
        "Applied config defaults: name={:?}, subnet_id={:?}",
        config_with_defaults.name,
        config_with_defaults.subnet_id
    );

    config_with_defaults
}

/// Create a minimal default configuration
/// This function provides a sensible default configuration for personal canister creation
pub fn create_default_config() -> CreatePersonalCanisterConfig {
    CreatePersonalCanisterConfig {
        name: Some("Personal Capsule".to_string()),
        subnet_id: None, // Let IC choose the subnet
    }
}

/// Prepare canister configuration with validation and defaults
/// This is a convenience function that combines validation and default application
pub fn prepare_canister_config(
    config: Option<CreatePersonalCanisterConfig>,
) -> Result<CreatePersonalCanisterConfig, String> {
    // Use provided config or create default
    let config = config.unwrap_or_else(create_default_config);

    // Validate and apply defaults
    let validated_config = crate::canister_factory::auth::validate_and_prepare_config(config)?;
    let final_config = apply_config_defaults(validated_config);

    Ok(final_config)
}

/// Check for unsupported configuration options and log warnings
/// This function logs warnings for any unsupported options but doesn't fail
/// This allows for future expansion without breaking existing clients
pub fn check_unsupported_config_options(_config: &CreatePersonalCanisterConfig) -> Vec<String> {
    let warnings = Vec::new();

    // For MVP, all current options are supported
    // Future unsupported options would be detected here

    warnings
}
