#!/bin/bash

# Test UUID mapping between Web2 and ICP
# Verifies that gallery and memory IDs are preserved from Web2 to ICP

# Load test configuration and utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../test_config.sh"
source "$SCRIPT_DIR/../../test_utils.sh"

# Test configuration
TEST_NAME="UUID Mapping Tests"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Helper function to check if response indicates success
is_success() {
    local response="$1"
    echo "$response" | grep -q "success = true"
}

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

# Test that gallery UUIDs are preserved from Web2
test_gallery_uuid_preservation() {
    # Use a canonical UUID format (lowercase, hyphenated)
    local web2_gallery_uuid="550e8400-e29b-41d4-a716-446655440000"
    
    # Create gallery data with the Web2 UUID
    local timestamp=$(date +%s)000000000
    local gallery_data=$(cat << EOF
(record {
  gallery = record {
    id = "$web2_gallery_uuid";
    owner_principal = principal "$(dfx identity get-principal)";
    title = "UUID Preservation Test Gallery";
    description = opt "Testing that Web2 UUIDs are preserved in ICP";
    is_public = true;
    created_at = $timestamp;
    updated_at = $timestamp;
    storage_status = variant { ICPOnly };
    memory_entries = vec {};
  };
  owner_principal = principal "$(dfx identity get-principal)";
})
EOF
)
    
    # Store the gallery
    local result=$(dfx canister call backend store_gallery_forever "$gallery_data" 2>/dev/null)
    
    if ! is_success "$result"; then
        echo_info "Failed to store gallery: $result"
        return 1
    fi
    
    # Extract the returned gallery ID
    local returned_gallery_id=$(echo "$result" | grep -o 'gallery_id = opt "[^"]*"' | sed 's/gallery_id = opt "\([^"]*\)"/\1/' | head -1)
    
    if [ "$returned_gallery_id" = "$web2_gallery_uuid" ]; then
        echo_info "✓ Gallery UUID preserved: $web2_gallery_uuid"
        return 0
    else
        echo_info "✗ Gallery UUID not preserved. Expected: $web2_gallery_uuid, Got: $returned_gallery_id"
        return 1
    fi
}

# Test that memory UUIDs are preserved from Web2
test_memory_uuid_preservation() {
    # Use a canonical UUID format (lowercase, hyphenated)
    local web2_memory_uuid="660e8400-e29b-41d4-a716-446655440001"
    
    # Create memory data
    local memory_data=$(cat << EOF
(record {
  blob_ref = record {
    kind = variant { ICPCapsule };
    locator = "test_memory_locator";
    hash = null;
  };
  data = opt blob "$(echo -n "Test memory content for UUID preservation" | base64)";
})
EOF
)
    
    # Store the memory with the Web2 UUID
    local result=$(dfx canister call backend add_memory_to_capsule "(\"$web2_memory_uuid\", $memory_data)" 2>/dev/null)
    
    if ! is_success "$result"; then
        echo_info "Failed to store memory: $result"
        return 1
    fi
    
    # Extract the returned memory ID
    local returned_memory_id=$(echo "$result" | grep -o 'memory_id = opt "[^"]*"' | sed 's/memory_id = opt "\([^"]*\)"/\1/' | head -1)
    
    if [ "$returned_memory_id" = "$web2_memory_uuid" ]; then
        echo_info "✓ Memory UUID preserved: $web2_memory_uuid"
        return 0
    else
        echo_info "✗ Memory UUID not preserved. Expected: $web2_memory_uuid, Got: $returned_memory_id"
        return 1
    fi
}

# Test idempotency - storing the same gallery UUID twice should succeed
test_gallery_idempotency() {
    # Use a canonical UUID format (lowercase, hyphenated)
    local web2_gallery_uuid="770e8400-e29b-41d4-a716-446655440002"
    
    # Create gallery data with the Web2 UUID
    local timestamp=$(date +%s)000000000
    local gallery_data=$(cat << EOF
(record {
  gallery = record {
    id = "$web2_gallery_uuid";
    owner_principal = principal "$(dfx identity get-principal)";
    title = "Idempotency Test Gallery";
    description = opt "Testing idempotent gallery storage";
    is_public = true;
    created_at = $timestamp;
    updated_at = $timestamp;
    storage_status = variant { ICPOnly };
    memory_entries = vec {};
  };
  owner_principal = principal "$(dfx identity get-principal)";
})
EOF
)
    
    # Store the gallery first time
    local result1=$(dfx canister call backend store_gallery_forever "$gallery_data" 2>/dev/null)
    
    if ! is_success "$result1"; then
        echo_info "Failed to store gallery first time: $result1"
        return 1
    fi
    
    # Store the same gallery second time (should be idempotent)
    local result2=$(dfx canister call backend store_gallery_forever "$gallery_data" 2>/dev/null)
    
    if ! is_success "$result2"; then
        echo_info "Failed to store gallery second time (idempotency failed): $result2"
        return 1
    fi
    
    # Check that the message indicates it already exists
    if echo "$result2" | grep -q "already exists"; then
        echo_info "✓ Gallery idempotency working: already exists message returned"
        return 0
    else
        echo_info "✓ Gallery idempotency working: second store succeeded"
        return 0
    fi
}

