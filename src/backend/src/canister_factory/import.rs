use crate::canister_factory::types::*;
use crate::types;
use candid::Principal;
use std::collections::HashMap;

/// Begin a new import session for chunked data transfer
/// This function creates a new import session and returns a session ID
pub fn begin_import() -> Result<ImportSessionResponse, String> {
    let caller = ic_cdk::api::msg_caller();

    // Reject anonymous callers
    if caller == Principal::anonymous() {
        return Ok(ImportSessionResponse {
            success: false,
            session_id: None,
            message: "Anonymous callers cannot begin import sessions".to_string(),
        });
    }

    // Generate unique session ID
    let session_id = generate_session_id(caller);
    let now = ic_cdk::api::time();

    // Create new import session
    let session = ImportSession {
        session_id: session_id.clone(),
        user: caller,
        created_at: now,
        last_activity_at: now,
        total_expected_size: 0,
        total_received_size: 0,
        memories_in_progress: HashMap::new(),
        completed_memories: HashMap::new(),
        import_manifest: None,
        status: ImportSessionStatus::Active,
    };

    // Store the session
    crate::memory::with_migration_state_mut(|state| {
        // Clean up expired sessions before creating new one
        cleanup_expired_sessions_internal(state);

        // Check if user already has an active session
        let existing_active = state
            .import_sessions
            .values()
            .any(|s| s.user == caller && s.status == ImportSessionStatus::Active);

        if existing_active {
            return Err("User already has an active import session".to_string());
        }

        state.import_sessions.insert(session_id.clone(), session);
        Ok(())
    })?;

    ic_cdk::println!("Created import session {} for user {}", session_id, caller);

    Ok(ImportSessionResponse {
        success: true,
        session_id: Some(session_id),
        message: "Import session created successfully".to_string(),
    })
}

/// Upload a memory chunk to an active import session
/// This function handles individual chunk uploads with validation
pub fn put_memory_chunk(
    session_id: String,
    memory_id: String,
    chunk_index: u32,
    bytes: Vec<u8>,
    sha256: String,
) -> Result<ChunkUploadResponse, String> {
    let caller = ic_cdk::api::msg_caller();

    // Reject anonymous callers
    if caller == Principal::anonymous() {
        return Ok(ChunkUploadResponse {
            success: false,
            message: "Anonymous callers cannot upload chunks".to_string(),
            received_size: 0,
            total_expected_size: 0,
        });
    }

    crate::memory::with_migration_state_mut(|state| {
        // Get import configuration
        let config = &state.import_config;

        // Validate chunk size
        if bytes.len() as u64 > config.max_chunk_size {
            return Ok(ChunkUploadResponse {
                success: false,
                message: format!(
                    "Chunk size {} exceeds maximum allowed size {}",
                    bytes.len(),
                    config.max_chunk_size
                ),
                received_size: 0,
                total_expected_size: 0,
            });
        }

        // Get and validate session
        let session = match state.import_sessions.get_mut(&session_id) {
            Some(s) => s,
            None => {
                return Ok(ChunkUploadResponse {
                    success: false,
                    message: "Import session not found".to_string(),
                    received_size: 0,
                    total_expected_size: 0,
                })
            }
        };

        // Validate session ownership
        if session.user != caller {
            return Ok(ChunkUploadResponse {
                success: false,
                message: "Access denied: session belongs to different user".to_string(),
                received_size: 0,
                total_expected_size: 0,
            });
        }

        // Check session status
        if session.status != ImportSessionStatus::Active {
            return Ok(ChunkUploadResponse {
                success: false,
                message: format!("Session is not active (status: {:?})", session.status),
                received_size: 0,
                total_expected_size: 0,
            });
        }

        // Check session timeout
        let now = ic_cdk::api::time();
        let session_age = (now - session.last_activity_at) / 1_000_000_000; // Convert to seconds
        if session_age > config.session_timeout_seconds {
            session.status = ImportSessionStatus::Expired;
            return Ok(ChunkUploadResponse {
                success: false,
                message: "Session has expired".to_string(),
                received_size: 0,
                total_expected_size: 0,
            });
        }

        // Validate total import size
        let new_total_size = session.total_received_size + bytes.len() as u64;
        if new_total_size > config.max_total_import_size {
            return Ok(ChunkUploadResponse {
                success: false,
                message: format!(
                    "Total import size would exceed maximum allowed size {}",
                    config.max_total_import_size
                ),
                received_size: session.total_received_size,
                total_expected_size: session.total_expected_size,
            });
        }

        // Validate chunk hash
        let calculated_hash = calculate_sha256(&bytes);
        if calculated_hash != sha256 {
            return Ok(ChunkUploadResponse {
                success: false,
                message: "Chunk hash validation failed".to_string(),
                received_size: session.total_received_size,
                total_expected_size: session.total_expected_size,
            });
        }

        // Get or create memory import state
        let memory_state = session
            .memories_in_progress
            .entry(memory_id.clone())
            .or_insert_with(|| MemoryImportState {
                memory_id: memory_id.clone(),
                expected_chunks: 0,
                received_chunks: HashMap::new(),
                total_size: 0,
                received_size: 0,
                memory_metadata: None,
                is_complete: false,
            });

        // Check if chunk already exists
        if memory_state.received_chunks.contains_key(&chunk_index) {
            return Ok(ChunkUploadResponse {
                success: false,
                message: format!(
                    "Chunk {} already received for memory {}",
                    chunk_index, memory_id
                ),
                received_size: session.total_received_size,
                total_expected_size: session.total_expected_size,
            });
        }

        // Store the chunk
        let chunk = ChunkData {
            chunk_index,
            data: bytes.clone(),
            sha256,
            received_at: now,
        };

        memory_state.received_chunks.insert(chunk_index, chunk);
        memory_state.received_size += bytes.len() as u64;

        // Update session totals
        session.total_received_size += bytes.len() as u64;
        session.last_activity_at = now;

        ic_cdk::println!(
            "Received chunk {} for memory {} in session {} ({} bytes)",
            chunk_index,
            memory_id,
            session_id,
            bytes.len()
        );

        Ok(ChunkUploadResponse {
            success: true,
            message: format!("Chunk {} uploaded successfully", chunk_index),
            received_size: session.total_received_size,
            total_expected_size: session.total_expected_size,
        })
    })
}

/// Commit a memory after all chunks have been uploaded
/// This function assembles chunks into a complete memory and validates integrity
pub fn commit_memory(
    session_id: String,
    manifest: MemoryManifest,
) -> Result<MemoryCommitResponse, String> {
    let caller = ic_cdk::api::msg_caller();

    // Reject anonymous callers
    if caller == Principal::anonymous() {
        return Ok(MemoryCommitResponse {
            success: false,
            message: "Anonymous callers cannot commit memories".to_string(),
            memory_id: manifest.memory_id.clone(),
            assembled_size: 0,
        });
    }

    crate::memory::with_migration_state_mut(|state| {
        // Get and validate session
        let session = match state.import_sessions.get_mut(&session_id) {
            Some(s) => s,
            None => {
                return Ok(MemoryCommitResponse {
                    success: false,
                    message: "Import session not found".to_string(),
                    memory_id: manifest.memory_id.clone(),
                    assembled_size: 0,
                })
            }
        };

        // Validate session ownership
        if session.user != caller {
            return Ok(MemoryCommitResponse {
                success: false,
                message: "Access denied: session belongs to different user".to_string(),
                memory_id: manifest.memory_id.clone(),
                assembled_size: 0,
            });
        }

        // Check session status
        if session.status != ImportSessionStatus::Active {
            return Ok(MemoryCommitResponse {
                success: false,
                message: format!("Session is not active (status: {:?})", session.status),
                memory_id: manifest.memory_id.clone(),
                assembled_size: 0,
            });
        }

        // Get memory import state
        let memory_state = match session.memories_in_progress.get_mut(&manifest.memory_id) {
            Some(state) => state,
            None => {
                return Ok(MemoryCommitResponse {
                    success: false,
                    message: format!("Memory {} not found in session", manifest.memory_id),
                    memory_id: manifest.memory_id.clone(),
                    assembled_size: 0,
                })
            }
        };

        // Validate chunk count
        if memory_state.received_chunks.len() as u32 != manifest.total_chunks {
            return Ok(MemoryCommitResponse {
                success: false,
                message: format!(
                    "Chunk count mismatch: received {}, expected {}",
                    memory_state.received_chunks.len(),
                    manifest.total_chunks
                ),
                memory_id: manifest.memory_id.clone(),
                assembled_size: 0,
            });
        }

        // Validate total size
        if memory_state.received_size != manifest.total_size {
            return Ok(MemoryCommitResponse {
                success: false,
                message: format!(
                    "Size mismatch: received {}, expected {}",
                    memory_state.received_size, manifest.total_size
                ),
                memory_id: manifest.memory_id.clone(),
                assembled_size: 0,
            });
        }

        // Assemble chunks in order
        let mut assembled_data = Vec::new();
        for chunk_index in 0..manifest.total_chunks {
            match memory_state.received_chunks.get(&chunk_index) {
                Some(chunk) => {
                    // Validate chunk checksum against manifest
                    if chunk_index < manifest.chunk_checksums.len() as u32 {
                        let expected_checksum = &manifest.chunk_checksums[chunk_index as usize];
                        if chunk.sha256 != *expected_checksum {
                            return Ok(MemoryCommitResponse {
                                success: false,
                                message: format!(
                                    "Chunk {} checksum mismatch for memory {}",
                                    chunk_index, manifest.memory_id
                                ),
                                memory_id: manifest.memory_id.clone(),
                                assembled_size: 0,
                            });
                        }
                    }
                    assembled_data.extend_from_slice(&chunk.data);
                }
                None => {
                    return Ok(MemoryCommitResponse {
                        success: false,
                        message: format!(
                            "Missing chunk {} for memory {}",
                            chunk_index, manifest.memory_id
                        ),
                        memory_id: manifest.memory_id.clone(),
                        assembled_size: 0,
                    })
                }
            }
        }

        // Validate final assembled data checksum
        let final_checksum = calculate_sha256(&assembled_data);
        if final_checksum != manifest.final_checksum {
            return Ok(MemoryCommitResponse {
                success: false,
                message: format!(
                    "Final checksum mismatch for memory {}: calculated {}, expected {}",
                    manifest.memory_id, final_checksum, manifest.final_checksum
                ),
                memory_id: manifest.memory_id.clone(),
                assembled_size: 0,
            });
        }

        // Create the complete memory object
        let memory = create_memory_from_assembled_data(
            &manifest.memory_id,
            assembled_data,
            memory_state.memory_metadata.as_ref(),
        )?;

        // Move memory from in-progress to completed
        session
            .completed_memories
            .insert(manifest.memory_id.clone(), memory);
        session.memories_in_progress.remove(&manifest.memory_id);

        // Update session activity
        session.last_activity_at = ic_cdk::api::time();

        ic_cdk::println!(
            "Successfully committed memory {} in session {} ({} bytes)",
            manifest.memory_id,
            session_id,
            manifest.total_size
        );

        Ok(MemoryCommitResponse {
            success: true,
            message: format!("Memory {} committed successfully", manifest.memory_id),
            memory_id: manifest.memory_id,
            assembled_size: manifest.total_size,
        })
    })
}

