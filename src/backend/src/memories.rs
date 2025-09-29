use crate::capsule_store::{types::PaginationOrder as Order, CapsuleStore};
use crate::memory::{with_capsule_store, with_capsule_store_mut};
use crate::types::{
    AssetMetadata, BlobRef, CapsuleId, Error, Memory, MemoryAssetBlobInternal, MemoryId,
    MemoryType, PersonRef, Result, StorageEdgeDatabaseType,
};
use crate::upload::blob_store::BlobStore;
use crate::upload::types::{CAPSULE_INLINE_BUDGET, INLINE_MAX};
use ic_cdk::api::time;
use sha2::{Digest, Sha256};

/// Utility function to compute SHA256 hash
fn compute_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Create a Memory object from the given parameters
/// This function handles all the memory construction logic
pub fn create_memory_object(
    memory_id: &str,
    blob: BlobRef,
    asset_metadata: AssetMetadata,
    now: u64,
) -> Memory {
    use crate::types::{MemoryAccess, MemoryMetadata};

    let memory_metadata = MemoryMetadata {
        memory_type: match &asset_metadata {
            AssetMetadata::Image(_) => MemoryType::Image,
            AssetMetadata::Video(_) => MemoryType::Video,
            AssetMetadata::Audio(_) => MemoryType::Audio,
            AssetMetadata::Document(_) => MemoryType::Document,
            AssetMetadata::Note(_) => MemoryType::Note,
        },
        title: None,
        description: None,
        content_type: match &asset_metadata {
            AssetMetadata::Image(img) => img.base.mime_type.clone(),
            AssetMetadata::Video(vid) => vid.base.mime_type.clone(),
            AssetMetadata::Audio(audio) => audio.base.mime_type.clone(),
            AssetMetadata::Document(doc) => doc.base.mime_type.clone(),
            AssetMetadata::Note(note) => note.base.mime_type.clone(),
        },
        created_at: now,
        updated_at: now,
        uploaded_at: now,
        date_of_memory: None,
        file_created_at: None,
        parent_folder_id: None, // Default to root folder
        tags: match &asset_metadata {
            AssetMetadata::Image(img) => img.base.tags.clone(),
            AssetMetadata::Video(vid) => vid.base.tags.clone(),
            AssetMetadata::Audio(audio) => audio.base.tags.clone(),
            AssetMetadata::Document(doc) => doc.base.tags.clone(),
            AssetMetadata::Note(note) => note.base.tags.clone(),
        },
        deleted_at: None,
        people_in_memory: None,
        location: None,
        memory_notes: None,
        created_by: None,
        database_storage_edges: vec![StorageEdgeDatabaseType::Icp],
    };

    let memory_access = MemoryAccess::Private {
        owner_secure_code: format!("mem_{}_{:x}", memory_id, now % 0xFFFF), // Generate secure code
    };

    // Create blob assets for ICP capsule storage
    let blob_internal_assets = vec![MemoryAssetBlobInternal {
        blob_ref: blob,
        metadata: asset_metadata,
    }];

    Memory {
        id: memory_id.to_string(),
        metadata: memory_metadata,
        access: memory_access,
        inline_assets: vec![],
        blob_internal_assets,
        blob_external_assets: vec![],
    }
}

