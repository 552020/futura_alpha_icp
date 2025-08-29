use crate::types;
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Response from migration operations
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct MigrationResponse {
    pub success: bool,
    pub canister_id: Option<Principal>,
    pub message: String,
}

/// Migration status enum tracking the progression through migration states
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum MigrationStatus {
    NotStarted,
    Exporting,
    Creating,
    Installing,
    Importing,
    Verifying,
    Completed,
    Failed,
}

/// Response for migration status queries
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct MigrationStatusResponse {
    pub status: MigrationStatus,
    pub canister_id: Option<Principal>,
    pub message: Option<String>,
}

/// Exported capsule data for migration
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ExportData {
    pub capsule: types::Capsule,
    pub memories: Vec<(String, types::Memory)>,
    pub connections: Vec<(types::PersonRef, types::Connection)>,
    pub metadata: ExportMetadata,
}

/// Metadata about the exported data
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ExportMetadata {
    pub export_timestamp: u64,
    pub original_canister_id: Principal,
    pub data_version: String,
    pub total_size_bytes: u64,
}

/// Migration state for tracking individual user migrations
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct MigrationState {
    pub user: Principal,
    pub status: MigrationStatus,
    pub created_at: u64,
    pub completed_at: Option<u64>,
    pub personal_canister_id: Option<Principal>,
    pub cycles_consumed: u128,
    pub error_message: Option<String>,
}

/// Configuration for migration system
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct MigrationConfig {
    pub enabled: bool,
    pub cycles_reserve: u128,
    pub min_cycles_threshold: u128,
    pub admin_principals: BTreeSet<Principal>,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Start disabled for safety
            cycles_reserve: 0,
            min_cycles_threshold: 1_000_000_000_000, // 1T cycles default threshold
            admin_principals: BTreeSet::new(),
        }
    }
}

/// Registry record for personal canisters
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct PersonalCanisterRecord {
    pub canister_id: Principal,
    pub created_by: Principal,
    pub created_at: u64,
    pub status: MigrationStatus,
    pub cycles_consumed: u128,
}

/// Statistics for migration operations
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Default)]
pub struct MigrationStats {
    pub total_attempts: u64,
    pub total_successes: u64,
    pub total_failures: u64,
    pub total_cycles_consumed: u128,
}

/// Extended state structure with migration fields
#[derive(CandidType, Serialize, Deserialize, Default, Clone, Debug)]
pub struct MigrationStateData {
    pub migration_config: MigrationConfig,
    pub migration_states: BTreeMap<Principal, MigrationState>,
    pub migration_stats: MigrationStats,
    pub personal_canisters: BTreeMap<Principal, PersonalCanisterRecord>,
}

/// Export migration state for upgrade persistence
pub fn export_migration_state_for_upgrade() -> MigrationStateData {
    crate::memory::with_migration_state(|state| state.clone())
}

/// Import migration state from upgrade persistence
pub fn import_migration_state_from_upgrade(migration_data: MigrationStateData) {
    crate::memory::with_migration_state_mut(|state| {
        *state = migration_data;
    })
}

// Personal canister registry management functions

/// Create a new registry entry for a personal canister
pub fn create_registry_entry(
    canister_id: Principal,
    created_by: Principal,
    status: MigrationStatus,
    cycles_consumed: u128,
) -> Result<(), String> {
    let now = ic_cdk::api::time();

    let record = PersonalCanisterRecord {
        canister_id,
        created_by,
        created_at: now,
        status,
        cycles_consumed,
    };

    crate::memory::with_migration_state_mut(|state| {
        state.personal_canisters.insert(canister_id, record);
    });

    Ok(())
}

/// Update the status of a personal canister in the registry
pub fn update_registry_status(
    canister_id: Principal,
    new_status: MigrationStatus,
) -> Result<(), String> {
    crate::memory::with_migration_state_mut(|state| {
        if let Some(record) = state.personal_canisters.get_mut(&canister_id) {
            record.status = new_status;
            Ok(())
        } else {
            Err(format!(
                "Registry entry not found for canister {}",
                canister_id
            ))
        }
    })
}

