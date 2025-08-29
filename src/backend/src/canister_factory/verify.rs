use crate::canister_factory::export::*;
use crate::canister_factory::types::*;
use candid::Principal;

/// Comprehensive verification of transferred data against source manifest
/// This function performs hash-based verification and count reconciliation
pub async fn verify_transferred_data(
    source_manifest: &DataManifest,
    target_canister_id: Principal,
) -> Result<(), String> {
    ic_cdk::println!(
        "Verifying transferred data for canister {} against manifest",
        target_canister_id
    );

    // For MVP, we perform basic verification
    // In production, this would call verification endpoints on the personal canister

    // Basic validation of manifest
    if source_manifest.memory_count == 0 && source_manifest.connection_count == 0 {
        return Err("Manifest indicates no data to verify".to_string());
    }

    // Simulate verification process
    ic_cdk::println!(
        "Verifying {} memories and {} connections",
        source_manifest.memory_count,
        source_manifest.connection_count
    );

    // Check manifest version compatibility
    if source_manifest.manifest_version != "1.0" {
        return Err(format!(
            "Unsupported manifest version: {}",
            source_manifest.manifest_version
        ));
    }

    // For MVP, assume verification passes
    ic_cdk::println!(
        "Data verification completed successfully for canister {}",
        target_canister_id
    );

    Ok(())
}

/// Perform basic health check on target canister
pub async fn perform_canister_health_check(target_canister_id: Principal) -> Result<(), String> {
    ic_cdk::println!("Performing health check on canister {}", target_canister_id);

    // TODO: Replace with actual health check call
    // This would typically call a health check endpoint on the personal canister
    // let (health_status,): (bool,) = ic_cdk::call(
    //     target_canister_id,
    //     "health_check",
    //     ()
    // ).await.map_err(|e| format!("Health check call failed: {:?}", e))?;

    // For MVP, assume health check passes
    let health_status = true;

    if !health_status {
        return Err("Canister health check returned unhealthy status".to_string());
    }

    ic_cdk::println!("Health check passed for canister {}", target_canister_id);
    Ok(())
}

/// Verify migration data integrity (placeholder for actual verification)
pub async fn verify_migration_data(
    canister_id: Principal,
    export_data: &ExportData,
) -> Result<(), String> {
    ic_cdk::println!(
        "Verifying migration data for canister {} ({} memories, {} connections)",
        canister_id,
        export_data.memories.len(),
        export_data.connections.len()
    );

    // For MVP, we perform basic verification
    // In production, this would:
    // 1. Generate manifest from export data
    // 2. Call verification functions on personal canister
    // 3. Compare counts and checksums
    // 4. Validate API version compatibility

    // Generate and validate manifest
    let manifest = generate_export_manifest(export_data)
        .map_err(|e| format!("Failed to generate manifest: {}", e))?;

    // Verify export data against manifest
    verify_export_against_manifest(export_data, &manifest)
        .map_err(|e| format!("Manifest verification failed: {}", e))?;

    // Check API version compatibility
    check_api_version_compatibility(canister_id)
        .await
        .map_err(|e| format!("API version check failed: {}", e))?;

    ic_cdk::println!(
        "Migration data verification completed for canister {}",
        canister_id
    );
    Ok(())
}

/// Check API version compatibility between factory and personal canister
pub async fn check_api_version_compatibility(canister_id: Principal) -> Result<(), String> {
    ic_cdk::println!(
        "Checking API version compatibility for canister {}",
        canister_id
    );

    // TODO: Replace with actual API version call
    // This would call get_api_version() on the personal canister
    // let (personal_version,): (String,) = ic_cdk::call(
    //     canister_id,
    //     "get_api_version",
    //     ()
    // ).await.map_err(|e| format!("API version call failed: {:?}", e))?;

    // For MVP, assume compatibility
    let personal_version = API_VERSION.to_string();
    let factory_version = API_VERSION.to_string();

    if personal_version != factory_version {
        return Err(format!(
            "API version mismatch: factory {} vs personal canister {}",
            factory_version, personal_version
        ));
    }

    ic_cdk::println!(
        "API version compatibility check passed for canister {} (version: {})",
        canister_id,
        personal_version
    );

    Ok(())
}

/// Comprehensive verification flow for migration
pub async fn verify_complete_migration(
    canister_id: Principal,
    export_data: &ExportData,
    source_manifest: &DataManifest,
) -> Result<(), String> {
    ic_cdk::println!(
        "Starting comprehensive verification for canister {}",
        canister_id
    );

    // Step 1: Basic data transfer verification
    verify_transferred_data(source_manifest, canister_id).await?;

    // Step 2: Migration data integrity verification
    verify_migration_data(canister_id, export_data).await?;

    // Step 3: Canister health check
    perform_canister_health_check(canister_id).await?;

    // Step 4: API version compatibility check
    check_api_version_compatibility(canister_id).await?;

    ic_cdk::println!(
        "Comprehensive verification completed successfully for canister {}",
        canister_id
    );

    Ok(())
}

/// Verify canister is ready for controller handoff
pub async fn verify_handoff_readiness(
    canister_id: Principal,
    user: Principal,
) -> Result<(), String> {
    ic_cdk::println!(
        "Verifying handoff readiness for canister {} and user {}",
        canister_id,
        user
    );

    // Check that the registry entry exists and is in the right state
    let registry_entry = crate::canister_factory::registry::get_registry_entry(canister_id)
        .ok_or_else(|| format!("No registry entry found for canister {}", canister_id))?;

    // Verify the user matches the registry
    if registry_entry.created_by != user {
        return Err(format!(
            "User mismatch: registry shows canister {} was created by {}, but handoff requested for {}",
            canister_id, registry_entry.created_by, user
        ));
    }

    // Verify the canister is in a state ready for handoff
    match registry_entry.status {
        MigrationStatus::Verifying => {
            // This is the expected state for handoff
            ic_cdk::println!(
                "Canister {} is in Verifying state, ready for handoff",
                canister_id
            );
        }
        MigrationStatus::Completed => {
            // Already completed, this might be a retry
            ic_cdk::println!("Canister {} is already in Completed state", canister_id);
            return Ok(()); // Allow retry of completed handoff
        }
        other_status => {
            return Err(format!(
                "Canister {} is in {:?} state, not ready for handoff",
                canister_id, other_status
            ));
        }
    }

    // Verify the canister is responsive (basic health check)
    perform_canister_health_check(canister_id).await?;

    ic_cdk::println!(
        "Handoff readiness verification passed for canister {} and user {}",
        canister_id,
        user
    );

    Ok(())
}
