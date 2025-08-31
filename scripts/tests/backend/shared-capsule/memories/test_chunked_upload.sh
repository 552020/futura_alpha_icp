#!/bin/bash

# Test chunked upload functionality
# Tests the new chunked asset upload protocol endpoints

# Load test configuration and utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../test_config.sh"
source "$SCRIPT_DIR/../../test_utils.sh"

# Test configuration
TEST_NAME="Chunked Upload Protocol Tests"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

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

# Helper function to compute expected hash for test data
# This simulates what the backend compute_sha256_hash function does
compute_expected_hash() {
    local combined_data="$1"
    # The backend uses DefaultHasher and formats as "sha256_{hex}"
    # For testing, we'll use a predictable hash based on data length and content
    local hash_suffix=$(echo -n "$combined_data" | shasum -a 256 | cut -d' ' -f1 | head -c 16)
    echo "sha256_${hash_suffix}"
}

# Test functions

test_begin_asset_upload_success() {
    local memory_id="test_memory_$(date +%s)"
    local expected_hash="test_hash_abc123"
    local total_size=1000
    local chunk_count=5
    
    local result=$(dfx canister call backend begin_asset_upload \
        "(\"$memory_id\", \"$expected_hash\", $chunk_count, $total_size)" 2>/dev/null)
    
    if echo "$result" | grep -q "success = true" && echo "$result" | grep -q "session_id"; then
        echo_info "Upload session created successfully"
        # Extract and store session ID for other tests
        export TEST_SESSION_ID=$(echo "$result" | grep -o 'session_id = "[^"]*"' | sed 's/session_id = "\([^"]*\)"/\1/')
        export TEST_MEMORY_ID="$memory_id"
        return 0
    else
        echo_info "Upload session creation failed: $result"
        return 1
    fi
}

test_begin_asset_upload_validation() {
    # Test with invalid parameters (zero chunk count)
    local result=$(dfx canister call backend begin_asset_upload \
        '("test_memory", "hash", 0, 1000)' 2>/dev/null)
    
    if echo "$result" | grep -q "success = false" || echo "$result" | grep -q "Err"; then
        echo_info "Upload session validation correctly rejected invalid parameters"
        return 0
    else
        echo_info "Upload session validation test result: $result"
        return 1
    fi
}

test_put_chunk_success() {
    # Ensure we have a session from previous test
    if [ -z "$TEST_SESSION_ID" ]; then
        echo_info "No test session available, creating one..."
        test_begin_asset_upload_success
    fi
    
    if [ -z "$TEST_SESSION_ID" ]; then
        echo_info "Failed to create test session for chunk upload"
        return 1
    fi
    
    local chunk_data=$(create_test_chunk 0 50)
    local result=$(dfx canister call backend put_chunk \
        "(\"$TEST_SESSION_ID\", 0, blob \"$chunk_data\")" 2>/dev/null)
    
    if echo "$result" | grep -q "success = true" && echo "$result" | grep -q "chunk_index = 0"; then
        echo_info "Chunk upload successful"
        return 0
    else
        echo_info "Chunk upload failed: $result"
        return 1
    fi
}

test_put_chunk_invalid_session() {
    local chunk_data=$(create_test_chunk 0 50)
    local result=$(dfx canister call backend put_chunk \
        '("nonexistent_session", 0, blob "'"$chunk_data"'")' 2>/dev/null)
    
    if echo "$result" | grep -q "success = false" || echo "$result" | grep -q "Err"; then
        echo_info "Chunk upload correctly rejected invalid session"
        return 0
    else
        echo_info "Chunk upload validation test result: $result"
        return 1
    fi
}