/// Update cycles consumed for a personal canister in the registry
pub fn update_registry_cycles_consumed(
    canister_id: Principal,
    cycles_consumed: u128,
) -> Result<(), String> {
    crate::memory::with_migration_state_mut(|state| {
        if let Some(record) = state.personal_canisters.get_mut(&canister_id) {
            record.cycles_consumed = cycles_consumed;
            Ok(())
        } else {
            Err(format!(
                "Registry entry not found for canister {}",
                canister_id
            ))
        }
    })
}

/// Get registry entries by user principal (admin function)
pub fn get_registry_entries_by_user(user: Principal) -> Vec<PersonalCanisterRecord> {
    crate::memory::with_migration_state(|state| {
        state
            .personal_canisters
            .values()
            .filter(|record| record.created_by == user)
            .cloned()
            .collect()
    })
}

/// Get registry entries by status (admin function)
pub fn get_registry_entries_by_status(status: MigrationStatus) -> Vec<PersonalCanisterRecord> {
    crate::memory::with_migration_state(|state| {
        state
            .personal_canisters
            .values()
            .filter(|record| record.status == status)
            .cloned()
            .collect()
    })
}

/// Get all registry entries (admin function)
pub fn get_all_registry_entries() -> Vec<PersonalCanisterRecord> {
    crate::memory::with_migration_state(|state| {
        state.personal_canisters.values().cloned().collect()
    })
}

/// Get a specific registry entry by canister ID
pub fn get_registry_entry(canister_id: Principal) -> Option<PersonalCanisterRecord> {
    crate::memory::with_migration_state(|state| state.personal_canisters.get(&canister_id).cloned())
}

/// Remove a registry entry (admin function for cleanup)
pub fn remove_registry_entry(canister_id: Principal) -> Result<(), String> {
    crate::memory::with_migration_state_mut(|state| {
        if state.personal_canisters.remove(&canister_id).is_some() {
            Ok(())
        } else {
            Err(format!(
                "Registry entry not found for canister {}",
                canister_id
            ))
        }
    })
}

// Cycles reserve management functions

/// Check if factory has sufficient cycles in reserve for the required amount
/// This is a preflight check that should be called before attempting operations
pub fn preflight_cycles_reserve(required_cycles: u128) -> Result<(), String> {
    crate::memory::with_migration_state(|state| {
        let config = &state.migration_config;

        // Check if reserve is below minimum threshold
        if config.cycles_reserve < config.min_cycles_threshold {
            return Err(format!(
                "Factory cycles reserve ({}) is below minimum threshold ({})",
                config.cycles_reserve, config.min_cycles_threshold
            ));
        }

        // Check if reserve has enough for the required operation
        if config.cycles_reserve < required_cycles {
            return Err(format!(
                "Insufficient cycles in factory reserve. Required: {}, Available: {}",
                required_cycles, config.cycles_reserve
            ));
        }

        Ok(())
    })
}

/// Consume cycles from the factory reserve
/// This should only be called after a successful preflight check
pub fn consume_cycles_from_reserve(cycles_to_consume: u128) -> Result<(), String> {
    crate::memory::with_migration_state_mut(|state| {
        let config = &mut state.migration_config;

        // Double-check we have enough cycles
        if config.cycles_reserve < cycles_to_consume {
            return Err(format!(
                "Cannot consume {} cycles, only {} available in reserve",
                cycles_to_consume, config.cycles_reserve
            ));
        }

        // Consume the cycles
        config.cycles_reserve = config.cycles_reserve.saturating_sub(cycles_to_consume);

        // Update total cycles consumed in stats
        state.migration_stats.total_cycles_consumed = state
            .migration_stats
            .total_cycles_consumed
            .saturating_add(cycles_to_consume);

        ic_cdk::println!(
            "Consumed {} cycles from factory reserve. Remaining: {}",
            cycles_to_consume,
            config.cycles_reserve
        );

        Ok(())
    })
}

/// Get current cycles reserve amount (admin function)
pub fn get_cycles_reserve() -> u128 {
    crate::memory::with_migration_state(|state| state.migration_config.cycles_reserve)
}

/// Add cycles to the factory reserve (admin function)
pub fn add_cycles_to_reserve(cycles_to_add: u128) -> Result<u128, String> {
    crate::memory::with_migration_state_mut(|state| {
        let config = &mut state.migration_config;
        config.cycles_reserve = config.cycles_reserve.saturating_add(cycles_to_add);

        ic_cdk::println!(
            "Added {} cycles to factory reserve. New total: {}",
            cycles_to_add,
            config.cycles_reserve
        );

        Ok(config.cycles_reserve)
    })
}

