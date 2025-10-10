//! Memory management module with decoupled architecture
//!
//! This module provides the canister-facing functions that route through
//! the decoupled core functions for better testability and maintainability.

use crate::capsule_store::{types::PaginationOrder as Order, CapsuleStore};
use crate::memory::{with_capsule_store, with_capsule_store_mut};
use crate::types::{CapsuleId, Error, Memory, MemoryId, PersonRef};

// ============================================================================
// CANISTER ENVIRONMENT AND STORE ADAPTER
// ============================================================================

/// Canister environment implementation for production use
pub struct CanisterEnv;

impl crate::memories::core::Env for CanisterEnv {
    fn caller(&self) -> PersonRef {
        PersonRef::Principal(ic_cdk::api::msg_caller())
    }

    fn now(&self) -> u64 {
        ic_cdk::api::time()
    }
}

/// Production store adapter that bridges the Store trait with CapsuleStore
pub struct StoreAdapter;

impl crate::memories::core::Store for StoreAdapter {
    // Removed unused method: get_capsule_mut

    fn insert_memory(
        &mut self,
        capsule: &CapsuleId,
        memory: Memory,
    ) -> std::result::Result<(), Error> {
        with_capsule_store_mut(|store| {
            match store.update_with(capsule, |capsule_data| {
                // Check if memory already exists
                if capsule_data.memories.contains_key(&memory.id) {
                    return Err(Error::Conflict(format!(
                        "Memory {} already exists in capsule {}",
                        memory.id, capsule
                    )));
                }

                // Insert the memory
                capsule_data.memories.insert(memory.id.clone(), memory);
                Ok(())
            }) {
                Ok(_) => Ok(()),
                Err(e) => Err(Error::Internal(format!("Failed to insert memory: {:?}", e))),
            }
        })
    }

    fn get_memory(&self, capsule: &CapsuleId, id: &MemoryId) -> Option<Memory> {
        with_capsule_store(|store| {
            store
                .get(capsule)
                .and_then(|capsule| capsule.memories.get(id).cloned())
        })
    }

    fn delete_memory(
        &mut self,
        capsule: &CapsuleId,
        id: &MemoryId,
    ) -> std::result::Result<(), Error> {
        with_capsule_store_mut(|store| {
            match store.update_with(capsule, |capsule_data| {
                match capsule_data.memories.remove(id) {
                    Some(_) => Ok(()),
                    None => Err(Error::NotFound),
                }
            }) {
                Ok(_) => Ok(()),
                Err(e) => Err(Error::Internal(format!("Failed to delete memory: {:?}", e))),
            }
        })
    }

    fn update_memory(
        &mut self,
        capsule: &CapsuleId,
        id: &MemoryId,
        memory: Memory,
    ) -> std::result::Result<(), Error> {
        with_capsule_store_mut(|store| {
            match store.update_with(capsule, |capsule_data| {
                if capsule_data.memories.contains_key(id) {
                    capsule_data.memories.insert(id.clone(), memory);
                    Ok(())
                } else {
                    Err(Error::NotFound)
                }
            }) {
                Ok(_) => Ok(()),
                Err(e) => Err(Error::Internal(format!("Failed to update memory: {:?}", e))),
            }
        })
    }

    fn get_all_memories(&self, capsule: &CapsuleId) -> Vec<Memory> {
        with_capsule_store(|store| {
            store
                .get(capsule)
                .map(|capsule_data| capsule_data.memories.values().cloned().collect())
                .unwrap_or_default()
        })
    }

    fn get_accessible_capsules(&self, caller: &PersonRef) -> Vec<CapsuleId> {
        with_capsule_store(|store| {
            let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
            all_capsules
                .items
                .into_iter()
                .filter(|capsule| capsule.has_write_access(caller))
                .map(|capsule| capsule.id)
                .collect()
        })
    }

    fn get_capsule_for_acl(&self, capsule_id: &CapsuleId) -> Option<crate::capsule_acl::CapsuleAccess> {
        use crate::capsule_acl::CapsuleAccess;
        with_capsule_store(|store| {
            store.get(capsule_id).map(|capsule| {
                CapsuleAccess::new(
                    capsule.subject.clone(),
                    capsule.owners.clone(),
                    capsule.controllers.clone(),
                )
            })
        })
    }
}

// ============================================================================
// MEMORY UTILITY FUNCTIONS
// ============================================================================

