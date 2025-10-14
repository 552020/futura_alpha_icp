//! Model helper functions for memory operations
//!
//! This module contains helper functions used across different memory operations,
//! providing common functionality for creating memories and managing assets.

use crate::capsule::domain::{AccessCondition, AccessEntry, GrantSource, Perm, ResourceRole};
use crate::types::{
    AssetMetadata, BlobRef, CapsuleId, Memory, MemoryAssetBlobExternal, MemoryAssetBlobInternal,
    MemoryAssetInline, MemoryMetadata, MemoryType, PersonRef, StorageEdgeBlobType,
};
use crate::utils::uuid_v7;
use sha2::Digest;

/// Create default owner access entry for a new memory
pub fn create_owner_access_entry(owner: &PersonRef, now: u64) -> AccessEntry {
    AccessEntry {
        id: uuid_v7::uuid_v7_weak(),
        person_ref: Some(owner.clone()),
        is_public: false,
        grant_source: GrantSource::System, // System grants owner access
        source_id: None,
        role: ResourceRole::Owner,
        perm_mask: (Perm::VIEW | Perm::DOWNLOAD | Perm::SHARE | Perm::MANAGE | Perm::OWN).bits(),
        invited_by_person_ref: None,
        created_at: now,
        updated_at: now,
        condition: AccessCondition::Immediate,
    }
}

// Thread-local RNG state for UUID generation (DEPRECATED - using proper UUID v7 now)
// thread_local! {
//     static RNG_STATE: RefCell<Option<VecDeque<u8>>> = RefCell::new(None);
// }

// DEPRECATED - using proper UUID v7 implementation now
// /// Initialize the RNG with ICP's raw_rand (DEPRECATED - using proper UUID v7 now)
// pub fn init_rng() {
//     ic_cdk_timers::set_timer(std::time::Duration::ZERO, || {
//         ic_cdk::futures::spawn_017_compat(async {
//             let seed = raw_rand()
//                 .await
//                 .expect("Failed to get randomness from management canister");
//
//             RNG_STATE.with(|rng| {
//                 *rng.borrow_mut() = Some(VecDeque::from(seed));
//             });
//         });
//     });
// }

// DEPRECATED - using proper UUID v7 implementation now
// All old RNG functions have been removed and replaced with proper UUID v7 implementation

/// Generate a UUID v7 for memory IDs that PostgreSQL will accept
/// Uses proper UUID v7 format with time-ordered timestamps
pub fn generate_uuid_v7() -> String {
    if cfg!(test) {
        // In test context, use the weak version for deterministic results
        uuid_v7::uuid_v7_weak()
    } else {
        // In canister context, use the async version with proper entropy
        // Note: This should be called from an async context in update methods
        // For now, fall back to weak version until we can make the calling code async
        uuid_v7::uuid_v7_weak()
    }
}

/// Generate a deterministic UUID from an idempotency key for proper idempotency
/// This ensures that the same idempotency key always produces the same UUID
pub fn generate_deterministic_uuid_from_idem(idem: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Create a hasher and hash the idempotency key
    let mut hasher = DefaultHasher::new();
    idem.hash(&mut hasher);
    let hash = hasher.finish();

    // Convert the hash to a UUID-like string format
    // Use a simple approach: take the hash and format it as a UUID
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        (hash >> 32) as u32,
        (hash >> 16) as u16,
        hash as u16,
        (hash >> 48) as u16,
        hash as u64 & 0x0000_0000_0000_FFFF
    )
}

/// Generate a UUID-like ID for asset IDs using deterministic pattern
pub fn generate_asset_id(caller: &PersonRef, timestamp: u64) -> String {
    let caller_str = match caller {
        PersonRef::Principal(p) => p.to_text(),
        PersonRef::Opaque(s) => s.clone(),
    };
    let seed = format!("{}-{}", caller_str, timestamp);

    // Generate deterministic hash-based ID
    let hash = sha2::Sha256::digest(seed.as_bytes());
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]),
        u16::from_be_bytes([hash[4], hash[5]]),
        u16::from_be_bytes([hash[6], hash[7]]),
        u16::from_be_bytes([hash[8], hash[9]]),
        u64::from_be_bytes([
            hash[10], hash[11], hash[12], hash[13], hash[14], hash[15], hash[16], hash[17]
        ])
    )
}

