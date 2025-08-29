# Code Analysis: src/backend/src/canister_factory.rs

## Scope overview

This file currently centralizes: types (responses, status enums, configs), registry and cycles reserve management, internal data export/validation, chunked import session management, verification helpers, access control guards, canister creation + WASM install, controller handoff, migration orchestration, and tests.

### High-level concerns co-located

- Domain types and DTOs (e.g., `MigrationResponse`, `ExportData`, `MigrationState`, `PersonalCanisterRecord`, config structs)
- State/registry accessors and admin queries
- Cycles reserve management, monitoring, and alerts
- Export serialization and integrity checks
- Import session model, chunk ingestion, manifest validation, finalization
- Verification logic and reports
- Access control (owner/admin guards)
- Factory ops (create, install, handoff, cleanup)
- Orchestration (`migrate_capsule`) and state machine
- Test modules

## Observations

- Single-module growth: the file mixes orchestration with plumbing (cycles, registry, chunking). Hard to reason about responsibilities or test in isolation.
- Types co-located with logic: shared DTOs and errors live alongside business logic; re-use becomes harder across modules.
- Import subsystem is sizeable: sessions, chunks, manifests, assembly, validation—this is a self-contained subsystem that merits its own module.
- Verification helpers are currently stubbed/simulated; still, their signatures are good seams for extraction and mocking.
- Access control guards duplicated patterns (caller checks); should live in a small auth/guards module to avoid drift.
- Registry and cycles: both are cross-cutting concerns touching state; better as dedicated modules with narrow APIs.
- Clear seams already present: create/install/verify/handoff functions are separable and can be organized under a `factory` submodule.

## Coupling and cohesion

- Cohesion: medium-to-low. Multiple domains in one file (factory, data transfer, verification, admin ops).
- Coupling: high. Functions reach into `crate::memory` directly. Suggested: centralize state access in a thin repository-like layer to decouple read/write details from logic modules.

## Risks

- Change ripple: small changes in import/verify or cycles can affect the orchestration inadvertently.
- Test fragility: unit tests co-located and reliant on global state patterns increase brittleness.
- Review/maintainability: >2K lines rapidly becomes hard to review; split reduces cognitive load.

## Immediate hygiene (without structural refactor)

- Extract typed error enum (`MigrationError`) used across functions to replace `String` errors for stronger invariants and easier matching (you already hinted this in tests).
- Add docstrings on public functions clarifying side effects (state writes, registry mutations).
- Normalize logging keys/prefixes (e.g., `MIGRATION_*`, `CYCLES_*`) for grepability.
- Validate chunk size and total size (already present) and surface config in admin get/set; ensure defaults are sane.
- Ensure all state transitions persist immediately after mutation.

## Seams identified for modularization

- factory/: creation, install, handoff, cleanup (IC management canister interactions)
- registry/: records, queries, admin listing
- cycles/: reserve, consume, alerting
- transfer/: export, import sessions, chunk assembly, manifests
- verify/: data verification, API version checks, health checks
- auth/: ensure_owner/admin, caller validation helpers
- types/: DTOs, enums, configs, shared errors
- orchestrator/: state machine for `migrate_capsule`

## Testability notes

- With modules split, you can unit-test cycles, registry, and transfer logic independently.
- Orchestrator can be tested with trait-bound dependencies (mockable factory/verify interfaces) once you introduce traits for external interactions.

## Conclusion

The file is functional and well-commented, but it bundles many distinct concerns. Splitting along the seams above will reduce coupling, improve testability, and align with long-term maintainability without changing external behavior.