/// Check presence for multiple memories on ICP
/// 
/// Returns presence status for the given memory IDs by checking if they exist
/// in the caller's accessible capsules.
pub fn ping(
    memory_ids: Vec<String>,
) -> std::result::Result<Vec<crate::types::MemoryPresenceResult>, Error> {
    let caller = PersonRef::from_caller();
    
    let results: Vec<crate::types::MemoryPresenceResult> = memory_ids
        .iter()
        .map(|memory_id| {
            // Check if memory exists in any of the caller's accessible capsules
            let exists = with_capsule_store(|store| {
                let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
                all_capsules
                    .items
                    .iter()
                    .any(|capsule| {
                        // Check if caller has read access to this capsule
                        capsule.has_read_access(&caller) && 
                        // Check if memory exists in this capsule
                        capsule.memories.contains_key(memory_id)
                    })
            });
            
            crate::types::MemoryPresenceResult {
                memory_id: memory_id.clone(),
                metadata_present: exists,
                asset_present: exists, // For now, assume if metadata exists, asset exists
            }
        })
        .collect();

    Ok(results)
}

// TODO: list() function is currently unused but may be needed for legacy API compatibility
// Uncomment when needed for frontend integration or legacy support
/*
/// List memories in a capsule
pub fn list(capsule_id: String) -> crate::types::MemoryListResponse {
    use crate::capsule_acl::{CapsuleAccess, CapsuleAcl};
    
    let caller = PersonRef::from_caller();
    let memories = with_capsule_store(|store| {
        store
            .get(&capsule_id)
            .and_then(|capsule| {
                // Use centralized ACL for consistent access control
                let capsule_access = CapsuleAccess::new(
                    capsule.subject.clone(),
                    capsule.owners.clone(),
                    capsule.controllers.clone(),
                );
                
                if capsule_access.can_read(&caller) {
                    // Log successful ACL check
                    ic_cdk::println!(
                        "[ACL] op=list caller={} cap={} read={} write={} delete={} - AUTHORIZED",
                        caller, capsule_id, capsule_access.can_read(&caller), capsule_access.can_write(&caller), capsule_access.can_delete(&caller)
                    );
                    
                    // Debug: Log memory count
                    ic_cdk::println!(
                        "[DEBUG] memories_list: capsule={} has {} memories",
                        capsule_id, capsule.memories.len()
                    );
                    
                    Some(
                        capsule
                            .memories
                            .values()
                            .map(|memory| memory.to_header())
                            .collect::<Vec<_>>(),
                    )
                } else {
                    // Log failed ACL check
                    ic_cdk::println!(
                        "[ACL] op=list caller={} cap={} read={} write={} delete={} - UNAUTHORIZED",
                        caller, capsule_id, capsule_access.can_read(&caller), capsule_access.can_write(&caller), capsule_access.can_delete(&caller)
                    );
                    None
                }
            })
            .unwrap_or_default()
    });
    crate::types::MemoryListResponse {
        success: true,
        memories,
        message: "Memories retrieved successfully".to_string(),
    }
}
*/

// ============================================================================
// MEMORY TYPE IMPLEMENTATIONS
// ============================================================================

impl Memory {
    // TODO: from_blob() method is currently unused but may be needed for blob-based memory creation
    // Uncomment when needed for alternative memory creation patterns
    /*
    /// Create a new memory with blob reference (>32KB)
    pub fn from_blob(
        blob_id: u64,
        size: u64,
        checksum: [u8; 32],
        asset_metadata: crate::types::AssetMetadata,
    ) -> Self {
        let now = ic_cdk::api::time();
        Self {
            id: format!("mem_{now}"), // Simple ID generation
            metadata: crate::types::MemoryMetadata {
                memory_type: crate::types::MemoryType::Note, // Default type for blob
                title: None,
                description: None,
                content_type: match &asset_metadata {
                    crate::types::AssetMetadata::Image(img) => img.base.mime_type.clone(),
                    crate::types::AssetMetadata::Video(vid) => vid.base.mime_type.clone(),
                    crate::types::AssetMetadata::Audio(audio) => audio.base.mime_type.clone(),
                    crate::types::AssetMetadata::Document(doc) => doc.base.mime_type.clone(),
                    crate::types::AssetMetadata::Note(note) => note.base.mime_type.clone(),
                },
                created_at: now,
                updated_at: now,
                uploaded_at: now,
                date_of_memory: None,
                file_created_at: None,
                parent_folder_id: None, // Default to root folder
                tags: match &asset_metadata {
                    crate::types::AssetMetadata::Image(img) => img.base.tags.clone(),
                    crate::types::AssetMetadata::Video(vid) => vid.base.tags.clone(),
                    crate::types::AssetMetadata::Audio(audio) => audio.base.tags.clone(),
                    crate::types::AssetMetadata::Document(doc) => doc.base.tags.clone(),
                    crate::types::AssetMetadata::Note(note) => note.base.tags.clone(),
                },
                deleted_at: None,
                people_in_memory: None,
                location: None,
                memory_notes: None,
                created_by: None,
                database_storage_edges: vec![crate::types::StorageEdgeDatabaseType::Icp],
            },
            access: crate::types::MemoryAccess::Private {
                owner_secure_code: format!("blob_{blob_id}_{:x}", now % 0xFFFF), // Generate secure code
            }, // Default to private access
            inline_assets: vec![],
            blob_internal_assets: vec![crate::types::MemoryAssetBlobInternal {
                asset_id: format!("blob_{}_{}", blob_id, now),
                blob_ref: crate::types::BlobRef {
                    locator: format!("blob_{blob_id}"),
                    hash: Some(checksum),
                    len: size,
                },
                metadata: asset_metadata,
            }],
            blob_external_assets: vec![],
        }
    }
    */

