use super::test_utils::*;
use crate::canister_factory::types::*;
use candid::Principal;

fn setup_import_test_state() {
    with_mock_creation_state_mut(|state| {
        *state = PersonalCanisterCreationStateData {
            import_config: ImportConfig {
                max_chunk_size: 1_000_000,          // 1MB max chunk size
                max_total_import_size: 100_000_000, // 100MB max total import size
                session_timeout_seconds: 3600,      // 1 hour session timeout
            },
            import_sessions: std::collections::HashMap::new(),
            ..Default::default()
        };
    });
}

fn mock_begin_import(user: Principal) -> Result<ImportSessionResponse, String> {
    let session_id = format!("import_{}", simple_hash(&user.to_text()));
    let now = mock_time();

    with_mock_creation_state_mut(|state| {
        // Clean up expired sessions
        let timeout_nanos = state.import_config.session_timeout_seconds * 1_000_000_000;
        let expired_sessions: Vec<String> = state
            .import_sessions
            .iter()
            .filter(|(_, session)| {
                let session_age = now - session.last_activity_at;
                session_age > timeout_nanos && session.status == ImportSessionStatus::Active
            })
            .map(|(id, _)| id.clone())
            .collect();

        for session_id in expired_sessions {
            state.import_sessions.remove(&session_id);
        }

        // Check if user already has an active session
        let existing_active = state
            .import_sessions
            .values()
            .any(|s| s.user == user && s.status == ImportSessionStatus::Active);

        if existing_active {
            return Ok(ImportSessionResponse {
                success: false,
                session_id: None,
                message: "User already has an active import session".to_string(),
            });
        }

        // Create new import session
        let session = ImportSession {
            session_id: session_id.clone(),
            user,
            created_at: now,
            last_activity_at: now,
            total_expected_size: 0,
            total_received_size: 0,
            memories_in_progress: std::collections::HashMap::new(),
            completed_memories: std::collections::HashMap::new(),
            import_manifest: None,
            status: ImportSessionStatus::Active,
        };

        state.import_sessions.insert(session_id.clone(), session);

        Ok(ImportSessionResponse {
            success: true,
            session_id: Some(session_id),
            message: "Import session created successfully".to_string(),
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_session_creation() {
        setup_import_test_state();
        let user = create_test_principal(1);

        let result = mock_begin_import(user);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.success);
        assert!(response.session_id.is_some());
        assert!(response.message.contains("successfully"));
    }

    #[test]
    fn test_import_session_duplicate_prevention() {
        setup_import_test_state();
        let user = create_test_principal(1);

        // Create first session
        let result1 = mock_begin_import(user);
        assert!(result1.is_ok());
        assert!(result1.unwrap().success);

        // Try to create second session for same user
        let result2 = mock_begin_import(user);
        assert!(result2.is_ok());
        let response2 = result2.unwrap();
        assert!(!response2.success);
        assert!(response2.message.contains("already has an active"));
    }

    #[test]
    fn test_import_session_different_users() {
        setup_import_test_state();
        let user1 = create_test_principal(1);
        let user2 = create_test_principal(2);

        // Create sessions for different users
        let result1 = mock_begin_import(user1);
        let result2 = mock_begin_import(user2);

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result1.unwrap().success);
        assert!(result2.unwrap().success);
    }
}
