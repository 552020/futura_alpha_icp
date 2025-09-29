#!/bin/bash

# Test script for memories_update endpoint
# Tests the new memories_update(memory_id, updates) endpoint that replaces update_memory_in_capsule

set -e

# Source test utilities
source "$(dirname "$0")/../../test_utils.sh"

# Configuration
CANISTER_ID="backend"
IDENTITY="default"

echo_header "🧪 Testing memories_update endpoint"

# Test 1: Test memories_update with valid memory ID and updates
test_memories_update_valid() {
    echo_debug "Testing memories_update with valid memory ID and updates..."
    
    # First, create a memory to test with
    local capsule_id=$(get_test_capsule_id)
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to get capsule ID for testing"
        return 1
    fi
    
    # Create test memory data using new API format
    local memory_bytes='blob "VGVzdCBtZW1vcnkgZGF0YQ=="'
    local asset_metadata='(variant {
      Document = record {
        base = record {
          name = "test_memory_update_123";
          description = opt "Test memory for update operations";
          tags = vec { "test"; "update" };
          asset_type = variant { Original };
          bytes = 16;
          mime_type = "text/plain";
          sha256 = null;
          width = null;
          height = null;
          url = null;
          storage_key = null;
          bucket = null;
          asset_location = null;
          processing_status = null;
          processing_error = null;
          created_at = 0;
          updated_at = 0;
          deleted_at = null;
        };
        page_count = null;
        document_type = null;
        language = null;
        word_count = null;
      }
    })'
    
    local idem="test_update_$(date +%s)"
    
    # Create the memory first
    local create_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule_id\", $memory_bytes, $asset_metadata, \"$idem\")" 2>/dev/null)
    
    if [[ $create_result != *"Ok"* ]]; then
        echo_error "Failed to create test memory"
        echo_debug "Create result: $create_result"
        return 1
    fi
    
    # Extract memory ID from creation result
    local memory_id=$(echo "$create_result" | grep -o '"mem_[^"]*"' | sed 's/"//g')
    
    if [[ -z "$memory_id" ]]; then
        echo_error "Failed to extract memory ID from creation result"
        return 1
    fi
    
    echo_debug "Testing with memory ID: $memory_id"
    
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
        echo_debug "Result: $result"
        
        # Verify the update by reading the memory
        local read_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"$memory_id\")" 2>/dev/null)
        
        if [[ $read_result == *"title = opt \"Updated Memory Name\""* ]]; then
            echo_success "✅ Memory update verification successful"
            echo_debug "Read result: $read_result"
        else
            echo_error "❌ Memory update verification failed"
            echo_debug "Read result: $read_result"
            return 1
        fi
    else
        echo_error "❌ memories_update with valid data failed"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 2: Test memories_update with invalid memory ID
test_memories_update_invalid_memory() {
    echo_debug "Testing memories_update with invalid memory ID..."
    
    local update_data='(record {
      name = opt "Test Update";
      metadata = null;
      access = null;
    })'
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_update "(\"invalid_memory_id_123\", $update_data)" 2>/dev/null)
    
    if [[ $result == *"success = false"* ]]; then
        if [[ $result == *"Memory not found in any accessible capsule"* ]] || [[ $result == *"No accessible capsule found for caller"* ]]; then
            echo_success "✅ memories_update with invalid memory ID returned expected error"
            echo_debug "Result: $result"
        else
            echo_error "❌ memories_update with invalid memory ID returned unexpected error message"
            echo_debug "Result: $result"
            return 1
        fi
    else
        echo_error "❌ memories_update with invalid memory ID should have failed"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 3: Test memories_update with empty update data
test_memories_update_empty_data() {
    echo_debug "Testing memories_update with empty update data..."
    
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
          name = "test_memory_update_empty";
          description = opt "Test memory for empty update test";
          tags = vec { "test"; "update"; "empty" };
        };
      }
    })'
    
    local idem="test_update_empty_$(date +%s)"
    
    # Create the memory first
    local create_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule_id\", $memory_data, \"$idem\")" 2>/dev/null)
    local memory_id=$(echo "$create_result" | grep -o '"mem_[^"]*"' | sed 's/"//g')
    
    # Create empty update data (all fields null)
    local empty_update_data='(record {
      name = null;
      metadata = null;
      access = null;
    })'
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_update "(\"$memory_id\", $empty_update_data)" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ memories_update with empty update data succeeded (no-op update)"
        echo_debug "Result: $result"
    else
        echo_error "❌ memories_update with empty update data failed"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 4: Test memories_update with access changes
test_memories_update_access() {
    echo_debug "Testing memories_update with access changes..."
    
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
        locator = "test_memory_update_metadata";
        hash = null;
      };
      data = opt blob "VGVzdCBtZW1vcnkgZGF0YQ==";
    })'
    
    # Create the memory first
    local create_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule_id\", $memory_data, \"$idem\")" 2>/dev/null)
    local memory_id=$(echo "$create_result" | grep -o '"mem_[^"]*"' | sed 's/"//g')
    
    # Create update data with metadata changes (simplified)
    local update_data='(record {
      name = null;
      metadata = null;
      access = opt (variant { Public })
    })'
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_update "(\"$memory_id\", $update_data)" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ memories_update with access changes succeeded"
        echo_debug "Result: $result"
        
        # Verify the access update by reading the memory
        local read_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"$memory_id\")" 2>/dev/null)
        
        if [[ $read_result == *"access = variant { Public }"* ]]; then
            echo_success "✅ Access update verification successful"
            echo_debug "Read result: $read_result"
        else
            echo_error "❌ Access update verification failed"
            echo_debug "Read result: $read_result"
            return 1
        fi
    else
        echo_error "❌ memories_update with access changes failed"
        echo_debug "Result: $result"
        return 1
    fi
}



# Test 6: Test memories_update with comprehensive info update (merged from test_update_memory.sh)
test_memories_update_comprehensive_info() {
    echo_debug "Testing memories_update with comprehensive info update..."
    
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
        locator = "test_memory_update_comprehensive";
        hash = null;
      };
      data = opt blob "VGVzdCBtZW1vcnkgZm9yIHVwZGF0ZSB0ZXN0";
    })'
    
    # Create the memory first
    local create_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule_id\", $memory_data, \"$idem\")" 2>/dev/null)
    
    if [[ $create_result != *"Ok"* ]]; then
        echo_error "Failed to create test memory"
        echo_debug "Create result: $create_result"
        return 1
    fi
    
    # Extract memory ID from creation result
    local memory_id=$(echo "$create_result" | grep -o '"mem_[^"]*"' | sed 's/"//g')
    
    if [[ -z "$memory_id" ]]; then
        echo_error "Failed to extract memory ID from creation result"
        return 1
    fi
    
    echo_debug "Testing with memory ID: $memory_id"
    
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
        echo_debug "Result: $result"
        
        # Verify the update by reading the memory (merged verification logic)
        local read_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"$memory_id\")" 2>/dev/null)
        
        if [[ $read_result == *'name = "Updated Test Memory"'* ]]; then
            echo_success "✅ Verification PASSED: Memory name updated correctly"
        else
            echo_error "❌ Verification FAILED: Memory name not updated"
            echo_debug "Read result: $read_result"
            return 1
        fi
        
        # Save memory ID for other tests (merged functionality)
        echo "$memory_id" > /tmp/test_memory_id.txt
        echo_debug "Memory ID saved to /tmp/test_memory_id.txt for other tests"
        
    else
        echo_error "❌ memories_update with comprehensive info failed"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 7: Verify old update_memory_in_capsule endpoint is removed
test_old_endpoint_removed() {
    echo_debug "Verifying old update_memory_in_capsule endpoint is removed..."
    
    local update_data='(record {
      name = opt "Test Update";
      metadata = null;
      access = null;
    })'
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID update_memory_in_capsule "(\"test_id\", $update_data)" 2>/dev/null 2>&1 || true)
    
    if [[ $result == *"Method not found"* ]] || [[ $result == *"Unknown method"* ]] || [[ $result == *"Canister has no update method"* ]]; then
        echo_success "✅ Old update_memory_in_capsule endpoint successfully removed"
    else
        echo_error "❌ Old update_memory_in_capsule endpoint still exists"
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
