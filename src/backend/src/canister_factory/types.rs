use crate::capsule::domain::Capsule;
use crate::types;
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// API version for compatibility checking between shared backend and personal canisters
pub const API_VERSION: &str = "1.0.0";

/// Response from personal canister creation operations
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct PersonalCanisterCreationResponse {
    pub success: bool,
    pub canister_id: Option<Principal>,
    pub message: String,
}

/// Personal canister creation status enum tracking the progression through creation states
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CreationStatus {
    NotStarted,
    Exporting,
    Creating,
    Installing,
    Importing,
    Verifying,
    Completed,
    Failed,
}

/// Response for personal canister creation status queries
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct CreationStatusResponse {
    pub status: CreationStatus,
    pub canister_id: Option<Principal>,
    pub message: Option<String>,
}

/// Detailed personal canister creation status with progress information
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct DetailedCreationStatus {
    pub status: CreationStatus,
    pub canister_id: Option<Principal>,
    pub created_at: u64,
    pub completed_at: Option<u64>,
    pub cycles_consumed: u128,
    pub error_message: Option<String>,
    pub progress_message: String,
}

/// Exported capsule data for migration
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ExportData {
    pub capsule: Capsule,
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

/// Personal canister creation state for tracking individual user creations
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct PersonalCanisterCreationState {
    pub user: Principal,
    pub status: CreationStatus,
    pub created_at: u64,
    pub completed_at: Option<u64>,
    pub personal_canister_id: Option<Principal>,
    pub cycles_consumed: u128,
    pub error_message: Option<String>,
}

/// Configuration for personal canister creation system
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct PersonalCanisterCreationConfig {
    pub enabled: bool,
    pub cycles_reserve: u128,
    pub min_cycles_threshold: u128,
    pub admin_principals: BTreeSet<Principal>,
}

impl Default for PersonalCanisterCreationConfig {
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
    pub status: CreationStatus,
    pub cycles_consumed: u128,
}

/// Statistics for personal canister creation operations
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Default)]
pub struct PersonalCanisterCreationStats {
    pub total_attempts: u64,
    pub total_successes: u64,
    pub total_failures: u64,
    pub total_cycles_consumed: u128,
}

/// Minimal configuration for creating personal canisters
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Default)]
pub struct CreatePersonalCanisterConfig {
    pub name: Option<String>,
    pub subnet_id: Option<Principal>,
}

/// Configuration for import operations
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ImportConfig {
    pub max_chunk_size: u64,
    pub max_total_import_size: u64,
    pub session_timeout_seconds: u64,
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self {
            max_chunk_size: 2_000_000,          // 2MB max chunk size
            max_total_import_size: 100_000_000, // 100MB max total import size
            session_timeout_seconds: 3600,      // 1 hour session timeout
        }
    }
}

/// Import session state for tracking chunked data transfers
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ImportSession {
    pub session_id: String,
    pub user: Principal,
    pub created_at: u64,
    pub last_activity_at: u64,
    pub total_expected_size: u64,
    pub total_received_size: u64,
    pub memories_in_progress: HashMap<String, MemoryImportState>,
    pub completed_memories: HashMap<String, types::Memory>,
    pub import_manifest: Option<DataManifest>,
    pub status: ImportSessionStatus,
}

/// Status of an import session
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ImportSessionStatus {
    Active,
    Finalizing,
    Completed,
    Failed,
    Expired,
}

/// State of a memory being imported in chunks
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct MemoryImportState {
    pub memory_id: String,
    pub expected_chunks: u32,
    pub received_chunks: HashMap<u32, ChunkData>,
    pub total_size: u64,
    pub received_size: u64,
    pub memory_metadata: Option<types::Memory>,
    pub is_complete: bool,
}

/// Individual chunk data
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ChunkData {
    pub chunk_index: u32,
    pub data: Vec<u8>,
    pub sha256: String,
    pub received_at: u64,
}

/// Memory manifest for chunk assembly verification
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct MemoryManifest {
    pub memory_id: String,
    pub total_chunks: u32,
    pub total_size: u64,
    pub chunk_checksums: Vec<String>,
    pub final_checksum: String,
}

/// Import session response
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ImportSessionResponse {
    pub success: bool,
    pub session_id: Option<String>,
    pub message: String,
}

/// Chunk upload response
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ChunkUploadResponse {
    pub success: bool,
    pub message: String,
    pub received_size: u64,
    pub total_expected_size: u64,
}

/// Memory commit response
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct MemoryCommitResponse {
    pub success: bool,
    pub message: String,
    pub memory_id: String,
    pub assembled_size: u64,
}

/// Import finalization response
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ImportFinalizationResponse {
    pub success: bool,
    pub message: String,
    pub total_memories_imported: u32,
    pub total_size_imported: u64,
}

/// Cycles reserve status including threshold information (admin function)
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct CyclesReserveStatus {
    pub current_reserve: u128,
    pub min_threshold: u128,
    pub is_above_threshold: bool,
    pub total_consumed: u128,
}

/// Alert levels for cycles reserve monitoring
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum CyclesAlertLevel {
    Normal,   // Above threshold
    Warning,  // Below threshold but above critical
    Critical, // Very low reserves
}

/// Comprehensive cycles monitoring report (admin function)
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct CyclesMonitoringReport {
    pub reserve_status: CyclesReserveStatus,
    pub alert_level: CyclesAlertLevel,
    pub recent_consumption_rate: Option<u128>, // Could be calculated from recent operations
    pub recommendations: Vec<String>,
}

/// Extended state structure with personal canister creation fields
#[derive(CandidType, Serialize, Deserialize, Default, Clone, Debug)]
pub struct PersonalCanisterCreationStateData {
    pub creation_config: PersonalCanisterCreationConfig,
    pub creation_states: BTreeMap<Principal, PersonalCanisterCreationState>,
    pub creation_stats: PersonalCanisterCreationStats,
    pub personal_canisters: BTreeMap<Principal, PersonalCanisterRecord>,
    pub import_config: ImportConfig,
    pub import_sessions: HashMap<String, ImportSession>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_types_simple() {
        let config = PersonalCanisterCreationConfig::default();
        assert!(!config.enabled);
    }
}