    /// Get memory header for listing
    pub fn to_header(&self) -> crate::types::MemoryHeader {
        // Calculate total size from all assets
        let inline_size: u64 = self
            .inline_assets
            .iter()
            .map(|asset| asset.bytes.len() as u64)
            .sum();
        let blob_internal_size: u64 = self
            .blob_internal_assets
            .iter()
            .map(|asset| asset.blob_ref.len)
            .sum();
        let blob_external_size: u64 = self
            .blob_external_assets
            .iter()
            .map(|asset| match &asset.metadata {
                crate::types::AssetMetadata::Image(img) => img.base.bytes,
                crate::types::AssetMetadata::Video(vid) => vid.base.bytes,
                crate::types::AssetMetadata::Audio(audio) => audio.base.bytes,
                crate::types::AssetMetadata::Document(doc) => doc.base.bytes,
                crate::types::AssetMetadata::Note(note) => note.base.bytes,
            })
            .sum();
        let _size = inline_size + blob_internal_size + blob_external_size;

        crate::types::MemoryHeader {
            // Existing fields
            id: self.id.clone(),
            capsule_id: self.capsule_id.clone(),
            name: self
                .metadata
                .title
                .clone()
                .unwrap_or_else(|| "Untitled".to_string()),
            memory_type: self.metadata.memory_type.clone(),
            size: self.metadata.total_size, // Use pre-computed value
            created_at: self.metadata.created_at,
            updated_at: self.metadata.updated_at,
            // access: self.access.clone(), // Legacy - commented out for greenfield
            
            // NEW: Dashboard-specific fields (pre-computed)
            title: self.metadata.title.clone(),
            description: self.metadata.description.clone(),
            parent_folder_id: self.metadata.parent_folder_id.clone(),
            tags: self.metadata.tags.clone(),
            // ❌ REMOVED: is_public: self.metadata.is_public,     // Redundant with sharing_status
            shared_count: self.metadata.shared_count,
            sharing_status: self.metadata.sharing_status.clone(),
            asset_count: self.metadata.asset_count,
            thumbnail_url: self.metadata.thumbnail_url.clone(),
            primary_asset_url: self.metadata.primary_asset_url.clone(),
            has_thumbnails: self.metadata.has_thumbnails,
            has_previews: self.metadata.has_previews,
        }
    }
    
    /// Compute and update dashboard fields in metadata
    pub fn update_dashboard_fields(&mut self) {
        // ❌ REMOVED: self.metadata.is_public = self.compute_is_public(); // Redundant with sharing_status
        self.metadata.shared_count = self.count_shared_recipients();
        self.metadata.sharing_status = self.compute_sharing_status();
        self.metadata.total_size = self.calculate_total_size();
        self.metadata.asset_count = self.count_assets();
        self.metadata.thumbnail_url = self.generate_thumbnail_url();
        self.metadata.primary_asset_url = self.generate_primary_asset_url();
        self.metadata.has_thumbnails = self.has_thumbnails();
        self.metadata.has_previews = self.has_previews();
    }
    
    /// Check if memory is public based on access rules
    #[allow(dead_code)]
    fn compute_is_public(&self) -> bool {
        // TODO: Replace with new access control system
        // match &self.access {
        //     crate::types::MemoryAccess::Public { .. } => true,
        //     crate::types::MemoryAccess::Private { .. } => false,
        //     crate::types::MemoryAccess::Custom { individuals, groups, .. } => {
        //         individuals.is_empty() && groups.is_empty()
        //     },
        //     crate::types::MemoryAccess::Scheduled { access, .. } => match access.as_ref() {
        //         crate::types::MemoryAccess::Public { .. } => true,
        //         _ => false,
        //     },
        //     crate::types::MemoryAccess::EventTriggered { .. } => false,
        // }
        false // Temporary - assume private for greenfield
    }
    
