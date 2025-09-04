// use crate::types::HttpRequest; // Disabled for now
// use ic_cdk::api::data_certificate; // Disabled for now
use candid::Principal;

// use ic_http_certification::{utils::add_skip_certification_header, HttpResponse}; // Disabled for now

// Import modules
mod admin;
mod auth;
mod canister_factory;
mod capsule;
pub mod capsule_store;
mod memory;
mod memory_manager;
mod metadata;
pub mod types;
pub mod upload;

// Import CapsuleStore trait for direct access to store methods
use crate::capsule_store::CapsuleStore;
// memories.rs removed - functionality moved to capsule-based architecture

// ============================================================================
// CORE SYSTEM & UTILITY FUNCTIONS (2 functions)
// ============================================================================

#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {name}!")
}

#[ic_cdk::query]
fn whoami() -> Principal {
    ic_cdk::api::msg_caller()
}

// ============================================================================
// AUTHENTICATION & USER MANAGEMENT (6 functions)
// ============================================================================
#[ic_cdk::update]
fn register() -> types::Result<()> {
    capsule::register()
}

// Register user and prove nonce in one call (optimized for II auth flow)
#[ic_cdk::update]
fn register_with_nonce(nonce: String) -> types::Result<()> {
    let caller = ic_cdk::api::msg_caller();
    let timestamp = ic_cdk::api::time();

    // Register the user
    capsule::register()?;

    // Store nonce proof
    memory::store_nonce_proof(nonce, caller, timestamp);

    Ok(())
}

// Nonce proof for II authentication (kept for backward compatibility)
#[ic_cdk::update]
fn prove_nonce(nonce: String) -> types::Result<()> {
    // Store the nonce proof with the caller's principal and timestamp
    let caller = ic_cdk::api::msg_caller();
    let timestamp = ic_cdk::api::time();

    // Store in memory (we'll implement this in memory.rs)
    memory::store_nonce_proof(nonce, caller, timestamp);

    Ok(())
}

#[ic_cdk::query]
fn verify_nonce(nonce: String) -> types::Result<Principal> {
    // Verify and return the principal who proved this nonce
    match memory::get_nonce_proof(nonce) {
        Some(principal) => Ok(principal),
        None => Err(types::Error::NotFound),
    }
}

// ============================================================================
// ADMINISTRATIVE FUNCTIONS (4 functions)
// ============================================================================
#[ic_cdk::update]
fn add_admin(principal: Principal) -> types::Result<()> {
    admin::add_admin(principal)
}

#[ic_cdk::update]
fn remove_admin(principal: Principal) -> types::Result<()> {
    admin::remove_admin(principal)
}

#[ic_cdk::query]
fn list_admins() -> Vec<Principal> {
    admin::list_admins()
}

#[ic_cdk::query]
fn list_superadmins() -> Vec<Principal> {
    admin::list_superadmins()
}

// ============================================================================
// CAPSULE MANAGEMENT (5 functions)
// ============================================================================
#[ic_cdk::update]
fn capsules_bind_neon(
    resource_type: types::ResourceType,
    resource_id: String,
    bind: bool,
) -> types::Result<()> {
    use crate::capsule_store::Order;
    use crate::types::PersonRef;
    use crate::memory::{with_capsule_store, with_capsule_store_mut};
    use ic_cdk::api::time;

    let caller_ref = PersonRef::from_caller();

    match resource_type {
        types::ResourceType::Capsule => {
            // Bind specific capsule if caller owns it
            with_capsule_store_mut(|store| {
                let update_result = store.update(&resource_id, |capsule| {
                    if capsule.owners.contains_key(&caller_ref) {
                        capsule.bound_to_neon = bind;
                        capsule.updated_at = time();

                        // Update owner activity
                        if let Some(owner_state) = capsule.owners.get_mut(&caller_ref) {
                            owner_state.last_activity_at = time();
                        }
                    }
                });
                if update_result.is_ok() {
                    Ok(())
                } else {
                    Err(types::Error::Internal(
                        "Failed to update capsule".to_string(),
                    ))
                }
            })
        }
        types::ResourceType::Gallery => {
            // Bind specific gallery if caller owns the capsule containing it
            with_capsule_store_mut(|store| {
                let all_capsules = store.paginate(None, u32::MAX, Order::Asc);

                for capsule in all_capsules.items {
                    if capsule.owners.contains_key(&caller_ref) {
                        if let Some(gallery) = capsule.galleries.get(&resource_id) {
                            // Update gallery binding
                            let update_result = store.update(&capsule.id, |capsule| {
                                if let Some(gallery) = capsule.galleries.get_mut(&resource_id) {
                                    gallery.bound_to_neon = bind;
                                    capsule.updated_at = time();

                                    // Update owner activity
                                    if let Some(owner_state) = capsule.owners.get_mut(&caller_ref) {
                                        owner_state.last_activity_at = time();
                                    }
                                }
                            });

                            if update_result.is_ok() {
                                return Ok(());
                            }
                        }
                    }
                }
                Err(types::Error::NotFound)
            })
        }
        types::ResourceType::Memory => {
            // Memory binding not implemented yet
            Err(types::Error::Internal(
                "Memory binding not yet implemented".to_string(),
            ))
        }
    }
}

