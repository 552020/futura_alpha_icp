#!/bin/bash

# Test store_gallery_forever_with_memories endpoint functionality
# Tests the enhanced gallery storage function that supports memory synchronization intent
# This tests the middle layer between basic gallery storage and full memory sync

# Load test configuration and utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_config.sh"
source "$SCRIPT_DIR/../test_utils.sh"

# Test configuration
TEST_NAME="Store Gallery Forever With Memories Tests"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Test data
TEST_TIMESTAMP=$(date +%s)
TEST_GALLERY_1_ID="test_gallery_with_memories_${TEST_TIMESTAMP}"
TEST_GALLERY_2_ID="test_gallery_without_memories_${TEST_TIMESTAMP}"

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

# Helper function to extract storage status from response
extract_storage_status() {
    local response="$1"
    echo "$response" | grep -o 'storage_status = variant { [^}]*}' | sed 's/storage_status = variant { \([^}]*\) }/\1/'
}

# Helper function to create gallery data
create_gallery_data() {
    local gallery_id="$1"
    local title="$2"
    local description="$3"
    local is_public="$4"
    
    local timestamp=${TEST_TIMESTAMP}000000000
    
    cat << EOF
(record {
  gallery = record {
    id = "$gallery_id";
    owner_principal = principal "$(dfx identity get-principal)";
    title = "$title";
    description = opt "$description";
    is_public = $is_public;
    created_at = $timestamp;
    updated_at = $timestamp;
    storage_status = variant { Web2Only };
    memory_entries = vec {};
  };
  owner_principal = principal "$(dfx identity get-principal)";
})
EOF
}

# Test setup - ensure user is registered and has a capsule
test_setup_user_and_capsule() {
    echo_info "Setting up test user and capsule..."
    
    # Register user
    local register_result=$(dfx canister call backend register 2>/dev/null)
    if ! is_success "$register_result"; then
        echo_warn "User registration failed, continuing with existing user..."
    fi
    
    # Mark capsule as bound to Web2
    local bind_result=$(dfx canister call backend mark_capsule_bound_to_web2 2>/dev/null)
    if ! is_success "$bind_result"; then
        echo_warn "Capsule binding failed, continuing..."
    fi
    
    echo_info "Setup complete"
    return 0
}

# Test 1: Store gallery with memory sync intent (sync_memories = true)
test_store_gallery_with_memory_sync_intent() {
    echo_info "Testing store_gallery_forever_with_memories with sync_memories = true..."
    
    local gallery_data=$(create_gallery_data "$TEST_GALLERY_1_ID" "Gallery With Memory Sync" "Gallery intended for memory synchronization" "false")
    
    # Call store_gallery_forever_with_memories with sync_memories = true
    local result=$(dfx canister call backend store_gallery_forever_with_memories "($gallery_data, true)" 2>/dev/null)
    
    if is_success "$result"; then
        local storage_status=$(extract_storage_status "$result")
        echo_info "Gallery stored with storage status: $storage_status"
        
        # Should set storage_status to Both when sync_memories = true
        if [ "$storage_status" = "Both" ]; then
            echo_info "Correct: Storage status set to 'Both' for memory sync intent"
            return 0
        else
            echo_warn "Expected storage status 'Both', got '$storage_status'"
            return 1
        fi
    else
        echo_fail "Failed to store gallery with memory sync intent: $result"
        return 1
    fi
}

# Test 2: Store gallery without memory sync intent (sync_memories = false)
test_store_gallery_without_memory_sync_intent() {
    echo_info "Testing store_gallery_forever_with_memories with sync_memories = false..."
    
    local gallery_data=$(create_gallery_data "$TEST_GALLERY_2_ID" "Gallery Without Memory Sync" "Gallery not intended for memory synchronization" "true")
    
    # Call store_gallery_forever_with_memories with sync_memories = false
    local result=$(dfx canister call backend store_gallery_forever_with_memories "($gallery_data, false)" 2>/dev/null)
    
    if is_success "$result"; then
        local storage_status=$(extract_storage_status "$result")
        echo_info "Gallery stored with storage status: $storage_status"
        
        # Should set storage_status to ICPOnly when sync_memories = false
        if [ "$storage_status" = "ICPOnly" ]; then
            echo_info "Correct: Storage status set to 'ICPOnly' for no memory sync intent"
            return 0
        else
            echo_warn "Expected storage status 'ICPOnly', got '$storage_status'"
            return 1
        fi
    else
        echo_fail "Failed to store gallery without memory sync intent: $result"
        return 1
    fi
}

