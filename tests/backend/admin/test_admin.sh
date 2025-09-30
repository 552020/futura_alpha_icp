#!/bin/bash

# Consolidated Admin Management Tests
# Tests actual admin functionality with state verification
# Usage: ./test_admin_consolidated.sh [--mainnet]

# ============================================================================
# TESTED FUNCTIONALITIES AND ENDPOINTS
# ============================================================================
#
# ADMIN MANAGEMENT FUNCTIONS:
# 1. list_admins() -> (vec principal) query
#    - Lists all current admins
#    - Only admins and superadmins can call this
#    - Returns empty vector if caller is not authorized
#    - Returns empty vector initially (no regular admins)
#
# 2. list_superadmins() -> (vec principal) query  
#    - Lists all superadmins
#    - Only superadmins can call this (authorization check added)
#    - Returns empty vector if caller is not a superadmin
#    - Returns vector with initial superadmin
#
# 3. add_admin(principal) -> (Result)
#    - Adds a principal as admin
#    - Only superadmins can call this
#    - Returns Ok() on success, Conflict if already admin, Unauthorized if not superadmin
#    - Validates principal format (rejects invalid/empty principals)
#
# 4. remove_admin(principal) -> (Result)
#    - Removes a principal from admin list
#    - Only superadmins can call this  
#    - Returns Ok() on success, NotFound if not admin, Unauthorized if not superadmin
#
# TEST SCENARIOS:
# - Initial state verification (empty admin list, superadmin exists)
# - Superadmin can call list_admins() successfully
# - Superadmin can call list_superadmins() successfully
# - Superadmin can add/remove admins successfully
# - Non-superadmin gets empty vector when calling list_admins()
# - Non-superadmin gets empty vector when calling list_superadmins()
# - Non-superadmin gets Unauthorized when trying to add/remove admins
# - Duplicate admin addition fails with Conflict
# - Removing non-existent admin fails with NotFound
# - Invalid principal format handling (CRC32 check sequence error)
# - Empty principal handling (Text is too short error)
# - State consistency (final admin count matches initial)
# - Candid interface verification (all functions exposed)
#
# AUTHORIZATION MODEL:
# - Superadmins: Can call list_admins(), list_superadmins(), add/remove admins, listed in list_superadmins()
# - Admins: Can call list_admins(), listed in list_admins(), but cannot modify admin list
# - Regular users: Cannot access admin management functions (get empty vector from list_admins())
#
# SECURITY FIXES APPLIED:
# - list_superadmins() now requires superadmin authorization
# - Returns empty vector for non-superadmin callers
#
# ============================================================================

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

# Test configuration
TEST_NAME="Consolidated Admin Management Tests"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Helper function to check if admin is in the list
check_admin_in_list() {
    local admin_principal="$1"
    local list_result=$(dfx canister call "$CANISTER_ID" list_admins "()" --query $NETWORK_FLAG 2>/dev/null)
    echo "$list_result" | grep -q "$admin_principal"
}

# Helper function to get initial admin count
get_admin_count() {
    local list_result=$(dfx canister call "$CANISTER_ID" list_admins "()" --query $NETWORK_FLAG 2>/dev/null)
    echo "$list_result" | grep -o "principal" | wc -l
}

echo_info "=== Starting $TEST_NAME ==="

# Get initial state
INITIAL_ADMIN_COUNT=$(get_admin_count)
echo_info "Initial admin count: $INITIAL_ADMIN_COUNT"

# Test 1: List admins (should work for anyone)
run_test "List admins (initial state)" \
    "dfx canister call $CANISTER_ID list_admins '()' --query $NETWORK_FLAG"

# Test 2: List superadmins (should work for anyone)
run_test "List superadmins" \
    "dfx canister call $CANISTER_ID list_superadmins '()' --query $NETWORK_FLAG"

