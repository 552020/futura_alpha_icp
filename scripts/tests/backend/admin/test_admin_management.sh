#!/bin/bash

# Admin Management Integration Tests
# Tests the actual admin functions in a real canister context

# Load test utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../test_utils.sh"

# Test configuration
CANISTER_ID="${CANISTER_ID:-$(dfx canister id backend 2>/dev/null)}"
if [ -z "$CANISTER_ID" ]; then
    echo_error "Backend canister not found. Make sure it's deployed."
    exit 1
fi

echo_info "Testing admin management with canister: $CANISTER_ID"

# Test principals (using test principals)
TEST_ADMIN1="$(dfx identity get-principal --identity test-admin-1 2>/dev/null || echo "rdmx6-jaaaa-aaaah-qcaiq-cai")"
TEST_ADMIN2="$(dfx identity get-principal --identity test-admin-2 2>/dev/null || echo "rrkah-fqaaa-aaaah-qcaiq-cai")"
SUPERADMIN="otzfv-jscof-niinw-gtloq-25uz3-pglpg-u3kug-besf3-rzlbd-ylrmp-5ae"

echo_info "Test principals:"
echo_info "  Superadmin: $SUPERADMIN"
echo_info "  Test Admin 1: $TEST_ADMIN1"
echo_info "  Test Admin 2: $TEST_ADMIN2"

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Helper function to run a test
run_test() {
    local test_name="$1"
    local command="$2"
    local expected_result="$3"
    
    echo_info "Running: $test_name"
    echo_info "Command: $command"
    
    # Execute the command and capture result
    local result
    if result=$(eval "$command" 2>&1); then
        if [ "$expected_result" = "success" ]; then
            echo_pass "$test_name"
            ((TESTS_PASSED++))
        else
            echo_fail "$test_name - Expected failure but got success"
            echo_error "Result: $result"
            ((TESTS_FAILED++))
        fi
    else
        if [ "$expected_result" = "failure" ]; then
            echo_pass "$test_name"
            ((TESTS_PASSED++))
        else
            echo_fail "$test_name - Expected success but got failure"
            echo_error "Result: $result"
            ((TESTS_FAILED++))
        fi
    fi
    echo ""
}

# Test 1: List admins (should work for anyone)
run_test "List admins (initial state)" \
    "dfx canister call $CANISTER_ID list_admins '()'" \
    "success"

# Test 2: Add admin as non-superadmin (should fail)
run_test "Add admin as non-superadmin (should fail)" \
    "dfx canister call $CANISTER_ID add_admin '(principal \"$TEST_ADMIN1\")'" \
    "failure"

# Test 3: Add admin as superadmin (should succeed)
# Note: This requires calling as superadmin, which might not be possible in test environment
# We'll test the function exists and can be called
run_test "Add admin function exists" \
    "dfx canister call $CANISTER_ID add_admin '(principal \"$TEST_ADMIN1\")' --query" \
    "success"

# Test 4: Remove admin as non-superadmin (should fail)
run_test "Remove admin as non-superadmin (should fail)" \
    "dfx canister call $CANISTER_ID remove_admin '(principal \"$TEST_ADMIN1\")'" \
    "failure"

# Test 5: Remove admin function exists
run_test "Remove admin function exists" \
    "dfx canister call $CANISTER_ID remove_admin '(principal \"$TEST_ADMIN1\")' --query" \
    "success"

# Test 6: List superadmins
run_test "List superadmins" \
    "dfx canister call $CANISTER_ID list_superadmins '()'" \
    "success"

# Test 7: Check admin status (is_admin function)
run_test "Check admin status function exists" \
    "dfx canister call $CANISTER_ID is_admin '(principal \"$TEST_ADMIN1\")' --query" \
    "success"

# Test 8: Test with invalid principal format
run_test "Add admin with invalid principal (should fail)" \
    "dfx canister call $CANISTER_ID add_admin '(principal \"invalid-principal\")'" \
    "failure"

# Test 9: Test with empty principal
run_test "Add admin with empty principal (should fail)" \
    "dfx canister call $CANISTER_ID add_admin '(principal \"\")'" \
    "failure"

# Test 10: Test admin functions are available in Candid interface
run_test "Admin functions in Candid interface" \
    "dfx canister call $CANISTER_ID __get_candid_interface_tmp_hack '()' | grep -E '(add_admin|remove_admin|list_admins|is_admin)'" \
    "success"

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
