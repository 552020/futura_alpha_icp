#!/bin/bash

# Test capsules_list endpoint functionality
# Tests the capsules_list function that replaces list_my_capsules

# Fix dfx color issues
export DFX_COLOR=0
export NO_COLOR=1
export TERM=dumb

# Parse command line arguments
MAINNET_MODE=false
CANISTER_ID="backend"
NETWORK_FLAG=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --mainnet)
            MAINNET_MODE=true
            CANISTER_ID="izhgj-eiaaa-aaaaj-a2f7q-cai"
            NETWORK_FLAG="--network ic"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--mainnet]"
            exit 1
            ;;
    esac
done

# Load test configuration and utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_config.sh"
source "$SCRIPT_DIR/../test_utils.sh"

# Test configuration
TEST_NAME="Capsules List Tests"
if [[ "$MAINNET_MODE" == "true" ]]; then
    TEST_NAME="Capsules List Tests (Mainnet)"
fi
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Helper function to extract capsule count from response
extract_capsule_count() {
    local response="$1"
    echo "$response" | grep -o "vec { [^}]* }" | tr -cd ';' | wc -c
}

# Test 1: Basic capsules_list call
test_basic_capsules_list() {
    echo_info "Testing basic capsules_list functionality..."
    
    local response=$(dfx canister call $CANISTER_ID capsules_list $NETWORK_FLAG 2>/dev/null)
    
    if [ $? -eq 0 ]; then
        echo_pass "capsules_list call successful"
        echo_info "Response: $response"
        return 0
    else
        echo_fail "capsules_list call failed"
        return 1
    fi
}

# Test 2: Verify response format
test_response_format() {
    echo_info "Testing response format..."
    
    local response=$(dfx canister call $CANISTER_ID capsules_list $NETWORK_FLAG 2>/dev/null)
    
    if has_capsules "$response" || is_empty_response "$response"; then
        echo_pass "Response format is valid"
        return 0
    else
        echo_fail "Response format is invalid"
        echo_info "Response: '$response'"
        return 1
    fi
}

# Test 3: Verify function returns expected data structure
test_data_structure() {
    echo_info "Testing data structure of capsules_list response..."
    
    local response=$(dfx canister call $CANISTER_ID capsules_list $NETWORK_FLAG 2>/dev/null)
    
    # Check if response contains expected fields (if not empty)
    if is_empty_response "$response"; then
        echo_pass "Empty response is valid for user with no capsules"
        return 0
    elif has_expected_capsule_header_fields "$response"; then
        echo_pass "Response contains expected capsule header fields"
        return 0
    else
        echo_fail "Response missing expected capsule header fields"
        echo_info "Response: '$response'"
        return 1
    fi
}

# Test 4: Test with authenticated user
test_authenticated_user() {
    echo_info "Testing capsules_list with authenticated user..."
    
    # Ensure we're using the current identity
    local current_principal=$(dfx identity get-principal)
    echo_info "Current principal: $current_principal"
    
    local response=$(dfx canister call $CANISTER_ID capsules_list $NETWORK_FLAG 2>/dev/null)
    
    if [ $? -eq 0 ]; then
        echo_pass "capsules_list works with authenticated user"
        return 0
    else
        echo_fail "capsules_list failed with authenticated user"
        return 1
    fi
}

# Test 5: Test response structure
test_response_structure() {
    echo_info "Testing response structure..."
    
    local response=$(dfx canister call $CANISTER_ID capsules_list $NETWORK_FLAG 2>/dev/null)
    
    # Check if response contains expected fields (if not empty)
    if ! is_empty_response "$response"; then
        if has_expected_capsule_header_fields "$response"; then
            echo_pass "Response contains expected capsule header fields"
            return 0
        else
            echo_fail "Response missing expected capsule header fields"
            echo_info "Response: '$response'"
            return 1
        fi
    else
        echo_pass "Empty response is valid for user with no capsules"
        return 0
    fi
}

# Main test execution
main() {
    echo_info "Starting $TEST_NAME"
    echo_info "=================================="
    
    # Setup
    setup_user_and_capsule
    
    # Run tests
    run_capsule_test "Basic capsules_list call" test_basic_capsules_list
    run_capsule_test "Response format validation" test_response_format
    run_capsule_test "Data structure validation" test_data_structure
    run_capsule_test "Authenticated user access" test_authenticated_user
    run_capsule_test "Response structure validation" test_response_structure
    
    # Test summary
    print_test_summary
}

# Run main function
main "$@"
