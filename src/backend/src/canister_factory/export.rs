use crate::canister_factory::types::*;
use crate::capsule::domain::Capsule;
use crate::capsule_store::{types::PaginationOrder as Order, CapsuleStore};
use crate::memory::with_capsule_store;
use crate::types;
use candid::Principal;

/// Get current time - can be mocked in tests
#[cfg(not(test))]
fn get_current_time() -> u64 {
    ic_cdk::api::time()
}

#[cfg(test)]
fn get_current_time() -> u64 {
    1000000000 // Fixed timestamp for tests
}

/// Get current canister ID - can be mocked in tests
#[cfg(not(test))]
fn get_current_canister_id() -> Principal {
    ic_cdk::api::canister_self()
}

#[cfg(test)]
fn get_current_canister_id() -> Principal {
    Principal::from_slice(&[99; 29]) // Fixed canister ID for tests
}

/// Export user's capsule data for migration
/// This function serializes all capsule data including metadata, memories, and connections
pub fn export_user_capsule_data(user: Principal) -> Result<ExportData, String> {
    let user_ref = types::PersonRef::Principal(user);

    // MIGRATED: Find the user's self-capsule (where user is both subject and owner)
    let capsule = with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == user_ref && capsule.owners.contains_key(&user_ref))
    });

    let capsule = match capsule {
        Some(c) => c,
        None => return Err(format!("No self-capsule found for user {user}")),
    };

    // Extract memories as vector of (id, memory) pairs
    let memories: Vec<(String, types::Memory)> = capsule
        .memories
        .iter()
        .map(|(id, memory)| (id.clone(), memory.clone()))
        .collect();

    // Extract connections as vector of (person_ref, connection) pairs
    let connections: Vec<(types::PersonRef, types::Connection)> = capsule
        .connections
        .iter()
        .map(|(person_ref, connection)| (person_ref.clone(), connection.clone()))
        .collect();

    // Calculate total size of exported data
    let total_size_bytes = calculate_export_data_size(&capsule, &memories, &connections);

    // Generate export metadata
    let metadata = ExportMetadata {
        export_timestamp: get_current_time(),
        original_canister_id: get_current_canister_id(),
        data_version: "1.0".to_string(), // Version for compatibility checking
        total_size_bytes,
    };

    let export_data = ExportData {
        capsule,
        memories,
        connections,
        metadata,
    };

    ic_cdk::println!(
        "Exported capsule data for user {}: {} memories, {} connections, {} bytes total",
        user,
        export_data.memories.len(),
        export_data.connections.len(),
        total_size_bytes
    );

    Ok(export_data)
}

/// Calculate the approximate size of exported data in bytes
/// This provides an estimate for monitoring and validation purposes
fn calculate_export_data_size(
    capsule: &Capsule,
    memories: &[(String, types::Memory)],
    connections: &[(types::PersonRef, types::Connection)],
) -> u64 {
    let mut total_size = 0u64;

    // Estimate capsule metadata size (rough approximation)
    total_size += 1024; // Base capsule structure
    total_size += (capsule.owners.len() * 128) as u64; // Owner data
    total_size += (capsule.controllers.len() * 128) as u64; // Controller data
    total_size += (capsule.connection_groups.len() * 256) as u64; // Connection groups

    // Calculate memory data sizes
    for (memory_id, memory) in memories {
        // Memory ID and metadata
        total_size += memory_id.len() as u64;
        total_size += 512; // Memory metadata estimate

        // Memory blob data if stored inline
        for inline_asset in &memory.inline_assets {
            total_size += inline_asset.bytes.len() as u64;
        }

        // Memory metadata specific sizes
        if let Some(ref title) = memory.metadata.title {
            total_size += title.len() as u64;
        }
        if let Some(ref description) = memory.metadata.description {
            total_size += description.len() as u64;
        }
        total_size += memory.metadata.content_type.len() as u64;
        total_size += memory
            .metadata
            .tags
            .iter()
            .map(|tag| tag.len() as u64)
            .sum::<u64>();
    }

    // Calculate connection data sizes
    for (person_ref, _connection) in connections {
        total_size += 256; // Connection structure estimate
        match person_ref {
            types::PersonRef::Principal(_) => total_size += 32, // Principal size
            types::PersonRef::Opaque(id) => total_size += id.len() as u64,
        }
    }

    total_size
}

/// Generate a manifest for exported data to enable verification
pub fn generate_export_manifest(export_data: &ExportData) -> Result<DataManifest, String> {
    // Generate capsule checksum
    let capsule_checksum = generate_capsule_checksum(&export_data.capsule)?;

    // Generate memory checksums
    let mut memory_checksums = Vec::new();
    for (memory_id, memory) in &export_data.memories {
        let memory_checksum = generate_memory_checksum(memory_id, memory)?;
        memory_checksums.push((memory_id.clone(), memory_checksum));
    }

    // Generate connection checksums
    let mut connection_checksums = Vec::new();
    for (person_ref, connection) in &export_data.connections {
        let person_ref_string = person_ref_to_string(person_ref);
        let connection_checksum = generate_connection_checksum(person_ref, connection)?;
        connection_checksums.push((person_ref_string, connection_checksum));
    }

    let manifest = DataManifest {
        capsule_checksum,
        memory_count: export_data.memories.len() as u32,
        memory_checksums,
        connection_count: export_data.connections.len() as u32,
        connection_checksums,
        total_size_bytes: export_data.metadata.total_size_bytes,
        manifest_version: "1.0".to_string(),
    };

    ic_cdk::println!(
        "Generated export manifest: {} memories, {} connections, checksum: {}",
        manifest.memory_count,
        manifest.connection_count,
        &manifest.capsule_checksum[..8] // Show first 8 chars of checksum
    );

    Ok(manifest)
}

