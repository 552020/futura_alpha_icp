// Folder Commands Module
// Write operations for folders

use crate::capsule_store::{types::PaginationOrder as Order, CapsuleStore};
use crate::folder::api_types::{FolderData, FolderUpdateData};
use crate::folder::domain::Folder;
use crate::memory::{with_capsule_store, with_capsule_store_mut};
use crate::types::{Error, PersonRef};

/// Create a folder in the caller's capsule
pub fn folders_create(folder_data: FolderData) -> std::result::Result<Folder, Error> {
    let caller = PersonRef::from_caller();

    // Use the folder ID provided by Web2 (don't generate new ID)
    let folder_id = folder_data.folder.id.clone();

    // Ensure caller has a capsule - create one if it doesn't exist
    let capsule = match with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller))
    }) {
        Some(capsule) => capsule,
        None => {
            // Create a new capsule for the caller
            crate::capsule::commands::capsules_create(Some(caller.clone()))?;
            with_capsule_store(|store| {
                let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
                all_capsules
                    .items
                    .into_iter()
                    .find(|capsule| {
                        capsule.subject == caller && capsule.owners.contains_key(&caller)
                    })
                    .ok_or(Error::NotFound)
            })?
        }
    };

    // Check if folder already exists with this UUID (idempotency)
    if let Some(existing_folder) = capsule.folders.get(&folder_id) {
        return Ok(existing_folder.clone());
    }

    // Create folder from data
    let mut folder = folder_data.folder;
    folder.created_at = ic_cdk::api::time();
    folder.updated_at = ic_cdk::api::time();

    // Store folder in capsule
    let folder_clone = folder.clone();
    with_capsule_store_mut(|store| {
        let mut capsule = store.get(&capsule.id).ok_or(Error::NotFound)?;
        capsule.folders.insert(folder_id.clone(), folder);
        capsule.updated_at = ic_cdk::api::time();
        store.upsert(capsule.id.clone(), capsule);
        Ok(())
    })?;

    Ok(folder_clone)
}

/// Update a folder in the caller's capsule
pub fn folders_update(
    folder_id: String,
    update_data: FolderUpdateData,
) -> std::result::Result<Folder, Error> {
    let caller = PersonRef::from_caller();

    // Find and update folder in caller's self-capsule
    with_capsule_store_mut(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        let self_capsule = all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller));

        match self_capsule {
            Some(mut capsule) => {
                if let Some(folder) = capsule.folders.get_mut(&folder_id) {
                    // Update folder fields
                    if let Some(title) = update_data.title {
                        folder.metadata.title = Some(title);
                    }
                    if let Some(description) = update_data.description {
                        folder.metadata.description = Some(description);
                    }

                    folder.updated_at = ic_cdk::api::time();
                    capsule.updated_at = ic_cdk::api::time();

                    // Save updated capsule
                    let capsule_id = capsule.id.clone();
                    let folder_clone = folder.clone();
                    store.upsert(capsule_id, capsule);
                    Ok(folder_clone)
                } else {
                    Err(Error::NotFound)
                }
            }
            None => Err(Error::NotFound),
        }
    })
}

/// Delete a folder from the caller's capsule
pub fn folders_delete(folder_id: String) -> std::result::Result<(), Error> {
    let caller = PersonRef::from_caller();

    // Find and delete folder from caller's self-capsule
    with_capsule_store_mut(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        let self_capsule = all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller));

        match self_capsule {
            Some(mut capsule) => {
                if capsule.folders.remove(&folder_id).is_some() {
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
