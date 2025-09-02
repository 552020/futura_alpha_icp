#!/bin/bash

# Test capsules_list endpoint functionality
# Tests the capsules_list function that replaces list_my_capsules

# Load test configuration and utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_config.sh"
source "$SCRIPT_DIR/../test_utils.sh"

# Test configuration
TEST_NAME="Capsules List Tests"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Helper function to check if response contains capsule data
has_capsules() {
    local response="$1"
    echo "$response" | grep -q "record {"
}

# Helper function to check if response is empty (no capsules)
is_empty_response() {
    local response="$1"
    echo "$response" | grep -q "vec {}"
}

# Helper function to check if response contains expected field hashes
has_expected_fields() {
    local response="$1"
    # Check for common Candid field hashes that indicate a valid capsule
    # These are the hash values from the actual response
    echo "$response" | grep -q "23_515 = " && \
    echo "$response" | grep -q "696_779_180 = " && \
    echo "$response" | grep -q "1_116_327_043 = "
}

# Helper function to extract capsule count from response
extract_capsule_count() {
    local response="$1"
    echo "$response" | grep -o "vec { [^}]* }" | tr -cd ';' | wc -c
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

# Test setup - ensure user is registered and has a capsule
test_setup_user_and_capsule() {
    echo_info "Setting up test user and capsule..."
    
    # Register user
    local register_result=$(dfx canister call backend register 2>/dev/null)
    if ! echo "$register_result" | grep -q "true"; then
        echo_warn "User registration failed, continuing with existing user..."
    fi
    
    # Mark capsule as bound to Web2
    local bind_result=$(dfx canister call backend mark_bound 2>/dev/null)
    if ! echo "$bind_result" | grep -q "true"; then
        echo_warn "Capsule binding failed, continuing..."
    fi
    
    echo_info "Setup complete"
}

# Test 1: Basic capsules_list call
test_basic_capsules_list() {
    echo_info "Testing basic capsules_list functionality..."
    
    local response=$(dfx canister call backend capsules_list 2>/dev/null)
    
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
    
    local response=$(dfx canister call backend capsules_list 2>/dev/null)
    
    if has_capsules "$response" || is_empty_response "$response"; then
        echo_pass "Response format is valid"
        return 0
    else
        echo_fail "Response format is invalid"
        echo_info "Response: $response"
        return 1
    fi
}

# Test 3: Verify function returns expected data structure
test_data_structure() {
    echo_info "Testing data structure of capsules_list response..."
    
    local response=$(dfx canister call backend capsules_list 2>/dev/null)
    
    # Check if response contains expected fields (if not empty)
    if echo "$response" | grep -q "vec {}"; then
        echo_pass "Empty response is valid for user with no capsules"
        return 0
    elif has_expected_fields "$response"; then
        echo_pass "Response contains expected capsule fields"
        return 0
    else
        echo_fail "Response missing expected capsule fields"
        echo_info "Response: $response"
        return 1
    fi
}

# Test 4: Test with authenticated user
test_authenticated_user() {
    echo_info "Testing capsules_list with authenticated user..."
    
    # Ensure we're using the current identity
    local current_principal=$(dfx identity get-principal)
    echo_info "Current principal: $current_principal"
    
    local response=$(dfx canister call backend capsules_list 2>/dev/null)
    
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
    
    local response=$(dfx canister call backend capsules_list 2>/dev/null)
    
    # Check if response contains expected fields (if not empty)
    if ! is_empty_response "$response"; then
        if has_expected_fields "$response"; then
            echo_pass "Response contains expected capsule fields"
            return 0
        else
            echo_fail "Response missing expected capsule fields"
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
    test_setup_user_and_capsule
    
    # Run tests
    run_test "Basic capsules_list call" test_basic_capsules_list
    run_test "Response format validation" test_response_format
    run_test "Data structure validation" test_data_structure
    run_test "Authenticated user access" test_authenticated_user
    run_test "Response structure validation" test_response_structure
    
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