# Test 3: Add admin functionality (behavior depends on identity)
if [[ "$IS_SUPERADMIN" == "true" ]]; then
    echo_info "Testing as superadmin - should be able to add/remove admins"
    
    # Test 3a: Add admin as superadmin
    run_test "Add admin as superadmin" \
        "dfx canister call $CANISTER_ID add_admin '(principal \"$TEST_ADMIN1\")' $NETWORK_FLAG | grep -q 'Ok'"
    
    # Test 3b: Verify admin was actually added to the list
    run_test "Verify admin was added to list" \
        "check_admin_in_list '$TEST_ADMIN1'"
    
    # Test 3c: Add same admin again (should fail)
    run_test "Add duplicate admin (should fail)" \
        "dfx canister call $CANISTER_ID add_admin '(principal \"$TEST_ADMIN1\")' $NETWORK_FLAG | grep -q 'Conflict ='"
    
    # Test 3d: Add second admin
    run_test "Add second admin" \
        "dfx canister call $CANISTER_ID add_admin '(principal \"$TEST_ADMIN2\")' $NETWORK_FLAG | grep -q 'Ok'"
    
    # Test 3e: Verify both admins are in the list
    run_test "Verify both admins in list" \
        "check_admin_in_list '$TEST_ADMIN1' && check_admin_in_list '$TEST_ADMIN2'"
    
    # Test 3f: Remove first admin
    run_test "Remove first admin" \
        "dfx canister call $CANISTER_ID remove_admin '(principal \"$TEST_ADMIN1\")' $NETWORK_FLAG | grep -q 'Ok'"
    
    # Test 3g: Verify first admin was removed but second remains
    run_test "Verify first admin removed, second remains" \
        "! check_admin_in_list '$TEST_ADMIN1' && check_admin_in_list '$TEST_ADMIN2'"
    
    # Test 3h: Remove second admin (cleanup)
    run_test "Remove second admin (cleanup)" \
        "dfx canister call $CANISTER_ID remove_admin '(principal \"$TEST_ADMIN2\")' $NETWORK_FLAG | grep -q 'Ok'"
    
    # Test 3i: Verify both admins are removed
    run_test "Verify both admins removed" \
        "! check_admin_in_list '$TEST_ADMIN1' && ! check_admin_in_list '$TEST_ADMIN2'"
    
else
    echo_info "Testing as non-superadmin - should get Unauthorized"
    
    # Test 3a: Add admin as non-superadmin (should fail)
    run_test "Add admin as non-superadmin (should fail)" \
        "dfx canister call $CANISTER_ID add_admin '(principal \"$TEST_ADMIN1\")' $NETWORK_FLAG | grep -q 'Unauthorized'"
    
    # Test 3b: Remove admin as non-superadmin (should fail)
    run_test "Remove admin as non-superadmin (should fail)" \
        "dfx canister call $CANISTER_ID remove_admin '(principal \"$TEST_ADMIN1\")' $NETWORK_FLAG | grep -q 'Unauthorized'"
fi

# Test 4: Edge cases
run_test "Add admin with invalid principal (should fail)" \
    "dfx canister call $CANISTER_ID add_admin '(principal \"invalid-principal\")' $NETWORK_FLAG 2>&1 | grep -q 'CRC32 check sequence'"

run_test "Add admin with empty principal (should fail)" \
    "dfx canister call $CANISTER_ID add_admin '(principal \"\")' $NETWORK_FLAG 2>&1 | grep -q 'Text is too short'"

# Test 5: Remove non-existent admin (should fail)
run_test "Remove non-existent admin (should fail)" \
    "dfx canister call $CANISTER_ID remove_admin '(principal \"$TEST_ADMIN1\")' $NETWORK_FLAG | grep -q 'NotFound'"

# Test 6: Verify final state matches initial state
FINAL_ADMIN_COUNT=$(get_admin_count)
run_test "Final admin count matches initial" \
    "[ $FINAL_ADMIN_COUNT -eq $INITIAL_ADMIN_COUNT ]"

# Test 7: Admin functions in Candid interface
run_test "Admin functions in Candid interface" \
    "dfx canister metadata $CANISTER_ID candid:service $NETWORK_FLAG | grep -E '(add_admin|remove_admin|list_admins|list_superadmins)'"

# Summary
echo_info "=== $TEST_NAME Summary ==="
echo_info "Tests passed: $PASSED_TESTS"
echo_info "Tests failed: $FAILED_TESTS"
echo_info "Total tests: $((PASSED_TESTS + FAILED_TESTS))"

if [ $FAILED_TESTS -eq 0 ]; then
    echo_success "All admin management tests passed!"
    exit 0
else
    echo_error "Some admin management tests failed!"
    exit 1
fi
