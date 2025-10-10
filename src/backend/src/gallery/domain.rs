// Gallery Domain Module
// Pure Rust (no ic_cdk dependencies)

use candid::{CandidType, Deserialize};
use serde::Serialize;

// Re-export types from memories module
use crate::capsule::domain::{AccessEntry, SharingStatus};
use crate::memories::types::MemoryType;
use crate::types::BlobHosting;

// Gallery domain types

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct GalleryMetadata {
    pub title: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub shared_count: u32,
    pub sharing_status: SharingStatus,
    pub total_memories: u32,
    pub storage_location: Vec<BlobHosting>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct GalleryItem {
    pub memory_id: String,
    pub memory_type: MemoryType,
    pub position: u32,
    pub caption: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Gallery {
    pub id: String,
    pub capsule_id: String,
    pub metadata: GalleryMetadata,
    pub items: Vec<GalleryItem>,
    pub cover_memory_id: Option<String>,
    pub access_entries: Vec<AccessEntry>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct GalleryHeader {
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

// Gallery implementations

impl crate::capsule::domain::AccessControlled for Gallery {
    fn access_entries(&self) -> &[crate::capsule::domain::AccessEntry] {
        &self.access_entries
    }
}

impl Gallery {
    /// Convert Gallery to GalleryHeader for listing operations
    pub fn to_header(&self) -> GalleryHeader {
        let title = self.metadata.title.clone();
        let name = title.as_ref()
            .map(|t| crate::utils::title_to_name(t))
            .unwrap_or_else(|| "untitled".to_string());

        GalleryHeader {
            id: self.id.clone(),
            title,
            name,                    // ✅ Now uses shared function
            memory_count: self.items.len() as u64,
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
        // All memories in a gallery must be in the same storage location
        // Return the cached storage_location from metadata (trust it's up-to-date)
        self.metadata.storage_location.clone()
    }

    #[allow(dead_code)]
    pub fn add_item(&mut self, memory_id: String, memory_type: MemoryType, position: u32) {
        let item = GalleryItem {
            memory_id: memory_id.clone(),
            memory_type,
            position,
            caption: None,
            metadata: std::collections::HashMap::new(),
        };
        self.items.push(item);
        self.metadata.total_memories += 1;
        // Note: updated_at should be set by the caller using ic_cdk::api::time()
    }

    #[allow(dead_code)]
    pub fn remove_memory(&mut self, memory_id: &str) {
        // If removing the cover memory, clear the cover reference
        if self.cover_memory_id.as_ref() == Some(&memory_id.to_string()) {
            self.cover_memory_id = None;
        }

        self.items.retain(|item| item.memory_id != memory_id);
        self.metadata.total_memories = self.items.len() as u32;
        // Note: updated_at should be set by the caller using ic_cdk::api::time()
    }

    #[allow(dead_code)]
    pub fn set_cover_memory(&mut self, memory_id: &str) -> Result<(), String> {
        // Verify the memory exists in this gallery
        if !self.items.iter().any(|item| item.memory_id == memory_id) {
            return Err("Memory not found in gallery".to_string());
        }

        self.cover_memory_id = Some(memory_id.to_string());
        // Note: updated_at should be set by the caller using ic_cdk::api::time()
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_cover_item(&self) -> Option<&GalleryItem> {
        self.cover_memory_id.as_ref().and_then(|cover_memory_id| {
            self.items
                .iter()
                .find(|item| item.memory_id == *cover_memory_id)
        })
    }
}
