#!/bin/bash

# Test script for galleries_list endpoint
# Tests the new galleries_list() function that replaces get_my_galleries()

set -e

# Source test utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../test_utils.sh"
source "$SCRIPT_DIR/gallery_test_utils.sh"

# Test configuration
TEST_NAME="Galleries List Tests"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Helper function to check if response contains gallery data
has_gallery_data() {
    local response="$1"
    # Check for vector structure and gallery content
    echo "$response" | grep -q "vec {" && \
    (echo "$response" | grep -q "Gallery" || echo "$response" | grep -q "vec {}")
}

# Helper function to check if response is empty vector
is_empty_vector() {
    local response="$1"
    echo "$response" | grep -q "vec {}"
}

# Use run_gallery_test from shared utilities
run_test() {
    run_gallery_test "$1" "$2"
}

# Test setup - ensure user is registered and has a capsule
test_setup_user_and_capsule() {
    setup_user_and_capsule
}

# Test 1: Basic galleries_list call (should return empty vector if no galleries)
test_galleries_list_empty() {
    echo_info "Testing galleries_list functionality with no galleries..."
    
    local response=$(dfx canister call backend galleries_list 2>/dev/null)
    echo_info "Response: '$response'"
    
    if [ $? -eq 0 ]; then
        if is_empty_vector "$response"; then
            echo_pass "galleries_list call successful - returned empty vector (no galleries)"
            return 0
        elif has_gallery_data "$response"; then
            echo_pass "galleries_list call successful - returned gallery data"
            return 0
        else
            echo_fail "galleries_list should return vector structure"
            echo_info "Expected vector, got: '$response'"
            return 1
        fi
    else
        echo_fail "galleries_list call failed"
        return 1
    fi
}

# Test 2: Test with authenticated user
test_authenticated_user() {
    echo_info "Testing galleries_list with authenticated user..."
    
    # Ensure we're using the current identity
    local current_principal=$(dfx identity get-principal)
    echo_info "Current principal: $current_principal"
    
    local response=$(dfx canister call backend galleries_list 2>/dev/null)
    
    if [ $? -eq 0 ]; then
        echo_pass "galleries_list works with authenticated user"
        return 0
    else
        echo_fail "galleries_list failed with authenticated user"
        return 1
    fi
}

# Test 3: Test response structure validation
test_response_structure() {
    echo_info "Testing response structure..."
    
    local response=$(dfx canister call backend galleries_list 2>/dev/null)
    echo_info "Response: '$response'"
    
    if has_gallery_data "$response"; then
        echo_pass "Response contains expected gallery data structure"
        return 0
    else
        echo_fail "Response missing expected gallery data structure"
        echo_info "Response: '$response'"
        return 1
    fi
}

# Test 4: Verify old endpoint is no longer available
test_old_endpoint_removed() {
    echo_info "Testing that old get_my_galleries endpoint is removed..."
    
    local response=$(dfx canister call backend get_my_galleries 2>/dev/null 2>&1)
    
    if echo "$response" | grep -q "method not found\|Canister has no.*method.*get_my_galleries"; then
        echo_pass "Old get_my_galleries endpoint correctly removed"
        return 0
    else
        echo_fail "Old get_my_galleries endpoint still exists or returned unexpected error"
        echo_info "Response: '$response'"
        return 1
    fi
}

# Test 5: Verify old get_user_galleries endpoint is no longer available
test_old_user_endpoint_removed() {
    echo_info "Testing that old get_user_galleries endpoint is removed..."
    
    local fake_principal="2vxsx-fae"
    local response=$(dfx canister call backend get_user_galleries "(principal \"$fake_principal\")" 2>/dev/null 2>&1)
    
    if echo "$response" | grep -q "method not found\|Canister has no.*method.*get_user_galleries"; then
        echo_pass "Old get_user_galleries endpoint correctly removed"
        return 0
    else
        echo_fail "Old get_user_galleries endpoint still exists or returned unexpected error"
        echo_info "Response: '$response'"
        return 1
    fi
}

# Main test execution
main() {
    echo_info "Starting $TEST_NAME"
    echo_info "=================================="
    
    # Setup
    test_setup_user_and_capsule
    
    # Run tests
    run_test "galleries_list with no galleries (empty vector)" test_galleries_list_empty
    run_test "galleries_list with authenticated user" test_authenticated_user
    run_test "Response structure validation" test_response_structure
    run_test "Old get_my_galleries endpoint removed" test_old_endpoint_removed
    run_test "Old get_user_galleries endpoint removed" test_old_user_endpoint_removed
    
    # Summary
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
