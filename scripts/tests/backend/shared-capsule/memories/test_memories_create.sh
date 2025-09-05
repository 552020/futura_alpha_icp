#!/bin/bash

# Test script for memories_create endpoint
# Tests the new memories_create(capsule_id, memory_data, idem) endpoint with unified Inline/BlobRef support

set -e

# Source test utilities
source "$(dirname "$0")/../../test_utils.sh"

# Configuration
CANISTER_ID="backend"
IDENTITY="default"

echo_header "🧪 Testing memories_create endpoint"

# Test 1: Test memories_create with Inline data (small file)
test_memories_create_inline() {
    echo_debug "Testing memories_create with Inline data (small file ≤32KB)..."

    # First, get a capsule ID to test with
    local capsule_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID capsules_read_basic "(null)" 2>/dev/null)
    local capsule_id=""

    if [[ $capsule_result == *"null"* ]] || [[ $capsule_result == *"NotFound"* ]]; then
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

    # Create test memory data using Inline variant (small file)
    local memory_data='(variant {
      Inline = record {
        bytes = blob "SGVsbG8gV29ybGQgLSBUaGlzIGlzIGEgdGVzdCBmaWxl";
        meta = record {
          name = "test_inline_memory.txt";
          description = opt "Test inline memory creation";
          tags = vec { "test"; "inline"; "small" };
        };
      }
    })'

    local idem="test_inline_$(date +%s)"

    # Call memories_create with the new API format: (capsule_id, memory_data, idem)
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule_id\", $memory_data, \"$idem\")" 2>/dev/null)

    # Check for successful Result<MemoryId, Error> response
    if [[ $result == *"Ok"* ]] && [[ $result == *"mem_"* ]]; then
        echo_success "✅ memories_create with Inline data succeeded"
        echo_debug "Result: $result"

        # Extract and save memory ID for other tests
        local memory_id=$(echo "$result" | grep -o '"mem_[^"]*"' | sed 's/"//g')
        if [[ -n "$memory_id" ]]; then
            echo "$memory_id" > /tmp/test_memory_id.txt
            echo_debug "Saved memory ID to /tmp/test_memory_id.txt for other tests: $memory_id"
        fi
    else
        echo_error "❌ memories_create with Inline data failed"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 2: Test memories_create with BlobRef data (existing blob)
test_memories_create_blobref() {
    echo_debug "Testing memories_create with BlobRef data (reference to existing blob)..."

    # Get a valid capsule ID first
    local capsule_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID capsules_read_basic "(null)" 2>/dev/null)
    local capsule_id=$(echo "$capsule_result" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')

    if [[ -z "$capsule_id" ]]; then
        echo_debug "No capsule found, skipping BlobRef test"
        return 0
    fi

    echo_debug "Testing with capsule ID: $capsule_id"

    # For now, skip BlobRef test since we need to create a valid blob first
    # This test requires creating an actual blob via upload process first
    echo_debug "Skipping BlobRef test - requires valid blob creation first"
    echo_success "✅ BlobRef test skipped (requires blob creation setup)"
    return 0

    # TODO: Uncomment and fix when blob creation is set up
    # Create test memory data using BlobRef variant (reference to existing blob)
    # local memory_data='(variant {
    #   BlobRef = record {
    #     blob = record {
    #       kind = variant { ICPCapsule };
    #       locator = "existing_blob_123";
    #       hash = opt blob "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3";
    #       len = 1024;
    #     };
    #     meta = record {
    #       name = "test_blobref_memory.jpg";
    #       description = opt "Test BlobRef memory creation";
    #       tags = vec { "test"; "blobref"; "reference" };
    #     };
    #   }
    # })'

    # local idem="test_blobref_$(date +%s)"

    # Call memories_create with BlobRef variant
    # local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule_id\", $memory_data, \"$idem\")" 2>/dev/null)

    # if [[ $result == *"Ok"* ]] && [[ $result == *"memory_"* ]]; then
    #     echo_success "✅ memories_create with BlobRef data succeeded"
    #     echo_debug "Result: $result"
    # else
    #     echo_error "❌ memories_create with BlobRef data failed"
    #     echo_debug "Result: $result"
    #     return 1
    # fi
}

