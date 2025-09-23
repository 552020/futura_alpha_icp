#!/bin/bash

# Admin Management Integration Tests
# Tests the actual admin functions in a real canister context
# Usage: ./test_admin_management.sh [--mainnet]

# Load test utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_utils.sh"

# Parse command line arguments
MAINNET_MODE=false
if [[ "$1" == "--mainnet" ]]; then
    MAINNET_MODE=true
    echo_info "Running in mainnet mode"
fi

# Get canister ID and network settings
if [[ "$MAINNET_MODE" == "true" ]]; then
    # Load mainnet configuration
    source "$SCRIPT_DIR/../mainnet/config.sh"
    CANISTER_ID="$MAINNET_CANISTER_ID"
    NETWORK_FLAG="--network $MAINNET_NETWORK"
    echo_info "Using mainnet canister: $CANISTER_ID"
else
    # Local mode
    CANISTER_ID="${CANISTER_ID:-$(dfx canister id backend 2>/dev/null)}"
    NETWORK_FLAG=""
    if [ -z "$CANISTER_ID" ]; then
        echo_error "Backend canister not found. Make sure it's deployed locally."
        exit 1
    fi
    echo_info "Using local canister: $CANISTER_ID"
fi

echo_info "Testing admin management with canister: $CANISTER_ID"

# Test principals (using helper function to get valid principals)
TEST_ADMIN1=$(get_test_principal "test-admin-1")
TEST_ADMIN2=$(get_test_principal "test-admin-2")
SUPERADMIN="otzfv-jscof-niinw-gtloq-25uz3-pglpg-u3kug-besf3-rzlbd-ylrmp-5ae"

# Check if current identity is a superadmin
IS_SUPERADMIN=$(check_superadmin_status "$CANISTER_ID" "$NETWORK_FLAG")

echo_info "Test principals:"
echo_info "  Superadmin: $SUPERADMIN"
echo_info "  Test Admin 1: $TEST_ADMIN1"
echo_info "  Test Admin 2: $TEST_ADMIN2"

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Use shared run_test function from test_utils.sh

# Test 1: List admins (should work for anyone)
run_test_with_counters "List admins (initial state)" \
    "dfx canister call $CANISTER_ID list_admins '()' --query" \
    "success" TESTS_PASSED TESTS_FAILED

# Test 2: Add admin (behavior depends on identity)
if [[ "$IS_SUPERADMIN" == "true" ]]; then
    run_test_with_counters "Add admin as superadmin (should succeed)" \
        "dfx canister call $CANISTER_ID add_admin '(principal \"$TEST_ADMIN1\")' | grep -q 'Ok'" \
        "success" TESTS_PASSED TESTS_FAILED
else
    run_test_with_counters "Add admin as non-superadmin (should fail with Unauthorized)" \
        "dfx canister call $CANISTER_ID add_admin '(principal \"$TEST_ADMIN1\")' | grep -q 'Unauthorized'" \
        "success" TESTS_PASSED TESTS_FAILED
fi

# Test 3: Add admin function exists (update function, not query)
run_test_with_counters "Add admin function exists" \
    "dfx canister call $CANISTER_ID add_admin '(principal \"$TEST_ADMIN1\")'" \
    "success" TESTS_PASSED TESTS_FAILED

# Test 4: Remove admin (behavior depends on identity)
if [[ "$IS_SUPERADMIN" == "true" ]]; then
    run_test_with_counters "Remove admin as superadmin (should succeed)" \
        "dfx canister call $CANISTER_ID remove_admin '(principal \"$TEST_ADMIN1\")' | grep -q 'Ok'" \
        "success" TESTS_PASSED TESTS_FAILED
else
    run_test_with_counters "Remove admin as non-superadmin (should fail with Unauthorized)" \
        "dfx canister call $CANISTER_ID remove_admin '(principal \"$TEST_ADMIN1\")' | grep -q 'Unauthorized'" \
        "success" TESTS_PASSED TESTS_FAILED
fi

# Test 5: Remove admin function exists (update function, not query)
run_test_with_counters "Remove admin function exists" \
    "dfx canister call $CANISTER_ID remove_admin '(principal \"$TEST_ADMIN1\")'" \
    "success" TESTS_PASSED TESTS_FAILED

# Test 6: List superadmins
run_test_with_counters "List superadmins" \
    "dfx canister call $CANISTER_ID list_superadmins '()' --query" \
    "success" TESTS_PASSED TESTS_FAILED

# Note: is_admin is not exposed in the public Candid interface
# It's an internal function used for authorization checks

# Test 7: Test with invalid principal format (expects dfx to fail/panic)
run_test_with_counters "Add admin with invalid principal (should fail)" \
    "dfx canister call $CANISTER_ID add_admin '(principal \"invalid-principal\")' 2>&1 | grep -q 'CRC32 check sequence'" \
    "success" TESTS_PASSED TESTS_FAILED

# Test 8: Test with empty principal (expects dfx to fail/panic)
run_test_with_counters "Add admin with empty principal (should fail)" \
    "dfx canister call $CANISTER_ID add_admin '(principal \"\")' 2>&1 | grep -q 'Text is too short'" \
    "success" TESTS_PASSED TESTS_FAILED

# Test 9: Test admin functions are available in Candid interface (via metadata)
run_test_with_counters "Admin functions in Candid interface" \
    "dfx canister metadata $CANISTER_ID candid:service | grep -E '(add_admin|remove_admin|list_admins|list_superadmins)'" \
    "success" TESTS_PASSED TESTS_FAILED

# Summary
echo_info "=== Admin Management Test Summary ==="
echo_info "Tests passed: $TESTS_PASSED"
echo_info "Tests failed: $TESTS_FAILED"
echo_info "Total tests: $((TESTS_PASSED + TESTS_FAILED))"

if [ $TESTS_FAILED -eq 0 ]; then
    echo_success "All admin management tests passed!"
    exit 0
else
    echo_error "Some admin management tests failed!"
    exit 1
fi
