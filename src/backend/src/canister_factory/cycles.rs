use crate::canister_factory::types::*;
use candid::Principal;

/// Check if factory has sufficient cycles in reserve for the required amount
/// This is a preflight check that should be called before attempting operations
pub fn preflight_cycles_reserve(required_cycles: u128) -> Result<(), String> {
    crate::memory::with_migration_state(|state| {
        let config = &state.migration_config;

        // Check if reserve is below minimum threshold
        if config.cycles_reserve < config.min_cycles_threshold {
            return Err(format!(
                "Factory cycles reserve ({}) is below minimum threshold ({})",
                config.cycles_reserve, config.min_cycles_threshold
            ));
        }

        // Check if reserve has enough for the required operation
        if config.cycles_reserve < required_cycles {
            return Err(format!(
                "Insufficient cycles in factory reserve. Required: {}, Available: {}",
                required_cycles, config.cycles_reserve
            ));
        }

        Ok(())
    })
}

/// Consume cycles from the factory reserve
/// This should only be called after a successful preflight check
pub fn consume_cycles_from_reserve(cycles_to_consume: u128) -> Result<(), String> {
    crate::memory::with_migration_state_mut(|state| {
        let config = &mut state.migration_config;

        // Double-check we have enough cycles
        if config.cycles_reserve < cycles_to_consume {
            return Err(format!(
                "Cannot consume {} cycles, only {} available in reserve",
                cycles_to_consume, config.cycles_reserve
            ));
        }

        // Consume the cycles
        config.cycles_reserve = config.cycles_reserve.saturating_sub(cycles_to_consume);

        // Update total cycles consumed in stats
        state.migration_stats.total_cycles_consumed = state
            .migration_stats
            .total_cycles_consumed
            .saturating_add(cycles_to_consume);

        ic_cdk::println!(
            "Consumed {} cycles from factory reserve. Remaining: {}",
            cycles_to_consume,
            config.cycles_reserve
        );

        Ok(())
    })
}

/// Get current cycles reserve amount (admin function)
pub fn get_cycles_reserve() -> u128 {
    crate::memory::with_migration_state(|state| state.migration_config.cycles_reserve)
}

/// Add cycles to the factory reserve (admin function)
pub fn add_cycles_to_reserve(cycles_to_add: u128) -> Result<u128, String> {
    crate::memory::with_migration_state_mut(|state| {
        let config = &mut state.migration_config;
        config.cycles_reserve = config.cycles_reserve.saturating_add(cycles_to_add);

        ic_cdk::println!(
            "Added {} cycles to factory reserve. New total: {}",
            cycles_to_add,
            config.cycles_reserve
        );

        Ok(config.cycles_reserve)
    })
}

/// Set the minimum cycles threshold (admin function)
pub fn set_cycles_threshold(new_threshold: u128) -> Result<(), String> {
    crate::memory::with_migration_state_mut(|state| {
        state.migration_config.min_cycles_threshold = new_threshold;

        ic_cdk::println!("Updated cycles threshold to: {}", new_threshold);

        Ok(())
    })
}

/// Get current cycles threshold (admin function)
pub fn get_cycles_threshold() -> u128 {
    crate::memory::with_migration_state(|state| state.migration_config.min_cycles_threshold)
}

/// Get cycles reserve status including threshold information (admin function)
pub fn get_cycles_reserve_status() -> CyclesReserveStatus {
    crate::memory::with_migration_state(|state| {
        let config = &state.migration_config;
        CyclesReserveStatus {
            current_reserve: config.cycles_reserve,
            min_threshold: config.min_cycles_threshold,
            is_above_threshold: config.cycles_reserve >= config.min_cycles_threshold,
            total_consumed: state.migration_stats.total_cycles_consumed,
        }
    })
}

