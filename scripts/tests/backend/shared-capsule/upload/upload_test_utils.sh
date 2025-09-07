#!/bin/bash

# Upload Test Utilities
# Shared functions for upload-related test scripts

# Helper function to check if response indicates success (Result<T, Error> format)
is_success() {
    local response="$1"
    echo "$response" | grep -q "variant {" && (echo "$response" | grep -q "Ok =" || echo "$response" | grep -q "Ok }")
}

# Helper function to check if response indicates failure (Result<T, Error> format)
is_failure() {
    local response="$1"
    echo "$response" | grep -q "variant {" && echo "$response" | grep -q "Err ="
}

# Helper function to print error messages
echo_error() {
    echo "[ERROR] $1" >&2
}

# Helper function to print info messages
echo_info() {
    echo "[INFO] $1"
}

# Helper function to get a capsule ID for testing
get_test_capsule_id() {
    local capsule_result=$(dfx canister call backend capsules_read_basic "(null)" 2>/dev/null)
    local capsule_id=""
    
    if [[ $capsule_result == *"null"* ]]; then
        echo_info "No capsule found, creating one first..."
        local create_result=$(dfx canister call backend capsules_create "(null)" 2>/dev/null)
        capsule_id=$(echo "$create_result" | grep -o 'id = "[^"]*"' | sed 's/id = "//' | sed 's/"//')
    else
        capsule_id=$(echo "$capsule_result" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')
    fi
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to get capsule ID for testing"
        return 1
    fi
    
    echo "$capsule_id"
}

# Helper function to create test chunk data
create_test_chunk() {
    local chunk_index="$1"
    local chunk_size="$2"
    
    # Create chunk data with pattern based on index
    local pattern=$(printf "%02d" $chunk_index)
    local chunk_data=""
    for ((i=0; i<chunk_size; i++)); do
        chunk_data="${chunk_data}${pattern}"
    done
    
    # Convert to base64 for binary data (macOS compatible)
    echo -n "$chunk_data" | base64
}

# Helper function to create a 32-byte hash for testing
create_test_hash() {
    local data="$1"
    # Create exactly 32 bytes of data
    local hash_data=""
    while [[ ${#hash_data} -lt 32 ]]; do
        hash_data="${hash_data}${data}"
    done
    # Take exactly 32 bytes and convert to Candid blob format
    local hash_bytes="${hash_data:0:32}"
    local blob_arg=""
    for ((i=0; i<32; i++)); do
        local byte=$(printf "%d" "'${hash_bytes:$i:1}")
        blob_arg="${blob_arg}\\$(printf "%02x" $byte)"
    done
    echo "$blob_arg"
}

# Helper function to compute SHA256 hash of test data
compute_test_hash() {
    local chunk_data="$1"
    local chunk_count="$2"
    
    # Compute hash of all chunks concatenated together
    # This matches what the backend computes in store_from_chunks
    for ((i=0; i<chunk_count; i++)); do
        local chunk=$(create_test_chunk $i 50)
        echo -n "$chunk" | base64 -d
    done | shasum -a 256 | cut -d' ' -f1
}

# Helper function to begin an upload session
begin_upload_session() {
    local capsule_id="$1"
    local chunk_count="$2"
    local idem="$3"
    
    local result=$(dfx canister call backend uploads_begin \
        "(\"$capsule_id\", record { name = \"test-upload\"; description = opt \"Test upload\"; tags = vec {} }, $chunk_count, \"$idem\")" 2>/dev/null)
    
    if is_success "$result"; then
        # Extract session ID from response (format: Ok = 1 : nat64)
        local session_id=$(echo "$result" | grep -o 'Ok = [0-9]*' | sed 's/Ok = //')
        echo "$session_id"
        return 0
    else
        echo_error "Failed to begin upload session: $result"
        return 1
    fi
}

# Helper function to upload a chunk
upload_chunk() {
    local session_id="$1"
    local chunk_index="$2"
    local chunk_data="$3"
    
    local result=$(dfx canister call backend uploads_put_chunk \
        "($session_id, $chunk_index, blob \"$chunk_data\")" 2>/dev/null)
    
    if is_success "$result"; then
        echo_info "Chunk $chunk_index uploaded successfully"
        return 0
    else
        echo_error "Failed to upload chunk $chunk_index: $result"
        return 1
    fi
}

# Helper function to finish an upload
finish_upload() {
    local session_id="$1"
    local expected_hash="$2"
    local total_len="$3"
    
    local result=$(dfx canister call backend uploads_finish \
        "($session_id, blob \"$expected_hash\", $total_len)" 2>/dev/null)
    
    if is_success "$result"; then
        echo_info "Upload finished successfully: $result"
        return 0
    else
        echo_error "Failed to finish upload: $result"
        return 1
    fi
}

# Helper function to abort an upload
abort_upload() {
    local session_id="$1"
    
    local result=$(dfx canister call backend uploads_abort "$session_id" 2>/dev/null)
    
    if is_success "$result"; then
        echo_info "Upload aborted successfully: $result"
        return 0
    else
        echo_error "Failed to abort upload: $result"
        return 1
    fi
}

# Helper function to run a complete upload workflow
run_complete_upload_workflow() {
    local capsule_id="$1"
    local chunk_count="$2"
    local chunk_size="$3"
    local idem="$4"
    
    echo_info "Starting complete upload workflow with $chunk_count chunks of $chunk_size bytes each"
    
    # Begin upload
    local session_id=$(begin_upload_session "$capsule_id" "$chunk_count" "$idem")
    if [[ -z "$session_id" ]]; then
        return 1
    fi
    
    # Upload all chunks
    for ((i=0; i<chunk_count; i++)); do
        local chunk_data=$(create_test_chunk $i $chunk_size)
        if ! upload_chunk "$session_id" $i "$chunk_data"; then
            return 1
        fi
    done
    
    # Finish upload
    local expected_hash=$(create_test_hash "test")
    local total_len=$((chunk_count * chunk_size))
    if ! finish_upload "$session_id" "$expected_hash" "$total_len"; then
        return 1
    fi
    
    echo_success "Complete upload workflow completed successfully"
    return 0
}

# Helper function to test upload validation
test_upload_validation() {
    local test_name="$1"
    local test_command="$2"
    local expected_error="$3"
    
    echo_info "Testing: $test_name"
    local result=$(eval "$test_command" 2>/dev/null)
    
    if is_failure "$result" && [[ -n "$expected_error" ]] && echo "$result" | grep -q "$expected_error"; then
        echo_success "$test_name - correctly rejected with expected error"
        return 0
    elif is_failure "$result"; then
        echo_success "$test_name - correctly rejected: $result"
        return 0
    else
        echo_error "$test_name - should have been rejected: $result"
        return 1
    fi
}

# Helper function to print test summary
print_test_summary() {
    local test_name="$1"
    local total_tests="$2"
    local passed_tests="$3"
    local failed_tests="$4"
    
    echo "========================================="
    echo "Test Summary for $test_name"
    echo "========================================="
    echo "Total tests: $total_tests"
    echo "Passed: $passed_tests"
    echo "Failed: $failed_tests"
    echo ""
    
    if [ $failed_tests -eq 0 ]; then
        echo_success "All $test_name tests passed!"
        return 0
    else
        echo_error "$failed_tests $test_name test(s) failed"
        return 1
    fi
}
