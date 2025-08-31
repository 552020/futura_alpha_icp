# Backend Test Scripts

This directory contains bash test scripts for testing the backend canister functionality.

## Directory Structure

- `test_config.sh` - Shared test configuration
- `test_utils.sh` - Shared test utilities
- `memories/` - Memory-related test scripts

## Setup

1. **Deploy your canisters first:**

   ```bash
   dfx start --background
   dfx deploy
   ```

2. **Configure test settings:**
   Edit `test_config.sh` and set your canister IDs:

   ```bash
   # Get your backend canister ID
   dfx canister id backend

   # Set it in test_config.sh
   export BACKEND_CANISTER_ID="your-backend-canister-id"
   ```

3. **Make sure you have a registered user:**
   ```bash
   # Register yourself as a user (if needed)
   dfx canister call backend register
   ```

## Available Tests

### Memory Upload Tests (`memories/test_memory_upload.sh`)

Tests the memory upload and retrieval functionality:

- **Text Memory Upload**: Tests uploading text-based memories (notes)
- **Image Memory Upload**: Tests uploading image memories with binary data
- **Document Memory Upload**: Tests uploading document memories (PDF)
- **Metadata Validation**: Tests that invalid memory data is properly rejected
- **Memory Retrieval**: Tests retrieving uploaded memories by ID
- **Non-existent Memory**: Tests handling of requests for non-existent memories
- **Storage Persistence**: Tests that memories persist across multiple retrievals
- **Large Memory Upload**: Tests uploading larger memory content

**Usage:**

```bash
./memories/test_memory_upload.sh
```

## Test Structure

Each test script follows this pattern:

1. **Setup**: Load configuration and utilities
2. **Helper Functions**: Create test data and utility functions
3. **Test Functions**: Individual test cases that return 0 for pass, 1 for fail
4. **Main Execution**: Run all tests and provide summary

## Test Data

The tests use minimal test data:

- **Text**: Simple string content encoded as base64
- **Images**: 1x1 PNG image (minimal valid PNG)
- **Documents**: Minimal valid PDF document
- **Binary Data**: Base64 encoded for Candid compatibility

## Troubleshooting

### Common Issues

1. **"BACKEND_CANISTER_ID not set"**

   - Solution: Set the canister ID in `test_config.sh`

2. **"dfx command not found"**

   - Solution: Install dfx and ensure it's in your PATH

3. **"Canister not found"**

   - Solution: Make sure your canisters are deployed with `dfx deploy`

4. **"Unauthorized" errors**
   - Solution: Make sure you're registered as a user with `dfx canister call backend register`

### Debug Mode

To see detailed dfx output, modify the test scripts to remove `2>/dev/null` from dfx calls.

## Adding New Tests

To add new test scripts:

1. Create a new script following the naming pattern `test_*.sh`
2. Source the configuration and utilities:
   ```bash
   source "$SCRIPT_DIR/../test_config.sh"
   source "$SCRIPT_DIR/../test_utils.sh"
   ```
3. Follow the established test pattern with proper error handling
4. Make the script executable: `chmod +x test_new_feature.sh`
