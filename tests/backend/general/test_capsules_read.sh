#!/bin/bash

# Test capsules_read endpoints functionality
# Tests both capsules_read_basic and capsules_read_full functions

# Load test configuration and utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_config.sh"
source "$SCRIPT_DIR/../test_utils.sh"
source "$SCRIPT_DIR/capsule_test_utils.sh"

# Test configuration
TEST_NAME="Capsules Read Basic/Full Tests"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Test data
TEST_CAPSULE_ID="test_capsule_$(date +%s)"


# Test 1: Basic capsules_read_basic call with no ID (should return self-capsule info)
test_capsules_read_basic_self() {
    echo_info "Testing capsules_read_basic functionality with no ID (self-capsule info)..."
    
    local response=$(dfx canister call backend capsules_read_basic 2>/dev/null)
    echo_info "Response: '$response'"
    
    if [ $? -eq 0 ]; then
        if is_success "$response" && has_capsule_data "$response"; then
            echo_pass "capsules_read_basic call successful with no ID (returns self-capsule info)"
            return 0
        elif is_failure "$response" && is_not_found "$response"; then
            echo_pass "capsules_read_basic call successful with no ID (returns NotFound - no self-capsule)"
            return 0
        else
            echo_fail "capsules_read_basic should return capsule info or NotFound for self-capsule"
            echo_info "Expected capsule info or NotFound, got: '$response'"
            return 1
        fi
    else
        echo_fail "capsules_read_basic call failed"
        return 1
    fi
}

# Test 2: Basic capsules_read_full call with no ID (should return full self-capsule)
test_capsules_read_full_self() {
    echo_info "Testing capsules_read_full functionality with no ID (full self-capsule)..."
    
    local response=$(dfx canister call backend capsules_read_full 2>/dev/null)
    echo_info "Response: '$response'"
    
    if [ $? -eq 0 ]; then
        if is_success "$response" && has_capsule_data "$response"; then
            echo_pass "capsules_read_full call successful with no ID (returns full self-capsule)"
            return 0
        elif is_failure "$response" && is_not_found "$response"; then
            echo_pass "capsules_read_full call successful with no ID (returns NotFound - no self-capsule)"
            return 0
        else
            echo_fail "capsules_read_full should return full capsule data or NotFound for self-capsule"
            echo_info "Expected full capsule data or NotFound, got: '$response'"
            return 1
        fi
    else
        echo_fail "capsules_read_full call failed"
        return 1
    fi
}

# Test 3: Basic capsules_read_basic call with invalid ID
test_basic_capsules_read_basic_invalid() {
    echo_info "Testing basic capsules_read_basic functionality with invalid ID..."
    
    local response=$(dfx canister call backend capsules_read_basic '(opt "test_invalid_id")' 2>/dev/null)
    echo_info "Response: '$response'"
    
    if [ $? -eq 0 ]; then
        if is_failure "$response" && is_not_found "$response"; then
            echo_pass "capsules_read_basic call successful with invalid ID (returns NotFound)"
            return 0
        else
            echo_fail "capsules_read_basic should return NotFound for invalid ID"
            echo_info "Expected NotFound, got: '$response'"
            return 1
        fi
    else
        echo_fail "capsules_read_basic call failed"
        return 1
    fi
}

# Test 3: Test capsules_read with empty string
test_capsules_read_empty_string() {
    echo_info "Testing capsules_read_basic with empty string..."
    
    local response=$(dfx canister call backend capsules_read_basic '(opt "")' 2>/dev/null)
    echo_info "Response: '$response'"
    
    if [ $? -eq 0 ]; then
        if is_failure "$response" && is_not_found "$response"; then
            echo_pass "capsules_read_basic call successful with empty string (returns NotFound)"
            return 0
        else
            echo_fail "capsules_read_basic should return NotFound for empty string"
            echo_info "Expected NotFound, got: '$response'"
            return 1
        fi
    else
        echo_fail "capsules_read_basic call failed with empty string"
        return 1
    fi
}

# Test 4: Test capsules_read with valid capsule ID (if user has capsules)
test_capsules_read_valid_id() {
    echo_info "Testing capsules_read_basic with valid capsule ID..."
    
    # First, get the user's capsules to see if they have any
    local capsules_response=$(dfx canister call backend capsules_list 2>/dev/null)
    
    if echo "$capsules_response" | grep -q "vec {}"; then
        echo_info "User has no capsules, testing with invalid ID instead"
        local response=$(dfx canister call backend capsules_read_basic '(opt "no_capsules_exist")' 2>/dev/null)
        
        if [ $? -eq 0 ] && is_failure "$response" && is_not_found "$response"; then
            echo_pass "capsules_read_basic correctly returns NotFound when no capsules exist"
            return 0
        else
            echo_fail "capsules_read_basic failed or returned unexpected result"
            return 1
        fi
    else
        # Extract the first capsule ID from the response
        local capsule_id=$(echo "$capsules_response" | grep -o '"[^"]*"' | head -1 | tr -d '"')
        
        if [ -n "$capsule_id" ]; then
            echo_info "Testing with existing capsule ID: $capsule_id"
            local response=$(dfx canister call backend capsules_read_basic '(opt "'"$capsule_id"'")' 2>/dev/null)
            echo_info "Response: '$response'"
            
            if [ $? -eq 0 ] && is_success "$response" && has_capsule_data "$response"; then
                echo_pass "capsules_read_basic successfully retrieved existing capsule"
                return 0
            else
                echo_fail "capsules_read_basic failed to retrieve existing capsule"
                echo_info "Response: '$response'"
                return 1
            fi
        else
            echo_fail "Could not extract capsule ID from response"
            return 1
        fi
    fi
}

# Test 5: Test with authenticated user
test_authenticated_user() {
    echo_info "Testing capsules_read_basic with authenticated user..."
    
    # Ensure we're using the current identity
    local current_principal=$(dfx identity get-principal)
    echo_info "Current principal: $current_principal"
    
    local response=$(dfx canister call backend capsules_read_basic '(opt "test_id")' 2>/dev/null)
    
    if [ $? -eq 0 ]; then
        echo_pass "capsules_read_basic works with authenticated user"
        return 0
    else
        echo_fail "capsules_read_basic failed with authenticated user"
        return 1
    fi
}

# Test 6: Test response structure for valid capsule
test_response_structure() {
    echo_info "Testing response structure..."
    
    # First, get the user's capsules to see if they have any
    local capsules_response=$(dfx canister call backend capsules_list 2>/dev/null)
    
    if echo "$capsules_response" | grep -q "vec {}"; then
        echo_info "User has no capsules, testing structure with invalid ID"
        local response=$(dfx canister call backend capsules_read_basic '(opt "no_capsules_exist")' 2>/dev/null)
        
        if is_failure "$response" && is_not_found "$response"; then
            echo_pass "Response structure is correct for non-existent capsule (NotFound)"
            return 0
        else
            echo_fail "Response structure is incorrect for non-existent capsule"
            return 1
        fi
    else
        # Extract the first capsule ID from the response
        local capsule_id=$(echo "$capsules_response" | grep -o '"[^"]*"' | head -1 | tr -d '"')
        
        if [ -n "$capsule_id" ]; then
            local response=$(dfx canister call backend capsules_read_basic '(opt "'"$capsule_id"'")' 2>/dev/null)
            echo_info "Response: '$response'"
            
            if is_success "$response" && has_expected_capsule_info_fields "$response"; then
                echo_pass "Response contains expected capsule info fields"
                return 0
            else
                echo_fail "Response missing expected capsule info fields"
                echo_info "Response: '$response'"
                return 1
            fi
        else
            echo_fail "Could not extract capsule ID for structure test"
            return 1
        fi
    fi
}

# Main test execution
main() {
    echo_info "Starting $TEST_NAME"
    echo_info "=================================="
    
    # Setup
    setup_user_and_capsule
    
    # Run tests
    run_capsule_test "capsules_read_basic with no ID (self-capsule info)" test_capsules_read_basic_self
    run_capsule_test "capsules_read_full with no ID (full self-capsule)" test_capsules_read_full_self
    run_capsule_test "Basic capsules_read_basic call with invalid ID" test_basic_capsules_read_basic_invalid
    run_capsule_test "capsules_read_basic with empty string" test_capsules_read_empty_string
    run_capsule_test "capsules_read_basic with valid capsule ID" test_capsules_read_valid_id
    run_capsule_test "Authenticated user access" test_authenticated_user
    run_capsule_test "Response structure validation" test_response_structure
    
    # Test summary
    print_test_summary
}

# Run main function
main "$@"