pub fn create_inline(
    capsule_id: CapsuleId,
    bytes: Vec<u8>,
    asset_metadata: AssetMetadata,
    idem: String,
) -> Result<MemoryId> {
    let caller = PersonRef::from_caller();

    // 1) Size check
    let len_u64 = bytes.len() as u64;
    if len_u64 > INLINE_MAX {
        return Err(Error::InvalidArgument(format!(
            "inline_too_large: {len_u64} > {INLINE_MAX}"
        )));
    }

    // 2) Persist bytes -> BlobRef (durable)
    // Trust blob store as single source of truth for hash and length
    let blob_store = BlobStore::new();
    let blob = blob_store
        .put_inline(&bytes)
        .map_err(|e| Error::Internal(format!("blob_store_put_inline: {e:?}")))?;

    // Use the hash and length from blob store directly
    let sha256 = blob
        .hash
        .ok_or_else(|| Error::Internal("blob_store_did_not_provide_hash".to_string()))?;

    // 3) Single atomic pass: auth + budget + idempotency + insert
    with_capsule_store_mut(|store| {
        // Use update_with for proper error propagation and atomicity
        store.update_with(&capsule_id, |cap| {
            // First, check for existing memory with same content (idempotency)
            // Move this inside the closure for full atomicity
            if let Some(existing_id) =
                find_existing_memory_by_content_in_capsule(cap, &sha256, len_u64, &idem)
            {
                return Ok(existing_id); // Return existing memory ID
            }
            // Check authorization
            if !cap.owners.contains_key(&caller) && cap.subject != caller {
                return Err(crate::types::Error::Unauthorized);
            }

            // Check budget (use maintained counter)
            let used = cap.inline_bytes_used;
            if used.saturating_add(len_u64) > CAPSULE_INLINE_BUDGET {
                return Err(crate::types::Error::ResourceExhausted);
            }

            // Pre-generate the ID
            let memory_id = generate_memory_id();
            let now = ic_cdk::api::time();

            // Create the memory object
            let memory =
                create_memory_object(&memory_id, blob.clone(), asset_metadata.clone(), now);

            // Insert the memory into the capsule
            cap.memories.insert(memory_id.clone(), memory);

            // Update inline budget counter for inline uploads
            cap.inline_bytes_used = used.saturating_add(len_u64);

            // Note: idempotency tracking not supported in current Capsule struct

            // Update timestamps
            cap.updated_at = now;

            // Update owner activity
            if let Some(owner_state) = cap.owners.get_mut(&caller) {
                owner_state.last_activity_at = now;
            }

            // Return the generated ID
            Ok(memory_id)
        })
    })
}

