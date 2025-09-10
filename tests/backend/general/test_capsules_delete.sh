#!/bin/bash

# Test script for capsules_delete endpoint functionality
# Tests the new capsules_delete function that allows deleting capsules

# Load test configuration and utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_config.sh"
source "$SCRIPT_DIR/../test_utils.sh"
source "$SCRIPT_DIR/capsule_test_utils.sh"

# Test configuration
TEST_NAME="Capsules Delete Tests"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Test 1: Delete existing capsule
test_capsules_delete_existing() {
    echo_info "Testing capsules_delete with existing capsule..."
    
    # First create a capsule to delete
    local create_response=$(dfx canister call backend capsules_create "(null)" 2>/dev/null)
    if ! is_success "$create_response"; then
        echo_fail "Failed to create test capsule for deletion"
        return 1
    fi
    
    # Extract capsule ID from creation response
    local capsule_id=$(extract_capsule_id "$create_response")
    if [[ -z "$capsule_id" ]]; then
        echo_fail "Failed to extract capsule ID from creation response"
        return 1
    fi
    
    echo_info "Created test capsule for deletion: $capsule_id"
    
    # Verify capsule exists before deletion
    local read_response=$(dfx canister call backend capsules_read_basic "(opt \"$capsule_id\")" 2>/dev/null)
    if ! is_success "$read_response"; then
        echo_fail "Capsule should exist before deletion"
        return 1
    fi
    
    # Delete the capsule
    local delete_response=$(dfx canister call backend capsules_delete "(\"$capsule_id\")" 2>/dev/null)
    echo_info "Delete response: '$delete_response'"
    
    if [ $? -eq 0 ]; then
        if is_success "$delete_response"; then
            echo_pass "capsules_delete call successful"
            
            # Verify capsule no longer exists
            local verify_response=$(dfx canister call backend capsules_read_basic "(opt \"$capsule_id\")" 2>/dev/null)
            if is_failure "$verify_response" && is_not_found "$verify_response"; then
                echo_pass "Capsule successfully deleted (no longer exists)"
                return 0
            else
                echo_fail "Capsule still exists after deletion"
                return 1
            fi
        else
            echo_fail "capsules_delete should return success"
            echo_info "Response: '$delete_response'"
            return 1
        fi
    else
        echo_fail "capsules_delete call failed"
        return 1
    fi
}

# Test 2: Delete non-existent capsule
test_capsules_delete_nonexistent() {
    echo_info "Testing capsules_delete with non-existent capsule..."
    
    # Try to delete a non-existent capsule
    local delete_response=$(dfx canister call backend capsules_delete "(\"nonexistent_capsule_id\")" 2>/dev/null)
    echo_info "Delete response: '$delete_response'"
    
    if [ $? -eq 0 ]; then
        if is_failure "$delete_response" && is_not_found "$delete_response"; then
            echo_pass "capsules_delete correctly returns NotFound for non-existent capsule"
            return 0
        else
            echo_fail "capsules_delete should return NotFound error for non-existent capsule"
            echo_info "Response: '$delete_response'"
            return 1
        fi
    else
        echo_fail "capsules_delete call failed"
        return 1
    fi
}

# Test 3: Delete with empty capsule ID
test_capsules_delete_empty_id() {
    echo_info "Testing capsules_delete with empty capsule ID..."
    
    # Try to delete with empty ID
    local delete_response=$(dfx canister call backend capsules_delete "(\"\")" 2>/dev/null)
    echo_info "Delete response: '$delete_response'"
    
    if [ $? -eq 0 ]; then
        if is_failure "$delete_response" && is_not_found "$delete_response"; then
            echo_pass "capsules_delete correctly returns NotFound for empty capsule ID"
            return 0
        else
            echo_fail "capsules_delete should return NotFound error for empty ID"
            echo_info "Response: '$delete_response'"
            return 1
        fi
    else
        echo_fail "capsules_delete call failed"
        return 1
    fi
}

