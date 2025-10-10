use crate::capsule::domain::{Capsule, PersonRef};
use crate::capsule_store::{types::PaginationOrder as Order, CapsuleStore};
use crate::memory::{with_capsule_store, with_capsule_store_mut};
use crate::types::Error;
use ic_cdk::api::time;
use ic_stable_structures::Storable;

/// Calculate the serialized size of a capsule
pub fn calculate_capsule_size(capsule: &Capsule) -> u64 {
    let bytes = capsule.to_bytes();
    bytes.len() as u64
}

/// Find a self-capsule for a given caller (where caller is both subject and owner)
pub fn find_self_capsule(caller: &PersonRef) -> Option<Capsule> {
    let all_capsules = with_capsule_store(|store| store.paginate(None, u32::MAX, Order::Asc));
    all_capsules
        .items
        .into_iter()
        .find(|capsule| capsule.subject == *caller && capsule.owners.contains_key(caller))
}

/// Update a capsule's activity timestamp for a specific owner
pub fn update_capsule_activity(
    capsule_id: &str,
    caller: &PersonRef,
) -> std::result::Result<(), Error> {
    let now = time();

    with_capsule_store_mut(|store| {
        store.update(&capsule_id.to_string(), |capsule| {
            if let Some(owner_state) = capsule.owners.get_mut(caller) {
                owner_state.last_activity_at = now;
            }
            capsule.updated_at = now;
        })
    })
    .map_err(|_| crate::types::Error::Internal("Failed to update capsule activity".to_string()))
}

/// Export all capsules for upgrade persistence
#[allow(dead_code)]
pub fn export_capsules_for_upgrade() -> Vec<(String, Capsule)> {
    // MIGRATED: Export all capsules using pagination
    with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .map(|capsule| (capsule.id.clone(), capsule))
            .collect()
    })
}

/// Import capsules from upgrade persistence
#[allow(dead_code)]
pub fn import_capsules_from_upgrade(capsule_data: Vec<(String, Capsule)>) {
    with_capsule_store_mut(|store| {
        for (id, capsule) in capsule_data {
            store.upsert(id, capsule);
        }
    })
}
