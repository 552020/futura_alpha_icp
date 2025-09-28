#!/bin/bash

# Test script for memories_read endpoint
# Tests the new memories_read(memory_id) endpoint that replaces get_memory_from_capsule

set -e

# Source test utilities
source "$(dirname "$0")/../../test_utils.sh"

# Configuration
CANISTER_ID="backend"
IDENTITY="default"

echo_header "🧪 Testing memories_read endpoint"

# Test 1: Test memories_read with valid memory ID
test_memories_read_valid() {
    echo_debug "Testing memories_read with valid memory ID..."
    
    # First, create a memory to test with
    local capsule_id=$(get_test_capsule_id)
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to get capsule ID for testing"
        return 1
    fi
    
    # Create test memory data using new MemoryData format
    local memory_data='(variant {
      Inline = record {
        bytes = blob "VGVzdCBtZW1vcnkgZGF0YQ==";
        meta = record {
          name = "test_memory_read_123";
          description = opt "Test memory for read operations";
          tags = vec { "test"; "read" };
        };
      }
    })'
    
    local idem="test_read_$(date +%s)"
    
    # Create the memory first
    local create_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule_id\", $memory_data, \"$idem\")" 2>/dev/null)
    
    if [[ $create_result != *"Ok"* ]]; then
        echo_error "Failed to create test memory"
        echo_debug "Create result: $create_result"
        return 1
    fi
    
    # Extract memory ID from creation result (new format returns Ok = "memory_id")
    local memory_id=$(echo "$create_result" | grep -o '"mem_[^"]*"' | sed 's/"//g')
    
    if [[ -z "$memory_id" ]]; then
        echo_error "Failed to extract memory ID from creation result"
        return 1
    fi
    
    echo_debug "Testing with memory ID: $memory_id"
    
    # Call memories_read with the memory ID
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"$memory_id\")" 2>/dev/null)
    
    if [[ $result == *"Ok = record"* ]] && [[ $result == *"id = \"$memory_id\""* ]]; then
        echo_success "✅ memories_read with valid memory ID succeeded"
        echo_debug "Result: $result"
    else
        echo_error "❌ memories_read with valid memory ID failed"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 2: Test memories_read with invalid memory ID
test_memories_read_invalid() {
    echo_debug "Testing memories_read with invalid memory ID..."
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"invalid_memory_id_123\")" 2>/dev/null)
    
    if [[ $result == *"Err"* ]] || [[ $result == "(null)" ]]; then
        echo_success "✅ memories_read with invalid memory ID returned null as expected"
        echo_debug "Result: $result"
    else
        echo_error "❌ memories_read with invalid memory ID returned unexpected result"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 3: Test memories_read with empty memory ID
test_memories_read_empty() {
    echo_debug "Testing memories_read with empty memory ID..."
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"\")" 2>/dev/null)
    
    if [[ $result == *"Err"* ]] || [[ $result == "(null)" ]]; then
        echo_success "✅ memories_read with empty memory ID returned null as expected"
        echo_debug "Result: $result"
    else
        echo_error "❌ memories_read with empty memory ID returned unexpected result"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 4: Test memories_read across different capsules
test_memories_read_cross_capsules() {
    echo_debug "Testing memories_read across different capsules..."
    
    # Create two capsules and memories in each
    local capsule1_id=$(get_test_capsule_id)
    local capsule2_id=$(get_test_capsule_id)
    
    if [[ -z "$capsule1_id" ]] || [[ -z "$capsule2_id" ]]; then
        echo_error "Failed to get capsule IDs for testing"
        return 1
    fi
    
    # Create memory in first capsule
    local memory1_data='(variant {
      Inline = record {
        bytes = blob "VGVzdCBtZW1vcnkgMQ==";
        meta = record {
          name = "cross_capsule_test_1";
          description = opt "Test memory 1 for cross-capsule test";
          tags = vec { "test"; "cross-capsule" };
        };
      }
    })'
    
    local idem1="test_cross_1_$(date +%s)"
    local create1_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule1_id\", $memory1_data, \"$idem1\")" 2>/dev/null)
    local memory1_id=$(echo "$create1_result" | grep -o '"mem_[^"]*"' | sed 's/"//g')
    
    # Create memory in second capsule
    local memory2_data='(variant {
      Inline = record {
        bytes = blob "VGVzdCBtZW1vcnkgMg==";
        meta = record {
          name = "cross_capsule_test_2";
          description = opt "Test memory 2 for cross-capsule test";
          tags = vec { "test"; "cross-capsule" };
        };
      }
    })'
    
    local idem2="test_cross_2_$(date +%s)"
    local create2_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule2_id\", $memory2_data, \"$idem2\")" 2>/dev/null)
    local memory2_id=$(echo "$create2_result" | grep -o '"mem_[^"]*"' | sed 's/"//g')
    
    # Test reading both memories
    local read1_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"$memory1_id\")" 2>/dev/null)
    local read2_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"$memory2_id\")" 2>/dev/null)
    
    if [[ $read1_result == *"Ok = record"* ]] && [[ $read1_result == *"id = \"$memory1_id\""* ]] && [[ $read2_result == *"Ok = record"* ]] && [[ $read2_result == *"id = \"$memory2_id\""* ]]; then
        echo_success "✅ memories_read successfully retrieved memories from different capsules"
        echo_debug "Memory 1 result: $read1_result"
        echo_debug "Memory 2 result: $read2_result"
    else
        echo_error "❌ memories_read failed to retrieve memories from different capsules"
        echo_debug "Memory 1 result: $read1_result"
        echo_debug "Memory 2 result: $read2_result"
        return 1
    fi
}

# Test 5: Test memories_read with saved memory ID (persistence test)
test_memories_read_persistence() {
    echo_debug "Testing memories_read with saved memory ID for persistence..."
    
    if [ ! -f /tmp/test_memory_id.txt ]; then
        echo_debug "No saved memory ID found, skipping persistence test"
        return 0
    fi
    
    local saved_memory_id=$(cat /tmp/test_memory_id.txt)
    echo_debug "Using saved memory ID: $saved_memory_id"
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"$saved_memory_id\")" 2>/dev/null)
    
    if [[ $result == *"Ok = record"* ]] && [[ $result == *"id = \"$saved_memory_id\""* ]]; then
        echo_success "✅ memories_read with saved memory ID succeeded (persistence verified)"
        echo_debug "Result: $result"
    else
        echo_error "❌ memories_read with saved memory ID failed"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 6: Verify old get_memory_from_capsule endpoint is removed
test_old_endpoint_removed() {
    echo_debug "Verifying old get_memory_from_capsule endpoint is removed..."
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID get_memory_from_capsule "(\"test_id\")" 2>/dev/null 2>&1 || true)
    
    if [[ $result == *"Method not found"* ]] || [[ $result == *"Unknown method"* ]] || [[ $result == *"Canister has no query method"* ]] || [[ $result == *"Canister has no update method"* ]]; then
        echo_success "✅ Old get_memory_from_capsule endpoint successfully removed"
    else
        echo_error "❌ Old get_memory_from_capsule endpoint still exists"
        echo_debug "Result: $result"
        return 1
    fi
}

# Helper function to get a capsule ID for testing
get_test_capsule_id() {
    local capsule_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID capsules_read_basic "(null)" 2>/dev/null)
    local capsule_id=""
    
    if [[ $capsule_result == *"null"* ]]; then
        echo_debug "No capsule found, creating one first..."
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
