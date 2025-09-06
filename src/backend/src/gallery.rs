// ============================================================================
// GALLERY MANAGEMENT FUNCTIONS
// ============================================================================

use crate::capsule::capsules_create;
use crate::capsule_store::{types::PaginationOrder as Order, CapsuleStore};
use crate::memory::{with_capsule_store, with_capsule_store_mut};
use crate::types::{
    CapsuleCreationResult, Error, Gallery, GalleryData, GalleryStorageLocation, GalleryUpdateData,
    PersonRef, Result,
};

/// Create a gallery in the caller's capsule (replaces store_gallery_forever)
pub fn galleries_create(gallery_data: GalleryData) -> Result<Gallery> {
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
                CapsuleCreationResult { success: true, .. } => {
                    // MIGRATED: Now get the newly created capsule
                    with_capsule_store(|store| {
                        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
                        all_capsules.items.into_iter().find(|capsule| {
                            capsule.subject == caller && capsule.owners.contains_key(&caller)
                        })
                    })
                }
                CapsuleCreationResult {
                    success: false,
                    message,
                    ..
                } => {
                    return Err(Error::Internal(format!(
                        "Failed to create capsule: {message}"
                    )));
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
            gallery.owner_principal = match caller {
                PersonRef::Principal(p) => p,
                PersonRef::Opaque(_) => {
                    return Err(Error::InvalidArgument(
                        "Only principals can store galleries".to_string(),
                    ));
                }
            };
            gallery.created_at = ic_cdk::api::time();
            gallery.updated_at = ic_cdk::api::time();
            gallery.storage_location = GalleryStorageLocation::ICPOnly;

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

/// Create a gallery with memories in the caller's capsule (replaces store_gallery_forever_with_memories)
pub fn galleries_create_with_memories(
    gallery_data: GalleryData,
    sync_memories: bool,
) -> Result<Gallery> {
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
                CapsuleCreationResult { success: true, .. } => {
                    // MIGRATED: Now get the newly created capsule
                    with_capsule_store(|store| {
                        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
                        all_capsules.items.into_iter().find(|capsule| {
                            capsule.subject == caller && capsule.owners.contains_key(&caller)
                        })
                    })
                }
                CapsuleCreationResult {
                    success: false,
                    message,
                    ..
                } => {
                    return Err(Error::Internal(format!(
                        "Failed to create capsule: {message}"
                    )));
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
            gallery.owner_principal = match caller {
                PersonRef::Principal(p) => p,
                PersonRef::Opaque(_) => {
                    return Err(Error::InvalidArgument(
                        "Only principals can store galleries".to_string(),
                    ));
                }
            };
            gallery.created_at = ic_cdk::api::time();
            gallery.updated_at = ic_cdk::api::time();

            // Set storage location based on whether memories will be synced
            let storage_location = if sync_memories {
                GalleryStorageLocation::Both // Will be updated after memory sync
            } else {
                GalleryStorageLocation::ICPOnly
            };
            gallery.storage_location = storage_location.clone();

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

/// Get all galleries for the caller (replaces get_user_galleries)
pub fn galleries_list() -> Result<Vec<Gallery>> {
    let caller = PersonRef::from_caller();

    // MIGRATED: List all galleries from caller's self-capsule
    Ok(with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .filter(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller))
            .flat_map(|capsule| capsule.galleries.values().cloned().collect::<Vec<_>>())
            .collect()
    }))
}

/// Get gallery by ID from caller's capsule (replaces get_gallery_by_id)
pub fn galleries_read(gallery_id: String) -> Result<Gallery> {
    let caller = PersonRef::from_caller();

    // MIGRATED: Find gallery in caller's self-capsule
    with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller))
            .and_then(|capsule| capsule.galleries.get(&gallery_id).cloned())
            .ok_or(Error::NotFound)
    })
}

/// Update gallery storage location after memory synchronization
pub fn update_gallery_storage_location(
    gallery_id: String,
    new_location: GalleryStorageLocation,
) -> Result<()> {
    let caller = PersonRef::from_caller();

    // MIGRATED: Update gallery storage status in caller's self-capsule
    with_capsule_store_mut(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);

        // Find the capsule containing the gallery
        if let Some(capsule) = all_capsules.items.into_iter().find(|capsule| {
            capsule.subject == caller
                && capsule.owners.contains_key(&caller)
                && capsule.galleries.contains_key(&gallery_id)
        }) {
            let capsule_id = capsule.id.clone();

            // Update the capsule with the new gallery location
            let update_result = store.update(&capsule_id, |capsule| {
                if let Some(gallery) = capsule.galleries.get_mut(&gallery_id) {
                    gallery.storage_location = new_location;
                    gallery.updated_at = ic_cdk::api::time();
                    capsule.updated_at = ic_cdk::api::time(); // Update capsule timestamp
                }
            });

            if update_result.is_ok() {
                Ok(())
            } else {
                Err(crate::types::Error::Internal(
                    "Failed to update gallery storage location".to_string(),
                ))
            }
        } else {
            Err(crate::types::Error::NotFound)
        }
    })
}

