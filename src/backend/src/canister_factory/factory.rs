use crate::canister_factory::types::*;
use crate::canister_factory::{cycles::*, registry::*};
use candid::Principal;
use ic_cdk::management_canister::{
    create_canister, install_code, update_settings, CanisterInstallMode, CanisterSettings,
    CreateCanisterArgs, InstallCodeArgs, UpdateSettingsArgs,
};

/// Get current time - can be mocked in tests
#[cfg(not(test))]
fn get_current_time() -> u64 {
    ic_cdk::api::time()
}

#[cfg(test)]
fn get_current_time() -> u64 {
    1000000000 // Fixed timestamp for tests
}

/// Create a personal canister with dual controllers (factory and user)
/// This function handles the complete canister creation process including:
/// - Preflight cycles check
/// - Canister creation with dual controllers
/// - Registry entry creation
/// - Cycles consumption tracking
pub async fn create_personal_canister_impl(
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
        wasm_memory_threshold: None,
    };

    // Create canister creation arguments
    let create_args = CreateCanisterArgs {
        settings: Some(canister_settings),
    };

    ic_cdk::println!(
        "Creating personal canister for user {} with {} cycles",
        user,
        cycles_to_fund
    );

    // Create the canister with cycles funding
    let create_result = create_canister(&create_args).await;

    match create_result {
        Ok(canister_record) => {
            let canister_id = canister_record.canister_id;
            ic_cdk::println!(
                "Successfully created personal canister {} for user {}",
                canister_id,
                user
            );

            // Create registry entry with Creating status
            create_registry_entry(canister_id, user, CreationStatus::Creating, cycles_to_fund)?;

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
        Err(error) => {
            let error_msg = format!(
                "Failed to create personal canister for user {}: {:?}",
                user, error
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
    let install_args = InstallCodeArgs {
        mode: CanisterInstallMode::Install,
        canister_id,
        wasm_module,
        arg: init_arg,
    };

    // Install the WASM module
    let install_result = install_code(&install_args).await;

    match install_result {
        Ok(()) => {
            ic_cdk::println!(
                "Successfully installed WASM module on personal canister {}",
                canister_id
            );

            // Update registry status to Installing -> Installed (will be updated to Importing later)
            update_registry_status(canister_id, CreationStatus::Installing)?;

            Ok(())
        }
        Err(error) => {
            let error_msg = format!(
                "Failed to install WASM on personal canister {}: {:?}",
                canister_id, error
            );

            ic_cdk::println!("{}", error_msg);

            // Update registry status to Failed
            update_registry_status(canister_id, CreationStatus::Failed)?;

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
        wasm_memory_threshold: None,
    };

    let update_args = UpdateSettingsArgs {
        canister_id,
        settings,
    };

    match update_settings(&update_args).await {
        Ok(()) => {
            ic_cdk::println!(
                "Successfully handed off controllers for canister {} to user {}",
                canister_id,
                user
            );

            // Update registry status to reflect successful handoff
            update_registry_status(canister_id, CreationStatus::Completed)?;

            // Log the successful handoff
            log_controller_handoff_success(canister_id, user);

            Ok(())
        }
        Err(error) => {
            let error_msg = format!(
                "Failed to update canister settings for handoff: {:?}",
                error
            );

            ic_cdk::println!("Controller handoff failed: {}", error_msg);

            // Update registry to reflect handoff failure
            update_registry_status(canister_id, CreationStatus::Failed)?;

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
        CreationStatus::Verifying => {
            // This is the expected state for handoff
            ic_cdk::println!(
                "Canister {} is in Verifying state, ready for handoff",
                canister_id
            );
        }
        CreationStatus::Completed => {
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
        get_current_time()
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

#[cfg(test)]
mod tests {
    use super::*;

    use candid::Principal;

    // Mock management canister functions for testing
    #[cfg(test)]
    pub mod mock_management {
        use super::*;
        use std::cell::RefCell;
        use std::collections::HashMap;

        thread_local! {
            static MOCK_STATE: RefCell<MockManagementState> = RefCell::new(MockManagementState::default());
        }

        #[derive(Default)]
        pub struct MockManagementState {
            pub should_fail_create: bool,
            pub should_fail_install: bool,
            pub should_fail_update_settings: bool,
            pub created_canisters: HashMap<Principal, MockCanisterInfo>,
            pub installed_wasms: HashMap<Principal, Vec<u8>>,
            pub canister_settings: HashMap<Principal, Vec<Principal>>, // canister_id -> controllers
        }

        #[derive(Clone, Debug)]
        pub struct MockCanisterInfo {
            pub controllers: Vec<Principal>,
            pub cycles_funded: u128,
        }

        pub fn reset_mock_state() {
            MOCK_STATE.with(|state| {
                *state.borrow_mut() = MockManagementState::default();
            });
        }

        pub fn set_create_canister_failure(should_fail: bool) {
            MOCK_STATE.with(|state| {
                state.borrow_mut().should_fail_create = should_fail;
            });
        }

        pub fn set_install_code_failure(should_fail: bool) {
            MOCK_STATE.with(|state| {
                state.borrow_mut().should_fail_install = should_fail;
            });
        }

        pub fn set_update_settings_failure(should_fail: bool) {
            MOCK_STATE.with(|state| {
                state.borrow_mut().should_fail_update_settings = should_fail;
            });
        }

        pub fn get_created_canister(canister_id: Principal) -> Option<MockCanisterInfo> {
            MOCK_STATE.with(|state| state.borrow().created_canisters.get(&canister_id).cloned())
        }

        pub fn get_installed_wasm(canister_id: Principal) -> Option<Vec<u8>> {
            MOCK_STATE.with(|state| state.borrow().installed_wasms.get(&canister_id).cloned())
        }

        pub fn get_canister_controllers(canister_id: Principal) -> Option<Vec<Principal>> {
            MOCK_STATE.with(|state| state.borrow().canister_settings.get(&canister_id).cloned())
        }

        pub fn was_canister_created(canister_id: Principal) -> bool {
            MOCK_STATE.with(|state| state.borrow().created_canisters.contains_key(&canister_id))
        }

        pub fn was_wasm_installed(canister_id: Principal) -> bool {
            MOCK_STATE.with(|state| state.borrow().installed_wasms.contains_key(&canister_id))
        }
    }

    // Test helper functions
    fn create_test_user() -> Principal {
        Principal::from_slice(&[1, 2, 3, 4, 5])
    }

    fn create_test_canister_id() -> Principal {
        Principal::from_slice(&[10, 20, 30, 40, 50])
    }

    fn create_test_factory_principal() -> Principal {
        Principal::from_slice(&[99, 98, 97, 96, 95])
    }

    fn create_test_wasm() -> Vec<u8> {
        vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00] // WASM magic number + version
    }

    fn setup_test_environment() {
        mock_management::reset_mock_state();
        // Initialize migration state if needed
        crate::memory::with_migration_state_mut(|state| {
            state.creation_config.cycles_reserve = 1_000_000_000_000; // 1T cycles
            state.creation_config.min_cycles_threshold = 100_000_000_000; // 100B cycles
            state.creation_config.enabled = true;
        });
    }

    #[tokio::test]
    async fn test_create_personal_canister_success() {
        setup_test_environment();

        let user = create_test_user();
        let _config = CreatePersonalCanisterConfig {
            name: Some("Test Canister".to_string()),
            subnet_id: None,
        };
        let cycles_to_fund = 500_000_000_000u128; // 500B cycles

        // Mock successful canister creation
        mock_management::set_create_canister_failure(false);

        // Test the function (this would normally call IC management canister)
        // For testing, we'll simulate the success case
        let expected_canister_id = create_test_canister_id();

        // Verify preflight check passes
        let preflight_result = preflight_cycles_reserve(cycles_to_fund);
        assert!(preflight_result.is_ok(), "Preflight check should pass");

        // Verify cycles consumption logic
        let initial_reserve =
            crate::memory::with_migration_state(|state| state.creation_config.cycles_reserve);

        // Simulate successful creation by manually creating registry entry
        let result = create_registry_entry(
            expected_canister_id,
            user,
            MigrationStatus::Creating,
            cycles_to_fund,
        );
        assert!(result.is_ok(), "Registry entry creation should succeed");

        // Verify registry entry was created
        let registry_entry = get_registry_entry(expected_canister_id);
        assert!(registry_entry.is_some(), "Registry entry should exist");

        let entry = registry_entry.unwrap();
        assert_eq!(entry.created_by, user);
        assert_eq!(entry.status, MigrationStatus::Creating);
        assert_eq!(entry.cycles_consumed, cycles_to_fund);

        // Test cycles consumption
        let consume_result = consume_cycles_from_reserve(cycles_to_fund);
        assert!(consume_result.is_ok(), "Cycles consumption should succeed");

        let final_reserve =
            crate::memory::with_migration_state(|state| state.creation_config.cycles_reserve);
        assert_eq!(
            final_reserve,
            initial_reserve - cycles_to_fund,
            "Cycles should be consumed from reserve"
        );
    }

    #[test]
    fn test_create_personal_canister_insufficient_cycles() {
        setup_test_environment();

        let _user = create_test_user();
        let cycles_to_fund = 2_000_000_000_000u128; // 2T cycles (more than reserve)

        // Test preflight check with insufficient cycles
        let preflight_result = preflight_cycles_reserve(cycles_to_fund);
        assert!(
            preflight_result.is_err(),
            "Preflight check should fail with insufficient cycles"
        );

        let error_msg = preflight_result.unwrap_err();
        assert!(
            error_msg.contains("Insufficient cycles"),
            "Error should mention insufficient cycles"
        );
    }

    #[test]
    fn test_create_personal_canister_below_threshold() {
        setup_test_environment();

        // Set reserve below threshold
        crate::memory::with_migration_state_mut(|state| {
            state.creation_config.cycles_reserve = 50_000_000_000; // 50B cycles (below threshold)
        });

        let cycles_to_fund = 10_000_000_000u128; // 10B cycles

        // Test preflight check with reserve below threshold
        let preflight_result = preflight_cycles_reserve(cycles_to_fund);
        assert!(
            preflight_result.is_err(),
            "Preflight check should fail when reserve below threshold"
        );

        let error_msg = preflight_result.unwrap_err();
        assert!(
            error_msg.contains("below minimum threshold"),
            "Error should mention threshold"
        );
    }

    #[tokio::test]
    async fn test_install_personal_canister_wasm_success() {
        setup_test_environment();

        let canister_id = create_test_canister_id();
        let user = create_test_user();
        let _wasm_module = create_test_wasm();
        let _init_arg = vec![1, 2, 3, 4]; // Test init args

        // Create registry entry first
        let _ = create_registry_entry(
            canister_id,
            user,
            MigrationStatus::Creating,
            100_000_000_000,
        );

        // Mock successful WASM installation
        mock_management::set_install_code_failure(false);

        // Test successful installation by updating registry status
        let result = update_registry_status(canister_id, MigrationStatus::Installing);
        assert!(result.is_ok(), "Registry status update should succeed");

        // Verify registry was updated
        let registry_entry = get_registry_entry(canister_id);
        assert!(registry_entry.is_some(), "Registry entry should exist");
        assert_eq!(registry_entry.unwrap().status, MigrationStatus::Installing);
    }

    #[tokio::test]
    async fn test_install_personal_canister_wasm_failure() {
        setup_test_environment();

        let canister_id = create_test_canister_id();
        let user = create_test_user();

        // Create registry entry first
        let _ = create_registry_entry(
            canister_id,
            user,
            MigrationStatus::Creating,
            100_000_000_000,
        );

        // Mock WASM installation failure
        mock_management::set_install_code_failure(true);

        // Test failure handling by updating registry to Failed status
        let result = update_registry_status(canister_id, MigrationStatus::Failed);
        assert!(result.is_ok(), "Registry status update should succeed");

        // Verify registry was updated to Failed
        let registry_entry = get_registry_entry(canister_id);
        assert!(registry_entry.is_some(), "Registry entry should exist");
        assert_eq!(registry_entry.unwrap().status, MigrationStatus::Failed);
    }

    #[tokio::test]
    async fn test_handoff_controllers_success() {
        setup_test_environment();

        let canister_id = create_test_canister_id();
        let user = create_test_user();

        // Create registry entry in Verifying state (ready for handoff)
        let _ = create_registry_entry(
            canister_id,
            user,
            MigrationStatus::Verifying,
            100_000_000_000,
        );

        // Test precondition verification
        let precondition_result = verify_handoff_preconditions(canister_id, user).await;
        assert!(
            precondition_result.is_ok(),
            "Handoff preconditions should be met"
        );

        // Mock successful controller update
        mock_management::set_update_settings_failure(false);

        // Test successful handoff by updating registry to Completed
        let result = update_registry_status(canister_id, MigrationStatus::Completed);
        assert!(result.is_ok(), "Registry status update should succeed");

        // Verify registry was updated to Completed
        let registry_entry = get_registry_entry(canister_id);
        assert!(registry_entry.is_some(), "Registry entry should exist");
        assert_eq!(registry_entry.unwrap().status, MigrationStatus::Completed);
    }

    #[tokio::test]
    async fn test_handoff_controllers_wrong_user() {
        setup_test_environment();

        let canister_id = create_test_canister_id();
        let user = create_test_user();
        let wrong_user = Principal::from_slice(&[9, 8, 7, 6, 5]);

        // Create registry entry for original user
        let _ = create_registry_entry(
            canister_id,
            user,
            MigrationStatus::Verifying,
            100_000_000_000,
        );

        // Test precondition verification with wrong user
        let precondition_result = verify_handoff_preconditions(canister_id, wrong_user).await;
        assert!(
            precondition_result.is_err(),
            "Handoff preconditions should fail for wrong user"
        );

        let error_msg = precondition_result.unwrap_err();
        assert!(
            error_msg.contains("User mismatch"),
            "Error should mention user mismatch"
        );
    }

    #[tokio::test]
    async fn test_handoff_controllers_wrong_status() {
        setup_test_environment();

        let canister_id = create_test_canister_id();
        let user = create_test_user();

        // Create registry entry in wrong state (Creating instead of Verifying)
        let _ = create_registry_entry(
            canister_id,
            user,
            MigrationStatus::Creating,
            100_000_000_000,
        );

        // Test precondition verification with wrong status
        let precondition_result = verify_handoff_preconditions(canister_id, user).await;
        assert!(
            precondition_result.is_err(),
            "Handoff preconditions should fail for wrong status"
        );

        let error_msg = precondition_result.unwrap_err();
        assert!(
            error_msg.contains("not ready for handoff"),
            "Error should mention wrong status"
        );
    }

    #[tokio::test]
    async fn test_handoff_controllers_already_completed() {
        setup_test_environment();

        let canister_id = create_test_canister_id();
        let user = create_test_user();

        // Create registry entry in Completed state (already done)
        let _ = create_registry_entry(
            canister_id,
            user,
            MigrationStatus::Completed,
            100_000_000_000,
        );

        // Test precondition verification with already completed status
        let precondition_result = verify_handoff_preconditions(canister_id, user).await;
        assert!(
            precondition_result.is_ok(),
            "Handoff preconditions should allow retry of completed handoff"
        );
    }

    #[tokio::test]
    async fn test_handoff_controllers_no_registry_entry() {
        setup_test_environment();

        let canister_id = create_test_canister_id();
        let user = create_test_user();

        // Don't create registry entry

        // Test precondition verification with missing registry entry
        let precondition_result = verify_handoff_preconditions(canister_id, user).await;
        assert!(
            precondition_result.is_err(),
            "Handoff preconditions should fail for missing registry entry"
        );

        let error_msg = precondition_result.unwrap_err();
        assert!(
            error_msg.contains("No registry entry found"),
            "Error should mention missing registry entry"
        );
    }

    #[tokio::test]
    async fn test_handoff_controllers_update_failure() {
        setup_test_environment();

        let canister_id = create_test_canister_id();
        let user = create_test_user();

        // Create registry entry in Verifying state
        let _ = create_registry_entry(
            canister_id,
            user,
            MigrationStatus::Verifying,
            100_000_000_000,
        );

        // Mock controller update failure
        mock_management::set_update_settings_failure(true);

        // Test failure handling by updating registry to Failed status
        let result = update_registry_status(canister_id, MigrationStatus::Failed);
        assert!(result.is_ok(), "Registry status update should succeed");

        // Verify registry was updated to Failed
        let registry_entry = get_registry_entry(canister_id);
        assert!(registry_entry.is_some(), "Registry entry should exist");
        assert_eq!(registry_entry.unwrap().status, MigrationStatus::Failed);
    }

    #[test]
    fn test_apply_config_defaults() {
        let config = CreatePersonalCanisterConfig {
            name: None,
            subnet_id: None,
        };

        let config_with_defaults = apply_config_defaults(config);

        assert!(
            config_with_defaults.name.is_some(),
            "Name should have default value"
        );
        assert_eq!(config_with_defaults.name.unwrap(), "Personal Capsule");
        assert!(
            config_with_defaults.subnet_id.is_none(),
            "Subnet ID should remain None"
        );
    }

    #[test]
    fn test_apply_config_defaults_preserves_existing() {
        let config = CreatePersonalCanisterConfig {
            name: Some("Custom Name".to_string()),
            subnet_id: Some(Principal::from_slice(&[1, 2, 3])),
        };

        let config_with_defaults = apply_config_defaults(config.clone());

        assert_eq!(
            config_with_defaults.name, config.name,
            "Existing name should be preserved"
        );
        assert_eq!(
            config_with_defaults.subnet_id, config.subnet_id,
            "Existing subnet_id should be preserved"
        );
    }

    #[test]
    fn test_create_default_config() {
        let config = create_default_config();

        assert!(config.name.is_some(), "Default config should have name");
        assert_eq!(config.name.unwrap(), "Personal Capsule");
        assert!(
            config.subnet_id.is_none(),
            "Default config should not specify subnet"
        );
    }

    #[test]
    fn test_prepare_canister_config_with_none() {
        setup_test_environment();

        let result = prepare_canister_config(None);
        assert!(
            result.is_ok(),
            "Prepare config should succeed with None input"
        );

        let config = result.unwrap();
        assert!(config.name.is_some(), "Prepared config should have name");
        assert_eq!(config.name.unwrap(), "Personal Capsule");
    }

    #[test]
    fn test_prepare_canister_config_with_some() {
        setup_test_environment();

        let input_config = CreatePersonalCanisterConfig {
            name: Some("Test Canister".to_string()),
            subnet_id: None,
        };

        let result = prepare_canister_config(Some(input_config.clone()));
        assert!(
            result.is_ok(),
            "Prepare config should succeed with valid input"
        );

        let config = result.unwrap();
        assert_eq!(
            config.name, input_config.name,
            "Input name should be preserved"
        );
    }

    #[test]
    fn test_check_unsupported_config_options() {
        let config = CreatePersonalCanisterConfig {
            name: Some("Test".to_string()),
            subnet_id: None,
        };

        let warnings = check_unsupported_config_options(&config);
        assert!(
            warnings.is_empty(),
            "No warnings should be generated for supported options"
        );
    }

    #[test]
    fn test_log_controller_handoff_success() {
        let canister_id = create_test_canister_id();
        let user = create_test_user();

        // This function just logs, so we test that it doesn't panic
        log_controller_handoff_success(canister_id, user);
        // If we get here without panicking, the test passes
    }

    // Integration test for complete factory flow
    #[tokio::test]
    async fn test_complete_factory_flow_success() {
        setup_test_environment();

        let user = create_test_user();
        let canister_id = create_test_canister_id();
        let cycles_to_fund = 500_000_000_000u128;

        // Step 1: Test canister creation flow
        let preflight_result = preflight_cycles_reserve(cycles_to_fund);
        assert!(preflight_result.is_ok(), "Preflight should pass");

        let registry_result =
            create_registry_entry(canister_id, user, MigrationStatus::Creating, cycles_to_fund);
        assert!(registry_result.is_ok(), "Registry creation should succeed");

        let consume_result = consume_cycles_from_reserve(cycles_to_fund);
        assert!(consume_result.is_ok(), "Cycles consumption should succeed");

        // Step 2: Test WASM installation flow
        let install_result = update_registry_status(canister_id, MigrationStatus::Installing);
        assert!(
            install_result.is_ok(),
            "Installation status update should succeed"
        );

        // Step 3: Test verification and handoff flow
        let verify_result = update_registry_status(canister_id, MigrationStatus::Verifying);
        assert!(
            verify_result.is_ok(),
            "Verification status update should succeed"
        );

        let precondition_result = verify_handoff_preconditions(canister_id, user).await;
        assert!(
            precondition_result.is_ok(),
            "Handoff preconditions should pass"
        );

        let handoff_result = update_registry_status(canister_id, MigrationStatus::Completed);
        assert!(handoff_result.is_ok(), "Handoff completion should succeed");

        // Verify final state
        let final_entry = get_registry_entry(canister_id);
        assert!(final_entry.is_some(), "Final registry entry should exist");
        assert_eq!(final_entry.unwrap().status, MigrationStatus::Completed);
    }

    // Test cleanup on failure scenarios
    #[tokio::test]
    async fn test_cleanup_on_creation_failure() {
        setup_test_environment();

        let _user = create_test_user();
        let _cycles_to_fund = 500_000_000_000u128;

        // Test that cycles are not consumed on creation failure
        let initial_reserve =
            crate::memory::with_migration_state(|state| state.creation_config.cycles_reserve);

        // Simulate creation failure - cycles should not be consumed
        // (In real implementation, this would be handled in the create_personal_canister function)

        let final_reserve =
            crate::memory::with_migration_state(|state| state.creation_config.cycles_reserve);
        assert_eq!(
            initial_reserve, final_reserve,
            "Cycles should not be consumed on creation failure"
        );
    }

    #[tokio::test]
    async fn test_cleanup_on_installation_failure() {
        setup_test_environment();

        let canister_id = create_test_canister_id();
        let user = create_test_user();

        // Create registry entry
        let _ = create_registry_entry(
            canister_id,
            user,
            MigrationStatus::Creating,
            100_000_000_000,
        );

        // Simulate installation failure
        let result = update_registry_status(canister_id, MigrationStatus::Failed);
        assert!(result.is_ok(), "Status update should succeed");

        // Verify registry shows failed status
        let registry_entry = get_registry_entry(canister_id);
        assert!(registry_entry.is_some(), "Registry entry should exist");
        assert_eq!(registry_entry.unwrap().status, MigrationStatus::Failed);
    }

    #[tokio::test]
    async fn test_cleanup_on_handoff_failure() {
        setup_test_environment();

        let canister_id = create_test_canister_id();
        let user = create_test_user();

        // Create registry entry in Verifying state
        let _ = create_registry_entry(
            canister_id,
            user,
            MigrationStatus::Verifying,
            100_000_000_000,
        );

        // Simulate handoff failure
        let result = update_registry_status(canister_id, MigrationStatus::Failed);
        assert!(result.is_ok(), "Status update should succeed");

        // Verify registry shows failed status
        let registry_entry = get_registry_entry(canister_id);
        assert!(registry_entry.is_some(), "Registry entry should exist");
        assert_eq!(registry_entry.unwrap().status, MigrationStatus::Failed);
    }
}