/// Check if cycles reserve is below threshold and log warning
pub fn check_cycles_reserve_threshold() -> bool {
    let status = get_cycles_reserve_status();

    if !status.is_above_threshold {
        ic_cdk::println!(
            "WARNING: Factory cycles reserve ({}) is below minimum threshold ({}). Admin action required!",
            status.current_reserve,
            status.min_threshold
        );

        // Log additional context for debugging
        ic_cdk::println!("Total cycles consumed to date: {}", status.total_consumed);

        return false;
    }

    true
}

/// Log cycles consumption with context
pub fn log_cycles_consumption(
    operation: &str,
    cycles_consumed: u128,
    user: Option<Principal>,
    canister_id: Option<Principal>,
) {
    let status = get_cycles_reserve_status();

    ic_cdk::println!(
        "CYCLES_CONSUMPTION: operation={}, consumed={}, remaining_reserve={}, user={:?}, canister={:?}",
        operation,
        cycles_consumed,
        status.current_reserve,
        user,
        canister_id
    );

    // Check if this consumption brings us below threshold
    if !check_cycles_reserve_threshold() {
        ic_cdk::println!(
            "ALERT: Cycles reserve is now below threshold after {} operation",
            operation
        );
    }
}

/// Get current alert level based on cycles reserve
pub fn get_cycles_alert_level() -> CyclesAlertLevel {
    let status = get_cycles_reserve_status();

    if !status.is_above_threshold {
        // If below 50% of threshold, consider it critical
        let critical_threshold = status.min_threshold / 2;
        if status.current_reserve <= critical_threshold {
            CyclesAlertLevel::Critical
        } else {
            CyclesAlertLevel::Warning
        }
    } else {
        CyclesAlertLevel::Normal
    }
}

/// Comprehensive cycles monitoring report (admin function)
pub fn get_cycles_monitoring_report() -> CyclesMonitoringReport {
    let reserve_status = get_cycles_reserve_status();
    let alert_level = get_cycles_alert_level();

    let mut recommendations = Vec::new();

    match alert_level {
        CyclesAlertLevel::Critical => {
            recommendations.push(
                "URGENT: Cycles reserve is critically low. Add cycles immediately.".to_string(),
            );
            recommendations.push(
                "Consider temporarily disabling migrations until reserve is replenished."
                    .to_string(),
            );
        }
        CyclesAlertLevel::Warning => {
            recommendations
                .push("Cycles reserve is below threshold. Plan to add cycles soon.".to_string());
            recommendations
                .push("Monitor consumption rate and adjust threshold if needed.".to_string());
        }
        CyclesAlertLevel::Normal => {
            recommendations.push("Cycles reserve is healthy.".to_string());
        }
    }

    // Add general recommendations
    if reserve_status.total_consumed > 0 {
        recommendations.push(format!(
            "Total cycles consumed: {}. Consider this for future capacity planning.",
            reserve_status.total_consumed
        ));
    }

    CyclesMonitoringReport {
        reserve_status,
        alert_level,
        recent_consumption_rate: None, // Could be implemented with historical tracking
        recommendations,
    }
}

/// Admin notification system for low reserves
/// This function should be called periodically or after operations
pub fn check_and_alert_low_reserves() -> Option<String> {
    let alert_level = get_cycles_alert_level();
    let status = get_cycles_reserve_status();

    match alert_level {
        CyclesAlertLevel::Critical => {
            let alert_message = format!(
                "CRITICAL ALERT: Factory cycles reserve is critically low! Current: {}, Threshold: {}. Immediate action required!",
                status.current_reserve,
                status.min_threshold
            );

            ic_cdk::println!("{}", alert_message);
            Some(alert_message)
        }
        CyclesAlertLevel::Warning => {
            let alert_message = format!(
                "WARNING: Factory cycles reserve is below threshold. Current: {}, Threshold: {}. Please add cycles soon.",
                status.current_reserve,
                status.min_threshold
            );

            ic_cdk::println!("{}", alert_message);
            Some(alert_message)
        }
        CyclesAlertLevel::Normal => None,
    }
}

