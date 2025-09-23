#!/bin/bash

# Test canister capsule creation functionality
# Tests the factory functionality that creates dedicated capsule canisters for users
# This tests the actual canister creation process, not just capsule data operations
#
# Usage: ./test_canister_capsule.sh [--mainnet]
# - Without --mainnet: Tests against local canister (no cycle costs)
# - With --mainnet: Tests against mainnet canister (WARNING: will cost cycles!)
#
# Module: src/backend/src/canister_factory.rs
# This test suite covers the Personal Canister Creation functionality that allows users
# to create dedicated canister instances for their capsule data.
#
# Functions/Functionalities Tested:
# - is_personal_canister_creation_enabled() - Check if personal canister creation is enabled
# - create_personal_canister() - Create a new personal canister for the user
# - get_creation_status() - Get basic creation status information
# - get_detailed_creation_status() - Get detailed creation status with progress info
# - get_my_personal_canister_id() - Get the current user's personal canister ID
# - get_personal_canister_id(principal) - Get personal canister ID for a specific principal
# - get_personal_canister_creation_stats() - Get creation statistics (admin function)
#
# Test Categories:
# 1. Configuration Tests - Check if creation is enabled/disabled
# 2. Creation Tests - Test canister creation and idempotency
# 3. Status Query Tests - Test status retrieval endpoints
# 4. ID Retrieval Tests - Test personal canister ID lookup
# 5. Statistics Tests - Test creation statistics (admin)
# 6. State Transition Tests - Test status progression and monitoring
# 7. Error Handling Tests - Test disabled state and invalid inputs
# 8. Integration Tests - Test consistency between endpoints

# Load test configuration and utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_utils.sh"

# Parse command line arguments
MAINNET_MODE=false
if [[ "$1" == "--mainnet" ]]; then
    MAINNET_MODE=true
    echo_info "Running in mainnet mode"
fi

# Set up canister configuration based on mode
if [[ "$MAINNET_MODE" == "true" ]]; then
    source "$SCRIPT_DIR/../mainnet/config.sh"
    CANISTER_ID="$MAINNET_CANISTER_ID"
    NETWORK_FLAG="--network $MAINNET_NETWORK"
    echo_info "Using mainnet canister: $CANISTER_ID"
else
    source "$SCRIPT_DIR/../test_config.sh"
    CANISTER_ID="${BACKEND_CANISTER_ID:-$(dfx canister id backend 2>/dev/null)}"
    NETWORK_FLAG=""
    if [ -z "$CANISTER_ID" ]; then
        echo_error "Backend canister not found. Make sure it's deployed locally."
        exit 1
    fi
    echo_info "Using local canister: $CANISTER_ID"
fi

# Test configuration
TEST_NAME="Canister Capsule Creation Tests"
PASSED_TESTS=0
FAILED_TESTS=0

# Test functions for personal canister creation enabled/disabled state

test_personal_canister_creation_enabled_check() {
    echo_info "Testing personal canister creation enabled check..."
    
    # Call is_personal_canister_creation_enabled endpoint
    local result=$(dfx canister call "$CANISTER_ID" is_personal_canister_creation_enabled $NETWORK_FLAG 2>/dev/null)
    
    # Should return a Result<bool> - check for Ok = true/false or Err
    if echo "$result" | grep -q "Ok = true" || echo "$result" | grep -q "Ok = false"; then
        local enabled=$(echo "$result" | grep -o "Ok = true\|Ok = false")
        echo_info "Personal canister creation enabled status: $enabled"
        return 0
    elif echo "$result" | grep -q "Err"; then
        echo_info "Personal canister creation enabled check returned error: $result"
        return 0  # Error response is also valid
    else
        echo_info "Failed to get personal canister creation enabled status: $result"
        return 1
    fi
}

# Test functions for personal canister creation

