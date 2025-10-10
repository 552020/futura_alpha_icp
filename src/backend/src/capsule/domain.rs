use ic_cdk::api::time;
use std::collections::HashMap;

use crate::types::*; // Memory, MemoryAccess, CapsuleHeader, HostingPreferences, Error, etc.
use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

// ============================================================================
// PERMISSION SYSTEM - Bitflags for Access Control
// ============================================================================

use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Perm: u32 {
        const VIEW = 1 << 0;
        const DOWNLOAD = 1 << 1;
        const SHARE = 1 << 2;
        const MANAGE = 1 << 3;
        const OWN = 1 << 4;
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct RoleTemplate {
    pub name: String,
    pub perm_mask: u32,
    pub description: String,
}

impl Default for RoleTemplate {
    fn default() -> Self {
        Self {
            name: "member".to_string(),
            perm_mask: (Perm::VIEW | Perm::DOWNLOAD).bits(),
            description: "Standard member access".to_string(),
        }
    }
}

#[allow(dead_code)]
pub fn get_default_role_templates() -> Vec<RoleTemplate> {
    vec![
        RoleTemplate {
            name: "owner".to_string(),
            perm_mask: (Perm::VIEW | Perm::DOWNLOAD | Perm::SHARE | Perm::MANAGE | Perm::OWN)
                .bits(),
            description: "Full ownership access".to_string(),
        },
        RoleTemplate {
            name: "admin".to_string(),
            perm_mask: (Perm::VIEW | Perm::DOWNLOAD | Perm::SHARE | Perm::MANAGE).bits(),
            description: "Administrative access".to_string(),
        },
        RoleTemplate {
            name: "member".to_string(),
            perm_mask: (Perm::VIEW | Perm::DOWNLOAD).bits(),
            description: "Standard member access".to_string(),
        },
        RoleTemplate {
            name: "guest".to_string(),
            perm_mask: Perm::VIEW.bits(),
            description: "Read-only access".to_string(),
        },
    ]
}

// ============================================================================
// UNIVERSAL ACCESS SYSTEM TYPES
// ============================================================================

// Decentralized access control system - access lives on each resource

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum AccessCondition {
    Immediate,                             // Access now
    Scheduled { accessible_after: u64 },   // Access after timestamp
    ExpiresAt { expires: u64 },            // Access until timestamp
    EventTriggered { event: AccessEvent }, // Access after event
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct AccessEntry {
    pub id: String,
    pub person_ref: Option<PersonRef>, // None for public access
    pub is_public: bool,               // Explicit public access flag
    pub grant_source: GrantSource,     // Who granted it (User/Group/MagicLink/System)
    pub source_id: Option<String>,
    pub role: ResourceRole,
    pub perm_mask: u32,
    pub invited_by_person_ref: Option<PersonRef>,
    pub created_at: u64,
    pub updated_at: u64,

    // ✅ NEW: Time/event-based access
    pub condition: AccessCondition,
}

// ❌ REMOVED: PublicPolicy struct - now unified in AccessEntry

#[derive(
    Clone, Debug, CandidType, Deserialize, Serialize, PartialEq, Eq, Hash, Ord, PartialOrd,
)]
pub enum ResourceType {
    Memory,
    Gallery,
    Folder,
    Capsule,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum GrantSource {
    User,      // A user granted this access
    Group,     // A group granted this access
    MagicLink, // A magic link granted this access
    System,    // The system granted this access
               // ❌ REMOVED: PublicMode, // Public is not a grant source
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum ResourceRole {
    Owner,
    SuperAdmin,
    Admin,
    Member,
    Guest,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum SharingStatus {
    Public,
    Shared,
    Private,
}

// ❌ REMOVED: PublicMode enum - now handled by is_public flag in AccessEntry

/// Events that can trigger access changes
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub enum AccessEvent {
    // Memorial events
    AfterDeath,       // revealed after subject's death is recorded
    Anniversary(u32), // revealed on specific anniversary (Nth year)

    // Life events
    Birthday(u32), // revealed on Nth birthday
    Graduation,    // revealed after graduation
    Wedding,       // revealed after wedding

    // Capsule events
    CapsuleMaturity(u32), // revealed when capsule reaches N years old
    ConnectionCount(u32), // revealed when capsule has N connections

    // Custom events
    Custom(String), // custom event identifier
}

// ============================================================================
// CORE DOMAIN TYPES - Business Logic Types
// ============================================================================

/// Core person reference - can be a live principal or opaque identifier
#[derive(
    CandidType, Deserialize, Serialize, Clone, Eq, PartialEq, Hash, Debug, PartialOrd, Ord,
)]
pub enum PersonRef {
    Principal(Principal), // live II user
    Opaque(String),       // non-principal subject (e.g., deceased), UUID-like
}

impl PersonRef {
    /// Extract the Principal if this is a Principal variant, None otherwise
    pub fn principal(&self) -> Option<&Principal> {
        match self {
            PersonRef::Principal(p) => Some(p),
            PersonRef::Opaque(_) => None,
        }
    }
}

impl std::fmt::Display for PersonRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersonRef::Principal(p) => write!(f, "{}", p),
            PersonRef::Opaque(s) => write!(f, "{}", s),
        }
    }
}

/// Connection status for peer relationships
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum ConnectionStatus {
    Pending,
    Accepted,
    Blocked,
    Revoked,
}

/// Connection between persons
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Connection {
    pub peer: PersonRef,
    pub status: ConnectionStatus,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Connection groups for organizing relationships
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq)]
pub struct ConnectionGroup {
    pub id: String,
    pub name: String, // "Family", "Close Friends", etc.
    pub description: Option<String>,
    pub members: Vec<PersonRef>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Controller state tracking (simplified - full control except ownership transfer)
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ControllerState {
    pub granted_at: u64,
    pub granted_by: PersonRef,
}

/// Owner state tracking
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct OwnerState {
    pub since: u64,
    pub last_activity_at: u64, // Track owner activity
}

// ============================================================================
// CAPSULE DOMAIN MODEL
// ============================================================================

/// Core capsule data structure with business logic
/// This struct contains all capsule data and its associated business rules
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Capsule {
    pub id: String,                                          // unique capsule identifier
    pub subject: PersonRef,                                  // who this capsule is about
    pub owners: HashMap<PersonRef, OwnerState>,              // 1..n owners (usually 1)
    pub controllers: HashMap<PersonRef, ControllerState>,    // delegated admins (full control)
    pub connections: HashMap<PersonRef, Connection>,         // social graph
    pub connection_groups: HashMap<String, ConnectionGroup>, // organized connection groups
    pub memories: HashMap<String, Memory>,                   // content
    pub galleries: HashMap<String, Gallery>,                 // galleries (collections of memories)
    pub folders: HashMap<String, Folder>,                    // folders (collections of memories)
    pub created_at: u64,
    pub updated_at: u64,
    pub bound_to_neon: bool,         // Neon database binding status
    pub inline_bytes_used: u64,      // Track inline storage consumption
    pub has_advanced_settings: bool, // Controls whether user sees advanced settings panels
    pub hosting_preferences: HostingPreferences, // User's preferred hosting providers
}

impl Capsule {
    pub fn new(subject: PersonRef, initial_owner: PersonRef) -> Self {
        let now = time();
        let capsule_id = format!("capsule_{now}");

        let mut owners = HashMap::new();
        owners.insert(
            initial_owner,
            OwnerState {
                since: now,
                last_activity_at: now,
            },
        );

        Capsule {
            id: capsule_id,
            subject,
            owners,
            controllers: HashMap::new(),
            connections: HashMap::new(),
            connection_groups: HashMap::new(),
            memories: HashMap::new(),
            galleries: HashMap::new(),
            folders: HashMap::new(),
            created_at: now,
            updated_at: now,
            bound_to_neon: false,        // Initially not bound to Neon
            inline_bytes_used: 0,        // Start with zero inline consumption
            has_advanced_settings: true, // Default to advanced settings for Web3 users
            hosting_preferences: HostingPreferences::default(), // Default to ICP hosting
        }
    }

    /// Check if a PersonRef is an owner
    pub fn is_owner(&self, person: &PersonRef) -> bool {
        self.owners.contains_key(person)
    }

    /// Check if a PersonRef is a controller
    pub fn is_controller(&self, person: &PersonRef) -> bool {
        self.controllers.contains_key(person)
    }

    /// Check if a PersonRef has write access (owner or controller)
    pub fn has_write_access(&self, person: &PersonRef) -> bool {
        self.is_owner(person) || self.is_controller(person)
    }

    /// Check if a PersonRef has read access to this capsule
    ///
    /// TODO: Implement proper read access logic based on capsule access model.
    /// Currently returns true for owners/controllers (same as write access).
    /// Future implementation should consider:
    /// - Connection-based read access
    /// - Group-based read access  
    /// - Public capsule access
    /// - Time-based access rules
    pub fn has_read_access(&self, person: &PersonRef) -> bool {
        // TODO: Implement proper read access logic
        // For now, use same logic as write access
        self.has_write_access(person)
    }

    /// Check if a PersonRef can read a specific memory
    /// TODO: Replace with new access control system
    #[allow(dead_code)]
    pub fn can_read_memory(&self, _person: &PersonRef, _memory: &Memory) -> bool {
        // TODO: Use new unified AccessEntry system
        true // Temporary - always allow for greenfield
    }

