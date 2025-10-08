# Rust Package and Crate Glossary

Here's the precise breakdown.

## Manifest

- A **manifest** is a `Cargo.toml` file that describes a Rust project.
- It contains metadata about the project (name, version, dependencies, etc.).
- Every folder with a `Cargo.toml` has a manifest, but not every manifest defines a package.
- The manifest can contain different sections:
  - `[package]` - defines a package
  - `[workspace]` - defines a workspace
  - `[dependencies]` - lists dependencies
  - Other configuration sections

## Package

- A folder with a `Cargo.toml` is a Cargo **manifest**.
- It's a **package** only if that manifest contains a `[package]` table.
- If it has only `[workspace]` (no `[package]`), it's a **virtual workspace root**, not a package.
- A package is the thing you version (`version = "x.y.z"`) and name (`name = "mypkg"`).
- A package can contain:

  - at most one library crate (`src/lib.rs`), and
  - zero or more binary crates (`src/main.rs` or `src/bin/*.rs`).

- Optional: a package can define examples, tests, benches (all are targets).

## Crate

- A crate is a single compile unit produced from a crate root:

  - **library crate** → `src/lib.rs`
  - **binary crate** → `src/main.rs` (or `src/bin/name.rs`)

- A package may produce multiple crates (one lib + many bins).
- Each crate is a separate target the compiler builds.

## "Unit You Build/Publish"

- **Build**: the compiler builds **crate targets** (lib/bin/examples/tests) that belong to one or more packages.

  - **Command scope**:

    - `cargo build` in a package dir builds that package's targets.
    - In a workspace root, `cargo build` can build **many packages** (all members) unless you select `-p <pkg>` or `--workspace`.

  - So, during build, the operational unit is a **crate target**, selected via the **package** context.

- **Publish**: Cargo publishes exactly **one package** (name + version) to a registry.

  - You cannot publish an individual crate from a multi-crate package separately.
  - `cargo publish` uploads the package source as a single archive.

## Build vs Publish (What Actually Happens)

### Build (`cargo build`, `cargo test`, etc.)

- Resolves dependencies, compiles selected **crate targets**.
- Produces artifacts in `target/` (debug or release).
- No network registry interaction (beyond fetching deps if needed).
- Controlled by flags like `--lib`, `--bin <name>`, `-p <package>`, `--workspace`.

### Publish (`cargo publish`)

- Packages the **source** of the current **package** (not the compiled artifacts).
- Runs checks (`cargo package` under the hood): includes/excludes files, verifies `Cargo.toml`, README, license, version uniqueness, etc.
- Uploads the tarball to a registry (e.g., crates.io). Version is immutable once published.
- Typical flow: `cargo publish --dry-run` → `cargo publish`.

## Quick Mental Model

- **Package** = what you **version & publish** (source bundle).
- **Crate** = what the compiler **builds** (lib/bin target) from that package.
- **Build** compiles crates; **publish** uploads the package's source.

## Package vs Manifest Clarification

You're right—the earlier phrasing was incomplete.

**Correct statement**:

- A folder with a `Cargo.toml` is a Cargo **manifest**.
- It's a **package** only if that manifest contains a `[package]` table.
- If it has only `[workspace]` (no `[package]`), it's a **virtual workspace root**, not a package.

## Quick Matrix

| Manifest Content                   | Is Package?          | Can Publish?       | Build Behavior                           |
| ---------------------------------- | -------------------- | ------------------ | ---------------------------------------- |
| `[package]` only                   | yes                  | yes                | builds this package's crates (lib/bin/…) |
| `[workspace]` only                 | no (virtual)         | no                 | builds selected member packages          |
| both `[package]` and `[workspace]` | yes (workspace root) | yes (root package) | can build root and/or other members      |

### Build Details

- **`[workspace]` only**: builds via `--workspace`, `-p`, or `default-members`
- **both sections**: can build the root package and/or other members

## Key Takeaways

- **Build** compiles crate targets; scope is controlled by the current directory (package or workspace root) and flags.
- **Publish** uploads exactly one package (the one with `[package]`) to a registry.
- In your example, the root is a virtual workspace: no package at the root; all real packages live under the listed `members/`.
