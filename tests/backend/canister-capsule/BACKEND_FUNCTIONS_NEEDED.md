# Backend Functions Needed for Personal Canister Management

## Current Functions (Already Implemented)

- `is_personal_canister_creation_enabled()` - Check if creation is enabled
- `create_personal_canister()` - Create a new personal canister
- `get_creation_status()` - Get basic creation status
- `get_detailed_creation_status()` - Get detailed creation status
- `get_my_personal_canister_id()` - Get current user's personal canister ID
- `get_personal_canister_id(principal)` - Get personal canister ID for a specific principal
- `get_personal_canister_creation_stats()` - Get creation statistics (admin)

## Functions That Should Be Added

### 1. List All Personal Canisters

```rust
#[ic_cdk::query]
fn list_all_personal_canisters() -> Vec<(Principal, Principal)> {
    // Returns a list of (user_principal, canister_id) pairs
    // This allows admins to see all personal canisters
}
```

### 2. List Personal Canisters by User

```rust
#[ic_cdk::query]
fn list_personal_canisters_by_user(user_principal: Principal) -> Vec<Principal> {
    // Returns all personal canisters for a specific user
    // Useful for users who might have multiple personal canisters
}
```

### 3. Get Personal Canister Details

```rust
#[ic_cdk::query]
fn get_personal_canister_details(canister_id: Principal) -> Option<PersonalCanisterDetails> {
    // Returns detailed information about a specific personal canister
    // Including creation date, owner, status, etc.
}
```

### 4. Delete Personal Canister

```rust
#[ic_cdk::update]
fn delete_personal_canister(canister_id: Principal) -> Result<(), String> {
    // Allows users to delete their personal canisters
    // Should only be callable by the canister owner
}
```

### 5. Transfer Personal Canister Ownership

```rust
#[ic_cdk::update]
fn transfer_personal_canister_ownership(
    canister_id: Principal,
    new_owner: Principal
) -> Result<(), String> {
    // Allows transferring ownership of a personal canister
    // Should only be callable by the current owner
}
```

## Data Structures Needed

```rust
#[derive(candid::CandidType, candid::Deserialize, Clone, Debug)]
pub struct PersonalCanisterDetails {
    pub canister_id: Principal,
    pub owner: Principal,
    pub created_at: u64, // timestamp
    pub status: String,
    pub cycles_balance: u64,
    pub memory_size: u64,
}

#[derive(candid::CandidType, candid::Deserialize, Clone, Debug)]
pub struct PersonalCanisterInfo {
    pub user_principal: Principal,
    pub canister_id: Principal,
    pub created_at: u64,
    pub is_active: bool,
}
```

## Implementation Priority

1. **High Priority**: `list_all_personal_canisters()` - Needed for admin management
2. **Medium Priority**: `get_personal_canister_details()` - Useful for debugging
3. **Low Priority**: `delete_personal_canister()` - For cleanup
4. **Low Priority**: `transfer_personal_canister_ownership()` - Advanced feature

## Testing

The test suite `test_canister_capsule_creation_cost.sh` already includes a test for `list_all_personal_canisters()` that will pass once the function is implemented.

