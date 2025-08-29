use crate::canister_factory::types::*;
use candid::Principal;

/// Ensure the caller owns a capsule (has a self-capsule where they are both subject and owner)
/// This function verifies that the caller principal has a capsule where they are the subject
/// and they are listed as an owner of that capsule
pub fn ensure_owner(caller: Principal) -> Result<(), String> {
    let user_ref = crate::types::PersonRef::Principal(caller);

    // Find the user's self-capsule (where user is both subject and owner)
    let has_self_capsule = crate::memory::with_capsules(|capsules| {
        capsules
            .values()
            .any(|capsule| capsule.subject == user_ref && capsule.owners.contains_key(&user_ref))
    });

    if has_self_capsule {
        Ok(())
    } else {
        Err(format!(
            "Access denied: Principal {} does not own a capsule",
            caller
        ))
    }
}

/// Ensure the caller is an admin (can perform admin-only operations)
/// This function checks if the caller is either a superadmin or a regular admin
pub fn ensure_admin(caller: Principal) -> Result<(), String> {
    if crate::admin::is_admin(&caller) {
        Ok(())
    } else {
        Err(format!(
            "Access denied: Principal {} is not an admin",
            caller
        ))
    }
}

/// Validate caller for migration endpoints - ensures they own a capsule
/// This is a convenience function that combines caller identification with ownership verification
pub fn validate_migration_caller() -> Result<Principal, String> {
    let caller = ic_cdk::api::msg_caller();

    // Reject anonymous callers
    if caller == Principal::anonymous() {
        return Err("Access denied: Anonymous callers cannot perform migrations".to_string());
    }

    // Ensure the caller owns a capsule
    ensure_owner(caller)?;

    Ok(caller)
}

/// Validate caller for admin-only migration operations
/// This function ensures the caller is an admin and returns their principal
pub fn validate_admin_caller() -> Result<Principal, String> {
    let caller = ic_cdk::api::msg_caller();

    // Reject anonymous callers
    if caller == Principal::anonymous() {
        return Err("Access denied: Anonymous callers cannot perform admin operations".to_string());
    }

    // Ensure the caller is an admin
    ensure_admin(caller)?;

    Ok(caller)
}

/// Check if a specific user owns a capsule (admin utility function)
/// This allows admins to check capsule ownership for any user
pub fn check_user_capsule_ownership(user: Principal) -> bool {
    let user_ref = crate::types::PersonRef::Principal(user);

    crate::memory::with_capsules(|capsules| {
        capsules
            .values()
            .any(|capsule| capsule.subject == user_ref && capsule.owners.contains_key(&user_ref))
    })
}

/// Get the capsule ID for a user (if they own one)
/// This is a utility function for migration operations
pub fn get_user_capsule_id(user: Principal) -> Option<String> {
    let user_ref = crate::types::PersonRef::Principal(user);

    crate::memory::with_capsules(|capsules| {
        capsules
            .values()
            .find(|capsule| capsule.subject == user_ref && capsule.owners.contains_key(&user_ref))
            .map(|capsule| capsule.id.clone())
    })
}

/// Validate configuration for personal canister creation
/// This function validates the minimal config and applies defaults
pub fn validate_and_prepare_config(
    config: CreatePersonalCanisterConfig,
) -> Result<CreatePersonalCanisterConfig, String> {
    let mut validated_config = config;

    // Validate name if provided
    if let Some(ref name) = validated_config.name {
        if name.is_empty() {
            return Err("Canister name cannot be empty".to_string());
        }
        if name.len() > 100 {
            return Err("Canister name cannot exceed 100 characters".to_string());
        }
        // Basic validation - only allow alphanumeric, spaces, hyphens, underscores
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_')
        {
            return Err(
                "Canister name can only contain alphanumeric characters, spaces, hyphens, and underscores"
                    .to_string(),
            );
        }
    }

    // Subnet ID validation is minimal for MVP - just check it's not anonymous
    if let Some(subnet_id) = validated_config.subnet_id {
        if subnet_id == Principal::anonymous() {
            return Err("Invalid subnet ID: cannot be anonymous principal".to_string());
        }
    }

    // Apply defaults if needed
    if validated_config.name.is_none() {
        validated_config.name = Some("Personal Canister".to_string());
    }

    Ok(validated_config)
}