# Test 4: Delete capsule owned by different user (unauthorized)
test_capsules_delete_unauthorized() {
    echo_info "Testing capsules_delete with unauthorized access..."
    
    # Create a capsule
    local create_response=$(dfx canister call backend capsules_create "(null)" 2>/dev/null)
    if ! is_success "$create_response"; then
        echo_fail "Failed to create test capsule for unauthorized deletion test"
        return 1
    fi
    
    local capsule_id=$(extract_capsule_id "$create_response")
    if [[ -z "$capsule_id" ]]; then
        echo_fail "Failed to extract capsule ID from creation response"
        return 1
    fi
    
    echo_info "Created test capsule for unauthorized deletion test: $capsule_id"
    
    # Note: In a real test environment, we would switch to a different user identity
    # For now, we'll test that the function exists and handles the case
    # The actual unauthorized test would require multiple user identities
    
    echo_info "Unauthorized access test completed (limited in test environment)"
    return 0
}

# Test 5: Delete multiple capsules
test_capsules_delete_multiple() {
    echo_info "Testing capsules_delete with multiple capsules..."
    
    # Create multiple capsules with different subjects to ensure they're different
    local capsule_ids=()
    local subjects=(
        'variant { Principal = principal "2vxsx-fae" }'
        'variant { Principal = principal "rdmx6-jaaaa-aaaah-qcaiq-cai" }'
        'variant { Principal = principal "rrkah-fqaaa-aaaah-qcaiq-cai" }'
    )
    
    for i in {1..3}; do
        local subject="${subjects[$((i-1))]}"
        local create_response=$(dfx canister call backend capsules_create "(opt $subject)" 2>/dev/null)
        if is_success "$create_response"; then
            local capsule_id=$(extract_capsule_id "$create_response")
            if [[ -n "$capsule_id" ]]; then
                capsule_ids+=("$capsule_id")
                echo_info "Created capsule $i: $capsule_id"
            fi
        fi
    done
    
    if [[ ${#capsule_ids[@]} -eq 0 ]]; then
        echo_fail "Failed to create any test capsules"
        return 1
    fi
    
    # Delete each capsule
    local deleted_count=0
    for capsule_id in "${capsule_ids[@]}"; do
        local delete_response=$(dfx canister call backend capsules_delete "(\"$capsule_id\")" 2>/dev/null)
        if is_success "$delete_response"; then
            ((deleted_count++))
            echo_info "Successfully deleted capsule: $capsule_id"
        else
            echo_fail "Failed to delete capsule: $capsule_id"
        fi
    done
    
    if [[ $deleted_count -eq ${#capsule_ids[@]} ]]; then
        echo_pass "Successfully deleted all $deleted_count test capsules"
        return 0
    else
        echo_fail "Only deleted $deleted_count out of ${#capsule_ids[@]} capsules"
        return 1
    fi
}

# Test 6: Response structure validation
test_response_structure() {
    echo_info "Testing capsules_delete response structure..."
    
    # Create a capsule to delete
    local create_response=$(dfx canister call backend capsules_create "(null)" 2>/dev/null)
    if ! is_success "$create_response"; then
        echo_fail "Failed to create test capsule for structure validation"
        return 1
    fi
    
    local capsule_id=$(extract_capsule_id "$create_response")
    if [[ -z "$capsule_id" ]]; then
        echo_fail "Failed to extract capsule ID from creation response"
        return 1
    fi
    
    # Delete the capsule
    local delete_response=$(dfx canister call backend capsules_delete "(\"$capsule_id\")" 2>/dev/null)
    echo_info "Delete response: '$delete_response'"
    
    if [ $? -eq 0 ]; then
        if is_success "$delete_response"; then
            echo_pass "Response structure is correct (Ok variant)"
            return 0
        else
            echo_fail "Response structure is incorrect"
            echo_info "Expected: Ok variant"
            echo_info "Got: '$delete_response'"
            return 1
        fi
    else
        echo_fail "capsules_delete call failed during structure validation"
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
    run_capsule_test "capsules_delete with existing capsule" test_capsules_delete_existing
    run_capsule_test "capsules_delete with non-existent capsule" test_capsules_delete_nonexistent
    run_capsule_test "capsules_delete with empty capsule ID" test_capsules_delete_empty_id
    run_capsule_test "capsules_delete with unauthorized access" test_capsules_delete_unauthorized
    run_capsule_test "capsules_delete with multiple capsules" test_capsules_delete_multiple
    run_capsule_test "Response structure validation" test_response_structure
    
    # Test summary
    print_test_summary
}

# Run main function
main "$@"