// Capsule management endpoints
#[ic_cdk::update]
fn capsules_create(subject: Option<types::PersonRef>) -> types::CapsuleCreationResult {
    use crate::capsule_store::Order;
    use crate::types::PersonRef;
    use crate::memory::{with_capsule_store, with_capsule_store_mut};
    use crate::types::{Capsule, CapsuleCreationResult};
    use ic_cdk::api::time;

    let caller = PersonRef::from_caller();

    // Check if caller already has a self-capsule when creating self-capsule
    let is_self_capsule = subject.is_none();

    if is_self_capsule {
        // Check if caller already has a self-capsule
        let all_capsules = with_capsule_store(|store| store.paginate(None, u32::MAX, Order::Asc));

        let existing_self_capsule = all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller));

        if let Some(capsule) = existing_self_capsule {
            // Update existing self-capsule activity
            let capsule_id = capsule.id.clone();
            let update_result = with_capsule_store_mut(|store| {
                store.update(&capsule_id, |capsule| {
                    let now = time();
                    capsule.updated_at = now;

                    if let Some(owner_state) = capsule.owners.get_mut(&caller) {
                        owner_state.last_activity_at = now;
                    }
                })
            });

            if update_result.is_ok() {
                return CapsuleCreationResult {
                    success: true,
                    capsule_id: Some(capsule_id),
                    message: "Welcome back! Your capsule is ready.".to_string(),
                };
            }
        }
    }

    // Create new capsule
    let actual_subject = subject.unwrap_or_else(|| caller.clone());
    let capsule = Capsule::new(actual_subject, caller.clone());
    let capsule_id = capsule.id.clone();

    // Use upsert to create new capsule (should succeed since we're checking for existing self-capsule above)
    with_capsule_store_mut(|store| {
        store.upsert(capsule_id.clone(), capsule);
    });

    CapsuleCreationResult {
        success: true,
        capsule_id: Some(capsule_id),
        message: if is_self_capsule {
            "Self-capsule created successfully!".to_string()
        } else {
            "Capsule created".to_string()
        },
    }
}

#[ic_cdk::query]
fn capsules_read_full(capsule_id: Option<String>) -> types::Result<types::Capsule> {
    use crate::capsule_store::Order;
    use crate::types::PersonRef;
    use crate::memory::{with_capsule_store, with_capsule_store_mut};

    let caller = PersonRef::from_caller();

    match capsule_id {
        Some(id) => {
            // Read specific capsule by ID with access check
            with_capsule_store(|store| {
                store
                    .get(&id)
                    .filter(|capsule| capsule.has_write_access(&caller))
                    .ok_or(types::Error::NotFound)
            })
        }
        None => {
            // Read caller's self-capsule
            with_capsule_store(|store| {
                let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
                all_capsules
                    .items
                    .into_iter()
                    .find(|capsule| capsule.subject == caller)
                    .ok_or(types::Error::NotFound)
            })
        }
    }
}

