#!/bin/bash

# Upload Workflow Tests
# Tests end-to-end upload workflows and edge cases

# Load test configuration and utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../test_config.sh"
source "$SCRIPT_DIR/../../test_utils.sh"
source "$SCRIPT_DIR/upload_test_utils.sh"

# Test configuration
TEST_NAME="Upload Workflow Tests"
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

# Helper function to clean up upload sessions
cleanup_upload_sessions() {
    echo_info "Cleaning up upload sessions..."
    for i in {1..20}; do
        dfx canister call "$BACKEND_CANISTER_ID" uploads_abort "$i" 2>/dev/null >/dev/null
    done
    # Give a moment for cleanup to complete
    sleep 1
}

# Test functions

test_uploads_begin_too_many_chunks() {
    # Test with too many chunks (exceeds MAX_CHUNKS)
    local result=$(dfx canister call "$BACKEND_CANISTER_ID" uploads_begin \
        '("test-capsule", (variant { Document = record { base = record { name = "test"; description = opt "test"; tags = vec {}; asset_type = variant { Original }; bytes = 0; mime_type = "text/plain"; sha256 = null; width = null; height = null; url = null; storage_key = null; bucket = null; asset_location = null; processing_status = null; processing_error = null; created_at = 0; updated_at = 0; deleted_at = null; }; page_count = null; document_type = null; language = null; word_count = null; }; }), 20000, "test-idem")' 2>/dev/null)
    
    if echo "$result" | grep -q "Err"; then
        echo_info "Upload session validation correctly rejected too many chunks: $result"
        return 0
    else
        echo_info "Upload session validation test result: $result"
        return 1
    fi
}

test_complete_upload_workflow() {
    # Test a complete upload workflow from begin to finish
    local capsule_id=$(get_test_capsule_id)
    local chunk_count=3
    local chunk_size=50
    local idem="workflow-test-$(date +%s)"
    
    echo_info "Testing complete upload workflow with $chunk_count chunks"
    
    # Begin upload
    local session_id=$(begin_upload_session "$capsule_id" "$chunk_count" "$idem")
    if [[ -z "$session_id" ]]; then
        echo_info "Failed to create upload session"
        return 1
    fi
    
    # Upload all chunks using regular endpoint
    for ((i=0; i<chunk_count; i++)); do
        local chunk_data=$(create_test_chunk $i $chunk_size)
        if ! upload_chunk "$session_id" $i "$chunk_data"; then
            echo_info "Failed to upload chunk $i"
            return 1
        fi
    done
    
    # Finish upload with correct hash using regular endpoint
    local chunk_data=$(create_test_chunk 0 $chunk_size)
    local expected_hash=$(compute_test_hash "$chunk_data" $chunk_count)
    local total_len=$((chunk_count * 100))  # Each chunk is actually 100 bytes
    if finish_upload "$session_id" "$expected_hash" "$total_len"; then
        echo_info "Complete upload workflow successful"
        return 0
    else
        echo_info "Upload workflow failed at finish"
        return 1
    fi
}

test_upload_workflow_missing_chunks() {
    # Test workflow with missing chunks
    local capsule_id=$(get_test_capsule_id)
    local chunk_count=3
    local idem="missing-chunks-test-$(date +%s)"
    
    echo_info "Testing upload workflow with missing chunks"
    
    # Begin upload
    local session_id=$(begin_upload_session "$capsule_id" "$chunk_count" "$idem")
    if [[ -z "$session_id" ]]; then
        echo_info "Failed to create upload session"
        return 1
    fi
    
    # Upload only first chunk (leaving 2 missing)
    local chunk_data=$(create_test_chunk 0 50)
    upload_chunk "$session_id" 0 "$chunk_data"
    
    # Try to finish with missing chunks
    local chunk_data=$(create_test_chunk 0 50)
    local expected_hash=$(compute_test_hash "$chunk_data" 1)
    if finish_upload "$session_id" "$expected_hash" 50; then
        echo_info "Should have failed with missing chunks"
        return 1
    else
        echo_info "Correctly rejected incomplete upload"
        return 0
    fi
}

test_upload_workflow_abort() {
    # Test upload abort workflow
    local capsule_id=$(get_test_capsule_id)
    local chunk_count=2
    local idem="abort-test-$(date +%s)"
    
    echo_info "Testing upload abort workflow"
    
    # Begin upload
    local session_id=$(begin_upload_session "$capsule_id" "$chunk_count" "$idem")
    if [[ -z "$session_id" ]]; then
        echo_info "Failed to create upload session"
        return 1
    fi
    
    # Upload one chunk
    local chunk_data=$(create_test_chunk 0 50)
    upload_chunk "$session_id" 0 "$chunk_data"
    
    # Abort the upload
    local result=$(dfx canister call "$BACKEND_CANISTER_ID" uploads_abort "$session_id" 2>/dev/null)
    
    if echo "$result" | grep -q "Ok"; then
        echo_info "Upload abort successful: $result"
        return 0
    else
        echo_info "Upload abort failed: $result"
        return 1
    fi
}

test_upload_workflow_idempotency() {
    # Test that begin_upload with same idem returns same session
    local capsule_id=$(get_test_capsule_id)
    local idem="idempotency-test-$(date +%s)"
    
    echo_info "Testing upload begin idempotency"
    
    local result1=$(dfx canister call "$BACKEND_CANISTER_ID" uploads_begin \
        "(\"$capsule_id\", (variant { Document = record { base = record { name = \"idempotency-test\"; description = opt \"Idempotency test\"; tags = vec {}; asset_type = variant { Original }; bytes = 0; mime_type = \"text/plain\"; sha256 = null; width = null; height = null; url = null; storage_key = null; bucket = null; asset_location = null; processing_status = null; processing_error = null; created_at = 0; updated_at = 0; deleted_at = null; }; page_count = null; document_type = null; language = null; word_count = null; }; }), 2, \"$idem\")" 2>/dev/null)
    
    local result2=$(dfx canister call "$BACKEND_CANISTER_ID" uploads_begin \
        "(\"$capsule_id\", (variant { Document = record { base = record { name = \"idempotency-test\"; description = opt \"Idempotency test\"; tags = vec {}; asset_type = variant { Original }; bytes = 0; mime_type = \"text/plain\"; sha256 = null; width = null; height = null; url = null; storage_key = null; bucket = null; asset_location = null; processing_status = null; processing_error = null; created_at = 0; updated_at = 0; deleted_at = null; }; page_count = null; document_type = null; language = null; word_count = null; }; }), 2, \"$idem\")" 2>/dev/null)
    
    local session_id1=$(echo "$result1" | grep -o 'Ok = [0-9]* : nat64' | sed 's/Ok = //' | sed 's/ : nat64//')
    local session_id2=$(echo "$result2" | grep -o 'Ok = [0-9]* : nat64' | sed 's/Ok = //' | sed 's/ : nat64//')
    
    if [ "$session_id1" = "$session_id2" ] && [ -n "$session_id1" ]; then
        echo_info "Upload begin idempotency working correctly: same session ID returned"
        return 0
    else
        echo_info "Upload begin idempotency test failed. Session1: $session_id1, Session2: $session_id2"
        echo_info "Result1: $result1"
        echo_info "Result2: $result2"
        return 1
    fi
}

test_large_file_workflow() {
    # Test with a larger number of chunks to simulate a bigger file
    local capsule_id=$(get_test_capsule_id)
    local chunk_count=10
    local chunk_size=50
    local idem="large-file-test-$(date +%s)"
    
    echo_info "Testing large file workflow with $chunk_count chunks"
    
    # Begin upload
    local session_id=$(begin_upload_session "$capsule_id" "$chunk_count" "$idem")
    if [[ -z "$session_id" ]]; then
        echo_info "Failed to create upload session"
        return 1
    fi
    
    # Upload all chunks
    for ((i=0; i<chunk_count; i++)); do
        local chunk_data=$(create_test_chunk $i $chunk_size)
        if ! upload_chunk "$session_id" $i "$chunk_data"; then
            echo_info "Failed to upload chunk $i in large file test"
            return 1
        fi
    done
    
    # Finish the large file with correct hash
    local chunk_data=$(create_test_chunk 0 $chunk_size)
    local expected_hash=$(compute_test_hash "$chunk_data" $chunk_count)
    local total_len=$((chunk_count * 100))  # Each chunk is actually 100 bytes
    if finish_upload "$session_id" "$expected_hash" "$total_len"; then
        echo_info "Large file workflow completed successfully"
        return 0
    else
        echo_info "Large file workflow failed"
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
    
    # Clean up any existing upload sessions
    cleanup_upload_sessions
    
    # Run all tests in order
    run_test "Uploads begin (too many chunks validation)" "test_uploads_begin_too_many_chunks"
    run_test "Complete upload workflow" "test_complete_upload_workflow"
    run_test "Upload workflow (missing chunks)" "test_upload_workflow_missing_chunks"
    run_test "Upload workflow (abort)" "test_upload_workflow_abort"
    run_test "Upload workflow (idempotency)" "test_upload_workflow_idempotency"
    run_test "Large file workflow" "test_large_file_workflow"
    
    # Print test summary
    print_test_summary "$TEST_NAME" "$TOTAL_TESTS" "$PASSED_TESTS" "$FAILED_TESTS"
    
    if [ $FAILED_TESTS -eq 0 ]; then
        exit 0
    else
        exit 1
    fi
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