    /// Count number of shared recipients
    /// TODO: Replace with new access control system
    fn count_shared_recipients(&self) -> u32 {
        // match &self.access {
        //     crate::types::MemoryAccess::Custom { individuals, groups, .. } => {
        //         (individuals.len() + groups.len()) as u32
        //     },
        //     _ => 0,
        // }
        self.access_entries.len() as u32 // Use new access control system
    }
    
    /// Compute sharing status string
    /// TODO: Replace with new access control system
    fn compute_sharing_status(&self) -> crate::capsule::domain::SharingStatus {
        // TODO: Replace with new access control system
        // match &self.access { ... }
        // TODO: Check if any access entry is public
        if self.access_entries.iter().any(|entry| entry.is_public) {
            crate::capsule::domain::SharingStatus::Public
        } else if !self.access_entries.is_empty() {
            crate::capsule::domain::SharingStatus::Shared
        } else {
            crate::capsule::domain::SharingStatus::Private
        }
    }
    
    /// Calculate total size of all assets
    fn calculate_total_size(&self) -> u64 {
        let mut total = 0u64;
        
        // Add inline assets
        for asset in &self.inline_assets {
            total += asset.bytes.len() as u64;
        }
        
        // Add blob internal assets
        for asset in &self.blob_internal_assets {
            total += asset.blob_ref.len;
        }
        
        // Add blob external assets
        for asset in &self.blob_external_assets {
            total += match &asset.metadata {
                crate::types::AssetMetadata::Image(img) => img.base.bytes,
                crate::types::AssetMetadata::Video(vid) => vid.base.bytes,
                crate::types::AssetMetadata::Audio(audio) => audio.base.bytes,
                crate::types::AssetMetadata::Document(doc) => doc.base.bytes,
                crate::types::AssetMetadata::Note(note) => note.base.bytes,
            };
        }
        
        total
    }
    
    /// Count total number of assets
    fn count_assets(&self) -> u32 {
        (self.inline_assets.len() + 
         self.blob_internal_assets.len() + 
         self.blob_external_assets.len()) as u32
    }
    
    /// Generate thumbnail URL if available
    fn generate_thumbnail_url(&self) -> Option<String> {
        // Look for thumbnail in blob internal assets
        for asset in &self.blob_internal_assets {
            let is_thumbnail = match &asset.metadata {
                crate::types::AssetMetadata::Image(img) => matches!(img.base.asset_type, crate::types::AssetType::Thumbnail),
                crate::types::AssetMetadata::Video(vid) => matches!(vid.base.asset_type, crate::types::AssetType::Thumbnail),
                crate::types::AssetMetadata::Audio(audio) => matches!(audio.base.asset_type, crate::types::AssetType::Thumbnail),
                crate::types::AssetMetadata::Document(doc) => matches!(doc.base.asset_type, crate::types::AssetType::Thumbnail),
                crate::types::AssetMetadata::Note(note) => matches!(note.base.asset_type, crate::types::AssetType::Thumbnail),
            };
            if is_thumbnail {
                return Some(format!("icp://memory/{}/blob/{}", self.id, asset.asset_id));
            }
        }
        
        // Look for thumbnail in inline assets
        for asset in &self.inline_assets {
            let is_thumbnail = match &asset.metadata {
                crate::types::AssetMetadata::Image(img) => matches!(img.base.asset_type, crate::types::AssetType::Thumbnail),
                crate::types::AssetMetadata::Video(vid) => matches!(vid.base.asset_type, crate::types::AssetType::Thumbnail),
                crate::types::AssetMetadata::Audio(audio) => matches!(audio.base.asset_type, crate::types::AssetType::Thumbnail),
                crate::types::AssetMetadata::Document(doc) => matches!(doc.base.asset_type, crate::types::AssetType::Thumbnail),
                crate::types::AssetMetadata::Note(note) => matches!(note.base.asset_type, crate::types::AssetType::Thumbnail),
            };
            if is_thumbnail {
                return Some(format!("icp://memory/{}/inline/{}", self.id, asset.asset_id));
            }
        }
        
        None
    }
    
