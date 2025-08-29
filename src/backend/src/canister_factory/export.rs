use crate::canister_factory::types::*;
use crate::types;
use candid::Principal;

/// Export user's capsule data for migration
/// This function serializes all capsule data including metadata, memories, and connections
pub fn export_user_capsule_data(user: Principal) -> Result<ExportData, String> {
    let user_ref = types::PersonRef::Principal(user);

    // Find the user's self-capsule (where user is both subject and owner)
    let capsule = crate::memory::with_capsules(|capsules| {
        capsules
            .values()
            .find(|capsule| capsule.subject == user_ref && capsule.owners.contains_key(&user_ref))
            .cloned()
    });

    let capsule = match capsule {
        Some(c) => c,
        None => return Err(format!("No self-capsule found for user {}", user)),
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
        export_timestamp: ic_cdk::api::time(),
        original_canister_id: ic_cdk::api::canister_self(),
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
    capsule: &types::Capsule,
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
        if let Some(ref data) = memory.data.data {
            total_size += data.len() as u64;
        }

        // Memory metadata specific sizes
        match &memory.metadata {
            types::MemoryMetadata::Image(img_meta) => {
                total_size += img_meta.base.original_name.len() as u64;
                total_size += img_meta.base.mime_type.len() as u64;
            }
            types::MemoryMetadata::Video(vid_meta) => {
                total_size += vid_meta.base.original_name.len() as u64;
                total_size += vid_meta.base.mime_type.len() as u64;
                if let Some(ref thumbnail) = vid_meta.thumbnail {
                    total_size += thumbnail.len() as u64;
                }
            }
            types::MemoryMetadata::Audio(audio_meta) => {
                total_size += audio_meta.base.original_name.len() as u64;
                total_size += audio_meta.base.mime_type.len() as u64;
            }
            types::MemoryMetadata::Document(doc_meta) => {
                total_size += doc_meta.base.original_name.len() as u64;
                total_size += doc_meta.base.mime_type.len() as u64;
            }
            types::MemoryMetadata::Note(note_meta) => {
                total_size += note_meta.base.original_name.len() as u64;
                total_size += note_meta.base.mime_type.len() as u64;
                if let Some(ref tags) = note_meta.tags {
                    total_size += tags.iter().map(|tag| tag.len() as u64).sum::<u64>();
                }
            }
        }
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
    let size_diff = if calculated_size > export_data.metadata.total_size_bytes {
        calculated_size - export_data.metadata.total_size_bytes
    } else {
        export_data.metadata.total_size_bytes - calculated_size
    };

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

    if memory.info.name.is_empty() {
        return Err(format!("Memory '{}' has empty name", memory.id));
    }

    if memory.info.content_type.is_empty() {
        return Err(format!("Memory '{}' has empty content_type", memory.id));
    }

    // Validate timestamps
    if memory.info.created_at == 0 {
        return Err(format!("Memory '{}' has invalid created_at", memory.id));
    }

    if memory.info.uploaded_at == 0 {
        return Err(format!("Memory '{}' has invalid uploaded_at", memory.id));
    }

    // Validate metadata consistency
    match &memory.metadata {
        types::MemoryMetadata::Image(img_meta) => {
            if img_meta.base.mime_type.is_empty() {
                return Err(format!("Image memory '{}' has empty mime_type", memory.id));
            }
        }
        types::MemoryMetadata::Video(vid_meta) => {
            if vid_meta.base.mime_type.is_empty() {
                return Err(format!("Video memory '{}' has empty mime_type", memory.id));
            }
        }
        types::MemoryMetadata::Audio(audio_meta) => {
            if audio_meta.base.mime_type.is_empty() {
                return Err(format!("Audio memory '{}' has empty mime_type", memory.id));
            }
        }
        types::MemoryMetadata::Document(doc_meta) => {
            if doc_meta.base.mime_type.is_empty() {
                return Err(format!(
                    "Document memory '{}' has empty mime_type",
                    memory.id
                ));
            }
        }
        types::MemoryMetadata::Note(note_meta) => {
            if note_meta.base.mime_type.is_empty() {
                return Err(format!("Note memory '{}' has empty mime_type", memory.id));
            }
        }
    }

    // Validate blob reference
    if memory.data.blob_ref.locator.is_empty() {
        return Err(format!("Memory '{}' has empty blob locator", memory.id));
    }

    Ok(())
}

/// Generate checksum for capsule data
fn generate_capsule_checksum(capsule: &types::Capsule) -> Result<String, String> {
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
    let memory_data = format!(
        "{}|{}|{}|{}|{}|{}|{}",
        memory_id,
        memory.info.name,
        memory.info.content_type,
        memory.info.created_at,
        memory.info.uploaded_at,
        memory.data.blob_ref.locator,
        memory.data.data.as_ref().map_or(0, |d| d.len())
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
        types::PersonRef::Opaque(id) => format!("opaque:{}", id),
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
    format!("{:016x}", hash)
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
            .ok_or_else(|| format!("Memory '{}' not found in manifest", memory_id))?;

        if memory_checksum != *expected_checksum {
            return Err(format!(
                "Memory '{}' checksum mismatch: calculated '{}', manifest expects '{}'",
                memory_id, memory_checksum, expected_checksum
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
            .ok_or_else(|| format!("Connection '{}' not found in manifest", person_ref_string))?;

        if connection_checksum != *expected_checksum {
            return Err(format!(
                "Connection '{}' checksum mismatch: calculated '{}', manifest expects '{}'",
                person_ref_string, connection_checksum, expected_checksum
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
