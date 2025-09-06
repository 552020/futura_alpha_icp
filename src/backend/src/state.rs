//! Canister State Management
//!
//! This module manages global canister state including size tracking,
//! limits, and other canister-wide metrics.

use crate::types::Error;
use std::cell::RefCell;

// ============================================================================
// CANISTER SIZE TRACKING
// ============================================================================

/// Maximum total size for the entire canister (100GB)
const MAX_CANISTER_SIZE: u64 = 100 * 1024 * 1024 * 1024; // 100GB

/// Global canister state tracker
thread_local! {
    pub static CANISTER_STATE: RefCell<CanisterState> = RefCell::new(CanisterState::new());
}

/// Canister state management
#[derive(Debug, Clone)]
pub struct CanisterState {
    /// Total size in bytes across all capsules and data
    pub total_size_bytes: u64,
}

impl CanisterState {
    /// Create new canister state
    pub fn new() -> Self {
        Self {
            total_size_bytes: 0,
        }
    }

    /// Add size to total and check limit
    pub fn add_size(&mut self, bytes: u64) -> Result<(), Error> {
        if self.total_size_bytes + bytes > MAX_CANISTER_SIZE {
            return Err(Error::ResourceExhausted);
        }
        self.total_size_bytes += bytes;
        Ok(())
    }

    /// Remove size from total
    pub fn remove_size(&mut self, bytes: u64) {
        self.total_size_bytes = self.total_size_bytes.saturating_sub(bytes);
    }

    /// Get current total size
    pub fn get_total_size(&self) -> u64 {
        self.total_size_bytes
    }

    /// Get remaining capacity
    pub fn get_remaining_capacity(&self) -> u64 {
        MAX_CANISTER_SIZE.saturating_sub(self.total_size_bytes)
    }

    /// Check if adding bytes would exceed limit
    pub fn would_exceed_limit(&self, additional_bytes: u64) -> bool {
        self.total_size_bytes + additional_bytes > MAX_CANISTER_SIZE
    }
}

impl Default for CanisterState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PUBLIC API FUNCTIONS
// ============================================================================

/// Track a size change (add new size, remove old size)
pub fn track_size_change(old_size: u64, new_size: u64) -> Result<(), Error> {
    CANISTER_STATE.with(|state| {
        let mut s = state.borrow_mut();
        s.remove_size(old_size);
        s.add_size(new_size)
    })
}

/// Add size to total canister size
pub fn add_canister_size(bytes: u64) -> Result<(), Error> {
    CANISTER_STATE.with(|state| {
        let mut s = state.borrow_mut();
        s.add_size(bytes)
    })
}

/// Remove size from total canister size
pub fn remove_canister_size(bytes: u64) {
    CANISTER_STATE.with(|state| {
        let mut s = state.borrow_mut();
        s.remove_size(bytes);
    });
}

/// Get current total canister size
pub fn get_total_canister_size() -> u64 {
    CANISTER_STATE.with(|state| state.borrow().get_total_size())
}

/// Get remaining canister capacity
pub fn get_remaining_canister_capacity() -> u64 {
    CANISTER_STATE.with(|state| state.borrow().get_remaining_capacity())
}

/// Check if canister would exceed size limit
pub fn would_exceed_canister_limit(additional_bytes: u64) -> bool {
    CANISTER_STATE.with(|state| state.borrow().would_exceed_limit(additional_bytes))
}

/// Get canister size statistics
pub fn get_canister_size_stats() -> CanisterSizeStats {
    CANISTER_STATE.with(|state| {
        let s = state.borrow();
        CanisterSizeStats {
            total_size_bytes: s.get_total_size(),
            remaining_capacity_bytes: s.get_remaining_capacity(),
            max_size_bytes: MAX_CANISTER_SIZE,
            usage_percentage: (s.get_total_size() as f64 / MAX_CANISTER_SIZE as f64) * 100.0,
        }
    })
}

/// Canister size statistics
#[derive(Debug, Clone, candid::CandidType, candid::Deserialize, serde::Serialize)]
pub struct CanisterSizeStats {
    pub total_size_bytes: u64,
    pub remaining_capacity_bytes: u64,
    pub max_size_bytes: u64,
    pub usage_percentage: f64,
}
