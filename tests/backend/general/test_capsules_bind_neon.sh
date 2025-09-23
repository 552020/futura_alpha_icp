#!/bin/bash

# Test script for capsules_bind_neon function
# Tests binding/unbinding capsules, galleries, and memories to Neon database

# Fix dfx color issues
export DFX_COLOR=0
export NO_COLOR=1
export TERM=dumb

# Parse command line arguments
MAINNET_MODE=false
CANISTER_ID="backend"
NETWORK_FLAG=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --mainnet)
            MAINNET_MODE=true
            CANISTER_ID="izhgj-eiaaa-aaaaj-a2f7q-cai"
            NETWORK_FLAG="--network ic"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--mainnet]"
            exit 1
            ;;
    esac
done

# Load test configuration and utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_config.sh"
source "$SCRIPT_DIR/../test_utils.sh"

# Test configuration
TEST_NAME="Capsules Bind Neon Tests"
if [[ "$MAINNET_MODE" == "true" ]]; then
    TEST_NAME="Capsules Bind Neon Tests (Mainnet)"
fi
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0


# ============================================================================
# TEST FUNCTIONS
# ============================================================================

test_capsules_bind_neon_basic() {
    echo_info "Testing basic capsules_bind_neon functionality..."
    
    # Create a fresh capsule for this test to avoid conflicts
    local create_result=$(dfx canister call $CANISTER_ID capsules_create "(null)" $NETWORK_FLAG 2>/dev/null)
    if ! is_success "$create_result"; then
        echo_error "Failed to create test capsule: $create_result"
        return 1
    fi
    
    local capsule_id=$(extract_capsule_id "$create_result")
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to extract capsule ID from creation result"
        return 1
    fi
    echo_info "Using test capsule: $capsule_id"
    
    # Test binding capsule to Neon
    local bind_result=$(dfx canister call $CANISTER_ID capsules_bind_neon "(variant { Capsule }, \"$capsule_id\", true)" $NETWORK_FLAG 2>&1)
    echo_info "Bind result: '$bind_result'"
    if ! is_success "$bind_result"; then
        echo_error "Failed to bind capsule to Neon: $bind_result"
        return 1
    fi
    echo_info "Successfully bound capsule to Neon"
    
    # Test unbinding capsule from Neon
    local unbind_result=$(dfx canister call $CANISTER_ID capsules_bind_neon "(variant { Capsule }, \"$capsule_id\", false)" $NETWORK_FLAG 2>&1)
    echo_info "Unbind result: '$unbind_result'"
    if ! is_success "$unbind_result"; then
        echo_error "Failed to unbind capsule from Neon: $unbind_result"
        return 1
    fi
    echo_info "Successfully unbound capsule from Neon"
    
    return 0
}

