#!/bin/bash

# Test script for memories_create endpoint
# Tests the enhanced memories_create endpoint with optional parameters for all memory types:
# memories_create(capsule_id, bytes, blob_ref, external_location, external_storage_key, external_url, external_size, external_hash, asset_metadata, idem)

set -e

# Source test utilities
unset -f get_test_capsule_id 2>/dev/null || true
source "$(dirname "$0")/../shared_test_utils.sh"
source "$(dirname "$0")/../upload/upload_test_utils.sh"

# Configuration
CANISTER_ID="backend"
IDENTITY="default"

echo_header "🧪 Testing memories_create endpoint"

# Test 1: Test memories_create with Inline data (small file)
test_memories_create_inline() {
    echo_debug "Testing memories_create with Inline data (small file ≤32KB)..."
    echo_debug "Function variables: CANISTER_ID=$CANISTER_ID, IDENTITY=$IDENTITY"

    # Get a capsule ID to test with using the helper function
    local capsule_id=$(get_test_capsule_id $CANISTER_ID $IDENTITY)
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to get capsule ID for testing"
        return 1
    fi

    echo_debug "Testing with capsule ID: $capsule_id"

    # Create test memory using the working utility function
    local memory_bytes='blob "SGVsbG8gV29ybGQgLSBUaGlzIGlzIGEgdGVzdCBmaWxl"'
    local memory_id=$(create_test_memory "$capsule_id" "test_inline_memory" "Test inline memory creation" '"test"; "inline"; "small"' "$memory_bytes" "$CANISTER_ID" "$IDENTITY")
    
    if [[ -z "$memory_id" ]]; then
        echo_error "Failed to create test memory"
        return 1
    fi
    
    echo_success "✅ memories_create with Inline data succeeded"
    echo_debug "Memory ID: $memory_id"
    
    # Save memory ID for other tests
    echo "$memory_id" > /tmp/test_memory_id.txt
    echo_debug "Saved memory ID to /tmp/test_memory_id.txt for other tests: $memory_id"
}

# Test 2: Test memories_create with BlobRef data (existing blob)
test_memories_create_blobref() {
    echo_debug "Testing memories_create with BlobRef data (reference to existing blob)..."
    
    # Get a valid capsule ID first
    local capsule_id=$(get_test_capsule_id $CANISTER_ID $IDENTITY)
    
    if [[ -z "$capsule_id" ]]; then
        echo_debug "No capsule found, skipping BlobRef test"
        return 0
    fi

    echo_debug "Testing with capsule ID: $capsule_id"

    # Step 1: Create a blob using the utility function
    echo_debug "Step 1: Creating blob using utility function..."
    
    # Create minimal test blob (1 byte = 8 bits as you suggested!)
    local blob_output=$(create_minimal_test_blob "$capsule_id" "$CANISTER_ID" "$IDENTITY")
    
    if [[ $? -ne 0 ]]; then
        echo_error "❌ Failed to create test blob"
        return 1
    fi
    
    # Extract blob information
    local blob_id=$(extract_blob_info "$blob_output" "ID")
    local blob_hash=$(extract_blob_info "$blob_output" "HASH")
    local blob_size=$(extract_blob_info "$blob_output" "SIZE")
    local blob_locator=$(extract_blob_info "$blob_output" "LOCATOR")
    
    echo_debug "Created blob - ID: $blob_id, Hash: $blob_hash, Size: $blob_size, Locator: $blob_locator"
    
    # Step 2: Now test memories_create with BlobRef
    echo_debug "Step 2: Testing memories_create with BlobRef..."
    
    # Create blob reference using the actual blob information
    local blob_ref='(record {
      locator = "'$blob_locator'";
      hash = opt blob "'$blob_hash'";
      len = '$blob_size';
    })'

    local asset_metadata='(variant {
      Document = record {
        base = record {
          name = "test_blobref_document.txt";
          description = opt "Test BlobRef memory creation";
          tags = vec { "test"; "blobref"; "document" };
          asset_type = variant { Original };
          bytes = '$blob_size';
          mime_type = "text/plain";
          sha256 = opt blob "'$blob_hash'";
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

    local idem="test_blobref_$(date +%s)"

    # Call memories_create with BlobRef: (capsule_id, null, blob_ref, null, null, null, null, null, asset_metadata, idem)
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule_id\", null, opt $blob_ref, null, null, null, null, null, $asset_metadata, \"$idem\")" 2>/dev/null)

    if [[ $result == *"Ok"* ]] && echo "$result" | grep -q "[0-9a-f-]\{36\}"; then
        echo_success "✅ memories_create with BlobRef data succeeded"
        echo_debug "Result: $result"
    else
        echo_error "❌ memories_create with BlobRef data failed"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 3: Test memories_create with external asset (S3)
test_memories_create_external() {
    echo_debug "Testing memories_create with external asset (S3)..."

    # Get a valid capsule ID first
    local capsule_id=$(get_test_capsule_id $CANISTER_ID $IDENTITY)
    
    if [[ -z "$capsule_id" ]]; then
        echo_debug "No capsule found, skipping external asset test"
        return 0
    fi

    echo_debug "Testing with capsule ID: $capsule_id"

    local asset_metadata='(variant {
      Document = record {
        base = record {
          name = "test_external_video.mp4";
          description = opt "Test external asset creation";
          tags = vec { "test"; "external"; "video" };
          asset_type = variant { Original };
          bytes = 5000000;
          mime_type = "video/mp4";
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
        format = null;
        language = null;
        word_count = null;
      }
    })'

    local idem="test_external_$(date +%s)"

    # Create a proper 32-byte hash for external asset
    local external_hash=$(create_test_hash "test_external_hash")
    echo_debug "Created external hash: '$external_hash'"

    # Call memories_create with external asset: (capsule_id, null, null, external_location, external_storage_key, external_url, external_size, external_hash, asset_metadata, idem)
    echo_debug "Calling memories_create with external asset..."
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule_id\", null, null, opt variant { S3 }, opt \"s3://bucket/test_video.mp4\", opt \"https://s3.amazonaws.com/bucket/test_video.mp4\", opt 5000000, opt blob \"$external_hash\", $asset_metadata, \"$idem\")" 2>/dev/null)

    if [[ $result == *"Ok"* ]] && echo "$result" | grep -q "[0-9a-f-]\{36\}"; then
        echo_success "✅ memories_create with external asset succeeded"
        echo_debug "Result: $result"
    else
        echo_error "❌ memories_create with external asset failed"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 4: Test memories_create with invalid capsule ID
test_memories_create_invalid_capsule() {
    echo_debug "Testing memories_create with invalid capsule ID..."

    local memory_bytes='blob "VGVzdCBtZW1vcnkgZGF0YQ=="'
    local asset_metadata='(variant {
      Document = record {
        base = record {
          name = "test_invalid.txt";
          description = null;
          tags = vec {};
          asset_type = variant { Original };
          bytes = 24;
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

    local idem="test_invalid_$(date +%s)"

    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"invalid_capsule_id\", opt $memory_bytes, null, null, null, null, null, null, $asset_metadata, \"$idem\")" 2>/dev/null)

    # Check for error response (Result<MemoryId, Error>)
    if [[ $result == *"Err"* ]] && ([[ $result == *"NotFound"* ]] || [[ $result == *"Unauthorized"* ]]); then
        echo_success "✅ memories_create with invalid capsule ID returned expected error"
        echo_debug "Result: $result"
    else
        echo_error "❌ memories_create with invalid capsule ID returned unexpected result"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 5: Test memories_create with large inline data (should fail)
test_memories_create_large_inline() {
    echo_debug "Testing memories_create with large inline data (>32KB, should fail)..."

    # Get a valid capsule ID first
    local capsule_id=$(get_test_capsule_id $CANISTER_ID $IDENTITY)
    
    if [[ -z "$capsule_id" ]]; then
        echo_debug "No capsule found, skipping large inline test"
        return 0
    fi

    # Create large data (>32KB limit)
    local large_data=""
    for i in {1..1000}; do
        large_data="${large_data}This is a test line to make the data larger than 32KB limit for inline uploads. "
    done

    local memory_bytes='blob "'$(echo -n "$large_data" | base64)'"'
    local asset_metadata='(variant {
      Document = record {
        base = record {
          name = "test_large_file.txt";
          description = opt "Test large inline file that should fail";
          tags = vec { "test"; "large"; "should-fail" };
          asset_type = variant { Original };
          bytes = '$(echo -n "$large_data" | base64 | wc -c)';
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

    local idem="test_large_$(date +%s)"

    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule_id\", opt $memory_bytes, null, null, null, null, null, null, $asset_metadata, \"$idem\")" 2>/dev/null)

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

# Test 6: Test memories_create idempotency with same idem key
test_memories_create_idempotency() {
    echo_debug "Testing memories_create idempotency with same idem key..."

    # Get a valid capsule ID first
    local capsule_id=$(get_test_capsule_id $CANISTER_ID $IDENTITY)
    
    if [[ -z "$capsule_id" ]]; then
        echo_debug "No capsule found, skipping idempotency test"
        return 0
    fi

    # Use same idem key for both calls
    local idem="test_idempotent_$(date +%s)"
    local memory_bytes='blob "VGVzdCBpZGVtcG90ZW5jeSBkYXRh"'

    # Both calls need to use the same idem key, so we'll do them manually with proper byte size handling
    local base64_content=$(echo "$memory_bytes" | sed 's/blob "//' | sed 's/"//')
    local actual_bytes=$(echo -n "$base64_content" | base64 -d | wc -c)
    local vec_content=$(b64_to_vec "$base64_content")
    
    local asset_metadata='(variant {
      Document = record {
        base = record {
          name = "test_idempotent";
          description = opt "Test idempotency with same idem key";
          tags = vec { "test"; "idempotent" };
          asset_type = variant { Original };
          bytes = '$actual_bytes';
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

    # First call
    local first_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule_id\", opt $vec_content, null, null, null, null, null, null, $asset_metadata, \"$idem\")" 2>/dev/null)

    if [[ $first_result != *"Ok"* ]]; then
        echo_error "❌ First memories_create call failed"
        echo_debug "Result: $first_result"
        return 1
    fi

    # Extract first memory ID
    local first_memory_id=$(echo "$first_result" | grep -o 'Ok = "[^"]*"' | sed 's/Ok = "//' | sed 's/"//')

    # Second call with same idem key (should return same memory ID)
    local second_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$capsule_id\", opt $vec_content, null, null, null, null, null, null, $asset_metadata, \"$idem\")" 2>/dev/null)

    if [[ $second_result == *"Ok"* ]]; then
        local second_memory_id=$(echo "$second_result" | grep -o 'Ok = "[^"]*"' | sed 's/Ok = "//' | sed 's/"//')
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
    run_test "External asset creation (S3)" test_memories_create_external
    run_test "Invalid capsule ID" test_memories_create_invalid_capsule
    run_test "Large inline data rejection" test_memories_create_large_inline
    run_test "Idempotency with same idem key" test_memories_create_idempotency

    echo_header "🎉 All memories_create tests completed successfully!"
}

# Run main function
main "$@"
