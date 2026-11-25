//! Shared by the PocketIC test binaries (`#[path = "common.rs"] mod common;`).
#![allow(dead_code)] // each binary uses a part of it

use candid::{encode_args, Principal};
use pocket_ic::PocketIc;

/// One trillion cycles
pub const T: u128 = 1_000_000_000_000;

/// A canister's release wasm. Build first:
/// `cargo build --target wasm32-unknown-unknown --release -p backend -p canister_factory`
pub fn wasm(name: &str) -> Vec<u8> {
    let path = format!("../../target/wasm32-unknown-unknown/release/{name}.wasm");
    std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}; build the wasms first"))
}

/// A shared backend (no owner) controlled by `controller`, with 5T cycles
pub fn install_backend(pic: &PocketIc, controller: Principal) -> Principal {
    let backend = pic.create_canister_with_settings(Some(controller), None);
    pic.add_cycles(backend, 5 * T);
    // `()`: the optional init argument left out. Empty bytes are not candid
    // and make init trap.
    pic.install_canister(
        backend,
        wasm("backend"),
        encode_args(()).unwrap(),
        Some(controller),
    );
    backend
}
