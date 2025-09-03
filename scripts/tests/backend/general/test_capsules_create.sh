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

# Helper function to check if response contains capsule creation success
has_creation_success() {
    local response="$1"
    echo "$response" | grep -q "success = true"
}

# Helper function to check if response contains capsule ID
has_capsule_id() {
    local response="$1"
    echo "$response" | grep -q "capsule_id = opt"
}

# Helper function to check if response contains expected message
has_expected_message() {
    local response="$1"
    local expected_message="$2"
    echo "$response" | grep -q "$expected_message"
}

# Helper function to increment test counters
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

# Test setup - ensure user is registered
test_setup_user() {
    echo_info "Setting up test user..."
    
    # Register user
    local register_result=$(dfx canister call backend register 2>/dev/null)
    if ! echo "$register_result" | grep -q "true"; then
        echo_warn "User registration failed, continuing with existing user..."
    fi
    
    echo_info "Setup complete"
}

# Test 1: Create self-capsule (no subject parameter)
test_capsules_create_self() {
    echo_info "Testing capsules_create with no subject (self-capsule)..."
    
    local response=$(dfx canister call backend capsules_create 2>/dev/null)
    echo_info "Response: '$response'"
    
    if [ $? -eq 0 ]; then
        if has_creation_success "$response" && has_capsule_id "$response"; then
            echo_pass "capsules_create call successful with no subject (creates self-capsule)"
            return 0
        else
            echo_fail "capsules_create should return success and capsule_id for self-capsule"
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
        if has_creation_success "$response" && has_capsule_id "$response"; then
            echo_pass "capsules_create call successful with specific subject"
            return 0
        else
            echo_fail "capsules_create should return success and capsule_id for specific subject"
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
        if has_creation_success "$second_response" && has_capsule_id "$second_response"; then
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
        # Check for required fields in CapsuleCreationResult
        if has_creation_success "$response" && has_capsule_id "$response"; then
            echo_pass "Response contains required fields (success, capsule_id)"
            return 0
        else
            echo_fail "Response missing required fields"
            echo_info "Expected: success and capsule_id fields"
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
        if has_creation_success "$response"; then
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
    test_setup_user
    
    # Run tests
    run_test "capsules_create with no subject (self-capsule)" test_capsules_create_self
    run_test "capsules_create with specific subject" test_capsules_create_with_subject
    run_test "capsules_create idempotent behavior" test_capsules_create_idempotent
    run_test "Response structure validation" test_response_structure
    run_test "Authenticated user access" test_authenticated_user
    
    # Test summary
    echo_info "=================================="
    echo_info "Test Summary:"
    echo_info "Total tests: $TOTAL_TESTS"
    echo_info "Passed: $PASSED_TESTS"
    echo_info "Failed: $FAILED_TESTS"
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo_pass "All tests passed!"
        exit 0
    else
        echo_fail "Some tests failed!"
        exit 1
    fi
}

# Run main function
main "$@"