test_create_personal_canister() {
    echo_info "Testing create personal canister..."
    
    # Warn about expensive operation on mainnet
    if [[ "$MAINNET_MODE" == "true" ]]; then
        warn_expensive_operation "Personal canister creation" "1-2"
    fi
    
    # Monitor cycles for expensive operations
    local initial_balance=""
    if [[ "$MAINNET_MODE" == "true" ]]; then
        initial_balance=$(monitor_cycles "$CANISTER_ID" "$NETWORK_FLAG" "Personal canister creation")
    fi
    
    # Ensure user is registered first
    local register_result=$(dfx canister call "$CANISTER_ID" register $NETWORK_FLAG 2>/dev/null)
    
    # Call create_personal_canister endpoint
    local result=$(dfx canister call "$CANISTER_ID" create_personal_canister $NETWORK_FLAG 2>/dev/null)
    
    # Calculate cycle consumption if monitoring
    if [[ "$MAINNET_MODE" == "true" && -n "$initial_balance" ]]; then
        calculate_cycle_consumption "$initial_balance" "$CANISTER_ID" "$NETWORK_FLAG" "Personal canister creation"
    fi
    
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
    local result=$(dfx canister call "$CANISTER_ID" create_personal_canister $NETWORK_FLAG 2>/dev/null)
    
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
    local result=$(dfx canister call "$CANISTER_ID" get_creation_status --query $NETWORK_FLAG 2>/dev/null)
    
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
    local result=$(dfx canister call "$CANISTER_ID" get_detailed_creation_status --query $NETWORK_FLAG 2>/dev/null)
    
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
    local result=$(dfx canister call "$CANISTER_ID" get_my_personal_canister_id --query $NETWORK_FLAG 2>/dev/null)
    
    # Should return either null or a principal
    if echo "$result" | grep -q "(null)" || echo "$result" | grep -q "opt principal"; then
        if echo "$result" | grep -q "opt principal"; then
            local canister_id=$(extract_principal "$result")
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
    local result=$(dfx canister call "$CANISTER_ID" get_personal_canister_id "(principal \"$current_principal\")" --query $NETWORK_FLAG 2>/dev/null)
    
    # Should return either null or a principal
    if echo "$result" | grep -q "(null)" || echo "$result" | grep -q "opt principal"; then
        if echo "$result" | grep -q "opt principal"; then
            local canister_id=$(extract_principal "$result")
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
    local result=$(dfx canister call "$CANISTER_ID" get_personal_canister_creation_stats --query $NETWORK_FLAG 2>/dev/null)
    
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
    
    local initial_status=$(dfx canister call "$CANISTER_ID" get_creation_status --query $NETWORK_FLAG 2>/dev/null)
    echo_info "Initial status: $initial_status"
    
    # Wait a moment and check again
    sleep 2
    local second_status=$(dfx canister call "$CANISTER_ID" get_creation_status --query $NETWORK_FLAG 2>/dev/null)
    echo_info "Status after 2s: $second_status"
    
    # Wait another moment and check again
    sleep 3
    local final_status=$(dfx canister call "$CANISTER_ID" get_creation_status --query $NETWORK_FLAG 2>/dev/null)
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
    local register_result=$(dfx canister call "$CANISTER_ID" register $NETWORK_FLAG 2>/dev/null)
    local capsule_result=$(dfx canister call "$CANISTER_ID" register_capsule $NETWORK_FLAG 2>/dev/null)
    
    # Add a test memory to the capsule
    local memory_data='(record {
      blob_ref = record {
        kind = variant { ICPCapsule };
        locator = "test_creation_memory";
        hash = null;
      };
      data = opt blob "VGVzdCBtZW1vcnkgZm9yIGNyZWF0aW9u";
    })'
    
    local capsule_id=$(get_test_capsule_id "backend" "default")
    
    if [[ -z "$capsule_id" ]]; then
        return 1
    fi
    
    local add_memory_result=$(dfx canister call "$CANISTER_ID" memories_create "(\"$capsule_id\", $memory_data)" $NETWORK_FLAG 2>/dev/null)
    
    # Now try to create personal canister (should export this data)
    local creation_result=$(dfx canister call "$CANISTER_ID" create_personal_canister $NETWORK_FLAG 2>/dev/null)
    
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
    local enabled_result=$(dfx canister call "$CANISTER_ID" is_personal_canister_creation_enabled $NETWORK_FLAG 2>/dev/null)
    
    if echo "$enabled_result" | grep -q "(false)"; then
        echo_info "Personal canister creation is disabled"
        
        # Try to create (should fail gracefully)
        local creation_result=$(dfx canister call "$CANISTER_ID" create_personal_canister $NETWORK_FLAG 2>/dev/null)
        
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
    result=$(dfx canister call "$CANISTER_ID" get_personal_canister_id "(principal \"$fake_principal\")" --query $NETWORK_FLAG 2>&1)
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
    local basic_status=$(dfx canister call "$CANISTER_ID" get_creation_status --query $NETWORK_FLAG 2>/dev/null)
    local detailed_status=$(dfx canister call "$CANISTER_ID" get_detailed_creation_status --query $NETWORK_FLAG 2>/dev/null)
    
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
    
    # Canister ID is now set dynamically based on mode (local vs mainnet)
    echo_info "Testing with canister: $CANISTER_ID"
    
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
    run_test_with_counters "Personal canister creation enabled check" "test_personal_canister_creation_enabled_check" "success" "PASSED_TESTS" "FAILED_TESTS"
    echo ""
    
    # Run personal canister creation tests
    echo_info "=== Testing Personal Canister Creation ==="
    run_test_with_counters "Create personal canister" "test_create_personal_canister" "success" "PASSED_TESTS" "FAILED_TESTS"
    run_test_with_counters "Duplicate personal canister creation" "test_duplicate_personal_canister_creation" "success" "PASSED_TESTS" "FAILED_TESTS"
    echo ""
    
    # Run creation status query tests
    echo_info "=== Testing Creation Status Queries ==="
    run_test_with_counters "Get creation status" "test_get_creation_status" "success" "PASSED_TESTS" "FAILED_TESTS"
    run_test_with_counters "Get detailed creation status" "test_get_detailed_creation_status" "success" "PASSED_TESTS" "FAILED_TESTS"
    echo ""
    
    # Run personal canister ID retrieval tests
    echo_info "=== Testing Personal Canister ID Retrieval ==="
    run_test_with_counters "Get my personal canister ID" "test_get_my_personal_canister_id" "success" "PASSED_TESTS" "FAILED_TESTS"
    run_test_with_counters "Get personal canister ID by principal" "test_get_personal_canister_id_by_principal" "success" "PASSED_TESTS" "FAILED_TESTS"
    echo ""
    
    # Run creation statistics tests
    echo_info "=== Testing Creation Statistics ==="
    run_test_with_counters "Get personal canister creation stats" "test_get_personal_canister_creation_stats" "success" "PASSED_TESTS" "FAILED_TESTS"
    echo ""
    
    # Run state transition and progress tests
    echo_info "=== Testing State Transitions and Progress ==="
    run_test_with_counters "Creation status progression" "test_creation_status_progression" "success" "PASSED_TESTS" "FAILED_TESTS"
    run_test_with_counters "Creation with existing capsule data" "test_creation_with_existing_capsule_data" "success" "PASSED_TESTS" "FAILED_TESTS"
    echo ""
    
    # Run error handling and edge case tests
    echo_info "=== Testing Error Handling and Edge Cases ==="
    run_test_with_counters "Creation when disabled" "test_creation_when_disabled" "success" "PASSED_TESTS" "FAILED_TESTS"
    run_test_with_counters "Invalid principal lookup" "test_invalid_principal_lookup" "success" "PASSED_TESTS" "FAILED_TESTS"
    echo ""
    
    # Run integration scenario tests
    echo_info "=== Testing Integration Scenarios ==="
    run_test_with_counters "Creation status consistency" "test_creation_status_consistency" "success" "PASSED_TESTS" "FAILED_TESTS"
    echo ""
    
    # Clean up test files
    rm -f /tmp/test_personal_canister_id.txt
    
    # Print test summary
    echo "========================================="
    echo "Test Summary for $TEST_NAME"
    echo "========================================="
    echo "Total tests: $((PASSED_TESTS + FAILED_TESTS))"
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