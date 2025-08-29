use crate::types;
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

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