/// Check presence for multiple memories on ICP (consolidated from get_memory_presence_icp and get_memory_list_presence_icp)
pub fn ping(memory_ids: Vec<String>) -> Result<Vec<crate::types::MemoryPresenceResult>> {
    // TODO: Implement memory presence checking using capsule system instead of artifacts
    // For now, return false for all memories since artifacts system is removed
    let results: Vec<crate::types::MemoryPresenceResult> = memory_ids
        .iter()
        .map(|memory_id| crate::types::MemoryPresenceResult {
            memory_id: memory_id.clone(),
            metadata_present: false, // Artifacts system removed
            asset_present: false,    // Artifacts system removed
        })
        .collect();

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memories_ping_not_found() {
        let memory_id = "nonexistent_memory".to_string();
        let result = ping(vec![memory_id]);

        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(response.len(), 1);
            assert!(!response[0].metadata_present);
            assert!(!response[0].asset_present);
        }
    }

    #[test]
    fn test_memories_ping_empty() {
        let memory_ids = vec![];
        let result = ping(memory_ids);

        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(response.len(), 0);
        }
    }

    #[test]
    fn test_memories_ping_multiple() {
        let memory_ids = vec![
            "mem1".to_string(),
            "mem2".to_string(),
            "mem3".to_string(),
            "mem4".to_string(),
            "mem5".to_string(),
        ];

        let result = ping(memory_ids);
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(response.len(), 5);
            // All memories should not be present (nonexistent)
            for memory_presence in response {
                assert!(!memory_presence.metadata_present);
                assert!(!memory_presence.asset_present);
            }
        }
    }

    #[test]
    fn test_memories_ping_single() {
        let memory_ids = vec!["mem1".to_string()];

        let result = ping(memory_ids);
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(response.len(), 1);
            assert!(!response[0].metadata_present);
            assert!(!response[0].asset_present);
        }
    }

    // ============================================================================
    // MEMORY CREATION TESTS
    // ============================================================================

    // #[test]
    fn _test_create_inline_memory() {
        // This test is commented out because it calls ic_cdk::api::msg_caller() which can only be called inside canisters
        // use crate::types::{
        //     AssetMetadata, AssetMetadataBase, AssetType, DocumentAssetMetadata, MemoryAccess,
        //     MemoryType,
        // };

        // let capsule_id = "test_capsule".to_string();
        // let bytes = vec![1, 2, 3, 4, 5]; // Small file for inline storage
        // let asset_metadata = AssetMetadata::Document(DocumentAssetMetadata {
        //     base: AssetMetadataBase {
        //         name: "test_document.txt".to_string(),
        //         description: Some("Test document".to_string()),
        //         tags: vec!["test".to_string()],
        //         asset_type: AssetType::Original,
        //         bytes: bytes.len() as u64,
        //         mime_type: "text/plain".to_string(),
        //         sha256: Some(compute_sha256(&bytes)),
        //         width: None,
        //         height: None,
        //         url: None,
        //         storage_key: None,
        //         bucket: None,
        //         asset_location: None,
        //         processing_status: None,
        //         processing_error: None,
        //         created_at: 1000000000,
        //         updated_at: 1000000000,
        //         deleted_at: None,
        //     },
        //     page_count: None,
        //     document_type: Some("text".to_string()),
        //     language: Some("en".to_string()),
        //     word_count: Some(5),
        // });
        // let idem = "test_idem".to_string();

        // let result = create_inline(capsule_id, bytes, asset_metadata, idem);

        // assert!(result.is_ok());
        // let memory_id = result.unwrap();
        // assert!(!memory_id.is_empty());
    }

    #[test]
    fn test_create_memory_object_with_inline_asset() {
        use crate::types::{
            AssetMetadata, AssetMetadataBase, AssetType, BlobRef, ImageAssetMetadata,
        };

        let memory_id = "test_memory_inline";
        let blob = BlobRef {
            locator: "inline_test".to_string(),
            hash: None,
            len: 100,
        };
        let asset_metadata = AssetMetadata::Image(ImageAssetMetadata {
            base: AssetMetadataBase {
                name: "test_image.jpg".to_string(),
                description: Some("Test image".to_string()),
                tags: vec!["image".to_string(), "test".to_string()],
                asset_type: AssetType::Thumbnail,
                bytes: 1024,
                mime_type: "image/jpeg".to_string(),
                sha256: Some([1u8; 32]),
                width: Some(800),
                height: Some(600),
                url: None,
                storage_key: None,
                bucket: None,
                asset_location: None,
                processing_status: Some("completed".to_string()),
                processing_error: None,
                created_at: 1000000000,
                updated_at: 1000000000,
                deleted_at: None,
            },
            color_space: Some("sRGB".to_string()),
            exif_data: Some("test_exif".to_string()),
            compression_ratio: Some(0.8),
            dpi: Some(300),
            orientation: Some(1),
        });
        let now = 1000000000;

        let memory = create_memory_object(memory_id, blob, asset_metadata, now);

        // Verify memory structure
        assert_eq!(memory.id, memory_id);
        assert_eq!(memory.metadata.memory_type, MemoryType::Image);
        assert_eq!(memory.metadata.content_type, "image/jpeg");
        assert_eq!(memory.inline_assets.len(), 0); // No inline assets in this test
        assert_eq!(memory.blob_internal_assets.len(), 1); // One blob asset created by create_memory_object
        assert_eq!(memory.blob_external_assets.len(), 0);
    }

    #[test]
    fn test_create_memory_object_with_external_blob_asset() {
        use crate::types::{
            AssetMetadata, AssetMetadataBase, AssetType, BlobRef, StorageEdgeBlobType,
            VideoAssetMetadata,
        };

        let memory_id = "test_memory_external";
        let blob = BlobRef {
            locator: "external_test".to_string(),
            hash: Some([2u8; 32]),
            len: 5000000, // 5MB file
        };
        let asset_metadata = AssetMetadata::Video(VideoAssetMetadata {
            base: AssetMetadataBase {
                name: "test_video.mp4".to_string(),
                description: Some("Test video".to_string()),
                tags: vec!["video".to_string(), "test".to_string()],
                asset_type: AssetType::Original,
                bytes: 5000000,
                mime_type: "video/mp4".to_string(),
                sha256: Some([2u8; 32]),
                width: Some(1920),
                height: Some(1080),
                url: Some("https://example.com/video.mp4".to_string()),
                storage_key: Some("video_key_123".to_string()),
                bucket: Some("video-bucket".to_string()),
                asset_location: Some("S3".to_string()),
                processing_status: Some("processing".to_string()),
                processing_error: None,
                created_at: 1000000000,
                updated_at: 1000000000,
                deleted_at: None,
            },
            duration: Some(120000), // 2 minutes
            frame_rate: Some(30.0),
            codec: Some("H.264".to_string()),
            bitrate: Some(5000000),
            resolution: Some("1920x1080".to_string()),
            aspect_ratio: Some(16.0 / 9.0),
        });
        let now = 1000000000;

        let memory = create_memory_object(memory_id, blob, asset_metadata, now);

        // Verify memory structure
        assert_eq!(memory.id, memory_id);
        assert_eq!(memory.metadata.memory_type, MemoryType::Video);
        assert_eq!(memory.metadata.content_type, "video/mp4");
        assert_eq!(memory.inline_assets.len(), 0);
        assert_eq!(memory.blob_internal_assets.len(), 1); // One blob asset created by create_memory_object
        assert_eq!(memory.blob_external_assets.len(), 0);
    }

    #[test]
    fn test_memory_with_multiple_asset_types() {
        use crate::types::{
            AssetMetadata, AssetMetadataBase, AssetType, AudioAssetMetadata, BlobRef,
            ImageAssetMetadata, MemoryAssetBlobExternal, MemoryAssetBlobInternal,
            MemoryAssetInline, NoteAssetMetadata, StorageEdgeBlobType, VideoAssetMetadata,
        };

        let memory_id = "test_memory_multi";
        let blob = BlobRef {
            locator: "multi_test".to_string(),
            hash: None,
            len: 100,
        };
        let asset_metadata = AssetMetadata::Audio(AudioAssetMetadata {
            base: AssetMetadataBase {
                name: "test_audio.mp3".to_string(),
                description: Some("Test audio".to_string()),
                tags: vec!["audio".to_string(), "test".to_string()],
                asset_type: AssetType::Original,
                bytes: 2000000, // 2MB
                mime_type: "audio/mpeg".to_string(),
                sha256: Some([3u8; 32]),
                width: None,
                height: None,
                url: None,
                storage_key: None,
                bucket: None,
                asset_location: None,
                processing_status: Some("completed".to_string()),
                processing_error: None,
                created_at: 1000000000,
                updated_at: 1000000000,
                deleted_at: None,
            },
            duration: Some(180000), // 3 minutes
            sample_rate: Some(44100),
            channels: Some(2),
            bitrate: Some(320000),
            codec: Some("MP3".to_string()),
            bit_depth: Some(16),
        });
        let now = 1000000000;

        let mut memory = create_memory_object(memory_id, blob, asset_metadata, now);

        // Add multiple asset types to test the multi-asset capability
        memory.inline_assets.push(MemoryAssetInline {
            bytes: vec![1, 2, 3, 4],
            metadata: AssetMetadata::Note(NoteAssetMetadata {
                base: AssetMetadataBase {
                    name: "note.txt".to_string(),
                    description: None,
                    tags: vec!["note".to_string()],
                    asset_type: AssetType::Metadata,
                    bytes: 4,
                    mime_type: "text/plain".to_string(),
                    sha256: None,
                    width: None,
                    height: None,
                    url: None,
                    storage_key: None,
                    bucket: None,
                    asset_location: None,
                    processing_status: None,
                    processing_error: None,
                    created_at: now,
                    updated_at: now,
                    deleted_at: None,
                },
                word_count: Some(1),
                language: Some("en".to_string()),
                format: Some("plain".to_string()),
            }),
        });

        memory.blob_internal_assets.push(MemoryAssetBlobInternal {
            blob_ref: BlobRef {
                locator: "internal_blob_123".to_string(),
                hash: Some([4u8; 32]),
                len: 1000000,
            },
            metadata: AssetMetadata::Image(ImageAssetMetadata {
                base: AssetMetadataBase {
                    name: "thumbnail.jpg".to_string(),
                    description: None,
                    tags: vec!["thumbnail".to_string()],
                    asset_type: AssetType::Thumbnail,
                    bytes: 1000000,
                    mime_type: "image/jpeg".to_string(),
                    sha256: Some([4u8; 32]),
                    width: Some(200),
                    height: Some(200),
                    url: None,
                    storage_key: None,
                    bucket: None,
                    asset_location: None,
                    processing_status: Some("completed".to_string()),
                    processing_error: None,
                    created_at: now,
                    updated_at: now,
                    deleted_at: None,
                },
                color_space: Some("sRGB".to_string()),
                exif_data: None,
                compression_ratio: Some(0.9),
                dpi: Some(72),
                orientation: Some(1),
            }),
        });

        memory.blob_external_assets.push(MemoryAssetBlobExternal {
            location: StorageEdgeBlobType::S3,
            storage_key: "external_video_456".to_string(),
            url: Some("https://s3.amazonaws.com/bucket/video.mp4".to_string()),
            metadata: AssetMetadata::Video(VideoAssetMetadata {
                base: AssetMetadataBase {
                    name: "external_video.mp4".to_string(),
                    description: None,
                    tags: vec!["external".to_string()],
                    asset_type: AssetType::Original,
                    bytes: 10000000, // 10MB
                    mime_type: "video/mp4".to_string(),
                    sha256: Some([5u8; 32]),
                    width: Some(1920),
                    height: Some(1080),
                    url: Some("https://s3.amazonaws.com/bucket/video.mp4".to_string()),
                    storage_key: Some("external_video_456".to_string()),
                    bucket: Some("video-bucket".to_string()),
                    asset_location: Some("S3".to_string()),
                    processing_status: Some("completed".to_string()),
                    processing_error: None,
                    created_at: now,
                    updated_at: now,
                    deleted_at: None,
                },
                duration: Some(300000), // 5 minutes
                frame_rate: Some(24.0),
                codec: Some("H.264".to_string()),
                bitrate: Some(8000000),
                resolution: Some("1920x1080".to_string()),
                aspect_ratio: Some(16.0 / 9.0),
            }),
        });

        // Verify the multi-asset memory structure
        assert_eq!(memory.id, memory_id);
        assert_eq!(memory.metadata.memory_type, MemoryType::Audio);
        assert_eq!(memory.metadata.content_type, "audio/mpeg");

        // Verify all asset types are present
        assert_eq!(memory.inline_assets.len(), 1);
        assert_eq!(memory.blob_internal_assets.len(), 2); // 1 from create_memory_object + 1 manually added
        assert_eq!(memory.blob_external_assets.len(), 1);

        // Verify asset types
        match &memory.inline_assets[0].metadata {
            AssetMetadata::Note(note) => {
                assert_eq!(note.base.name, "note.txt");
                assert_eq!(note.base.asset_type, AssetType::Metadata);
            }
            _ => panic!("Expected Note asset metadata"),
        }

        match &memory.blob_internal_assets[1].metadata {
            AssetMetadata::Image(img) => {
                assert_eq!(img.base.name, "thumbnail.jpg");
                assert_eq!(img.base.asset_type, AssetType::Thumbnail);
            }
            _ => panic!("Expected Image asset metadata"),
        }

        match &memory.blob_external_assets[0].metadata {
            AssetMetadata::Video(vid) => {
                assert_eq!(vid.base.name, "external_video.mp4");
                assert_eq!(vid.base.asset_type, AssetType::Original);
            }
            _ => panic!("Expected Video asset metadata"),
        }
    }
}

