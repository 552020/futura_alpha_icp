#!/bin/bash

# Test galleries_create endpoint functionality (replaces store_gallery_forever)
# This script tests the new galleries_create endpoint and verifies that the old store_gallery_forever is removed

set -e

# Source test utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../test_utils.sh"
source "$SCRIPT_DIR/gallery_test_utils.sh"

# Test configuration
TEST_NAME="Galleries Create Tests"
CANISTER_NAME="backend"

# Test data
GALLERY_ID="test-gallery-$(date +%s)"
GALLERY_NAME="Test Gallery for Create"
GALLERY_DESCRIPTION="A test gallery created via galleries_create endpoint"

# Test functions for galleries_create
test_galleries_create_basic() {
    echo_info "Testing galleries_create with basic gallery data..."
    
    # Create gallery data using shared utilities
    local gallery_data=$(create_basic_gallery_data "$GALLERY_ID" "$GALLERY_NAME" "$GALLERY_DESCRIPTION" "true" "Web2Only")
    
    # Call galleries_create
    local result=$(dfx canister call backend galleries_create "$gallery_data" 2>/dev/null)
    
    # Verify response structure (Result<T, Error> format)
    if is_success "$result"; then
        echo_success "galleries_create returned success"
    else
        echo_error "galleries_create failed or returned unexpected response"
        echo_debug "Response: $result"
        return 1
    fi
    
    # Verify gallery_id is returned
    local returned_id=$(extract_gallery_id "$result")
    if [[ "$returned_id" == "$GALLERY_ID" ]]; then
        echo_success "galleries_create returned correct gallery_id: $returned_id"
    else
        echo_error "galleries_create did not return expected gallery_id. Expected: $GALLERY_ID, Got: $returned_id"
        echo_debug "Response: $result"
        return 1
    fi
    
    echo_success "Basic galleries_create test passed"
}

test_galleries_create_idempotency() {
    echo_info "Testing galleries_create idempotency (calling twice with same ID)..."
    
    # Create gallery data using shared utilities (same ID as before)
    local gallery_data=$(create_basic_gallery_data "$GALLERY_ID" "$GALLERY_NAME Updated" "Updated description" "false" "Web2Only")
    
    # Call galleries_create again with same ID
    local result=$(dfx canister call backend galleries_create "$gallery_data" 2>/dev/null)
    
    # Should return success (idempotency - same result for duplicate calls)
    if is_success "$result"; then
        echo_success "galleries_create correctly handled duplicate ID (idempotency)"
    else
        echo_error "galleries_create failed on duplicate ID call"
        echo_debug "Response: $result"
        return 1
    fi
    
    echo_success "Idempotency test passed"
}

test_galleries_create_verification() {
    echo_info "Verifying gallery was actually created via galleries_list..."
    
    # Get galleries list
    local result=$(dfx canister call backend galleries_list 2>/dev/null)
    
    # Check if our gallery is in the list (Result<Vec<Gallery>, Error> format)
    if is_success "$result" && echo "$result" | grep -q "\"$GALLERY_ID\""; then
        echo_success "Gallery found in galleries_list after creation"
    else
        echo_error "Gallery not found in galleries_list after creation"
        echo_debug "galleries_list response: $result"
        return 1
    fi
    
    echo_success "Verification test passed"
}

test_galleries_create_with_memories() {
    echo_info "Testing galleries_create with memories data..."
    
    # Create new gallery ID for this test
    local gallery_id_with_memories="test-gallery-memories-$(date +%s)"
    
    # Create gallery data with memories using shared utilities
    local gallery_data=$(create_gallery_data_with_memories "$gallery_id_with_memories" "Gallery with Memories" "A test gallery with memory data" "true" "memory-1" "" "Web2Only")
    
    # Call galleries_create
    local result=$(dfx canister call backend galleries_create "$gallery_data" 2>/dev/null)
    
    # Verify response (Result<T, Error> format)
    if is_success "$result"; then
        echo_success "galleries_create with memories returned success"
    else
        echo_error "galleries_create with memories failed"
        echo_debug "Response: $result"
        return 1
    fi
    
    echo_success "galleries_create with memories test passed"
}

test_old_endpoint_removed() {
    echo_info "Verifying that old store_gallery_forever endpoint is removed..."
    
    # Try to call the old endpoint
    local result=$(dfx canister call backend store_gallery_forever "(record { gallery = record { id = \"test\"; title = \"test\"; description = opt \"test\"; is_public = true; created_at = 0; updated_at = 0; owner_principal = principal \"2vxsx-fae\"; storage_location = variant { Web2Only }; memory_entries = vec {}; }; owner_principal = principal \"2vxsx-fae\"; })" 2>&1 || true)
    
    # Should get an error about method not found
    if echo "$result" | grep -q "method not found\|no update method"; then
        echo_success "Old store_gallery_forever endpoint correctly removed"
    else
        echo_error "Old store_gallery_forever endpoint still exists or returned unexpected error"
        echo_debug "Response: $result"
        return 1
    fi
    
    echo_success "Old endpoint removal verification passed"
}

test_old_endpoint_with_memories_removed() {
    echo_info "Verifying that old store_gallery_forever_with_memories endpoint is removed..."
    
    # Try to call the old endpoint
    local result=$(dfx canister call backend store_gallery_forever_with_memories "(record { gallery = record { id = \"test\"; title = \"test\"; description = opt \"test\"; is_public = true; created_at = 0; updated_at = 0; owner_principal = principal \"2vxsx-fae\"; storage_location = variant { Web2Only }; memory_entries = vec {}; }; owner_principal = principal \"2vxsx-fae\"; }, true)" 2>&1 || true)
    
    # Should get an error about method not found
    if echo "$result" | grep -q "method not found\|no update method"; then
        echo_success "Old store_gallery_forever_with_memories endpoint correctly removed"
    else
        echo_error "Old store_gallery_forever_with_memories endpoint still exists or returned unexpected error"
        echo_debug "Response: $result"
        return 1
    fi
    
    echo_success "Old endpoint with memories removal verification passed"
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
    run_test "Basic galleries_create functionality" "test_galleries_create_basic"
    run_test "galleries_create idempotency" "test_galleries_create_idempotency"
    run_test "Gallery creation verification" "test_galleries_create_verification"
    run_test "galleries_create with memories" "test_galleries_create_with_memories"
    run_test "Old store_gallery_forever endpoint removal" "test_old_endpoint_removed"
    run_test "Old store_gallery_forever_with_memories endpoint removal" "test_old_endpoint_with_memories_removed"
    
    echo_header "$TEST_NAME completed successfully! 🎉"
}

# Run main function
main "$@"