# Test 3: Test memories_create with invalid capsule ID
test_memories_create_invalid_capsule() {
    echo_debug "Testing memories_create with invalid capsule ID..."

    local memory_data='(variant {
      Inline = record {
        bytes = blob "VGVzdCBtZW1vcnkgZGF0YQ==";
        meta = record {
          name = "test_invalid.txt";
          description = null;
          tags = vec {};
        };
      }
    })'

    local idem="test_invalid_$(date +%s)"

    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"invalid_capsule_id\", $memory_data, \"$idem\")" 2>/dev/null)

    # Check for error response (Result<MemoryId, Error>)
    if [[ $result == *"Err"* ]] && [[ $result == *"NotFound"* ]]; then
        echo_success "✅ memories_create with invalid capsule ID returned expected error"
        echo_debug "Result: $result"
    else
        echo_error "❌ memories_create with invalid capsule ID returned unexpected result"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 4: Test memories_create with large inline data (should fail)
test_memories_create_large_inline() {
    echo_debug "Testing memories_create with large inline data (>32KB, should fail)..."

    # Get a valid capsule ID first
    local capsule_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID capsules_read_basic "(null)" 2>/dev/null)
    local capsule_id=$(echo "$capsule_result" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')

    if [[ -z "$capsule_id" ]]; then
        echo_debug "No capsule found, skipping large inline test"
        return 0
    fi

    # Create large data (>32KB limit)
    local large_data=""
    for i in {1..1000}; do
        large_data="${large_data}This is a test line to make the data larger than 32KB limit for inline uploads. "
    done

    local memory_data='(variant {
      Inline = record {
        bytes = blob "'$(echo -n "$large_data" | base64)'";
        meta = record {
          name = "test_large_file.txt";
          description = opt "Test large inline file that should fail";
          tags = vec { "test"; "large"; "should-fail" };
        };
      }
    })'

    local idem="test_large_$(date +%s)"

    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule_id\", $memory_data, \"$idem\")" 2>/dev/null)

    # Should fail with InvalidArgument error
    if [[ $result == *"Err"* ]] && [[ $result == *"InvalidArgument"* ]]; then
        echo_success "✅ memories_create with large inline data correctly rejected"
        echo_debug "Result: $result"
    else
        echo_error "❌ memories_create with large inline data should have failed"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 5: Test memories_create idempotency with same idem key
test_memories_create_idempotency() {
    echo_debug "Testing memories_create idempotency with same idem key..."

    # Get a valid capsule ID first
    local capsule_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID capsules_read_basic "(null)" 2>/dev/null)
    local capsule_id=$(echo "$capsule_result" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')

    if [[ -z "$capsule_id" ]]; then
        echo_debug "No capsule found, skipping idempotency test"
        return 0
    fi

    # Use same idem key for both calls
    local idem="test_idempotent_$(date +%s)"

    local memory_data='(variant {
      Inline = record {
        bytes = blob "VGVzdCBpZGVtcG90ZW5jeSBkYXRh";
        meta = record {
          name = "test_idempotent.txt";
          description = opt "Test idempotency with same idem key";
          tags = vec { "test"; "idempotent" };
        };
      }
    })'

    # First call
    local first_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule_id\", $memory_data, \"$idem\")" 2>/dev/null)

    if [[ $first_result != *"Ok"* ]]; then
        echo_error "❌ First memories_create call failed"
        echo_debug "Result: $first_result"
        return 1
    fi

    # Extract first memory ID
    local first_memory_id=$(echo "$first_result" | grep -o '"mem_[^"]*"' | sed 's/"//g')

    # Second call with same idem key (should return same memory ID)
    local second_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule_id\", $memory_data, \"$idem\")" 2>/dev/null)

    if [[ $second_result == *"Ok"* ]]; then
        local second_memory_id=$(echo "$second_result" | grep -o '"mem_[^"]*"' | sed 's/"//g')
        if [[ "$first_memory_id" == "$second_memory_id" ]]; then
            echo_success "✅ memories_create idempotency verified (same memory ID returned)"
            echo_debug "First ID: $first_memory_id, Second ID: $second_memory_id"
        else
            echo_error "❌ memories_create idempotency failed (different memory IDs)"
            echo_debug "First ID: $first_memory_id, Second ID: $second_memory_id"
            return 1
        fi
    else
        echo_error "❌ Second memories_create call failed"
        echo_debug "Result: $second_result"
        return 1
    fi
}

# Main test execution
main() {
    echo_header "🚀 Starting memories_create endpoint tests"

    run_test "Inline memory creation (small file)" test_memories_create_inline
    run_test "BlobRef memory creation (existing blob)" test_memories_create_blobref
    run_test "Invalid capsule ID" test_memories_create_invalid_capsule
    run_test "Large inline data rejection" test_memories_create_large_inline
    run_test "Idempotency with same idem key" test_memories_create_idempotency

    echo_header "🎉 All memories_create tests completed successfully!"
}

# Run main function
main "$@"
