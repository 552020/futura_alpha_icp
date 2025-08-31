#!/bin/bash
# Test utilities and helper functions for personal canister creation integration tests

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_TIMEOUT=30
CANISTER_NAME="backend"
DFX_NETWORK="local"

# Global test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Test assertion functions
assert_success() {
    local result=$1
    local message=$2
    
    TESTS_RUN=$((TESTS_RUN + 1))
    
    if [ $result -eq 0 ]; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_success "✓ $message"
        return 0
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        log_error "✗ $message"
        return 1
    fi
}

assert_failure() {
    local result=$1
    local message=$2
    
    TESTS_RUN=$((TESTS_RUN + 1))
    
    if [ $result -ne 0 ]; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_success "✓ $message"
        return 0
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        log_error "✗ $message"
        return 1
    fi
}

assert_equals() {
    local expected="$1"
    local actual="$2"
    local message="$3"
    
    TESTS_RUN=$((TESTS_RUN + 1))
    
    if [ "$expected" = "$actual" ]; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_success "✓ $message"
        return 0
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        log_error "✗ $message (expected: '$expected', actual: '$actual')"
        return 1
    fi
}

assert_contains() {
    local haystack="$1"
    local needle="$2"
    local message="$3"
    
    TESTS_RUN=$((TESTS_RUN + 1))
    
    if [[ "$haystack" == *"$needle"* ]]; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_success "✓ $message"
        return 0
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        log_error "✗ $message (haystack: '$haystack', needle: '$needle')"
        return 1
    fi
}

assert_not_contains() {
    local haystack="$1"
    local needle="$2"
    local message="$3"
    
    TESTS_RUN=$((TESTS_RUN + 1))
    
    if [[ "$haystack" != *"$needle"* ]]; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_success "✓ $message"
        return 0
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        log_error "✗ $message (haystack: '$haystack', needle: '$needle')"
        return 1
    fi
}

# DFX helper functions
dfx_call() {
    local method="$1"
    local args="$2"
    local timeout="${3:-$TEST_TIMEOUT}"
    
    log_info "Calling: dfx canister call $CANISTER_NAME $method $args --network $DFX_NETWORK"
    
    if [ -n "$args" ]; then
        timeout $timeout dfx canister call $CANISTER_NAME $method "$args" --network $DFX_NETWORK 2>&1
    else
        timeout $timeout dfx canister call $CANISTER_NAME $method --network $DFX_NETWORK 2>&1
    fi
}

dfx_call_query() {
    local method="$1"
    local args="$2"
    local timeout="${3:-$TEST_TIMEOUT}"
    
    log_info "Querying: dfx canister call $CANISTER_NAME $method $args --query --network $DFX_NETWORK"
    
    if [ -n "$args" ]; then
        timeout $timeout dfx canister call $CANISTER_NAME $method "$args" --query --network $DFX_NETWORK 2>&1
    else
        timeout $timeout dfx canister call $CANISTER_NAME $method --query --network $DFX_NETWORK 2>&1
    fi
}

# JSON parsing helpers
extract_json_field() {
    local json="$1"
    local field="$2"
    
    echo "$json" | jq -r ".$field" 2>/dev/null || echo ""
}

extract_candid_field() {
    local candid_output="$1"
    local field="$2"
    
    # Simple extraction for Candid format - this is basic and may need enhancement
    echo "$candid_output" | grep -o "$field = [^;,}]*" | cut -d'=' -f2 | xargs
}

# Test environment setup
setup_test_environment() {
    log_info "Setting up test environment..."
    
    # Check if dfx is available
    if ! command -v dfx &> /dev/null; then
        log_error "dfx CLI not found. Please install dfx."
        exit 1
    fi
    
    # Check if jq is available
    if ! command -v jq &> /dev/null; then
        log_error "jq not found. Please install jq for JSON parsing."
        exit 1
    fi
    
    # Check if local replica is running
    if ! dfx ping --network $DFX_NETWORK &> /dev/null; then
        log_error "Local replica not running. Please start with 'dfx start'."
        exit 1
    fi
    
    # Check if backend canister is deployed
    if ! dfx canister status $CANISTER_NAME --network $DFX_NETWORK &> /dev/null; then
        log_error "Backend canister not deployed. Please deploy with 'dfx deploy'."
        exit 1
    fi
    
    log_success "Test environment ready"
}

# Test cleanup
cleanup_test_environment() {
    log_info "Cleaning up test environment..."
    # Add any cleanup logic here
    log_success "Cleanup completed"
}

# Test reporting
print_test_summary() {
    echo ""
    echo "=================================="
    echo "Test Summary"
    echo "=================================="
    echo "Tests Run: $TESTS_RUN"
    echo "Tests Passed: $TESTS_PASSED"
    echo "Tests Failed: $TESTS_FAILED"
    
    if [ $TESTS_FAILED -eq 0 ]; then
        log_success "All tests passed! 🎉"
        return 0
    else
        log_error "$TESTS_FAILED test(s) failed"
        return 1
    fi
}

# Principal generation for testing
generate_test_principal() {
    local id="$1"
    # Generate a test principal - this is a simple approach
    # In real tests, you might want to use actual identities
    echo "rdmx6-jaaaa-aaaah-qcaiq-cai"
}

# Wait for condition with timeout
wait_for_condition() {
    local condition_cmd="$1"
    local timeout="${2:-30}"
    local interval="${3:-1}"
    local elapsed=0
    
    while [ $elapsed -lt $timeout ]; do
        if eval "$condition_cmd"; then
            return 0
        fi
        sleep $interval
        elapsed=$((elapsed + interval))
    done
    
    return 1
}

# Mock data generators
generate_mock_capsule_data() {
    cat << EOF
{
    "memories": [
        {
            "id": "memory_1",
            "content": "Test memory content 1",
            "timestamp": 1234567890
        },
        {
            "id": "memory_2", 
            "content": "Test memory content 2",
            "timestamp": 1234567891
        }
    ],
    "connections": [
        {
            "from": "memory_1",
            "to": "memory_2",
            "strength": 0.8
        }
    ],
    "metadata": {
        "version": "1.0",
        "created_at": 1234567890,
        "updated_at": 1234567891
    }
}
EOF
}

# Export functions for use in test scripts
export -f log_info log_success log_warning log_error
export -f assert_success assert_failure assert_equals assert_contains assert_not_contains
export -f dfx_call dfx_call_query
export -f extract_json_field extract_candid_field
export -f setup_test_environment cleanup_test_environment print_test_summary
export -f generate_test_principal wait_for_condition generate_mock_capsule_data