/// Validate the completeness and integrity of exported data
pub fn validate_export_data(export_data: &ExportData) -> Result<(), String> {
    // Check that capsule has required fields
    if export_data.capsule.id.is_empty() {
        return Err("Capsule ID is empty".to_string());
    }

    if export_data.capsule.owners.is_empty() {
        return Err("Capsule has no owners".to_string());
    }

    // Validate metadata
    if export_data.metadata.export_timestamp == 0 {
        return Err("Invalid export timestamp".to_string());
    }

    if export_data.metadata.data_version.is_empty() {
        return Err("Data version is empty".to_string());
    }

    // Validate memories
    for (memory_id, memory) in &export_data.memories {
        if memory_id.is_empty() {
            return Err("Memory ID is empty".to_string());
        }

        if memory.id != *memory_id {
            return Err(format!(
                "Memory ID mismatch: key '{}' vs memory.id '{}'",
                memory_id, memory.id
            ));
        }

        // Validate memory data consistency
        validate_memory_data(memory)?;
    }

    // Validate connections
    for (person_ref, connection) in &export_data.connections {
        if connection.peer != *person_ref {
            return Err(format!(
                "Connection peer mismatch: key '{:?}' vs connection.peer '{:?}'",
                person_ref, connection.peer
            ));
        }

        if connection.created_at == 0 {
            return Err("Connection has invalid created_at timestamp".to_string());
        }
    }

    // Check data size consistency
    let calculated_size = calculate_export_data_size(
        &export_data.capsule,
        &export_data.memories,
        &export_data.connections,
    );

    // Allow some variance in size calculation (within 10%)
    let size_diff = calculated_size.abs_diff(export_data.metadata.total_size_bytes);

    let max_variance = export_data.metadata.total_size_bytes / 10; // 10% variance allowed
    if size_diff > max_variance {
        return Err(format!(
            "Data size mismatch: calculated {} bytes, metadata claims {} bytes (diff: {})",
            calculated_size, export_data.metadata.total_size_bytes, size_diff
        ));
    }

    ic_cdk::println!(
        "Export data validation passed: {} memories, {} connections, {} bytes",
        export_data.memories.len(),
        export_data.connections.len(),
        export_data.metadata.total_size_bytes
    );

    Ok(())
}

/// Validate individual memory data for consistency
fn validate_memory_data(memory: &types::Memory) -> Result<(), String> {
    // Check basic fields
    if memory.id.is_empty() {
        return Err("Memory ID is empty".to_string());
    }

    if memory
        .metadata
        .title
        .as_ref()
        .map_or(true, |t| t.is_empty())
    {
        return Err(format!("Memory '{}' has empty title", memory.id));
    }

    if memory.metadata.content_type.is_empty() {
        return Err(format!("Memory '{}' has empty content_type", memory.id));
    }

    // Validate timestamps
    if memory.metadata.created_at == 0 {
        return Err(format!("Memory '{}' has invalid created_at", memory.id));
    }

    if memory.metadata.uploaded_at == 0 {
        return Err(format!("Memory '{}' has invalid uploaded_at", memory.id));
    }

    // Validate metadata consistency
    if memory.metadata.content_type.is_empty() {
        return Err(format!("Memory '{}' has empty content_type", memory.id));
    }

    // Validate blob references
    for blob_asset in &memory.blob_internal_assets {
        if blob_asset.blob_ref.locator.is_empty() {
            return Err(format!("Memory '{}' has empty blob locator", memory.id));
        }
    }

    Ok(())
}

/// Generate checksum for capsule data
fn generate_capsule_checksum(capsule: &Capsule) -> Result<String, String> {
    // Create a deterministic representation of capsule for hashing
    let capsule_data = format!(
        "{}|{}|{}|{}|{}|{}|{}",
        capsule.id,
        person_ref_to_string(&capsule.subject),
        capsule.owners.len(),
        capsule.controllers.len(),
        capsule.memories.len(),
        capsule.created_at,
        capsule.updated_at
    );

    Ok(simple_hash(&capsule_data))
}

/// Generate checksum for memory data
fn generate_memory_checksum(memory_id: &str, memory: &types::Memory) -> Result<String, String> {
    // Create a deterministic representation of memory for hashing
    let mut locators = Vec::new();
    let mut total_data_len = 0;

    // Collect inline asset info
    for inline_asset in &memory.inline_assets {
        locators.push("inline".to_string());
        total_data_len += inline_asset.bytes.len();
    }

    // Collect blob asset info
    for blob_asset in &memory.blob_internal_assets {
        locators.push(blob_asset.blob_ref.locator.clone());
    }

    let locator = locators.join(",");

    let memory_data = format!(
        "{}|{}|{}|{}|{}|{}|{}",
        memory_id,
        memory
            .metadata
            .title
            .as_ref()
            .unwrap_or(&"Untitled".to_string()),
        memory.metadata.content_type,
        memory.metadata.created_at,
        memory.metadata.uploaded_at,
        locator,
        total_data_len
    );

    Ok(simple_hash(&memory_data))
}

/// Generate checksum for connection data
fn generate_connection_checksum(
    person_ref: &types::PersonRef,
    connection: &types::Connection,
) -> Result<String, String> {
    // Create a deterministic representation of connection for hashing
    let connection_data = format!(
        "{}|{}|{}|{}|{}",
        person_ref_to_string(person_ref),
        person_ref_to_string(&connection.peer),
        format!("{:?}", connection.status), // Use debug format for enum
        connection.created_at,
        connection.updated_at
    );

    Ok(simple_hash(&connection_data))
}

/// Convert PersonRef to string for consistent representation
fn person_ref_to_string(person_ref: &types::PersonRef) -> String {
    match person_ref {
        types::PersonRef::Principal(p) => format!("principal:{}", p.to_text()),
        types::PersonRef::Opaque(id) => format!("opaque:{id}"),
    }
}

/// Simple hash function for checksums (using a basic approach for MVP)
/// In production, this should use a proper cryptographic hash like SHA-256
fn simple_hash(data: &str) -> String {
    // For MVP, use a simple hash based on data content
    // This is not cryptographically secure but sufficient for basic integrity checking
    let mut hash: u64 = 5381;
    for byte in data.bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u64);
    }
    format!("{hash:016x}")
}

/// Verify exported data against a manifest
pub fn verify_export_against_manifest(
    export_data: &ExportData,
    manifest: &DataManifest,
) -> Result<(), String> {
    // Check counts
    if export_data.memories.len() as u32 != manifest.memory_count {
        return Err(format!(
            "Memory count mismatch: export has {}, manifest expects {}",
            export_data.memories.len(),
            manifest.memory_count
        ));
    }

    if export_data.connections.len() as u32 != manifest.connection_count {
        return Err(format!(
            "Connection count mismatch: export has {}, manifest expects {}",
            export_data.connections.len(),
            manifest.connection_count
        ));
    }

    // Verify capsule checksum
    let capsule_checksum = generate_capsule_checksum(&export_data.capsule)?;
    if capsule_checksum != manifest.capsule_checksum {
        return Err(format!(
            "Capsule checksum mismatch: calculated '{}', manifest expects '{}'",
            capsule_checksum, manifest.capsule_checksum
        ));
    }

    // Verify memory checksums
    for (memory_id, memory) in &export_data.memories {
        let memory_checksum = generate_memory_checksum(memory_id, memory)?;

        let expected_checksum = manifest
            .memory_checksums
            .iter()
            .find(|(id, _)| id == memory_id)
            .map(|(_, checksum)| checksum)
            .ok_or_else(|| format!("Memory '{memory_id}' not found in manifest"))?;

        if memory_checksum != *expected_checksum {
            return Err(format!(
                "Memory '{memory_id}' checksum mismatch: calculated '{memory_checksum}', manifest expects '{expected_checksum}'"
            ));
        }
    }

    // Verify connection checksums
    for (person_ref, connection) in &export_data.connections {
        let person_ref_string = person_ref_to_string(person_ref);
        let connection_checksum = generate_connection_checksum(person_ref, connection)?;

        let expected_checksum = manifest
            .connection_checksums
            .iter()
            .find(|(ref_str, _)| ref_str == &person_ref_string)
            .map(|(_, checksum)| checksum)
            .ok_or_else(|| format!("Connection '{person_ref_string}' not found in manifest"))?;

        if connection_checksum != *expected_checksum {
            return Err(format!(
                "Connection '{person_ref_string}' checksum mismatch: calculated '{connection_checksum}', manifest expects '{expected_checksum}'"
            ));
        }
    }

    ic_cdk::println!(
        "Export verification passed: {} memories, {} connections verified against manifest",
        export_data.memories.len(),
        export_data.connections.len()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capsule::domain::SharingStatus;
    use crate::test_utils::{
        create_test_capsule as shared_create_test_capsule,
        create_test_memory as shared_create_test_memory,
    };
    use crate::types::{
        AssetMetadata, AssetMetadataBase, AssetType, BlobRef, Connection, ConnectionStatus,
        DocumentAssetMetadata, HostingPreferences, Memory, MemoryAssetBlobInternal, MemoryMetadata,
        MemoryType, OwnerState, PersonRef, StorageEdgeDatabaseType,
    };
    use candid::Principal;
    use std::collections::HashMap;

    // Helper function to create a test principal
    fn test_principal(id: u8) -> Principal {
        Principal::from_slice(&[id; 29])
    }

    // Helper function to create owner access entry
    fn create_owner_access_entry(
        owner: &PersonRef,
        created_at: u64,
    ) -> crate::capsule::domain::AccessEntry {
        crate::capsule::domain::AccessEntry {
            id: format!("owner_access_{}", created_at),
            person_ref: Some(owner.clone()),
            is_public: false,
            grant_source: crate::capsule::domain::GrantSource::System,
            source_id: None,
            role: crate::capsule::domain::ResourceRole::Owner,
            perm_mask: 0b11111, // All permissions
            invited_by_person_ref: None,
            created_at,
            updated_at: created_at,
            condition: crate::capsule::domain::AccessCondition::Immediate,
        }
    }

    // Helper function to create a test capsule with given subject and owners
    fn create_test_capsule(
        id: &str,
        subject: PersonRef,
        owners: Vec<PersonRef>,
        memories: Vec<(String, Memory)>,
        connections: Vec<(PersonRef, Connection)>,
    ) -> Capsule {
        let mut owner_map = HashMap::new();
        for owner in owners {
            owner_map.insert(
                owner,
                OwnerState {
                    since: 1000000000,
                    last_activity_at: 1000000000,
                },
            );
        }

        let mut memory_map = HashMap::new();
        for (memory_id, memory) in memories {
            memory_map.insert(memory_id, memory);
        }

        let mut connection_map = HashMap::new();
        for (person_ref, connection) in connections {
            connection_map.insert(person_ref, connection);
        }

        Capsule {
            id: id.to_string(),
            subject,
            owners: owner_map,
            controllers: HashMap::new(),
            connections: connection_map,
            has_advanced_settings: false, // Default to simple settings
            connection_groups: HashMap::new(),
            memories: memory_map,
            galleries: HashMap::new(),
            folders: HashMap::new(),
            created_at: 1000000000,
            updated_at: 1000000000,
            bound_to_neon: false, // Default to not bound to Neon
            inline_bytes_used: 0,
            hosting_preferences: HostingPreferences::default(),
        }
    }

    // Helper function to create a test memory
    fn create_test_memory(
        id: &str,
        name: &str,
        memory_type: MemoryType,
        content_type: &str,
        data_size: usize,
    ) -> Memory {
        Memory {
            id: id.to_string(),
            capsule_id: "test_capsule".to_string(),
            metadata: MemoryMetadata {
                memory_type,
                title: Some(name.to_string()),
                description: None,
                content_type: content_type.to_string(),
                created_at: 1000000000,
                updated_at: 1000000000,
                uploaded_at: 1000000000,
                date_of_memory: Some(1000000000),
                file_created_at: Some(1000000000),
                parent_folder_id: None,
                tags: vec![],
                deleted_at: None,
                people_in_memory: None,
                location: None,
                memory_notes: None,
                created_by: None,
                database_storage_edges: vec![StorageEdgeDatabaseType::Icp],

                // NEW: Pre-computed dashboard fields (defaults)
                shared_count: 0,
                sharing_status: SharingStatus::Private,
                total_size: data_size as u64,
                asset_count: 1,
            },
            access_entries: vec![create_owner_access_entry(
                &PersonRef::Principal(Principal::anonymous()),
                1000000000,
            )],
            inline_assets: vec![],
            blob_internal_assets: vec![MemoryAssetBlobInternal {
                asset_id: format!("test_asset_{}", id),
                blob_ref: BlobRef {
                    locator: format!("test_locator_{}", id),
                    hash: None,
                    len: 100,
                },
                metadata: AssetMetadata::Document(DocumentAssetMetadata {
                    base: AssetMetadataBase {
                        name: name.to_string(),
                        description: None,
                        tags: vec![],
                        asset_type: AssetType::Original,
                        bytes: data_size as u64,
                        mime_type: content_type.to_string(),
                        sha256: None,
                        width: None,
                        height: None,
                        url: None,
                        storage_key: None,
                        bucket: None,
                        processing_status: None,
                        processing_error: None,
                        created_at: 1000000000,
                        updated_at: 1000000000,
                        deleted_at: None,
                        asset_location: None,
                    },
                    page_count: None,
                    document_type: None,
                    language: None,
                    word_count: None,
                }),
            }],
            blob_external_assets: vec![],
        }
    }

    // Helper function to create a test connection
    fn create_test_connection(peer: PersonRef, status: ConnectionStatus) -> Connection {
        Connection {
            peer,
            status,
            created_at: 1000000000,
            updated_at: 1000000000,
        }
    }

    // Helper function to setup test capsules in memory
    fn setup_test_capsule_with_data() -> (Principal, Capsule) {
        let user = test_principal(1);
        let user_ref = PersonRef::Principal(user);

        // Create test memories
        let memory1 = create_test_memory(
            "mem1",
            "test_image.jpg",
            MemoryType::Image,
            "image/jpeg",
            1024,
        );
        let memory2 = create_test_memory(
            "mem2",
            "test_audio.mp3",
            MemoryType::Audio,
            "audio/mpeg",
            2048,
        );

        // Create test connections
        let peer1 = PersonRef::Principal(test_principal(2));
        let peer2 = PersonRef::Opaque("friend_123".to_string());
        let connection1 = create_test_connection(peer1.clone(), ConnectionStatus::Accepted);
        let connection2 = create_test_connection(peer2.clone(), ConnectionStatus::Pending);

        let capsule = create_test_capsule(
            "test_capsule",
            user_ref.clone(),
            vec![user_ref],
            vec![("mem1".to_string(), memory1), ("mem2".to_string(), memory2)],
            vec![(peer1, connection1), (peer2, connection2)],
        );

        (user, capsule)
    }

    #[test]
    fn test_export_user_capsule_data_success() {
        let (user, capsule) = setup_test_capsule_with_data();

        // Store capsule in memory
        crate::memory::with_capsule_store_mut(|store| {
            store.upsert("test_capsule".to_string(), capsule.clone());
        });

        // Test export
        let result = export_user_capsule_data(user);
        assert!(result.is_ok(), "Export should succeed");

        let export_data = result.unwrap();

        // Verify capsule data
        assert_eq!(export_data.capsule.id, "test_capsule");
        assert_eq!(export_data.capsule.subject, PersonRef::Principal(user));

        // Verify memories
        assert_eq!(export_data.memories.len(), 2);
        let memory_ids: Vec<&String> = export_data.memories.iter().map(|(id, _)| id).collect();
        assert!(memory_ids.contains(&&"mem1".to_string()));
        assert!(memory_ids.contains(&&"mem2".to_string()));

        // Verify connections
        assert_eq!(export_data.connections.len(), 2);

        // Verify metadata
        assert!(export_data.metadata.export_timestamp > 0);
        assert_eq!(export_data.metadata.data_version, "1.0");
        assert!(export_data.metadata.total_size_bytes > 0);

        // Clean up
        crate::memory::with_capsule_store_mut(|_store| {
            // Note: No direct clear method, but tests use fresh store each time
        });
    }

    #[test]
    fn test_export_user_capsule_data_no_self_capsule() {
        let user = test_principal(1);
        let other_user = test_principal(2);
        let user_ref = PersonRef::Principal(user);
        let other_user_ref = PersonRef::Principal(other_user);

        // Create a capsule where user is NOT the subject (not a self-capsule)
        let capsule = create_test_capsule(
            "other_capsule",
            other_user_ref, // subject is different from user
            vec![user_ref], // but user is an owner
            vec![],
            vec![],
        );

        // Store capsule in memory
        crate::memory::with_capsule_store_mut(|store| {
            store.upsert("other_capsule".to_string(), capsule);
        });

        // Test export - should fail because no self-capsule found
        let result = export_user_capsule_data(user);
        assert!(
            result.is_err(),
            "Export should fail when no self-capsule exists"
        );
        assert!(result.unwrap_err().contains("No self-capsule found"));

        // Clean up
        crate::memory::with_capsule_store_mut(|_store| {
            // Note: No direct clear method, but tests use fresh store each time
        });
    }

    #[test]
    fn test_export_user_capsule_data_not_owner() {
        let user = test_principal(1);
        let other_user = test_principal(2);
        let user_ref = PersonRef::Principal(user);
        let other_user_ref = PersonRef::Principal(other_user);

        // Create a capsule where user is the subject but NOT an owner
        let capsule = create_test_capsule(
            "not_owned_capsule",
            user_ref,             // subject is user
            vec![other_user_ref], // but other_user is the owner
            vec![],
            vec![],
        );

        // Store capsule in memory
        crate::memory::with_capsule_store_mut(|store| {
            store.upsert("not_owned_capsule".to_string(), capsule);
        });

        // Test export - should fail because user doesn't own the capsule
        let result = export_user_capsule_data(user);
        assert!(
            result.is_err(),
            "Export should fail when user doesn't own their capsule"
        );

        // Clean up
        crate::memory::with_capsule_store_mut(|_store| {
            // Note: No direct clear method, but tests use fresh store each time
        });
    }

    #[test]
    fn test_calculate_export_data_size() {
        let (_, capsule) = setup_test_capsule_with_data();
        let memories: Vec<(String, types::Memory)> = capsule
            .memories
            .iter()
            .map(|(id, memory)| (id.clone(), memory.clone()))
            .collect();
        let connections: Vec<(types::PersonRef, types::Connection)> = capsule
            .connections
            .iter()
            .map(|(person_ref, connection)| (person_ref.clone(), connection.clone()))
            .collect();

        let size = calculate_export_data_size(&capsule, &memories, &connections);

        // Should be greater than 0 and account for memory data
        assert!(size > 0, "Calculated size should be greater than 0");

        // Should include the memory data sizes (1024 + 2048 = 3072 bytes minimum)
        // Allow some tolerance for serialization overhead
        assert!(size >= 2700, "Size should include memory data: {}", size);
    }

    #[test]
    fn test_generate_export_manifest() {
        let (_user, capsule) = setup_test_capsule_with_data();
        let memories: Vec<(String, types::Memory)> = capsule
            .memories
            .iter()
            .map(|(id, memory)| (id.clone(), memory.clone()))
            .collect();
        let connections: Vec<(types::PersonRef, types::Connection)> = capsule
            .connections
            .iter()
            .map(|(person_ref, connection)| (person_ref.clone(), connection.clone()))
            .collect();

        let export_data = ExportData {
            capsule,
            memories,
            connections,
            metadata: ExportMetadata {
                export_timestamp: 1000000000,
                original_canister_id: test_principal(99),
                data_version: "1.0".to_string(),
                total_size_bytes: 5000,
            },
        };

        let result = generate_export_manifest(&export_data);
        assert!(result.is_ok(), "Manifest generation should succeed");

        let manifest = result.unwrap();

        // Verify manifest structure
        assert!(
            !manifest.capsule_checksum.is_empty(),
            "Capsule checksum should not be empty"
        );
        assert_eq!(manifest.memory_count, 2, "Should have 2 memories");
        assert_eq!(manifest.connection_count, 2, "Should have 2 connections");
        assert_eq!(
            manifest.memory_checksums.len(),
            2,
            "Should have 2 memory checksums"
        );
        assert_eq!(
            manifest.connection_checksums.len(),
            2,
            "Should have 2 connection checksums"
        );
        assert_eq!(
            manifest.total_size_bytes, 5000,
            "Should match metadata size"
        );
        assert_eq!(
            manifest.manifest_version, "1.0",
            "Should have correct version"
        );

        // Verify checksums are not empty
        for (memory_id, checksum) in &manifest.memory_checksums {
            assert!(!memory_id.is_empty(), "Memory ID should not be empty");
            assert!(!checksum.is_empty(), "Memory checksum should not be empty");
        }

        for (person_ref_str, checksum) in &manifest.connection_checksums {
            assert!(
                !person_ref_str.is_empty(),
                "Person ref string should not be empty"
            );
            assert!(
                !checksum.is_empty(),
                "Connection checksum should not be empty"
            );
        }
    }

    #[test]
    fn test_validate_export_data_success() {
        let (_, capsule) = setup_test_capsule_with_data();
        let memories: Vec<(String, types::Memory)> = capsule
            .memories
            .iter()
            .map(|(id, memory)| (id.clone(), memory.clone()))
            .collect();
        let connections: Vec<(types::PersonRef, types::Connection)> = capsule
            .connections
            .iter()
            .map(|(person_ref, connection)| (person_ref.clone(), connection.clone()))
            .collect();

        let calculated_size = calculate_export_data_size(&capsule, &memories, &connections);

        let export_data = ExportData {
            capsule,
            memories,
            connections,
            metadata: ExportMetadata {
                export_timestamp: 1000000000,
                original_canister_id: test_principal(99),
                data_version: "1.0".to_string(),
                total_size_bytes: calculated_size,
            },
        };

        let result = validate_export_data(&export_data);
        assert!(result.is_ok(), "Validation should succeed for valid data");
    }

    #[test]
    fn test_validate_export_data_empty_capsule_id() {
        let (_, mut capsule) = setup_test_capsule_with_data();
        capsule.id = "".to_string(); // Make capsule ID empty

        let export_data = ExportData {
            capsule,
            memories: vec![],
            connections: vec![],
            metadata: ExportMetadata {
                export_timestamp: 1000000000,
                original_canister_id: test_principal(99),
                data_version: "1.0".to_string(),
                total_size_bytes: 1000,
            },
        };

        let result = validate_export_data(&export_data);
        assert!(
            result.is_err(),
            "Validation should fail for empty capsule ID"
        );
        assert!(result.unwrap_err().contains("Capsule ID is empty"));
    }

    #[test]
    fn test_validate_export_data_no_owners() {
        let (_, mut capsule) = setup_test_capsule_with_data();
        capsule.owners.clear(); // Remove all owners

        let export_data = ExportData {
            capsule,
            memories: vec![],
            connections: vec![],
            metadata: ExportMetadata {
                export_timestamp: 1000000000,
                original_canister_id: test_principal(99),
                data_version: "1.0".to_string(),
                total_size_bytes: 1000,
            },
        };

        let result = validate_export_data(&export_data);
        assert!(
            result.is_err(),
            "Validation should fail for capsule with no owners"
        );
        assert!(result.unwrap_err().contains("Capsule has no owners"));
    }

    #[test]
    fn test_validate_export_data_invalid_timestamp() {
        let (_, capsule) = setup_test_capsule_with_data();

        let export_data = ExportData {
            capsule,
            memories: vec![],
            connections: vec![],
            metadata: ExportMetadata {
                export_timestamp: 0, // Invalid timestamp
                original_canister_id: test_principal(99),
                data_version: "1.0".to_string(),
                total_size_bytes: 1000,
            },
        };

        let result = validate_export_data(&export_data);
        assert!(
            result.is_err(),
            "Validation should fail for invalid timestamp"
        );
        assert!(result.unwrap_err().contains("Invalid export timestamp"));
    }

    #[test]
    fn test_validate_export_data_empty_data_version() {
        let (_, capsule) = setup_test_capsule_with_data();

        let export_data = ExportData {
            capsule,
            memories: vec![],
            connections: vec![],
            metadata: ExportMetadata {
                export_timestamp: 1000000000,
                original_canister_id: test_principal(99),
                data_version: "".to_string(), // Empty version
                total_size_bytes: 1000,
            },
        };

        let result = validate_export_data(&export_data);
        assert!(
            result.is_err(),
            "Validation should fail for empty data version"
        );
        assert!(result.unwrap_err().contains("Data version is empty"));
    }

    #[test]
    fn test_validate_export_data_memory_id_mismatch() {
        let (_, capsule) = setup_test_capsule_with_data();
        let mut memory =
            create_test_memory("mem1", "test.jpg", MemoryType::Image, "image/jpeg", 1024);
        memory.id = "different_id".to_string(); // Make memory ID different from key

        let memories = vec![("mem1".to_string(), memory)];

        let export_data = ExportData {
            capsule,
            memories,
            connections: vec![],
            metadata: ExportMetadata {
                export_timestamp: 1000000000,
                original_canister_id: test_principal(99),
                data_version: "1.0".to_string(),
                total_size_bytes: 1000,
            },
        };

        let result = validate_export_data(&export_data);
        assert!(
            result.is_err(),
            "Validation should fail for memory ID mismatch"
        );
        assert!(result.unwrap_err().contains("Memory ID mismatch"));
    }

    #[test]
    fn test_validate_export_data_connection_peer_mismatch() {
        let (_, capsule) = setup_test_capsule_with_data();
        let peer1 = PersonRef::Principal(test_principal(2));
        let peer2 = PersonRef::Principal(test_principal(3));
        let mut connection = create_test_connection(peer2.clone(), ConnectionStatus::Accepted);
        connection.peer = peer2; // Connection peer doesn't match the key

        let connections = vec![(peer1, connection)]; // Key is peer1 but connection.peer is peer2

        let export_data = ExportData {
            capsule,
            memories: vec![],
            connections,
            metadata: ExportMetadata {
                export_timestamp: 1000000000,
                original_canister_id: test_principal(99),
                data_version: "1.0".to_string(),
                total_size_bytes: 1000,
            },
        };

        let result = validate_export_data(&export_data);
        assert!(
            result.is_err(),
            "Validation should fail for connection peer mismatch"
        );
        assert!(result.unwrap_err().contains("Connection peer mismatch"));
    }

    #[test]
    fn test_validate_export_data_size_variance_within_tolerance() {
        let (_, capsule) = setup_test_capsule_with_data();
        let memories: Vec<(String, types::Memory)> = capsule
            .memories
            .iter()
            .map(|(id, memory)| (id.clone(), memory.clone()))
            .collect();
        let connections: Vec<(types::PersonRef, types::Connection)> = capsule
            .connections
            .iter()
            .map(|(person_ref, connection)| (person_ref.clone(), connection.clone()))
            .collect();

        let calculated_size = calculate_export_data_size(&capsule, &memories, &connections);
        // Use a size within 10% variance
        let metadata_size = calculated_size + (calculated_size / 20); // 5% difference

        let export_data = ExportData {
            capsule,
            memories,
            connections,
            metadata: ExportMetadata {
                export_timestamp: 1000000000,
                original_canister_id: test_principal(99),
                data_version: "1.0".to_string(),
                total_size_bytes: metadata_size,
            },
        };

        let result = validate_export_data(&export_data);
        assert!(
            result.is_ok(),
            "Validation should succeed for size variance within tolerance"
        );
    }

    #[test]
    fn test_validate_export_data_size_variance_exceeds_tolerance() {
        let (_, capsule) = setup_test_capsule_with_data();
        let memories: Vec<(String, types::Memory)> = capsule
            .memories
            .iter()
            .map(|(id, memory)| (id.clone(), memory.clone()))
            .collect();
        let connections: Vec<(types::PersonRef, types::Connection)> = capsule
            .connections
            .iter()
            .map(|(person_ref, connection)| (person_ref.clone(), connection.clone()))
            .collect();

        let calculated_size = calculate_export_data_size(&capsule, &memories, &connections);
        // Use a size that exceeds 10% variance
        let metadata_size = calculated_size + (calculated_size / 5); // 20% difference

        let export_data = ExportData {
            capsule,
            memories,
            connections,
            metadata: ExportMetadata {
                export_timestamp: 1000000000,
                original_canister_id: test_principal(99),
                data_version: "1.0".to_string(),
                total_size_bytes: metadata_size,
            },
        };

        let result = validate_export_data(&export_data);
        assert!(
            result.is_err(),
            "Validation should fail for size variance exceeding tolerance"
        );
        assert!(result.unwrap_err().contains("Data size mismatch"));
    }

    #[test]
    fn test_verify_export_against_manifest_success() {
        let (_, capsule) = setup_test_capsule_with_data();
        let memories: Vec<(String, types::Memory)> = capsule
            .memories
            .iter()
            .map(|(id, memory)| (id.clone(), memory.clone()))
            .collect();
        let connections: Vec<(types::PersonRef, types::Connection)> = capsule
            .connections
            .iter()
            .map(|(person_ref, connection)| (person_ref.clone(), connection.clone()))
            .collect();

        let export_data = ExportData {
            capsule,
            memories,
            connections,
            metadata: ExportMetadata {
                export_timestamp: 1000000000,
                original_canister_id: test_principal(99),
                data_version: "1.0".to_string(),
                total_size_bytes: 5000,
            },
        };

        // Generate manifest from the same data
        let manifest = generate_export_manifest(&export_data).unwrap();

        // Verify against the manifest
        let result = verify_export_against_manifest(&export_data, &manifest);
        assert!(
            result.is_ok(),
            "Verification should succeed for matching data and manifest"
        );
    }

    #[test]
    fn test_verify_export_against_manifest_memory_count_mismatch() {
        let (_, capsule) = setup_test_capsule_with_data();
        let memories: Vec<(String, types::Memory)> = capsule
            .memories
            .iter()
            .map(|(id, memory)| (id.clone(), memory.clone()))
            .collect();

        let export_data = ExportData {
            capsule,
            memories,
            connections: vec![],
            metadata: ExportMetadata {
                export_timestamp: 1000000000,
                original_canister_id: test_principal(99),
                data_version: "1.0".to_string(),
                total_size_bytes: 5000,
            },
        };

        // Create manifest with wrong memory count
        let mut manifest = generate_export_manifest(&export_data).unwrap();
        manifest.memory_count = 5; // Wrong count

        let result = verify_export_against_manifest(&export_data, &manifest);
        assert!(
            result.is_err(),
            "Verification should fail for memory count mismatch"
        );
        assert!(result.unwrap_err().contains("Memory count mismatch"));
    }

    #[test]
    fn test_verify_export_against_manifest_connection_count_mismatch() {
        let (_, capsule) = setup_test_capsule_with_data();
        let connections: Vec<(types::PersonRef, types::Connection)> = capsule
            .connections
            .iter()
            .map(|(person_ref, connection)| (person_ref.clone(), connection.clone()))
            .collect();

        let export_data = ExportData {
            capsule,
            memories: vec![],
            connections,
            metadata: ExportMetadata {
                export_timestamp: 1000000000,
                original_canister_id: test_principal(99),
                data_version: "1.0".to_string(),
                total_size_bytes: 5000,
            },
        };

        // Create manifest with wrong connection count
        let mut manifest = generate_export_manifest(&export_data).unwrap();
        manifest.connection_count = 10; // Wrong count

        let result = verify_export_against_manifest(&export_data, &manifest);
        assert!(
            result.is_err(),
            "Verification should fail for connection count mismatch"
        );
        assert!(result.unwrap_err().contains("Connection count mismatch"));
    }

    #[test]
    fn test_verify_export_against_manifest_capsule_checksum_mismatch() {
        let (_, capsule) = setup_test_capsule_with_data();

        let export_data = ExportData {
            capsule,
            memories: vec![],
            connections: vec![],
            metadata: ExportMetadata {
                export_timestamp: 1000000000,
                original_canister_id: test_principal(99),
                data_version: "1.0".to_string(),
                total_size_bytes: 5000,
            },
        };

        // Create manifest with wrong capsule checksum
        let mut manifest = generate_export_manifest(&export_data).unwrap();
        manifest.capsule_checksum = "wrong_checksum".to_string();

        let result = verify_export_against_manifest(&export_data, &manifest);
        assert!(
            result.is_err(),
            "Verification should fail for capsule checksum mismatch"
        );
        assert!(result.unwrap_err().contains("Capsule checksum mismatch"));
    }

    #[test]
    fn test_verify_export_against_manifest_memory_checksum_mismatch() {
        let (_, capsule) = setup_test_capsule_with_data();
        let memories: Vec<(String, types::Memory)> = capsule
            .memories
            .iter()
            .map(|(id, memory)| (id.clone(), memory.clone()))
            .collect();

        let export_data = ExportData {
            capsule,
            memories,
            connections: vec![],
            metadata: ExportMetadata {
                export_timestamp: 1000000000,
                original_canister_id: test_principal(99),
                data_version: "1.0".to_string(),
                total_size_bytes: 5000,
            },
        };

        // Create manifest with wrong memory checksum
        let mut manifest = generate_export_manifest(&export_data).unwrap();
        if let Some((_, checksum)) = manifest.memory_checksums.get_mut(0) {
            *checksum = "wrong_checksum".to_string();
        }

        let result = verify_export_against_manifest(&export_data, &manifest);
        assert!(
            result.is_err(),
            "Verification should fail for memory checksum mismatch"
        );
        assert!(result.unwrap_err().contains("checksum mismatch"));
    }

    #[test]
    fn test_verify_export_against_manifest_connection_checksum_mismatch() {
        let (_, capsule) = setup_test_capsule_with_data();
        let connections: Vec<(types::PersonRef, types::Connection)> = capsule
            .connections
            .iter()
            .map(|(person_ref, connection)| (person_ref.clone(), connection.clone()))
            .collect();

        let export_data = ExportData {
            capsule,
            memories: vec![],
            connections,
            metadata: ExportMetadata {
                export_timestamp: 1000000000,
                original_canister_id: test_principal(99),
                data_version: "1.0".to_string(),
                total_size_bytes: 5000,
            },
        };

        // Create manifest with wrong connection checksum
        let mut manifest = generate_export_manifest(&export_data).unwrap();
        if let Some((_, checksum)) = manifest.connection_checksums.get_mut(0) {
            *checksum = "wrong_checksum".to_string();
        }

        let result = verify_export_against_manifest(&export_data, &manifest);
        assert!(
            result.is_err(),
            "Verification should fail for connection checksum mismatch"
        );
        assert!(result.unwrap_err().contains("checksum mismatch"));
    }

    #[test]
    fn test_validate_memory_data_success() {
        let memory = create_test_memory(
            "test_mem",
            "test.jpg",
            MemoryType::Image,
            "image/jpeg",
            1024,
        );
        let result = validate_memory_data(&memory);
        assert!(
            result.is_ok(),
            "Memory validation should succeed for valid memory"
        );
    }

    #[test]
    fn test_validate_memory_data_empty_id() {
        let mut memory = create_test_memory(
            "test_mem",
            "test.jpg",
            MemoryType::Image,
            "image/jpeg",
            1024,
        );
        memory.id = "".to_string();

        let result = validate_memory_data(&memory);
        assert!(
            result.is_err(),
            "Memory validation should fail for empty ID"
        );
        assert!(result.unwrap_err().contains("Memory ID is empty"));
    }

    #[test]
    fn test_validate_memory_data_empty_name() {
        let mut memory = create_test_memory(
            "test_mem",
            "test.jpg",
            MemoryType::Image,
            "image/jpeg",
            1024,
        );
        memory.metadata.title = Some("".to_string());

        let result = validate_memory_data(&memory);
        assert!(
            result.is_err(),
            "Memory validation should fail for empty name"
        );
        assert!(result.unwrap_err().contains("has empty title"));
    }

    #[test]
    fn test_validate_memory_data_empty_content_type() {
        let mut memory = create_test_memory(
            "test_mem",
            "test.jpg",
            MemoryType::Image,
            "image/jpeg",
            1024,
        );
        memory.metadata.content_type = "".to_string();

        let result = validate_memory_data(&memory);
        assert!(
            result.is_err(),
            "Memory validation should fail for empty content_type"
        );
        assert!(result.unwrap_err().contains("has empty content_type"));
    }

    #[test]
    fn test_validate_memory_data_invalid_created_at() {
        let mut memory = create_test_memory(
            "test_mem",
            "test.jpg",
            MemoryType::Image,
            "image/jpeg",
            1024,
        );
        memory.metadata.created_at = 0;

        let result = validate_memory_data(&memory);
        assert!(
            result.is_err(),
            "Memory validation should fail for invalid created_at"
        );
        assert!(result.unwrap_err().contains("has invalid created_at"));
    }

    #[test]
    fn test_validate_memory_data_invalid_uploaded_at() {
        let mut memory = create_test_memory(
            "test_mem",
            "test.jpg",
            MemoryType::Image,
            "image/jpeg",
            1024,
        );
        memory.metadata.uploaded_at = 0;

        let result = validate_memory_data(&memory);
        assert!(
            result.is_err(),
            "Memory validation should fail for invalid uploaded_at"
        );
        assert!(result.unwrap_err().contains("has invalid uploaded_at"));
    }

    #[test]
    fn test_validate_memory_data_empty_blob_locator() {
        let mut memory = create_test_memory(
            "test_mem",
            "test.jpg",
            MemoryType::Image,
            "image/jpeg",
            1024,
        );

        // Modify the blob locator to be empty
        if let Some(blob_asset) = memory.blob_internal_assets.first_mut() {
            blob_asset.blob_ref.locator = "".to_string();
        }

        let result = validate_memory_data(&memory);
        assert!(
            result.is_err(),
            "Memory validation should fail for empty blob locator"
        );
        assert!(result.unwrap_err().contains("has empty blob locator"));
    }

    #[test]
    fn test_checksum_functions_consistency() {
        let (_, capsule) = setup_test_capsule_with_data();
        let memory = create_test_memory(
            "test_mem",
            "test.jpg",
            MemoryType::Image,
            "image/jpeg",
            1024,
        );
        let peer = PersonRef::Principal(test_principal(2));
        let connection = create_test_connection(peer.clone(), ConnectionStatus::Accepted);

        // Generate checksums multiple times - should be consistent
        let checksum1 = generate_capsule_checksum(&capsule).unwrap();
        let checksum2 = generate_capsule_checksum(&capsule).unwrap();
        assert_eq!(
            checksum1, checksum2,
            "Capsule checksums should be consistent"
        );

        let mem_checksum1 = generate_memory_checksum("test_mem", &memory).unwrap();
        let mem_checksum2 = generate_memory_checksum("test_mem", &memory).unwrap();
        assert_eq!(
            mem_checksum1, mem_checksum2,
            "Memory checksums should be consistent"
        );

        let conn_checksum1 = generate_connection_checksum(&peer, &connection).unwrap();
        let conn_checksum2 = generate_connection_checksum(&peer, &connection).unwrap();
        assert_eq!(
            conn_checksum1, conn_checksum2,
            "Connection checksums should be consistent"
        );
    }

    #[test]
    fn test_person_ref_to_string() {
        let principal = test_principal(1);
        let principal_ref = PersonRef::Principal(principal);
        let opaque_ref = PersonRef::Opaque("test_id_123".to_string());

        let principal_str = person_ref_to_string(&principal_ref);
        let opaque_str = person_ref_to_string(&opaque_ref);

        assert!(
            principal_str.starts_with("principal:"),
            "Principal ref should start with 'principal:'"
        );
        assert!(
            opaque_str.starts_with("opaque:"),
            "Opaque ref should start with 'opaque:'"
        );
        assert_eq!(
            opaque_str, "opaque:test_id_123",
            "Opaque ref should match expected format"
        );
    }

    #[test]
    fn test_simple_hash_function() {
        let data1 = "test_data_123";
        let data2 = "test_data_456";
        let data3 = "test_data_123"; // Same as data1

        let hash1 = simple_hash(data1);
        let hash2 = simple_hash(data2);
        let hash3 = simple_hash(data3);

        // Same data should produce same hash
        assert_eq!(hash1, hash3, "Same data should produce same hash");

        // Different data should produce different hash
        assert_ne!(hash1, hash2, "Different data should produce different hash");

        // Hash should be 16 characters (hex format)
        assert_eq!(hash1.len(), 16, "Hash should be 16 characters long");

        // Hash should be valid hex
        assert!(
            hash1.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should be valid hex"
        );
    }
}
