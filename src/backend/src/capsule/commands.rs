use crate::capsule::domain::Capsule;
use crate::capsule::util::calculate_capsule_size;
use crate::capsule_store::{types::PaginationOrder as Order, CapsuleStore};
use crate::memory::{with_capsule_store, with_capsule_store_mut};
use crate::state::add_canister_size;
use crate::types::*;

use ic_cdk::api::time;

/// Create a new capsule with optional subject
/// If subject is None, creates a self-capsule (subject = caller)
/// If subject is provided, creates a capsule for that subject
pub fn capsules_create(subject: Option<PersonRef>) -> std::result::Result<Capsule, Error> {
    let caller = PersonRef::from_caller();

    // Check if caller already has a self-capsule when creating self-capsule
    let is_self_capsule = subject.is_none();

    if is_self_capsule {
        // MIGRATED: Check if caller already has a self-capsule
        let all_capsules = with_capsule_store(|store| store.paginate(None, u32::MAX, Order::Asc));

        let existing_self_capsule = all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller));

        if let Some(capsule) = existing_self_capsule {
            // MIGRATED: Update existing self-capsule activity
            let capsule_id = capsule.id.clone();
            let update_result = with_capsule_store_mut(|store| {
                store.update(&capsule_id, |capsule| {
                    let now = time();
                    capsule.updated_at = now;

                    if let Some(owner_state) = capsule.owners.get_mut(&caller) {
                        owner_state.last_activity_at = now;
                    }
                })
            });

            if update_result.is_ok() {
                // Return the existing capsule
                return Ok(capsule);
            } else {
                return Err(Error::Internal(
                    "Failed to update capsule activity".to_string(),
                ));
            }
        }
    }

    // MIGRATED: Create new capsule
    let actual_subject = subject.unwrap_or_else(|| caller.clone());
    let capsule = Capsule::new(actual_subject, caller);
    let capsule_id = capsule.id.clone();

    // Track size before creating capsule
    let capsule_size = calculate_capsule_size(&capsule);
    if let Err(_e) = add_canister_size(capsule_size) {
        return Err(Error::ResourceExhausted);
    }

    // Use upsert to create new capsule (should succeed since we're checking for existing self-capsule above)
    with_capsule_store_mut(|store| {
        store.upsert(capsule_id.clone(), capsule.clone());
    });

    Ok(capsule)
}

/// Update a capsule with the provided data
/// Only allows updates to mutable fields (binding status, timestamps)
pub fn capsules_update(
    capsule_id: String,
    updates: CapsuleUpdateData,
) -> std::result::Result<Capsule, Error> {
    let caller = PersonRef::from_caller();

    // First check if the capsule exists and caller has access
    let _capsule = with_capsule_store(|store| {
        store
            .get(&capsule_id)
            .filter(|capsule| capsule.has_write_access(&caller))
            .ok_or(Error::NotFound)
    })?;

    // Update the capsule
    with_capsule_store_mut(|store| {
        store.update(&capsule_id, |capsule| {
            // Update mutable fields
            if let Some(bound_to_neon) = updates.bound_to_neon {
                capsule.bound_to_neon = bound_to_neon;
            }

            // Update timestamp
            capsule.updated_at = time();

            // Update owner activity
            if let Some(owner_state) = capsule.owners.get_mut(&caller) {
                owner_state.last_activity_at = time();
            }
        })
    })?;

    // Return the updated capsule
    with_capsule_store(|store| store.get(&capsule_id).ok_or(Error::NotFound))
}

/// Delete a capsule (permanent deletion)
/// Only allows deletion by capsule owners
pub fn capsules_delete(capsule_id: String) -> std::result::Result<(), Error> {
    let caller = PersonRef::from_caller();

    // First, get the capsule to check ownership and calculate size for tracking
    let capsule = with_capsule_store(|store| {
        store
            .get(&capsule_id)
            .filter(|capsule| capsule.has_write_access(&caller))
            .ok_or(Error::NotFound)
    })?;

    // Calculate size for tracking removal
    let _capsule_size = calculate_capsule_size(&capsule);

    // Remove from storage
    with_capsule_store_mut(|store| store.remove(&capsule_id).ok_or(Error::NotFound))?;

    // Track size removal (note: we don't have remove_canister_size implemented yet)
    // For now, we'll just log this - in a full implementation, we'd subtract from total size
    // TODO: Implement remove_canister_size in state.rs

    Ok(())
}