    /// Update the last activity timestamp for a person
    #[allow(dead_code)]
    pub fn touch_activity(&mut self, person: &PersonRef) {
        let now = time();
        self.updated_at = now;

        // Update owner activity if they are an owner
        if let Some(owner_state) = self.owners.get_mut(person) {
            owner_state.last_activity_at = now;
        }

        // Note: ControllerState doesn't have last_activity_at field
        // Controllers don't track activity separately
    }

    /// Update the capsule's updated_at timestamp
    #[allow(dead_code)]
    pub fn touch(&mut self) {
        self.updated_at = time();
    }

    /// Convert capsule to header for listing
    pub fn to_header(&self) -> CapsuleHeader {
        CapsuleHeader {
            id: self.id.clone(),
            subject: self.subject.clone(),
            owner_count: self.owners.len() as u64,
            controller_count: self.controllers.len() as u64,
            memory_count: self.memories.len() as u64,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

// ============================================================================
// ACCESS CONTROL TRAIT AND EVALUATION LOGIC
// ============================================================================

/// Trait for resources that have access control
pub trait AccessControlled {
    #[allow(dead_code)]
    fn access_entries(&self) -> &[AccessEntry];
    // ❌ REMOVED: fn public_policy(&self) -> Option<&PublicPolicy>; // Now unified in AccessEntry
}

/// Context for permission evaluation
#[derive(Clone, Debug, PartialEq)]
pub struct PrincipalContext {
    pub principal: Principal,
    pub groups: Vec<String>,
    pub link: Option<String>,
    pub now_ns: u64,
}

impl PrincipalContext {
    #[allow(dead_code)]
    pub fn new(principal: Principal, groups: Vec<String>, link: Option<String>) -> Self {
        Self {
            principal,
            groups,
            link,
            now_ns: time(),
        }
    }
}

/// Calculate effective permission mask for a resource
#[allow(dead_code)]
pub fn effective_perm_mask<T: AccessControlled>(resource: &T, ctx: &PrincipalContext) -> u32 {
    use Perm as P;

    // 1) Ownership short-circuit (implement is_owner in domain, pure)
    if is_owner(resource, ctx) {
        return (P::VIEW | P::DOWNLOAD | P::SHARE | P::MANAGE | P::OWN).bits();
    }

    let mut m = 0u32;

    // 2) Check all access entries (unified individual and public access)
    for entry in resource.access_entries() {
        // Check if access is currently active (time/event conditions)
        if !is_access_active(&entry.condition, ctx.now_ns) {
            continue;
        }

        // Check if this entry applies to the current principal
        if entry.is_public {
            // Public access - check if principal can access public content
            if entry.grant_source == GrantSource::User || entry.grant_source == GrantSource::System
            {
                m |= entry.perm_mask;
            }
        } else if let Some(person_ref) = &entry.person_ref {
            // Individual access - check if principal matches
            if person_ref == &PersonRef::Principal(ctx.principal) {
                m |= entry.perm_mask;
            }
            // TODO: Add group membership checks
        }
    }

    // 3) Magic link access (if provided)
    if let Some(_token) = &ctx.link {
        // TODO: Implement magic link validation
        // defer to repo (magic links) only if that feature exists; otherwise 0
        m |= 0;
    }

    m
}

/// Check if an access condition is currently active
#[allow(dead_code)]
fn is_access_active(condition: &AccessCondition, now_ns: u64) -> bool {
    match condition {
        AccessCondition::Immediate => true,
        AccessCondition::Scheduled { accessible_after } => now_ns >= *accessible_after,
        AccessCondition::ExpiresAt { expires } => now_ns <= *expires,
        AccessCondition::EventTriggered { event: _ } => {
            // TODO: Implement event checking
            // For now, treat as inactive until event system is implemented
            false
        }
    }
}

/// Check if a resource has a specific permission
#[allow(dead_code)]
pub fn has_perm<T: AccessControlled>(res: &T, ctx: &PrincipalContext, want: Perm) -> bool {
    (effective_perm_mask(res, ctx) & want.bits()) != 0
}

/// Check if the principal is the owner of the resource
#[allow(dead_code)]
fn is_owner<T: AccessControlled>(_res: &T, _ctx: &PrincipalContext) -> bool {
    // TODO: Implement ownership logic based on resource type
    // For now, return false - this will be implemented when we add the trait to Memory/Gallery
    false
}

/// Sum permissions from user and group access entries
#[allow(dead_code)]
fn sum_user_and_groups(entries: &[AccessEntry], ctx: &PrincipalContext) -> u32 {
    let mut mask = 0;
    for e in entries {
        if e.person_ref == Some(PersonRef::Principal(ctx.principal)) {
            mask |= e.perm_mask;
        }
        // TODO: Add group membership checks
    }
    mask
}