# Test 3: Test idempotency - storing same gallery twice
test_store_gallery_idempotency() {
    echo_info "Testing idempotency of store_gallery_forever_with_memories..."
    
    local gallery_id="test_gallery_idempotent_${TEST_TIMESTAMP}"
    local gallery_data=$(create_gallery_data "$gallery_id" "Idempotent Gallery Test" "Testing idempotency" "false")
    
    # Store gallery first time
    local result1=$(dfx canister call backend store_gallery_forever_with_memories "($gallery_data, true)" 2>/dev/null)
    
    if ! is_success "$result1"; then
        echo_fail "First gallery storage failed: $result1"
        return 1
    fi
    
    # Store same gallery again - should be idempotent
    local result2=$(dfx canister call backend store_gallery_forever_with_memories "($gallery_data, false)" 2>/dev/null)
    
    if is_success "$result2"; then
        if echo "$result2" | grep -q "already exists"; then
            echo_info "Idempotency working: Gallery already exists message received"
            return 0
        else
            echo_info "Gallery stored again successfully (acceptable behavior)"
            return 0
        fi
    else
        echo_fail "Second gallery storage failed unexpectedly: $result2"
        return 1
    fi
}

# Test 4: Compare with basic store_gallery_forever
test_compare_with_basic_store_gallery() {
    echo_info "Testing comparison between store_gallery_forever and store_gallery_forever_with_memories..."
    
    local gallery_id_basic="test_gallery_basic_${TEST_TIMESTAMP}"
    local gallery_id_enhanced="test_gallery_enhanced_${TEST_TIMESTAMP}"
    
    local gallery_data_basic=$(create_gallery_data "$gallery_id_basic" "Basic Gallery" "Basic storage test" "true")
    local gallery_data_enhanced=$(create_gallery_data "$gallery_id_enhanced" "Enhanced Gallery" "Enhanced storage test" "true")
    
    # Store with basic endpoint
    local result_basic=$(dfx canister call backend store_gallery_forever "$gallery_data_basic" 2>/dev/null)
    
    # Store with enhanced endpoint (sync_memories = false, should behave similarly)
    local result_enhanced=$(dfx canister call backend store_gallery_forever_with_memories "($gallery_data_enhanced, false)" 2>/dev/null)
    
    if is_success "$result_basic" && is_success "$result_enhanced"; then
        local status_basic=$(extract_storage_status "$result_basic")
        local status_enhanced=$(extract_storage_status "$result_enhanced")
        
        echo_info "Basic endpoint storage status: $status_basic"
        echo_info "Enhanced endpoint storage status: $status_enhanced"
        
        # Both should result in ICPOnly status
        if [ "$status_basic" = "ICPOnly" ] && [ "$status_enhanced" = "ICPOnly" ]; then
            echo_info "Both endpoints produce consistent results"
            return 0
        else
            echo_warn "Storage statuses differ: basic='$status_basic', enhanced='$status_enhanced'"
            return 1
        fi
    else
        echo_fail "One or both gallery storage operations failed"
        echo_info "Basic result: $result_basic"
        echo_info "Enhanced result: $result_enhanced"
        return 1
    fi
}

