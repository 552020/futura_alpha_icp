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
                    if let Some(is_public) = update_data.is_public {
                        apply_visibility(gallery, is_public, ic_cdk::api::time());
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

/// Apply a public/private change to a gallery.
///
/// `metadata.sharing_status` is what clients read today (the web app sets it
/// at creation and leaves access_entries empty), and the public access entry
/// is what the AccessControlled impl evaluates. Both are kept in step:
/// - public: one public Guest entry (VIEW + DOWNLOAD), status Public
/// - private: public entries removed; status Shared if non-owner entries
///   remain, otherwise Private
fn apply_visibility(gallery: &mut Gallery, is_public: bool, now: u64) {
    use crate::capsule::domain::{
        AccessCondition, AccessEntry, GrantSource, Perm, ResourceRole, SharingStatus,
    };

    if is_public {
        if !gallery.access_entries.iter().any(|e| e.is_public) {
            gallery.access_entries.push(AccessEntry {
                id: format!("public_{}", gallery.id),
                person_ref: None,
                is_public: true,
                grant_source: GrantSource::User,
                source_id: None,
                role: ResourceRole::Guest,
                perm_mask: (Perm::VIEW | Perm::DOWNLOAD).bits(),
                invited_by_person_ref: None,
                created_at: now,
                updated_at: now,
                condition: AccessCondition::Immediate,
            });
        }
        gallery.metadata.sharing_status = SharingStatus::Public;
    } else {
        gallery.access_entries.retain(|e| !e.is_public);
        let shared = gallery
            .access_entries
            .iter()
            .any(|e| e.person_ref.is_some() && e.role != ResourceRole::Owner);
        gallery.metadata.sharing_status = if shared {
            SharingStatus::Shared
        } else {
            SharingStatus::Private
        };
    }
    gallery.metadata.shared_count = gallery
        .access_entries
        .iter()
        .filter(|e| e.role != ResourceRole::Owner)
        .count() as u32;
}

#[cfg(test)]
mod visibility_tests {
    use super::apply_visibility;
    use crate::capsule::domain::{
        AccessCondition, AccessEntry, GrantSource, ResourceRole, SharingStatus,
    };
    use crate::gallery::domain::Gallery;
    use crate::types::PersonRef;
    use candid::Principal;

    fn gallery() -> Gallery {
        Gallery {
            id: "g1".into(),
            capsule_id: "c1".into(),
            metadata: crate::gallery::domain::GalleryMetadata {
                name: "g".into(),
                title: Some("G".into()),
                description: None,
                sharing_status: SharingStatus::Private,
                storage_location: vec![],
                total_memories: 0,
                shared_count: 0,
            },
            cover_memory_id: None,
            created_at: 0,
            updated_at: 0,
            items: vec![],
            access_entries: vec![],
        }
    }

    fn person_entry(role: ResourceRole) -> AccessEntry {
        AccessEntry {
            id: format!("{:?}", role),
            person_ref: Some(PersonRef::Principal(Principal::anonymous())),
            is_public: false,
            grant_source: GrantSource::User,
            source_id: None,
            role,
            perm_mask: 1,
            invited_by_person_ref: None,
            created_at: 0,
            updated_at: 0,
            condition: AccessCondition::Immediate,
        }
    }

    #[test]
    fn making_public_adds_one_public_entry() {
        let mut g = gallery();
        apply_visibility(&mut g, true, 1);
        apply_visibility(&mut g, true, 2); // idempotent
        assert_eq!(g.metadata.sharing_status, SharingStatus::Public);
        assert_eq!(g.access_entries.iter().filter(|e| e.is_public).count(), 1);
    }

    #[test]
    fn making_private_removes_public_entries() {
        let mut g = gallery();
        g.metadata.sharing_status = SharingStatus::Public; // as the web app creates it
        apply_visibility(&mut g, true, 1);
        apply_visibility(&mut g, false, 2);
        assert_eq!(g.metadata.sharing_status, SharingStatus::Private);
        assert!(g.access_entries.is_empty());
        assert_eq!(g.metadata.shared_count, 0);
    }

    #[test]
    fn private_with_guests_is_shared_and_owner_does_not_count() {
        let mut g = gallery();
        g.access_entries.push(person_entry(ResourceRole::Owner));
        g.access_entries.push(person_entry(ResourceRole::Guest));
        apply_visibility(&mut g, true, 1);
        apply_visibility(&mut g, false, 2);
        assert_eq!(g.metadata.sharing_status, SharingStatus::Shared);
        assert_eq!(g.access_entries.len(), 2);
        assert_eq!(g.metadata.shared_count, 1);
    }
}
