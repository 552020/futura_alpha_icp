use candid::Principal;
use std::cell::RefCell;
use std::collections::HashMap;

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
