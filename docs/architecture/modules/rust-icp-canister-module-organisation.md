# Rust Module Separation (General)

- **units**: package → crate → modules (files/folders).
- **keep api small**: expose with `pub`/`pub(crate)`; default is private.
- **visibility you'll use most**: `pub(crate)` for intra-crate, `pub` only for public API, `pub(super)` for tightly-scoped helpers.
- **re-export** with `pub use` from a single entry (clean surface).
- **split by responsibility**, not by type: domain, storage, transport, etc.
- **separate crates only when**: reuse across projects, heavy optional deps, or to enforce strict boundaries.

## ICP-Specific Separation

- **keep `ic_cdk`** (macros, candid types) at the edge only (canister layer).
- **domain logic and data types** must be IC-agnostic (pure Rust).
- **storage modules own their persistence details**:

  - stable memory layout/memory IDs (document them),
  - migrations/versions for their data,
  - no cross-module globals touching stable memory.

- **a single "state access" gateway** (e.g., `with_state(|s| { ... })`) to avoid borrows across `await`.
- **upgrades**: each storage module ships its own migration step; canister `post_upgrade` orchestrates them in order.
- **optional features** (e.g., sqlite) behind Cargo features + runtime flags; the canister only depends on traits.

## Minimal Layering That Works

- **domain**: entities + use-case traits (no IC, no storage).
- **storage_kv**: implements domain traits using stable structures.
- **storage_sqlite** (optional): implements same traits with SQLite.
- **canister**: candid API, auth, flags, routing to one trait impl.

## When to Split Into Separate Crates (ICP)

- **yes**: domain (core), storage backends (kv/sqlite), canister shell.
- **no**: if experiment is small—keep one crate but separate modules and feature-gate.

## Checklist Before Coding

- do we have a tiny trait boundary for the use cases?
- what is public vs `pub(crate)`?
- which module owns which stable memory region and migration?
- can the domain compile without `ic_cdk` and without sqlite?
- can we run host tests for domain/storage logic?

## Common Pitfalls

- leaking `ic_cdk` or candid types into domain/storage.
- shared mutable state held across `await`.
- undocumented stable memory IDs and schema versions.
- wide `pub` surface; prefer `pub(crate)`.

---

_If you want, I can turn this into a one-page style guide for your repo._
