//! PocketIC Integration Tests for Memory Management
//!
//! This module contains integration tests that use PocketIC to test our memory management
//! functions in a real ICP environment. These tests complement our unit tests by providing
//! end-to-end validation of the canister functions.
//!
//! Performance Notes:
//! - These tests are slower than unit tests due to PocketIC overhead
//! - Each test creates a fresh canister to avoid interference
//! - Consider running with `cargo test --release` for better performance
//! - Use `--test-threads=1` to avoid PocketIC server conflicts

#[path = "common.rs"]
mod common;

use anyhow::Result;
use candid::{CandidType, Decode, Encode, Principal};
use pocket_ic::PocketIc;
use serde::Deserialize;
// The reply and argument types are the backend's own: a copy kept here went
// out of date with every change of the interface.
use backend::types::*;

type Result5 = std::result::Result<String, Error>;
type Result6 = std::result::Result<String, Error>;
type Result20 = std::result::Result<Memory, Error>;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================


/// Create a new canister with optimized setup
fn create_test_canister() -> (PocketIc, Principal, Vec<u8>) {
    let pic = PocketIc::new();
    let wasm = common::wasm("backend");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);
    pic.install_canister(canister_id, wasm.clone(), candid::encode_args(()).unwrap(), None);
    (pic, canister_id, wasm)
}

/// Helper function to create a capsule for a user
fn create_capsule_for_user(
    pic: &mut PocketIc,
    canister_id: Principal,
    user: Principal,
) -> Result<String> {
    let capsule_creation_result = pic
        .update_call(
            canister_id,
            user,
            "capsules_create",
            Encode!(&Option::<PersonRef>::None)?, // No subject specified, will use caller
        )
        .map_err(|e| anyhow::anyhow!("Capsule creation failed: {:?}", e))?;

    // The capsules_create function returns std::result::Result<Capsule>, so we need to handle the Result wrapper
    #[derive(CandidType, Deserialize)]
    struct CapsuleIdOnly {
        id: String,
        // We'll ignore all other fields - the decoder will skip unknown fields
    }

    // The capsules_create function returns Result_5 (Ok: Capsule, Err: Error)
    #[derive(CandidType, Deserialize)]
    enum Result5 {
        Ok(CapsuleIdOnly),
        Err(Error),
    }

    let result: Result5 = Decode!(&capsule_creation_result, Result5)?;
    match result {
        Result5::Ok(capsule) => Ok(capsule.id),
        Result5::Err(e) => Err(anyhow::anyhow!("Capsule creation failed: {:?}", e)),
    }
}

fn image_meta_now(
    name: &str,
    mime: &str,
    bytes: u64,
    width: u32,
    height: u32,
    now: u64,
) -> AssetMetadata {
    let base = AssetMetadataBase {
        url: None,
        height: Some(height),
        updated_at: now,
        asset_type: AssetType::Original,
        sha256: None,
        name: name.to_string(),
        storage_key: None,
        tags: vec![],
        processing_error: None,
        mime_type: mime.to_string(),
        description: None,
        created_at: now,
        deleted_at: None,
        bytes,
        asset_location: None,
        width: Some(width),
        processing_status: None,
        bucket: None,
    };
    AssetMetadata::Image(ImageAssetMetadata {
        dpi: None,
        color_space: Some("RGB".into()),
        base,
        exif_data: None,
        compression_ratio: None,
        orientation: None,
    })
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_create_and_read_memory_happy_path() -> Result<()> {
    let (mut pic, canister_id, _wasm) = create_test_canister();
    let controller = Principal::from_slice(&[1; 29]);

    // First, create a capsule for the user
    let capsule_id = create_capsule_for_user(&mut pic, canister_id, controller)?;

    // Inputs per .did:
    // memories_create(
    //   text, opt blob, opt BlobRef, opt StorageEdgeBlobType,
    //   opt text, opt text, opt nat64, opt blob, AssetMetadata, text
    // ) -> Result_5 (Ok text)
    let inline_bytes: Option<Vec<u8>> = Some(vec![1, 2, 3, 4]);
    let blob_ref: Option<BlobRef> = None;
    let storage_loc: Option<StorageEdgeBlobType> = None;
    let external_storage_key: Option<String> = None;
    let external_url: Option<String> = None;
    let external_size: Option<u64> = None;
    let external_hash: Option<Vec<u8>> = None;
    let now = 1_695_000_000_000u64;
    let meta = image_meta_now("sample.jpg", "image/jpeg", 4, 2, 2, now);
    let idem = "idem-1".to_string();

    let raw = pic
        .update_call(
            canister_id,
            controller,
            "memories_create",
            Encode!(
                &capsule_id,
                &inline_bytes,
                &blob_ref,
                &storage_loc,
                &external_storage_key,
                &external_url,
                &external_size,
                &external_hash,
                &meta,
                &idem
            )?,
        )
        .map_err(|e| anyhow::anyhow!("Update call failed: {:?}", e))?;

    let memory_id = match Decode!(&raw, Result6)? {
        Result6::Ok(id) => id,
        Result6::Err(e) => panic!("memories_create Err: {:?}", e),
    };

    assert!(!memory_id.is_empty());

    // Read back
    let raw = pic
        .query_call(
            canister_id,
            controller,
            "memories_read",
            Encode!(&memory_id)?,
        )
        .map_err(|e| anyhow::anyhow!("Query call failed: {:?}", e))?;

    let mem = match Decode!(&raw, Result20)? {
        Result20::Ok(m) => m,
        Result20::Err(e) => panic!("memories_read Err: {:?}", e),
    };

    assert_eq!(mem.id, memory_id);
    Ok(())
}

#[test]
fn test_delete_forbidden_for_non_owner() -> Result<()> {
    let mut pic = PocketIc::new();
    let wasm = common::wasm("backend");
    let owner = Principal::from_slice(&[1; 29]);
    let stranger = Principal::from_slice(&[2; 29]);
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);
    // Use None for sender = PocketIC's default controller (already authorized)
    pic.install_canister(canister_id, wasm, candid::encode_args(()).unwrap(), None);

    // First, create a capsule for the owner
    let capsule_id = create_capsule_for_user(&mut pic, canister_id, owner)?;

    // Create as owner - using external storage branch
    // Use the capsule_id we just created
    let bytes: Option<Vec<u8>> = None;
    let blob_ref: Option<BlobRef> = None;
    let external_location: Option<StorageEdgeBlobType> = Some(StorageEdgeBlobType::Icp);
    let external_storage_key: Option<String> = Some("x.png".into());
    let external_url: Option<String> = None;
    let external_size: Option<u64> = Some(1024);
    let external_hash: Option<Vec<u8>> = None;
    let asset_metadata = image_meta_now("x.png", "image/png", 1024, 1, 1, 42);
    let idem = "idem-X".to_string();

    let payload = candid::Encode!(
        &capsule_id,
        &bytes,
        &blob_ref,
        &external_location,
        &external_storage_key,
        &external_url,
        &external_size,
        &external_hash,
        &asset_metadata,
        &idem
    )?;
    let raw = pic
        .update_call(canister_id, owner, "memories_create", payload)
        .map_err(|e| anyhow::anyhow!("Update call failed: {:?}", e))?;
    let mem_id = match Decode!(&raw, Result6)? {
        Result6::Ok(id) => id,
        Result6::Err(e) => panic!("create Err: {:?}", e),
    };

    // Try delete as stranger: memories_delete(text, bool) -> Result
    let raw = pic
        .update_call(
            canister_id,
            stranger,
            "memories_delete",
            Encode!(&mem_id, &false)?,
        )
        .map_err(|e| anyhow::anyhow!("Update call failed: {:?}", e))?;

    // The memories_delete function returns Result (Ok with no data, or Err with Error)
    #[derive(CandidType, Deserialize)]
    enum SimpleResult {
        Ok,
        Err(Error),
    }

    let resp: SimpleResult = Decode!(&raw, SimpleResult)?;

    match resp {
        SimpleResult::Ok => panic!("delete should be forbidden for non-owner"),
        SimpleResult::Err(e) => {
            // Check that it's an authorization or not found error
            // NotFound is also acceptable since the stranger can't see the memory
            match e {
                Error::Unauthorized => {
                    println!("✅ Delete correctly forbidden for non-owner (Unauthorized)");
                }
                Error::NotFound => {
                    println!("✅ Delete correctly forbidden for non-owner (NotFound - memory not visible)");
                }
                _ => panic!("Expected Unauthorized or NotFound error, got: {:?}", e),
            }
        }
    }

    Ok(())
}

#[test]
fn test_memory_creation_idempotency() -> Result<()> {
    let mut pic = PocketIc::new();
    let wasm = common::wasm("backend");
    let controller = Principal::from_slice(&[1; 29]);
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);
    pic.install_canister(canister_id, wasm, candid::encode_args(()).unwrap(), None);

    // First, create a capsule for the user
    let capsule_id = create_capsule_for_user(&mut pic, canister_id, controller)?;
    let idem = "same-idem".to_string();
    let now = 1_695_000_000_000u64;

    // First creation - using inline bytes branch (all external fields must be None)
    let bytes1 = Some(vec![1u8, 2, 3]);
    let blob_ref1: Option<BlobRef> = None;
    let external_location1: Option<StorageEdgeBlobType> = None;
    let external_storage_key1: Option<String> = None; // inline branch: must be None
    let external_url1: Option<String> = None;
    let external_size1: Option<u64> = None;
    let external_hash1: Option<Vec<u8>> = None;
    let asset_metadata1 = image_meta_now("test1.jpg", "image/jpeg", 3, 1, 1, now);

    let payload1 = candid::Encode!(
        &capsule_id,
        &bytes1,
        &blob_ref1,
        &external_location1,
        &external_storage_key1,
        &external_url1,
        &external_size1,
        &external_hash1,
        &asset_metadata1,
        &idem
    )?;
    let raw1 = pic
        .update_call(canister_id, controller, "memories_create", payload1)
        .map_err(|e| anyhow::anyhow!("Update call failed: {:?}", e))?;
    let id1 = match Decode!(&raw1, Result6)? {
        Result6::Ok(id) => id,
        Result6::Err(e) => panic!("first create Err: {:?}", e),
    };

    // Second creation with same idem - using inline bytes branch (all external fields must be None)
    let bytes2 = Some(vec![4u8, 5, 6]); // Different data
    let blob_ref2: Option<BlobRef> = None;
    let external_location2: Option<StorageEdgeBlobType> = None;
    let external_storage_key2: Option<String> = None; // inline branch: must be None
    let external_url2: Option<String> = None;
    let external_size2: Option<u64> = None;
    let external_hash2: Option<Vec<u8>> = None;
    let asset_metadata2 = image_meta_now("test2.jpg", "image/jpeg", 3, 1, 1, now);

    let payload2 = candid::Encode!(
        &capsule_id,
        &bytes2,
        &blob_ref2,
        &external_location2,
        &external_storage_key2,
        &external_url2,
        &external_size2,
        &external_hash2,
        &asset_metadata2,
        &idem
    )?;
    let raw2 = pic
        .update_call(canister_id, controller, "memories_create", payload2)
        .map_err(|e| anyhow::anyhow!("Update call failed: {:?}", e))?;
    let id2 = match Decode!(&raw2, Result6)? {
        Result6::Ok(id) => id,
        Result6::Err(e) => panic!("second create Err: {:?}", e),
    };

    // Should get the same ID (idempotency)
    assert_eq!(id1, id2);

    // Read back and verify it has the original data (first create wins)
    let raw = pic
        .query_call(canister_id, controller, "memories_read", Encode!(&id1)?)
        .map_err(|e| anyhow::anyhow!("Query call failed: {:?}", e))?;
    let mem = match Decode!(&raw, Result20)? {
        Result20::Ok(m) => m,
        Result20::Err(e) => panic!("read Err: {:?}", e),
    };

    assert_eq!(mem.metadata.title, Some("test1.jpg".to_string())); // Original name
    assert_eq!(mem.inline_assets[0].bytes, vec![1, 2, 3]); // Original data

    Ok(())
}