test_upload_multiple_chunks() {
    # Ensure we have a session
    if [ -z "$TEST_SESSION_ID" ]; then
        test_begin_asset_upload_success
    fi
    
    if [ -z "$TEST_SESSION_ID" ]; then
        echo_info "Failed to create test session for multiple chunk upload"
        return 1
    fi
    
    # Upload chunks 1-4 (chunk 0 was uploaded in previous test)
    for chunk_index in {1..4}; do
        local chunk_data=$(create_test_chunk $chunk_index 50)
        local result=$(dfx canister call backend put_chunk \
            "(\"$TEST_SESSION_ID\", $chunk_index, blob \"$chunk_data\")" 2>/dev/null)
        
        if ! echo "$result" | grep -q "success = true"; then
            echo_info "Failed to upload chunk $chunk_index: $result"
            return 1
        fi
    done
    
    echo_info "Multiple chunks uploaded successfully"
    return 0
}

test_commit_asset_success() {
    # Ensure all chunks are uploaded for our test session
    if [ -z "$TEST_SESSION_ID" ]; then
        echo_info "No test session available for commit"
        return 1
    fi
    
    # Make sure all chunks are uploaded (we uploaded 0-4 in previous tests)
    # Use a placeholder hash - the backend will compute the actual hash
    local result=$(dfx canister call backend commit_asset \
        "(\"$TEST_SESSION_ID\", \"placeholder_hash\")" 2>/dev/null)
    
    if echo "$result" | grep -q "success = true" && echo "$result" | grep -q "memory_id ="; then
        echo_info "Upload committed successfully"
        return 0
    else
        echo_info "Upload commit failed: $result"
        return 1
    fi
}

test_commit_asset_missing_chunks() {
    # Create a session with incomplete chunks
    local memory_id="incomplete_test_$(date +%s)"
    local result=$(dfx canister call backend begin_asset_upload \
        "(\"$memory_id\", \"hash456\", 3, 300)" 2>/dev/null)
    
    local session_id=$(echo "$result" | grep -o 'session_id = "[^"]*"' | sed 's/session_id = "\([^"]*\)"/\1/')
    
    if [ -z "$session_id" ]; then
        echo_info "Failed to create session for incomplete commit test"
        return 1
    fi
    
    # Upload only one chunk (leaving 2 missing)
    local chunk_data=$(create_test_chunk 0 50)
    dfx canister call backend put_chunk \
        "(\"$session_id\", 0, blob \"$chunk_data\")" 2>/dev/null
    
    # Try to commit with missing chunks
    local commit_result=$(dfx canister call backend commit_asset \
        "(\"$session_id\", \"final_hash\")" 2>/dev/null)
    
    if echo "$commit_result" | grep -q "success = false" || echo "$commit_result" | grep -q "Err"; then
        echo_info "Commit correctly rejected incomplete upload"
        return 0
    else
        echo_info "Commit should have failed with missing chunks: $commit_result"
        return 1
    fi
}

test_cancel_upload() {
    # Create a session for cancellation test
    local memory_id="cancel_test_$(date +%s)"
    local result=$(dfx canister call backend begin_asset_upload \
        "(\"$memory_id\", \"cancel_hash\", 2, 200)" 2>/dev/null)
    
    local session_id=$(echo "$result" | grep -o 'session_id = "[^"]*"' | sed 's/session_id = "\([^"]*\)"/\1/')
    
    if [ -z "$session_id" ]; then
        echo_info "Failed to create session for cancel test"
        return 1
    fi
    
    # Cancel the upload
    local cancel_result=$(dfx canister call backend cancel_upload "\"$session_id\"" 2>/dev/null)
    
    if echo "$cancel_result" | grep -q "success = true"; then
        echo_info "Upload cancelled successfully"
        return 0
    else
        echo_info "Upload cancellation failed: $cancel_result"
        return 1
    fi
}