/// Generate a new unique memory ID
fn generate_memory_id() -> MemoryId {
    format!("mem_{}", ic_cdk::api::time())
}

// ensure_capsule_access helper removed - authorization logic is inline in create()

/// Helper: Find existing memory by content hash and length within a capsule
/// This version works on a single capsule for use within update_with closures
fn find_existing_memory_by_content_in_capsule(
    cap: &crate::types::Capsule,
    sha256: &[u8; 32],
    len: u64,
    _idem: &str, // Keep parameter for API compatibility but don't use it
) -> Option<MemoryId> {
    cap.memories
        .values()
        .find(|memory| {
            // Content-based deduplication
            // Check inline assets first
            for inline_asset in &memory.inline_assets {
                let memory_sha256 = compute_sha256(&inline_asset.bytes);
                if memory_sha256 == *sha256 && inline_asset.bytes.len() as u64 == len {
                    return true;
                }
            }

            // Check blob internal assets
            for blob_asset in &memory.blob_internal_assets {
                if let Some(ref hash) = blob_asset.blob_ref.hash {
                    if *hash == *sha256 && blob_asset.blob_ref.len == len {
                        return true;
                    }
                }
            }

            // Check blob external assets
            for blob_asset in &memory.blob_external_assets {
                let (hash, bytes) = match &blob_asset.metadata {
                    AssetMetadata::Image(img) => (img.base.sha256.as_ref(), img.base.bytes),
                    AssetMetadata::Video(vid) => (vid.base.sha256.as_ref(), vid.base.bytes),
                    AssetMetadata::Audio(audio) => (audio.base.sha256.as_ref(), audio.base.bytes),
                    AssetMetadata::Document(doc) => (doc.base.sha256.as_ref(), doc.base.bytes),
                    AssetMetadata::Note(note) => (note.base.sha256.as_ref(), note.base.bytes),
                };
                if let Some(hash) = hash {
                    if hash == sha256 && bytes == len {
                        return true;
                    }
                }
            }

            false
        })
        .map(|memory| memory.id.clone())
}