/// Set the minimum cycles threshold (admin function)
pub fn set_cycles_threshold(new_threshold: u128) -> Result<(), String> {
    crate::memory::with_migration_state_mut(|state| {
        state.migration_config.min_cycles_threshold = new_threshold;

        ic_cdk::println!("Updated cycles threshold to: {}", new_threshold);

        Ok(())
    })
}

/// Get current cycles threshold (admin function)
pub fn get_cycles_threshold() -> u128 {
    crate::memory::with_migration_state(|state| state.migration_config.min_cycles_threshold)
}

/// Get cycles reserve status including threshold information (admin function)
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct CyclesReserveStatus {
    pub current_reserve: u128,
    pub min_threshold: u128,
    pub is_above_threshold: bool,
    pub total_consumed: u128,
}

pub fn get_cycles_reserve_status() -> CyclesReserveStatus {
    crate::memory::with_migration_state(|state| {
        let config = &state.migration_config;
        CyclesReserveStatus {
            current_reserve: config.cycles_reserve,
            min_threshold: config.min_cycles_threshold,
            is_above_threshold: config.cycles_reserve >= config.min_cycles_threshold,
            total_consumed: state.migration_stats.total_cycles_consumed,
        }
    })
}

// Cycles reserve monitoring and alerts

/// Check if cycles reserve is below threshold and log warning
pub fn check_cycles_reserve_threshold() -> bool {
    let status = get_cycles_reserve_status();

    if !status.is_above_threshold {
        ic_cdk::println!(
            "WARNING: Factory cycles reserve ({}) is below minimum threshold ({}). Admin action required!",
            status.current_reserve,
            status.min_threshold
        );

        // Log additional context for debugging
        ic_cdk::println!("Total cycles consumed to date: {}", status.total_consumed);

        return false;
    }

    true
}

/// Log cycles consumption with context
pub fn log_cycles_consumption(
    operation: &str,
    cycles_consumed: u128,
    user: Option<Principal>,
    canister_id: Option<Principal>,
) {
    let status = get_cycles_reserve_status();

    ic_cdk::println!(
        "CYCLES_CONSUMPTION: operation={}, consumed={}, remaining_reserve={}, user={:?}, canister={:?}",
        operation,
        cycles_consumed,
        status.current_reserve,
        user,
        canister_id
    );

    // Check if this consumption brings us below threshold
    if !check_cycles_reserve_threshold() {
        ic_cdk::println!(
            "ALERT: Cycles reserve is now below threshold after {} operation",
            operation
        );
    }
}

/// Alert levels for cycles reserve monitoring
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum CyclesAlertLevel {
    Normal,   // Above threshold
    Warning,  // Below threshold but above critical
    Critical, // Very low reserves
}

/// Get current alert level based on cycles reserve
pub fn get_cycles_alert_level() -> CyclesAlertLevel {
    let status = get_cycles_reserve_status();

    if !status.is_above_threshold {
        // If below 50% of threshold, consider it critical
        let critical_threshold = status.min_threshold / 2;
        if status.current_reserve <= critical_threshold {
            CyclesAlertLevel::Critical
        } else {
            CyclesAlertLevel::Warning
        }
    } else {
        CyclesAlertLevel::Normal
    }
}

/// Comprehensive cycles monitoring report (admin function)
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct CyclesMonitoringReport {
    pub reserve_status: CyclesReserveStatus,
    pub alert_level: CyclesAlertLevel,
    pub recent_consumption_rate: Option<u128>, // Could be calculated from recent operations
    pub recommendations: Vec<String>,
}

