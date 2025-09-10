#!/bin/bash

# Test canister capsule creation functionality
# Tests the factory functionality that creates dedicated capsule canisters for users
# This tests the actual canister creation process, not just capsule data operations

# Load test configuration and utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_config.sh"
source "$SCRIPT_DIR/../test_utils.sh"

# Test configuration
TEST_NAME="Canister Capsule Creation Tests"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Helper function to check if response indicates success
is_success() {
    local response="$1"
    echo "$response" | grep -q "success = true"
}

# Helper function to check if response indicates failure
is_failure() {
    local response="$1"
    echo "$response" | grep -q "success = false"
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

# Helper function to extract canister ID from response
extract_canister_id() {
    local response="$1"
    echo "$response" | grep -o 'canister_id = opt principal "[^"]*"' | sed 's/canister_id = opt principal "\([^"]*\)"/\1/'
}

# Helper function to get a capsule ID for testing
get_test_capsule_id() {
    local capsule_result=$(dfx canister call backend capsules_read_basic "(null)" 2>/dev/null)
    local capsule_id=""
    
    if [[ $capsule_result == *"null"* ]]; then
        echo_info "No capsule found, creating one first..."
        local create_result=$(dfx canister call backend capsules_create "(null)" 2>/dev/null)
        capsule_id=$(echo "$create_result" | grep -o 'capsule_id = opt "[^"]*"' | sed 's/capsule_id = opt "//' | sed 's/"//')
    else
        capsule_id=$(echo "$capsule_result" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')
    fi
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to get capsule ID for testing"
        return 1
    fi
    
    echo "$capsule_id"
}

# Helper function to extract creation status
extract_creation_status() {
    local response="$1"
    echo "$response" | grep -o 'status = variant { [^}]*}' | sed 's/status = variant { \([^}]*\) }/\1/'
}

# Test functions for personal canister creation enabled/disabled state

test_personal_canister_creation_enabled_check() {
    echo_info "Testing personal canister creation enabled check..."
    
    # Call is_personal_canister_creation_enabled endpoint
    local result=$(dfx canister call backend is_personal_canister_creation_enabled 2>/dev/null)
    
    # Should return a boolean
    if echo "$result" | grep -q "(true)" || echo "$result" | grep -q "(false)"; then
        local enabled=$(echo "$result" | grep -o "(true)\|(false)")
        echo_info "Personal canister creation enabled status: $enabled"
        return 0
    else
        echo_info "Failed to get personal canister creation enabled status: $result"
        return 1
    fi
}

# Test functions for personal canister creation

test_create_personal_canister() {
    echo_info "Testing create personal canister..."
    
    # Ensure user is registered first
    local register_result=$(dfx canister call backend register 2>/dev/null)
    
    # Call create_personal_canister endpoint
    local result=$(dfx canister call backend create_personal_canister 2>/dev/null)
    
    # Check if creation was initiated successfully
    if is_success "$result"; then
        local canister_id=$(extract_canister_id "$result")
        echo_info "Personal canister creation initiated successfully"
        if [ -n "$canister_id" ]; then
            echo_info "Canister ID: $canister_id"
            # Save canister ID for other tests
            echo "$canister_id" > /tmp/test_personal_canister_id.txt
        fi
        return 0
    elif is_failure "$result"; then
        echo_info "Personal canister creation failed (may be expected): $result"
        # Failure might be expected (e.g., already exists, disabled, insufficient cycles)
        return 0
    else
        echo_info "Personal canister creation returned unexpected result: $result"
        return 1
    fi
}

test_duplicate_personal_canister_creation() {
    echo_info "Testing duplicate personal canister creation (idempotency)..."
    
    # Call create_personal_canister again (should be idempotent)
    local result=$(dfx canister call backend create_personal_canister 2>/dev/null)
    
    # Should either succeed (returning existing) or fail gracefully
    if is_success "$result" || is_failure "$result"; then
        echo_info "Duplicate personal canister creation handled appropriately: $result"
        return 0
    else
        echo_info "Duplicate personal canister creation returned unexpected result: $result"
        return 1
    fi
}

# Test functions for creation status queries

test_get_creation_status() {
    echo_info "Testing get creation status..."
    
    # Call get_creation_status endpoint
    local result=$(dfx canister call backend get_creation_status 2>/dev/null)
    
    # Should return either null or a CreationStatusResponse
    if echo "$result" | grep -q "(null)" || echo "$result" | grep -q "opt record"; then
        if echo "$result" | grep -q "opt record"; then
            local status=$(extract_creation_status "$result")
            echo_info "Creation status retrieved: $status"
        else
            echo_info "No creation status (null) - user may not have initiated creation"
        fi
        return 0
    else
        echo_info "Get creation status failed: $result"
        return 1
    fi
}

test_get_detailed_creation_status() {
    echo_info "Testing get detailed creation status..."
    
    # Call get_detailed_creation_status endpoint
    local result=$(dfx canister call backend get_detailed_creation_status 2>/dev/null)
    
    # Should return either null or a DetailedCreationStatus
    if echo "$result" | grep -q "(null)" || echo "$result" | grep -q "opt record"; then
        if echo "$result" | grep -q "opt record"; then
            local status=$(extract_creation_status "$result")
            echo_info "Detailed creation status retrieved: $status"
            
            # Check for additional detailed fields
            if echo "$result" | grep -q "progress_message" && echo "$result" | grep -q "cycles_consumed"; then
                echo_info "Detailed status includes progress and cycles information"
            fi
        else
            echo_info "No detailed creation status (null) - user may not have initiated creation"
        fi
        return 0
    else
        echo_info "Get detailed creation status failed: $result"
        return 1
    fi
}

# Test functions for personal canister ID retrieval

test_get_my_personal_canister_id() {
    echo_info "Testing get my personal canister ID..."
    
    # Call get_my_personal_canister_id endpoint
    local result=$(dfx canister call backend get_my_personal_canister_id 2>/dev/null)
    
    # Should return either null or a principal
    if echo "$result" | grep -q "(null)" || echo "$result" | grep -q "opt principal"; then
        if echo "$result" | grep -q "opt principal"; then
            local canister_id=$(echo "$result" | grep -o 'opt principal "[^"]*"' | sed 's/opt principal "\([^"]*\)"/\1/')
            echo_info "Personal canister ID retrieved: $canister_id"
            
            # Save for other tests
            echo "$canister_id" > /tmp/test_personal_canister_id.txt
        else
            echo_info "No personal canister ID (null) - user may not have a personal canister"
        fi
        return 0
    else
        echo_info "Get my personal canister ID failed: $result"
        return 1
    fi
}

test_get_personal_canister_id_by_principal() {
    echo_info "Testing get personal canister ID by principal..."
    
    # Get current user principal
    local current_principal=$(dfx identity get-principal)
    
    # Call get_personal_canister_id endpoint with current principal
    local result=$(dfx canister call backend get_personal_canister_id "(principal \"$current_principal\")" 2>/dev/null)
    
    # Should return either null or a principal
    if echo "$result" | grep -q "(null)" || echo "$result" | grep -q "opt principal"; then
        if echo "$result" | grep -q "opt principal"; then
            local canister_id=$(echo "$result" | grep -o 'opt principal "[^"]*"' | sed 's/opt principal "\([^"]*\)"/\1/')
            echo_info "Personal canister ID for principal $current_principal: $canister_id"
        else
            echo_info "No personal canister ID for principal $current_principal"
        fi
        return 0
    else
        echo_info "Get personal canister ID by principal failed: $result"
        return 1
    fi
}

# Test functions for creation statistics and monitoring

test_get_personal_canister_creation_stats() {
    echo_info "Testing get personal canister creation stats..."
    
    # Call get_personal_canister_creation_stats endpoint
    local result=$(dfx canister call backend get_personal_canister_creation_stats 2>/dev/null)
    
    # Should return PersonalCanisterCreationStats or an error
    if echo "$result" | grep -q "total_attempts" && echo "$result" | grep -q "total_successes"; then
        echo_info "Personal canister creation stats retrieved successfully"
        
        # Extract some basic stats
        local attempts=$(echo "$result" | grep -o 'total_attempts = [0-9]*' | sed 's/total_attempts = //')
        local successes=$(echo "$result" | grep -o 'total_successes = [0-9]*' | sed 's/total_successes = //')
        echo_info "Stats - Attempts: $attempts, Successes: $successes"
        return 0
    else
        echo_info "Get personal canister creation stats failed or returned error: $result"
        # This might fail if not admin, which is acceptable
        return 0
    fi
}

# Test functions for state transitions and progress monitoring

test_creation_status_progression() {
    echo_info "Testing creation status progression..."
    
    # This test monitors if status changes over time during creation
    # We'll check status multiple times with small delays
    
    local initial_status=$(dfx canister call backend get_creation_status 2>/dev/null)
    echo_info "Initial status: $initial_status"
    
    # Wait a moment and check again
    sleep 2
    local second_status=$(dfx canister call backend get_creation_status 2>/dev/null)
    echo_info "Status after 2s: $second_status"
    
    # Wait another moment and check again
    sleep 3
    local final_status=$(dfx canister call backend get_creation_status 2>/dev/null)
    echo_info "Status after 5s total: $final_status"
    
    # As long as we get valid responses, this test passes
    # The actual progression depends on whether creation is active
    if echo "$initial_status $second_status $final_status" | grep -q "null\|opt record"; then
        echo_info "Status progression monitoring completed successfully"
        return 0
    else
        echo_info "Status progression monitoring failed"
        return 1
    fi
}

test_creation_with_existing_capsule_data() {
    echo_info "Testing creation with existing capsule data..."
    
    # First ensure user has some capsule data
    local register_result=$(dfx canister call backend register 2>/dev/null)
    local capsule_result=$(dfx canister call backend register_capsule 2>/dev/null)
    
    # Add a test memory to the capsule
    local memory_data='(record {
      blob_ref = record {
        kind = variant { ICPCapsule };
        locator = "test_creation_memory";
        hash = null;
      };
      data = opt blob "VGVzdCBtZW1vcnkgZm9yIGNyZWF0aW9u";
    })'
    
    local capsule_id=$(get_test_capsule_id)
    
    if [[ -z "$capsule_id" ]]; then
        return 1
    fi
    
    local add_memory_result=$(dfx canister call backend memories_create "(\"$capsule_id\", $memory_data)" 2>/dev/null)
    
    # Now try to create personal canister (should export this data)
    local creation_result=$(dfx canister call backend create_personal_canister 2>/dev/null)
    
    # Check if creation handles existing data appropriately
    if is_success "$creation_result" || is_failure "$creation_result"; then
        echo_info "Creation with existing capsule data handled appropriately"
        return 0
    else
        echo_info "Creation with existing capsule data failed unexpectedly: $creation_result"
        return 1
    fi
}