/// Update a gallery in caller's capsule (replaces update_gallery)
pub fn galleries_update(gallery_id: String, update_data: GalleryUpdateData) -> Result<Gallery> {
    let caller = PersonRef::from_caller();

    // MIGRATED: Find and update gallery in caller's self-capsule
    let mut updated_gallery: Option<Gallery> = None;

    with_capsule_store_mut(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);

        // Find the capsule containing the gallery
        if let Some(capsule) = all_capsules.items.into_iter().find(|capsule| {
            capsule.subject == caller
                && capsule.owners.contains_key(&caller)
                && capsule.galleries.contains_key(&gallery_id)
        }) {
            let capsule_id = capsule.id.clone();

            // Update the capsule with the modified gallery
            let update_result = store.update(&capsule_id, |capsule| {
                if let Some(gallery) = capsule.galleries.get(&gallery_id) {
                    // Update gallery fields
                    let mut gallery_clone = gallery.clone();
                    if let Some(title) = update_data.title.clone() {
                        gallery_clone.title = title;
                    }
                    if let Some(description) = update_data.description.clone() {
                        gallery_clone.description = Some(description);
                    }
                    if let Some(is_public) = update_data.is_public {
                        gallery_clone.is_public = is_public;
                    }
                    if let Some(memory_entries) = update_data.memory_entries.clone() {
                        gallery_clone.memory_entries = memory_entries;
                    }

                    gallery_clone.updated_at = ic_cdk::api::time();

                    // Store updated gallery
                    capsule.galleries.insert(gallery_id, gallery_clone.clone());
                    capsule.updated_at = ic_cdk::api::time(); // Update capsule timestamp

                    updated_gallery = Some(gallery_clone);
                }
            });

            if update_result.is_err() {
                updated_gallery = None;
            }
        }
    });

    match updated_gallery {
        Some(gallery) => Ok(gallery),
        None => Err(Error::NotFound),
    }
}

/// Delete a gallery from caller's capsule (replaces delete_gallery)
pub fn galleries_delete(gallery_id: String) -> Result<()> {
    let caller = PersonRef::from_caller();

    // MIGRATED: Find and delete gallery from caller's self-capsule
    let mut gallery_found = false;

    with_capsule_store_mut(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);

        // Find the capsule containing the gallery
        if let Some(capsule) = all_capsules.items.into_iter().find(|capsule| {
            capsule.subject == caller
                && capsule.owners.contains_key(&caller)
                && capsule.galleries.contains_key(&gallery_id)
        }) {
            let capsule_id = capsule.id.clone();

            // Update the capsule to remove the gallery
            let update_result = store.update(&capsule_id, |capsule| {
                if capsule.galleries.contains_key(&gallery_id) {
                    capsule.galleries.remove(&gallery_id);
                    capsule.updated_at = ic_cdk::api::time(); // Update capsule timestamp
                    gallery_found = true;
                }
            });

            if update_result.is_err() {
                gallery_found = false;
            }
        }
    });

    if gallery_found {
        Ok(())
    } else {
        Err(Error::NotFound)
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Estimate the size of a gallery data structure
/// This provides a rough estimate for debugging the 8192-byte stable memory limit issue
pub fn estimate_gallery_size(gallery: &Gallery) -> u64 {
    let mut size = 0u64;

    // Basic gallery fields
    size += gallery.id.len() as u64; // ID string
    size += 32; // owner_principal (Principal is ~32 bytes)
    size += gallery.title.len() as u64; // title string
    size += gallery.description.as_ref().map_or(0, |d| d.len() as u64); // description
    size += 1; // is_public (bool)
    size += 8; // created_at (u64)
    size += 8; // updated_at (u64)
    size += 1; // storage_status (enum)
    size += 1; // bound_to_neon (bool)

    // Memory entries
    size += gallery.memory_entries.len() as u64 * 200; // Rough estimate per memory entry

    size
}

/// Estimate the size of a gallery when stored in a capsule context
/// This includes the overhead of the capsule structure
pub fn estimate_gallery_capsule_size(gallery: &Gallery) -> u64 {
    let gallery_size = estimate_gallery_size(gallery);

    // Add capsule overhead
    let capsule_overhead = 1024; // Rough estimate for capsule metadata, owners, etc.

    gallery_size + capsule_overhead
}

/// Get a human-readable size report for a gallery
pub fn get_gallery_size_report(gallery: &Gallery) -> String {
    let gallery_size = estimate_gallery_size(gallery);
    let capsule_size = estimate_gallery_capsule_size(gallery);
    let memory_entries_count = gallery.memory_entries.len();

    format!(
        "Gallery '{}': ~{} bytes (gallery only), ~{} bytes (in capsule), {} memory entries. {} bytes over 8192 limit.",
        gallery.id,
        gallery_size,
        capsule_size,
        memory_entries_count,
        if capsule_size > 8192 { capsule_size - 8192 } else { 0 }
    )
}

/// Get detailed size breakdown for a gallery
pub fn get_gallery_size_breakdown(gallery: &Gallery) -> GallerySizeInfo {
    let gallery_size = estimate_gallery_size(gallery);
    let capsule_size = estimate_gallery_capsule_size(gallery);
    let memory_entries_count = gallery.memory_entries.len();
    let memory_entries_size = (memory_entries_count * 200) as u64; // Rough estimate per entry

    GallerySizeInfo {
        total_size: capsule_size,
        gallery_id: gallery.id.clone(),
        memory_entries_count: memory_entries_count as u32,
        estimated_memory_entries_size: memory_entries_size,
        is_over_limit: capsule_size > 8192,
        over_limit_by: if capsule_size > 8192 {
            capsule_size - 8192
        } else {
            0
        },
    }
}

/// Size information for a gallery
#[derive(Debug, Clone, candid::CandidType, candid::Deserialize, serde::Serialize)]
pub struct GallerySizeInfo {
    pub total_size: u64,
    pub gallery_id: String,
    pub memory_entries_count: u32,
    pub estimated_memory_entries_size: u64,
    pub is_over_limit: bool,
    pub over_limit_by: u64,
}
