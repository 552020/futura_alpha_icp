use crate::canister_factory::types::*;
use candid::Principal;
use std::cell::RefCell;
use std::collections::BTreeMap;

#[cfg(test)]
mod tests {
    #[test]
    fn test_simple_test_utils() {
        assert_eq!(1 + 1, 2);
    }
}

// Mock state for testing
thread_local! {
    static MOCK_STATE: RefCell<PersonalCanisterCreationStateData> = RefCell::new(PersonalCanisterCreationStateData::default());
}

// Mock implementation of memory functions for testing
pub fn with_mock_creation_state<R>(f: impl FnOnce(&PersonalCanisterCreationStateData) -> R) -> R {
    MOCK_STATE.with(|state| f(&state.borrow()))
}

pub fn with_mock_creation_state_mut<R>(
    f: impl FnOnce(&mut PersonalCanisterCreationStateData) -> R,
) -> R {
    MOCK_STATE.with(|state| f(&mut state.borrow_mut()))
}

// Legacy functions for backward compatibility
pub fn with_mock_migration_state<R>(f: impl FnOnce(&PersonalCanisterCreationStateData) -> R) -> R {
    with_mock_creation_state(f)
}

pub fn with_mock_migration_state_mut<R>(
    f: impl FnOnce(&mut PersonalCanisterCreationStateData) -> R,
) -> R {
    with_mock_creation_state_mut(f)
}

pub fn setup_test_state() {
    with_mock_creation_state_mut(|state| {
        *state = PersonalCanisterCreationStateData {
            creation_config: PersonalCanisterCreationConfig {
                enabled: true,
                cycles_reserve: 10_000_000_000_000, // 10T cycles
                min_cycles_threshold: 2_000_000_000_000, // 2T cycles
                admin_principals: std::collections::BTreeSet::new(),
            },
            creation_stats: PersonalCanisterCreationStats {
                total_cycles_consumed: 1_000_000_000_000, // 1T cycles consumed
                ..Default::default()
            },
            personal_canisters: BTreeMap::new(),
            ..Default::default()
        };
    });
}

pub fn setup_low_reserve_state() {
    with_mock_creation_state_mut(|state| {
        *state = PersonalCanisterCreationStateData {
            creation_config: PersonalCanisterCreationConfig {
                enabled: true,
                cycles_reserve: 1_000_000_000_000, // 1T cycles (below threshold)
                min_cycles_threshold: 2_000_000_000_000, // 2T cycles
                admin_principals: std::collections::BTreeSet::new(),
            },
            creation_stats: PersonalCanisterCreationStats {
                total_cycles_consumed: 5_000_000_000_000, // 5T cycles consumed
                ..Default::default()
            },
            personal_canisters: BTreeMap::new(),
            ..Default::default()
        };
    });
}

pub fn create_test_principal(id: u8) -> Principal {
    let mut bytes = [0u8; 29];
    bytes[0] = id;
    Principal::from_slice(&bytes)
}

pub fn simple_hash(input: &str) -> String {
    // Simple hash function for testing - not cryptographically secure
    let mut hash = 0u64;
    for byte in input.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    format!("{:x}", hash)
}

pub fn mock_time() -> u64 {
    1234567890000000000 // Mock timestamp in nanoseconds
}