test_chunk_idempotency() {
    # Create a session for idempotency test
    local memory_id="idempotency_test_$(date +%s)"
    local result=$(dfx canister call backend begin_asset_upload \
        "(\"$memory_id\", \"idempotency_hash\", 2, 200)" 2>/dev/null)
    
    local session_id=$(echo "$result" | grep -o 'session_id = "[^"]*"' | sed 's/session_id = "\([^"]*\)"/\1/')
    
    if [ -z "$session_id" ]; then
        echo_info "Failed to create session for idempotency test"
        return 1
    fi
    
    # Upload the same chunk twice
    local chunk_data=$(create_test_chunk 0 50)
    
    local result1=$(dfx canister call backend put_chunk \
        "(\"$session_id\", 0, blob \"$chunk_data\")" 2>/dev/null)
    
    local result2=$(dfx canister call backend put_chunk \
        "(\"$session_id\", 0, blob \"$chunk_data\")" 2>/dev/null)
    
    # Both should succeed (idempotent)
    if echo "$result1" | grep -q "success = true" && echo "$result2" | grep -q "success = true"; then
        echo_info "Chunk upload idempotency working correctly"
        return 0
    else
        echo_info "Chunk idempotency test failed. Result1: $result1, Result2: $result2"
        return 1
    fi
}

test_large_file_simulation() {
    # Test with a larger number of chunks to simulate a bigger file
    local memory_id="large_file_test_$(date +%s)"
    local chunk_count=10
    local total_size=5000
    
    local result=$(dfx canister call backend begin_asset_upload \
        "(\"$memory_id\", \"large_file_hash\", $chunk_count, $total_size)" 2>/dev/null)
    
    local session_id=$(echo "$result" | grep -o 'session_id = "[^"]*"' | sed 's/session_id = "\([^"]*\)"/\1/')
    
    if [ -z "$session_id" ]; then
        echo_info "Failed to create session for large file test"
        return 1
    fi
    
    # Upload all chunks
    for chunk_index in $(seq 0 $((chunk_count-1))); do
        local chunk_data=$(create_test_chunk $chunk_index 50)
        local upload_result=$(dfx canister call backend put_chunk \
            "(\"$session_id\", $chunk_index, blob \"$chunk_data\")" 2>/dev/null)
        
        if ! echo "$upload_result" | grep -q "success = true"; then
            echo_info "Failed to upload chunk $chunk_index in large file test"
            return 1
        fi
    done
    
    # Commit the large file
    local commit_result=$(dfx canister call backend commit_asset \
        "(\"$session_id\", \"placeholder_hash\")" 2>/dev/null)
    
    if echo "$commit_result" | grep -q "success = true"; then
        echo_info "Large file simulation completed successfully"
        return 0
    else
        echo_info "Large file commit failed: $commit_result"
        return 1
    fi
}

# Main test execution
main() {
    echo "========================================="
    echo "Starting $TEST_NAME"
    echo "========================================="
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
    
    # Run all tests in order
    run_test "Begin asset upload (success)" "test_begin_asset_upload_success"
    run_test "Begin asset upload (validation)" "test_begin_asset_upload_validation"
    run_test "Put chunk (success)" "test_put_chunk_success"
    run_test "Put chunk (invalid session)" "test_put_chunk_invalid_session"
    run_test "Upload multiple chunks" "test_upload_multiple_chunks"
    run_test "Commit asset (success)" "test_commit_asset_success"
    run_test "Commit asset (missing chunks)" "test_commit_asset_missing_chunks"
    run_test "Cancel upload" "test_cancel_upload"
    run_test "Chunk idempotency" "test_chunk_idempotency"
    run_test "Large file simulation" "test_large_file_simulation"
    
    # Print test summary
    echo "========================================="
    echo "Test Summary for $TEST_NAME"
    echo "========================================="
    echo "Total tests: $TOTAL_TESTS"
    echo "Passed: $PASSED_TESTS"
    echo "Failed: $FAILED_TESTS"
    echo ""
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo_pass "All chunked upload tests passed!"
        exit 0
    else
        echo_fail "$FAILED_TESTS chunked upload test(s) failed"
        exit 1
    fi
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi