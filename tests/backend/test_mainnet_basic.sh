#!/bin/bash

# Basic Mainnet Connectivity Test
# Tests that the mainnet canister is accessible and responding

# Load test utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/test_utils.sh"

# Test configuration
TEST_NAME="Mainnet Basic Connectivity Tests"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Mainnet canister ID from canister_ids.json
MAINNET_CANISTER_ID="izhgj-eiaaa-aaaaj-a2f7q-cai"

# Helper function to run a test with proper counting
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo_info "Running: $test_name"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if eval "$test_command"; then
        echo_pass "$test_name"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo_fail "$test_name"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    echo ""
}

# Test 1: Check if mainnet canister ID is set
test_canister_id_set() {
    echo_info "Testing mainnet canister ID configuration..."
    
    if [[ -n "$MAINNET_CANISTER_ID" ]]; then
        echo_info "Mainnet canister ID: $MAINNET_CANISTER_ID"
        return 0
    else
        echo_error "Mainnet canister ID not set"
        return 1
    fi
}


# Test 2: Check if canister exists on mainnet
test_canister_exists() {
    echo_info "Testing if canister exists on mainnet..."
    
    # Check if canister exists
    if dfx canister id backend --network ic >/dev/null 2>&1; then
        local canister_id=$(dfx canister id backend --network ic)
        echo_info "Canister exists on mainnet: $canister_id"
        
        # Verify it matches our expected ID
        if [[ "$canister_id" == "$MAINNET_CANISTER_ID" ]]; then
            echo_info "Canister ID matches expected: $MAINNET_CANISTER_ID"
            return 0
        else
            echo_warn "Canister ID mismatch. Expected: $MAINNET_CANISTER_ID, Got: $canister_id"
            return 1
        fi
    else
        echo_error "Canister not found on mainnet"
        return 1
    fi
}

# Test 3: Test basic canister communication (greet function)
test_basic_communication() {
    echo_info "Testing basic canister communication..."
    
    local result=$(dfx canister call backend --network ic greet '("Basic Mainnet Test")' 2>/dev/null)
    
    if [[ $? -eq 0 ]]; then
        if echo "$result" | grep -q "Hello, Basic Mainnet Test!"; then
            echo_info "Basic communication successful: $result"
            return 0
        else
            echo_error "Unexpected response: $result"
            return 1
        fi
    else
        echo_error "Failed to communicate with canister"
        return 1
    fi
}

# Test 4: Test whoami function
test_whoami_function() {
    echo_info "Testing whoami function..."
    
    local result=$(dfx canister call backend --network ic whoami 2>/dev/null)
    
    if [[ $? -eq 0 ]]; then
        if echo "$result" | grep -q "principal"; then
            echo_info "whoami function working: $result"
            return 0
        else
            echo_error "Unexpected whoami response: $result"
            return 1
        fi
    else
        echo_error "whoami function failed"
        return 1
    fi
}

# Test 5: Test canister status
test_canister_status() {
    echo_info "Testing canister status..."
    
    local result=$(dfx canister status backend --network ic 2>/dev/null)
    
    if [[ $? -eq 0 ]]; then
        echo_info "Canister status retrieved successfully"
        echo_info "Status: $result"
        return 0
    else
        echo_error "Failed to get canister status"
        return 1
    fi
}

# Test 6: Test cycles balance
test_cycles_balance() {
    echo_info "Testing cycles balance..."
    
    local result=$(dfx cycles balance --network ic 2>/dev/null)
    
    if [[ $? -eq 0 ]]; then
        echo_info "Cycles balance retrieved: $result"
        return 0
    else
        echo_error "Failed to get cycles balance"
        return 1
    fi
}

# Main test execution
main() {
    echo_info "Starting $TEST_NAME"
    echo_info "=================================="
    echo_info "Testing mainnet canister: $MAINNET_CANISTER_ID"
    echo_info "Network: ICP Mainnet"
    echo ""
    
    # Check prerequisites
    if ! command -v dfx &> /dev/null; then
        echo_error "dfx command not found. Please install dfx first."
        exit 1
    fi
    
    # Run tests
    run_test "Canister ID configuration" "test_canister_id_set"
    run_test "Canister exists on mainnet" "test_canister_exists"
    run_test "Basic canister communication" "test_basic_communication"
    run_test "whoami function" "test_whoami_function"
    run_test "Canister status" "test_canister_status"
    run_test "Cycles balance" "test_cycles_balance"
    
    # Print test summary
    echo_info "=================================="
    echo_info "Test Summary:"
    echo_info "Total tests: $TOTAL_TESTS"
    echo_info "Passed: $PASSED_TESTS"
    echo_info "Failed: $FAILED_TESTS"
    echo ""
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo_pass "All mainnet connectivity tests passed! 🎉"
        echo_info "Mainnet canister is accessible and responding correctly."
        exit 0
    else
        echo_fail "Some mainnet connectivity tests failed!"
        echo_info "Please check the mainnet canister status and connectivity."
        exit 1
    fi
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
