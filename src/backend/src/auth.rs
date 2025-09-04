use crate::memory::with_stable_upload_sessions;
use crate::types::ICPErrorCode;
use candid::Principal;

#[cfg(not(test))]
use ic_cdk::api;

// ============================================================================
// BASIC AUTHORIZATION FOR MVP - Simple caller verification and rate limiting
// ============================================================================

/// Check if caller is authorized for write operations
pub fn verify_caller_authorized() -> Result<Principal, ICPErrorCode> {
    let caller = get_caller();

    // Anonymous principal is not allowed for write operations
    if caller == Principal::anonymous() {
        return Err(ICPErrorCode::Unauthorized);
    }

    Ok(caller)
}

/// Check concurrent upload limit for user (max 3 per user)
#[allow(dead_code)]
pub fn check_upload_rate_limit(caller: &Principal) -> Result<(), ICPErrorCode> {
    const MAX_CONCURRENT_UPLOADS: usize = 3;

    let active_sessions = with_stable_upload_sessions(|sessions| {
        sessions
            .iter()
            .filter(|(_, session)| {
                // For MVP, we'll use a simple heuristic: check if memory_id contains caller info
                // In production, we'd store caller principal in the session
                session.memory_id.contains(&caller.to_string())
            })
            .count()
    });

    if active_sessions >= MAX_CONCURRENT_UPLOADS {
        return Err(ICPErrorCode::Internal(format!(
            "Rate limit exceeded: {active_sessions} concurrent uploads (max {MAX_CONCURRENT_UPLOADS})"
        )));
    }

    Ok(())
}

/// Get caller principal - works in both canister and test environments
fn get_caller() -> Principal {
    #[cfg(test)]
    {
        // Return a simple test principal for testing
        Principal::from_slice(&[1, 2, 3, 4])
    }
    #[cfg(not(test))]
    {
        api::msg_caller()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_caller_authorized_success() {
        // In test environment, get_caller() returns a valid test principal
        let result = verify_caller_authorized();
        assert!(result.is_ok());

        let caller = result.unwrap();
        assert_ne!(caller, Principal::anonymous());
    }

    #[test]
    fn test_check_upload_rate_limit_no_sessions() {
        let caller = Principal::from_slice(&[1, 2, 3, 4]);
        let result = check_upload_rate_limit(&caller);
        assert!(result.is_ok());
    }
}