#[test]
fn test_memory_update_roundtrip() -> Result<()> {
    let mut pic = PocketIc::new();
    let wasm = common::wasm("backend");
    let controller = Principal::from_slice(&[1; 29]);
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);
    pic.install_canister(canister_id, wasm, candid::encode_args(()).unwrap(), None);

    // First, create a capsule for the user
    let capsule_id = create_capsule_for_user(&mut pic, canister_id, controller)?;

    // Create memory - using inline bytes branch (all external fields must be None)
    // Use the capsule_id we just created
    let bytes = Some(vec![1u8, 2, 3, 4]);
    let blob_ref: Option<BlobRef> = None;
    let external_location: Option<StorageEdgeBlobType> = None;
    let external_storage_key: Option<String> = None; // inline branch: must be None
    let external_url: Option<String> = None;
    let external_size: Option<u64> = None;
    let external_hash: Option<Vec<u8>> = None;
    let asset_metadata = image_meta_now("original.jpg", "image/jpeg", 4, 2, 2, 1_695_000_000_000);
    let idem = "update-test".to_string();

    let payload = candid::Encode!(
        &capsule_id,
        &bytes,
        &blob_ref,
        &external_location,
        &external_storage_key,
        &external_url,
        &external_size,
        &external_hash,
        &asset_metadata,
        &idem
    )?;
    let raw = pic
        .update_call(canister_id, controller, "memories_create", payload)
        .map_err(|e| anyhow::anyhow!("Update call failed: {:?}", e))?;
    let memory_id = match Decode!(&raw, Result6)? {
        Result6::Ok(id) => id,
        Result6::Err(e) => panic!("create Err: {:?}", e),
    };

    // Update memory - create proper MemoryUpdateData structure
    let update_data = MemoryUpdateData {
        metadata: None,
        name: Some("Updated Name".to_string()),
        access_entries: None,
    };

    let raw = pic
        .update_call(
            canister_id,
            controller,
            "memories_update",
            Encode!(&memory_id, &update_data)?,
        )
        .map_err(|e| anyhow::anyhow!("Update call failed: {:?}", e))?;

    // The memories_update function returns Result_20 (Ok: Memory, Err: Error)
    let _update_resp = match Decode!(&raw, Result20)? {
        Result20::Ok(memory) => {
            println!("✅ Update successful, memory ID: {}", memory.id);
            memory
        }
        Result20::Err(e) => panic!("Update should succeed, got error: {:?}", e),
    };

    // Read back and verify
    let raw = pic
        .query_call(
            canister_id,
            controller,
            "memories_read",
            Encode!(&memory_id)?,
        )
        .map_err(|e| anyhow::anyhow!("Update call failed: {:?}", e))?;
    let mem = match Decode!(&raw, Result20)? {
        Result20::Ok(m) => m,
        Result20::Err(e) => panic!("read Err: {:?}", e),
    };

    assert_eq!(mem.metadata.title, Some("Updated Name".to_string()));
    assert!(mem.metadata.updated_at > mem.metadata.created_at);

    Ok(())
}