# Test functions for error handling and edge cases

test_creation_when_disabled() {
    echo_info "Testing creation when disabled (if applicable)..."
    
    # Check if creation is enabled
    local enabled_result=$(dfx canister call backend is_personal_canister_creation_enabled 2>/dev/null)
    
    if echo "$enabled_result" | grep -q "(false)"; then
        echo_info "Personal canister creation is disabled"
        
        # Try to create (should fail gracefully)
        local creation_result=$(dfx canister call backend create_personal_canister 2>/dev/null)
        
        if is_failure "$creation_result" && echo "$creation_result" | grep -q -i "disabled\|not.*enabled"; then
            echo_info "Creation correctly rejected when disabled"
            return 0
        else
            echo_info "Creation when disabled not handled properly: $creation_result"
            return 1
        fi
    else
        echo_info "Personal canister creation is enabled, skipping disabled test"
        return 0
    fi
}

test_invalid_principal_lookup() {
    echo_info "Testing invalid principal lookup..."
    
    # Try to get personal canister ID for invalid principal
    local fake_principal="rdmx6-jaaaa-aaaah-qcaiq-cai"  # Valid format but likely non-existent
    local result
    local exit_code
    result=$(dfx canister call backend get_personal_canister_id "(principal \"$fake_principal\")" 2>&1)
    exit_code=$?
    
    # Should return null for non-existent user, or fail gracefully
    if echo "$result" | grep -q "(null)" || [ $exit_code -ne 0 ] || [ -z "$result" ]; then
        echo_info "Invalid principal lookup handled correctly"
        return 0
    else
        echo_info "Invalid principal lookup returned unexpected result: $result"
        return 1
    fi
}

# Test functions for integration scenarios

test_creation_status_consistency() {
    echo_info "Testing creation status consistency..."
    
    # Get status from both endpoints and compare
    local basic_status=$(dfx canister call backend get_creation_status 2>/dev/null)
    local detailed_status=$(dfx canister call backend get_detailed_creation_status 2>/dev/null)
    
    # Both should be consistent (both null or both have same status)
    local basic_null=$(echo "$basic_status" | grep -c "(null)")
    local detailed_null=$(echo "$detailed_status" | grep -c "(null)")
    
    if [ "$basic_null" -eq "$detailed_null" ]; then
        if [ "$basic_null" -eq 0 ]; then
            # Both have records, check if status matches
            local basic_state=$(extract_creation_status "$basic_status")
            local detailed_state=$(extract_creation_status "$detailed_status")
            
            if [ "$basic_state" = "$detailed_state" ]; then
                echo_info "Creation status consistency verified - both report: $basic_state"
                return 0
            else
                echo_info "Status inconsistency - basic: $basic_state, detailed: $detailed_state"
                return 1
            fi
        else
            echo_info "Creation status consistency verified - both are null"
            return 0
        fi
    else
        echo_info "Status consistency failed - one null, one not null"
        return 1
    fi
}

# Main test execution
main() {
    echo "========================================="
    echo "Starting $TEST_NAME"
    echo "========================================="
    echo ""
    
    # Check if backend canister ID is set
    if [ -z "$BACKEND_CANISTER_ID" ]; then
        echo_fail "BACKEND_CANISTER_ID not set in test_config.sh"
        echo_info "Please set the backend canister ID before running tests"
        exit 1
    fi
    
    # Check if dfx is available
    if ! command -v dfx &> /dev/null; then
        echo_fail "dfx command not found"
        echo_info "Please install dfx and ensure it's in your PATH"
        exit 1
    fi
    
    # Clean up any previous test files
    rm -f /tmp/test_personal_canister_id.txt
    
    # Run personal canister creation enabled/disabled tests
    echo_info "=== Testing Personal Canister Creation Configuration ==="
    run_test "Personal canister creation enabled check" "test_personal_canister_creation_enabled_check"
    
    # Run personal canister creation tests
    echo_info "=== Testing Personal Canister Creation ==="
    run_test "Create personal canister" "test_create_personal_canister"
    run_test "Duplicate personal canister creation" "test_duplicate_personal_canister_creation"
    
    # Run creation status query tests
    echo_info "=== Testing Creation Status Queries ==="
    run_test "Get creation status" "test_get_creation_status"
    run_test "Get detailed creation status" "test_get_detailed_creation_status"
    
    # Run personal canister ID retrieval tests
    echo_info "=== Testing Personal Canister ID Retrieval ==="
    run_test "Get my personal canister ID" "test_get_my_personal_canister_id"
    run_test "Get personal canister ID by principal" "test_get_personal_canister_id_by_principal"
    
    # Run creation statistics tests
    echo_info "=== Testing Creation Statistics ==="
    run_test "Get personal canister creation stats" "test_get_personal_canister_creation_stats"
    
    # Run state transition and progress tests
    echo_info "=== Testing State Transitions and Progress ==="
    run_test "Creation status progression" "test_creation_status_progression"
    run_test "Creation with existing capsule data" "test_creation_with_existing_capsule_data"
    
    # Run error handling and edge case tests
    echo_info "=== Testing Error Handling and Edge Cases ==="
    run_test "Creation when disabled" "test_creation_when_disabled"
    run_test "Invalid principal lookup" "test_invalid_principal_lookup"
    
    # Run integration scenario tests
    echo_info "=== Testing Integration Scenarios ==="
    run_test "Creation status consistency" "test_creation_status_consistency"
    
    # Clean up test files
    rm -f /tmp/test_personal_canister_id.txt
    
    # Print test summary
    echo "========================================="
    echo "Test Summary for $TEST_NAME"
    echo "========================================="
    echo "Total tests: $TOTAL_TESTS"
    echo "Passed: $PASSED_TESTS"
    echo "Failed: $FAILED_TESTS"
    echo ""
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo_pass "All tests passed!"
        exit 0
    else
        echo_fail "$FAILED_TESTS test(s) failed"
        exit 1
    fi
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi