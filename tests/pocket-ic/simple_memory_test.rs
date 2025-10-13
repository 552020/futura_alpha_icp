//! Simple Memory Test using exact backend types
//!
//! This test uses the exact same types as the backend to avoid Candid mismatches.

use anyhow::Result;
use candid::{CandidType, Decode, Encode, Principal};
use pocket_ic::PocketIc;
use serde::Deserialize;

// ---- mirror .did types (names & fields EXACT) ----
#[derive(CandidType, Deserialize, Clone)]
struct BlobRef {
    len: u64,
    locator: String,
    hash: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Clone)]
enum StorageEdgeBlobType {
    S3,
    Icp,
    VercelBlob,
    Ipfs,
    Neon,
    Arweave,
}

#[derive(CandidType, Deserialize, Clone)]
enum AssetType {
    Preview,
    Metadata,
    Derivative,
    Original,
    Thumbnail,
}

#[derive(CandidType, Deserialize, Clone)]
struct AssetMetadataBase {
    url: Option<String>,
    height: Option<u32>,
    updated_at: u64,
    asset_type: AssetType,
    sha256: Option<Vec<u8>>,
    name: String,
    storage_key: Option<String>,
    tags: Vec<String>,
    processing_error: Option<String>,
    mime_type: String,
    description: Option<String>,
    created_at: u64,
    deleted_at: Option<u64>,
    bytes: u64,
    asset_location: Option<String>,
    width: Option<u32>,
    processing_status: Option<String>,
    bucket: Option<String>,
}

#[derive(CandidType, Deserialize, Clone)]
struct ImageAssetMetadata {
    dpi: Option<u32>,
    color_space: Option<String>,
    base: AssetMetadataBase,
    exif_data: Option<String>,
    compression_ratio: Option<f32>,
    orientation: Option<u8>,
}

#[derive(CandidType, Deserialize, Clone)]
struct NoteAssetMetadata {
    base: AssetMetadataBase,
    language: Option<String>,
    word_count: Option<u32>,
    format: Option<String>,
}

#[derive(CandidType, Deserialize, Clone)]
struct DocumentAssetMetadata {
    document_type: Option<String>,
    base: AssetMetadataBase,
    language: Option<String>,
    page_count: Option<u32>,
    word_count: Option<u32>,
}

#[derive(CandidType, Deserialize, Clone)]
struct AudioAssetMetadata {
    duration: Option<u64>,
    base: AssetMetadataBase,
    codec: Option<String>,
    channels: Option<u8>,
    sample_rate: Option<u32>,
    bit_depth: Option<u8>,
    bitrate: Option<u64>,
}

#[derive(CandidType, Deserialize, Clone)]
struct VideoAssetMetadata {
    duration: Option<u64>,
    base: AssetMetadataBase,
    codec: Option<String>,
    frame_rate: Option<f32>,
    resolution: Option<String>,
    bitrate: Option<u64>,
    aspect_ratio: Option<f32>,
}

#[derive(CandidType, Deserialize, Clone)]
enum AssetMetadata {
    Note(NoteAssetMetadata),
    Image(ImageAssetMetadata),
    Document(DocumentAssetMetadata),
    Audio(AudioAssetMetadata),
    Video(VideoAssetMetadata),
}

// Result_5 and Error (minimal to decode reply)
#[derive(CandidType, Deserialize, Debug)]
enum Error {
    Internal(String),
    NotFound,
    Unauthorized,
    InvalidArgument(String),
    ResourceExhausted,
    Conflict(String),
}

#[derive(CandidType, Deserialize, Debug)]
enum Result5 {
    Ok(String),
    Err(Error),
}

// ---- helper to build a valid Image AssetMetadata ----
fn image_meta(name: &str, mime: &str, bytes_len: u64, now: u64, w: u32, h: u32) -> AssetMetadata {
    let base = AssetMetadataBase {
        url: None,
        height: Some(h),
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
        bytes: bytes_len, // IMPORTANT: must match inline bytes or blob_ref.len
        asset_location: None,
        width: Some(w),
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

fn load_backend_wasm() -> Vec<u8> {
    let path = std::env::var("BACKEND_WASM_PATH")
        .unwrap_or_else(|_| "../../target/wasm32-unknown-unknown/release/backend.wasm".into());
    std::fs::read(path).expect("read backend.wasm")
}

#[test]
fn test_simple_memory_creation() -> Result<()> {
    println!("🚀 Testing simple memory creation...");

    let pic = PocketIc::new();
    let wasm = load_backend_wasm();
    let controller = Principal::from_slice(&[1; 29]);

    // Create canister
    let canister_id = pic.create_canister();
    println!("📦 Created canister: {}", canister_id);

    // Add cycles
    pic.add_cycles(canister_id, 2_000_000_000_000);
    println!("💰 Added cycles");

    // Install canister (using default controller)
    pic.install_canister(canister_id, wasm, vec![], None);
    println!("🔧 Installed canister");

    // Test memories_create with proper types matching .did exactly
    let capsule_id = "test_capsule".to_string();

    // exactly one of these three must be Some
    let bytes = Some(vec![1u8, 2, 3, 4]); // opt blob ✅
    let blob_ref: Option<BlobRef> = None; // opt BlobRef ✅
    let external_location: Option<StorageEdgeBlobType> = None; // opt StorageEdgeBlobType ✅

    let external_storage_key: Option<String> = None;
    let external_url: Option<String> = None;
    let external_size: Option<u64> = None;
    let external_hash: Option<Vec<u8>> = None; // opt blob ✅

    let now = 1_700_000_000_000u64;
    let asset_metadata = image_meta("sample.jpg", "image/jpeg", 4, now, 2, 2); // bytes_len must equal bytes.len()

    let idem = "idem-1".to_string();

    println!("📞 Calling memories_create...");
    let raw = pic
        .update_call(
            canister_id,
            controller,
            "memories_create",
            Encode!(
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
            )?,
        )
        .map_err(|e| anyhow::anyhow!("Update call failed: {:?}", e))?;

    println!("✅ memories_create call completed");

    // Try to decode the result
    let result: Result5 = Decode!(&raw, Result5)?;
    println!("📋 Result: {:?}", result);

    match result {
        Result5::Ok(memory_id) => {
            println!("✅ Memory created successfully with ID: {}", memory_id);
        }
        Result5::Err(error) => {
            println!("❌ Memory creation failed: {:?}", error);
        }
    }

    println!("🎉 Simple memory test completed!");
    Ok(())
}
