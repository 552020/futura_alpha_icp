use crate::memory::{with_admins, with_admins_mut};
use crate::types::Result;
use candid::Principal;
use ic_cdk::api::msg_caller;

// Hardcoded superadmin principals
// These principals have ultimate control and cannot be removed
const SUPERADMIN_PRINCIPALS: [&str; 1] = [
    "otzfv-jscof-niinw-gtloq-25uz3-pglpg-u3kug-besf3-rzlbd-ylrmp-5ae", // 552020
];

/// Check if principal is a superadmin (hardcoded)
pub fn is_superadmin(principal: &Principal) -> bool {
    let principal_str = principal.to_string();
    SUPERADMIN_PRINCIPALS.contains(&principal_str.as_str())
}

/// Check if principal is an admin (auto-bootstrap first caller)
pub fn is_admin(principal: &Principal) -> bool {
    // Superadmins are always considered admins
    if is_superadmin(principal) {
        return true;
    }

    with_admins_mut(|admins| {
        // If no admins exist, the first caller becomes admin
        if admins.is_empty() {
            admins.insert(*principal);
            return true;
        }

        admins.contains(principal)
    })
}

/// Add new admin (only superadmins can add admins)
pub fn add_admin(new_admin_principal: Principal) -> Result<()> {
    let caller = msg_caller();

    // Only superadmins can add new admins
    if !is_superadmin(&caller) {
        return Err(crate::types::Error::Unauthorized);
    }

    // Cannot add a superadmin as a regular admin
    if is_superadmin(&new_admin_principal) {
        return Err(crate::types::Error::InvalidArgument(
            "Cannot add superadmin as regular admin".to_string(),
        ));
    }

    with_admins_mut(|admins| {
        admins.insert(new_admin_principal);
    });

    Ok(())
}

/// Remove admin (only superadmins can remove admins)
pub fn remove_admin(admin_principal: Principal) -> Result<()> {
    let caller = msg_caller();

    // Only superadmins can remove admins
    if !is_superadmin(&caller) {
        return Err(crate::types::Error::Unauthorized);
    }

    // Cannot remove a superadmin
    if is_superadmin(&admin_principal) {
        return Err(crate::types::Error::InvalidArgument(
            "Cannot remove superadmin".to_string(),
        ));
    }

    with_admins_mut(|admins| {
        if admins.remove(&admin_principal) {
            Ok(())
        } else {
            Err(crate::types::Error::NotFound)
        }
    })
}

/// List all admins
pub fn list_admins() -> Vec<Principal> {
    let caller = msg_caller();

    if !is_admin(&caller) {
        return Vec::new();
    }

    with_admins(|admins| admins.iter().cloned().collect())
}

/// List all superadmins (hardcoded)
pub fn list_superadmins() -> Vec<Principal> {
    SUPERADMIN_PRINCIPALS
        .iter()
        .map(|s| Principal::from_text(s).unwrap_or_else(|_| Principal::anonymous()))
        .collect()
}

/// Export all admins for upgrade persistence
pub fn export_admins_for_upgrade() -> Vec<Principal> {
    with_admins(|admins| admins.iter().cloned().collect())
}

/// Import admins from stable storage after canister upgrade
pub fn import_admins_from_upgrade(admin_data: Vec<Principal>) {
    with_admins_mut(|admins| {
        *admins = admin_data.into_iter().collect();
    })
}
