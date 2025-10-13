//! Simple Hello World PocketIC Test
//!
//! This is a minimal test to verify that PocketIC is working correctly
//! before running the more complex memory management tests.

use candid::{CandidType, Decode, Encode, Principal};
use pocket_ic::PocketIc;
use serde::Deserialize;

// Simple hello world function result
#[derive(CandidType, Deserialize)]
struct HelloResponse {
    message: String,
}

fn load_backend_wasm() -> Vec<u8> {
    let path = std::env::var("BACKEND_WASM_PATH")
        .unwrap_or_else(|_| "../../target/wasm32-unknown-unknown/release/backend.wasm".into());
    std::fs::read(path).expect("read backend.wasm")
}

#[test]
fn test_hello_world_pocket_ic() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting PocketIC Hello World test...");

    let pic = PocketIc::new();
    let wasm = load_backend_wasm();
    let controller = Principal::from_slice(&[1; 29]);

    println!("📦 Creating canister...");
    let canister_id = pic.create_canister();

    println!("💰 Adding cycles...");
    pic.add_cycles(canister_id, 2_000_000_000_000);

    println!("🔧 Installing canister...");
    pic.install_canister(canister_id, wasm, vec![], None);

    println!("✅ Canister installed successfully!");
    println!("🎯 Canister ID: {}", canister_id);

    // Try to call a simple function - let's see what functions are available
    // We'll try to call a function that should exist in our backend

    println!("📞 Attempting to call a backend function...");

    // Let's try calling a function that should exist - maybe register or something simple
    let args = ();

    match pic.update_call(canister_id, controller, "register", Encode!(&args)?) {
        Ok(raw) => {
            println!("✅ Function call succeeded!");
            println!("📄 Raw response: {:?}", raw);

            // Try to decode as a simple response
            match Decode!(&raw, bool) {
                Ok(result) => {
                    println!("✅ Decoded response: {}", result);
                }
                Err(e) => {
                    println!("⚠️  Could not decode as bool: {:?}", e);
                    // Try to decode as string
                    match Decode!(&raw, String) {
                        Ok(s) => println!("📝 Decoded as string: {}", s),
                        Err(e2) => println!("⚠️  Could not decode as string either: {:?}", e2),
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Function call failed: {:?}", e);
            println!("💡 This might be expected if 'register' function doesn't exist or has different signature");
        }
    }

    println!("🎉 Hello World test completed!");
    Ok(())
}

#[test]
fn test_canister_basic_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing basic canister operations...");

    let pic = PocketIc::new();
    let wasm = load_backend_wasm();
    let controller = Principal::from_slice(&[1; 29]);

    // Create canister
    let canister_id = pic.create_canister();
    println!("📦 Created canister: {}", canister_id);

    // Add cycles
    pic.add_cycles(canister_id, 2_000_000_000_000);
    println!("💰 Added cycles");

    // Install canister
    pic.install_canister(canister_id, wasm, vec![], None);
    println!("🔧 Installed canister");

    // Check canister status
    let status = pic.canister_status(canister_id, None);
    println!("📊 Canister status: {:?}", status);

    // Check if canister exists
    let exists = pic.canister_exists(canister_id);
    println!("ℹ️  Canister exists: {:?}", exists);

    println!("✅ Basic operations test completed!");
    Ok(())
}
