#!/bin/bash

# Test script for memories_create endpoint
# Tests the new memories_create(capsule_id, memory_data) endpoint that replaces add_memory_to_capsule

set -e

# Source test utilities
source "$(dirname "$0")/../test_utils.sh"

# Configuration
CANISTER_ID="backend"
IDENTITY="default"

echo_header "🧪 Testing memories_create endpoint"

# Test 1: Test memories_create with valid capsule ID and memory data
test_memories_create_valid() {
    echo_debug "Testing memories_create with valid capsule ID and memory data..."
    
    # First, get a capsule ID to test with
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
    
    echo_debug "Testing with capsule ID: $capsule_id"
    
    # Create test memory data
    local memory_data='(record {
      blob_ref = record {
        kind = variant { ICPCapsule };
        locator = "test_memory_123";
        hash = null;
      };
      data = opt blob "VGVzdCBtZW1vcnkgZGF0YQ==";
    })'
    
    # Call memories_create with the capsule ID and memory data
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule_id\", $memory_data)" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ memories_create with valid data succeeded"
        echo_debug "Result: $result"
    else
        echo_error "❌ memories_create with valid data failed"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 2: Test memories_create with invalid capsule ID
test_memories_create_invalid_capsule() {
    echo_debug "Testing memories_create with invalid capsule ID..."
    
    local memory_data='(record {
      blob_ref = record {
        kind = variant { ICPCapsule };
        locator = "test_memory_invalid";
        hash = null;
      };
      data = opt blob "VGVzdCBtZW1vcnkgZGF0YQ==";
    })'
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"invalid_capsule_id\", $memory_data)" 2>/dev/null)
    
    if [[ $result == *"success = false"* ]] && [[ $result == *"Capsule not found or access denied"* ]]; then
        echo_success "✅ memories_create with invalid capsule ID returned expected error"
        echo_debug "Result: $result"
    else
        echo_error "❌ memories_create with invalid capsule ID returned unexpected result"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 3: Test memories_create idempotency
test_memories_create_idempotency() {
    echo_debug "Testing memories_create idempotency..."
    
    # Get a valid capsule ID first
    local capsule_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID capsules_read_basic "(null)" 2>/dev/null)
    local capsule_id=$(echo "$capsule_result" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')
    
    if [[ -z "$capsule_id" ]]; then
        echo_debug "No capsule found, skipping idempotency test"
        return 0
    fi
    
    local memory_data='(record {
      blob_ref = record {
        kind = variant { ICPCapsule };
        locator = "test_memory_idempotent";
        hash = null;
      };
      data = opt blob "VGVzdCBtZW1vcnkgZGF0YQ==";
    })'
    
    # First call
    local first_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule_id\", $memory_data)" 2>/dev/null)
    
    if [[ $first_result != *"success = true"* ]]; then
        echo_error "❌ First memories_create call failed"
        echo_debug "Result: $first_result"
        return 1
    fi
    
    # Second call with same data (should be idempotent)
    local second_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule_id\", $memory_data)" 2>/dev/null)
    
    if [[ $second_result == *"success = true"* ]] && [[ $second_result == *"Memory already exists with this UUID"* ]]; then
        echo_success "✅ memories_create idempotency verified"
        echo_debug "Second call result: $second_result"
    else
        echo_error "❌ memories_create idempotency failed"
        echo_debug "Second call result: $second_result"
        return 1
    fi
}

# Test 4: Test memories_create with empty memory data
test_memories_create_empty_data() {
    echo_debug "Testing memories_create with empty memory data..."
    
    # Get a valid capsule ID first
    local capsule_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID capsules_read_basic "(null)" 2>/dev/null)
    local capsule_id=$(echo "$capsule_result" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')
    
    if [[ -z "$capsule_id" ]]; then
        echo_debug "No capsule found, skipping empty data test"
        return 0
    fi
    
    local empty_memory_data='(record {
      blob_ref = record {
        kind = variant { ICPCapsule };
        locator = "test_memory_empty";
        hash = null;
      };
      data = opt blob "";
    })'
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule_id\", $empty_memory_data)" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ memories_create with empty data succeeded"
        echo_debug "Result: $result"
    else
        echo_error "❌ memories_create with empty data failed"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 5: Verify old add_memory_to_capsule endpoint is removed
test_old_endpoint_removed() {
    echo_debug "Verifying old add_memory_to_capsule endpoint is removed..."
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID add_memory_to_capsule "(\"test_id\", record { blob_ref = record { kind = variant { ICPCapsule }; locator = \"test\"; hash = null; }; data = null; })" 2>/dev/null 2>&1 || true)
    
    if [[ $result == *"Method not found"* ]] || [[ $result == *"Unknown method"* ]] || [[ $result == *"Canister has no update method"* ]]; then
        echo_success "✅ Old add_memory_to_capsule endpoint successfully removed"
    else
        echo_error "❌ Old add_memory_to_capsule endpoint still exists"
        echo_debug "Result: $result"
        return 1
    fi
}

# Main test execution
main() {
    echo_header "🚀 Starting memories_create endpoint tests"
    
    run_test "Valid capsule ID and memory data" test_memories_create_valid
    run_test "Invalid capsule ID" test_memories_create_invalid_capsule
    run_test "Idempotency" test_memories_create_idempotency
    run_test "Empty memory data" test_memories_create_empty_data
    run_test "Old endpoint removal" test_old_endpoint_removed
    
    echo_header "🎉 All memories_create tests completed successfully!"
}

# Run main function
main "$@"
