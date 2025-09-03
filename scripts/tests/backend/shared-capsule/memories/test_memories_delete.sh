#!/bin/bash

# ==========================================
# Test script for memories_delete endpoint
# ==========================================
# Tests the new memories_delete(memory_id) endpoint that replaces delete_memory_from_capsule
# 
# Test scenarios:
# 1. Valid memory ID and deletion
# 2. Invalid memory ID
# 3. Empty memory ID
# 4. Cross-capsule deletion (if user has access to multiple capsules)
# 5. Verify old delete_memory_from_capsule endpoint is removed

# Load test configuration and utilities
source scripts/tests/backend/test_config.sh
source scripts/tests/backend/test_utils.sh

# Test configuration
TEST_NAME="memories_delete"
CANISTER_ID="${BACKEND_CANISTER_ID:-backend}"
IDENTITY="default"

# ==========================================
# Test Functions
# ==========================================

# Test 1: Test memories_delete with valid memory ID
test_memories_delete_valid() {
    echo_debug "Testing memories_delete with valid memory ID..."
    
    # First, create a memory to test with
    local capsule_id=$(get_test_capsule_id)
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to get capsule ID for testing"
        return 1
    fi
    
    # Create test memory data
    local memory_data='(record {
      blob_ref = record {
        kind = variant { ICPCapsule };
        locator = "test_memory_delete_valid";
        hash = null;
      };
      data = opt blob "VGVzdCBtZW1vcnkgZGF0YQ==";
    })'
    
    # Create the memory first
    local create_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule_id\", $memory_data)" 2>/dev/null)
    local memory_id=$(echo "$create_result" | grep -o 'memory_id = opt "[^"]*"' | sed 's/memory_id = opt "//' | sed 's/"//')
    
    if [[ -z "$memory_id" ]]; then
        echo_error "Failed to create test memory for deletion"
        return 1
    fi
    
    echo_debug "Testing with memory ID: $memory_id"
    
    # Save memory ID for other tests (like the old test_delete_memory.sh did)
    echo "$memory_id" > /tmp/test_memory_id.txt
    echo_debug "Saved memory ID to /tmp/test_memory_id.txt for other tests"
    
    # Verify memory exists before deletion
    local read_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"$memory_id\")" 2>/dev/null)
    
    if [[ $read_result == *"opt record"* ]] || [[ $read_result == *"record {"* ]]; then
        echo_success "✅ Memory exists before deletion"
    else
        echo_error "❌ Memory not found before deletion"
        return 1
    fi
    
    # Delete the memory
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_delete "(\"$memory_id\")" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ memories_delete with valid data succeeded"
        echo_debug "Result: $result"
        
        # Verify the memory was actually deleted
        local read_result_after=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"$memory_id\")" 2>/dev/null)
        
        if [[ $read_result_after == *"(null)"* ]]; then
            echo_success "✅ Memory deletion verification successful"
            echo_debug "Read result: $read_result_after"
        else
            echo_error "❌ Memory deletion verification failed - memory still exists"
            echo_debug "Read result: $read_result_after"
            return 1
        fi
    else
        echo_error "❌ memories_delete with valid data failed"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 2: Test memories_delete with invalid memory ID
test_memories_delete_invalid_memory() {
    echo_debug "Testing memories_delete with invalid memory ID..."
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_delete "(\"invalid_memory_id_123\")" 2>/dev/null)
    
    if [[ $result == *"success = false"* ]]; then
        if [[ $result == *"Memory not found in any accessible capsule"* ]] || [[ $result == *"No accessible capsule found for caller"* ]]; then
            echo_success "✅ memories_delete with invalid memory ID returned expected error"
            echo_debug "Result: $result"
        else
            echo_error "❌ memories_delete with invalid memory ID returned unexpected error message"
            echo_debug "Result: $result"
            return 1
        fi
    else
        echo_error "❌ memories_delete with invalid memory ID should have failed"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 3: Test memories_delete with empty memory ID
test_memories_delete_empty_id() {
    echo_debug "Testing memories_delete with empty memory ID..."
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_delete "(\"\")" 2>/dev/null)
    
    if [[ $result == *"success = false"* ]]; then
        if [[ $result == *"Memory not found in any accessible capsule"* ]] || [[ $result == *"No accessible capsule found for caller"* ]]; then
            echo_success "✅ memories_delete with empty memory ID returned expected error"
            echo_debug "Result: $result"
        else
            echo_error "❌ memories_delete with empty memory ID returned unexpected error message"
            echo_debug "Result: $result"
            return 1
        fi
    else
        echo_error "❌ memories_delete with empty memory ID should have failed"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 4: Test memories_delete with cross-capsule access
test_memories_delete_cross_capsule() {
    echo_debug "Testing memories_delete with cross-capsule access..."
    
    # First, create a memory in a capsule
    local capsule_id=$(get_test_capsule_id)
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to get capsule ID for testing"
        return 1
    fi
    
    # Create test memory data
    local memory_data='(record {
      blob_ref = record {
        kind = variant { ICPCapsule };
        locator = "test_memory_delete_cross";
        hash = null;
      };
      data = opt blob "VGVzdCBtZW1vcnkgZGF0YQ==";
    })'
    
    # Create the memory first
    local create_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule_id\", $memory_data)" 2>/dev/null)
    local memory_id=$(echo "$create_result" | grep -o 'memory_id = opt "[^"]*"' | sed 's/memory_id = opt "//' | sed 's/"//')
    
    if [[ -z "$memory_id" ]]; then
        echo_error "Failed to create test memory for cross-capsule deletion test"
        return 1
    fi
    
    # Delete the memory using memories_delete (which searches across all accessible capsules)
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_delete "(\"$memory_id\")" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ memories_delete with cross-capsule access succeeded"
        echo_debug "Result: $result"
        
        # Verify the memory was actually deleted
        local read_result_after=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"$memory_id\")" 2>/dev/null)
        
        if [[ $read_result_after == *"(null)"* ]]; then
            echo_success "✅ Cross-capsule memory deletion verification successful"
            echo_debug "Read result: $read_result_after"
        else
            echo_error "❌ Cross-capsule memory deletion verification failed"
            echo_debug "Read result: $read_result_after"
            return 1
        fi
    else
        echo_error "❌ memories_delete with cross-capsule access failed"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 5: Verify old delete_memory_from_capsule endpoint is removed
test_old_endpoint_removed() {
    echo_debug "Verifying old delete_memory_from_capsule endpoint is removed..."
    
    # Try to call the old endpoint - it should fail
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID delete_memory_from_capsule "(\"test_id\")" 2>/dev/null 2>&1 || true)
    
    # Check if the endpoint is completely removed
    if [[ $result == *"Canister has no update method"* ]] || [[ $result == *"Canister has no query method"* ]]; then
        echo_success "✅ Old delete_memory_from_capsule endpoint successfully removed"
    else
        echo_error "❌ Old delete_memory_from_capsule endpoint still exists"
        echo_debug "Result: $result"
        return 1
    fi
}

# ==========================================
# Main test execution
# ==========================================

main() {
    echo "=========================================="
    echo "🧪 Testing memories_delete endpoint"
    echo "=========================================="
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
    
    # Register user first (required for memory operations)
    echo_info "Registering user for memory operations..."
    local register_result=$(dfx canister call backend register 2>/dev/null)
    if ! echo "$register_result" | grep -q "true"; then
        echo_warn "User registration returned: $register_result"
    fi
    
    # Run tests
    echo_info "=== Testing memories_delete endpoint ==="
    run_test "Valid memory ID and deletion" test_memories_delete_valid
    run_test "Invalid memory ID" test_memories_delete_invalid_memory
    run_test "Empty memory ID" test_memories_delete_empty_id
    run_test "Cross-capsule deletion" test_memories_delete_cross_capsule
    run_test "Old endpoint removal" test_old_endpoint_removed
    
    echo ""
    echo "=========================================="
    echo "🎉 All memories_delete tests completed successfully!"
    echo "=========================================="
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
