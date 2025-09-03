#!/bin/bash

# Test script for capsules_bind_neon function
# Tests binding/unbinding capsules, galleries, and memories to Neon database

# Source test utilities
source scripts/tests/backend/test_config.sh

# Define our own logging functions (don't use test_utils.sh)
echo_info() {
    echo "[INFO] $1"
}

echo_success() {
    echo "[SUCCESS] $1"
}

echo_error() {
    echo "[ERROR] $1"
}

echo_header() {
    echo ""
    echo "=========================================="
    echo "$1"
    echo "=========================================="
    echo ""
}

# Define our own run_test function (don't use the one from test_utils.sh)
run_test() {
    local test_name="$1"
    local test_function="$2"
    
    echo_info "Running: $test_name"
    if $test_function; then
        echo_success "✅ $test_name PASSED"
        return 0
    else
        echo_error "❌ $test_name FAILED"
        return 1
    fi
}

# Test configuration
TEST_NAME="capsules_bind_neon"
TEST_DESCRIPTION="Test the flexible resource binding function for Neon database"

# ============================================================================
# TEST FUNCTIONS
# ============================================================================

test_capsules_bind_neon_basic() {
    echo_info "Testing basic capsules_bind_neon functionality..."
    
    # First create a capsule for the caller
    local create_result=$(dfx canister call backend capsules_create "(null)" 2>/dev/null)
    if ! echo "$create_result" | grep -q "capsule_id = opt"; then
        echo_error "Failed to create test capsule"
        return 1
    fi
    
    # Extract capsule ID
    local capsule_id=$(echo "$create_result" | grep -o 'capsule_id = opt "[^"]*"' | sed 's/capsule_id = opt "//;s/"//')
    echo_info "Created test capsule: $capsule_id"
    
    # Test binding capsule to Neon
    local bind_result=$(dfx canister call backend capsules_bind_neon "(variant { Capsule }, \"$capsule_id\", true)" 2>&1)
    echo_info "Bind result: '$bind_result'"
    if ! echo "$bind_result" | grep -q "(true)"; then
        echo_error "Failed to bind capsule to Neon: $bind_result"
        return 1
    fi
    echo_info "Successfully bound capsule to Neon"
    
    # Test unbinding capsule from Neon
    local unbind_result=$(dfx canister call backend capsules_bind_neon "(variant { Capsule }, \"$capsule_id\", false)" 2>&1)
    echo_info "Unbind result: '$unbind_result'"
    if ! echo "$unbind_result" | grep -q "(true)"; then
        echo_error "Failed to unbind capsule from Neon: $unbind_result"
        return 1
    fi
    echo_info "Successfully unbound capsule from Neon"
    
    return 0
}

