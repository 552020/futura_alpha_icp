# Phase 1 — Canister Foundations Implementation

**Status**: Implementation Guide  
**Phase**: Phase 1 from `http_request_implementation_final.md`  
**Duration**: 1-2 days  
**Owner**: Mid-senior Backend Engineer (Rust)

---

## 🎯 Phase 1 Objectives

Implement the foundational token system for the canister:

1. **Secret Management** - Generate and store HMAC secrets
2. **Token Model** - Stateless HMAC token structure
3. **Token Mint API** - Query method to generate tokens
4. **Verification Helper** - Token validation logic

---

## 📋 Task Breakdown

### Task 1: Secret Management (2-3 hours)

**Objective**: Generate and store 32-byte HMAC secrets with rotation capability.

#### Implementation

```rust
// Add to src/backend/src/lib.rs

use std::sync::Mutex;
use ic_cdk::api::management_canister::main::raw_rand;
use serde::{Deserialize, Serialize};

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
async fn init_secret_store() -> Result<(), String> {
    let secret = generate_hmac_secret().await?;
    let store = SecretStore {
        current_secret: secret,
        previous_secret: None,
    };

    let mut global_store = SECRET_STORE.lock().unwrap();
    *global_store = Some(store);
    Ok(())
}

// Generate 32-byte HMAC secret
async fn generate_hmac_secret() -> Result<[u8; 32], String> {
    let rand_bytes = raw_rand().await.map_err(|e| format!("Failed to get random bytes: {:?}", e))?;

    if rand_bytes.0.len() < 32 {
        return Err("Insufficient random bytes".to_string());
    }

    let mut secret = [0u8; 32];
    secret.copy_from_slice(&rand_bytes.0[..32]);
    Ok(secret)
}

// Rotate secret (for post_upgrade)
async fn rotate_secret() -> Result<(), String> {
    let new_secret = generate_hmac_secret().await?;

    let mut global_store = SECRET_STORE.lock().unwrap();
    if let Some(ref mut store) = *global_store {
        store.previous_secret = Some(store.current_secret);
        store.current_secret = new_secret;
    } else {
        return Err("Secret store not initialized".to_string());
    }

    Ok(())
}

// Get current secret for signing
fn get_current_secret() -> Result<[u8; 32], String> {
    let store = SECRET_STORE.lock().unwrap();
    match &*store {
        Some(store) => Ok(store.current_secret),
        None => Err("Secret store not initialized".to_string()),
    }
}
```

#### Testing

```rust
#[cfg(test)]
mod secret_tests {
    use super::*;

    #[tokio::test]
    async fn test_secret_generation() {
        let secret = generate_hmac_secret().await.unwrap();
        assert_eq!(secret.len(), 32);
        assert_ne!(secret, [0u8; 32]); // Not all zeros
    }

    #[tokio::test]
    async fn test_secret_rotation() {
        init_secret_store().await.unwrap();

        let original_secret = get_current_secret().unwrap();
        rotate_secret().await.unwrap();
        let new_secret = get_current_secret().unwrap();

        assert_ne!(original_secret, new_secret);
    }
}
```

---

### Task 2: Token Model (3-4 hours)

**Objective**: Define stateless HMAC token structure with canonical JSON payload.

#### Implementation

```rust
// Add to src/backend/src/lib.rs

use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde_json::{json, Value};
use base64::{Engine as _, engine::general_purpose};
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

// Token payload structure
#[derive(Debug, Clone)]
struct TokenPayload {
    version: u32,
    scope: TokenScope,
    expiration: u64,
    nonce: String,
}

#[derive(Debug, Clone)]
struct TokenScope {
    memory_id: String,
    variants: Vec<String>,
}

// Token minting result
#[derive(Debug)]
struct TokenResult {
    token: String,
    expires_at: u64,
}

// Create canonical JSON payload
fn create_token_payload(
    memory_id: String,
    variants: Vec<String>,
    ttl_secs: u64,
) -> Result<TokenPayload, String> {
    let now = ic_cdk::api::time() / 1_000_000_000; // Convert to seconds
    let expiration = now + ttl_secs;
    let nonce = generate_nonce()?;

    Ok(TokenPayload {
        version: 1,
        scope: TokenScope {
            memory_id,
            variants,
        },
        expiration,
        nonce,
    })
}

// Generate 96-bit random nonce
fn generate_nonce() -> Result<String, String> {
    let rand_bytes = ic_cdk::api::management_canister::main::raw_rand()
        .await
        .map_err(|e| format!("Failed to get random bytes: {:?}", e))?;

    if rand_bytes.0.len() < 12 {
        return Err("Insufficient random bytes for nonce".to_string());
    }

    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&rand_bytes.0[..12]);
    Ok(general_purpose::STANDARD.encode(nonce))
}

// Convert payload to canonical JSON
fn payload_to_canonical_json(payload: &TokenPayload) -> Result<String, String> {
    let json_value = json!({
        "ver": payload.version,
        "scope": {
            "memory_id": payload.scope.memory_id,
            "variants": payload.scope.variants
        },
        "exp": payload.expiration,
        "nonce": payload.nonce
    });

    serde_json::to_string(&json_value)
        .map_err(|e| format!("Failed to serialize payload: {}", e))
}
```

