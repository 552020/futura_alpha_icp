#!/bin/bash

# Test gallery CRUD operations
# Tests galleries_create, galleries_update, galleries_delete, and galleries_list endpoints

# Load test configuration and utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../test_config.sh"
source "$SCRIPT_DIR/../../test_utils.sh"
source "$SCRIPT_DIR/gallery_test_utils.sh"

# Test configuration
TEST_NAME="Gallery CRUD Operations Tests"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Use run_gallery_test from shared utilities
run_test() {
    run_gallery_test "$1" "$2"
}

# Helper functions are now available from gallery_test_utils.sh

# Helper function to create and store a test gallery, returning its ID
create_test_gallery() {
    local gallery_type="$1"
    local memory_id="$2"
    
    # Get current timestamp in nanoseconds
    local timestamp=$(date +%s)000000000
    local gallery_id="gallery_${timestamp}_${RANDOM}"
    
    # Use shared utilities to create gallery data with memories
    local gallery_data=$(create_gallery_data_with_memories "$gallery_id" "Test Gallery for CRUD" "Gallery created for CRUD testing" "true" "$memory_id" "" "ICPOnly")
    
    local result=$(dfx canister call backend galleries_create "$gallery_data" 2>/dev/null)

    if is_success "$result"; then
        # Extract gallery ID from the response (Result<Gallery, Error> format)
        local returned_gallery_id=$(extract_gallery_id "$result")
        if [ -n "$returned_gallery_id" ]; then
            echo "$returned_gallery_id"
        else
            echo "$gallery_id"  # fallback to our generated ID
        fi
        return 0
    else
        echo ""
        return 1
    fi
}

# Test functions for update_gallery

test_update_gallery_title() {
    # Create a test gallery first
    local memory_id=$(upload_test_memory "Content for update test" "update_title_memory")
    
    if [ -z "$memory_id" ]; then
        echo_info "Failed to upload test memory for gallery update"
        return 1
    fi
    
    local gallery_id=$(create_test_gallery "basic" "$memory_id")
    
    if [ -z "$gallery_id" ]; then
        echo_info "Failed to create test gallery for title update"
        return 1
    fi
    
    # Update gallery title
    local update_data='(record { title = opt "Updated Gallery Title"; description = null; is_public = null; memory_entries = null; })'
    local result=$(dfx canister call backend galleries_update "(\"$gallery_id\", $update_data)" 2>/dev/null)

    if is_success "$result"; then
        echo_info "Gallery title update successful for ID: $gallery_id"
        return 0
    else
        echo_info "Gallery title update failed: $result"
        return 1
    fi
}

test_update_gallery_description() {
    # Create a test gallery first
    local memory_id=$(upload_test_memory "Content for description update" "update_desc_memory")
    
    if [ -z "$memory_id" ]; then
        echo_info "Failed to upload test memory for gallery description update"
        return 1
    fi
    
    local gallery_id=$(create_test_gallery "basic" "$memory_id")
    
    if [ -z "$gallery_id" ]; then
        echo_info "Failed to create test gallery for description update"
        return 1
    fi
    
    # Update gallery description
    local update_data='(record { title = null; description = opt "Updated description for testing"; is_public = null; memory_entries = null; })'
    local result=$(dfx canister call backend galleries_update "(\"$gallery_id\", $update_data)" 2>/dev/null)

    if is_success "$result"; then
        echo_info "Gallery description update successful for ID: $gallery_id"
        return 0
    else
        echo_info "Gallery description update failed: $result"
        return 1
    fi
}

test_update_gallery_visibility() {
    # Create a test gallery first
    local memory_id=$(upload_test_memory "Content for visibility update" "update_visibility_memory")
    
    if [ -z "$memory_id" ]; then
        echo_info "Failed to upload test memory for gallery visibility update"
        return 1
    fi
    
    local gallery_id=$(create_test_gallery "basic" "$memory_id")
    
    if [ -z "$gallery_id" ]; then
        echo_info "Failed to create test gallery for visibility update"
        return 1
    fi
    
    # Update gallery visibility to private
    local update_data='(record { title = null; description = null; is_public = opt false; memory_entries = null; })'
    local result=$(dfx canister call backend galleries_update "(\"$gallery_id\", $update_data)" 2>/dev/null)

    if is_success "$result"; then
        echo_info "Gallery visibility update successful for ID: $gallery_id"
        return 0
    else
        echo_info "Gallery visibility update failed: $result"
        return 1
    fi
}

test_update_gallery_memory_entries() {
    # Create test memories first
    local memory_id1=$(upload_test_memory "First memory for entries update" "update_entries_memory1")
    local memory_id2=$(upload_test_memory "Second memory for entries update" "update_entries_memory2")
    
    if [ -z "$memory_id1" ] || [ -z "$memory_id2" ]; then
        echo_info "Failed to upload test memories for gallery entries update"
        return 1
    fi
    
    local gallery_id=$(create_test_gallery "basic" "$memory_id1")
    
    if [ -z "$gallery_id" ]; then
        echo_info "Failed to create test gallery for entries update"
        return 1
    fi
    
    # Update gallery with new memory entries
    local update_data=$(cat << EOF
(record {
  title = null;
  description = null;
  is_public = null;
  memory_entries = opt vec {
    record {
      memory_id = "$memory_id1";
      position = 1;
      gallery_caption = opt "Updated first memory";
      is_featured = true;
      gallery_metadata = "{\"updated\": true}";
    };
    record {
      memory_id = "$memory_id2";
      position = 2;
      gallery_caption = opt "Added second memory";
      is_featured = false;
      gallery_metadata = "{\"new\": true}";
    };
  };
})
EOF
)
    
    local result=$(dfx canister call backend galleries_update "(\"$gallery_id\", $update_data)" 2>/dev/null)

    if is_success "$result"; then
        echo_info "Gallery memory entries update successful for ID: $gallery_id"
        return 0
    else
        echo_info "Gallery memory entries update failed: $result"
        return 1
    fi
}

test_update_nonexistent_gallery() {
    # Try to update a gallery that doesn't exist
    local fake_id="nonexistent_gallery_12345"
    local update_data='(record { title = opt "Should Fail"; description = null; is_public = null; memory_entries = null; })'
    local result=$(dfx canister call backend galleries_update "(\"$fake_id\", $update_data)" 2>/dev/null)
    
    # Should fail with appropriate error
    if is_failure "$result"; then
        echo_info "Correctly failed to update non-existent gallery"
        return 0
    else
        echo_info "Unexpected result for non-existent gallery update: $result"
        return 1
    fi
}

# Test functions for galleries_delete

test_delete_existing_gallery() {
    # Create a test gallery first
    local memory_id=$(upload_test_memory "Content for deletion test" "galleries_delete_memory")
    
    if [ -z "$memory_id" ]; then
        echo_info "Failed to upload test memory for gallery deletion"
        return 1
    fi
    
    local gallery_id=$(create_test_gallery "basic" "$memory_id")
    
    if [ -z "$gallery_id" ]; then
        echo_info "Failed to create test gallery for deletion"
        return 1
    fi
    
    # Delete the gallery
    local result=$(dfx canister call backend galleries_delete "(\"$gallery_id\")" 2>/dev/null)

    if is_success "$result"; then
        echo_info "Gallery deletion successful for ID: $gallery_id"
        return 0
    else
        echo_info "Gallery deletion failed: $result"
        return 1
    fi
}

test_delete_nonexistent_gallery() {
    # Try to delete a gallery that doesn't exist
    local fake_id="nonexistent_gallery_54321"
    local result=$(dfx canister call backend galleries_delete "(\"$fake_id\")" 2>/dev/null)
    
    # Should fail with appropriate error
    if is_failure "$result"; then
        echo_info "Correctly failed to delete non-existent gallery"
        return 0
    else
        echo_info "Unexpected result for non-existent gallery deletion: $result"
        return 1
    fi
}

test_galleries_delete_verify_removal() {
    # Create a test gallery first
    local memory_id=$(upload_test_memory "Content for deletion verification" "delete_verify_memory")
    
    if [ -z "$memory_id" ]; then
        echo_info "Failed to upload test memory for gallery deletion verification"
        return 1
    fi
    
    local gallery_id=$(create_test_gallery "basic" "$memory_id")
    
    if [ -z "$gallery_id" ]; then
        echo_info "Failed to create test gallery for deletion verification"
        return 1
    fi
    
    # Delete the gallery
    local delete_result=$(dfx canister call backend galleries_delete "(\"$gallery_id\")" 2>/dev/null)

    if ! is_success "$delete_result"; then
        echo_info "Failed to delete gallery for verification test"
        return 1
    fi
    
    # Try to retrieve the deleted gallery
    local retrieve_result=$(dfx canister call backend galleries_read "(\"$gallery_id\")" 2>/dev/null)
    
    if echo "$retrieve_result" | grep -q "Err"; then
        echo_info "Gallery deletion verification successful - gallery not found after deletion"
        return 0
    else
        echo_info "Gallery deletion verification failed - gallery still exists: $retrieve_result"
        return 1
    fi
}

# Test functions for get_my_galleries

test_get_my_galleries_empty() {
    # This test checks the structure when user has no galleries or few galleries
    local result=$(dfx canister call backend galleries_list 2>/dev/null)

    # Should return a vector (empty or with galleries) - Result<Vec<Gallery>, Error> format
    if is_success "$result" && (echo "$result" | grep -q "vec {" || echo "$result" | grep -q "vec{}"); then
        echo_info "galleries_list query successful - returned vector structure"
        return 0
    else
        echo_info "galleries_list query failed or returned unexpected format: $result"
        return 1
    fi
}

test_get_my_galleries_with_data() {
    # Create a test gallery to ensure we have data
    local memory_id=$(upload_test_memory "Content for my galleries test" "my_galleries_memory")
    
    if [ -z "$memory_id" ]; then
        echo_info "Failed to upload test memory for my galleries test"
        return 1
    fi
    
    local gallery_id=$(create_test_gallery "basic" "$memory_id")
    
    if [ -z "$gallery_id" ]; then
        echo_info "Failed to create test gallery for my galleries test"
        return 1
    fi
    
    # Query my galleries
    local result=$(dfx canister call backend galleries_list 2>/dev/null)

    # Should return a vector with gallery data - Result<Vec<Gallery>, Error> format
    if is_success "$result" && echo "$result" | grep -q "vec {" && echo "$result" | grep -q "record {"; then
        echo_info "galleries_list returned galleries successfully"
        return 0
    else
        echo_info "galleries_list returned unexpected format: $result"
        return 1
    fi
}

# Test functions for get_user_galleries

test_get_user_galleries_self() {
    # Create a test gallery first
    local memory_id=$(upload_test_memory "Content for user galleries test" "user_galleries_memory")
    
    if [ -z "$memory_id" ]; then
        echo_info "Failed to upload test memory for user galleries test"
        return 1
    fi
    
    local gallery_id=$(create_test_gallery "basic" "$memory_id")
    
    if [ -z "$gallery_id" ]; then
        echo_info "Failed to create test gallery for user galleries test"
        return 1
    fi
    
    # Query galleries for current user using galleries_list
    local result=$(dfx canister call backend galleries_list 2>/dev/null)

    # Should return a vector with gallery data - Result<Vec<Gallery>, Error> format
    if is_success "$result" && (echo "$result" | grep -q "vec {" || echo "$result" | grep -q "vec{}"); then
        echo_info "galleries_list returned galleries successfully for self"
        return 0
    else
        echo_info "galleries_list returned unexpected format: $result"
        return 1
    fi
}

test_get_user_galleries_nonexistent_user() {
    # Since get_user_galleries was removed, test that galleries_list works correctly
    local result=$(dfx canister call backend galleries_list 2>/dev/null)

    # Should return empty or populated vector - galleries_list always succeeds - Result<Vec<Gallery>, Error> format
    if is_success "$result" && (echo "$result" | grep -q "vec {" || echo "$result" | grep -q "vec{" || echo "$result" | grep -q "(vec {})" || echo "$result" | grep -q "(vec{})"); then
        echo_info "galleries_list correctly returned result"
        return 0
    else
        echo_info "galleries_list returned unexpected format: '$result'"
        return 1
    fi
}

test_gallery_crud_consistency() {
    # Test the full CRUD cycle: Create -> Update -> Read -> Delete
    local memory_id=$(upload_test_memory "Content for CRUD consistency test" "crud_consistency_memory")
    
    if [ -z "$memory_id" ]; then
        echo_info "Failed to upload test memory for CRUD consistency test"
        return 1
    fi
    
    # Create gallery
    local gallery_id=$(create_test_gallery "basic" "$memory_id")
    
    if [ -z "$gallery_id" ]; then
        echo_info "Failed to create test gallery for CRUD consistency test"
        return 1
    fi
    
    # Update gallery
    local update_data='(record { title = opt "CRUD Test Updated"; description = null; is_public = null; memory_entries = null; })'
    local update_result=$(dfx canister call backend galleries_update "(\"$gallery_id\", $update_data)" 2>/dev/null)
    
    if ! is_success "$update_result"; then
        echo_info "Failed to update gallery in CRUD consistency test"
        return 1
    fi
    
    # Read gallery to verify update
    local read_result=$(dfx canister call backend galleries_read "(\"$gallery_id\")" 2>/dev/null)
    
    if ! echo "$read_result" | grep -q "CRUD Test Updated"; then
        echo_info "Gallery update not reflected in read operation"
        return 1
    fi
    
    # Delete gallery
    local delete_result=$(dfx canister call backend galleries_delete "(\"$gallery_id\")" 2>/dev/null)
    
    if ! is_success "$delete_result"; then
        echo_info "Failed to delete gallery in CRUD consistency test"
        return 1
    fi
    
    # Verify deletion
    local verify_result=$(dfx canister call backend galleries_read "(\"$gallery_id\")" 2>/dev/null)
    
    if echo "$verify_result" | grep -q "NotFound"; then
        echo_info "CRUD consistency test completed successfully"
        return 0
    else
        echo_info "Gallery still exists after deletion in CRUD consistency test"
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
    
    # Register user first (required for gallery operations)
    echo_info "Registering user for gallery operations..."
    local register_result=$(dfx canister call backend register 2>/dev/null)
    if ! echo "$register_result" | grep -q "true"; then
        echo_warn "User registration returned: $register_result"
    fi
    
    # Run galleries_update tests
    echo_info "=== Testing galleries_update endpoint ==="
    run_test "Update gallery title" "test_update_gallery_title"
    run_test "Update gallery description" "test_update_gallery_description"
    run_test "Update gallery visibility" "test_update_gallery_visibility"
    run_test "Update gallery memory entries" "test_update_gallery_memory_entries"
    run_test "Update non-existent gallery" "test_update_nonexistent_gallery"
    
    echo_info "=== Testing galleries_delete endpoint ==="
    run_test "Delete existing gallery" "test_delete_existing_gallery"
    run_test "Delete non-existent gallery" "test_delete_nonexistent_gallery"
    run_test "Delete gallery and verify removal" "test_galleries_delete_verify_removal"
    
    echo_info "=== Testing get_my_galleries query ==="
    run_test "Get my galleries (empty or populated)" "test_get_my_galleries_empty"
    run_test "Get my galleries with data" "test_get_my_galleries_with_data"
    
    echo_info "=== Testing get_user_galleries query ==="
    run_test "Get user galleries for self" "test_get_user_galleries_self"
    run_test "Get user galleries for non-existent user" "test_get_user_galleries_nonexistent_user"
    
    echo_info "=== Testing gallery CRUD consistency ==="
    run_test "Full CRUD cycle consistency" "test_gallery_crud_consistency"
    
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