#[ic_cdk::query]
fn capsules_read_basic(capsule_id: Option<String>) -> types::Result<types::CapsuleInfo> {
    use crate::capsule_store::Order;
    use crate::types::PersonRef;
    use crate::memory::{with_capsule_store, with_capsule_store_mut};

    let caller = PersonRef::from_caller();

    match capsule_id {
        Some(id) => {
            // Read specific capsule by ID with access check (basic info)
            with_capsule_store(|store| {
                store
                    .get(&id)
                    .filter(|capsule| capsule.has_write_access(&caller))
                    .map(|capsule| types::CapsuleInfo {
                        capsule_id: capsule.id.clone(),
                        subject: capsule.subject.clone(),
                        is_owner: capsule.owners.contains_key(&caller),
                        is_controller: capsule.controllers.contains_key(&caller),
                        is_self_capsule: capsule.subject == caller,
                        bound_to_neon: capsule.bound_to_neon,
                        created_at: capsule.created_at,
                        updated_at: capsule.updated_at,

                        // Add lightweight counts for summary information
                        memory_count: capsule.memories.len() as u32,
                        gallery_count: capsule.galleries.len() as u32,
                        connection_count: capsule.connections.len() as u32,
                    })
                    .ok_or(types::Error::NotFound)
            })
        }
        None => {
            // Read caller's self-capsule (basic info)
            with_capsule_store(|store| {
                let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
                all_capsules
                    .items
                    .into_iter()
                    .find(|capsule| capsule.subject == caller)
                    .map(|capsule| types::CapsuleInfo {
                        capsule_id: capsule.id.clone(),
                        subject: capsule.subject.clone(),
                        is_owner: capsule.owners.contains_key(&caller),
                        is_controller: capsule.controllers.contains_key(&caller),
                        is_self_capsule: capsule.subject == caller,
                        bound_to_neon: capsule.bound_to_neon,
                        created_at: capsule.created_at,
                        updated_at: capsule.updated_at,

                        // Add lightweight counts for summary information
                        memory_count: capsule.memories.len() as u32,
                        gallery_count: capsule.galleries.len() as u32,
                        connection_count: capsule.connections.len() as u32,
                    })
                    .ok_or(types::Error::NotFound)
            })
        }
    }
}

#[ic_cdk::query]
fn capsules_list() -> Vec<types::CapsuleHeader> {
    use crate::capsule_store::Order;
    use crate::types::PersonRef;
    use crate::memory::{with_capsule_store, with_capsule_store_mut};

    let caller = PersonRef::from_caller();

    // Using new trait-based API with pagination
    // Note: Using large limit to get all capsules, maintaining backward compatibility
    with_capsule_store(|store| {
        let page = store.paginate(None, u32::MAX, Order::Asc);
        page.items
            .into_iter()
            .filter(|capsule| capsule.has_write_access(&caller))
            .map(|capsule| capsule.to_header())
            .collect()
    })
}

// ============================================================================
// GALLERY MANAGEMENT (7 functions)
// ============================================================================
#[ic_cdk::update]
async fn galleries_create(gallery_data: types::GalleryData) -> types::StoreGalleryResponse {
    capsule::galleries_create(gallery_data)
}

#[ic_cdk::update]
async fn galleries_create_with_memories(
    gallery_data: types::GalleryData,
    sync_memories: bool,
) -> types::StoreGalleryResponse {
    capsule::galleries_create_with_memories(gallery_data, sync_memories)
}