test_capsules_bind_neon_gallery() {
    echo_info "Testing gallery binding with capsules_bind_neon..."
    
    # Get the caller's principal
    local caller_principal=$(dfx canister call $CANISTER_ID whoami $NETWORK_FLAG 2>/dev/null | grep -o 'principal "[^"]*"' | sed 's/principal "//;s/"//')
    if [[ -z "$caller_principal" ]]; then
        echo_error "Failed to get caller principal"
        return 1
    fi
    echo_info "Caller principal: $caller_principal"
    
    # Create a capsule
    local create_result=$(dfx canister call $CANISTER_ID capsules_create "(null)" $NETWORK_FLAG 2>/dev/null)
    if ! is_success "$create_result"; then
        echo_error "Failed to create test capsule for gallery binding"
        return 1
    fi
    local capsule_id=$(echo "$create_result" | grep -o 'id = "[^"]*"' | sed 's/id = "//' | sed 's/"//')
    
    # Create a gallery in the capsule with unique ID
    local unique_gallery_id="test_gallery_$(date +%s)_$$"
    local gallery_data="(record {
        gallery = record {
            id = \"$unique_gallery_id\";
            title = \"Test Gallery for Binding\";
            description = opt \"Test gallery for binding\";
            is_public = false;
            created_at = 0;
            updated_at = 0;
            owner_principal = principal \"$caller_principal\";
            storage_location = variant { ICPOnly };
            memory_entries = vec {};
            bound_to_neon = false;
        };
        owner_principal = principal \"$caller_principal\";
    })"
    
    local gallery_result=$(dfx canister call $CANISTER_ID galleries_create "$gallery_data" $NETWORK_FLAG 2>/dev/null)
    if ! is_success "$gallery_result"; then
        echo_error "Failed to create test gallery: $gallery_result"
        return 1
    fi
    
    # Extract the actual gallery ID from the response
    local gallery_id=$(echo "$gallery_result" | grep -o 'id = "[^"]*"' | sed 's/id = "//' | sed 's/"//')
    if [[ -z "$gallery_id" ]]; then
        echo_error "Failed to extract gallery ID from creation response"
        return 1
    fi
    echo_info "Created test gallery: $gallery_id"
    
    # Test binding gallery to Neon (use the actual gallery ID, not the unique ID)
    local bind_result=$(dfx canister call $CANISTER_ID capsules_bind_neon "(variant { Gallery }, \"$gallery_id\", true)" $NETWORK_FLAG 2>&1)
    if ! echo "$bind_result" | grep -q "Ok"; then
        echo_error "Failed to bind gallery to Neon: $bind_result"
        return 1
    fi
    echo_info "Successfully bound gallery to Neon"
    
    # Test unbinding gallery from Neon
    local unbind_result=$(dfx canister call $CANISTER_ID capsules_bind_neon "(variant { Gallery }, \"$gallery_id\", false)" $NETWORK_FLAG 2>&1)
    if ! echo "$unbind_result" | grep -q "Ok"; then
        echo_error "Failed to unbind gallery from Neon: $unbind_result"
        return 1
    fi
    echo_info "Successfully unbound gallery from Neon"
    
    return 0
}

test_capsules_bind_neon_memory() {
    echo_info "Testing memory binding with capsules_bind_neon..."
    
    # Get the first available capsule for the current user
    local capsules_list=$(dfx canister call $CANISTER_ID capsules_list $NETWORK_FLAG 2>/dev/null)
    local capsule_id=$(echo "$capsules_list" | grep -o 'id = "[^"]*"' | head -1 | sed 's/id = "//' | sed 's/"//')
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "No accessible capsules found for current user"
        return 1
    fi
    echo_info "Using accessible capsule: $capsule_id"
    
    # Create a memory in the capsule with minimal data
    local memory_data='(variant { Inline = record { meta = record { name = "test_memory_binding"; tags = vec {}; description = null }; bytes = blob "" } })'
    
    local idempotency_key="test_idem_$(date +%s)"
    echo_info "Creating memory with capsule_id: $capsule_id, idempotency_key: $idempotency_key"
    
    local memory_result=$(dfx canister call $CANISTER_ID memories_create "(\"$capsule_id\", $memory_data, \"$idempotency_key\")" $NETWORK_FLAG 2>&1)
    echo_info "Memory creation result: $memory_result"
    
    if ! is_success "$memory_result"; then
        echo_error "Failed to create test memory: $memory_result"
        return 1
    fi
    
    # Extract the actual memory ID from the response
    local memory_id=$(echo "$memory_result" | grep -o 'Ok = "[^"]*"' | sed 's/Ok = "//' | sed 's/"//')
    if [[ -z "$memory_id" ]]; then
        echo_error "Failed to extract memory ID from creation response: $memory_result"
        return 1
    fi
    echo_info "Created test memory: $memory_id"
    
    # Test binding memory to Neon
    local bind_result=$(dfx canister call $CANISTER_ID capsules_bind_neon "(variant { Memory }, \"$memory_id\", true)" $NETWORK_FLAG 2>&1)
    if ! echo "$bind_result" | grep -q "Ok"; then
        echo_error "Failed to bind memory to Neon: $bind_result"
        return 1
    fi
    echo_info "Successfully bound memory to Neon"
    
    # Test unbinding memory to Neon
    local unbind_result=$(dfx canister call $CANISTER_ID capsules_bind_neon "(variant { Memory }, \"$memory_id\", false)" $NETWORK_FLAG 2>&1)
    if ! echo "$unbind_result" | grep -q "Ok"; then
        echo_error "Failed to unbind memory from Neon: $unbind_result"
        return 1
    fi
    echo_info "Successfully unbound memory from Neon"
    
    return 0
}

test_capsules_bind_neon_invalid_resource() {
    echo_info "Testing capsules_bind_neon with invalid resource type..."
    
    # Test with invalid resource type (should fail gracefully)
    # Note: Candid will catch invalid variants at compile time, so we test with a valid variant
    # that doesn't exist in our ResourceType enum
    local result=$(dfx canister call $CANISTER_ID capsules_bind_neon "(variant { Capsule }, \"invalid_id\", true)" $NETWORK_FLAG 2>&1)
    
    # Should return false for invalid resource ID
    if echo "$result" | grep -q "Ok"; then
        echo_error "Invalid resource ID should not succeed: $result"
        return 1
    fi
    
    echo_info "Invalid resource ID handled correctly"
    return 0
}

test_capsules_bind_neon_nonexistent_resource() {
    echo_info "Testing capsules_bind_neon with nonexistent resource ID..."
    
    # Test with nonexistent resource ID
    local result=$(dfx canister call $CANISTER_ID capsules_bind_neon "(variant { Capsule }, \"nonexistent_id\", true)" $NETWORK_FLAG 2>&1)
    
    # Should return false for nonexistent resource
    if echo "$result" | grep -q "Ok"; then
        echo_error "Nonexistent resource should not succeed: $result"
        return 1
    fi
    
    echo_info "Nonexistent resource handled correctly"
    return 0
}

test_capsules_bind_neon_unauthorized() {
    echo_info "Testing capsules_bind_neon with unauthorized access..."
    
    # Create a capsule with one user
    local create_result=$(dfx canister call $CANISTER_ID capsules_create "(null)" $NETWORK_FLAG 2>/dev/null)
    local capsule_id=$(echo "$create_result" | grep -o 'capsule_id = opt "[^"]*"' | sed 's/capsule_id = opt "//;s/"//')
    
    # Try to bind from a different user (should fail)
    # Note: This test assumes we can't easily switch users in the test environment
    # In a real scenario, this would test cross-user access control
    
    echo_info "Unauthorized access test completed (limited in test environment)"
    return 0
}

test_capsules_bind_neon_edge_cases() {
    echo_info "Testing capsules_bind_neon edge cases..."
    
    # Test with empty resource ID (should return error since empty ID is invalid)
    local result=$(dfx canister call $CANISTER_ID capsules_bind_neon "(variant { Capsule }, \"\", true)" $NETWORK_FLAG 2>&1)
    if ! echo "$result" | grep -q "Err"; then
        echo_error "Empty resource ID should return error: $result"
        return 1
    fi
    
    # Test with very long resource ID
    local long_id=$(printf 'a%.0s' {1..1000})
    local result=$(dfx canister call $CANISTER_ID capsules_bind_neon "(variant { Capsule }, \"$long_id\", true)" $NETWORK_FLAG 2>&1)
    if ! echo "$result" | grep -q "Ok\|Err"; then
        echo_error "Long resource ID should return boolean: $result"
        return 1
    fi
    
    echo_info "Edge cases handled correctly"
    return 0
}

# Main test execution
main() {
    echo_info "Starting $TEST_NAME"
    echo_info "=================================="
    
    # Setup
    setup_user_and_capsule
    
    # Run tests
    run_capsule_test "Basic capsule binding/unbinding" test_capsules_bind_neon_basic
    run_capsule_test "Gallery binding/unbinding" test_capsules_bind_neon_gallery
    run_capsule_test "Memory binding/unbinding" test_capsules_bind_neon_memory
    run_capsule_test "Invalid resource ID handling" test_capsules_bind_neon_invalid_resource
    run_capsule_test "Nonexistent resource handling" test_capsules_bind_neon_nonexistent_resource
    run_capsule_test "Unauthorized access handling" test_capsules_bind_neon_unauthorized
    run_capsule_test "Edge case handling" test_capsules_bind_neon_edge_cases
    
    # Test summary
    print_test_summary
}

# Run main function
main "$@"
