// Folder API Types
// Request/Response DTOs for folder API endpoints

use candid::{CandidType, Deserialize};
use serde::Serialize;

use crate::folder::domain::Folder;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct FolderData {
    pub folder: Folder,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct FolderUpdateData {
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct FolderCreationResult {
    pub folder_id: String,
    pub success: bool,
}