#[ic_cdk::update]
fn update_gallery_storage_status(
    gallery_id: String,
    new_status: types::GalleryStorageStatus,
) -> types::Result<()> {
    use crate::capsule_store::Order;
    use crate::types::PersonRef;
    use crate::memory::{with_capsule_store, with_capsule_store_mut};
    use ic_cdk::api::time;

    let caller = PersonRef::from_caller();

    // Update gallery storage status in caller's self-capsule
    with_capsule_store_mut(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);

        // Find the capsule containing the gallery
        if let Some(capsule) = all_capsules.items.into_iter().find(|capsule| {
            capsule.subject == caller
                && capsule.owners.contains_key(&caller)
                && capsule.galleries.contains_key(&gallery_id)
        }) {
            let capsule_id = capsule.id.clone();

            // Update the capsule with the new gallery status
            let update_result = store.update(&capsule_id, |capsule| {
                if let Some(gallery) = capsule.galleries.get_mut(&gallery_id) {
                    gallery.storage_status = new_status;
                    gallery.updated_at = time();
                    capsule.updated_at = time(); // Update capsule timestamp
                }
            });

            if update_result.is_ok() {
                Ok(())
            } else {
                Err(types::Error::Internal(
                    "Failed to update gallery storage status".to_string(),
                ))
            }
        } else {
            Err(types::Error::NotFound)
        }
    })
}

#[ic_cdk::query]
fn galleries_list() -> Vec<types::Gallery> {
    use crate::capsule_store::Order;
    use crate::types::PersonRef;
    use crate::memory::{with_capsule_store, with_capsule_store_mut};

    let caller = PersonRef::from_caller();

    // List all galleries from caller's self-capsule
    with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .filter(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller))
            .flat_map(|capsule| capsule.galleries.values().cloned().collect::<Vec<_>>())
            .collect()
    })
}

#[ic_cdk::query]
fn galleries_read(gallery_id: String) -> types::Result<types::Gallery> {
    use crate::capsule_store::Order;
    use crate::types::PersonRef;
    use crate::memory::{with_capsule_store, with_capsule_store_mut};

    let caller = PersonRef::from_caller();

    // Find gallery in caller's self-capsule
    with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .find(|capsule| capsule.subject == caller && capsule.owners.contains_key(&caller))
            .and_then(|capsule| capsule.galleries.get(&gallery_id).cloned())
            .ok_or(types::Error::NotFound)
    })
}

#[ic_cdk::update]
async fn galleries_update(
    gallery_id: String,
    update_data: types::GalleryUpdateData,
) -> types::UpdateGalleryResponse {
    capsule::galleries_update(gallery_id, update_data)
}

#[ic_cdk::update]
async fn galleries_delete(gallery_id: String) -> types::DeleteGalleryResponse {
    capsule::galleries_delete(gallery_id)
}

// ============================================================================
// MEMORY MANAGEMENT (5 functions)
// ============================================================================
#[ic_cdk::update]
async fn memories_create(
    capsule_id: String,
    memory_data: types::MemoryData,
) -> types::MemoryOperationResponse {
    capsule::memories_create(capsule_id, memory_data)
}

#[ic_cdk::query]
fn memories_read(memory_id: String) -> types::Result<types::Memory> {
    use crate::capsule_store::{Order, PersonRef};

    let caller = PersonRef::from_caller();

    // Find memory across caller's accessible capsules
    with_capsule_store(|store| {
        let all_capsules = store.paginate(None, u32::MAX, Order::Asc);
        all_capsules
            .items
            .into_iter()
            .find(|capsule| {
                // Check if caller has access to this capsule
                capsule.owners.contains_key(&caller) || capsule.subject == caller
            })
            .and_then(|capsule| capsule.memories.get(&memory_id).cloned())
            .ok_or(types::Error::NotFound)
    })
}

#[ic_cdk::update]
async fn memories_update(
    memory_id: String,
    updates: types::MemoryUpdateData,
) -> types::MemoryOperationResponse {
    capsule::memories_update(memory_id, updates)
}

#[ic_cdk::update]
async fn memories_delete(memory_id: String) -> types::MemoryOperationResponse {
    capsule::memories_delete(memory_id)
}