// Legacy find_existing_memory_by_content function removed - using find_existing_memory_by_content_in_capsule instead

// create_memory_from_blob helper removed - not used anywhere

// read function removed - duplicate of memories_read in lib.rs

pub fn update(
    memory_id: String,
    updates: crate::types::MemoryUpdateData,
) -> crate::types::MemoryOperationResponse {
    let caller = PersonRef::from_caller();
    let memory_id_clone = memory_id.clone();
    let mut capsule_found = false;
    let mut memory_found = false;
    with_capsule_store_mut(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        if let Some(capsule) = all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.owners.contains_key(&caller) || capsule.subject == caller)
            .filter(|capsule| capsule.memories.contains_key(&memory_id))
        {
            capsule_found = true;
            let capsule_id = capsule.id.clone();
            let update_result = store.update(&capsule_id, |capsule| {
                if let Some(memory) = capsule.memories.get(&memory_id) {
                    memory_found = true;
                    let mut updated_memory = memory.clone();
                    if let Some(name) = updates.name.clone() {
                        updated_memory.metadata.title = Some(name);
                    }
                    if let Some(metadata) = updates.metadata.clone() {
                        updated_memory.metadata = metadata;
                    }
                    if let Some(access) = updates.access.clone() {
                        updated_memory.access = access;
                    }
                    updated_memory.metadata.updated_at = time();
                    capsule.memories.insert(memory_id.clone(), updated_memory);
                    capsule.updated_at = time();
                }
            });
            if update_result.is_err() {
                capsule_found = false;
            }
        }
    });
    if !capsule_found {
        return crate::types::MemoryOperationResponse {
            success: false,
            memory_id: None,
            message: "No accessible capsule found for caller".to_string(),
        };
    }
    if !memory_found {
        return crate::types::MemoryOperationResponse {
            success: false,
            memory_id: None,
            message: "Memory not found in any accessible capsule".to_string(),
        };
    }
    crate::types::MemoryOperationResponse {
        success: true,
        memory_id: Some(memory_id_clone),
        message: "Memory updated successfully".to_string(),
    }
}