#### Testing

```rust
#[cfg(test)]
mod token_model_tests {
    use super::*;

    #[tokio::test]
    async fn test_payload_creation() {
        let payload = create_token_payload(
            "test-memory".to_string(),
            vec!["thumbnail".to_string(), "preview".to_string()],
            180,
        ).await.unwrap();

        assert_eq!(payload.version, 1);
        assert_eq!(payload.scope.memory_id, "test-memory");
        assert_eq!(payload.scope.variants.len(), 2);
        assert!(payload.expiration > ic_cdk::api::time() / 1_000_000_000);
    }

    #[tokio::test]
    async fn test_canonical_json() {
        let payload = create_token_payload(
            "test-memory".to_string(),
            vec!["thumbnail".to_string()],
            180,
        ).await.unwrap();

        let json = payload_to_canonical_json(&payload).unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["ver"], 1);
        assert_eq!(parsed["scope"]["memory_id"], "test-memory");
        assert_eq!(parsed["exp"], payload.expiration);
    }
}
```

---

### Task 3: Token Mint API (4-5 hours)

**Objective**: Implement query method to generate tokens with permission validation.

#### Implementation

```rust
// Add to src/backend/src/lib.rs

use ic_cdk::api::msg_caller;
use ic_cdk::query;

// Token minting API (QUERY method)
#[query]
async fn mint_http_token(
    memory_id: String,
    variants: Vec<String>,
    ttl_secs: u64,
) -> Result<TokenResult, String> {
    // Validate inputs
    if memory_id.is_empty() {
        return Err("Memory ID cannot be empty".to_string());
    }

    if variants.is_empty() {
        return Err("At least one variant must be specified".to_string());
    }

    if ttl_secs == 0 || ttl_secs > 3600 { // Max 1 hour
        return Err("TTL must be between 1 and 3600 seconds".to_string());
    }

    // Validate variant names
    let valid_variants = ["thumbnail", "preview", "placeholder", "original"];
    for variant in &variants {
        if !valid_variants.contains(&variant.as_str()) {
            return Err(format!("Invalid variant: {}", variant));
        }
    }

    // Validate caller has VIEW permission on memory
    let caller = msg_caller();
    validate_memory_access(&memory_id, caller).await?;

    // Create token payload
    let payload = create_token_payload(memory_id, variants, ttl_secs).await?;

    // Sign token
    let token = sign_token(&payload).await?;

    Ok(TokenResult {
        token,
        expires_at: payload.expiration,
    })
}

// Sign token with HMAC
async fn sign_token(payload: &TokenPayload) -> Result<String, String> {
    let secret = get_current_secret()?;
    let canonical_json = payload_to_canonical_json(payload)?;

    // Create HMAC
    let mut mac = HmacSha256::new_from_slice(&secret)
        .map_err(|e| format!("Failed to create HMAC: {}", e))?;
    mac.update(canonical_json.as_bytes());
    let signature = mac.finalize();

    // Combine payload and signature
    let payload_b64 = general_purpose::STANDARD.encode(canonical_json.as_bytes());
    let signature_b64 = general_purpose::STANDARD.encode(signature.into_bytes());

    // Format: payload.signature
    Ok(format!("{}.{}", payload_b64, signature_b64))
}

// Validate memory access using existing Futura logic
async fn validate_memory_access(memory_id: &str, caller: Principal) -> Result<(), String> {
    // Use existing Futura access control
    let env = CanisterEnv;
    let store = StoreAdapter;

    let memory = memories_read_core(&env, &store, memory_id.to_string())
        .map_err(|e| format!("Memory not found: {}", e))?;

    // Check if caller has VIEW permission
    let ctx = PrincipalContext {
        principal: caller,
        now_ns: ic_cdk::api::time(),
        link: None,
    };

    let perm_mask = effective_perm_mask(&memory, &ctx);
    if perm_mask & Perm::VIEW.bits() == 0 {
        return Err("Access denied: insufficient permissions".to_string());
    }

    Ok(())
}
```

#### Testing

```rust
#[cfg(test)]
mod token_mint_tests {
    use super::*;
    use ic_cdk::api::caller;

    #[tokio::test]
    async fn test_mint_token_success() {
        // Setup test memory and permissions
        setup_test_memory().await;

        let result = mint_http_token(
            "test-memory".to_string(),
            vec!["thumbnail".to_string(), "preview".to_string()],
            180,
        ).await.unwrap();

        assert!(!result.token.is_empty());
        assert!(result.expires_at > ic_cdk::api::time() / 1_000_000_000);
    }

    #[tokio::test]
    async fn test_mint_token_invalid_variant() {
        let result = mint_http_token(
            "test-memory".to_string(),
            vec!["invalid-variant".to_string()],
            180,
        ).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid variant"));
    }

    #[tokio::test]
    async fn test_mint_token_ttl_validation() {
        let result = mint_http_token(
            "test-memory".to_string(),
            vec!["thumbnail".to_string()],
            0,
        ).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("TTL must be between 1 and 3600 seconds"));
    }
}
```

---

### Task 4: Verification Helper (3-4 hours)

**Objective**: Implement token validation logic with signature verification and expiration checks.

#### Implementation

```rust
// Add to src/backend/src/lib.rs

// Token verification result
#[derive(Debug)]
struct TokenVerificationResult {
    scope: TokenScope,
    expires_at: u64,
}

// Verify token signature and extract scope
async fn verify_http_token(token: &str) -> Result<TokenVerificationResult, String> {
    // Parse token format: payload.signature
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 2 {
        return Err("Invalid token format".to_string());
    }

    let payload_b64 = parts[0];
    let signature_b64 = parts[1];

    // Decode payload
    let payload_bytes = general_purpose::STANDARD
        .decode(payload_b64)
        .map_err(|_| "Invalid token payload encoding")?;

    let payload_str = String::from_utf8(payload_bytes)
        .map_err(|_| "Invalid token payload UTF-8")?;

    // Decode signature
    let signature_bytes = general_purpose::STANDARD
        .decode(signature_b64)
        .map_err(|_| "Invalid token signature encoding")?;

    // Verify signature
    let secret = get_current_secret()?;
    let mut mac = HmacSha256::new_from_slice(&secret)
        .map_err(|e| format!("Failed to create HMAC: {}", e))?;
    mac.update(payload_str.as_bytes());

    mac.verify_slice(&signature_bytes)
        .map_err(|_| "Invalid token signature")?;

    // Parse payload JSON
    let payload_json: Value = serde_json::from_str(&payload_str)
        .map_err(|e| format!("Invalid token payload JSON: {}", e))?;

    // Extract fields
    let version = payload_json["ver"].as_u64()
        .ok_or("Missing version field")?;

    if version != 1 {
        return Err("Unsupported token version".to_string());
    }

    let memory_id = payload_json["scope"]["memory_id"].as_str()
        .ok_or("Missing memory_id in scope")?
        .to_string();

    let variants = payload_json["scope"]["variants"].as_array()
        .ok_or("Missing variants in scope")?
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();

    let expiration = payload_json["exp"].as_u64()
        .ok_or("Missing expiration field")?;

    // Check expiration
    let now = ic_cdk::api::time() / 1_000_000_000;
    if now > expiration {
        return Err("Token expired".to_string());
    }

    Ok(TokenVerificationResult {
        scope: TokenScope {
            memory_id,
            variants,
        },
        expires_at: expiration,
    })
}

// Validate token scope against request path
fn validate_token_scope(
    verification_result: &TokenVerificationResult,
    memory_id: &str,
    variant: &str,
) -> Result<(), String> {
    // Check memory ID match
    if verification_result.scope.memory_id != memory_id {
        return Err("Token scope mismatch: memory_id".to_string());
    }

    // Check variant in scope
    if !verification_result.scope.variants.contains(&variant.to_string()) {
        return Err("Token scope mismatch: variant not authorized".to_string());
    }

    Ok(())
}
```

#### Testing

```rust
#[cfg(test)]
mod verification_tests {
    use super::*;

    #[tokio::test]
    async fn test_verify_valid_token() {
        // Create and verify token
        let payload = create_token_payload(
            "test-memory".to_string(),
            vec!["thumbnail".to_string()],
            180,
        ).await.unwrap();

        let token = sign_token(&payload).await.unwrap();
        let result = verify_http_token(&token).await.unwrap();

        assert_eq!(result.scope.memory_id, "test-memory");
        assert_eq!(result.scope.variants, vec!["thumbnail"]);
    }

    #[tokio::test]
    async fn test_verify_expired_token() {
        // Create expired token
        let mut payload = create_token_payload(
            "test-memory".to_string(),
            vec!["thumbnail".to_string()],
            1, // 1 second TTL
        ).await.unwrap();

        // Manually set expiration to past
        payload.expiration = ic_cdk::api::time() / 1_000_000_000 - 100;

        let token = sign_token(&payload).await.unwrap();
        let result = verify_http_token(&token).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expired"));
    }

    #[tokio::test]
    async fn test_verify_invalid_signature() {
        let payload = create_token_payload(
            "test-memory".to_string(),
            vec!["thumbnail".to_string()],
            180,
        ).await.unwrap();

        let token = sign_token(&payload).await.unwrap();
        let mut invalid_token = token.clone();
        invalid_token.push_str("invalid");

        let result = verify_http_token(&invalid_token).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid token signature"));
    }

    #[tokio::test]
    async fn test_scope_validation() {
        let verification_result = TokenVerificationResult {
            scope: TokenScope {
                memory_id: "test-memory".to_string(),
                variants: vec!["thumbnail".to_string(), "preview".to_string()],
            },
            expires_at: ic_cdk::api::time() / 1_000_000_000 + 180,
        };

        // Valid scope
        assert!(validate_token_scope(&verification_result, "test-memory", "thumbnail").is_ok());
        assert!(validate_token_scope(&verification_result, "test-memory", "preview").is_ok());

        // Invalid scope
        assert!(validate_token_scope(&verification_result, "other-memory", "thumbnail").is_err());
        assert!(validate_token_scope(&verification_result, "test-memory", "original").is_err());
    }
}
```

---

## 🧪 Integration Tests

### End-to-End Token Flow Test

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_token_flow() {
        // Initialize secret store
        init_secret_store().await.unwrap();

        // Setup test memory with proper permissions
        setup_test_memory_with_permissions().await;

        // Mint token
        let mint_result = mint_http_token(
            "test-memory".to_string(),
            vec!["thumbnail".to_string(), "preview".to_string()],
            180,
        ).await.unwrap();

        // Verify token
        let verification_result = verify_http_token(&mint_result.token).await.unwrap();

        // Validate scope
        assert!(validate_token_scope(&verification_result, "test-memory", "thumbnail").is_ok());
        assert!(validate_token_scope(&verification_result, "test-memory", "preview").is_ok());
        assert!(validate_token_scope(&verification_result, "test-memory", "original").is_err());

        // Test expiration
        assert!(verification_result.expires_at > ic_cdk::api::time() / 1_000_000_000);
    }
}
```

---

## 📦 Dependencies to Add

Add these to `Cargo.toml`:

```toml
[dependencies]
hmac = "0.12"
sha2 = "0.10"
base64 = "0.21"
serde_json = "1.0"
```

---

## ✅ Acceptance Criteria

- [ ] **Secret Management**: 32-byte secrets generated on init/upgrade
- [ ] **Token Model**: Canonical JSON payload with version, scope, expiration, nonce
- [ ] **Token Mint API**: Query method validates permissions and returns signed tokens
- [ ] **Verification Helper**: Validates signatures, expiration, and scope
- [ ] **Unit Tests**: All functions have comprehensive test coverage
- [ ] **Integration Tests**: End-to-end token flow works correctly
- [ ] **Error Handling**: Proper error messages for all failure cases
- [ ] **Performance**: Token operations complete in <10ms

---

## 🚀 Next Steps

After completing Phase 1:

1. **Code Review**: Have tech lead review implementation
2. **Performance Testing**: Benchmark token operations
3. **Security Review**: Validate HMAC implementation
4. **Phase 2**: Implement HTTP routes with token verification

---

## 📝 Notes

- **Async Functions**: All token operations are async due to `raw_rand()` calls
- **Error Messages**: Keep error messages descriptive but don't leak sensitive info
- **Memory Safety**: Use `Mutex` for thread-safe secret storage
- **Testing**: Mock `raw_rand()` for deterministic tests
- **Logging**: Add structured logging for debugging (optional in Phase 1)

This implementation provides a solid foundation for the token-gated HTTP request system and follows all Phase 0 design decisions.