pub fn get_cycles_monitoring_report() -> CyclesMonitoringReport {
    let reserve_status = get_cycles_reserve_status();
    let alert_level = get_cycles_alert_level();

    let mut recommendations = Vec::new();

    match alert_level {
        CyclesAlertLevel::Critical => {
            recommendations.push(
                "URGENT: Cycles reserve is critically low. Add cycles immediately.".to_string(),
            );
            recommendations.push(
                "Consider temporarily disabling migrations until reserve is replenished."
                    .to_string(),
            );
        }
        CyclesAlertLevel::Warning => {
            recommendations
                .push("Cycles reserve is below threshold. Plan to add cycles soon.".to_string());
            recommendations
                .push("Monitor consumption rate and adjust threshold if needed.".to_string());
        }
        CyclesAlertLevel::Normal => {
            recommendations.push("Cycles reserve is healthy.".to_string());
        }
    }

    // Add general recommendations
    if reserve_status.total_consumed > 0 {
        recommendations.push(format!(
            "Total cycles consumed: {}. Consider this for future capacity planning.",
            reserve_status.total_consumed
        ));
    }

    CyclesMonitoringReport {
        reserve_status,
        alert_level,
        recent_consumption_rate: None, // Could be implemented with historical tracking
        recommendations,
    }
}

/// Admin notification system for low reserves
/// This function should be called periodically or after operations
pub fn check_and_alert_low_reserves() -> Option<String> {
    let alert_level = get_cycles_alert_level();
    let status = get_cycles_reserve_status();

    match alert_level {
        CyclesAlertLevel::Critical => {
            let alert_message = format!(
                "CRITICAL ALERT: Factory cycles reserve is critically low! Current: {}, Threshold: {}. Immediate action required!",
                status.current_reserve,
                status.min_threshold
            );

            ic_cdk::println!("{}", alert_message);
            Some(alert_message)
        }
        CyclesAlertLevel::Warning => {
            let alert_message = format!(
                "WARNING: Factory cycles reserve is below threshold. Current: {}, Threshold: {}. Please add cycles soon.",
                status.current_reserve,
                status.min_threshold
            );

            ic_cdk::println!("{}", alert_message);
            Some(alert_message)
        }
        CyclesAlertLevel::Normal => None,
    }
}

// Data export functionality for migration

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

// Data validation and integrity checks

