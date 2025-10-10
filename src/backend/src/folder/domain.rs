// Folder Domain Module
// Pure Rust (no ic_cdk dependencies)

use candid::{CandidType, Deserialize};
use serde::Serialize;

// Re-export types from other modules
use crate::capsule::domain::{AccessEntry, SharingStatus};
use crate::types::BlobHosting;

// Folder domain types

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct FolderMetadata {
    pub title: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub shared_count: u32,
    pub sharing_status: SharingStatus,
    pub total_memories: u32,
    pub storage_location: Vec<BlobHosting>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Folder {
    pub id: String,
    pub capsule_id: String,
    pub metadata: FolderMetadata,
    pub access_entries: Vec<AccessEntry>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct FolderHeader {
    pub id: String,
    pub title: Option<String>,
    pub name: String,
    pub memory_count: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub shared_count: u32,
    pub sharing_status: SharingStatus,
    pub total_memories: u32,
    pub storage_location: Vec<BlobHosting>,
}

// Folder implementations

impl crate::capsule::domain::AccessControlled for Folder {
    fn access_entries(&self) -> &[crate::capsule::domain::AccessEntry] {
        &self.access_entries
    }
}

impl Folder {
    /// Convert Folder to FolderHeader for listing operations
    pub fn to_header(&self) -> FolderHeader {
        FolderHeader {
            id: self.id.clone(),
            title: self.metadata.title.clone(),
            name: self.metadata.name.clone(),
            memory_count: self.metadata.total_memories as u64,
            created_at: self.created_at,
            updated_at: self.updated_at,
            shared_count: self.metadata.shared_count,
            sharing_status: self.metadata.sharing_status.clone(),
            total_memories: self.metadata.total_memories,
            storage_location: self.metadata.storage_location.clone(),
        }
    }

    #[allow(dead_code)]
    pub fn compute_storage_location(&self) -> Vec<BlobHosting> {
        // Return the cached storage_location from metadata (trust it's up-to-date)
        self.metadata.storage_location.clone()
    }
}