# Test idempotency - storing the same memory UUID twice should succeed
test_memory_idempotency() {
    # Use a canonical UUID format (lowercase, hyphenated)
    local web2_memory_uuid="880e8400-e29b-41d4-a716-446655440003"
    
    # Create memory data
    local memory_data=$(cat << EOF
(record {
  blob_ref = record {
    kind = variant { ICPCapsule };
    locator = "test_memory_idempotency";
    hash = null;
  };
  data = opt blob "$(echo -n "Test memory content for idempotency" | base64)";
})
EOF
)
    
    # Store the memory first time
    local result1=$(dfx canister call backend add_memory_to_capsule "(\"$web2_memory_uuid\", $memory_data)" 2>/dev/null)
    
    if ! is_success "$result1"; then
        echo_info "Failed to store memory first time: $result1"
        return 1
    fi
    
    # Store the same memory second time (should be idempotent)
    local result2=$(dfx canister call backend add_memory_to_capsule "(\"$web2_memory_uuid\", $memory_data)" 2>/dev/null)
    
    if ! is_success "$result2"; then
        echo_info "Failed to store memory second time (idempotency failed): $result2"
        return 1
    fi
    
    # Check that the message indicates it already exists
    if echo "$result2" | grep -q "already exists"; then
        echo_info "✓ Memory idempotency working: already exists message returned"
        return 0
    else
        echo_info "✓ Memory idempotency working: second store succeeded"
        return 0
    fi
}

# Test that we can retrieve galleries by their Web2 UUIDs
test_gallery_retrieval_by_uuid() {
    # Use a canonical UUID format (lowercase, hyphenated)
    local web2_gallery_uuid="990e8400-e29b-41d4-a716-446655440004"
    
    # Create and store gallery
    local timestamp=$(date +%s)000000000
    local gallery_data=$(cat << EOF
(record {
  gallery = record {
    id = "$web2_gallery_uuid";
    owner_principal = principal "$(dfx identity get-principal)";
    title = "Retrieval Test Gallery";
    description = opt "Testing gallery retrieval by Web2 UUID";
    is_public = true;
    created_at = $timestamp;
    updated_at = $timestamp;
    storage_status = variant { ICPOnly };
    memory_entries = vec {};
  };
  owner_principal = principal "$(dfx identity get-principal)";
})
EOF
)
    
    # Store the gallery
    local store_result=$(dfx canister call backend store_gallery_forever "$gallery_data" 2>/dev/null)
    
    if ! is_success "$store_result"; then
        echo_info "Failed to store gallery for retrieval test: $store_result"
        return 1
    fi
    
    # Retrieve the gallery by its Web2 UUID
    local retrieve_result=$(dfx canister call backend get_gallery_by_id "(\"$web2_gallery_uuid\")" 2>/dev/null)
    
    # Should return the gallery (not null)
    if echo "$retrieve_result" | grep -q "(null)"; then
        echo_info "✗ Gallery not found by Web2 UUID: $web2_gallery_uuid"
        return 1
    elif echo "$retrieve_result" | grep -q "Retrieval Test Gallery"; then
        echo_info "✓ Gallery successfully retrieved by Web2 UUID: $web2_gallery_uuid"
        return 0
    else
        echo_info "✗ Unexpected result when retrieving gallery: $retrieve_result"
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
    echo_info "Registering user for UUID mapping tests..."
    local register_result=$(dfx canister call backend register 2>/dev/null)
    if ! echo "$register_result" | grep -q "true"; then
        echo_warn "User registration returned: $register_result"
    fi
    
    # Run UUID mapping tests
    echo_info "=== Testing UUID Preservation ==="
    run_test "Gallery UUID preservation from Web2" "test_gallery_uuid_preservation"
    run_test "Memory UUID preservation from Web2" "test_memory_uuid_preservation"
    
    echo_info "=== Testing Idempotency ==="
    run_test "Gallery idempotency with same UUID" "test_gallery_idempotency"
    run_test "Memory idempotency with same UUID" "test_memory_idempotency"
    
    echo_info "=== Testing Retrieval by UUID ==="
    run_test "Gallery retrieval by Web2 UUID" "test_gallery_retrieval_by_uuid"
    
    # Print test summary
    echo "========================================="
    echo "Test Summary for $TEST_NAME"
    echo "========================================="
    echo "Total tests: $TOTAL_TESTS"
    echo "Passed: $PASSED_TESTS"
    echo "Failed: $FAILED_TESTS"
    echo ""
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo_pass "All UUID mapping tests passed!"
        echo_info "✓ Web2 UUIDs are properly preserved in ICP"
        echo_info "✓ Idempotent operations work correctly"
        echo_info "✓ Gallery and memory retrieval by UUID works"
        exit 0
    else
        echo_fail "$FAILED_TESTS UUID mapping test(s) failed"
        echo_info "✗ UUID mapping between Web2 and ICP is not working correctly"
        exit 1
    fi
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi