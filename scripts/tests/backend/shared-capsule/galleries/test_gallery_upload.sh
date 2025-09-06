#!/bin/bash

# Test gallery upload functionality
# Tests store_gallery_forever endpoint, gallery creation, metadata, and retrieval

# Load test configuration and utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../test_config.sh"
source "$SCRIPT_DIR/../../test_utils.sh"
source "$SCRIPT_DIR/gallery_test_utils.sh"

# Test configuration
TEST_NAME="Gallery Upload Functionality Tests"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Use run_gallery_test from shared utilities
run_test() {
    run_gallery_test "$1" "$2"
}

# Helper functions are now available from gallery_test_utils.sh

# get_test_capsule_id is now available from gallery_test_utils.sh

# Gallery data creation functions are now available from gallery_test_utils.sh

# Test functions for galleries_create

test_store_basic_gallery() {
    # Upload a test memory first
    local memory_id=$(upload_test_memory "Content for basic gallery" "basic_gallery_memory")
    
    if [ -z "$memory_id" ]; then
        echo_info "Failed to upload test memory for basic gallery"
        return 1
    fi
    
    # Create and store basic gallery using shared utilities
    local gallery_data=$(create_gallery_data_with_memories "" "Basic Test Gallery" "A simple test gallery" "true" "$memory_id" "First memory in gallery" "ICPOnly")
    local result=$(dfx canister call backend galleries_create "$gallery_data" 2>/dev/null)
    
    if is_success "$result"; then
        local gallery_id=$(extract_gallery_id "$result")
        echo_info "Basic gallery creation successful with ID: $gallery_id"
        return 0
    else
        echo_info "Basic gallery creation failed: $result"
        return 1
    fi
}

test_store_gallery_with_multiple_memories() {
    # Upload test memories first
    local memory_id1=$(upload_test_memory "First memory content" "multi_gallery_memory1")
    local memory_id2=$(upload_test_memory "Second memory content" "multi_gallery_memory2")
    
    if [ -z "$memory_id1" ] || [ -z "$memory_id2" ]; then
        echo_info "Failed to upload test memories for multi-memory gallery"
        return 1
    fi
    
    # Create and store gallery with multiple memories using shared utilities
    local gallery_data=$(create_gallery_data_with_memories "" "Multi-Memory Gallery" "Gallery with multiple memories" "false" "$memory_id1" "$memory_id2" "ICPOnly")
    local result=$(dfx canister call backend galleries_create "$gallery_data" 2>/dev/null)
    
    if is_success "$result"; then
        local gallery_id=$(extract_gallery_id "$result")
        echo_info "Multi-memory gallery creation successful with ID: $gallery_id"
        return 0
    else
        echo_info "Multi-memory gallery creation failed: $result"
        return 1
    fi
}

test_store_empty_gallery() {
    # Create and store empty gallery (no memories) using shared utilities
    local gallery_data=$(create_basic_gallery_data "" "Empty Gallery" "" "true" "ICPOnly")
    local result=$(dfx canister call backend galleries_create "$gallery_data" 2>/dev/null)
    
    if is_success "$result"; then
        local gallery_id=$(extract_gallery_id "$result")
        echo_info "Empty gallery creation successful with ID: $gallery_id"
        return 0
    else
        echo_info "Empty gallery creation failed: $result"
        return 1
    fi
}

test_store_private_gallery() {
    # Upload a test memory first
    local memory_id=$(upload_test_memory "Private gallery content" "private_gallery_memory")
    
    if [ -z "$memory_id" ]; then
        echo_info "Failed to upload test memory for private gallery"
        return 1
    fi
    
    # Create and store private gallery using shared utilities
    local gallery_data=$(create_gallery_data_with_memories "" "Private Gallery" "This is a private gallery" "false" "$memory_id" "" "ICPOnly")
    local result=$(dfx canister call backend galleries_create "$gallery_data" 2>/dev/null)
    
    if is_success "$result"; then
        local gallery_id=$(extract_gallery_id "$result")
        echo_info "Private gallery creation successful with ID: $gallery_id"
        return 0
    else
        echo_info "Private gallery creation failed: $result"
        return 1
    fi
}

test_store_gallery_with_metadata() {
    # Upload a test memory first
    local memory_id=$(upload_test_memory "Content with metadata" "metadata_gallery_memory")
    
    if [ -z "$memory_id" ]; then
        echo_info "Failed to upload test memory for metadata gallery"
        return 1
    fi
    
    # Create gallery with rich metadata using shared utilities
    local gallery_data=$(create_gallery_data_with_memories "" "Gallery with Rich Metadata" "This gallery has detailed metadata for testing" "true" "$memory_id" "Memory with detailed metadata" "ICPOnly")
    local result=$(dfx canister call backend galleries_create "$gallery_data" 2>/dev/null)
    
    if is_success "$result"; then
        echo_info "Gallery with metadata creation successful"
        return 0
    else
        echo_info "Gallery with metadata creation failed: $result"
        return 1
    fi
}

# Test functions for galleries_read

test_retrieve_gallery_by_id() {
    # Upload a test memory first
    local memory_id=$(upload_test_memory "Content for retrieval test" "retrieval_gallery_memory")
    
    if [ -z "$memory_id" ]; then
        echo_info "Failed to upload test memory for retrieval test"
        return 1
    fi
    
    # Create and store gallery using shared utilities
    local gallery_data=$(create_gallery_data_with_memories "" "Basic Test Gallery" "A simple test gallery" "true" "$memory_id" "First memory in gallery" "ICPOnly")
    local store_result=$(dfx canister call backend galleries_create "$gallery_data" 2>/dev/null)
    
    if ! is_success "$store_result"; then
        echo_info "Failed to store gallery for retrieval test"
        return 1
    fi
    
    # Extract gallery ID using shared utilities
    local gallery_id=$(extract_gallery_id "$store_result")
    
    if [ -z "$gallery_id" ]; then
        echo_info "Failed to extract gallery ID from store result: $store_result"
        return 1
    fi
    
    # Retrieve the gallery (add small delay for persistence)
    echo_info "Attempting to retrieve gallery with ID: $gallery_id"
    sleep 1
    local retrieve_result=$(dfx canister call backend galleries_read "(\"$gallery_id\")" 2>/dev/null)
    
    # Check if gallery was retrieved successfully
    if is_success "$retrieve_result" && echo "$retrieve_result" | grep -q "title = \"Basic Test Gallery\""; then
        echo_info "Gallery retrieval successful for ID: $gallery_id"
        return 0
    else
        echo_info "Gallery retrieval failed: $retrieve_result"
        return 1
    fi
}

test_retrieve_nonexistent_gallery() {
    # Try to retrieve a gallery that doesn't exist
    local fake_id="nonexistent_gallery_12345"
    local result=$(dfx canister call backend galleries_read "(\"$fake_id\")" 2>/dev/null)
    
    # Should return error for non-existent gallery
    if is_failure "$result"; then
        echo_info "Correctly returned error for non-existent gallery"
        return 0
    else
        echo_info "Unexpected result for non-existent gallery: $result"
        return 1
    fi
}

test_gallery_memory_associations() {
    # Upload test memories
    local memory_id1=$(upload_test_memory "First associated memory" "assoc_memory1")
    local memory_id2=$(upload_test_memory "Second associated memory" "assoc_memory2")
    
    if [ -z "$memory_id1" ] || [ -z "$memory_id2" ]; then
        echo_info "Failed to upload test memories for association test"
        return 1
    fi
    
    # Create gallery with multiple memories using shared utilities
    local gallery_data=$(create_gallery_data_with_memories "" "Multi-Memory Gallery" "Gallery with multiple memories" "false" "$memory_id1" "$memory_id2" "ICPOnly")
    local store_result=$(dfx canister call backend galleries_create "$gallery_data" 2>/dev/null)
    
    if ! is_success "$store_result"; then
        echo_info "Failed to store gallery for association test"
        return 1
    fi
    
    # Extract gallery ID using shared utilities
    local gallery_id=$(extract_gallery_id "$store_result")
    
    # Retrieve and verify memory associations
    echo_info "Retrieving gallery for association test with ID: $gallery_id"
    sleep 1
    local retrieve_result=$(dfx canister call backend galleries_read "(\"$gallery_id\")" 2>/dev/null)
    
    # Check if both memory IDs are present in the gallery
    if echo "$retrieve_result" | grep -q "$memory_id1" && echo "$retrieve_result" | grep -q "$memory_id2"; then
        echo_info "Gallery memory associations verified successfully"
        return 0
    else
        echo_info "Gallery memory associations verification failed. Expected: $memory_id1, $memory_id2"
        echo_info "Actual result: $retrieve_result"
        return 1
    fi
}

test_gallery_metadata_preservation() {
    # Upload a test memory
    local memory_id=$(upload_test_memory "Content for metadata test" "metadata_preservation_memory")
    
    if [ -z "$memory_id" ]; then
        echo_info "Failed to upload test memory for metadata preservation test"
        return 1
    fi
    
    # Create gallery with specific metadata using shared utilities
    local gallery_data=$(create_gallery_data_with_memories "" "Basic Test Gallery" "A simple test gallery" "true" "$memory_id" "First memory in gallery" "ICPOnly")
    local store_result=$(dfx canister call backend galleries_create "$gallery_data" 2>/dev/null)
    
    if ! is_success "$store_result"; then
        echo_info "Failed to store gallery for metadata preservation test"
        return 1
    fi
    
    # Extract gallery ID using shared utilities
    local gallery_id=$(extract_gallery_id "$store_result")
    
    # Retrieve and verify metadata preservation
    echo_info "Retrieving gallery for metadata test with ID: $gallery_id"
    sleep 1
    local retrieve_result=$(dfx canister call backend galleries_read "(\"$gallery_id\")" 2>/dev/null)
    
    # Check if key metadata fields are preserved
    if echo "$retrieve_result" | grep -q "title = \"Basic Test Gallery\"" && \
       echo "$retrieve_result" | grep -q "description = opt \"A simple test gallery\"" && \
       echo "$retrieve_result" | grep -q "is_public = true"; then
        echo_info "Gallery metadata preservation verified successfully"
        return 0
    else
        echo_info "Gallery metadata preservation verification failed. Result: $retrieve_result"
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
    
    # User registration is handled by the shared utilities when needed
    
    # Run galleries_create tests
    echo_info "=== Testing galleries_create endpoint ==="
    run_test "Store basic gallery" "test_store_basic_gallery"
    run_test "Store gallery with multiple memories" "test_store_gallery_with_multiple_memories"
    run_test "Store empty gallery" "test_store_empty_gallery"
    run_test "Store private gallery" "test_store_private_gallery"
    run_test "Store gallery with metadata" "test_store_gallery_with_metadata"
    
    echo_info "=== Testing galleries_read endpoint ==="
    run_test "Retrieve gallery by ID" "test_retrieve_gallery_by_id"
    run_test "Retrieve non-existent gallery" "test_retrieve_nonexistent_gallery"
    
    echo_info "=== Testing gallery functionality ==="
    run_test "Gallery memory associations" "test_gallery_memory_associations"
    run_test "Gallery metadata preservation" "test_gallery_metadata_preservation"
    
    # Print test summary
    echo "========================================="
    echo "Test Summary for $TEST_NAME"
    echo "========================================="
    echo "Total tests: $TOTAL_TESTS"
    echo "Passed: $PASSED_TESTS"
    echo "Failed: $FAILED_TESTS"
    echo ""
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo_pass "All tests passed!"
        exit 0
    else
        echo_fail "$FAILED_TESTS test(s) failed"
        exit 1
    fi
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi