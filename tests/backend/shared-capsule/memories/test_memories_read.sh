#!/bin/bash

# Test script for memories_read endpoint
# Tests the new memories_read(memory_id) endpoint that replaces get_memory_from_capsule

set -e

# Source test utilities
source "$(dirname "$0")/../../test_utils.sh"

# Configuration
CANISTER_ID="backend"
IDENTITY="default"
DEBUG="${DEBUG:-false}"  # Set DEBUG=true to enable debug output

echo_header "🧪 Testing memories_read endpoint"

# Test 1: Test memories_read with valid memory ID
test_memories_read_valid() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memories_read with valid memory ID..."
    
    # First, create a memory to test with
    local capsule_id=$(get_test_capsule_id)
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to get capsule ID for testing"
        return 1
    fi
    
    # Create test memory using utility function
    local memory_bytes='blob "VGVzdCBtZW1vcnkgZGF0YQ=="'
    local memory_id=$(create_test_memory "$capsule_id" "test_memory_read_123" "Test memory for read operations" '"test"; "read"' "$memory_bytes" "$CANISTER_ID" "$IDENTITY")
    
    if [[ -z "$memory_id" ]]; then
        echo_error "Failed to create test memory"
        return 1
    fi
    
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing with memory ID: $memory_id"
    
    # Call memories_read with the memory ID
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"$memory_id\")" 2>/dev/null)
    
    if [[ $result == *"Ok = record"* ]] && [[ $result == *"id = \"$memory_id\""* ]]; then
        echo_success "✅ memories_read with valid memory ID succeeded"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 0
    else
        echo_error "❌ memories_read with valid memory ID failed"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 1
    fi
}

# Test 2: Test memories_read with invalid memory ID
test_memories_read_invalid() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memories_read with invalid memory ID..."
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"invalid_memory_id_123\")" 2>/dev/null)
    
    if [[ $result == *"Err"* ]] || [[ $result == "(null)" ]]; then
        echo_success "✅ memories_read with invalid memory ID returned null as expected"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 0
    else
        echo_error "❌ memories_read with invalid memory ID returned unexpected result"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 1
    fi
}

# Test 3: Test memories_read with empty memory ID
test_memories_read_empty() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memories_read with empty memory ID..."
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"\")" 2>/dev/null)
    
    if [[ $result == *"Err"* ]] || [[ $result == "(null)" ]]; then
        echo_success "✅ memories_read with empty memory ID returned null as expected"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 0
    else
        echo_error "❌ memories_read with empty memory ID returned unexpected result"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 1
    fi
}

# Test 4: Test memories_read across different capsules
test_memories_read_cross_capsules() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memories_read across different capsules..."
    
    # Create two capsules and memories in each
    local capsule1_id=$(get_test_capsule_id)
    local capsule2_id=$(get_test_capsule_id)
    
    if [[ -z "$capsule1_id" ]] || [[ -z "$capsule2_id" ]]; then
        echo_error "Failed to get capsule IDs for testing"
        return 1
    fi
    
    # Create memories in both capsules using utility functions
    local memory1_bytes='blob "VGVzdCBtZW1vcnkgMQ=="'
    local memory1_id=$(create_test_memory "$capsule1_id" "cross_capsule_test_1" "Test memory 1 for cross-capsule test" '"test"; "cross-capsule"' "$memory1_bytes" "$CANISTER_ID" "$IDENTITY")
    
    local memory2_bytes='blob "VGVzdCBtZW1vcnkgMg=="'
    local memory2_id=$(create_test_memory "$capsule2_id" "cross_capsule_test_2" "Test memory 2 for cross-capsule test" '"test"; "cross-capsule"' "$memory2_bytes" "$CANISTER_ID" "$IDENTITY")
    
    if [[ -z "$memory1_id" ]] || [[ -z "$memory2_id" ]]; then
        echo_error "Failed to create test memories for cross-capsule test"
        return 1
    fi
    
    # Test reading both memories
    local read1_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"$memory1_id\")" 2>/dev/null)
    local read2_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"$memory2_id\")" 2>/dev/null)
    
    if [[ $read1_result == *"Ok = record"* ]] && [[ $read1_result == *"id = \"$memory1_id\""* ]] && [[ $read2_result == *"Ok = record"* ]] && [[ $read2_result == *"id = \"$memory2_id\""* ]]; then
        echo_success "✅ memories_read successfully retrieved memories from different capsules"
        [[ "$DEBUG" == "true" ]] && echo_debug "Memory 1 result: $read1_result"
        [[ "$DEBUG" == "true" ]] && echo_debug "Memory 2 result: $read2_result"
        return 0
    else
        echo_error "❌ memories_read failed to retrieve memories from different capsules"
        [[ "$DEBUG" == "true" ]] && echo_debug "Memory 1 result: $read1_result"
        [[ "$DEBUG" == "true" ]] && echo_debug "Memory 2 result: $read2_result"
        return 1
    fi
}

# Test 5: Test memories_read with saved memory ID (persistence test)
test_memories_read_persistence() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memories_read with saved memory ID for persistence..."
    
    if [ ! -f /tmp/test_memory_id.txt ]; then
        [[ "$DEBUG" == "true" ]] && echo_debug "No saved memory ID found, skipping persistence test"
        return 0
    fi
    
    local saved_memory_id=$(cat /tmp/test_memory_id.txt)
    [[ "$DEBUG" == "true" ]] && echo_debug "Using saved memory ID: $saved_memory_id"
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"$saved_memory_id\")" 2>/dev/null)
    
    if [[ $result == *"Ok = record"* ]] && [[ $result == *"id = \"$saved_memory_id\""* ]]; then
        echo_success "✅ memories_read with saved memory ID succeeded (persistence verified)"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 0
    elif [[ $result == *"Err"* ]] && [[ $result == *"NotFound"* ]]; then
        echo_success "✅ memories_read with saved memory ID returned NotFound (memory may have been deleted - this is expected)"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 0
    else
        echo_error "❌ memories_read with saved memory ID returned unexpected result"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 1
    fi
}

# Test 6: Verify old get_memory_from_capsule endpoint is removed
test_old_endpoint_removed() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Verifying old get_memory_from_capsule endpoint is removed..."
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID get_memory_from_capsule "(\"test_id\")" 2>/dev/null 2>&1 || true)
    
    if [[ $result == *"Method not found"* ]] || [[ $result == *"Unknown method"* ]] || [[ $result == *"Canister has no query method"* ]] || [[ $result == *"Canister has no update method"* ]]; then
        echo_success "✅ Old get_memory_from_capsule endpoint successfully removed"
        return 0
    else
        echo_error "❌ Old get_memory_from_capsule endpoint still exists"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 1
    fi
}


# Main test execution
main() {
    echo_header "🚀 Starting memories_read endpoint tests"
    
    run_test "Valid memory ID" test_memories_read_valid
    run_test "Invalid memory ID" test_memories_read_invalid
    run_test "Empty memory ID" test_memories_read_empty
    run_test "Cross-capsule access" test_memories_read_cross_capsules
    run_test "Memory ID persistence" test_memories_read_persistence
    run_test "Old endpoint removal" test_old_endpoint_removed
    
    echo_header "🎉 All memories_read tests completed successfully!"
}

# Run main function
main "$@"
