use crate::canister_factory::types::*;
use crate::capsule_store::{CapsuleStore, Order};
use crate::memory::with_capsule_store;
use candid::Principal;

/// Ensure the caller owns a capsule (has a self-capsule where they are both subject and owner)
/// This function verifies that the caller principal has a capsule where they are the subject
/// and they are listed as an owner of that capsule
pub fn ensure_owner(caller: Principal) -> Result<(), String> {
    let user_ref = crate::types::PersonRef::Principal(caller);

    // MIGRATED: Find the user's self-capsule (where user is both subject and owner)
    let has_self_capsule = with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .iter()
            .any(|capsule| capsule.subject == user_ref && capsule.owners.contains_key(&user_ref))
    });

    if has_self_capsule {
        Ok(())
    } else {
        Err(format!(
            "Access denied: Principal {caller} does not own a capsule"
        ))
    }
}

/// Ensure the caller is an admin (can perform admin-only operations)
/// This function checks if the caller is either a superadmin or a regular admin
pub fn ensure_admin(caller: Principal) -> Result<(), String> {
    if crate::admin::is_admin(&caller) {
        Ok(())
    } else {
        Err(format!("Access denied: Principal {caller} is not an admin"))
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

    with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .iter()
            .any(|capsule| capsule.subject == user_ref && capsule.owners.contains_key(&user_ref))
    })
}

/// Get the capsule ID for a user (if they own one)
/// This is a utility function for migration operations
pub fn get_user_capsule_id(user: Principal) -> Option<String> {
    let user_ref = crate::types::PersonRef::Principal(user);

    with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .iter()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Capsule, OwnerState, PersonRef};
    use std::collections::HashMap;

    // Helper function to create a test principal
    fn test_principal(id: u8) -> Principal {
        Principal::from_slice(&[id; 29])
    }

    // Helper function to create a test capsule with given subject and owners
    fn create_test_capsule(id: &str, subject: PersonRef, owners: Vec<PersonRef>) -> Capsule {
        let mut owner_map = HashMap::new();
        for owner in owners {
            owner_map.insert(
                owner,
                OwnerState {
                    since: 1000000000,
                    last_activity_at: 1000000000,
                },
            );
        }

        Capsule {
            id: id.to_string(),
            subject,
            owners: owner_map,
            controllers: HashMap::new(),
            connections: HashMap::new(),
            connection_groups: HashMap::new(),
            memories: HashMap::new(),
            galleries: HashMap::new(),
            created_at: 1000000000,
            updated_at: 1000000000,
            bound_to_neon: false,
            inline_bytes_used: 0,
        }
    }

    // Helper function to setup test capsules in memory
    fn setup_test_capsules() {
        let user1 = test_principal(1);
        let user2 = test_principal(2);
        let user3 = test_principal(3);

        let user1_ref = PersonRef::Principal(user1);
        let user2_ref = PersonRef::Principal(user2);
        let user3_ref = PersonRef::Principal(user3);

        // User1 has a self-capsule (owns their own capsule)
        let capsule1 = create_test_capsule("capsule1", user1_ref.clone(), vec![user1_ref.clone()]);

        // User2 has a self-capsule
        let capsule2 = create_test_capsule("capsule2", user2_ref.clone(), vec![user2_ref.clone()]);

        // User3 does not have a self-capsule (they own a capsule about someone else)
        let other_subject = PersonRef::Opaque("other_person".to_string());
        let capsule3 = create_test_capsule("capsule3", other_subject, vec![user3_ref]);

        // Store capsules in memory
        crate::memory::with_capsule_store_mut(|capsules| {
            capsules.upsert("capsule1".to_string(), capsule1);
            capsules.upsert("capsule2".to_string(), capsule2);
            capsules.upsert("capsule3".to_string(), capsule3);
        });
    }

    // Helper function to setup test admins
    fn setup_test_admins() {
        let admin1 = test_principal(10);
        let admin2 = test_principal(11);

        crate::memory::with_admins_mut(|admins| {
            admins.insert(admin1);
            admins.insert(admin2);
        });
    }

    // Helper function to clear test data
    fn clear_test_data() {
        crate::memory::with_capsule_store_mut(|_capsules| {
            // Clear is not available on Store - just continue
        });
        crate::memory::with_admins_mut(|admins| {
            admins.clear();
        });
    }

    #[test]
    fn test_ensure_owner_with_valid_self_capsule() {
        clear_test_data();
        setup_test_capsules();

        let user1 = test_principal(1);
        let result = ensure_owner(user1);

        assert!(
            result.is_ok(),
            "User with self-capsule should pass ownership check"
        );

        clear_test_data();
    }

    #[test]
    fn test_ensure_owner_with_no_self_capsule() {
        clear_test_data();
        setup_test_capsules();

        let user3 = test_principal(3);
        let result = ensure_owner(user3);

        assert!(
            result.is_err(),
            "User without self-capsule should fail ownership check"
        );
        assert!(result.unwrap_err().contains("does not own a capsule"));

        clear_test_data();
    }

    #[test]
    fn test_ensure_owner_with_nonexistent_user() {
        clear_test_data();
        setup_test_capsules();

        let nonexistent_user = test_principal(99);
        let result = ensure_owner(nonexistent_user);

        assert!(
            result.is_err(),
            "Nonexistent user should fail ownership check"
        );
        assert!(result.unwrap_err().contains("does not own a capsule"));

        clear_test_data();
    }

    #[test]
    fn test_ensure_owner_with_anonymous_caller() {
        clear_test_data();
        setup_test_capsules();

        let anonymous = Principal::anonymous();
        let result = ensure_owner(anonymous);

        assert!(
            result.is_err(),
            "Anonymous caller should fail ownership check"
        );
        assert!(result.unwrap_err().contains("does not own a capsule"));

        clear_test_data();
    }

    #[test]
    fn test_ensure_admin_with_valid_admin() {
        clear_test_data();
        setup_test_admins();

        let admin1 = test_principal(10);
        let result = ensure_admin(admin1);

        assert!(result.is_ok(), "Valid admin should pass admin check");

        clear_test_data();
    }

    #[test]
    fn test_ensure_admin_with_non_admin() {
        clear_test_data();
        setup_test_admins();

        let non_admin = test_principal(50);
        let result = ensure_admin(non_admin);

        assert!(result.is_err(), "Non-admin should fail admin check");
        assert!(result.unwrap_err().contains("is not an admin"));

        clear_test_data();
    }

    #[test]
    fn test_ensure_admin_with_superadmin() {
        clear_test_data();

        // Test with the hardcoded superadmin principal
        let superadmin =
            Principal::from_text("otzfv-jscof-niinw-gtloq-25uz3-pglpg-u3kug-besf3-rzlbd-ylrmp-5ae")
                .unwrap();
        let result = ensure_admin(superadmin);

        assert!(result.is_ok(), "Superadmin should pass admin check");

        clear_test_data();
    }

    #[test]
    fn test_ensure_admin_with_anonymous_caller() {
        clear_test_data();
        setup_test_admins();

        let anonymous = Principal::anonymous();
        let result = ensure_admin(anonymous);

        assert!(result.is_err(), "Anonymous caller should fail admin check");
        assert!(result.unwrap_err().contains("is not an admin"));

        clear_test_data();
    }

    #[test]
    fn test_check_user_capsule_ownership_true() {
        clear_test_data();
        setup_test_capsules();

        let user1 = test_principal(1);
        let result = check_user_capsule_ownership(user1);

        assert!(result, "User with self-capsule should return true");

        clear_test_data();
    }

    #[test]
    fn test_check_user_capsule_ownership_false() {
        clear_test_data();
        setup_test_capsules();

        let user3 = test_principal(3);
        let result = check_user_capsule_ownership(user3);

        assert!(!result, "User without self-capsule should return false");

        clear_test_data();
    }

    #[test]
    fn test_check_user_capsule_ownership_nonexistent() {
        clear_test_data();
        setup_test_capsules();

        let nonexistent_user = test_principal(99);
        let result = check_user_capsule_ownership(nonexistent_user);

        assert!(!result, "Nonexistent user should return false");

        clear_test_data();
    }

    #[test]
    fn test_get_user_capsule_id_existing() {
        clear_test_data();
        setup_test_capsules();

        let user1 = test_principal(1);
        let result = get_user_capsule_id(user1);

        assert!(
            result.is_some(),
            "User with self-capsule should return capsule ID"
        );
        assert_eq!(result.unwrap(), "capsule1");

        clear_test_data();
    }

    #[test]
    fn test_get_user_capsule_id_nonexistent() {
        clear_test_data();
        setup_test_capsules();

        let user3 = test_principal(3);
        let result = get_user_capsule_id(user3);

        assert!(
            result.is_none(),
            "User without self-capsule should return None"
        );

        clear_test_data();
    }

    #[test]
    fn test_validate_and_prepare_config_valid_name() {
        let config = CreatePersonalCanisterConfig {
            name: Some("My Personal Canister".to_string()),
            subnet_id: None,
        };

        let result = validate_and_prepare_config(config);

        assert!(result.is_ok(), "Valid config should pass validation");
        let validated = result.unwrap();
        assert_eq!(validated.name, Some("My Personal Canister".to_string()));
    }

    #[test]
    fn test_validate_and_prepare_config_empty_name() {
        let config = CreatePersonalCanisterConfig {
            name: Some("".to_string()),
            subnet_id: None,
        };

        let result = validate_and_prepare_config(config);

        assert!(result.is_err(), "Empty name should fail validation");
        assert!(result.unwrap_err().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_and_prepare_config_long_name() {
        let long_name = "a".repeat(101);
        let config = CreatePersonalCanisterConfig {
            name: Some(long_name),
            subnet_id: None,
        };

        let result = validate_and_prepare_config(config);

        assert!(result.is_err(), "Long name should fail validation");
        assert!(result.unwrap_err().contains("cannot exceed 100 characters"));
    }

    #[test]
    fn test_validate_and_prepare_config_invalid_characters() {
        let config = CreatePersonalCanisterConfig {
            name: Some("Invalid@Name!".to_string()),
            subnet_id: None,
        };

        let result = validate_and_prepare_config(config);

        assert!(result.is_err(), "Invalid characters should fail validation");
        assert!(result
            .unwrap_err()
            .contains("can only contain alphanumeric"));
    }

    #[test]
    fn test_validate_and_prepare_config_valid_characters() {
        let config = CreatePersonalCanisterConfig {
            name: Some("Valid Name-123_test".to_string()),
            subnet_id: None,
        };

        let result = validate_and_prepare_config(config);

        assert!(result.is_ok(), "Valid characters should pass validation");
    }

    #[test]
    fn test_validate_and_prepare_config_anonymous_subnet() {
        let config = CreatePersonalCanisterConfig {
            name: None,
            subnet_id: Some(Principal::anonymous()),
        };

        let result = validate_and_prepare_config(config);

        assert!(
            result.is_err(),
            "Anonymous subnet ID should fail validation"
        );
        assert!(result
            .unwrap_err()
            .contains("cannot be anonymous principal"));
    }

    #[test]
    fn test_validate_and_prepare_config_valid_subnet() {
        let subnet_id = test_principal(20);
        let config = CreatePersonalCanisterConfig {
            name: None,
            subnet_id: Some(subnet_id),
        };

        let result = validate_and_prepare_config(config);

        assert!(result.is_ok(), "Valid subnet ID should pass validation");
        let validated = result.unwrap();
        assert_eq!(validated.subnet_id, Some(subnet_id));
    }

    #[test]
    fn test_validate_and_prepare_config_default_name() {
        let config = CreatePersonalCanisterConfig {
            name: None,
            subnet_id: None,
        };

        let result = validate_and_prepare_config(config);

        assert!(result.is_ok(), "Config without name should get default");
        let validated = result.unwrap();
        assert_eq!(validated.name, Some("Personal Canister".to_string()));
    }

    #[test]
    fn test_multiple_users_with_self_capsules() {
        clear_test_data();
        setup_test_capsules();

        let user1 = test_principal(1);
        let user2 = test_principal(2);

        let result1 = ensure_owner(user1);
        let result2 = ensure_owner(user2);

        assert!(result1.is_ok(), "User1 should pass ownership check");
        assert!(result2.is_ok(), "User2 should pass ownership check");

        clear_test_data();
    }

    #[test]
    fn test_access_control_guards_comprehensive() {
        clear_test_data();
        setup_test_capsules();
        setup_test_admins();

        // Test owner access
        let user1 = test_principal(1); // Has self-capsule
        let user3 = test_principal(3); // No self-capsule
        let admin1 = test_principal(10); // Is admin
        let non_admin = test_principal(50); // Not admin
        let anonymous = Principal::anonymous();

        // Owner checks
        assert!(
            ensure_owner(user1).is_ok(),
            "User with self-capsule should pass"
        );
        assert!(
            ensure_owner(user3).is_err(),
            "User without self-capsule should fail"
        );
        assert!(
            ensure_owner(anonymous).is_err(),
            "Anonymous should fail owner check"
        );

        // Admin checks
        assert!(
            ensure_admin(admin1).is_ok(),
            "Admin should pass admin check"
        );
        assert!(
            ensure_admin(non_admin).is_err(),
            "Non-admin should fail admin check"
        );
        assert!(
            ensure_admin(anonymous).is_err(),
            "Anonymous should fail admin check"
        );

        // Cross-validation: admin who is not owner
        assert!(
            ensure_admin(admin1).is_ok(),
            "Admin should pass admin check"
        );
        assert!(
            ensure_owner(admin1).is_err(),
            "Admin without self-capsule should fail owner check"
        );

        clear_test_data();
    }

    #[test]
    fn test_edge_cases_and_boundary_conditions() {
        clear_test_data();

        // Test with empty state (no capsules, no admins)
        let user = test_principal(1);
        assert!(ensure_owner(user).is_err(), "Should fail with no capsules");

        // First caller becomes admin due to auto-bootstrap
        assert!(
            ensure_admin(user).is_ok(),
            "First caller should become admin"
        );

        // Test with maximum length name (100 characters)
        let max_name = "a".repeat(100);
        let config = CreatePersonalCanisterConfig {
            name: Some(max_name.clone()),
            subnet_id: None,
        };
        let result = validate_and_prepare_config(config);
        assert!(result.is_ok(), "100 character name should be valid");
        assert_eq!(result.unwrap().name, Some(max_name));

        clear_test_data();
    }
}
