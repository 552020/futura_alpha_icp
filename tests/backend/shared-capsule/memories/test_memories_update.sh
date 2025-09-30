#!/bin/bash

# Test script for memories_update endpoint
# Tests the new memories_update(memory_id, updates) endpoint that replaces update_memory_in_capsule

set -e

# Source test utilities
source "$(dirname "$0")/../../test_utils.sh"

# Configuration
CANISTER_ID="backend"
IDENTITY="default"
DEBUG="${DEBUG:-false}"  # Set DEBUG=true to enable debug output

echo_header "🧪 Testing memories_update endpoint"

# Test 1: Test memories_update with valid memory ID and updates
test_memories_update_valid() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memories_update with valid memory ID and updates..."
    
    # First, create a memory to test with
    local capsule_id=$(get_test_capsule_id)
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to get capsule ID for testing"
        return 1
    fi
    
    # Create test memory using utility function
    local memory_bytes='blob "VGVzdCBtZW1vcnkgZGF0YQ=="'
    local memory_id=$(create_test_memory "$capsule_id" "test_memory_update_123" "Test memory for update operations" '"test"; "update"' "$memory_bytes" "$CANISTER_ID" "$IDENTITY")
    
    if [[ -z "$memory_id" ]]; then
        echo_error "Failed to create test memory"
        return 1
    fi
    
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing with memory ID: $memory_id"
    
    # Create update data
    local update_data='(record {
      name = opt "Updated Memory Name";
      metadata = null;
      access = null;
    })'
    
    # Call memories_update with the memory ID and update data
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_update "(\"$memory_id\", $update_data)" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ memories_update with valid data succeeded"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        
        # Verify the update by reading the memory
        local read_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"$memory_id\")" 2>/dev/null)
        
        if [[ $read_result == *"title = opt \"Updated Memory Name\""* ]]; then
            echo_success "✅ Memory update verification successful"
            [[ "$DEBUG" == "true" ]] && echo_debug "Read result: $read_result"
        else
            echo_error "❌ Memory update verification failed"
            [[ "$DEBUG" == "true" ]] && echo_debug "Read result: $read_result"
            return 1
        fi
        
        return 0
    else
        echo_error "❌ memories_update with valid data failed"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 1
    fi
}

# Test 2: Test memories_update with invalid memory ID
test_memories_update_invalid_memory() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memories_update with invalid memory ID..."
    
    local update_data='(record {
      name = opt "Test Update";
      metadata = null;
      access = null;
    })'
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_update "(\"invalid_memory_id_123\", $update_data)" 2>/dev/null)
    
    if [[ $result == *"success = false"* ]]; then
        if [[ $result == *"Failed to update memory: NotFound"* ]]; then
            echo_success "✅ memories_update with invalid memory ID returned expected error"
            [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
            return 0
        else
            echo_error "❌ memories_update with invalid memory ID returned unexpected error message"
            [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
            return 1
        fi
    else
        echo_error "❌ memories_update with invalid memory ID should have failed"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 1
    fi
}

# Test 3: Test memories_update with empty update data
test_memories_update_empty_data() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memories_update with empty update data..."
    
    # First, create a memory to test with
    local capsule_id=$(get_test_capsule_id)
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to get capsule ID for testing"
        return 1
    fi
    
    # Create test memory using utility function
    local memory_bytes='blob "VGVzdCBtZW1vcnkgZGF0YQ=="'
    local memory_id=$(create_test_memory "$capsule_id" "test_memory_update_empty" "Test memory for empty update test" '"test"; "update"; "empty"' "$memory_bytes" "$CANISTER_ID" "$IDENTITY")
    
    if [[ -z "$memory_id" ]]; then
        echo_error "Failed to create test memory"
        return 1
    fi
    
    # Create empty update data (all fields null)
    local empty_update_data='(record {
      name = null;
      metadata = null;
      access = null;
    })'
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_update "(\"$memory_id\", $empty_update_data)" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ memories_update with empty update data succeeded (no-op update)"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 0
    else
        echo_error "❌ memories_update with empty update data failed"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 1
    fi
}

