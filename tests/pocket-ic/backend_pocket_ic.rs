//! The backend canister on PocketIC: it starts, it replaces its signing key
//! by a random one, and it serves a file only for a link it signed.
//!
//!   cargo build --target wasm32-unknown-unknown --release -p backend
//!   cargo test --bin backend_pocket_ic -- --nocapture

#[path = "common.rs"]
mod common;

use backend::types::{
    AssetMetadata, AssetMetadataBase, AssetType, BlobRef, Error, ImageAssetMetadata,
    Memory, PersonRef, StorageEdgeBlobType,
};
use backend::personal::CanisterMode;
use candid::{decode_one, encode_args, encode_one, CandidType, Principal};
use common::{install_backend, wasm};
use pocket_ic::PocketIc;
use serde::Deserialize;

const KEY_REPLACED: &str = "signing key: replaced by a random one";

#[derive(CandidType, Deserialize)]
struct HttpRequest {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

#[derive(CandidType, Deserialize)]
struct HttpResponse {
    status_code: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

fn controller() -> Principal {
    Principal::from_slice(&[1; 29])
}

fn stranger() -> Principal {
    Principal::from_slice(&[3; 29])
}

/// GET a path the way the HTTP gateway does
fn get(pic: &PocketIc, backend: Principal, url: &str) -> HttpResponse {
    let request = HttpRequest {
        method: "GET".into(),
        url: url.into(),
        headers: vec![],
        body: vec![],
    };
    let raw = pic
        .query_call(
            backend,
            Principal::anonymous(),
            "http_request",
            encode_one(request).unwrap(),
        )
        .expect("http_request");
    decode_one(&raw).expect("http response")
}

/// How often the canister logged that it replaced its signing key
fn key_replacements(pic: &PocketIc, backend: Principal) -> usize {
    pic.fetch_canister_logs(backend, controller())
        .expect("canister logs")
        .iter()
        .filter(|r| String::from_utf8_lossy(&r.content).contains(KEY_REPLACED))
        .count()
}

fn ticks(pic: &PocketIc) {
    for _ in 0..5 {
        pic.tick();
    }
}

#[test]
fn the_backend_starts_and_answers() {
    let pic = PocketIc::new();
    let backend = install_backend(&pic, controller());

    let raw = pic
        .query_call(backend, stranger(), "whoami", encode_args(()).unwrap())
        .unwrap();
    assert_eq!(decode_one::<Principal>(&raw).unwrap(), stranger());

    let raw = pic
        .query_call(backend, stranger(), "greet", encode_one("World").unwrap())
        .unwrap();
    assert_eq!(decode_one::<String>(&raw).unwrap(), "Hello, World!");

    let raw = pic
        .query_call(backend, stranger(), "canister_mode", encode_args(()).unwrap())
        .unwrap();
    assert_eq!(decode_one::<CanisterMode>(&raw).unwrap(), CanisterMode::Shared);

    assert_eq!(get(&pic, backend, "/health").status_code, 200);
}

/// The key made at install comes from canister id and time, which anybody
/// can look up. It must be gone right after install, and stay random.
#[test]
fn the_signing_key_is_replaced_by_a_random_one_once() {
    let pic = PocketIc::new();
    let backend = install_backend(&pic, controller());
    ticks(&pic);
    assert_eq!(key_replacements(&pic, backend), 1, "replaced right after install");

    pic.upgrade_canister(
        backend,
        wasm("backend"),
        encode_args(()).unwrap(),
        Some(controller()),
    )
    .expect("upgrade");
    ticks(&pic);
    assert_eq!(
        key_replacements(&pic, backend),
        1,
        "a random key is kept across upgrades, so links stay valid"
    );
}

/// The reply of `capsules_create`, as far as a test needs it. The old
/// backend in `fixtures/` answers with an older Capsule; its id is the same.
#[derive(CandidType, Deserialize)]
struct CapsuleId {
    id: String,
}

/// A capsule for `owner` with one memory holding `file` inline.
/// Returns the memory id.
fn store_file(pic: &PocketIc, backend: Principal, owner: Principal, file: &[u8]) -> String {
    let raw = pic
        .update_call(
            backend,
            owner,
            "capsules_create",
            encode_one(None::<PersonRef>).unwrap(),
        )
        .unwrap();
    let capsule = decode_one::<Result<CapsuleId, Error>>(&raw).unwrap().unwrap();

    let now = pic.get_time().as_nanos_since_unix_epoch();
    let metadata = AssetMetadata::Image(ImageAssetMetadata {
        base: AssetMetadataBase {
            name: "tiny.png".into(),
            description: None,
            tags: vec![],
            asset_type: AssetType::Original,
            bytes: file.len() as u64,
            mime_type: "image/png".into(),
            sha256: None,
            width: Some(1),
            height: Some(1),
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
        color_space: None,
        exif_data: None,
        compression_ratio: None,
        dpi: None,
        orientation: None,
    });
    let raw = pic
        .update_call(
            backend,
            owner,
            "memories_create",
            encode_args((
                capsule.id,
                Some(file.to_vec()),
                None::<BlobRef>,
                None::<StorageEdgeBlobType>,
                None::<String>,
                None::<String>,
                None::<u64>,
                None::<Vec<u8>>,
                metadata,
                "idem-1".to_string(),
            ))
            .unwrap(),
        )
        .unwrap();
    decode_one::<Result<String, Error>>(&raw).unwrap().unwrap()
}

fn read_memory(pic: &PocketIc, backend: Principal, who: Principal, memory_id: &str) -> Memory {
    let raw = pic
        .query_call(backend, who, "memories_read", encode_one(memory_id).unwrap())
        .unwrap();
    decode_one::<Result<Memory, Error>>(&raw).unwrap().unwrap()
}

/// A signed link to the inline file of a memory, as its owner gets it
fn signed_link(pic: &PocketIc, backend: Principal, who: Principal, memory: &Memory) -> String {
    let asset_id = memory.inline_assets[0].asset_id.clone();
    let raw = pic
        .query_call(
            backend,
            who,
            "mint_http_token",
            encode_args((
                memory.id.clone(),
                vec!["inline".to_string()],
                Some(vec![asset_id.clone()]),
                180u32,
            ))
            .unwrap(),
        )
        .expect("mint a link");
    let token: String = decode_one(&raw).unwrap();
    format!("/asset/{}/inline?id={asset_id}&token={token}", memory.id)
}

#[test]
fn a_file_is_served_only_for_a_link_the_canister_signed() {
    let pic = PocketIc::new();
    let backend = install_backend(&pic, controller());
    ticks(&pic); // the random signing key is in place
    let owner = Principal::from_slice(&[2; 29]);

    let file = vec![7u8, 1, 7, 1];
    let memory_id = store_file(&pic, backend, owner, &file);
    let memory = read_memory(&pic, backend, owner, &memory_id);
    let asset_id = memory.inline_assets[0].asset_id.clone();

    // The owner gets a link; a stranger does not
    let mint = |who: Principal| {
        pic.query_call(
            backend,
            who,
            "mint_http_token",
            encode_args((
                memory_id.clone(),
                vec!["inline".to_string()],
                Some(vec![asset_id.clone()]),
                180u32,
            ))
            .unwrap(),
        )
    };
    let token: String = decode_one(&mint(owner).expect("the owner mints a link")).unwrap();
    assert!(mint(stranger()).is_err(), "a stranger gets no link");

    let path = format!("/asset/{memory_id}/inline?id={asset_id}");
    let served = get(&pic, backend, &format!("{path}&token={token}"));
    assert_eq!(served.status_code, 200);
    assert_eq!(served.body, file, "the link serves the file");

    assert_eq!(get(&pic, backend, &path).status_code, 401, "no token");

    // One changed character in the signature
    let mut forged = token.clone().into_bytes();
    let last = forged.len() - 1;
    forged[last] = if forged[last] == b'A' { b'B' } else { b'A' };
    let forged = String::from_utf8(forged).unwrap();
    let refused = get(&pic, backend, &format!("{path}&token={forged}"));
    assert!(
        matches!(refused.status_code, 400 | 403),
        "a changed token is refused, got {}",
        refused.status_code
    );

    // The link is for this file only
    let other = get(&pic, backend, &format!("/asset/{memory_id}/inline?id=other&token={token}"));
    assert_ne!(other.status_code, 200, "the link does not open another file");

    // An upgrade keeps the key: the link still works
    pic.upgrade_canister(
        backend,
        wasm("backend"),
        encode_args(()).unwrap(),
        Some(controller()),
    )
    .expect("upgrade");
    ticks(&pic);
    let served = get(&pic, backend, &format!("{path}&token={token}"));
    assert_eq!(served.status_code, 200, "the link survives an upgrade");
}


/// Upgrading from a backend as it ran before November 2026 (commit bee859d,
/// `fixtures/`): that version wrote a snapshot over the start of stable
/// memory in `pre_upgrade`, and its signing key is the guessable one.
/// The data must come through, and the key must be replaced.
#[test]
fn an_upgrade_from_the_old_backend_keeps_the_data() {
    let pic = PocketIc::new();
    let backend = pic.create_canister_with_settings(Some(controller()), None);
    pic.add_cycles(backend, 5 * common::T);
    let old = std::fs::read("fixtures/backend-bee859d.wasm.gz").expect("old backend fixture");
    pic.install_canister(backend, old, encode_args(()).unwrap(), Some(controller()));
    let owner = Principal::from_slice(&[2; 29]);

    let file = vec![9u8, 8, 7, 6, 5];
    let memory_id = store_file(&pic, backend, owner, &file);
    assert_eq!(key_replacements(&pic, backend), 0, "the old backend keeps its first key");

    pic.upgrade_canister(
        backend,
        wasm("backend"),
        encode_args(()).unwrap(),
        Some(controller()),
    )
    .expect("upgrade from the old backend");
    ticks(&pic);

    let memory = read_memory(&pic, backend, owner, &memory_id);
    assert_eq!(memory.inline_assets[0].bytes, file, "the file came through");
    assert_eq!(key_replacements(&pic, backend), 1, "the guessable key is gone");
    let link = signed_link(&pic, backend, owner, &memory);
    assert_eq!(get(&pic, backend, &link).body, file, "and it is served");

    // Once more, now from the new version to itself
    pic.upgrade_canister(
        backend,
        wasm("backend"),
        encode_args(()).unwrap(),
        Some(controller()),
    )
    .expect("second upgrade");
    ticks(&pic);
    let memory = read_memory(&pic, backend, owner, &memory_id);
    assert_eq!(memory.inline_assets[0].bytes, file);
    assert_eq!(key_replacements(&pic, backend), 1);
    assert_eq!(get(&pic, backend, &link).status_code, 200, "the link survives");
}

fn main() {}
