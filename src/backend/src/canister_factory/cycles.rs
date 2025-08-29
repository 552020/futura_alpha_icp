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