    /// Generate primary asset URL for display
    fn generate_primary_asset_url(&self) -> Option<String> {
        // Look for original asset in blob internal assets
        for asset in &self.blob_internal_assets {
            let is_original = match &asset.metadata {
                crate::types::AssetMetadata::Image(img) => matches!(img.base.asset_type, crate::types::AssetType::Original),
                crate::types::AssetMetadata::Video(vid) => matches!(vid.base.asset_type, crate::types::AssetType::Original),
                crate::types::AssetMetadata::Audio(audio) => matches!(audio.base.asset_type, crate::types::AssetType::Original),
                crate::types::AssetMetadata::Document(doc) => matches!(doc.base.asset_type, crate::types::AssetType::Original),
                crate::types::AssetMetadata::Note(note) => matches!(note.base.asset_type, crate::types::AssetType::Original),
            };
            if is_original {
                return Some(format!("icp://memory/{}/blob/{}", self.id, asset.asset_id));
            }
        }
        
        // Look for original asset in inline assets
        for asset in &self.inline_assets {
            let is_original = match &asset.metadata {
                crate::types::AssetMetadata::Image(img) => matches!(img.base.asset_type, crate::types::AssetType::Original),
                crate::types::AssetMetadata::Video(vid) => matches!(vid.base.asset_type, crate::types::AssetType::Original),
                crate::types::AssetMetadata::Audio(audio) => matches!(audio.base.asset_type, crate::types::AssetType::Original),
                crate::types::AssetMetadata::Document(doc) => matches!(doc.base.asset_type, crate::types::AssetType::Original),
                crate::types::AssetMetadata::Note(note) => matches!(note.base.asset_type, crate::types::AssetType::Original),
            };
            if is_original {
                return Some(format!("icp://memory/{}/inline/{}", self.id, asset.asset_id));
            }
        }
        
        None
    }
    
    /// Check if memory has thumbnails
    fn has_thumbnails(&self) -> bool {
        self.blob_internal_assets.iter().any(|asset| {
            match &asset.metadata {
                crate::types::AssetMetadata::Image(img) => matches!(img.base.asset_type, crate::types::AssetType::Thumbnail),
                crate::types::AssetMetadata::Video(vid) => matches!(vid.base.asset_type, crate::types::AssetType::Thumbnail),
                crate::types::AssetMetadata::Audio(audio) => matches!(audio.base.asset_type, crate::types::AssetType::Thumbnail),
                crate::types::AssetMetadata::Document(doc) => matches!(doc.base.asset_type, crate::types::AssetType::Thumbnail),
                crate::types::AssetMetadata::Note(note) => matches!(note.base.asset_type, crate::types::AssetType::Thumbnail),
            }
        }) || self.inline_assets.iter().any(|asset| {
            match &asset.metadata {
                crate::types::AssetMetadata::Image(img) => matches!(img.base.asset_type, crate::types::AssetType::Thumbnail),
                crate::types::AssetMetadata::Video(vid) => matches!(vid.base.asset_type, crate::types::AssetType::Thumbnail),
                crate::types::AssetMetadata::Audio(audio) => matches!(audio.base.asset_type, crate::types::AssetType::Thumbnail),
                crate::types::AssetMetadata::Document(doc) => matches!(doc.base.asset_type, crate::types::AssetType::Thumbnail),
                crate::types::AssetMetadata::Note(note) => matches!(note.base.asset_type, crate::types::AssetType::Thumbnail),
            }
        })
    }
    
    /// Check if memory has previews
    fn has_previews(&self) -> bool {
        self.blob_internal_assets.iter().any(|asset| {
            match &asset.metadata {
                crate::types::AssetMetadata::Image(img) => matches!(img.base.asset_type, crate::types::AssetType::Preview),
                crate::types::AssetMetadata::Video(vid) => matches!(vid.base.asset_type, crate::types::AssetType::Preview),
                crate::types::AssetMetadata::Audio(audio) => matches!(audio.base.asset_type, crate::types::AssetType::Preview),
                crate::types::AssetMetadata::Document(doc) => matches!(doc.base.asset_type, crate::types::AssetType::Preview),
                crate::types::AssetMetadata::Note(note) => matches!(note.base.asset_type, crate::types::AssetType::Preview),
            }
        }) || self.inline_assets.iter().any(|asset| {
            match &asset.metadata {
                crate::types::AssetMetadata::Image(img) => matches!(img.base.asset_type, crate::types::AssetType::Preview),
                crate::types::AssetMetadata::Video(vid) => matches!(vid.base.asset_type, crate::types::AssetType::Preview),
                crate::types::AssetMetadata::Audio(audio) => matches!(audio.base.asset_type, crate::types::AssetType::Preview),
                crate::types::AssetMetadata::Document(doc) => matches!(doc.base.asset_type, crate::types::AssetType::Preview),
                crate::types::AssetMetadata::Note(note) => matches!(note.base.asset_type, crate::types::AssetType::Preview),
            }
        })
    }
}