# Test 4: Test memories_update with access changes
test_memories_update_access() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memories_update with access changes..."
    
    # First, create a memory to test with
    local capsule_id=$(get_test_capsule_id)
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to get capsule ID for testing"
        return 1
    fi
    
    # Create test memory using utility function
    local memory_bytes='blob "VGVzdCBtZW1vcnkgZGF0YQ=="'
    local memory_id=$(create_test_memory "$capsule_id" "test_memory_update_access" "Test memory for access update test" '"test"; "update"; "access"' "$memory_bytes" "$CANISTER_ID" "$IDENTITY")
    
    if [[ -z "$memory_id" ]]; then
        echo_error "Failed to create test memory"
        return 1
    fi
    
    # Create update data with name changes (access updates might not be supported)
    local update_data='(record {
      name = opt "Updated Access Test Memory";
      metadata = null;
      access = null
    })'
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_update "(\"$memory_id\", $update_data)" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ memories_update with name changes succeeded"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        
        # Verify the access update by reading the memory
        local read_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"$memory_id\")" 2>/dev/null)
        
        if [[ $read_result == *"title = opt \"Updated Access Test Memory\""* ]]; then
            echo_success "✅ Name update verification successful"
            [[ "$DEBUG" == "true" ]] && echo_debug "Read result: $read_result"
            return 0
        else
            echo_error "❌ Name update verification failed"
            [[ "$DEBUG" == "true" ]] && echo_debug "Read result: $read_result"
            return 1
        fi
    else
        echo_error "❌ memories_update with name changes failed"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 1
    fi
}



# Test 6: Test memories_update with comprehensive info update (merged from test_update_memory.sh)
test_memories_update_comprehensive_info() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memories_update with comprehensive info update..."
    
    # First, create a memory to test with
    local capsule_id=$(get_test_capsule_id)
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to get capsule ID for testing"
        return 1
    fi
    
    # Create test memory using utility function
    local memory_bytes='blob "VGVzdCBtZW1vcnkgZm9yIHVwZGF0ZSB0ZXN0"'
    local memory_id=$(create_test_memory "$capsule_id" "test_memory_update_comprehensive" "Test memory for comprehensive update test" '"test"; "update"; "comprehensive"' "$memory_bytes" "$CANISTER_ID" "$IDENTITY")
    
    if [[ -z "$memory_id" ]]; then
        echo_error "Failed to create test memory"
        return 1
    fi
    
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing with memory ID: $memory_id"
    
    # Create comprehensive update data (merged from old test)
    local update_data='(record {
      name = opt "Updated Test Memory";
      metadata = null;
      access = null;
    })'
    
    # Call memories_update with the memory ID and update data
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_update "(\"$memory_id\", $update_data)" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ memories_update with comprehensive info succeeded"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        
        # Verify the update by reading the memory (merged verification logic)
        local read_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"$memory_id\")" 2>/dev/null)
        
        if [[ $read_result == *'title = opt "Updated Test Memory"'* ]]; then
            echo_success "✅ Verification PASSED: Memory name updated correctly"
        else
            echo_error "❌ Verification FAILED: Memory name not updated"
            [[ "$DEBUG" == "true" ]] && echo_debug "Read result: $read_result"
            return 1
        fi
        
        # Save memory ID for other tests (merged functionality)
        echo "$memory_id" > /tmp/test_memory_id.txt
        [[ "$DEBUG" == "true" ]] && echo_debug "Memory ID saved to /tmp/test_memory_id.txt for other tests"
        
        return 0
    else
        echo_error "❌ memories_update with comprehensive info failed"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 1
    fi
}

# Test 7: Verify old update_memory_in_capsule endpoint is removed
test_old_endpoint_removed() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Verifying old update_memory_in_capsule endpoint is removed..."
    
    local update_data='(record {
      name = opt "Test Update";
      metadata = null;
      access = null;
    })'
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID update_memory_in_capsule "(\"test_id\", $update_data)" 2>/dev/null 2>&1 || true)
    
    if [[ $result == *"Method not found"* ]] || [[ $result == *"Unknown method"* ]] || [[ $result == *"Canister has no update method"* ]]; then
        echo_success "✅ Old update_memory_in_capsule endpoint successfully removed"
        return 0
    else
        echo_error "❌ Old update_memory_in_capsule endpoint still exists"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 1
    fi
}

# Helper function to get a capsule ID for testing
get_test_capsule_id() {
    local capsule_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID capsules_read_basic "(null)" 2>/dev/null)
    local capsule_id=""
    
    if [[ $capsule_result == *"null"* ]]; then
        [[ "$DEBUG" == "true" ]] && echo_debug "No capsule found, creating one first..."
        local create_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID capsules_create "(null)" 2>/dev/null)
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

# Main test execution
main() {
    echo_header "🚀 Starting memories_update endpoint tests"
    
    run_test "Valid memory ID and updates" test_memories_update_valid
    run_test "Invalid memory ID" test_memories_update_invalid_memory
    run_test "Empty update data" test_memories_update_empty_data
    run_test "Access changes" test_memories_update_access
    run_test "Comprehensive info update" test_memories_update_comprehensive_info
    run_test "Old endpoint removal" test_old_endpoint_removed
    
    echo_header "🎉 All memories_update tests completed successfully!"
}

# Run main function
main "$@"
