#!/bin/bash

# Test galleries_update endpoint functionality (replaces update_gallery)
# This script tests the new galleries_update endpoint and verifies that the old update_gallery is removed

set -e

# Source test utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../test_utils.sh"
source "$SCRIPT_DIR/gallery_test_utils.sh"

# Test configuration
TEST_NAME="Galleries Update Tests"
CANISTER_NAME="backend"

# Test data
GALLERY_ID="test-gallery-update-$(date +%s)"
GALLERY_NAME="Test Gallery for Update"
GALLERY_DESCRIPTION="A test gallery for update operations"

# Test functions for galleries_update
test_galleries_update_basic() {
    echo_info "Testing galleries_update with basic gallery data..."
    
    # First create a gallery to update using shared utilities
    local gallery_data=$(create_basic_gallery_data "$GALLERY_ID" "$GALLERY_NAME" "$GALLERY_DESCRIPTION" "true" "Web2Only")
    
    # Create the gallery
    local create_result=$(dfx canister call backend galleries_create "$gallery_data" 2>/dev/null)
    if ! is_success "$create_result"; then
        echo_error "Failed to create gallery for update test"
        return 1
    fi
    
    # Now update the gallery
    local update_data="(record {
        title = opt \"Updated Gallery Title\";
        description = opt \"Updated description\";
        is_public = opt false;
        memory_entries = opt vec {};
    })"
    
    # Call galleries_update
    local result=$(dfx canister call backend galleries_update "(\"$GALLERY_ID\", $update_data)" 2>/dev/null)
    
    # Verify response structure (Result<T, Error> format)
    if is_success "$result"; then
        echo_success "galleries_update returned success"
    else
        echo_error "galleries_update failed or returned unexpected response"
        echo_debug "Response: $result"
        return 1
    fi
    
    # Verify gallery is returned (Result<Gallery, Error> format)
    if echo "$result" | grep -q "id = \"$GALLERY_ID\""; then
        echo_success "galleries_update returned gallery data"
    else
        echo_error "galleries_update did not return gallery data"
        echo_debug "Response: $result"
        return 1
    fi
    
    # Verify title was updated
    if echo "$result" | grep -q "title = \"Updated Gallery Title\""; then
        echo_success "galleries_update correctly updated title"
    else
        echo_error "galleries_update did not update title correctly"
        echo_debug "Response: $result"
        return 1
    fi
    
    # Verify description was updated
    if echo "$result" | grep -q "description = opt \"Updated description\""; then
        echo_success "galleries_update correctly updated description"
    else
        echo_error "galleries_update did not update description correctly"
        echo_debug "Response: $result"
        return 1
    fi
    
    # Verify is_public was updated
    if echo "$result" | grep -q "is_public = false"; then
        echo_success "galleries_update correctly updated is_public"
    else
        echo_error "galleries_update did not update is_public correctly"
        echo_debug "Response: $result"
        return 1
    fi
    
    echo_success "Basic galleries_update test passed"
}

test_galleries_update_partial() {
    echo_info "Testing galleries_update with partial data (only title)..."
    
    # Update only the title
    local update_data="(record {
        title = opt \"Partially Updated Title\";
        description = null;
        is_public = null;
        memory_entries = null;
    })"
    
    # Call galleries_update
    local result=$(dfx canister call backend galleries_update "(\"$GALLERY_ID\", $update_data)" 2>/dev/null)
    
    # Verify response (Result<T, Error> format)
    if is_success "$result"; then
        if echo "$result" | grep -q "title = \"Partially Updated Title\""; then
            echo_success "galleries_update correctly handled partial update (title only)"
        else
            echo_error "galleries_update did not update title in partial update"
            echo_debug "Response: $result"
            return 1
        fi
    else
        echo_error "galleries_update failed on partial update"
        echo_debug "Response: $result"
        return 1
    fi
    
    echo_success "Partial update test passed"
}

test_galleries_update_nonexistent() {
    echo_info "Testing galleries_update with non-existent gallery ID..."
    
    local update_data="(record {
        title = opt \"This Should Fail\";
        description = null;
        is_public = null;
        memory_entries = null;
    })"
    
    # Call galleries_update with non-existent ID
    local result=$(dfx canister call backend galleries_update "(\"non-existent-gallery-123\", $update_data)" 2>/dev/null)
    
    # Should return failure (Result<T, Error> format)
    if is_failure "$result"; then
        echo_success "galleries_update correctly handled non-existent gallery"
    else
        echo_error "galleries_update should have failed for non-existent gallery"
        echo_debug "Response: $result"
        return 1
    fi
    
    echo_success "Non-existent gallery test passed"
}

test_galleries_update_verification() {
    echo_info "Verifying gallery was actually updated via galleries_list..."
    
    # Get galleries list
    local result=$(dfx canister call backend galleries_list 2>/dev/null)
    
    # Check if our updated gallery is in the list with new title (Result<Vec<Gallery>, Error> format)
    if is_success "$result" && echo "$result" | grep -q "\"$GALLERY_ID\""; then
        if echo "$result" | grep -q "Partially Updated Title"; then
            echo_success "Gallery found in galleries_list with updated title"
        else
            echo_error "Gallery found but title was not updated"
            echo_debug "galleries_list response: $result"
            return 1
        fi
    else
        echo_error "Gallery not found in galleries_list after update"
        echo_debug "galleries_list response: $result"
        return 1
    fi
    
    echo_success "Verification test passed"
}

test_old_endpoint_removed() {
    echo_info "Verifying that old update_gallery endpoint is removed..."
    
    # Try to call the old endpoint
    local update_data="(record { title = opt \"test\"; description = null; is_public = null; memory_entries = null; })"
    local result=$(dfx canister call backend update_gallery "(\"test\", $update_data)" 2>&1 || true)
    
    # Should get an error about method not found
    if echo "$result" | grep -q "method not found\|no update method"; then
        echo_success "Old update_gallery endpoint correctly removed"
    else
        echo_error "Old update_gallery endpoint still exists or returned unexpected error"
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
    run_test "Basic galleries_update functionality" "test_galleries_update_basic"
    run_test "galleries_update partial update" "test_galleries_update_partial"
    run_test "galleries_update non-existent gallery" "test_galleries_update_nonexistent"
    run_test "Gallery update verification" "test_galleries_update_verification"
    run_test "Old update_gallery endpoint removal" "test_old_endpoint_removed"
    
    echo_header "$TEST_NAME completed successfully! 🎉"
}

# Run main function
main "$@"


