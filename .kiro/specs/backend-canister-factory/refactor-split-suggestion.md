# Refactor Plan: Split `canister_factory` Into Maintainable Modules

Goal: Improve cohesion, reduce coupling, and make testing easier by splitting `src/backend/src/canister_factory.rs` into focused modules. Keep external behavior (Candid/API) unchanged.

## Guiding principles

- MVP first: no behavior changes—only file/module reorganization and typed errors where trivial.
- Single responsibility per module; narrow, explicit interfaces.
- No AGPL code usage; adopt patterns, not code.
- Prepare for future split of "orchestrator" and "creator" without forcing it now.

## Proposed module layout

```
src/backend/src/
  canister_factory/
    mod.rs                      # Public facade: exports orchestrated functions and types
    types.rs                    # DTOs, enums, configs, typed errors
    auth.rs                     # ensure_owner/admin, caller validation
    registry.rs                 # PersonalCanisterRecord CRUD + queries
    cycles.rs                   # Reserve preflight/consume/status/alerts
    export.rs                   # Export capsule data + validation + manifest
    import.rs                   # Import sessions, chunk API, assemble, finalize
    verify.rs                   # Verification against manifest, health/API checks
    factory.rs                  # Create canister, install wasm, handoff, cleanup
    orchestrator.rs             # migrate_capsule state machine and coordination
```

Optional future:

- `adapters/management.rs` to abstract `create_canister/install_code/update_settings` for easier mocking.

## What moves where

- types.rs

  - MigrationResponse, MigrationStatus(+Response), ExportData(+Metadata)
  - MigrationState, MigrationConfig, MigrationStats
  - CreatePersonalCanisterConfig, ImportConfig, ImportSession(+Status), MemoryImportState, ChunkData, MemoryManifest, Import\* responses
  - DataManifest, Verification\* DTOs
  - AuthorizationRole, MigrationError, VerificationType (introduce typed errors now if feasible)

- auth.rs

  - ensure_owner, ensure_admin, validate_migration_caller, validate_admin_caller
  - helper: get_user_capsule_id, check_user_capsule_ownership

- registry.rs

  - PersonalCanisterRecord, CRUD functions: create/update/status/cycles, list/get/remove, by_user, by_status

- cycles.rs

  - preflight_cycles_reserve, consume, get/add/set threshold, status, monitoring report, alerts, logging helpers

- export.rs

  - export_user_capsule_data, calculate_export_data_size, validate_export_data
  - generate_export_manifest, verify_export_against_manifest, checksum helpers

- import.rs

  - begin_import, put_memory_chunk, commit_memory, finalize_import
  - import sessions store, cleanup*expired_sessions, get*\* session queries
  - create_memory_from_assembled_data, calculate_sha256 (later: real sha256)

- verify.rs

  - verify_transferred_data, perform_comprehensive_verification, perform_canister_health_check
  - check_api_version_compatibility / check_api_compatibility, is_version_compatible, handle_version_mismatch
  - helper verifiers: verify*\*count, verify*\*hashes, verify_total_size

- factory.rs

  - get_default_canister_cycles, load_personal_canister_wasm, prepare_personal_canister_init_args
  - create_personal_canister, install_personal_canister_wasm, complete_wasm_installation
  - cleanup_failed_canister_creation, cleanup_failed_migration
  - handoff_controllers, handoff_controllers_with_retry, rollback_controller_handoff

- orchestrator.rs

  - migrate_capsule, execute_migration_state_machine, get_migration_status
  - get_personal_canister_id/get_my_personal_canister_id, detailed status helpers

- mod.rs
  - pub use of selected types/functions for the external crate surface
  - single place to wire state access helpers from `crate::memory`

## State access pattern

- Keep `crate::memory::with_migration_state(_mut)` usage but confine it to modules that own the state (registry, cycles, import). Orchestrator/factory should call module APIs, not memory directly.

## Typed errors (lightweight)

Introduce a small `MigrationError` enum in `types.rs` and migrate functions that currently return `String` where trivial (auth/guards, preflights, basic validation). Keep complex changes for later.

```rust
pub enum MigrationError {
  Unauthorized { caller: Principal, required_role: AuthorizationRole },
  Disabled,
  ReserveInsufficient { required: u128, available: u128 },
  CreateFailed(String),
  InstallFailed(String),
  ImportFailed(String),
  VerifyFailed { reason: String, verification_type: VerificationType },
  HandoffFailed { reason: String, canister_id: Principal, user: Principal },
}
```

## Migration plan (low risk)

1. Create directory `canister_factory/` and move types.rs first. Update imports.
2. Extract `registry.rs` and `cycles.rs` (pure state helpers). Fix compile.
3. Extract `auth.rs` (guards). Fix compile.
4. Extract `factory.rs` (create/install/handoff). Fix compile.
5. Extract `export.rs` and `verify.rs`. Fix compile.
6. Extract `import.rs`. Fix compile (largest change—keep signatures identical).
7. Extract `orchestrator.rs` (`migrate_capsule`, state machine, status queries). Thin `mod.rs` facade.
8. Run build/tests; rename imports in callers as needed.

## Test strategy

- Keep existing unit tests; move them next to modules or keep a combined tests mod in `mod.rs` initially.
- Add quick unit tests for `cycles.rs` thresholds, `registry.rs` CRUD, and `auth.rs` guards.

## Notes

- Keep API_VERSION check, controller handoff sequence, and idempotency intact.
- Keep chunk/size limits configurable via `ImportConfig` with sensible defaults.
- Defer cryptographic hash to a later task; keep simple hash for MVP with a clear comment.

This split mirrors maintainable factory patterns while staying within MVP scope and minimizes risk by slicing along existing seams.
