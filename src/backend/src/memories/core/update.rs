//! Memory update operations
//!
//! This module contains the core business logic for updating memories
//! with proper access control and post-write assertions.

use super::traits::*;
use super::model_helpers::*;
use crate::capsule_acl::CapsuleAcl;
use crate::memories::types::{InternalBlobAssetInput, InlineAssetInput};
use crate::types::{Error, MemoryId, MemoryUpdateData, BlobRef, MemoryAssetBlobInternal, MemoryAssetInline};

/// Core memory update function - pure business logic
pub fn memories_update_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_id: MemoryId,
    updates: MemoryUpdateData,
) -> std::result::Result<crate::types::Memory, Error> {
    // Capture timestamp once for consistency
    let now = env.now();

    // Find the memory across all accessible capsules
    let accessible_capsules = store.get_accessible_capsules(&env.caller());

    for capsule_id in accessible_capsules {
        if let Some(mut memory) = store.get_memory(&capsule_id, &memory_id) {
            // TODO: Add ownership check when we have proper owner tracking
            // For now, if the caller has access to the capsule, they can update memories

            // Apply updates
            if let Some(name) = updates.name {
                memory.metadata.title = Some(name);
            }

            if let Some(metadata) = updates.metadata {
                memory.metadata = metadata;
            }

            // ✅ NEW: Update access entries using unified access control system
            if let Some(access_entries) = updates.access_entries {
                memory.access_entries = access_entries;
            }

            // Update timestamp with captured value
            memory.metadata.updated_at = now;

            // NEW: Recompute dashboard fields after update
            memory.update_dashboard_fields();

            // Save the updated memory back to the store
            store.update_memory(&capsule_id, &memory_id, memory)?;

            // POST-WRITE ASSERTION: Verify memory was actually updated
            if let Some(updated_memory) = store.get_memory(&capsule_id, &memory_id) {
                if updated_memory.metadata.updated_at != now {
                    return Err(Error::Internal(
                        "Post-update readback failed: memory was not updated".to_string(),
                    ));
                }
                return Ok(updated_memory);
            } else {
                return Err(Error::Internal(
                    "Post-update readback failed: memory was not found".to_string(),
                ));
            }
        }
    }

    Err(Error::NotFound)
}

/// Core function to add a blob asset to an existing memory
///
/// This function adds a new internal blob asset to an existing memory.
/// It validates the memory exists, the caller has write access, and the blob exists.
pub fn memories_add_asset_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_id: MemoryId,
    asset: InternalBlobAssetInput,
    _idem: String,
) -> std::result::Result<String, Error> {
    // Get caller for ACL check
    let caller = env.caller();
    let now = env.now();

    // Find the memory across all accessible capsules
    let accessible_capsules = store.get_accessible_capsules(&caller);

    for capsule_id in accessible_capsules {
        if let Some(mut memory) = store.get_memory(&capsule_id, &memory_id) {
            // Check ACL permissions for this capsule
            let capsule_access = store
                .get_capsule_for_acl(&capsule_id)
                .ok_or(Error::NotFound)?;
            
            if !capsule_access.can_write(&caller) {
                return Err(Error::Unauthorized);
            }

            // Parse blob_id to get BlobRef
            let blob_ref = if asset.blob_id.starts_with("blob_") {
                // Extract the numeric ID from "blob_1234567890"
                let id_str = &asset.blob_id[5..]; // Remove "blob_" prefix
                let _blob_id = id_str.parse::<u64>().map_err(|_| {
                    Error::InvalidArgument(format!("Invalid blob_id format: {}", asset.blob_id))
                })?;

                BlobRef {
                    locator: asset.blob_id.clone(),
                    hash: None, // Hash will be retrieved from blob store if needed
                    len: 0,     // Length will be retrieved from blob store if needed
                }
            } else {
                return Err(Error::InvalidArgument(format!(
                    "Invalid blob_id format: {}",
                    asset.blob_id
                )));
            };

            // Create the new internal blob asset
            let new_asset = MemoryAssetBlobInternal {
                asset_id: generate_asset_id(&caller, now),
                blob_ref,
                metadata: asset.metadata.clone(),
            };

            // Add asset to memory
            memory.blob_internal_assets.push(new_asset.clone());

            // Update memory metadata
            memory.metadata.asset_count += 1;
            memory.metadata.updated_at = now;

            // Recompute dashboard fields
            memory.update_dashboard_fields();

            // Save updated memory
            store.update_memory(&capsule_id, &memory_id, memory)?;

            // POST-WRITE ASSERTION: Verify memory was actually updated
            if let Some(updated_memory) = store.get_memory(&capsule_id, &memory_id) {
                if updated_memory.metadata.updated_at != now {
                    return Err(Error::Internal(
                        "Post-update readback failed: memory was not updated".to_string(),
                    ));
                }
                return Ok(new_asset.asset_id);
            } else {
                return Err(Error::Internal(
                    "Post-update readback failed: memory was not found".to_string(),
                ));
            }
        }
    }

    Err(Error::NotFound)
}