/// Finalize the import session after all memories have been committed
/// This function completes the import process and makes the data available
pub fn finalize_import(session_id: String) -> Result<ImportFinalizationResponse, String> {
    let caller = ic_cdk::api::msg_caller();

    // Reject anonymous callers
    if caller == Principal::anonymous() {
        return Ok(ImportFinalizationResponse {
            success: false,
            message: "Anonymous callers cannot finalize imports".to_string(),
            total_memories_imported: 0,
            total_size_imported: 0,
        });
    }

    crate::memory::with_migration_state_mut(|state| {
        // Get and validate session
        let session = match state.import_sessions.get_mut(&session_id) {
            Some(s) => s,
            None => {
                return Ok(ImportFinalizationResponse {
                    success: false,
                    message: "Import session not found".to_string(),
                    total_memories_imported: 0,
                    total_size_imported: 0,
                })
            }
        };

        // Validate session ownership
        if session.user != caller {
            return Ok(ImportFinalizationResponse {
                success: false,
                message: "Access denied: session belongs to different user".to_string(),
                total_memories_imported: 0,
                total_size_imported: 0,
            });
        }

        // Check session status
        if session.status != ImportSessionStatus::Active {
            return Ok(ImportFinalizationResponse {
                success: false,
                message: format!("Session is not active (status: {:?})", session.status),
                total_memories_imported: 0,
                total_size_imported: 0,
            });
        }

        // Check that all memories in progress have been committed
        if !session.memories_in_progress.is_empty() {
            return Ok(ImportFinalizationResponse {
                success: false,
                message: format!(
                    "Cannot finalize: {} memories still in progress",
                    session.memories_in_progress.len()
                ),
                total_memories_imported: 0,
                total_size_imported: 0,
            });
        }

        // Update session status
        session.status = ImportSessionStatus::Finalizing;
        session.last_activity_at = ic_cdk::api::time();

        let total_memories = session.completed_memories.len() as u32;
        let total_size = session.total_received_size;

        // Perform final validation if manifest was provided
        if let Some(ref manifest) = session.import_manifest {
            if let Err(e) = validate_import_against_manifest(session, manifest) {
                session.status = ImportSessionStatus::Failed;
                return Ok(ImportFinalizationResponse {
                    success: false,
                    message: format!("Import validation failed: {}", e),
                    total_memories_imported: 0,
                    total_size_imported: 0,
                });
            }
        }

        // Mark session as completed
        session.status = ImportSessionStatus::Completed;

        ic_cdk::println!(
            "Successfully finalized import session {} for user {}: {} memories, {} bytes",
            session_id,
            caller,
            total_memories,
            total_size
        );

        Ok(ImportFinalizationResponse {
            success: true,
            message: format!(
                "Import finalized successfully: {} memories imported",
                total_memories
            ),
            total_memories_imported: total_memories,
            total_size_imported: total_size,
        })
    })
}

/// Clean up expired import sessions
pub fn cleanup_expired_sessions() -> u32 {
    crate::memory::with_migration_state_mut(|state| cleanup_expired_sessions_internal(state))
}

// Helper functions for import system

/// Generate a unique session ID for import operations
fn generate_session_id(user: Principal) -> String {
    let timestamp = ic_cdk::api::time();
    let user_text = user.to_text();
    let session_data = format!("{}:{}", user_text, timestamp);
    format!("import_{}", simple_hash(&session_data))
}

/// Calculate SHA-256 hash of data (simplified implementation for MVP)
fn calculate_sha256(data: &[u8]) -> String {
    // For MVP, use a simple hash function
    // In production, this should use proper SHA-256
    simple_hash(&String::from_utf8_lossy(data))
}

/// Simple hash function for checksums (using a basic approach for MVP)
fn simple_hash(data: &str) -> String {
    let mut hash: u64 = 5381;
    for byte in data.bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u64);
    }
    format!("{:016x}", hash)
}

/// Create a memory object from assembled chunk data
fn create_memory_from_assembled_data(
    memory_id: &str,
    data: Vec<u8>,
    metadata: Option<&types::Memory>,
) -> Result<types::Memory, String> {
    let now = ic_cdk::api::time();

    // Use provided metadata or create default
    let memory = if let Some(existing_memory) = metadata {
        // Clone existing memory and update data
        let mut memory = existing_memory.clone();
        memory.data.data = Some(data);
        memory
    } else {
        // Create basic memory structure for MVP
        let data_size = data.len() as u64;
        types::Memory {
            id: memory_id.to_string(),
            info: types::MemoryInfo {
                memory_type: types::MemoryType::Document,
                name: format!("Imported Memory {}", memory_id),
                content_type: "application/octet-stream".to_string(),
                created_at: now,
                updated_at: now,
                uploaded_at: now,
                date_of_memory: Some(now),
            },
            data: types::MemoryData {
                blob_ref: types::BlobRef {
                    kind: types::MemoryBlobKind::ICPCapsule,
                    locator: format!("imported:{}", memory_id),
                    hash: None,
                },
                data: Some(data),
            },
            access: types::MemoryAccess::Private,
            metadata: types::MemoryMetadata::Document(types::DocumentMetadata {
                base: types::MemoryMetadataBase {
                    size: data_size,
                    mime_type: "application/octet-stream".to_string(),
                    original_name: format!("imported_{}.bin", memory_id),
                    uploaded_at: now.to_string(),
                    date_of_memory: Some(now.to_string()),
                    people_in_memory: None,
                    format: Some("binary".to_string()),
                },
            }),
        }
    };

    Ok(memory)
}

/// Internal function to clean up expired sessions
fn cleanup_expired_sessions_internal(state: &mut PersonalCanisterCreationStateData) -> u32 {
    let now = ic_cdk::api::time();
    let timeout_nanos = state.import_config.session_timeout_seconds * 1_000_000_000;

    let mut expired_sessions = Vec::new();

    // Find expired sessions
    for (session_id, session) in &state.import_sessions {
        let session_age = now - session.last_activity_at;
        if session_age > timeout_nanos && session.status == ImportSessionStatus::Active {
            expired_sessions.push(session_id.clone());
        }
    }

    // Remove expired sessions
    let cleanup_count = expired_sessions.len() as u32;
    for session_id in expired_sessions {
        if let Some(mut session) = state.import_sessions.remove(&session_id) {
            session.status = ImportSessionStatus::Expired;
            ic_cdk::println!("Cleaned up expired import session: {}", session_id);
        }
    }

    cleanup_count
}

/// Validate import session against manifest
fn validate_import_against_manifest(
    session: &ImportSession,
    manifest: &DataManifest,
) -> Result<(), String> {
    // Check memory count
    if session.completed_memories.len() as u32 != manifest.memory_count {
        return Err(format!(
            "Memory count mismatch: imported {}, manifest expects {}",
            session.completed_memories.len(),
            manifest.memory_count
        ));
    }

    // Validate each memory against manifest checksums
    for (memory_id, memory) in &session.completed_memories {
        let expected_checksum = manifest
            .memory_checksums
            .iter()
            .find(|(id, _)| id == memory_id)
            .map(|(_, checksum)| checksum)
            .ok_or_else(|| format!("Memory '{}' not found in manifest", memory_id))?;

        // Calculate checksum for imported memory
        let empty_vec = Vec::new();
        let memory_data = memory.data.data.as_ref().unwrap_or(&empty_vec);
        let calculated_checksum = calculate_sha256(memory_data);

        if calculated_checksum != *expected_checksum {
            return Err(format!(
                "Memory '{}' checksum mismatch: calculated '{}', manifest expects '{}'",
                memory_id, calculated_checksum, expected_checksum
            ));
        }
    }

    ic_cdk::println!(
        "Import validation passed: {} memories validated against manifest",
        session.completed_memories.len()
    );

    Ok(())
}
