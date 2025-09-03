#!/bin/bash

# Test galleries_delete endpoint functionality (replaces delete_gallery)
# This script tests the new galleries_delete endpoint and verifies that the old delete_gallery is removed

set -e

# Source test utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_utils.sh"

# Test configuration
TEST_NAME="Galleries Delete Tests"
CANISTER_NAME="backend"

# Test data
GALLERY_ID="test-gallery-delete-$(date +%s)"
GALLERY_NAME="Test Gallery for Delete"
GALLERY_DESCRIPTION="A test gallery for delete operations"

# Test functions for galleries_delete
test_galleries_delete_basic() {
    echo_info "Testing galleries_delete with basic gallery data..."
    
    # First create a gallery to delete
    local gallery_data="(record {
        gallery = record {
            id = \"$GALLERY_ID\";
            title = \"$GALLERY_NAME\";
            description = opt \"$GALLERY_DESCRIPTION\";
            is_public = true;
            created_at = 0;
            updated_at = 0;
            owner_principal = principal \"2vxsx-fae\";
            storage_status = variant { Web2Only };
            memory_entries = vec {};
        };
        owner_principal = principal \"2vxsx-fae\";
    })"
    
    # Create the gallery
    local create_result=$(dfx canister call backend galleries_create "$gallery_data" 2>/dev/null)
    if ! echo "$create_result" | grep -q "success = true"; then
        echo_error "Failed to create gallery for delete test"
        return 1
    fi
    
    # Verify gallery exists in list
    local list_result=$(dfx canister call backend galleries_list 2>/dev/null)
    if ! echo "$list_result" | grep -q "\"$GALLERY_ID\""; then
        echo_error "Gallery not found in list after creation"
        return 1
    fi
    
    # Now delete the gallery
    local result=$(dfx canister call backend galleries_delete "(\"$GALLERY_ID\")" 2>/dev/null)
    
    # Verify response structure
    if echo "$result" | grep -q "success = true"; then
        echo_success "galleries_delete returned success = true"
    else
        echo_error "galleries_delete failed or returned unexpected response"
        echo_debug "Response: $result"
        return 1
    fi
    
    # Verify success message
    if echo "$result" | grep -q "Gallery deleted successfully"; then
        echo_success "galleries_delete returned correct success message"
    else
        echo_error "galleries_delete did not return expected success message"
        echo_debug "Response: $result"
        return 1
    fi
    
    echo_success "Basic galleries_delete test passed"
}

test_galleries_delete_verification() {
    echo_info "Verifying gallery was actually deleted via galleries_list..."
    
    # Get galleries list
    local result=$(dfx canister call backend galleries_list 2>/dev/null)
    
    # Check if our deleted gallery is NOT in the list
    if echo "$result" | grep -q "\"$GALLERY_ID\""; then
        echo_error "Gallery still found in galleries_list after deletion"
        echo_debug "galleries_list response: $result"
        return 1
    else
        echo_success "Gallery correctly removed from galleries_list after deletion"
    fi
    
    echo_success "Verification test passed"
}

test_galleries_delete_nonexistent() {
    echo_info "Testing galleries_delete with non-existent gallery ID..."
    
    # Try to delete a gallery that doesn't exist
    local result=$(dfx canister call backend galleries_delete "(\"non-existent-gallery-123\")" 2>/dev/null)
    
    # Should return failure
    if echo "$result" | grep -q "success = false"; then
        if echo "$result" | grep -q "Gallery not found"; then
            echo_success "galleries_delete correctly handled non-existent gallery"
        else
            echo_warning "galleries_delete returned failure but unexpected message"
        fi
    else
        echo_error "galleries_delete should have failed for non-existent gallery"
        echo_debug "Response: $result"
        return 1
    fi
    
    echo_success "Non-existent gallery test passed"
}

test_galleries_delete_idempotency() {
    echo_info "Testing galleries_delete idempotency (calling twice with same ID)..."
    
    # Try to delete the same gallery again
    local result=$(dfx canister call backend galleries_delete "(\"$GALLERY_ID\")" 2>/dev/null)
    
    # Should return failure with "Gallery not found" message
    if echo "$result" | grep -q "success = false"; then
        if echo "$result" | grep -q "Gallery not found"; then
            echo_success "galleries_delete correctly handled duplicate deletion (idempotency)"
        else
            echo_warning "galleries_delete returned failure but unexpected message"
        fi
    else
        echo_error "galleries_delete should have failed for already deleted gallery"
        echo_debug "Response: $result"
        return 1
    fi
    
    echo_success "Idempotency test passed"
}

test_old_endpoint_removed() {
    echo_info "Verifying that old delete_gallery endpoint is removed..."
    
    # Try to call the old endpoint
    local result=$(dfx canister call backend delete_gallery "(\"test\")" 2>&1 || true)
    
    # Should get an error about method not found
    if echo "$result" | grep -q "method not found\|no update method"; then
        echo_success "Old delete_gallery endpoint correctly removed"
    else
        echo_error "Old delete_gallery endpoint still exists or returned unexpected error"
        echo_debug "Response: $result"
        return 1
    fi
    
    echo_success "Old endpoint removal verification passed"
}

# Main test execution
main() {
    echo_header "Running $TEST_NAME"
    
    # Check if backend canister is running
    if ! dfx canister status backend >/dev/null 2>&1; then
        echo_error "Backend canister is not running. Please run ./scripts/deploy-local.sh first."
        exit 1
    fi
    
    # Run tests
    run_test "Basic galleries_delete functionality" "test_galleries_delete_basic"
    run_test "Gallery deletion verification" "test_galleries_delete_verification"
    run_test "galleries_delete non-existent gallery" "test_galleries_delete_nonexistent"
    run_test "galleries_delete idempotency" "test_galleries_delete_idempotency"
    run_test "Old delete_gallery endpoint removal" "test_old_endpoint_removed"
    
    echo_header "$TEST_NAME completed successfully! 🎉"
}

# Run main function
main "$@"