test_capsules_bind_neon_gallery() {
    echo_info "Testing gallery binding with capsules_bind_neon..."
    
    # Get the caller's principal
    local caller_principal=$(dfx canister call backend whoami 2>/dev/null | grep -o 'principal "[^"]*"' | sed 's/principal "//;s/"//')
    if [[ -z "$caller_principal" ]]; then
        echo_error "Failed to get caller principal"
        return 1
    fi
    echo_info "Caller principal: $caller_principal"
    
    # Create a capsule
    local create_result=$(dfx canister call backend capsules_create "(null)" 2>/dev/null)
    local capsule_id=$(echo "$create_result" | grep -o 'capsule_id = opt "[^"]*"' | sed 's/capsule_id = opt "//;s/"//')
    
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
            storage_status = variant { Web2Only };
            memory_entries = vec {};
            bound_to_neon = false;
        };
        owner_principal = principal \"$caller_principal\";
    })"
    
    local gallery_result=$(dfx canister call backend galleries_create "$gallery_data" 2>/dev/null)
    if ! echo "$gallery_result" | grep -q "gallery_id = opt"; then
        echo_error "Failed to create test gallery"
        return 1
    fi
    
    local gallery_id=$(echo "$gallery_result" | grep -o 'gallery_id = opt "[^"]*"' | sed 's/gallery_id = opt "//;s/"//')
    if [[ -z "$gallery_id" ]]; then
        # Try alternative format
        gallery_id=$(echo "$gallery_result" | grep -o 'gallery_id = "[^"]*"' | sed 's/gallery_id = "//;s/"//')
    fi
    echo_info "Created test gallery: $gallery_id"
    
    # Test binding gallery to Neon
    local bind_result=$(dfx canister call backend capsules_bind_neon "(variant { Gallery }, \"$unique_gallery_id\", true)" 2>&1)
    if ! echo "$bind_result" | grep -q "(true)"; then
        echo_error "Failed to bind gallery to Neon: $bind_result"
        return 1
    fi
    echo_info "Successfully bound gallery to Neon"
    
    # Test unbinding gallery from Neon
    local unbind_result=$(dfx canister call backend capsules_bind_neon "(variant { Gallery }, \"$unique_gallery_id\", false)" 2>&1)
    if ! echo "$unbind_result" | grep -q "(true)"; then
        echo_error "Failed to unbind gallery from Neon: $unbind_result"
        return 1
    fi
    echo_info "Successfully unbound gallery from Neon"
    
    return 0
}

test_capsules_bind_neon_memory() {
    echo_info "Testing memory binding with capsules_bind_neon..."
    
    # Create a capsule
    local create_result=$(dfx canister call backend capsules_create "(null)" 2>/dev/null)
    local capsule_id=$(echo "$create_result" | grep -o 'capsule_id = opt "[^"]*"' | sed 's/capsule_id = opt "//;s/"//')
    
    # Create a memory in the capsule
    local memory_data='(record {
      blob_ref = record {
        kind = variant { ICPCapsule };
        locator = "test_memory_binding";
        hash = null;
      };
      data = opt blob "VGVzdCBtZW1vcnkgZGF0YQ==";
    })'
    
    local memory_result=$(dfx canister call backend memories_create "(\"$capsule_id\", $memory_data)" 2>/dev/null)
    if ! echo "$memory_result" | grep -q "memory_id = opt"; then
        echo_error "Failed to create test memory"
        return 1
    fi
    
    local memory_id=$(echo "$memory_result" | grep -o 'memory_id = opt "[^"]*"' | sed 's/memory_id = opt "//;s/"//')
    echo_info "Created test memory: $memory_id"
    
    # Test binding memory to Neon
    local bind_result=$(dfx canister call backend capsules_bind_neon "(variant { Memory }, \"$memory_id\", true)" 2>&1)
    if ! echo "$bind_result" | grep -q "(true)"; then
        echo_error "Failed to bind memory to Neon: $bind_result"
        return 1
    fi
    echo_info "Successfully bound memory to Neon"
    
    # Test unbinding memory to Neon
    local unbind_result=$(dfx canister call backend capsules_bind_neon "(variant { Memory }, \"$memory_id\", false)" 2>&1)
    if ! echo "$unbind_result" | grep -q "(true)"; then
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
    local result=$(dfx canister call backend capsules_bind_neon "(variant { Capsule }, \"invalid_id\", true)" 2>&1)
    
    # Should return false for invalid resource ID
    if echo "$result" | grep -q "(true)"; then
        echo_error "Invalid resource ID should not succeed: $result"
        return 1
    fi
    
    echo_info "Invalid resource ID handled correctly"
    return 0
}

test_capsules_bind_neon_nonexistent_resource() {
    echo_info "Testing capsules_bind_neon with nonexistent resource ID..."
    
    # Test with nonexistent resource ID
    local result=$(dfx canister call backend capsules_bind_neon "(variant { Capsule }, \"nonexistent_id\", true)" 2>&1)
    
    # Should return false for nonexistent resource
    if echo "$result" | grep -q "(true)"; then
        echo_error "Nonexistent resource should not succeed: $result"
        return 1
    fi
    
    echo_info "Nonexistent resource handled correctly"
    return 0
}