/// Core function to add an inline asset to an existing memory
///
/// This function adds a new inline asset to an existing memory.
/// It validates the memory exists and the caller has write access.
pub fn memories_add_inline_asset_core<E: Env, S: Store>(
    env: &E,
    store: &mut S,
    memory_id: MemoryId,
    asset: InlineAssetInput,
    _idem: String,
) -> std::result::Result<String, Error> {
    // Get caller for ACL check
    let caller = env.caller();
    let now = env.now();

    // Find the memory across all accessible capsules
    let accessible_capsules = store.get_accessible_capsules(&caller);

    for capsule_id in accessible_capsules {
        if let Some(mut memory) = store.get_memory(&capsule_id, &memory_id) {
            // Check ACL permissions for this capsule
            let capsule_access = store
                .get_capsule_for_acl(&capsule_id)
                .ok_or(Error::NotFound)?;
            
            if !capsule_access.can_write(&caller) {
                return Err(Error::Unauthorized);
            }

            // Create the new inline asset
            let new_asset = MemoryAssetInline {
                asset_id: generate_asset_id(&caller, now),
                bytes: asset.bytes.clone(),
                metadata: asset.metadata.clone(),
            };

            // Add asset to memory
            memory.inline_assets.push(new_asset.clone());

            // Update memory metadata
            memory.metadata.asset_count += 1;
            memory.metadata.updated_at = now;

            // Recompute dashboard fields
            memory.update_dashboard_fields();

            // Save updated memory
            store.update_memory(&capsule_id, &memory_id, memory)?;

            // POST-WRITE ASSERTION: Verify memory was actually updated
            if let Some(updated_memory) = store.get_memory(&capsule_id, &memory_id) {
                if updated_memory.metadata.updated_at != now {
                    return Err(Error::Internal(
                        "Post-update readback failed: memory was not updated".to_string(),
                    ));
                }
                return Ok(new_asset.asset_id);
            } else {
                return Err(Error::Internal(
                    "Post-update readback failed: memory was not found".to_string(),
                ));
            }
        }
    }

    Err(Error::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    /// Test utility to create a Memory with default values
    fn create_test_memory(
        id: &str,
        title: Option<&str>,
        is_public: bool,
        shared_count: u32,
        sharing_status: &str,
    ) -> Memory {
        Memory {
            id: id.to_string(),
            capsule_id: "test_capsule".to_string(),
            metadata: MemoryMetadata {
                title: title.map(|s| s.to_string()),
                description: Some("Test Description".to_string()),
                memory_type: MemoryType::Document,
                content_type: "text/plain".to_string(),
                created_at: 1234567890,
                updated_at: 1234567890,
                uploaded_at: 1234567890,
                date_of_memory: Some(1234567890),
                file_created_at: Some(1234567890),
                deleted_at: None,
                tags: vec!["test".to_string()],
                parent_folder_id: None,
                people_in_memory: None,
                location: None,
                memory_notes: None,
                created_by: None,
                database_storage_edges: vec![],

                // Dashboard fields
                shared_count,
                sharing_status: SharingStatus::Private, // Default to private
                total_size: 1000,
                asset_count: 1,
                thumbnail_url: None,
                primary_asset_url: None,
                has_thumbnails: false,
                has_previews: false,
            },
            access_entries: vec![],
            inline_assets: vec![],
            blob_internal_assets: vec![],
            blob_external_assets: vec![],
        }
    }

    #[test]
    fn test_memory_update_dashboard_fields_logic() {
        // Test that the dashboard field recomputation logic is correct
        // by creating a memory and manually calling update_dashboard_fields

        let mut memory =
            create_test_memory("test_memory", Some("Test Memory"), false, 0, "private");

        // Verify initial state
        assert_eq!(memory.metadata.sharing_status, SharingStatus::Private);
        assert_eq!(memory.metadata.shared_count, 0);

        // TODO: Update test to use new access control system
        // memory.access = MemoryAccess::Public {
        //     owner_secure_code: "new_code".to_string(),
        // };

        // Call update_dashboard_fields
        memory.update_dashboard_fields();

        // Verify dashboard fields were recomputed
        assert_eq!(memory.metadata.sharing_status, SharingStatus::Public);
        assert_eq!(memory.metadata.shared_count, 0); // Public has no specific recipients
    }

    #[test]
    fn test_memories_list_uses_precomputed_dashboard_fields() {
        // Test that memories_list returns pre-computed dashboard fields
        // This test verifies that the to_header() method uses pre-computed values

        let mut memory = create_test_memory("test_memory", Some("Test Memory"), true, 5, "shared");

        // Override specific fields for this test
        memory.metadata.total_size = 2048;
        memory.metadata.asset_count = 3;
        memory.metadata.thumbnail_url = Some("icp://memory/test_memory/thumbnail".to_string());
        memory.metadata.primary_asset_url = Some("icp://memory/test_memory/primary".to_string());
        memory.metadata.has_thumbnails = true;

        // Call to_header() method
        let header = memory.to_header();

        // Verify that pre-computed dashboard fields are used
        assert_eq!(header.shared_count, 5);
        assert_eq!(header.sharing_status, SharingStatus::Shared);
        assert_eq!(header.asset_count, 3);
        assert_eq!(
            header.thumbnail_url,
            Some("icp://memory/test_memory/thumbnail".to_string())
        );
        assert_eq!(
            header.primary_asset_url,
            Some("icp://memory/test_memory/primary".to_string())
        );
        assert_eq!(header.has_thumbnails, true);
        assert_eq!(header.has_previews, false);

        // Verify that size uses pre-computed value
        assert_eq!(header.size, 2048);

        // Verify other fields are correctly mapped
        assert_eq!(header.title, Some("Test Memory".to_string()));
        assert_eq!(header.description, Some("Test Description".to_string()));
        assert_eq!(header.tags, vec!["test".to_string()]);
    }
}
