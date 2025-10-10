use crate::capsule::commands::capsules_create;
use crate::capsule_store::{types::PaginationOrder as Order, CapsuleStore};
use crate::gallery::api_types::{GalleryData, GalleryUpdateData};
use crate::gallery::domain::Gallery;
use crate::memory::{with_capsule_store, with_capsule_store_mut};
use crate::types::{Error, PersonRef};

/// Create a gallery in the caller's capsule (replaces store_gallery_forever)
pub fn galleries_create(gallery_data: GalleryData) -> std::result::Result<Gallery, Error> {
    let caller = PersonRef::from_caller();

    // Use the gallery ID provided by Web2 (don't generate new ID)
    let gallery_id = gallery_data.gallery.id.clone();

    // MIGRATED: Ensure caller has a capsule - create one if it doesn't exist
    let capsule = match with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller))
    }) {
        Some(capsule) => Some(capsule),
        None => {
            // No capsule found - create one automatically for first-time users
            match capsules_create(None) {
                Ok(capsule) => Some(capsule),
                Err(e) => {
                    return Err(Error::Internal(format!("Failed to create capsule: {e}")));
                }
            }
        }
    };

    match capsule {
        Some(mut capsule) => {
            // Check if gallery already exists with this UUID (idempotency)
            if let Some(existing_gallery) = capsule.galleries.get(&gallery_id) {
                return Ok(existing_gallery.clone());
            }

            // Create gallery from data (don't overwrite gallery.id - it's already set by Web2)
            let mut gallery = gallery_data.gallery;
            gallery.created_at = ic_cdk::api::time();
            gallery.updated_at = ic_cdk::api::time();
            // Note: owner_principal and storage_location are now handled through access_entries and metadata

            // Store gallery in capsule
            let gallery_clone = gallery.clone();
            capsule.galleries.insert(gallery_id.clone(), gallery);
            capsule.updated_at = ic_cdk::api::time(); // Update capsule timestamp

            // MIGRATED: Save updated capsule
            let capsule_id = capsule.id.clone();
            with_capsule_store_mut(|store| {
                store.upsert(capsule_id, capsule);
            });

            Ok(gallery_clone)
        }
        None => Err(Error::NotFound),
    }
}

pub fn galleries_create_with_memories(
    gallery_data: GalleryData,
    _sync_memories: bool,
) -> std::result::Result<Gallery, Error> {
    let caller = PersonRef::from_caller();

    // Use the gallery ID provided by Web2 (don't generate new ID)
    let gallery_id = gallery_data.gallery.id.clone();

    // MIGRATED: Ensure caller has a capsule - create one if it doesn't exist
    let capsule = match with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller))
    }) {
        Some(capsule) => Some(capsule),
        None => {
            // No capsule found - create one automatically for first-time users
            match capsules_create(None) {
                Ok(capsule) => Some(capsule),
                Err(e) => {
                    return Err(Error::Internal(format!("Failed to create capsule: {e}")));
                }
            }
        }
    };

    match capsule {
        Some(mut capsule) => {
            // Check if gallery already exists with this UUID (idempotency)
            if let Some(existing_gallery) = capsule.galleries.get(&gallery_id) {
                return Ok(existing_gallery.clone());
            }

            // Create gallery from data (don't overwrite gallery.id - it's already set by Web2)
            let mut gallery = gallery_data.gallery;
            gallery.created_at = ic_cdk::api::time();
            gallery.updated_at = ic_cdk::api::time();
            // Note: owner_principal and storage_location are now handled through access_entries and metadata

            // Store gallery in capsule
            let gallery_clone = gallery.clone();
            capsule.galleries.insert(gallery_id.clone(), gallery);
            capsule.updated_at = ic_cdk::api::time(); // Update capsule timestamp

            // MIGRATED: Save updated capsule
            with_capsule_store_mut(|store| {
                store.upsert(capsule.id.clone(), capsule);
            });

            Ok(gallery_clone)
        }
        None => Err(Error::NotFound),
    }
}

/// Update gallery storage location (replaces update_gallery_storage_location)
pub fn update_gallery_storage_location(
    gallery_id: String,
    new_location: Vec<crate::types::BlobHosting>,
) -> std::result::Result<(), Error> {
    let caller = PersonRef::from_caller();

    // MIGRATED: Find and update gallery in caller's self-capsule
    with_capsule_store_mut(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        let self_capsule = all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller));

        match self_capsule {
            Some(mut capsule) => {
                if let Some(gallery) = capsule.galleries.get_mut(&gallery_id) {
                    gallery.metadata.storage_location = new_location;
                    gallery.updated_at = ic_cdk::api::time();
                    capsule.updated_at = ic_cdk::api::time();

                    // Save updated capsule
                    let capsule_id = capsule.id.clone();
                    store.upsert(capsule_id, capsule);
                    Ok(())
                } else {
                    Err(Error::NotFound)
                }
            }
            None => Err(Error::NotFound),
        }
    })
}

/// Update a gallery in the caller's capsule (replaces update_gallery_forever)
pub fn galleries_update(
    gallery_id: String,
    update_data: GalleryUpdateData,
) -> std::result::Result<Gallery, Error> {
    let caller = PersonRef::from_caller();

    // MIGRATED: Find and update gallery in caller's self-capsule
    with_capsule_store_mut(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        let self_capsule = all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller));

        match self_capsule {
            Some(mut capsule) => {
                if let Some(gallery) = capsule.galleries.get_mut(&gallery_id) {
                    // Update gallery fields
                    if let Some(title) = update_data.title {
                        gallery.metadata.title = Some(title);
                    }
                    if let Some(description) = update_data.description {
                        gallery.metadata.description = Some(description);
                    }
                    if let Some(_is_public) = update_data.is_public {
                        // Note: This field might not exist in the new Gallery struct
                        // gallery.is_public = is_public;
                    }

                    gallery.updated_at = ic_cdk::api::time();
                    capsule.updated_at = ic_cdk::api::time();

                    // Save updated capsule
                    let capsule_id = capsule.id.clone();
                    let gallery_clone = gallery.clone();
                    store.upsert(capsule_id, capsule);
                    Ok(gallery_clone)
                } else {
                    Err(Error::NotFound)
                }
            }
            None => Err(Error::NotFound),
        }
    })
}

/// Delete a gallery from the caller's capsule (replaces delete_gallery_forever)
pub fn galleries_delete(gallery_id: String) -> std::result::Result<(), Error> {
    let caller = PersonRef::from_caller();

    // MIGRATED: Find and delete gallery from caller's self-capsule
    with_capsule_store_mut(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        let self_capsule = all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller));

        match self_capsule {
            Some(mut capsule) => {
                if capsule.galleries.remove(&gallery_id).is_some() {
                    capsule.updated_at = ic_cdk::api::time();

                    // Save updated capsule
                    let capsule_id = capsule.id.clone();
                    store.upsert(capsule_id, capsule);
                    Ok(())
                } else {
                    Err(Error::NotFound)
                }
            }
            None => Err(Error::NotFound),
        }
    })
}
