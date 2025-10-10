// Folder Query Module
// Read operations for folders

use crate::capsule_store::{types::PaginationOrder as Order, CapsuleStore};
use crate::folder::domain::FolderHeader;
use crate::memory::with_capsule_store;
use crate::types::PersonRef;

/// Get all folders for the caller
pub fn folders_list() -> Vec<FolderHeader> {
    let caller = PersonRef::from_caller();

    // List all folders from caller's self-capsule
    with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        let self_capsule = all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller));

        match self_capsule {
            Some(capsule) => {
                let mut folder_headers: Vec<FolderHeader> = capsule
                    .folders
                    .values()
                    .map(|folder| folder.to_header())
                    .collect();

                // Sort by updated_at descending (most recent first)
                folder_headers.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

                folder_headers
            }
            None => Vec::new(), // No capsule found - return empty list
        }
    })
}

