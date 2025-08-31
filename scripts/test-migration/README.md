# Personal Canister Creation Integration Tests

This directory contains comprehensive bash integration tests for the personal canister creation functionality.

## Directory Structure

- `utils/` - Test utilities and helper functions
- `data/` - Test data and fixtures
- `tests/` - Individual test suites
- `config/` - Test configuration files
- `run-tests.sh` - Main test runner script

## Usage

```bash
# Run all tests
./scripts/test-migration/run-tests.sh

# Run specific test suite
./scripts/test-migration/tests/test-api-endpoints.sh

# Run tests with verbose output
./scripts/test-migration/run-tests.sh --verbose

# Run tests with cleanup
./scripts/test-migration/run-tests.sh --cleanup
```

## Test Suites

1. **API Endpoints** - Test individual API endpoints
2. **State Transitions** - Test creation state machine
3. **Error Conditions** - Test error handling and edge cases
4. **Data Integrity** - Test data export/import integrity
5. **Admin Functions** - Test admin controls and monitoring
6. **Feature Flags** - Test feature flag functionality

## Requirements

- dfx CLI tool
- jq for JSON parsing
- Internet Computer local replica running
- Backend canister deployed