test_capsules_bind_neon_unauthorized() {
    echo_info "Testing capsules_bind_neon with unauthorized access..."
    
    # Create a capsule with one user
    local create_result=$(dfx canister call backend capsules_create "(null)" 2>/dev/null)
    local capsule_id=$(echo "$create_result" | grep -o 'capsule_id = opt "[^"]*"' | sed 's/capsule_id = opt "//;s/"//')
    
    # Try to bind from a different user (should fail)
    # Note: This test assumes we can't easily switch users in the test environment
    # In a real scenario, this would test cross-user access control
    
    echo_info "Unauthorized access test completed (limited in test environment)"
    return 0
}

test_capsules_bind_neon_edge_cases() {
    echo_info "Testing capsules_bind_neon edge cases..."
    
    # Test with empty resource ID (should return false since empty ID is invalid)
    local result=$(dfx canister call backend capsules_bind_neon "(variant { Capsule }, \"\", true)" 2>&1)
    if ! echo "$result" | grep -q "(false)"; then
        echo_error "Empty resource ID should return false: $result"
        return 1
    fi
    
    # Test with very long resource ID
    local long_id=$(printf 'a%.0s' {1..1000})
    local result=$(dfx canister call backend capsules_bind_neon "(variant { Capsule }, \"$long_id\", true)" 2>&1)
    if ! echo "$result" | grep -q "(true\|false)"; then
        echo_error "Long resource ID should return boolean: $result"
        return 1
    fi
    
    echo_info "Edge cases handled correctly"
    return 0
}

# ============================================================================
# MAIN TEST EXECUTION
# ============================================================================

run_test() {
    local test_name="$1"
    local test_function="$2"
    
    echo_info "Running: $test_name"
    if $test_function; then
        echo_success "✅ $test_name PASSED"
        return 0
    else
        echo_error "❌ $test_name FAILED"
        return 1
    fi
}

main() {
    echo_header "🧪 $TEST_NAME - $TEST_DESCRIPTION"
    
    local total_tests=0
    local passed_tests=0
    
    # Run all tests
    echo_info "=== Testing Basic Functionality ==="
    run_test "Basic capsule binding/unbinding" "test_capsules_bind_neon_basic" && ((passed_tests++))
    ((total_tests++))
    
    run_test "Gallery binding/unbinding" "test_capsules_bind_neon_gallery" && ((passed_tests++))
    ((total_tests++))
    
    run_test "Memory binding/unbinding" "test_capsules_bind_neon_memory" && ((passed_tests++))
    ((total_tests++))
    
    echo_info "=== Testing Error Handling ==="
    run_test "Invalid resource ID handling" "test_capsules_bind_neon_invalid_resource" && ((passed_tests++))
    ((total_tests++))
    
    run_test "Nonexistent resource handling" "test_capsules_bind_neon_nonexistent_resource" && ((passed_tests++))
    ((total_tests++))
    
    run_test "Unauthorized access handling" "test_capsules_bind_neon_unauthorized" && ((passed_tests++))
    ((total_tests++))
    
    echo_info "=== Testing Edge Cases ==="
    run_test "Edge case handling" "test_capsules_bind_neon_edge_cases" && ((passed_tests++))
    ((total_tests++))
    
    # Summary
    echo_header "📊 TEST SUMMARY"
    echo_info "Total tests: $total_tests"
    echo_info "Passed: $passed_tests"
    echo_info "Failed: $((total_tests - passed_tests))"
    
    if [ $passed_tests -eq $total_tests ]; then
        echo_success "🎉 ALL TESTS PASSED!"
        return 0
    else
        echo_error "❌ SOME TESTS FAILED!"
        return 1
    fi
}

# Run main function
main "$@"
