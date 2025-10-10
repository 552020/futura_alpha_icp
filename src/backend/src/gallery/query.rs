use crate::capsule_store::{types::PaginationOrder as Order, CapsuleStore};
use crate::memory::with_capsule_store;
use crate::types::PersonRef;
use crate::gallery::domain::GalleryHeader;

/// Get all galleries for the caller (replaces get_user_galleries)
pub fn galleries_list() -> Vec<GalleryHeader> {
    let caller = PersonRef::from_caller();

    // MIGRATED: List all galleries from caller's self-capsule
    with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        let self_capsule = all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller));

        match self_capsule {
            Some(capsule) => {
                let mut gallery_headers: Vec<GalleryHeader> = capsule
                    .galleries
                    .values()
                    .map(|gallery| gallery.to_header())
                    .collect();

                // Sort by updated_at descending (most recent first)
                gallery_headers.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

                gallery_headers
            }
            None => Vec::new(), // No capsule found - return empty list
        }
    })
}