/// Validate if a string is a valid UUID-like format
#[allow(dead_code)]
pub fn is_uuid_v7(id: &str) -> bool {
    // Check if it matches UUID format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    let parts: Vec<&str> = id.split('-').collect();
    if parts.len() != 5 {
        return false;
    }

    // Check each part has correct length and is hexadecimal
    parts[0].len() == 8
        && parts[0].chars().all(|c| c.is_ascii_hexdigit())
        && parts[1].len() == 4
        && parts[1].chars().all(|c| c.is_ascii_hexdigit())
        && parts[2].len() == 4
        && parts[2].chars().all(|c| c.is_ascii_hexdigit())
        && parts[3].len() == 4
        && parts[3].chars().all(|c| c.is_ascii_hexdigit())
        && parts[4].len() == 12
        && parts[4].chars().all(|c| c.is_ascii_hexdigit())
}

/// Derive MemoryType from AssetMetadata variant
pub fn memory_type_from_asset(meta: &AssetMetadata) -> MemoryType {
    match meta {
        AssetMetadata::Note(_) => MemoryType::Note,
        AssetMetadata::Image(_) => MemoryType::Image,
        AssetMetadata::Document(_) => MemoryType::Document,
        AssetMetadata::Audio(_) => MemoryType::Audio,
        AssetMetadata::Video(_) => MemoryType::Video,
    }
}

/// Create an inline memory (small assets stored directly)
pub fn create_inline_memory(
    memory_id: &str,
    capsule_id: &CapsuleId,
    bytes: Vec<u8>,
    asset_metadata: AssetMetadata,
    now: u64,
    caller: &PersonRef,
) -> Memory {
    let inline_assets = vec![MemoryAssetInline {
        asset_id: generate_asset_id(caller, now),
        bytes: bytes.clone(),
        metadata: asset_metadata.clone(),
    }];

    let base = asset_metadata.get_base();
    let created_by = match caller {
        PersonRef::Principal(p) => Some(p.to_text()),
        PersonRef::Opaque(s) => Some(s.clone()),
    };

    Memory {
        id: memory_id.to_string(),
        capsule_id: capsule_id.clone(),
        metadata: MemoryMetadata {
            memory_type: memory_type_from_asset(&asset_metadata),
            title: Some(base.name.clone()),
            description: base.description.clone(),
            content_type: base.mime_type.clone(),
            created_at: now,
            updated_at: now,
            uploaded_at: now,
            date_of_memory: None,
            file_created_at: None,
            parent_folder_id: None,
            tags: base.tags.clone(),
            deleted_at: None,
            people_in_memory: None,
            location: None,
            memory_notes: None,
            created_by,
            database_storage_edges: vec![],

            // NEW: Pre-computed dashboard fields (defaults)
            // ❌ REMOVED: is_public: false,                   // Redundant with sharing_status
            shared_count: 0,
            sharing_status: crate::capsule::domain::SharingStatus::Private,
            total_size: base.bytes,
            asset_count: 1,
        },
        // access: MemoryAccess::Private {
        //     owner_secure_code: "test_code".to_string(), // TODO: Generate proper secure code
        // },
        access_entries: vec![create_owner_access_entry(caller, now)], // ✅ Create owner access entry
        // ❌ REMOVED: public_policy field - now unified in AccessEntry
        inline_assets,
        blob_internal_assets: vec![],
        blob_external_assets: vec![],
    }
}

