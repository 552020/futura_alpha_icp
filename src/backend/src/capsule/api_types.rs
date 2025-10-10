use candid::{CandidType, Deserialize};
use serde::Serialize;

use crate::capsule::domain::PersonRef;
use crate::types::HostingPreferences;

// ============================================================================
// CAPSULE API TYPES - Request/Response DTOs
// ============================================================================

/// Capsule information for user queries (API response)
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct CapsuleInfo {
    pub capsule_id: String,
    pub subject: PersonRef,
    pub is_owner: bool,
    pub is_controller: bool,
    pub is_self_capsule: bool, // true if subject == caller
    pub bound_to_neon: bool,
    pub created_at: u64,
    pub updated_at: u64,

    // Lightweight counts for summary information
    pub memory_count: u64,     // Number of memories in this capsule
    pub gallery_count: u64,    // Number of galleries in this capsule
    pub connection_count: u64, // Number of connections to other people
}

/// Capsule header for listing (API response)
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct CapsuleHeader {
    pub id: String,
    pub subject: PersonRef,
    pub owner_count: u64,
    pub controller_count: u64,
    pub memory_count: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Capsule update data for partial updates (API request)
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct CapsuleUpdateData {
    pub bound_to_neon: Option<bool>, // Update binding status
                                     // Note: Most capsule fields (id, subject, owners, etc.) are immutable
                                     // Only binding status and timestamps can be updated
}

/// User settings data for updating capsule settings (API request)
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct UserSettingsUpdateData {
    pub has_advanced_settings: Option<bool>,
}

/// User settings response for reading capsule settings (API response)
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct UserSettingsResponse {
    pub has_advanced_settings: bool,
    pub hosting_preferences: HostingPreferences,
}
