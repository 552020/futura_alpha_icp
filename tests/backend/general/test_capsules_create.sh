#!/bin/bash

# Test script for capsules_create endpoint
# Tests both self-capsule creation (no subject) and specific subject creation

# Load test configuration and utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_config.sh"
source "$SCRIPT_DIR/../test_utils.sh"

# Test configuration
TEST_NAME="Capsules Create Tests"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Helper function to check if response contains expected message
has_expected_message() {
    local response="$1"
    local expected_message="$2"
    echo "$response" | grep -q "$expected_message"
}

# Test 1: Create self-capsule (no subject parameter)
test_capsules_create_self() {
    echo_info "Testing capsules_create with no subject (self-capsule)..."
    
    local response=$(dfx canister call backend capsules_create 2>/dev/null)
    echo_info "Response: '$response'"
    
    if [ $? -eq 0 ]; then
        if is_success "$response" && has_capsule_data "$response"; then
            echo_pass "capsules_create call successful with no subject (creates self-capsule)"
            return 0
        else
            echo_fail "capsules_create should return success and capsule data for self-capsule"
            echo_info "Response: '$response'"
            return 1
        fi
    else
        echo_fail "capsules_create call failed"
        return 1
    fi
}

# Test 2: Create capsule for specific subject
test_capsules_create_with_subject() {
    echo_info "Testing capsules_create with specific subject..."
    
    # Create a PersonRef for testing (using caller as subject for simplicity)
    local response=$(dfx canister call backend capsules_create '(opt variant { Principal = principal "2vxsx-fae" })' 2>/dev/null)
    echo_info "Response: '$response'"
    
    if [ $? -eq 0 ]; then
        if is_success "$response" && has_capsule_data "$response"; then
            echo_pass "capsules_create call successful with specific subject"
            return 0
        else
            echo_fail "capsules_create should return success and capsule data for specific subject"
            echo_info "Response: '$response'"
            return 1
        fi
    else
        echo_fail "capsules_create call failed with specific subject"
        return 1
    fi
}

# Test 3: Test idempotent behavior for self-capsule
test_capsules_create_idempotent() {
    echo_info "Testing capsules_create idempotent behavior for self-capsule..."
    
    # First call
    local first_response=$(dfx canister call backend capsules_create 2>/dev/null)
    echo_info "First call response: '$first_response'"
    
    if [ $? -ne 0 ]; then
        echo_fail "First capsules_create call failed"
        return 1
    fi
    
    # Second call (should return existing capsule)
    local second_response=$(dfx canister call backend capsules_create 2>/dev/null)
    echo_info "Second call response: '$second_response'"
    
    if [ $? -eq 0 ]; then
        if is_success "$second_response" && has_capsule_data "$second_response"; then
            echo_pass "Second capsules_create call successful (idempotent behavior)"
            return 0
        else
            echo_fail "Second capsules_create call should succeed for idempotent behavior"
            return 1
        fi
    else
        echo_fail "Second capsules_create call failed"
        return 1
    fi
}

# Test 4: Test response structure validation
test_response_structure() {
    echo_info "Testing response structure validation..."
    
    local response=$(dfx canister call backend capsules_create 2>/dev/null)
    echo_info "Response: '$response'"
    
    if [ $? -eq 0 ]; then
        # Check for required fields in Result<Capsule> format
        if is_success "$response" && has_expected_capsule_fields "$response"; then
            echo_pass "Response contains required capsule fields"
            return 0
        else
            echo_fail "Response missing required capsule fields"
            echo_info "Expected: Ok variant with capsule fields"
            echo_info "Got: '$response'"
            return 1
        fi
    else
        echo_fail "capsules_create call failed during structure validation"
        return 1
    fi
}

# Test 5: Test with authenticated user
test_authenticated_user() {
    echo_info "Testing capsules_create with authenticated user..."
    
    local response=$(dfx canister call backend capsules_create 2>/dev/null)
    echo_info "Response: '$response'"
    
    if [ $? -eq 0 ]; then
        if is_success "$response"; then
            echo_pass "capsules_create call successful with authenticated user"
            return 0
        else
            echo_fail "capsules_create should succeed with authenticated user"
            return 1
        fi
    else
        echo_fail "capsules_create call failed with authenticated user"
        return 1
    fi
}

# Main test execution
main() {
    echo_info "Starting $TEST_NAME"
    echo_info "=================================="
    
    # Setup
    setup_user_and_capsule
    
    # Run tests
    run_capsule_test "capsules_create with no subject (self-capsule)" test_capsules_create_self
    run_capsule_test "capsules_create with specific subject" test_capsules_create_with_subject
    run_capsule_test "capsules_create idempotent behavior" test_capsules_create_idempotent
    run_capsule_test "Response structure validation" test_response_structure
    run_capsule_test "Authenticated user access" test_authenticated_user
    
    # Test summary
    print_test_summary
}

# Run main function
main "$@"