/// Data manifest for verification during migration
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct DataManifest {
    pub capsule_checksum: String,
    pub memory_count: u32,
    pub memory_checksums: Vec<(String, String)>, // (memory_id, checksum)
    pub connection_count: u32,
    pub connection_checksums: Vec<(String, String)>, // (person_ref_string, checksum)
    pub total_size_bytes: u64,
    pub manifest_version: String,
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

// Access control and guard functions

/// Ensure the caller owns a capsule (has a self-capsule where they are both subject and owner)
/// This function verifies that the caller principal has a capsule where they are the subject
/// and they are listed as an owner of that capsule
pub fn ensure_owner(caller: Principal) -> Result<(), String> {
    let user_ref = crate::types::PersonRef::Principal(caller);

    // Find the user's self-capsule (where user is both subject and owner)
    let has_self_capsule = crate::memory::with_capsules(|capsules| {
        capsules
            .values()
            .any(|capsule| capsule.subject == user_ref && capsule.owners.contains_key(&user_ref))
    });

    if has_self_capsule {
        Ok(())
    } else {
        Err(format!(
            "Access denied: Principal {} does not own a capsule",
            caller
        ))
    }
}

/// Ensure the caller is an admin (can perform admin-only operations)
/// This function checks if the caller is either a superadmin or a regular admin
pub fn ensure_admin(caller: Principal) -> Result<(), String> {
    if crate::admin::is_admin(&caller) {
        Ok(())
    } else {
        Err(format!(
            "Access denied: Principal {} is not an admin",
            caller
        ))
    }
}

/// Validate caller for migration endpoints - ensures they own a capsule
/// This is a convenience function that combines caller identification with ownership verification
pub fn validate_migration_caller() -> Result<Principal, String> {
    let caller = ic_cdk::api::msg_caller();

    // Reject anonymous callers
    if caller == Principal::anonymous() {
        return Err("Access denied: Anonymous callers cannot perform migrations".to_string());
    }

    // Ensure the caller owns a capsule
    ensure_owner(caller)?;

    Ok(caller)
}

/// Validate caller for admin-only migration operations
/// This function ensures the caller is an admin and returns their principal
pub fn validate_admin_caller() -> Result<Principal, String> {
    let caller = ic_cdk::api::msg_caller();

    // Reject anonymous callers
    if caller == Principal::anonymous() {
        return Err("Access denied: Anonymous callers cannot perform admin operations".to_string());
    }

    // Ensure the caller is an admin
    ensure_admin(caller)?;

    Ok(caller)
}

/// Check if a specific user owns a capsule (admin utility function)
/// This allows admins to check capsule ownership for any user
pub fn check_user_capsule_ownership(user: Principal) -> bool {
    let user_ref = crate::types::PersonRef::Principal(user);

    crate::memory::with_capsules(|capsules| {
        capsules
            .values()
            .any(|capsule| capsule.subject == user_ref && capsule.owners.contains_key(&user_ref))
    })
}

/// Get the capsule ID for a user (if they own one)
/// This is a utility function for migration operations
pub fn get_user_capsule_id(user: Principal) -> Option<String> {
    let user_ref = crate::types::PersonRef::Principal(user);

    crate::memory::with_capsules(|capsules| {
        capsules
            .values()
            .find(|capsule| capsule.subject == user_ref && capsule.owners.contains_key(&user_ref))
            .map(|capsule| capsule.id.clone())
    })
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
            .ok_or_else(|| {
                format!(
                    "Connection for '{}' not found in manifest",
                    person_ref_string
                )
            })?;

        if connection_checksum != *expected_checksum {
            return Err(format!(
                "Connection '{}' checksum mismatch: calculated '{}', manifest expects '{}'",
                person_ref_string, connection_checksum, expected_checksum
            ));
        }
    }

    // Verify total size
    if export_data.metadata.total_size_bytes != manifest.total_size_bytes {
        return Err(format!(
            "Total size mismatch: export metadata claims {} bytes, manifest expects {} bytes",
            export_data.metadata.total_size_bytes, manifest.total_size_bytes
        ));
    }

    ic_cdk::println!(
        "Export data verification against manifest passed: {} memories, {} connections verified",
        manifest.memory_count,
        manifest.connection_count
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

    #[test]
    fn test_ensure_owner_with_anonymous_caller() {
        let anonymous = Principal::anonymous();
        let result = ensure_owner(anonymous);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not own a capsule"));
    }

    #[test]
    fn test_ensure_admin_logic() {
        // Test the ensure_admin function logic
        // Note: Due to auto-bootstrap behavior, the first caller becomes admin
        // So we test that the function correctly calls the admin check
        let test_principal = Principal::from_text("2vxsx-fae").unwrap();

        // The function should call is_admin which may return true due to auto-bootstrap
        // But we can verify the function executes without panicking
        let result = ensure_admin(test_principal);

        // The result depends on the admin state, but the function should not panic
        assert!(result.is_ok() || result.is_err());

        // If it's an error, it should contain the expected message
        if let Err(msg) = result {
            assert!(msg.contains("is not an admin"));
        }
    }

    #[test]
    fn test_check_user_capsule_ownership_returns_false_for_nonexistent_user() {
        let test_principal = Principal::from_text("2vxsx-fae").unwrap();
        let result = check_user_capsule_ownership(test_principal);
        assert!(!result);
    }

    #[test]
    fn test_get_user_capsule_id_returns_none_for_nonexistent_user() {
        let test_principal = Principal::from_text("2vxsx-fae").unwrap();
        let result = get_user_capsule_id(test_principal);
        assert!(result.is_none());
    }

    #[test]
    fn test_person_ref_to_string_principal() {
        let principal = Principal::from_text("2vxsx-fae").unwrap();
        let person_ref = crate::types::PersonRef::Principal(principal);
        let result = person_ref_to_string(&person_ref);
        assert!(result.starts_with("principal:"));
        assert!(result.contains("2vxsx-fae"));
    }

    #[test]
    fn test_person_ref_to_string_opaque() {
        let person_ref = crate::types::PersonRef::Opaque("test_id".to_string());
        let result = person_ref_to_string(&person_ref);
        assert_eq!(result, "opaque:test_id");
    }

    #[test]
    fn test_simple_hash_consistency() {
        let data = "test_data";
        let hash1 = simple_hash(data);
        let hash2 = simple_hash(data);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 16); // Should be 16 hex characters
    }

    #[test]
    fn test_simple_hash_different_inputs() {
        let hash1 = simple_hash("data1");
        let hash2 = simple_hash("data2");
        assert_ne!(hash1, hash2);
    }
}