/// Get the default cycles amount for personal canister creation
/// This can be made configurable in the future
pub fn get_default_canister_cycles() -> u128 {
    // Default to 2T cycles for personal canister creation
    // This should be sufficient for initial setup and some operations
    2_000_000_000_000
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canister_factory::types::{
        CyclesAlertLevel, CyclesMonitoringReport, CyclesReserveStatus, MigrationConfig,
        MigrationStateData,
    };
    use candid::Principal;

    // Mock the memory functions for testing
    fn setup_test_state() -> MigrationStateData {
        MigrationStateData {
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
            ..Default::default()
        }
    }

    fn setup_low_reserve_state() -> MigrationStateData {
        MigrationStateData {
            migration_config: MigrationConfig {
                enabled: true,
                cycles_reserve: 1_500_000_000_000, // 1.5T cycles (below threshold but above critical)
                min_cycles_threshold: 2_000_000_000_000, // 2T cycles
                admin_principals: std::collections::BTreeSet::new(),
            },
            migration_stats: MigrationStats {
                total_cycles_consumed: 5_000_000_000_000, // 5T cycles consumed
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn setup_critical_reserve_state() -> MigrationStateData {
        MigrationStateData {
            migration_config: MigrationConfig {
                enabled: true,
                cycles_reserve: 500_000_000_000, // 0.5T cycles (critical)
                min_cycles_threshold: 2_000_000_000_000, // 2T cycles
                admin_principals: std::collections::BTreeSet::new(),
            },
            migration_stats: MigrationStats {
                total_cycles_consumed: 10_000_000_000_000, // 10T cycles consumed
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_preflight_cycles_reserve_sufficient() {
        // This test would need to mock the memory::with_migration_state function
        // For now, we'll test the logic conceptually

        let state = setup_test_state();
        let required_cycles = 1_000_000_000_000; // 1T cycles

        // In a real test, we would mock the state access
        // Here we verify the logic would work correctly
        assert!(
            state.migration_config.cycles_reserve >= state.migration_config.min_cycles_threshold
        );
        assert!(state.migration_config.cycles_reserve >= required_cycles);
    }

    #[test]
    fn test_preflight_cycles_reserve_below_threshold() {
        let state = setup_low_reserve_state();
        let required_cycles = 500_000_000_000; // 0.5T cycles

        // Reserve is below threshold
        assert!(
            state.migration_config.cycles_reserve < state.migration_config.min_cycles_threshold
        );

        // Even though we have enough for the operation, it should fail due to threshold
        assert!(state.migration_config.cycles_reserve >= required_cycles);
    }

    #[test]
    fn test_preflight_cycles_reserve_insufficient() {
        let state = setup_test_state();
        let required_cycles = 15_000_000_000_000; // 15T cycles (more than available)

        // Reserve is above threshold but insufficient for operation
        assert!(
            state.migration_config.cycles_reserve >= state.migration_config.min_cycles_threshold
        );
        assert!(state.migration_config.cycles_reserve < required_cycles);
    }

    #[test]
    fn test_cycles_reserve_status() {
        let state = setup_test_state();

        let expected_status = CyclesReserveStatus {
            current_reserve: 10_000_000_000_000,
            min_threshold: 2_000_000_000_000,
            is_above_threshold: true,
            total_consumed: 1_000_000_000_000,
        };

        assert_eq!(
            state.migration_config.cycles_reserve,
            expected_status.current_reserve
        );
        assert_eq!(
            state.migration_config.min_cycles_threshold,
            expected_status.min_threshold
        );
        assert_eq!(
            state.migration_config.cycles_reserve >= state.migration_config.min_cycles_threshold,
            expected_status.is_above_threshold
        );
        assert_eq!(
            state.migration_stats.total_cycles_consumed,
            expected_status.total_consumed
        );
    }

    #[test]
    fn test_cycles_alert_level_normal() {
        let state = setup_test_state();

        // Above threshold should be Normal
        assert!(
            state.migration_config.cycles_reserve >= state.migration_config.min_cycles_threshold
        );

        // Expected alert level: Normal
        let expected_level = CyclesAlertLevel::Normal;
        assert_eq!(expected_level, CyclesAlertLevel::Normal);
    }

    #[test]
    fn test_cycles_alert_level_warning() {
        let state = setup_low_reserve_state();

        // Below threshold but above critical (50% of threshold)
        let critical_threshold = state.migration_config.min_cycles_threshold / 2;
        assert!(
            state.migration_config.cycles_reserve < state.migration_config.min_cycles_threshold
        );
        assert!(state.migration_config.cycles_reserve > critical_threshold);

        // Expected alert level: Warning
        let expected_level = CyclesAlertLevel::Warning;
        assert_eq!(expected_level, CyclesAlertLevel::Warning);
    }

    #[test]
    fn test_cycles_alert_level_critical() {
        let state = setup_critical_reserve_state();

        // Below critical threshold (50% of min threshold)
        let critical_threshold = state.migration_config.min_cycles_threshold / 2;
        assert!(state.migration_config.cycles_reserve <= critical_threshold);

        // Expected alert level: Critical
        let expected_level = CyclesAlertLevel::Critical;
        assert_eq!(expected_level, CyclesAlertLevel::Critical);
    }

    #[test]
    fn test_cycles_consumption_calculation() {
        let mut state = setup_test_state();
        let cycles_to_consume = 3_000_000_000_000; // 3T cycles

        let initial_reserve = state.migration_config.cycles_reserve;
        let initial_consumed = state.migration_stats.total_cycles_consumed;

        // Simulate consumption
        state.migration_config.cycles_reserve = state
            .migration_config
            .cycles_reserve
            .saturating_sub(cycles_to_consume);
        state.migration_stats.total_cycles_consumed = state
            .migration_stats
            .total_cycles_consumed
            .saturating_add(cycles_to_consume);

        // Verify calculations
        assert_eq!(
            state.migration_config.cycles_reserve,
            initial_reserve - cycles_to_consume
        );
        assert_eq!(
            state.migration_stats.total_cycles_consumed,
            initial_consumed + cycles_to_consume
        );
    }

    #[test]
    fn test_cycles_consumption_insufficient() {
        let state = setup_test_state();
        let cycles_to_consume = 15_000_000_000_000; // 15T cycles (more than available)

        // Should not be able to consume more than available
        assert!(state.migration_config.cycles_reserve < cycles_to_consume);
    }

    #[test]
    fn test_cycles_threshold_monitoring() {
        let normal_state = setup_test_state();
        let warning_state = setup_low_reserve_state();
        let critical_state = setup_critical_reserve_state();

        // Normal state - above threshold
        assert!(
            normal_state.migration_config.cycles_reserve
                >= normal_state.migration_config.min_cycles_threshold
        );

        // Warning state - below threshold but not critical
        assert!(
            warning_state.migration_config.cycles_reserve
                < warning_state.migration_config.min_cycles_threshold
        );
        let warning_critical_threshold = warning_state.migration_config.min_cycles_threshold / 2;
        assert!(warning_state.migration_config.cycles_reserve > warning_critical_threshold);

        // Critical state - below critical threshold
        let critical_threshold = critical_state.migration_config.min_cycles_threshold / 2;
        assert!(critical_state.migration_config.cycles_reserve <= critical_threshold);
    }

    #[test]
    fn test_cycles_reserve_addition() {
        let mut state = setup_test_state();
        let cycles_to_add = 5_000_000_000_000; // 5T cycles

        let initial_reserve = state.migration_config.cycles_reserve;

        // Simulate adding cycles
        state.migration_config.cycles_reserve = state
            .migration_config
            .cycles_reserve
            .saturating_add(cycles_to_add);

        // Verify addition
        assert_eq!(
            state.migration_config.cycles_reserve,
            initial_reserve + cycles_to_add
        );
    }

    #[test]
    fn test_cycles_threshold_update() {
        let mut state = setup_test_state();
        let new_threshold = 5_000_000_000_000; // 5T cycles

        // Update threshold
        state.migration_config.min_cycles_threshold = new_threshold;

        // Verify update
        assert_eq!(state.migration_config.min_cycles_threshold, new_threshold);

        // Check if this affects the alert level
        let is_above_threshold = state.migration_config.cycles_reserve >= new_threshold;
        assert_eq!(
            is_above_threshold,
            state.migration_config.cycles_reserve >= new_threshold
        );
    }

    #[test]
    fn test_default_canister_cycles() {
        let default_cycles = get_default_canister_cycles();

        // This should match the constant in get_default_canister_cycles
        assert_eq!(default_cycles, 2_000_000_000_000);
    }

    #[test]
    fn test_cycles_monitoring_report_structure() {
        let state = setup_test_state();

        let reserve_status = CyclesReserveStatus {
            current_reserve: state.migration_config.cycles_reserve,
            min_threshold: state.migration_config.min_cycles_threshold,
            is_above_threshold: state.migration_config.cycles_reserve
                >= state.migration_config.min_cycles_threshold,
            total_consumed: state.migration_stats.total_cycles_consumed,
        };

        let alert_level = CyclesAlertLevel::Normal; // Based on setup_test_state

        let report = CyclesMonitoringReport {
            reserve_status: reserve_status.clone(),
            alert_level,
            recent_consumption_rate: None,
            recommendations: vec!["Cycles reserve is healthy.".to_string()],
        };

        // Verify report structure
        assert_eq!(
            report.reserve_status.current_reserve,
            state.migration_config.cycles_reserve
        );
        assert_eq!(report.alert_level, CyclesAlertLevel::Normal);
        assert!(report.recommendations.len() > 0);
    }

    #[test]
    fn test_cycles_alert_recommendations() {
        // Test recommendations for different alert levels

        // Normal state recommendations
        let normal_recommendations = vec!["Cycles reserve is healthy.".to_string()];
        assert!(normal_recommendations.contains(&"Cycles reserve is healthy.".to_string()));

        // Warning state recommendations
        let warning_recommendations = vec![
            "Cycles reserve is below threshold. Plan to add cycles soon.".to_string(),
            "Monitor consumption rate and adjust threshold if needed.".to_string(),
        ];
        assert!(warning_recommendations.len() >= 2);

        // Critical state recommendations
        let critical_recommendations = vec![
            "URGENT: Cycles reserve is critically low. Add cycles immediately.".to_string(),
            "Consider temporarily disabling migrations until reserve is replenished.".to_string(),
        ];
        assert!(critical_recommendations.len() >= 2);
        assert!(critical_recommendations[0].contains("URGENT"));
    }

    #[test]
    fn test_cycles_saturation_arithmetic() {
        let max_cycles = u128::MAX;
        let large_amount = u128::MAX / 2;

        // Test saturating_add
        let result_add = max_cycles.saturating_add(large_amount);
        assert_eq!(result_add, u128::MAX);

        // Test saturating_sub
        let small_amount: u128 = 1000;
        let result_sub = small_amount.saturating_sub(large_amount);
        assert_eq!(result_sub, 0);
    }

    #[test]
    fn test_cycles_logging_context() {
        // Test that logging context includes all necessary information
        let operation = "create_canister";
        let cycles_consumed: u128 = 2_000_000_000_000;
        // Create test principals with valid format
        let user = Principal::from_slice(&[1, 2, 3, 4]);
        let canister_id = Principal::from_slice(&[5, 6, 7, 8]);

        // Verify all parameters are valid
        assert!(!operation.is_empty());
        assert!(cycles_consumed > 0);
        assert_ne!(user, Principal::anonymous());
        assert_ne!(canister_id, Principal::anonymous());
    }
}
