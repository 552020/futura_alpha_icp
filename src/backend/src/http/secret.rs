// src/http/secret.rs
// Phase-1 secret management - stable secret init/rotation

use ic_cdk::api::management_canister::main::raw_rand;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

// Secret storage in stable memory
#[derive(Serialize, Deserialize)]
struct SecretStore {
    current_secret: [u8; 32],
    previous_secret: Option<[u8; 32]>, // For graceful rotation
}

impl Default for SecretStore {
    fn default() -> Self {
        Self {
            current_secret: [0u8; 32], // Will be generated on init
            previous_secret: None,
        }
    }
}

// Global secret store
static SECRET_STORE: Mutex<Option<SecretStore>> = Mutex::new(None);

// Initialize secret store
pub async fn init_secret() -> Result<(), String> {
    let secret = generate_hmac_secret().await?;
    let store = SecretStore {
        current_secret: secret,
        previous_secret: None,
    };

    let mut global_store = SECRET_STORE.lock().unwrap();
    *global_store = Some(store);
    Ok(())
}

// Rotate secret (for post_upgrade)
pub async fn post_upgrade_secret() -> Result<(), String> {
    let new_secret = generate_hmac_secret().await?;

    let mut global_store = SECRET_STORE.lock().unwrap();
    if let Some(ref mut store) = *global_store {
        store.previous_secret = Some(store.current_secret);
        store.current_secret = new_secret;
    } else {
        // If no store exists, initialize it
        let store = SecretStore {
            current_secret: new_secret,
            previous_secret: None,
        };
        *global_store = Some(store);
    }

    Ok(())
}

// Generate 32-byte HMAC secret
async fn generate_hmac_secret() -> Result<[u8; 32], String> {
    let rand_bytes = raw_rand()
        .await
        .map_err(|e| format!("Failed to get random bytes: {:?}", e))?;

    if rand_bytes.0.len() < 32 {
        return Err("Insufficient random bytes".to_string());
    }

    let mut secret = [0u8; 32];
    secret.copy_from_slice(&rand_bytes.0[..32]);
    Ok(secret)
}

// Get current secret for signing
pub fn get_current_secret() -> Result<[u8; 32], String> {
    let store = SECRET_STORE.lock().unwrap();
    match &*store {
        Some(store) => Ok(store.current_secret),
        None => Err("Secret store not initialized".to_string()),
    }
}

// Get previous secret for verification (during rotation)
pub fn get_previous_secret() -> Result<Option<[u8; 32]>, String> {
    let store = SECRET_STORE.lock().unwrap();
    match &*store {
        Some(store) => Ok(store.previous_secret),
        None => Err("Secret store not initialized".to_string()),
    }
}
