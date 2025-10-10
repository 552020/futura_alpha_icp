// Gallery API Types Module
// Request/Response DTOs for gallery API endpoints

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

// Re-export domain types
use crate::gallery::domain::Gallery;
use crate::memories::types::GalleryMemoryEntry;

// ============================================================================
// GALLERY API REQUEST/RESPONSE TYPES
// ============================================================================

/// Gallery creation result - API response DTO
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct GalleryCreationResult {
    pub success: bool,
    pub gallery_id: Option<String>,
    pub message: String,
}

/// Gallery data for storage operations - API request DTO
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct GalleryData {
    pub gallery: Gallery,
    pub owner_principal: Principal,
}

/// Gallery update data - API request DTO
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct GalleryUpdateData {
    pub title: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
    pub memory_entries: Option<Vec<GalleryMemoryEntry>>,
}
