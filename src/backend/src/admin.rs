use crate::memory::{MEM_ADMINS, MM};
use crate::types::Error;
use candid::Principal;
use ic_cdk::api::msg_caller;
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static STABLE_ADMINS: RefCell<StableBTreeMap<Principal, (), Memory>> = RefCell::new(
        StableBTreeMap::init(MM.with(|m| m.borrow().get(MEM_ADMINS)))
    );
}

/// Admin store for managing admin principals
///
/// TODO (Post-MVP): Consider creating an Admin struct instead of just storing principals
/// - Current: StableBTreeMap<Principal, (), Memory> (just principals)
/// - Future: StableBTreeMap<Principal, Admin, Memory> (with metadata)
/// - Benefits: Audit trail, permissions, creation tracking, admin history
/// - Use cases: When we need admin metadata, permission levels, or audit requirements
pub struct AdminStore;

impl AdminStore {
    /// Check if principal is an admin (auto-bootstrap first caller)
    pub fn is_admin(principal: &Principal) -> bool {
        // Superadmins are always considered admins
        if is_superadmin(principal) {
            return true;
        }

        Self::with_admins_mut(|admins| {
            // If no admins exist, the first caller becomes admin
            if admins.is_empty() {
                admins.insert(*principal, ());
                return true;
            }

            admins.contains_key(principal)
        })
    }

    /// Add new admin (only superadmins can add admins)
    pub fn add_admin(new_admin_principal: Principal) -> std::result::Result<(), Error> {
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

        Self::with_admins_mut(|admins| {
            admins.insert(new_admin_principal, ());
        });

        Ok(())
    }

    /// Remove admin (only superadmins can remove admins)
    pub fn remove_admin(admin_principal: Principal) -> std::result::Result<(), Error> {
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

        Self::with_admins_mut(|admins| {
            if admins.remove(&admin_principal).is_some() {
                Ok(())
            } else {
                Err(crate::types::Error::NotFound)
            }
        })
    }

    /// List all admins
    pub fn list_admins() -> Vec<Principal> {
        let caller = msg_caller();

        if !Self::is_admin(&caller) {
            return Vec::new();
        }

        Self::with_admins(|admins| admins.iter().map(|(k, _)| k).collect())
    }

    /// Export all admins for upgrade persistence
    pub fn export_admins_for_upgrade() -> Vec<Principal> {
        Self::with_admins(|admins| admins.iter().map(|(k, _)| k).collect())
    }

    /// Import admins from stable storage after canister upgrade
    pub fn import_admins_from_upgrade(admin_data: Vec<Principal>) {
        Self::with_admins_mut(|admins| {
            // Clear existing admins by removing all keys
            let keys_to_remove: Vec<Principal> = admins.iter().map(|(k, _)| k).collect();
            for key in keys_to_remove {
                admins.remove(&key);
            }
            // Insert new admins
            for admin in admin_data {
                admins.insert(admin, ());
            }
        })
    }

    /// Direct access to admin store (read-only)
    /// Allows complex operations on the admin store with a single borrow
    pub fn with_admins<F, R>(f: F) -> R
    where
        F: FnOnce(&StableBTreeMap<Principal, (), Memory>) -> R,
    {
        STABLE_ADMINS.with(|admins| f(&admins.borrow()))
    }

    /// Direct mutable access to admin store
    /// Allows complex operations on the admin store with a single mutable borrow
    pub fn with_admins_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut StableBTreeMap<Principal, (), Memory>) -> R,
    {
        STABLE_ADMINS.with(|admins| f(&mut admins.borrow_mut()))
    }
}

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

// Convenience functions that use AdminStore for backward compatibility
/// Check if principal is an admin (auto-bootstrap first caller)
pub fn is_admin(principal: &Principal) -> bool {
    AdminStore::is_admin(principal)
}

/// Add new admin (only superadmins can add admins)
pub fn add_admin(new_admin_principal: Principal) -> std::result::Result<(), Error> {
    AdminStore::add_admin(new_admin_principal)
}

/// Remove admin (only superadmins can remove admins)
pub fn remove_admin(admin_principal: Principal) -> std::result::Result<(), Error> {
    AdminStore::remove_admin(admin_principal)
}

/// List all admins
pub fn list_admins() -> Vec<Principal> {
    AdminStore::list_admins()
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
    AdminStore::export_admins_for_upgrade()
}

/// Import admins from stable storage after canister upgrade
pub fn import_admins_from_upgrade(admin_data: Vec<Principal>) {
    AdminStore::import_admins_from_upgrade(admin_data)
}
