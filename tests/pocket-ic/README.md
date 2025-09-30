# PocketIC Integration Tests

This folder contains scripts and documentation for running PocketIC integration tests.

## Why PocketIC Tests Are Slow

PocketIC tests are inherently slower than unit tests due to several factors:

1. **PocketIC Server Startup**: Each test starts a new PocketIC server instance
2. **WASM Compilation**: The backend WASM needs to be compiled and loaded
3. **Canister Installation**: Each test creates, installs, and initializes a new canister
4. **Network Simulation**: PocketIC simulates the entire ICP network stack
5. **Memory Management**: PocketIC can have memory leaks if not properly cleaned up

## Performance Optimization

### Use the Optimized Test Runner

```bash
# Run all PocketIC tests
./tests/pocket-ic/run_pocket_ic_tests.sh

# Run a specific test
./tests/pocket-ic/run_pocket_ic_tests.sh test_create_and_read_memory_happy_path
```

### Manual Optimization

If you need to run tests manually, use these settings:

```bash
# Build in release mode for better performance
cargo build --release --package backend

# Run with optimal settings
cargo test --release \
    --test-threads=1 \
    --package backend \
    --test memories_pocket_ic \
    -- \
    --nocapture
```

### Key Settings Explained

- `--release`: Uses optimized build (significantly faster execution)
- `--test-threads=1`: Prevents PocketIC server conflicts
- `--package backend`: Only runs backend tests
- `--test memories_pocket_ic`: Only runs PocketIC tests
- `--nocapture`: Shows output for debugging

## Troubleshooting

### Tests Getting Stuck

If tests appear to hang:

1. **Check for compilation errors**: Make sure the project compiles first
2. **Use timeout**: Add `timeout 60s` before the cargo test command
3. **Check system resources**: PocketIC is memory-intensive
4. **Run single tests**: Test one function at a time to isolate issues

### Common Issues

1. **"Fail to decode argument"**: Candid type mismatch - check argument encoding
2. **"Unauthorized"**: User doesn't have access to capsule - create capsule first
3. **"NotFound"**: Memory/capsule doesn't exist - check creation logic
4. **Controller permission errors**: Use `None` as sender in `install_canister`

## Test Structure

- `memories_pocket_ic.rs`: Main test file with integration tests
- `run_pocket_ic_tests.sh`: Optimized test runner script
- `README.md`: This documentation

## Best Practices

1. **Use helper functions**: Avoid code duplication in tests
2. **Create capsules first**: Always create a capsule before creating memories
3. **Use proper Candid encoding**: Encode arguments individually, not as tuples
4. **Handle errors gracefully**: Don't panic on expected errors
5. **Clean up resources**: Let PocketIC clean up automatically (don't share instances)

## Performance Expectations

- **Unit tests**: ~1-5 seconds total
- **PocketIC tests**: ~30-60 seconds per test
- **Full test suite**: ~5-10 minutes

Consider running PocketIC tests separately from unit tests in CI/CD pipelines.