/// Flexible resource binding function for Neon database
/// Can bind capsules, galleries, or memories to Neon
pub fn resources_bind_neon(
    resource_type: ResourceType,
    resource_id: String,
    bind: bool,
) -> std::result::Result<(), Error> {
    let caller_ref = PersonRef::from_caller();

    match resource_type {
        ResourceType::Capsule => {
            // MIGRATED: Bind specific capsule if caller owns it
            with_capsule_store_mut(|store| {
                let update_result = store.update(&resource_id, |capsule| {
                    if capsule.owners.contains_key(&caller_ref) {
                        capsule.bound_to_neon = bind;
                        capsule.updated_at = time();

                        // Update owner activity
                        if let Some(owner_state) = capsule.owners.get_mut(&caller_ref) {
                            owner_state.last_activity_at = time();
                        }
                    }
                });
                if update_result.is_ok() {
                    Ok(())
                } else {
                    Err(crate::types::Error::Internal(
                        "Failed to update capsule".to_string(),
                    ))
                }
            })
        }
        ResourceType::Gallery => {
            // MIGRATED: Bind specific gallery if caller owns the capsule containing it
            with_capsule_store_mut(|store| {
                let all_capsules = store.paginate(None, u32::MAX, Order::Asc);

                for capsule in all_capsules.items {
                    if capsule.owners.contains_key(&caller_ref)
                        && capsule.galleries.contains_key(&resource_id)
                    {
                        // Found the capsule containing the gallery
                        let update_result = store.update(&capsule.id, |capsule| {
                            if let Some(_gallery) = capsule.galleries.get_mut(&resource_id) {
                                // Note: bound_to_neon field no longer exists in new Gallery struct
                                capsule.updated_at = time();

                                // Update owner activity
                                if let Some(owner_state) = capsule.owners.get_mut(&caller_ref) {
                                    owner_state.last_activity_at = time();
                                }
                            }
                        });
                        return if update_result.is_ok() {
                            Ok(())
                        } else {
                            Err(crate::types::Error::Internal(
                                "Failed to update gallery".to_string(),
                            ))
                        };
                    }
                }
                Err(crate::types::Error::NotFound)
            })
        }
        ResourceType::Memory => {
            // MIGRATED: Bind specific memory if caller owns the capsule containing it
            with_capsule_store_mut(|store| {
                let all_capsules = store.paginate(None, u32::MAX, Order::Asc);

                for capsule in all_capsules.items {
                    if capsule.owners.contains_key(&caller_ref)
                        && capsule.memories.contains_key(&resource_id)
                    {
                        // Found the capsule containing the memory
                        let update_result = store.update(&capsule.id, |capsule| {
                            if let Some(memory) = capsule.memories.get_mut(&resource_id) {
                                // Update the database_storage_edges field in the memory's info
                                if bind {
                                    // Add Neon to storage edges if not already present
                                    if !memory
                                        .metadata
                                        .database_storage_edges
                                        .contains(&StorageEdgeDatabaseType::Neon)
                                    {
                                        memory
                                            .metadata
                                            .database_storage_edges
                                            .push(StorageEdgeDatabaseType::Neon);
                                    }
                                } else {
                                    // Remove Neon from storage edges
                                    memory
                                        .metadata
                                        .database_storage_edges
                                        .retain(|edge| *edge != StorageEdgeDatabaseType::Neon);
                                }

                                capsule.updated_at = time();

                                // Update owner activity
                                if let Some(owner_state) = capsule.owners.get_mut(&caller_ref) {
                                    owner_state.last_activity_at = time();
                                }
                            }
                        });
                        return if update_result.is_ok() {
                            Ok(())
                        } else {
                            Err(crate::types::Error::Internal(
                                "Failed to update memory".to_string(),
                            ))
                        };
                    }
                }
                Err(crate::types::Error::NotFound)
            })
        }
    }
}

/// Update user settings for the caller's capsule
pub fn update_user_settings(
    updates: crate::types::UserSettingsUpdateData,
) -> std::result::Result<crate::types::UserSettingsResponse, Error> {
    let caller = PersonRef::from_caller();

    // Find the caller's capsule
    let capsule_id = with_capsule_store(|store| {
        store
            .find_by_subject(&caller)
            .map(|capsule| capsule.id.clone())
            .ok_or(Error::NotFound)
    })?;

    // Update the capsule settings
    with_capsule_store_mut(|store| {
        store.update_with(&capsule_id, |capsule| {
            // Check if caller has write access
            if !capsule.has_write_access(&caller) {
                return Err(Error::Unauthorized);
            }

            // Update settings if provided
            if let Some(has_advanced_settings) = updates.has_advanced_settings {
                capsule.has_advanced_settings = has_advanced_settings;
            }

            // Update timestamp
            capsule.updated_at = time();

            // Update owner activity
            if let Some(owner_state) = capsule.owners.get_mut(&caller) {
                owner_state.last_activity_at = time();
            }

            Ok(())
        })
    })?;

    // Return the updated settings
    let updated_capsule =
        with_capsule_store(|store| store.get(&capsule_id).ok_or(Error::NotFound))?;

    Ok(crate::types::UserSettingsResponse {
        has_advanced_settings: updated_capsule.has_advanced_settings,
        hosting_preferences: updated_capsule.hosting_preferences.clone(),
    })
}
