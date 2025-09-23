#!/bin/bash

# Test script for capsules_update endpoint functionality
# Tests the new capsules_update function that allows updating capsule properties

# Load test configuration and utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_config.sh"
source "$SCRIPT_DIR/../test_utils.sh"

# Test configuration
TEST_NAME="Capsules Update Tests"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Test 1: Update capsule binding status
test_capsules_update_binding() {
    echo_info "Testing capsules_update with binding status change..."
    
    # Get the current user's self-capsule ID directly
    local self_capsule_result=$(dfx canister call backend capsules_read_basic "(null)" 2>/dev/null)
    if ! is_success "$self_capsule_result"; then
        echo_fail "Failed to get self-capsule for update test"
        return 1
    fi
    
    local capsule_id=$(echo "$self_capsule_result" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')
    if [[ -z "$capsule_id" ]]; then
        echo_fail "Failed to extract capsule ID from self-capsule response"
        return 1
    fi
    
    echo_info "Testing with self-capsule ID: $capsule_id"
    
    # Create update data to bind to Neon
    local update_data=$(create_capsule_update_data "true")
    echo_info "Update data: $update_data"
    
    # Call capsules_update
    local response=$(dfx canister call backend capsules_update "(\"$capsule_id\", $update_data)" 2>/dev/null)
    echo_info "Response: '$response'"
    
    if [ $? -eq 0 ]; then
        if is_success "$response" && has_capsule_data "$response"; then
            echo_pass "capsules_update call successful with binding status change"
            return 0
        else
            echo_fail "capsules_update should return success and capsule data"
            echo_info "Response: '$response'"
            return 1
        fi
    else
        echo_fail "capsules_update call failed"
        return 1
    fi
}

# Test 2: Update capsule unbinding status
test_capsules_update_unbinding() {
    echo_info "Testing capsules_update with unbinding status change..."
    
    # Get the current user's self-capsule ID directly
    local self_capsule_result=$(dfx canister call backend capsules_read_basic "(null)" 2>/dev/null)
    if ! is_success "$self_capsule_result"; then
        echo_fail "Failed to get self-capsule for update test"
        return 1
    fi
    
    local capsule_id=$(echo "$self_capsule_result" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')
    if [[ -z "$capsule_id" ]]; then
        echo_fail "Failed to extract capsule ID from self-capsule response"
        return 1
    fi
    
    echo_info "Testing with self-capsule ID: $capsule_id"
    
    # Create update data to unbind from Neon
    local update_data=$(create_capsule_update_data "false")
    echo_info "Update data: $update_data"
    
    # Call capsules_update
    local response=$(dfx canister call backend capsules_update "(\"$capsule_id\", $update_data)" 2>/dev/null)
    echo_info "Response: '$response'"
    
    if [ $? -eq 0 ]; then
        if is_success "$response" && has_capsule_data "$response"; then
            echo_pass "capsules_update call successful with unbinding status change"
            return 0
        else
            echo_fail "capsules_update should return success and capsule data"
            echo_info "Response: '$response'"
            return 1
        fi
    else
        echo_fail "capsules_update call failed"
        return 1
    fi
}

# Test 3: Update with invalid capsule ID
test_capsules_update_invalid_id() {
    echo_info "Testing capsules_update with invalid capsule ID..."
    
    # Create update data
    local update_data=$(create_capsule_update_data "true")
    
    # Call capsules_update with invalid ID
    local response=$(dfx canister call backend capsules_update "(\"invalid_capsule_id\", $update_data)" 2>/dev/null)
    echo_info "Response: '$response'"
    
    if [ $? -eq 0 ]; then
        if is_failure "$response" && is_not_found "$response"; then
            echo_pass "capsules_update correctly returns NotFound for invalid capsule ID"
            return 0
        else
            echo_fail "capsules_update should return NotFound error for invalid ID"
            echo_info "Response: '$response'"
            return 1
        fi
    else
        echo_fail "capsules_update call failed"
        return 1
    fi
}

# Test 4: Update with empty capsule ID
test_capsules_update_empty_id() {
    echo_info "Testing capsules_update with empty capsule ID..."
    
    # Create update data
    local update_data=$(create_capsule_update_data "true")
    
    # Call capsules_update with empty ID
    local response=$(dfx canister call backend capsules_update "(\"\", $update_data)" 2>/dev/null)
    echo_info "Response: '$response'"
    
    if [ $? -eq 0 ]; then
        if is_failure "$response" && is_not_found "$response"; then
            echo_pass "capsules_update correctly returns NotFound for empty capsule ID"
            return 0
        else
            echo_fail "capsules_update should return NotFound error for empty ID"
            echo_info "Response: '$response'"
            return 1
        fi
    else
        echo_fail "capsules_update call failed"
        return 1
    fi
}

# Test 5: Update with no changes (empty update data)
test_capsules_update_no_changes() {
    echo_info "Testing capsules_update with no changes..."
    
    # Get the current user's self-capsule ID directly
    local self_capsule_result=$(dfx canister call backend capsules_read_basic "(null)" 2>/dev/null)
    if ! is_success "$self_capsule_result"; then
        echo_fail "Failed to get self-capsule for update test"
        return 1
    fi
    
    local capsule_id=$(echo "$self_capsule_result" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')
    if [[ -z "$capsule_id" ]]; then
        echo_fail "Failed to extract capsule ID from self-capsule response"
        return 1
    fi
    
    echo_info "Testing with self-capsule ID: $capsule_id"
    
    # Create update data with no changes (all fields null)
    local update_data="(record { bound_to_neon = null; })"
    echo_info "Update data: $update_data"
    
    # Call capsules_update
    local response=$(dfx canister call backend capsules_update "(\"$capsule_id\", $update_data)" 2>/dev/null)
    echo_info "Response: '$response'"
    
    if [ $? -eq 0 ]; then
        if is_success "$response" && has_capsule_data "$response"; then
            echo_pass "capsules_update call successful with no changes"
            return 0
        else
            echo_fail "capsules_update should return success even with no changes"
            echo_info "Response: '$response'"
            return 1
        fi
    else
        echo_fail "capsules_update call failed"
        return 1
    fi
}

# Test 6: Response structure validation
test_response_structure() {
    echo_info "Testing capsules_update response structure..."
    
    # Get the current user's self-capsule ID directly
    local self_capsule_result=$(dfx canister call backend capsules_read_basic "(null)" 2>/dev/null)
    if ! is_success "$self_capsule_result"; then
        echo_fail "Failed to get self-capsule for update test"
        return 1
    fi
    
    local capsule_id=$(echo "$self_capsule_result" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')
    if [[ -z "$capsule_id" ]]; then
        echo_fail "Failed to extract capsule ID from self-capsule response"
        return 1
    fi
    
    # Create update data
    local update_data=$(create_capsule_update_data "true")
    
    # Call capsules_update
    local response=$(dfx canister call backend capsules_update "(\"$capsule_id\", $update_data)" 2>/dev/null)
    echo_info "Response: '$response'"
    
    if [ $? -eq 0 ]; then
        if is_success "$response" && has_expected_capsule_fields "$response"; then
            echo_pass "Response contains expected capsule fields"
            return 0
        else
            echo_fail "Response missing expected capsule fields"
            echo_info "Response: '$response'"
            return 1
        fi
    else
        echo_fail "capsules_update call failed during structure validation"
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
    run_capsule_test "capsules_update with binding status change" test_capsules_update_binding
    run_capsule_test "capsules_update with unbinding status change" test_capsules_update_unbinding
    run_capsule_test "capsules_update with invalid capsule ID" test_capsules_update_invalid_id
    run_capsule_test "capsules_update with empty capsule ID" test_capsules_update_empty_id
    run_capsule_test "capsules_update with no changes" test_capsules_update_no_changes
    run_capsule_test "Response structure validation" test_response_structure
    
    # Test summary
    print_test_summary
}

# Run main function
main "$@"

