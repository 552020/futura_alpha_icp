//! Time normalization utilities for ICP access control system
//!
//! This module provides utilities to handle time differences between:
//! - ICP: Uses nanoseconds (ic_cdk::api::time())
//! - Neon database: Uses milliseconds
//! - Magic links: Need expiration checking

use ic_cdk::api::time;

// ============================================================================
// TIME CONSTANTS
// ============================================================================

/// Magic link time-to-live: 7 days in nanoseconds
#[allow(dead_code)]
pub const MAGIC_LINK_TTL_NS: u64 = 7 * 24 * 60 * 60 * 1_000_000_000; // 7 days in nanoseconds

/// One hour in nanoseconds
#[allow(dead_code)]
pub const HOUR_NS: u64 = 60 * 60 * 1_000_000_000;

/// One day in nanoseconds
#[allow(dead_code)]
pub const DAY_NS: u64 = 24 * HOUR_NS;

// ============================================================================
// TIME CONVERSION UTILITIES
// ============================================================================

/// Convert ICP time (nanoseconds) to Neon time (milliseconds)
#[allow(dead_code)]
pub fn icp_time_to_neon_ms(icp_time_ns: u64) -> u64 {
    icp_time_ns / 1_000_000
}

/// Convert Neon time (milliseconds) to ICP time (nanoseconds)
#[allow(dead_code)]
pub fn neon_ms_to_icp_time(neon_time_ms: u64) -> u64 {
    neon_time_ms * 1_000_000
}

/// Get current ICP time in nanoseconds
#[allow(dead_code)]
pub fn now_icp_ns() -> u64 {
    time()
}

/// Get current ICP time in milliseconds (for Neon compatibility)
#[allow(dead_code)]
pub fn now_icp_ms() -> u64 {
    icp_time_to_neon_ms(now_icp_ns())
}

// ============================================================================
// EXPIRATION UTILITIES
// ============================================================================

/// Check if a timestamp has expired based on TTL
#[allow(dead_code)]
pub fn is_expired(created_at_ns: u64, ttl_ns: u64) -> bool {
    now_icp_ns() > created_at_ns + ttl_ns
}

/// Check if a magic link has expired
#[allow(dead_code)]
pub fn is_magic_link_expired(created_at_ns: u64) -> bool {
    is_expired(created_at_ns, MAGIC_LINK_TTL_NS)
}

/// Get expiration time for a magic link
#[allow(dead_code)]
pub fn get_magic_link_expires_at(created_at_ns: u64) -> u64 {
    created_at_ns + MAGIC_LINK_TTL_NS
}

// ============================================================================
// TIME VALIDATION UTILITIES
// ============================================================================

/// Validate that a timestamp is reasonable (not in the future, not too old)
#[allow(dead_code)]
pub fn is_valid_timestamp(timestamp_ns: u64) -> bool {
    let now = now_icp_ns();
    let max_age = 10 * 365 * DAY_NS; // 10 years in nanoseconds

    // Not in the future (allow 1 hour tolerance for clock skew)
    let not_future = timestamp_ns <= now + HOUR_NS;

    // Not too old (not more than 10 years)
    let not_too_old = timestamp_ns >= now - max_age;

    not_future && not_too_old
}

/// Get time remaining until expiration (returns 0 if expired)
#[allow(dead_code)]
pub fn time_until_expiry(created_at_ns: u64, ttl_ns: u64) -> u64 {
    let expires_at = created_at_ns + ttl_ns;
    let now = now_icp_ns();

    if now >= expires_at {
        0
    } else {
        expires_at - now
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_conversions() {
        let icp_time = 1_000_000_000; // 1 second in nanoseconds
        let neon_time = 1_000; // 1 second in milliseconds

        assert_eq!(icp_time_to_neon_ms(icp_time), neon_time);
        assert_eq!(neon_ms_to_icp_time(neon_time), icp_time);
    }

    #[test]
    fn test_expiration_check() {
        let now = now_icp_ns();
        let ttl = HOUR_NS; // 1 hour

        // Not expired
        assert!(!is_expired(now, ttl));

        // Expired (created 2 hours ago)
        let old_time = now - (2 * HOUR_NS);
        assert!(is_expired(old_time, ttl));
    }

    #[test]
    fn test_magic_link_expiration() {
        let now = now_icp_ns();

        // Not expired
        assert!(!is_magic_link_expired(now));

        // Expired (created 8 days ago)
        let old_time = now - (8 * DAY_NS);
        assert!(is_magic_link_expired(old_time));
    }

    #[test]
    fn test_time_validation() {
        let now = now_icp_ns();

        // Valid timestamp (1 hour ago)
        assert!(is_valid_timestamp(now - HOUR_NS));

        // Invalid: too far in future
        assert!(!is_valid_timestamp(now + (2 * HOUR_NS)));

        // Invalid: too old
        assert!(!is_valid_timestamp(now - (11 * 365 * DAY_NS)));
    }

    #[test]
    fn test_time_until_expiry() {
        let now = now_icp_ns();
        let ttl = HOUR_NS;

        // 30 minutes remaining
        let created_30min_ago = now - (30 * 60 * 1_000_000_000);
        let remaining = time_until_expiry(created_30min_ago, ttl);
        assert!(remaining > 0);
        assert!(remaining < HOUR_NS);

        // Expired
        let created_2hours_ago = now - (2 * HOUR_NS);
        assert_eq!(time_until_expiry(created_2hours_ago, ttl), 0);
    }
}
