//! Simple PocketIC Test to verify basic setup
//!
//! This test verifies that PocketIC is working correctly with our backend canister.

use anyhow::Result;
use candid::{CandidType, Decode, Encode, Principal};
use pocket_ic::PocketIc;
use serde::Deserialize;

fn load_backend_wasm() -> Vec<u8> {
    let path = std::env::var("BACKEND_WASM_PATH")
        .unwrap_or_else(|_| "../../target/wasm32-unknown-unknown/release/backend.wasm".into());
    std::fs::read(path).expect("read backend.wasm")
}

#[test]
fn test_simple_pocket_ic_setup() -> Result<()> {
    println!("🚀 Testing basic PocketIC setup...");
    
    let pic = PocketIc::new();
    let wasm = load_backend_wasm();
    let controller = Principal::from_slice(&[1; 29]);
    
    // Create canister
    let canister_id = pic.create_canister();
    println!("📦 Created canister: {}", canister_id);
    
    // Add cycles
    pic.add_cycles(canister_id, 2_000_000_000_000);
    println!("💰 Added cycles");
    
    // Install canister (using default controller)
    pic.install_canister(canister_id, wasm, vec![], None);
    println!("🔧 Installed canister");
    
    // Test simple query function
    let raw = pic
        .query_call(canister_id, controller, "whoami", Encode!()?)
        .map_err(|e| anyhow::anyhow!("Query call failed: {:?}", e))?;
    
    let result: Principal = Decode!(&raw, Principal)?;
    println!("✅ whoami returned: {}", result);
    
    // Test greet function
    let raw = pic
        .query_call(canister_id, controller, "greet", Encode!(&"World")?)
        .map_err(|e| anyhow::anyhow!("Query call failed: {:?}", e))?;
    
    let result: String = Decode!(&raw, String)?;
    println!("✅ greet returned: {}", result);
    
    println!("🎉 Basic PocketIC setup test completed successfully!");
    Ok(())
}