# Test 5: Test gallery retrieval after storage
test_gallery_retrieval_after_storage() {
    echo_info "Testing gallery retrieval after store_gallery_forever_with_memories..."
    
    local gallery_id="test_gallery_retrieval_${TEST_TIMESTAMP}"
    local gallery_data=$(create_gallery_data "$gallery_id" "Retrieval Test Gallery" "Testing retrieval functionality" "false")
    
    # Store gallery
    local store_result=$(dfx canister call backend store_gallery_forever_with_memories "($gallery_data, true)" 2>/dev/null)
    
    if ! is_success "$store_result"; then
        echo_fail "Failed to store gallery for retrieval test: $store_result"
        return 1
    fi
    
    # Try to retrieve the gallery
    local get_result=$(dfx canister call backend get_gallery_by_id "(\"$gallery_id\")" 2>/dev/null)
    
    if echo "$get_result" | grep -q "opt record"; then
        echo_info "Gallery successfully retrieved after storage"
        
        # Check if the title matches
        if echo "$get_result" | grep -q "Retrieval Test Gallery"; then
            echo_info "Gallery title matches expected value"
            return 0
        else
            echo_warn "Gallery title doesn't match expected value"
            return 1
        fi
    elif echo "$get_result" | grep -q "null"; then
        echo_fail "Gallery not found after storage"
        return 1
    else
        echo_fail "Unexpected response from get_gallery_by_id: $get_result"
        return 1
    fi
}

# Test 6: Test with different gallery configurations
test_different_gallery_configurations() {
    echo_info "Testing store_gallery_forever_with_memories with different configurations..."
    
    # Test public gallery with memory sync
    local public_gallery_id="test_gallery_public_${TEST_TIMESTAMP}"
    local public_gallery_data=$(create_gallery_data "$public_gallery_id" "Public Gallery" "Public gallery with memory sync" "true")
    
    local public_result=$(dfx canister call backend store_gallery_forever_with_memories "($public_gallery_data, true)" 2>/dev/null)
    
    # Test private gallery without memory sync
    local private_gallery_id="test_gallery_private_${TEST_TIMESTAMP}"
    local private_gallery_data=$(create_gallery_data "$private_gallery_id" "Private Gallery" "Private gallery without memory sync" "false")
    
    local private_result=$(dfx canister call backend store_gallery_forever_with_memories "($private_gallery_data, false)" 2>/dev/null)
    
    if is_success "$public_result" && is_success "$private_result"; then
        local public_status=$(extract_storage_status "$public_result")
        local private_status=$(extract_storage_status "$private_result")
        
        echo_info "Public gallery (sync=true): $public_status"
        echo_info "Private gallery (sync=false): $private_status"
        
        if [ "$public_status" = "Both" ] && [ "$private_status" = "ICPOnly" ]; then
            echo_info "Both gallery configurations stored correctly"
            return 0
        else
            echo_warn "Unexpected storage statuses"
            return 1
        fi
    else
        echo_fail "Failed to store one or both gallery configurations"
        return 1
    fi
}

# Main test execution
main() {
    echo_info "Starting $TEST_NAME"
    echo_info "Testing store_gallery_forever_with_memories endpoint functionality"
    echo ""
    
    # Check if backend canister is running
    if ! check_canister_status "$BACKEND_CANISTER_ID"; then
        echo_fail "Backend canister is not running. Please start dfx first."
        exit 1
    fi
    
    echo_info "Backend canister ID: $BACKEND_CANISTER_ID"
    echo ""
    
    # Run tests
    run_test "Setup user and capsule" "test_setup_user_and_capsule"
    run_test "Store gallery with memory sync intent" "test_store_gallery_with_memory_sync_intent"
    run_test "Store gallery without memory sync intent" "test_store_gallery_without_memory_sync_intent"
    run_test "Test gallery storage idempotency" "test_store_gallery_idempotency"
    run_test "Compare with basic store_gallery_forever" "test_compare_with_basic_store_gallery"
    run_test "Test gallery retrieval after storage" "test_gallery_retrieval_after_storage"
    run_test "Test different gallery configurations" "test_different_gallery_configurations"
    
    # Print test results
    echo ""
    echo_info "Test Results Summary:"
    echo_info "Total Tests: $TOTAL_TESTS"
    echo_info "Passed: $PASSED_TESTS"
    echo_info "Failed: $FAILED_TESTS"
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo_pass "All tests passed! 🎉"
        exit 0
    else
        echo_fail "Some tests failed. Please review the output above."
        exit 1
    fi
}

# Run main function
main "$@"
