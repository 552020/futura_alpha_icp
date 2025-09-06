use crate::types::Error;
use candid::Principal;
use std::cell::RefCell;
use std::collections::HashMap;

#[cfg(not(test))]
use ic_cdk::api;

// ============================================================================
// NONCE PROOF STORAGE FOR INTERNET IDENTITY AUTHENTICATION
// ============================================================================

thread_local! {
    // Nonce proof storage for II authentication
    static NONCE_PROOFS: RefCell<HashMap<String, (Principal, u64)>> = RefCell::new(HashMap::new());
}

/// Store nonce proof for Internet Identity authentication
pub fn store_nonce_proof(nonce: String, principal: Principal, timestamp: u64) -> bool {
    NONCE_PROOFS.with(|proofs| {
        proofs.borrow_mut().insert(nonce, (principal, timestamp));
    });
    true
}

/// Get nonce proof for Internet Identity authentication
pub fn get_nonce_proof(nonce: String) -> Option<Principal> {
    NONCE_PROOFS.with(|proofs| proofs.borrow().get(&nonce).map(|(principal, _)| *principal))
}

// ============================================================================
// BASIC AUTHORIZATION FOR MVP - Simple caller verification and rate limiting
// ============================================================================

/// Check if caller is authorized for write operations
pub fn verify_caller_authorized() -> Result<Principal, Error> {
    let caller = get_caller();

    // Anonymous principal is not allowed for write operations
    if caller == Principal::anonymous() {
        return Err(Error::Unauthorized);
    }

    Ok(caller)
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
}