#[ic_cdk::query]
fn memories_list(capsule_id: String) -> types::MemoryListResponse {
    use crate::capsule_store::PersonRef;

    let caller = PersonRef::from_caller();

    // Get memories from specified capsule
    let memories = with_capsule_store(|store| {
        store
            .get(&capsule_id)
            .and_then(|capsule| {
                // Check if caller has access to this capsule
                if capsule.owners.contains_key(&caller) || capsule.subject == caller {
                    Some(capsule.memories.values().cloned().collect::<Vec<_>>())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    });

    types::MemoryListResponse {
        success: true,
        memories,
        message: "Memories retrieved successfully".to_string(),
    }
}

// Note: User principal management is handled through capsule registration
// The existing register() function handles user principal management

// ============================================================================
// PERSONAL CANISTER MANAGEMENT (22 functions)
// ============================================================================
#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::update]
async fn create_personal_canister() -> canister_factory::PersonalCanisterCreationResponse {
    match canister_factory::create_personal_canister().await {
        Ok(response) => response,
        Err(error) => canister_factory::PersonalCanisterCreationResponse {
            success: false,
            canister_id: None,
            message: format!("Personal canister creation failed: {error}"),
        },
    }
}

#[ic_cdk::query]
fn get_creation_status() -> Option<canister_factory::CreationStatusResponse> {
    canister_factory::get_creation_status()
}

#[ic_cdk::query]
fn get_personal_canister_id(user: Principal) -> Option<Principal> {
    canister_factory::get_personal_canister_id(user)
}

#[ic_cdk::query]
fn get_my_personal_canister_id() -> Option<Principal> {
    canister_factory::get_my_personal_canister_id()
}

#[ic_cdk::query]
fn get_detailed_creation_status() -> Option<canister_factory::DetailedCreationStatus> {
    canister_factory::get_detailed_creation_status()
}

// Admin personal canister creation functions
#[ic_cdk::query]
fn get_user_creation_status(
    user: Principal,
) -> types::Result<Option<canister_factory::DetailedCreationStatus>> {
    canister_factory::get_user_creation_status(user)
}

#[ic_cdk::query]
fn get_user_migration_status(
    user: Principal,
) -> types::Result<Option<canister_factory::DetailedCreationStatus>> {
    get_user_creation_status(user)
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn list_all_creation_states(
) -> types::Result<Vec<(Principal, canister_factory::DetailedCreationStatus)>> {
    canister_factory::list_all_creation_states()
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn list_all_migration_states(
) -> types::Result<Vec<(Principal, canister_factory::DetailedCreationStatus)>> {
    list_all_creation_states()
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn get_creation_states_by_status(
    status: canister_factory::CreationStatus,
) -> types::Result<Vec<(Principal, canister_factory::DetailedCreationStatus)>> {
    canister_factory::get_creation_states_by_status(status)
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn get_migration_states_by_status(
    status: canister_factory::CreationStatus,
) -> types::Result<Vec<(Principal, canister_factory::DetailedCreationStatus)>> {
    get_creation_states_by_status(status)
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::update]
fn clear_creation_state(user: Principal) -> types::Result<bool> {
    canister_factory::clear_creation_state(user)
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::update]
fn clear_migration_state(user: Principal) -> types::Result<bool> {
    clear_creation_state(user)
}

// Admin controls for migration (only available with migration feature)
#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::update]
fn set_personal_canister_creation_enabled(enabled: bool) -> types::Result<()> {
    canister_factory::set_personal_canister_creation_enabled(enabled)
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn get_personal_canister_creation_stats(
) -> types::Result<canister_factory::PersonalCanisterCreationStats> {
    canister_factory::get_personal_canister_creation_stats()
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn is_personal_canister_creation_enabled() -> types::Result<bool> {
    canister_factory::is_personal_canister_creation_enabled()
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn is_migration_enabled() -> types::Result<bool> {
    canister_factory::is_migration_enabled()
}

// Legacy function names for backward compatibility
#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::update]
async fn migrate_capsule() -> canister_factory::PersonalCanisterCreationResponse {
    create_personal_canister().await
}

#[cfg(any(feature = "migration", feature = "personal_canister_creation"))]
#[ic_cdk::query]
fn get_migration_status() -> Option<canister_factory::CreationStatusResponse> {
    get_creation_status()
}

#[cfg(feature = "migration")]
#[ic_cdk::query]
fn get_detailed_migration_status() -> Option<canister_factory::DetailedCreationStatus> {
    get_detailed_creation_status()
}

#[cfg(feature = "migration")]
#[ic_cdk::update]
fn set_migration_enabled(enabled: bool) -> types::Result<()> {
    set_personal_canister_creation_enabled(enabled)
}

#[cfg(feature = "migration")]
#[ic_cdk::query]
fn get_migration_stats() -> types::Result<canister_factory::PersonalCanisterCreationStats> {
    get_personal_canister_creation_stats()
}

// pub use memories::*; // Disabled for now

// HTTP request handler for serving memories (disabled for now)
// #[ic_cdk::query]
// fn http_request(request: HttpRequest) -> HttpResponse<'static> {
//     use crate::memory::{create_memory_response, create_not_found_response, load_memory};
//
//     let mut response = match load_memory(&request.url) {
//         Some((data, content_type)) => create_memory_response(data, content_type),
//         None => create_not_found_response(),
//     };
//
//     add_skip_certification_header(data_certificate().unwrap(), &mut response);
//     response
// }

// Persistence hooks for canister upgrades
#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    // Stable memory structures (StableBTreeMap) automatically persist their data
    // No explicit action needed for stable memory - ic-stable-structures handles this

    // For backward compatibility, also serialize thread_local data using the old approach
    let capsule_data = capsule::export_capsules_for_upgrade();
    let admin_data = admin::export_admins_for_upgrade();

    #[cfg(feature = "migration")]
    {
        // Also serialize migration state if migration feature is enabled
        let migration_data = canister_factory::export_migration_state_for_upgrade();
        ic_cdk::storage::stable_save((capsule_data, admin_data, migration_data))
            .expect("Failed to save data to stable storage");
    }

    #[cfg(not(feature = "migration"))]
    {
        // Save without migration data if migration feature is disabled
        ic_cdk::storage::stable_save((capsule_data, admin_data))
            .expect("Failed to save data to stable storage");
    }

    ic_cdk::println!("Pre-upgrade: stable memory structures will persist automatically");
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    // Stable memory structures (StableBTreeMap) automatically restore their data
    // No explicit action needed for stable memory - ic-stable-structures handles this

    // For backward compatibility, restore thread_local data using the old approach
    #[cfg(feature = "migration")]
    {
        // Restore capsules, admins, and migration state after upgrade
        if let Ok((capsule_data, admin_data, migration_data)) = ic_cdk::storage::stable_restore::<(
            Vec<(String, types::Capsule)>,
            Vec<Principal>,
            canister_factory::PersonalCanisterCreationStateData,
        )>() {
            capsule::import_capsules_from_upgrade(capsule_data);
            admin::import_admins_from_upgrade(admin_data);
            canister_factory::import_migration_state_from_upgrade(migration_data);
        }
    }

    #[cfg(not(feature = "migration"))]
    {
        // Restore capsules and admins only if migration feature is disabled
        if let Ok((capsule_data, admin_data)) =
            ic_cdk::storage::stable_restore::<(Vec<(String, types::Capsule)>, Vec<Principal>)>()
        {
            capsule::import_capsules_from_upgrade(capsule_data);
            admin::import_admins_from_upgrade(admin_data);
        }
    }
    // If restore fails, start with empty state (no panic)

    ic_cdk::println!("Post-upgrade: stable memory structures restored automatically");
}

// ============================================================================
// MEMORY METADATA & PRESENCE (2 functions)
// ============================================================================

/// Store memory metadata on ICP with idempotency support
#[ic_cdk::update]
fn upsert_metadata(
    memory_id: String,
    memory_type: types::MemoryType,
    metadata: types::SimpleMemoryMetadata,
    idempotency_key: String,
) -> types::Result<types::MetadataResponse> {
    metadata::upsert_metadata(memory_id, memory_type, metadata, idempotency_key)
}

/// Check presence for multiple memories on ICP (consolidated from get_memory_presence_icp and get_memory_list_presence_icp)
#[ic_cdk::query]
fn memories_ping(memory_ids: Vec<String>) -> types::Result<Vec<types::MemoryPresenceResult>> {
    metadata::memories_ping(memory_ids)
}

// ============================================================================
// EMERGENCY RECOVERY ENDPOINTS (Admin Only)
// ============================================================================

/// Emergency function to clear all stable memory data
/// WARNING: This will delete all stored data and should only be used for recovery
#[ic_cdk::update]
fn clear_all_stable_memory() -> types::Result<()> {
    // Only allow admin to call this
    let caller = ic_cdk::caller();
    if !admin::is_admin(&caller) {
        return Err(types::Error::Unauthorized);
    }

    memory::clear_all_stable_memory().map_err(|e| types::Error::Internal(e))
}

// ============================================================================
// CHUNKED ASSET UPLOAD ENDPOINTS - ICP Canister API
// ============================================================================

// OLD UPLOAD FUNCTIONS REMOVED - Migration to new hybrid architecture complete
// All old upload functions have been removed and replaced with the new workflow:
// - memories_create_inline (≤32KB files)
// - memories_begin_upload + memories_put_chunk + memories_commit (large files)
// - memories_abort (cancel uploads)

// ============================================================================
// FILE UPLOAD & ASSET MANAGEMENT (5 functions)
// ============================================================================

/// Create memory with inline data (≤32KB only)
#[ic_cdk::update]
async fn memories_create_inline(
    capsule_id: types::CapsuleId,
    file_data: Vec<u8>,
    metadata: types::MemoryMeta,
) -> types::Result<types::MemoryId> {
    // Use real UploadService with actual store integration
    memory::with_capsule_store_mut(|store| {
        let mut upload_service = upload::service::UploadService::new(store);
        match upload_service.create_inline(&capsule_id, file_data, metadata) {
            Ok(memory_id) => Ok(memory_id),
            Err(err) => Err(err),
        }
    })
}

/// Begin chunked upload for large files
#[ic_cdk::update]
async fn memories_begin_upload(
    capsule_id: types::CapsuleId,
    metadata: types::MemoryMeta,
    expected_chunks: u32,
) -> types::Result<u64> {
    // Use real UploadService with actual store integration
    memory::with_capsule_store_mut(|store| {
        let mut upload_service = upload::service::UploadService::new(store);
        match upload_service.begin_upload(capsule_id, metadata, expected_chunks) {
            Ok(session_id) => Ok(session_id.0),
            Err(err) => Err(err),
        }
    })
}

/// Upload a chunk for an active session
#[ic_cdk::update]
async fn memories_put_chunk(session_id: u64, chunk_idx: u32, bytes: Vec<u8>) -> types::Result<()> {
    // Use real UploadService with actual store integration
    memory::with_capsule_store_mut(|store| {
        let mut upload_service = upload::service::UploadService::new(store);
        let session_id = upload::types::SessionId(session_id);
        match upload_service.put_chunk(&session_id, chunk_idx, bytes) {
            Ok(()) => Ok(()),
            Err(err) => Err(err),
        }
    })
}

/// Commit chunks to create final memory
#[ic_cdk::update]
async fn memories_commit(
    session_id: u64,
    expected_sha256: Vec<u8>,
    total_len: u64,
) -> types::Result<types::MemoryId> {
    // Use real UploadService with actual store integration
    let hash: [u8; 32] = match expected_sha256.try_into() {
        Ok(h) => h,
        Err(_) => return Err(types::Error::InvalidArgument("invalid hash".to_string())),
    };

    memory::with_capsule_store_mut(|store| {
        let mut upload_service = upload::service::UploadService::new(store);
        let session_id = upload::types::SessionId(session_id);
        match upload_service.commit(session_id, hash, total_len) {
            Ok(memory_id) => Ok(memory_id),
            Err(err) => Err(err),
        }
    })
}

/// Abort upload session and cleanup
#[ic_cdk::update]
async fn memories_abort(session_id: u64) -> types::Result<()> {
    // Use real UploadService with actual store integration
    memory::with_capsule_store_mut(|store| {
        let mut upload_service = upload::service::UploadService::new(store);
        let session_id = upload::types::SessionId(session_id);
        match upload_service.abort(session_id) {
            Ok(()) => Ok(()),
            Err(err) => Err(err),
        }
    })
}

// Export the interface for the smart contract.
ic_cdk::export_candid!();
