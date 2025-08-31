use super::test_utils::*;
use crate::canister_factory::types::*;

// Mock cycles functions that use our mock state
fn mock_preflight_cycles_reserve(required_cycles: u128) -> Result<(), String> {
    with_mock_creation_state(|state| {
        let config = &state.creation_config;

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
    with_mock_creation_state_mut(|state| {
        let config = &mut state.creation_config;

        if config.cycles_reserve < cycles_to_consume {
            return Err(format!(
                "Cannot consume {} cycles, only {} available in reserve",
                cycles_to_consume, config.cycles_reserve
            ));
        }

        config.cycles_reserve = config.cycles_reserve.saturating_sub(cycles_to_consume);
        state.creation_stats.total_cycles_consumed = state
            .creation_stats
            .total_cycles_consumed
            .saturating_add(cycles_to_consume);

        Ok(())
    })
}

fn mock_get_cycles_reserve_status() -> CyclesReserveStatus {
    with_mock_creation_state(|state| {
        let config = &state.creation_config;
        CyclesReserveStatus {
            current_reserve: config.cycles_reserve,
            min_threshold: config.min_cycles_threshold,
            is_above_threshold: config.cycles_reserve >= config.min_cycles_threshold,
            total_consumed: state.creation_stats.total_cycles_consumed,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
        with_mock_creation_state_mut(|state| {
            state.creation_config.cycles_reserve = 500_000_000_000; // 0.5T cycles
        });
        let critical_status = mock_get_cycles_reserve_status();
        assert!(!critical_status.is_above_threshold);
        let critical_threshold = critical_status.min_threshold / 2;
        assert!(critical_status.current_reserve <= critical_threshold);
    }
}
