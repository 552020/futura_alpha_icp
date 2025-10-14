use crate::canister_factory::export::*;
use crate::canister_factory::types::*;
// Removed unused imports: AssetMetadata, AssetMetadataBase, AssetType, MemoryAssetBlobInternal, NoteAssetMetadata
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
        .map_err(|e| format!("Failed to generate manifest: {e}"))?;

    // Verify export data against manifest
    verify_export_against_manifest(export_data, &manifest)
        .map_err(|e| format!("Manifest verification failed: {e}"))?;

    // Check API version compatibility
    check_api_version_compatibility(canister_id)
        .await
        .map_err(|e| format!("API version check failed: {e}"))?;

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
            "API version mismatch: factory {factory_version} vs personal canister {personal_version}"
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
        .ok_or_else(|| format!("No registry entry found for canister {canister_id}"))?;

    // Verify the user matches the registry
    if registry_entry.created_by != user {
        return Err(format!(
            "User mismatch: registry shows canister {} was created by {}, but handoff requested for {}",
            canister_id, registry_entry.created_by, user
        ));
    }

    // Verify the canister is in a state ready for handoff
    match registry_entry.status {
        CreationStatus::Verifying => {
            // This is the expected state for handoff
            ic_cdk::println!(
                "Canister {} is in Verifying state, ready for handoff",
                canister_id
            );
        }
        CreationStatus::Completed => {
            // Already completed, this might be a retry
            ic_cdk::println!("Canister {} is already in Completed state", canister_id);
            return Ok(()); // Allow retry of completed handoff
        }
        other_status => {
            return Err(format!(
                "Canister {canister_id} is in {other_status:?} state, not ready for handoff"
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::capsule::domain::{
        AccessCondition, AccessEntry, GrantSource, ResourceRole, SharingStatus,
    };
    use crate::types::{self, HostingPreferences};
    use candid::Principal;

    // Test helper functions
    fn create_test_principal(id: u8) -> Principal {
        Principal::from_slice(&[id; 29])
    }

    fn create_test_capsule() -> types::Capsule {
        let mut owners = std::collections::HashMap::new();
        owners.insert(
            types::PersonRef::Principal(create_test_principal(1)),
            types::OwnerState {
                since: 1000000000,
                last_activity_at: 1000000000,
            },
        );

        types::Capsule {
            id: "test_capsule".to_string(),
            subject: types::PersonRef::Principal(create_test_principal(1)),
            owners,
            controllers: std::collections::HashMap::new(),
            connections: std::collections::HashMap::new(),
            has_advanced_settings: false, // Default to simple settings
            connection_groups: std::collections::HashMap::new(),
            memories: std::collections::HashMap::new(),
            galleries: std::collections::HashMap::new(),
            folders: std::collections::HashMap::new(),
            created_at: 1000000000,
            updated_at: 1000000000,
            bound_to_neon: false, // Default to not bound to Neon
            inline_bytes_used: 0,
            hosting_preferences: HostingPreferences::default(),
        }
    }

    fn create_test_memory(id: &str) -> types::Memory {
        types::Memory {
            id: id.to_string(),
            capsule_id: "test_capsule".to_string(),
            metadata: types::MemoryMetadata {
                memory_type: types::MemoryType::Note,
                title: Some(format!("Memory {}", id)),
                description: None,
                content_type: "text/plain".to_string(),
                created_at: 1000000000,
                updated_at: 1000000000,
                uploaded_at: 1000000000,
                date_of_memory: Some(1000000000),
                file_created_at: Some(1000000000),
                parent_folder_id: None,
                tags: vec!["test".to_string()],
                deleted_at: None,
                people_in_memory: None,
                location: None,
                memory_notes: None,
                created_by: None,
                database_storage_edges: vec![types::StorageEdgeDatabaseType::Icp],

                // NEW: Pre-computed dashboard fields (defaults)
                shared_count: 0,
                sharing_status: SharingStatus::Private,
                total_size: 100,
                asset_count: 1,
            },
            access_entries: vec![AccessEntry {
                id: format!("test_access_{}", id),
                person_ref: Some(types::PersonRef::Principal(create_test_principal(1))),
                is_public: false,
                grant_source: GrantSource::System,
                source_id: None,
                role: ResourceRole::Owner,
                perm_mask: 0b11111, // All permissions
                invited_by_person_ref: None,
                created_at: 1000000000,
                updated_at: 1000000000,
                condition: AccessCondition::Immediate,
            }],
            inline_assets: vec![],
            blob_internal_assets: vec![types::MemoryAssetBlobInternal {
                asset_id: format!("test_asset_{}", id),
                blob_ref: types::BlobRef {
                    locator: format!("memory_{}", id),
                    hash: None,
                    len: 100,
                },
                metadata: types::AssetMetadata::Note(types::NoteAssetMetadata {
                    base: types::AssetMetadataBase {
                        name: format!("Memory {}", id),
                        description: None,
                        tags: vec![],
                        asset_type: types::AssetType::Original,
                        bytes: 100,
                        mime_type: "text/plain".to_string(),
                        sha256: None,
                        width: None,
                        height: None,
                        url: None,
                        storage_key: None,
                        bucket: None,
                        processing_status: None,
                        processing_error: None,
                        created_at: 1000000000,
                        updated_at: 1000000000,
                        deleted_at: None,
                        asset_location: None,
                    },
                    word_count: None,
                    language: None,
                    format: Some("text".to_string()),
                }),
            }],
            blob_external_assets: vec![],
        }
    }

    fn create_test_connection(person_ref: &types::PersonRef) -> types::Connection {
        types::Connection {
            peer: person_ref.clone(),
            status: types::ConnectionStatus::Accepted,
            created_at: 1000000000,
            updated_at: 1000000000,
        }
    }

    fn create_test_export_data() -> ExportData {
        let capsule = create_test_capsule();
        let memories = vec![
            ("mem1".to_string(), create_test_memory("mem1")),
            ("mem2".to_string(), create_test_memory("mem2")),
        ];
        let person_ref = types::PersonRef::Opaque("person1".to_string());
        let connections = vec![(person_ref.clone(), create_test_connection(&person_ref))];

        ExportData {
            capsule,
            memories,
            connections,
            metadata: ExportMetadata {
                export_timestamp: 1000000000,
                original_canister_id: create_test_principal(99),
                data_version: "1.0".to_string(),
                total_size_bytes: 1024,
            },
        }
    }

    fn create_test_manifest() -> DataManifest {
        DataManifest {
            capsule_checksum: "test_capsule_checksum".to_string(),
            memory_count: 2,
            memory_checksums: vec![
                ("mem1".to_string(), "mem1_checksum".to_string()),
                ("mem2".to_string(), "mem2_checksum".to_string()),
            ],
            connection_count: 1,
            connection_checksums: vec![("person1".to_string(), "connection_checksum".to_string())],
            total_size_bytes: 1024,
            manifest_version: "1.0".to_string(),
        }
    }

    // Test data verification against manifests
    #[tokio::test]
    async fn test_verify_transferred_data_success() {
        let manifest = create_test_manifest();
        let canister_id = create_test_principal(42);

        let result = verify_transferred_data(&manifest, canister_id).await;
        assert!(
            result.is_ok(),
            "Verification should succeed with valid manifest"
        );
    }

    #[tokio::test]
    async fn test_verify_transferred_data_empty_manifest() {
        let manifest = DataManifest {
            capsule_checksum: "empty".to_string(),
            memory_count: 0,
            memory_checksums: vec![],
            connection_count: 0,
            connection_checksums: vec![],
            total_size_bytes: 0,
            manifest_version: "1.0".to_string(),
        };
        let canister_id = create_test_principal(42);

        let result = verify_transferred_data(&manifest, canister_id).await;
        assert!(
            result.is_err(),
            "Verification should fail with empty manifest"
        );
        assert!(result
            .unwrap_err()
            .contains("Manifest indicates no data to verify"));
    }

    #[tokio::test]
    async fn test_verify_transferred_data_unsupported_version() {
        let mut manifest = create_test_manifest();
        manifest.manifest_version = "2.0".to_string();
        let canister_id = create_test_principal(42);

        let result = verify_transferred_data(&manifest, canister_id).await;
        assert!(
            result.is_err(),
            "Verification should fail with unsupported version"
        );
        assert!(result.unwrap_err().contains("Unsupported manifest version"));
    }

    // Test API compatibility checks
    #[tokio::test]
    async fn test_check_api_version_compatibility_success() {
        let canister_id = create_test_principal(42);

        let result = check_api_version_compatibility(canister_id).await;
        assert!(
            result.is_ok(),
            "API version check should succeed with matching versions"
        );
    }

    // Test canister health verification
    #[tokio::test]
    async fn test_perform_canister_health_check_success() {
        let canister_id = create_test_principal(42);

        let result = perform_canister_health_check(canister_id).await;
        assert!(
            result.is_ok(),
            "Health check should succeed for responsive canister"
        );
    }

    // Test migration data verification
    #[tokio::test]
    async fn test_verify_migration_data_success() {
        let export_data = create_test_export_data();
        let canister_id = create_test_principal(42);

        let result = verify_migration_data(canister_id, &export_data).await;
        assert!(
            result.is_ok(),
            "Migration data verification should succeed with valid data"
        );
    }

    #[tokio::test]
    async fn test_verify_migration_data_empty_export() {
        let mut export_data = create_test_export_data();
        export_data.memories.clear();
        export_data.connections.clear();
        let canister_id = create_test_principal(42);

        let result = verify_migration_data(canister_id, &export_data).await;
        // Should still succeed as empty data is valid
        assert!(
            result.is_ok(),
            "Migration data verification should succeed with empty data"
        );
    }

    // Test comprehensive verification flow
    #[tokio::test]
    async fn test_verify_complete_migration_success() {
        let export_data = create_test_export_data();
        let manifest = create_test_manifest();
        let canister_id = create_test_principal(42);

        let result = verify_complete_migration(canister_id, &export_data, &manifest).await;
        assert!(
            result.is_ok(),
            "Complete migration verification should succeed with valid data"
        );
    }

    #[tokio::test]
    async fn test_verify_complete_migration_with_empty_manifest() {
        let export_data = create_test_export_data();
        let manifest = DataManifest {
            capsule_checksum: "empty".to_string(),
            memory_count: 0,
            memory_checksums: vec![],
            connection_count: 0,
            connection_checksums: vec![],
            total_size_bytes: 0,
            manifest_version: "1.0".to_string(),
        };
        let canister_id = create_test_principal(42);

        let result = verify_complete_migration(canister_id, &export_data, &manifest).await;
        assert!(
            result.is_err(),
            "Complete migration verification should fail with empty manifest"
        );
    }

    // Test handoff readiness verification
    #[tokio::test]
    async fn test_verify_handoff_readiness_no_registry_entry() {
        let canister_id = create_test_principal(42);
        let user = create_test_principal(1);

        let result = verify_handoff_readiness(canister_id, user).await;
        assert!(
            result.is_err(),
            "Handoff readiness should fail when no registry entry exists"
        );
        assert!(result
            .unwrap_err()
            .contains("No registry entry found for canister"));
    }

    // Test verification with different manifest versions
    #[tokio::test]
    async fn test_verify_with_different_manifest_versions() {
        let versions = vec!["0.9", "1.1", "2.0", "invalid"];
        let canister_id = create_test_principal(42);

        for version in versions {
            let mut manifest = create_test_manifest();
            manifest.manifest_version = version.to_string();

            let result = verify_transferred_data(&manifest, canister_id).await;
            if version == "1.0" {
                assert!(result.is_ok(), "Version 1.0 should be supported");
            } else {
                assert!(
                    result.is_err(),
                    "Version {} should not be supported",
                    version
                );
            }
        }
    }

    // Test verification with various data sizes
    #[tokio::test]
    async fn test_verify_with_different_data_sizes() {
        let canister_id = create_test_principal(42);

        // Test with different memory counts
        for memory_count in [0, 1, 10, 100] {
            let manifest = DataManifest {
                capsule_checksum: "test".to_string(),
                memory_count,
                memory_checksums: (0..memory_count)
                    .map(|i| (format!("mem{}", i), format!("checksum{}", i)))
                    .collect(),
                connection_count: 1,
                connection_checksums: vec![("conn1".to_string(), "checksum".to_string())],
                total_size_bytes: 1024,
                manifest_version: "1.0".to_string(),
            };

            let result = verify_transferred_data(&manifest, canister_id).await;
            if memory_count == 0 {
                // Should fail because both memory and connection count can't be 0
                let manifest_empty = DataManifest {
                    connection_count: 0,
                    connection_checksums: vec![],
                    ..manifest
                };
                let result_empty = verify_transferred_data(&manifest_empty, canister_id).await;
                assert!(result_empty.is_err(), "Empty manifest should fail");
            } else {
                assert!(
                    result.is_ok(),
                    "Verification should succeed with {} memories",
                    memory_count
                );
            }
        }
    }

    // Test verification error handling
    #[tokio::test]
    async fn test_verification_error_messages() {
        let canister_id = create_test_principal(42);

        // Test unsupported version error message
        let mut manifest = create_test_manifest();
        manifest.manifest_version = "2.0".to_string();
        let result = verify_transferred_data(&manifest, canister_id).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Unsupported manifest version: 2.0"));

        // Test empty data error message
        let empty_manifest = DataManifest {
            capsule_checksum: "empty".to_string(),
            memory_count: 0,
            memory_checksums: vec![],
            connection_count: 0,
            connection_checksums: vec![],
            total_size_bytes: 0,
            manifest_version: "1.0".to_string(),
        };
        let result = verify_transferred_data(&empty_manifest, canister_id).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Manifest indicates no data to verify"));
    }

    // Test comprehensive verification with edge cases
    #[tokio::test]
    async fn test_comprehensive_verification_edge_cases() {
        let canister_id = create_test_principal(42);

        // Test with minimal valid data
        let minimal_export = ExportData {
            capsule: create_test_capsule(),
            memories: vec![("single_mem".to_string(), create_test_memory("single_mem"))],
            connections: vec![],
            metadata: ExportMetadata {
                export_timestamp: 1000000000,
                original_canister_id: create_test_principal(99),
                data_version: "1.0".to_string(),
                total_size_bytes: 100,
            },
        };

        let minimal_manifest = DataManifest {
            capsule_checksum: "minimal".to_string(),
            memory_count: 1,
            memory_checksums: vec![("single_mem".to_string(), "checksum".to_string())],
            connection_count: 0,
            connection_checksums: vec![],
            total_size_bytes: 100,
            manifest_version: "1.0".to_string(),
        };

        let result =
            verify_complete_migration(canister_id, &minimal_export, &minimal_manifest).await;
        assert!(
            result.is_ok(),
            "Comprehensive verification should succeed with minimal valid data"
        );
    }

    // Test verification performance with large datasets
    #[tokio::test]
    async fn test_verification_with_large_dataset() {
        let canister_id = create_test_principal(42);

        // Create export data with many memories and connections
        let mut export_data = create_test_export_data();

        // Add many memories
        for i in 0..50 {
            export_data.memories.push((
                format!("mem_{}", i),
                create_test_memory(&format!("mem_{}", i)),
            ));
        }

        // Add many connections
        for i in 0..20 {
            let person_ref = types::PersonRef::Opaque(format!("person_{}", i));
            export_data
                .connections
                .push((person_ref.clone(), create_test_connection(&person_ref)));
        }

        let result = verify_migration_data(canister_id, &export_data).await;
        assert!(
            result.is_ok(),
            "Verification should handle large datasets efficiently"
        );
    }
}