pub fn delete(memory_id: String) -> crate::types::MemoryOperationResponse {
    let caller = PersonRef::from_caller();
    let memory_id_clone = memory_id.clone();
    let mut memory_found = false;
    let mut capsule_found = false;
    with_capsule_store_mut(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        if let Some(capsule) = all_capsules.items.into_iter().find(|capsule| {
            capsule.has_write_access(&caller) && capsule.memories.contains_key(&memory_id)
        }) {
            capsule_found = true;
            let capsule_id = capsule.id.clone();
            let update_result = store.update(&capsule_id, |capsule| {
                if capsule.memories.contains_key(&memory_id) {
                    capsule.memories.remove(&memory_id);
                    capsule.updated_at = time();
                    memory_found = true;
                }
            });
            if update_result.is_err() {
                capsule_found = false;
            }
        }
    });
    if !capsule_found {
        return crate::types::MemoryOperationResponse {
            success: false,
            memory_id: None,
            message: "No accessible capsule found for caller".to_string(),
        };
    }
    if !memory_found {
        return crate::types::MemoryOperationResponse {
            success: false,
            memory_id: None,
            message: "Memory not found in any accessible capsule".to_string(),
        };
    }
    crate::types::MemoryOperationResponse {
        success: true,
        memory_id: Some(memory_id_clone),
        message: "Memory deleted successfully".to_string(),
    }
}

pub fn list(capsule_id: String) -> crate::types::MemoryListResponse {
    let caller = PersonRef::from_caller();
    let memories = with_capsule_store(|store| {
        store
            .get(&capsule_id)
            .and_then(|capsule| {
                if capsule.owners.contains_key(&caller) || capsule.subject == caller {
                    Some(
                        capsule
                            .memories
                            .values()
                            .map(|memory| memory.to_header())
                            .collect::<Vec<_>>(),
                    )
                } else {
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

/// Read a memory by ID from caller's accessible capsules
pub fn read(memory_id: String) -> Result<crate::types::Memory> {
    use crate::capsule_store::types::PaginationOrder as Order;
    use crate::types::PersonRef;

    let caller = PersonRef::from_caller();

    // Find memory across caller's accessible capsules
    crate::memory::with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .find(|capsule| {
                // Check if caller has access to this capsule
                capsule.owners.contains_key(&caller) || capsule.subject == caller
            })
            .and_then(|capsule| capsule.memories.get(&memory_id).cloned())
            .ok_or(crate::types::Error::NotFound)
    })
}
