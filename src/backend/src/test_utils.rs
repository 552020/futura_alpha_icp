//! Shared test utilities for creating test data
//!
//! This module provides common test utilities that can be used across all tests
//! to create consistent test capsules, memories, and other test data.

use crate::capsule::domain::{
    AccessCondition, AccessEntry, Capsule, GrantSource, OwnerState, ResourceRole, SharingStatus,
};
use crate::memories::types::{Memory, MemoryMetadata, MemoryType};
use crate::types::PersonRef;
use candid::Principal;
use std::collections::HashMap;

/// Create a test capsule with default values
pub fn create_test_capsule() -> Capsule {
    let now = ic_cdk::api::time();
    let subject = PersonRef::Principal(Principal::anonymous());

    let mut owners = HashMap::new();
    owners.insert(
        subject.clone(),
        OwnerState {
            since: now,
            last_activity_at: now,
        },
    );

    Capsule {
        id: "test_capsule_1".to_string(),
        subject: subject.clone(),
        owners,
        controllers: HashMap::new(),
        connections: HashMap::new(),
        connection_groups: HashMap::new(),
        memories: HashMap::new(),
        galleries: HashMap::new(),
        folders: HashMap::new(),
        created_at: now,
        updated_at: now,
        bound_to_neon: false,
        inline_bytes_used: 0,
        has_advanced_settings: true,
        hosting_preferences: crate::types::HostingPreferences::default(),
    }
}

/// Create a test memory with default values
pub fn create_test_memory() -> Memory {
    let now = ic_cdk::api::time();
    let owner = PersonRef::Principal(Principal::anonymous());

    // Create owner access entry
    let owner_access_entry = AccessEntry {
        id: "test_access_1".to_string(),
        person_ref: Some(owner.clone()),
        is_public: false,
        grant_source: GrantSource::System,
        source_id: None,
        role: ResourceRole::Owner,
        perm_mask: 0b11111, // All permissions
        invited_by_person_ref: None,
        created_at: now,
        updated_at: now,
        condition: AccessCondition::Immediate,
    };

    Memory {
        id: "test_memory_1".to_string(),
        capsule_id: "test_capsule_1".to_string(),
        metadata: MemoryMetadata {
            memory_type: MemoryType::Note,
            title: Some("Test Memory".to_string()),
            description: Some("A test memory for unit tests".to_string()),
            content_type: "text/plain".to_string(),
            created_at: now,
            updated_at: now,
            uploaded_at: now,
            date_of_memory: Some(now),
            file_created_at: Some(now),
            parent_folder_id: None,
            tags: vec!["test".to_string()],
            deleted_at: None,
            people_in_memory: None,
            location: None,
            memory_notes: None,
            created_by: Some(owner.to_string()),
            database_storage_edges: vec![],
            shared_count: 0,
            sharing_status: SharingStatus::Private,
            total_size: 0,
            asset_count: 0,
        },
        inline_assets: vec![],
        blob_internal_assets: vec![],
        blob_external_assets: vec![],
        access_entries: vec![owner_access_entry],
    }
}

/// Create a test memory with public access
pub fn create_test_public_memory() -> Memory {
    let mut memory = create_test_memory();
    memory.id = "test_public_memory_1".to_string();

    // Add public access entry
    let public_access_entry = AccessEntry {
        id: "test_public_access_1".to_string(),
        person_ref: None,
        is_public: true,
        grant_source: GrantSource::User,
        source_id: None,
        role: ResourceRole::Guest,
        perm_mask: 0b00011, // VIEW and DOWNLOAD only
        invited_by_person_ref: Some(PersonRef::Principal(Principal::anonymous())),
        created_at: ic_cdk::api::time(),
        updated_at: ic_cdk::api::time(),
        condition: AccessCondition::Immediate,
    };

    memory.access_entries.push(public_access_entry);
    memory.metadata.sharing_status = SharingStatus::Public;
    memory.metadata.shared_count = 1;

    memory
}

/// Create a test memory with shared access (not public, but shared with specific users)
pub fn create_test_shared_memory() -> Memory {
    let mut memory = create_test_memory();
    memory.id = "test_shared_memory_1".to_string();

    // Add shared access entry
    let shared_access_entry = AccessEntry {
        id: "test_shared_access_1".to_string(),
        person_ref: Some(PersonRef::Principal(Principal::management_canister())),
        is_public: false,
        grant_source: GrantSource::User,
        source_id: None,
        role: ResourceRole::Member,
        perm_mask: 0b00011, // VIEW and DOWNLOAD only
        invited_by_person_ref: Some(PersonRef::Principal(Principal::anonymous())),
        created_at: ic_cdk::api::time(),
        updated_at: ic_cdk::api::time(),
        condition: AccessCondition::Immediate,
    };

    memory.access_entries.push(shared_access_entry);
    memory.metadata.sharing_status = SharingStatus::Shared;
    memory.metadata.shared_count = 1;

    memory
}

/// Create a test capsule with a test memory
pub fn create_test_capsule_with_memory() -> (Capsule, Memory) {
    let mut capsule = create_test_capsule();
    let memory = create_test_memory();

    capsule.memories.insert(memory.id.clone(), memory.clone());

    (capsule, memory)
}

/// Create a test capsule with multiple test memories
pub fn create_test_capsule_with_memories() -> (Capsule, Vec<Memory>) {
    let mut capsule = create_test_capsule();
    let memories = vec![
        create_test_memory(),
        create_test_public_memory(),
        create_test_shared_memory(),
    ];

    for memory in &memories {
        capsule.memories.insert(memory.id.clone(), memory.clone());
    }

    (capsule, memories)
}