#[test]
fn test_memory_crud_full_workflow() -> Result<()> {
    // The memories_delete function returns Result (Ok with no data, or Err with Error)
    #[derive(CandidType, Deserialize)]
    enum SimpleResult {
        Ok,
        Err(Error),
    }

    let mut pic = PocketIc::new();
    let wasm = common::wasm("backend");
    let controller = Principal::from_slice(&[1; 29]);
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);
    pic.install_canister(canister_id, wasm, candid::encode_args(()).unwrap(), None);

    // First, create a capsule for the user
    let capsule_id = create_capsule_for_user(&mut pic, canister_id, controller)?;

    // 1. Create - using inline bytes branch (all external fields must be None)
    // Use the capsule_id we just created
    let bytes = Some(vec![1u8, 2, 3, 4, 5]);
    let blob_ref: Option<BlobRef> = None;
    let external_location: Option<StorageEdgeBlobType> = None;
    let external_storage_key: Option<String> = None; // inline branch: must be None
    let external_url: Option<String> = None;
    let external_size: Option<u64> = None;
    let external_hash: Option<Vec<u8>> = None;
    let asset_metadata = image_meta_now("crud-test.jpg", "image/jpeg", 5, 1, 1, 1_695_000_000_000);
    let idem = "crud-workflow".to_string();

    let payload = candid::Encode!(
        &capsule_id,
        &bytes,
        &blob_ref,
        &external_location,
        &external_storage_key,
        &external_url,
        &external_size,
        &external_hash,
        &asset_metadata,
        &idem
    )?;
    let raw = pic
        .update_call(canister_id, controller, "memories_create", payload)
        .map_err(|e| anyhow::anyhow!("Update call failed: {:?}", e))?;
    let memory_id = match Decode!(&raw, Result6)? {
        Result6::Ok(id) => id,
        Result6::Err(e) => panic!("create Err: {:?}", e),
    };

    // 2. Read
    let raw = pic
        .query_call(
            canister_id,
            controller,
            "memories_read",
            Encode!(&memory_id)?,
        )
        .map_err(|e| anyhow::anyhow!("Update call failed: {:?}", e))?;
    let mem = match Decode!(&raw, Result20)? {
        Result20::Ok(m) => m,
        Result20::Err(e) => panic!("read Err: {:?}", e),
    };

    assert_eq!(mem.id, memory_id);
    assert_eq!(mem.inline_assets[0].bytes, vec![1, 2, 3, 4, 5]);

    // 3. Update
    let update_data = MemoryUpdateData {
        metadata: None,
        name: Some("CRUD Updated".to_string()),
        access_entries: None,
    };

    let raw = pic
        .update_call(
            canister_id,
            controller,
            "memories_update",
            Encode!(&memory_id, &update_data)?,
        )
        .map_err(|e| anyhow::anyhow!("Update call failed: {:?}", e))?;

    // The memories_update function returns Result_20 (Ok: Memory, Err: Error)
    let _update_resp = match Decode!(&raw, Result20)? {
        Result20::Ok(memory) => {
            println!("✅ CRUD Update successful, memory ID: {}", memory.id);
            memory
        }
        Result20::Err(e) => panic!("Update should succeed, got error: {:?}", e),
    };

    // 4. Read again to verify update
    let raw = pic
        .query_call(
            canister_id,
            controller,
            "memories_read",
            Encode!(&memory_id)?,
        )
        .map_err(|e| anyhow::anyhow!("Update call failed: {:?}", e))?;
    let mem = match Decode!(&raw, Result20)? {
        Result20::Ok(m) => m,
        Result20::Err(e) => panic!("read after update Err: {:?}", e),
    };

    assert_eq!(mem.metadata.title, Some("CRUD Updated".to_string()));

    // 5. Delete
    let raw = pic
        .update_call(
            canister_id,
            controller,
            "memories_delete",
            Encode!(&memory_id, &false)?, // memories_delete(text, bool) -> Result
        )
        .map_err(|e| anyhow::anyhow!("Update call failed: {:?}", e))?;

    // The memories_delete function returns Result (Ok with no data, or Err with Error)
    match Decode!(&raw, SimpleResult)? {
        SimpleResult::Ok => {
            println!("✅ CRUD Delete successful");
        }
        SimpleResult::Err(e) => panic!("Delete should succeed, got error: {:?}", e),
    }

    // 6. Try to read deleted memory (should fail)
    let raw = pic
        .query_call(
            canister_id,
            controller,
            "memories_read",
            Encode!(&memory_id)?,
        )
        .map_err(|e| anyhow::anyhow!("Update call failed: {:?}", e))?;
    match Decode!(&raw, Result20)? {
        Result20::Ok(_) => panic!("Should not be able to read deleted memory"),
        Result20::Err(Error::NotFound) => {
            println!("✅ Correctly got NotFound when reading deleted memory");
        } // Expected
        Result20::Err(e) => panic!("Unexpected error: {:?}", e),
    }

    Ok(())
}
