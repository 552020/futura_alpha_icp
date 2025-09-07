# Lab

Small, isolated crates to explore Rust/IC patterns without touching production code.

## Conventions

- Each experiment is its own crate.
- Keep deps minimal.
- Put unit tests in `src/lib.rs`.
- Add `src/bin/demo.rs` if you want `cargo run`.

## Run

- `cargo test -p service_lifetimes_stateless`
- `cargo run  -p service_lifetimes_threadlocal`
