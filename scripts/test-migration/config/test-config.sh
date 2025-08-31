#!/bin/bash
# Test configuration for personal canister creation integration tests

# Test environment settings
export TEST_NETWORK="local"
export TEST_CANISTER="backend"
export TEST_TIMEOUT=30
export TEST_VERBOSE=${TEST_VERBOSE:-false}
export TEST_CLEANUP=${TEST_CLEANUP:-false}

# Test data settings
export TEST_DATA_DIR="$(dirname "$0")/../data"
export TEST_RESULTS_DIR="$(dirname "$0")/../results"

# Cycles settings for testing
export TEST_CYCLES_RESERVE=10000000000000  # 10T cycles
export TEST_CYCLES_THRESHOLD=2000000000000 # 2T cycles
export TEST_CYCLES_REQUIRED=1000000000000  # 1T cycles for creation

# Test user principals (mock)
export TEST_USER_1="rdmx6-jaaaa-aaaah-qcaiq-cai"
export TEST_USER_2="rrkah-fqaaa-aaaah-qcaiq-cai"
export TEST_ADMIN_USER="rjmov-eqaaa-aaaah-qcaiq-cai"

# API endpoint timeouts
export API_TIMEOUT_SHORT=10
export API_TIMEOUT_MEDIUM=30
export API_TIMEOUT_LONG=60

# Test feature flags
export TEST_MIGRATION_FEATURE=true
export TEST_PERSONAL_CANISTER_CREATION_FEATURE=true

# Logging settings
export LOG_LEVEL=${LOG_LEVEL:-"INFO"}
export LOG_FILE="${TEST_RESULTS_DIR}/test.log"

# Create results directory if it doesn't exist
mkdir -p "$TEST_RESULTS_DIR"

# Test suite configuration
declare -A TEST_SUITES
TEST_SUITES[api-endpoints]="Test individual API endpoints"
TEST_SUITES[state-transitions]="Test creation state machine"
TEST_SUITES[error-conditions]="Test error handling and edge cases"
TEST_SUITES[data-integrity]="Test data export/import integrity"
TEST_SUITES[admin-functions]="Test admin controls and monitoring"
TEST_SUITES[feature-flags]="Test feature flag functionality"

export TEST_SUITES

# Test data files
export MOCK_CAPSULE_DATA_FILE="${TEST_DATA_DIR}/mock-capsule-data.json"
export MOCK_EXPORT_MANIFEST_FILE="${TEST_DATA_DIR}/mock-export-manifest.json"
export MOCK_MEMORY_CHUNKS_DIR="${TEST_DATA_DIR}/memory-chunks"

# Performance test settings
export PERF_TEST_ITERATIONS=10
export PERF_TEST_CONCURRENT_USERS=5
export PERF_TEST_MAX_RESPONSE_TIME=5000  # 5 seconds

# Error test settings
export ERROR_TEST_INVALID_PRINCIPALS=("invalid-principal" "too-short" "")
export ERROR_TEST_OVERSIZED_DATA_MB=200  # 200MB to test size limits

# Validation settings
export VALIDATE_CHECKSUMS=true
export VALIDATE_DATA_INTEGRITY=true
export VALIDATE_STATE_PERSISTENCE=true

# Cleanup settings
export CLEANUP_TEMP_FILES=true
export CLEANUP_TEST_CANISTERS=true
export CLEANUP_LOG_FILES=false

# Debug settings
export DEBUG_MODE=${DEBUG_MODE:-false}
export TRACE_DFX_CALLS=${TRACE_DFX_CALLS:-false}
export SAVE_INTERMEDIATE_RESULTS=${SAVE_INTERMEDIATE_RESULTS:-false}