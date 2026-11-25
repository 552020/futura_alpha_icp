# Canister Factory

A production-ready canister factory for creating and managing user canisters on the Internet Computer.

## Features

- 🔒 **Security**: Admin controls, allowlists, emergency stop
- 📦 **Chunked Uploads**: Handle large WASM files efficiently
- ✅ **Hash Verification**: SHA-256 integrity checking
- 🔄 **Multiple Install Modes**: Install, Reinstall, Upgrade
- 📊 **Statistics & Monitoring**: Track usage and health
- 🧹 **Automatic Cleanup**: TTL-based upload expiration
- 💰 **Cycles Management**: Configurable balance limits
- 🛡️ **Rate Limiting**: Per-caller quotas

## Quick Start

### 1. Deploy the Factory

```bash
# Deploy locally
dfx deploy canister_factory --argument '(
  opt record {
    max_canisters_per_caller = opt (50 : nat32);
    min_factory_cycles = opt (10_000_000_000_000 : nat);
    admins = opt vec { principal "your-admin-principal" };
    max_upload_size = opt (100_000_000 : nat64);
    upload_ttl_seconds = opt (86400 : nat64);
  }
)'

# Or with defaults
dfx deploy canister_factory
```

### 2. Upload WASM File

```bash
# Create upload session
dfx canister call canister_factory create_upload

# Upload chunks (repeat for each chunk)
dfx canister call canister_factory put_chunk '(1, blob "wasm_chunk_data")'

# Commit with hash
dfx canister call canister_factory commit_upload '(
  1,
  record { expected_sha256_hex = "your_calculated_sha256_hex" }
)'
```

### 3. Create User Canister

```bash
dfx canister call canister_factory create_and_install_with '(
  record {
    upload_id = 1 : nat64;
    extra_controllers = null;
    init_arg = blob "candid_encoded_init_args";
    mode = variant { Install };
    cycles = 1_000_000_000_000 : nat;
    handoff = true;
  }
)'
```

## Configuration Options

| Parameter                  | Type            | Default  | Description                   |
| -------------------------- | --------------- | -------- | ----------------------------- |
| `max_canisters_per_caller` | `nat32`         | 100      | Max canisters per user        |
| `min_factory_cycles`       | `nat`           | 5T       | Min cycles to keep in factory |
| `allowlist`                | `vec principal` | None     | Allowed callers (if set)      |
| `admins`                   | `vec principal` | Deployer | Factory administrators        |
| `max_upload_size`          | `nat64`         | 50MB     | Max WASM file size            |
| `upload_ttl_seconds`       | `nat64`         | 24h      | Upload expiration time        |

## Admin Functions

```bash
# Emergency stop
dfx canister call canister_factory set_emergency_stop '(true)'

# Update allowlist
dfx canister call canister_factory set_allowlist '(
  opt vec {
    principal "allowed-user-1";
    principal "allowed-user-2";
  }
)'

# Add admin
dfx canister call canister_factory add_admin '(principal "new-admin")'

# Manual cleanup
dfx canister call canister_factory cleanup_expired_uploads_manual
```

## Monitoring

```bash
# Check factory health
dfx canister call canister_factory health_check

# Get factory statistics
dfx canister call canister_factory get_factory_stats

# Check personal usage
dfx canister call canister_factory my_stats

# View configuration
dfx canister call canister_factory get_config
```

## Cycles top-up of personal canisters

After the hand-off the factory is not a controller of a personal canister. It
can still keep it alive: the canister reports its cycles to the factory that
installed it, and depositing cycles needs no rights.

- Every hour a timer looks whether the last check is a day old; if so, the
  factory asks each `Ready` canister for `cycles_liquid_balance` (the cycles it
  can spend before it freezes).
- Below the floor (default 1T) it deposits up to the target (default 2T).
- Per owner it sends at most the monthly cap (default 3T per calendar month,
  UTC), and it never goes below its own `min_factory_cycles`.
- A frozen canister cannot answer; if it reported before, it counts as empty
  and is topped up.
- A canister whose owner installed other code can report anything. It still
  gets no more than its monthly cap.

```bash
# Settings (admin). enabled = false stops the timer; the calls below still work
dfx canister call canister_factory top_up_config
dfx canister call canister_factory top_up_config_set \
  '(record { enabled = true; floor = 1_000_000_000_000; target = 2_000_000_000_000; monthly_cap = 3_000_000_000_000 })'

# Another cap for one owner (admin); null = default, 0 = no top-ups
dfx canister call canister_factory top_up_cap_set '(principal "<owner>", opt 10_000_000_000_000)'

# Check now: one owner (admin, or the owner for themselves), or all (admin)
dfx canister call canister_factory top_up_check '(principal "<owner>")'
dfx canister call canister_factory top_up_check_all

# What happened: last balance, last top-up, this month's total, last error
dfx canister call canister_factory personal_canisters_list
```

Personal canisters installed before the backend remembered its factory do not
answer the factory. Their owner names it once:

```bash
dfx canister call <personal canister> personal_factory_set '(opt principal "<factory>")'
```

Tests: `tests/pocket-ic/top_up_pocket_ic.rs` (timer, month change, upgrade,
frozen canister, on PocketIC) and `tests/backend/personal/test_top_up.sh` (on
the local replica).

## Integration Example

For your family memory app, here's how to integrate the factory:

```rust
// In your main backend canister
use candid::{encode_one, Principal};

pub async fn create_user_canister(
    factory_id: Principal,
    user_profile: UserProfile,
) -> Result<Principal, String> {
    // 1. Upload user canister WASM (done once, reuse upload_id)
    let upload_id = 1u64; // Pre-uploaded user canister WASM

    // 2. Prepare init args for user canister
    let init_arg = encode_one(UserCanisterInit {
        owner: ic_cdk::caller(),
        profile: user_profile,
    })?;

    // 3. Create the canister
    let request = CreateInstallRequest {
        upload_id,
        extra_controllers: None,
        init_arg,
        mode: Mode::Install,
        cycles: 1_000_000_000_000, // 1T cycles
        handoff: true,
    };

    let response: CreateInstallResponse = ic_cdk::call(
        factory_id,
        "create_and_install_with",
        (request,)
    ).await?;

    Ok(response.canister_id)
}
```

## Security Best Practices

1. **Set Admin List**: Always configure admins on deployment
2. **Use Allowlists**: For controlled beta releases
3. **Monitor Cycles**: Set appropriate minimum balance
4. **Regular Cleanup**: Monitor upload storage usage
5. **Emergency Stop**: Have a plan for emergency situations

## Error Handling

Common errors and solutions:

- `"Factory low on cycles"`: Top up factory canister
- `"Per-caller canister limit reached"`: Increase limit or clean up old canisters
- `"Upload too large"`: Check file size limits
- `"hash mismatch"`: Verify SHA-256 calculation
- `"Caller not in allowlist"`: Add caller to allowlist

## Development

```bash
# Build
cargo build --target wasm32-unknown-unknown --release

# Test
cargo test

# Generate Candid
dfx generate canister_factory
```

## Production Deployment

For mainnet deployment:

1. Set proper admin principals
2. Configure reasonable quotas
3. Fund with sufficient cycles
4. Set up monitoring alerts
5. Plan upgrade procedures

## License

This canister factory is part of the Futura family memory sharing application.
