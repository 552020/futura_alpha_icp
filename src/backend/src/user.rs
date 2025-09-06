// ============================================================================
// FRONTEND ADAPTER - USER MANAGEMENT
// ============================================================================
//
// This file acts as a frontend adapter that translates frontend "user" concepts
// to backend "capsule" concepts. The frontend thinks in terms of "users" while
// the backend works with "capsules" (where a user = a self-capsule).
//
// Purpose: Bridge the conceptual gap between frontend and backend terminology
// - Frontend: "Register a user"
// - Backend: "Create/manage a self-capsule"
// - This module: Translates between the two concepts
//
// ============================================================================

use crate::memory;
use crate::types::{self, PersonRef};
use ic_cdk::api::time;

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Store nonce proof for Internet Identity authentication
/// This is a utility function used by both prove_nonce and register_with_nonce
pub fn store_nonce_proof_utility(nonce: String, caller: &PersonRef) -> types::Result<()> {
    let timestamp = time();

    let principal = match caller {
        PersonRef::Principal(p) => p,
        PersonRef::Opaque(_) => {
            return Err(types::Error::Internal(
                "Only principals can store nonce proofs".to_string(),
            ))
        }
    };

    memory::store_nonce_proof(nonce, *principal, timestamp);
    Ok(())
}

// ============================================================================
// FRONTEND ADAPTER FUNCTIONS
// ============================================================================

/// Register a user with nonce proof (frontend: "register user", backend: "create self-capsule")
/// This is the main user registration function that the frontend calls
pub fn register_user_with_nonce(nonce: String) -> types::Result<()> {
    let caller = PersonRef::from_caller();

    // Frontend perspective: "Register the user"
    // Backend reality: "Check if user already has a self-capsule"
    if let Some(capsule) = crate::capsule::find_self_capsule(&caller) {
        // Frontend perspective: "Update user activity"
        // Backend reality: "Update capsule activity timestamp"
        crate::capsule::update_capsule_activity(&capsule.id, &caller)?;
    } else {
        // Frontend perspective: "Create new user"
        // Backend reality: "Create new self-capsule"
        match crate::capsules_create(None) {
            types::CapsuleCreationResult { success: true, .. } => {}
            types::CapsuleCreationResult { success: false, .. } => {
                return Err(types::Error::Internal(
                    "Failed to create user capsule".to_string(),
                ))
            }
        }
    }

    // Store nonce proof for Internet Identity authentication
    store_nonce_proof_utility(nonce, &caller)?;

    Ok(())
}
