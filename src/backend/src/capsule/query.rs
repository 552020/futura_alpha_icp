use crate::capsule::domain::Capsule;
use crate::capsule_store::{types::PaginationOrder as Order, CapsuleStore};
use crate::memory::with_capsule_store;
use crate::types::*;

/// Get capsule by ID (with read access check)

pub fn capsules_read(capsule_id: String) -> std::result::Result<Capsule, Error> {
    let caller = PersonRef::from_caller();

    // MIGRATED: Using new trait-based API
    with_capsule_store(|store| {
        store
            .get(&capsule_id)
            .filter(|capsule| capsule.has_write_access(&caller))
            .ok_or(Error::NotFound)
    })
}

/// Get capsule info by ID (basic version with read access check)

pub fn capsules_read_basic(capsule_id: String) -> std::result::Result<CapsuleInfo, Error> {
    let caller = PersonRef::from_caller();

    // MIGRATED: Using new trait-based API
    with_capsule_store(|store| {
        store
            .get(&capsule_id)
            .filter(|capsule| capsule.has_write_access(&caller))
            .map(|capsule| CapsuleInfo {
                capsule_id: capsule.id.clone(),
                subject: capsule.subject.clone(),
                is_owner: capsule.owners.contains_key(&caller),
                is_controller: capsule.controllers.contains_key(&caller),
                is_self_capsule: capsule.subject == caller,
                bound_to_neon: capsule.bound_to_neon,
                created_at: capsule.created_at,
                updated_at: capsule.updated_at,

                // Add lightweight counts for summary information
                memory_count: capsule.memories.len() as u64,
                gallery_count: capsule.galleries.len() as u64,
                connection_count: capsule.connections.len() as u64,
            })
            .ok_or(Error::NotFound)
    })
}

/// Get caller's self-capsule (where caller is the subject)

pub fn capsule_read_self() -> std::result::Result<Capsule, Error> {
    let caller = PersonRef::from_caller();

    // MIGRATED: Find caller's self-capsule
    with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == caller)
            .ok_or(Error::NotFound)
    })
}

/// Get caller's self-capsule info (basic version)
pub fn capsule_read_self_basic() -> std::result::Result<CapsuleInfo, Error> {
    let caller = PersonRef::from_caller();

    // MIGRATED: Find caller's self-capsule and create basic info
    with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == caller)
            .map(|capsule| CapsuleInfo {
                capsule_id: capsule.id.clone(),
                subject: capsule.subject.clone(),
                is_owner: capsule.owners.contains_key(&caller),
                is_controller: capsule.controllers.contains_key(&caller),
                is_self_capsule: true,
                bound_to_neon: capsule.bound_to_neon,
                created_at: capsule.created_at,
                updated_at: capsule.updated_at,

                // Add lightweight counts for summary information
                memory_count: capsule.memories.len() as u64,
                gallery_count: capsule.galleries.len() as u64,
                connection_count: capsule.connections.len() as u64,
            })
            .ok_or(Error::NotFound)
    })
}

/// List capsules owned or controlled by caller
pub fn capsules_list() -> Vec<CapsuleHeader> {
    let caller = PersonRef::from_caller();

    // MIGRATED: Using new trait-based API with pagination
    // Note: Using large limit to get all capsules, maintaining backward compatibility
    with_capsule_store(|store| {
        let page = store.paginate(None, u32::MAX, Order::Asc);
        page.items
            .into_iter()
            .filter(|capsule| capsule.has_write_access(&caller))
            .map(|capsule| capsule.to_header())
            .collect()
    })
}

/// Get user settings for the caller's capsule
pub fn get_user_settings() -> std::result::Result<crate::types::UserSettingsResponse, Error> {
    let caller = PersonRef::from_caller();

    // Find the caller's capsule
    let capsule =
        with_capsule_store(|store| store.find_by_subject(&caller).ok_or(Error::NotFound))?;

    // Check if caller has read access
    if !capsule.has_read_access(&caller) {
        return Err(Error::Unauthorized);
    }

    // Return settings
    Ok(crate::types::UserSettingsResponse {
        has_advanced_settings: capsule.has_advanced_settings,
        hosting_preferences: capsule.hosting_preferences.clone(),
    })
}