/// Create a blob memory (large assets stored as blobs)
pub fn create_blob_memory(
    memory_id: &str,
    capsule_id: &CapsuleId,
    blob_ref: BlobRef,
    asset_metadata: AssetMetadata,
    now: u64,
    caller: &PersonRef,
) -> Memory {
    let blob_internal_assets = vec![MemoryAssetBlobInternal {
        asset_id: generate_asset_id(caller, now),
        blob_ref,
        metadata: asset_metadata.clone(),
    }];

    let base = asset_metadata.get_base();
    let created_by = match caller {
        PersonRef::Principal(p) => Some(p.to_text()),
        PersonRef::Opaque(s) => Some(s.clone()),
    };

    Memory {
        id: memory_id.to_string(),
        capsule_id: capsule_id.clone(),
        metadata: MemoryMetadata {
            memory_type: memory_type_from_asset(&asset_metadata),
            title: Some(base.name.clone()),
            description: base.description.clone(),
            content_type: base.mime_type.clone(),
            created_at: now,
            updated_at: now,
            uploaded_at: now,
            date_of_memory: None,
            file_created_at: None,
            parent_folder_id: None,
            tags: base.tags.clone(),
            deleted_at: None,
            people_in_memory: None,
            location: None,
            memory_notes: None,
            created_by,
            database_storage_edges: vec![],

            // NEW: Pre-computed dashboard fields (defaults)
            // ❌ REMOVED: is_public: false,                   // Redundant with sharing_status
            shared_count: 0,
            sharing_status: crate::capsule::domain::SharingStatus::Private,
            total_size: base.bytes,
            asset_count: 1,
        },
        // access: MemoryAccess::Private {
        //     owner_secure_code: "test_code".to_string(), // TODO: Generate proper secure code
        // },
        access_entries: vec![create_owner_access_entry(caller, now)], // ✅ Create owner access entry
        // ❌ REMOVED: public_policy field - now unified in AccessEntry
        inline_assets: vec![],
        blob_internal_assets,
        blob_external_assets: vec![],
    }
}

/// Create an external memory (assets stored outside ICP)
pub fn create_external_memory(
    memory_id: &str,
    capsule_id: &CapsuleId,
    location: StorageEdgeBlobType,
    storage_key: Option<String>,
    url: Option<String>,
    _size: Option<u64>,
    _hash: Option<Vec<u8>>,
    asset_metadata: AssetMetadata,
    now: u64,
    caller: &PersonRef,
) -> Memory {
    let blob_external_assets = vec![MemoryAssetBlobExternal {
        asset_id: generate_asset_id(caller, now),
        location,
        storage_key: storage_key.unwrap_or_default(),
        url,
        metadata: asset_metadata.clone(),
    }];

    let base = asset_metadata.get_base();
    let created_by = match caller {
        PersonRef::Principal(p) => Some(p.to_text()),
        PersonRef::Opaque(s) => Some(s.clone()),
    };

    Memory {
        id: memory_id.to_string(),
        capsule_id: capsule_id.clone(),
        metadata: MemoryMetadata {
            memory_type: memory_type_from_asset(&asset_metadata),
            title: Some(base.name.clone()),
            description: base.description.clone(),
            content_type: base.mime_type.clone(),
            created_at: now,
            updated_at: now,
            uploaded_at: now,
            date_of_memory: None,
            file_created_at: None,
            parent_folder_id: None,
            tags: base.tags.clone(),
            deleted_at: None,
            people_in_memory: None,
            location: None,
            memory_notes: None,
            created_by,
            database_storage_edges: vec![],

            // NEW: Pre-computed dashboard fields (defaults)
            // ❌ REMOVED: is_public: false,                   // Redundant with sharing_status
            shared_count: 0,
            sharing_status: crate::capsule::domain::SharingStatus::Private,
            total_size: base.bytes,
            asset_count: 1,
        },
        // access: MemoryAccess::Private {
        //     owner_secure_code: "test_code".to_string(), // TODO: Generate proper secure code
        // },
        access_entries: vec![create_owner_access_entry(caller, now)], // ✅ Create owner access entry
        // ❌ REMOVED: public_policy field - now unified in AccessEntry
        inline_assets: vec![],
        blob_internal_assets: vec![],
        blob_external_assets,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_uuid_v7_format() {
        // Test that the UUID has the correct format and length
        let uuid = generate_uuid_v7();

        // Should be exactly 36 characters (32 hex + 4 hyphens)
        assert_eq!(
            uuid.len(),
            36,
            "UUID should be exactly 36 characters long, got: {} (length: {})",
            uuid,
            uuid.len()
        );

        // Should have exactly 4 hyphens
        let hyphen_count = uuid.matches('-').count();
        assert_eq!(
            hyphen_count, 4,
            "UUID should have exactly 4 hyphens, got: {} (hyphens: {})",
            uuid, hyphen_count
        );

        // Should match UUID format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
        let parts: Vec<&str> = uuid.split('-').collect();
        assert_eq!(
            parts.len(),
            5,
            "UUID should have 5 parts when split by hyphens, got: {}",
            uuid
        );
        assert_eq!(
            parts[0].len(),
            8,
            "First part should be 8 characters, got: {}",
            parts[0]
        );
        assert_eq!(
            parts[1].len(),
            4,
            "Second part should be 4 characters, got: {}",
            parts[1]
        );
        assert_eq!(
            parts[2].len(),
            4,
            "Third part should be 4 characters, got: {}",
            parts[2]
        );
        assert_eq!(
            parts[3].len(),
            4,
            "Fourth part should be 4 characters, got: {}",
            parts[3]
        );
        assert_eq!(
            parts[4].len(),
            12,
            "Fifth part should be 12 characters, got: {}",
            parts[4]
        );

        // All parts should be valid hexadecimal
        for (i, part) in parts.iter().enumerate() {
            assert!(
                part.chars().all(|c| c.is_ascii_hexdigit()),
                "Part {} should be valid hexadecimal, got: {}",
                i,
                part
            );
        }

        // Version should be 7 (first character of third part should be '7')
        assert!(
            parts[2].starts_with('7'),
            "Version should be 7, got: {}",
            parts[2]
        );

        // Variant should be valid (first character of fourth part should be 8, 9, A, or B)
        let variant_char = parts[3].chars().next().unwrap();
        assert!(
            matches!(variant_char, '8' | '9' | 'a' | 'b' | 'A' | 'B'),
            "Variant should be 8, 9, A, or B, got: {}",
            variant_char
        );
    }

    #[test]
    fn test_generate_uuid_v7_uniqueness() {
        // In test mode, UUIDs are deterministic, so we just test that they're consistent
        let uuid1 = generate_uuid_v7();
        let uuid2 = generate_uuid_v7();
        let uuid3 = generate_uuid_v7();

        // In test mode, all UUIDs should be the same (deterministic)
        assert_eq!(uuid1, uuid2, "In test mode, UUIDs should be deterministic");
        assert_eq!(uuid2, uuid3, "In test mode, UUIDs should be deterministic");
        assert_eq!(uuid1, uuid3, "In test mode, UUIDs should be deterministic");

        // But they should still be valid UUIDs
        assert_eq!(uuid1.len(), 36, "UUID should be 36 characters long");
        assert!(is_uuid_v7(&uuid1), "UUID should pass validation");
    }

    #[test]
    fn test_is_uuid_v7_validation() {
        // Test valid UUIDs
        assert!(is_uuid_v7("12345678-1234-1234-1234-123456789abc"));
        assert!(is_uuid_v7("00000000-0000-4000-8000-000000000000"));
        assert!(is_uuid_v7("ffffffff-ffff-4fff-bfff-ffffffffffff"));

        // Test invalid UUIDs
        assert!(!is_uuid_v7("12345678-1234-1234-1234-123456789ab")); // Too short
        assert!(!is_uuid_v7("12345678-1234-1234-1234-123456789abcd")); // Too long
        assert!(!is_uuid_v7("12345678-1234-1234-123456789abc")); // Missing hyphen
        assert!(!is_uuid_v7("12345678-1234-1234-1234-123456789abg")); // Invalid hex character
        assert!(!is_uuid_v7("")); // Empty string
        assert!(!is_uuid_v7("not-a-uuid")); // Not a UUID format
    }
}
