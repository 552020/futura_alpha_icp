#[cfg(feature = "migration")]
use crate::canister_factory::PersonalCanisterCreationStateData;
use crate::types::{Capsule, ChunkData, MemoryArtifact, UploadSession};
use candid::Principal;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

// ============================================================================
// STABLE MEMORY INFRASTRUCTURE - MVP Implementation
// ============================================================================

// Memory manager for multiple stable memory regions
type Memory = VirtualMemory<DefaultMemoryImpl>;

// Memory IDs for different data types
const CAPSULES_MEMORY_ID: MemoryId = MemoryId::new(0);
const UPLOAD_SESSIONS_MEMORY_ID: MemoryId = MemoryId::new(1);
const MEMORY_ARTIFACTS_MEMORY_ID: MemoryId = MemoryId::new(2);
const CHUNK_DATA_MEMORY_ID: MemoryId = MemoryId::new(3);

// Stable storage structures
thread_local! {
    // Memory manager for stable storage
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    // Stable BTree maps for persistent storage
    static STABLE_CAPSULES: RefCell<StableBTreeMap<String, Capsule, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(CAPSULES_MEMORY_ID))
        )
    );

    static STABLE_UPLOAD_SESSIONS: RefCell<StableBTreeMap<String, UploadSession, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(UPLOAD_SESSIONS_MEMORY_ID))
        )
    );

    static STABLE_MEMORY_ARTIFACTS: RefCell<StableBTreeMap<String, MemoryArtifact, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MEMORY_ARTIFACTS_MEMORY_ID))
        )
    );

    static STABLE_CHUNK_DATA: RefCell<StableBTreeMap<String, ChunkData, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(CHUNK_DATA_MEMORY_ID))
        )
    );
}

// Legacy thread_local storage (kept for backward compatibility during MVP transition)
thread_local! {
    // Admin storage - auto-bootstrap first caller as admin
    static ADMINS: std::cell::RefCell<HashSet<Principal>> = std::cell::RefCell::new(HashSet::new());

    // Capsule storage (centralized data storage) - will be migrated to stable storage
    static CAPSULES: std::cell::RefCell<HashMap<String, Capsule>> = std::cell::RefCell::new(HashMap::new());

    // Nonce proof storage for II authentication
    static NONCE_PROOFS: std::cell::RefCell<HashMap<String, (Principal, u64)>> = std::cell::RefCell::new(HashMap::new());

    // Migration state storage (only available with migration feature)
    #[cfg(feature = "migration")]
    static MIGRATION_STATE: std::cell::RefCell<PersonalCanisterCreationStateData> = std::cell::RefCell::new(PersonalCanisterCreationStateData::default());
}

// ============================================================================
// STABLE MEMORY ACCESS FUNCTIONS
// ============================================================================

// Stable capsule storage functions
pub fn with_stable_capsules<F, R>(f: F) -> R
where
    F: FnOnce(&StableBTreeMap<String, Capsule, Memory>) -> R,
{
    STABLE_CAPSULES.with(|capsules| f(&capsules.borrow()))
}

pub fn with_stable_capsules_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut StableBTreeMap<String, Capsule, Memory>) -> R,
{
    STABLE_CAPSULES.with(|capsules| f(&mut capsules.borrow_mut()))
}

// Stable upload session storage functions
pub fn with_stable_upload_sessions<F, R>(f: F) -> R
where
    F: FnOnce(&StableBTreeMap<String, UploadSession, Memory>) -> R,
{
    STABLE_UPLOAD_SESSIONS.with(|sessions| f(&sessions.borrow()))
}

pub fn with_stable_upload_sessions_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut StableBTreeMap<String, UploadSession, Memory>) -> R,
{
    STABLE_UPLOAD_SESSIONS.with(|sessions| f(&mut sessions.borrow_mut()))
}

// Stable memory artifact storage functions
pub fn with_stable_memory_artifacts<F, R>(f: F) -> R
where
    F: FnOnce(&StableBTreeMap<String, MemoryArtifact, Memory>) -> R,
{
    STABLE_MEMORY_ARTIFACTS.with(|artifacts| f(&artifacts.borrow()))
}

pub fn with_stable_memory_artifacts_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut StableBTreeMap<String, MemoryArtifact, Memory>) -> R,
{
    STABLE_MEMORY_ARTIFACTS.with(|artifacts| f(&mut artifacts.borrow_mut()))
}

// Stable chunk data storage functions
pub fn with_stable_chunk_data<F, R>(f: F) -> R
where
    F: FnOnce(&StableBTreeMap<String, ChunkData, Memory>) -> R,
{
    STABLE_CHUNK_DATA.with(|chunks| f(&chunks.borrow()))
}

pub fn with_stable_chunk_data_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut StableBTreeMap<String, ChunkData, Memory>) -> R,
{
    STABLE_CHUNK_DATA.with(|chunks| f(&mut chunks.borrow_mut()))
}

// ============================================================================
// LEGACY ACCESS FUNCTIONS (for backward compatibility during MVP transition)
// ============================================================================

// Access functions for centralized storage
pub fn with_capsules_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut HashMap<String, Capsule>) -> R,
{
    CAPSULES.with(|capsules| f(&mut capsules.borrow_mut()))
}

pub fn with_capsules<F, R>(f: F) -> R
where
    F: FnOnce(&HashMap<String, Capsule>) -> R,
{
    CAPSULES.with(|capsules| f(&capsules.borrow()))
}

pub fn with_admins_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut HashSet<Principal>) -> R,
{
    ADMINS.with(|admins| f(&mut admins.borrow_mut()))
}

pub fn with_admins<F, R>(f: F) -> R
where
    F: FnOnce(&HashSet<Principal>) -> R,
{
    ADMINS.with(|admins| f(&admins.borrow()))
}

// Nonce proof functions for II authentication
pub fn store_nonce_proof(nonce: String, principal: Principal, timestamp: u64) -> bool {
    NONCE_PROOFS.with(|proofs| {
        proofs.borrow_mut().insert(nonce, (principal, timestamp));
    });
    true
}

pub fn get_nonce_proof(nonce: String) -> Option<Principal> {
    NONCE_PROOFS.with(|proofs| proofs.borrow().get(&nonce).map(|(principal, _)| *principal))
}

// Migration state access functions (only available with migration feature)
#[cfg(feature = "migration")]
pub fn with_migration_state_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut PersonalCanisterCreationStateData) -> R,
{
    MIGRATION_STATE.with(|state| f(&mut state.borrow_mut()))
}

#[cfg(feature = "migration")]
pub fn with_migration_state<F, R>(f: F) -> R
where
    F: FnOnce(&PersonalCanisterCreationStateData) -> R,
{
    MIGRATION_STATE.with(|state| f(&state.borrow()))
}
// ============================================================================
// CANISTER UPGRADE HOOKS - Stable Memory Persistence
// ============================================================================

// Note: pre_upgrade hook is defined in lib.rs to avoid conflicts
// The stable memory data will be automatically persisted by ic-stable-structures

// Note: post_upgrade hook is defined in lib.rs to avoid conflicts
// The stable memory data will be automatically restored by ic-stable-structures

// ============================================================================
// HELPER FUNCTIONS FOR STABLE MEMORY OPERATIONS
// ============================================================================

// Helper to migrate data from thread_local to stable storage (for future use)
#[allow(dead_code)]
pub fn migrate_capsules_to_stable() -> Result<u32, String> {
    let mut migrated_count = 0;

    // Get all capsules from thread_local storage
    let capsules_to_migrate = with_capsules(|capsules| {
        capsules
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>()
    });

    // Insert into stable storage
    with_stable_capsules_mut(|stable_capsules| {
        for (id, capsule) in capsules_to_migrate {
            if stable_capsules.get(&id).is_none() {
                stable_capsules.insert(id, capsule);
                migrated_count += 1;
            }
        }
    });

    Ok(migrated_count)
}

// Helper to check stable memory health
pub fn get_stable_memory_stats() -> (u64, u64, u64) {
    let capsule_count = with_stable_capsules(|capsules| capsules.len());
    let session_count = with_stable_upload_sessions(|sessions| sessions.len());
    let artifact_count = with_stable_memory_artifacts(|artifacts| artifacts.len());

    (capsule_count, session_count, artifact_count)
}
// ============================================================================
// TESTS FOR STABLE MEMORY INFRASTRUCTURE
// ============================================================================

#[cfg(test)]
mod stable_memory_tests {
    use super::*;
    use crate::types::{ArtifactType, MemoryArtifact, MemoryType, PersonRef, UploadSession};
    use candid::Principal;
    use std::collections::HashMap;

    #[test]
    fn test_stable_memory_stats() {
        let (capsules, sessions, artifacts) = get_stable_memory_stats();
        // Just verify the function works - counts start at 0 in test environment
        assert_eq!(capsules, 0);
        assert_eq!(sessions, 0);
        assert_eq!(artifacts, 0);
    }

    #[test]
    fn test_stable_upload_session_operations() {
        let session = UploadSession {
            session_id: "test_session_123".to_string(),
            memory_id: "memory_456".to_string(),
            memory_type: MemoryType::Image,
            expected_hash: "abc123def456".to_string(),
            chunk_count: 5,
            total_size: 1024,
            created_at: 1234567890,
            chunks_received: vec![false; 5],
            bytes_received: 0,
        };

        // Test insert
        with_stable_upload_sessions_mut(|sessions| {
            sessions.insert(session.session_id.clone(), session.clone());
        });

        // Test retrieve
        let retrieved = with_stable_upload_sessions(|sessions| sessions.get(&session.session_id));

        assert!(retrieved.is_some());

        // Test remove
        with_stable_upload_sessions_mut(|sessions| {
            sessions.remove(&session.session_id);
        });

        let after_remove =
            with_stable_upload_sessions(|sessions| sessions.get(&session.session_id));

        assert!(after_remove.is_none());
    }

    #[test]
    fn test_stable_memory_artifact_operations() {
        let artifact = MemoryArtifact {
            memory_id: "memory_789".to_string(),
            memory_type: MemoryType::Image,
            artifact_type: ArtifactType::Metadata,
            content_hash: "hash_xyz".to_string(),
            size: 2048,
            stored_at: 9876543210,
            metadata: Some("{\"test\": true}".to_string()),
        };

        let artifact_key = format!(
            "{}:{}:{}",
            artifact.memory_id,
            format!("{:?}", artifact.memory_type).to_lowercase(),
            format!("{:?}", artifact.artifact_type).to_lowercase()
        );

        // Test insert
        with_stable_memory_artifacts_mut(|artifacts| {
            artifacts.insert(artifact_key.clone(), artifact.clone());
        });

        // Test retrieve
        let retrieved = with_stable_memory_artifacts(|artifacts| artifacts.get(&artifact_key));

        assert!(retrieved.is_some());

        // Test remove
        with_stable_memory_artifacts_mut(|artifacts| {
            artifacts.remove(&artifact_key);
        });

        let after_remove = with_stable_memory_artifacts(|artifacts| artifacts.get(&artifact_key));

        assert!(after_remove.is_none());
    }

    #[test]
    fn test_memory_artifact_key_generation() {
        let artifact = MemoryArtifact {
            memory_id: "test_memory".to_string(),
            memory_type: MemoryType::Video,
            artifact_type: ArtifactType::Asset,
            content_hash: "test_hash".to_string(),
            size: 1024,
            stored_at: 123456,
            metadata: None,
        };

        let key = format!(
            "{}:{}:{}",
            artifact.memory_id,
            format!("{:?}", artifact.memory_type).to_lowercase(),
            format!("{:?}", artifact.artifact_type).to_lowercase()
        );

        assert_eq!(key, "test_memory:video:asset");
    }

    #[test]
    fn test_stable_capsule_basic_operations() {
        // Create a test capsule manually to avoid ic_cdk::api::time() call
        let test_capsule = Capsule {
            id: "test_capsule_123".to_string(),
            subject: PersonRef::Principal(Principal::anonymous()),
            owners: HashMap::new(),
            controllers: HashMap::new(),
            connections: HashMap::new(),
            connection_groups: HashMap::new(),
            memories: HashMap::new(),
            galleries: HashMap::new(),
            created_at: 1234567890,
            updated_at: 1234567890,
            bound_to_web2: false,
        };

        // Test insert
        with_stable_capsules_mut(|capsules| {
            capsules.insert(test_capsule.id.clone(), test_capsule.clone());
        });

        // Test that we can retrieve something
        let retrieved_exists =
            with_stable_capsules(|capsules| capsules.get(&test_capsule.id).is_some());

        assert!(retrieved_exists);

        // Test remove
        with_stable_capsules_mut(|capsules| {
            capsules.remove(&test_capsule.id);
        });

        let after_remove_exists =
            with_stable_capsules(|capsules| capsules.get(&test_capsule.id).is_some());

        assert!(!after_remove_exists);
    }

    #[test]
    fn test_stable_memory_upgrade_persistence() {
        // This test verifies that stable memory structures would persist across upgrades
        // In a real upgrade scenario, the StableBTreeMap data would be automatically preserved

        let session = UploadSession {
            session_id: "upgrade_test_session".to_string(),
            memory_id: "upgrade_test_memory".to_string(),
            memory_type: MemoryType::Document,
            expected_hash: "upgrade_test_hash".to_string(),
            chunk_count: 10,
            total_size: 5120,
            created_at: 1234567890,
            chunks_received: vec![false; 10],
            bytes_received: 0,
        };

        // Store data in stable memory
        with_stable_upload_sessions_mut(|sessions| {
            sessions.insert(session.session_id.clone(), session.clone());
        });

        // Verify data exists
        let exists_before =
            with_stable_upload_sessions(|sessions| sessions.get(&session.session_id).is_some());
        assert!(exists_before);

        // In a real upgrade, the pre_upgrade and post_upgrade hooks would run
        // and the StableBTreeMap would automatically preserve this data
        // This test just verifies the basic operations work

        let retrieved = with_stable_upload_sessions(|sessions| sessions.get(&session.session_id));
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().memory_id, "upgrade_test_memory");
    }

    #[test]
    fn test_stable_memory_concurrent_access() {
        // Test that multiple operations on stable memory work correctly
        let artifact1 = MemoryArtifact {
            memory_id: "concurrent_memory_1".to_string(),
            memory_type: MemoryType::Image,
            artifact_type: ArtifactType::Metadata,
            content_hash: "hash1".to_string(),
            size: 1024,
            stored_at: 1111111111,
            metadata: Some("{\"test\": 1}".to_string()),
        };

        let artifact2 = MemoryArtifact {
            memory_id: "concurrent_memory_2".to_string(),
            memory_type: MemoryType::Video,
            artifact_type: ArtifactType::Asset,
            content_hash: "hash2".to_string(),
            size: 2048,
            stored_at: 2222222222,
            metadata: Some("{\"test\": 2}".to_string()),
        };

        // Store multiple artifacts
        with_stable_memory_artifacts_mut(|artifacts| {
            let key1 = format!(
                "{}:{}:{}",
                artifact1.memory_id,
                format!("{:?}", artifact1.memory_type).to_lowercase(),
                format!("{:?}", artifact1.artifact_type).to_lowercase()
            );
            let key2 = format!(
                "{}:{}:{}",
                artifact2.memory_id,
                format!("{:?}", artifact2.memory_type).to_lowercase(),
                format!("{:?}", artifact2.artifact_type).to_lowercase()
            );

            artifacts.insert(key1, artifact1);
            artifacts.insert(key2, artifact2);
        });

        // Verify both exist
        let (count, artifact1_exists, artifact2_exists) =
            with_stable_memory_artifacts(|artifacts| {
                let key1 = "concurrent_memory_1:image:metadata";
                let key2 = "concurrent_memory_2:video:asset";

                (
                    artifacts.len(),
                    artifacts.get(&key1.to_string()).is_some(),
                    artifacts.get(&key2.to_string()).is_some(),
                )
            });

        assert!(count >= 2);
        assert!(artifact1_exists);
        assert!(artifact2_exists);
    }
}
