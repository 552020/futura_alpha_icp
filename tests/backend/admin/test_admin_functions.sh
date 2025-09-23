#!/bin/bash

# Simple Admin Functions Test
# Tests that admin functions exist and can be called

# Load test utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_utils.sh"

# Get canister ID
CANISTER_ID="${CANISTER_ID:-$(dfx canister id backend 2>/dev/null)}"
if [ -z "$CANISTER_ID" ]; then
    echo_error "Backend canister not found. Make sure it's deployed."
    exit 1
fi

echo_info "Testing admin functions with canister: $CANISTER_ID"

# Test principals (use helper function to get valid principal)
TEST_PRINCIPAL=$(get_test_principal "test-admin")

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Helper function
test_function() {
    local func_name="$1"
    local args="$2"
    local description="$3"
    
    echo_info "Testing: $description"
    
    if dfx canister call "$CANISTER_ID" "$func_name" "$args" --query >/dev/null 2>&1; then
        echo_pass "$description"
        ((TESTS_PASSED++))
    else
        echo_fail "$description"
        ((TESTS_FAILED++))
    fi
}

# Test admin functions exist and can be called
# Note: add_admin and remove_admin are update functions (not query functions)
# They should return "Unauthorized" for non-superadmins, which is correct behavior
echo_info "Testing: add_admin function exists (update function)"
if result=$(dfx canister call "$CANISTER_ID" add_admin "(principal \"$TEST_PRINCIPAL\")" 2>&1); then
    if echo "$result" | grep -q "Unauthorized"; then
        echo_pass "add_admin function exists (returns Unauthorized as expected)"
        ((TESTS_PASSED++))
    else
        echo_fail "add_admin function exists (unexpected response: $result)"
        ((TESTS_FAILED++))
    fi
else
    echo_fail "add_admin function exists (call failed)"
    ((TESTS_FAILED++))
fi

echo_info "Testing: remove_admin function exists (update function)"
if result=$(dfx canister call "$CANISTER_ID" remove_admin "(principal \"$TEST_PRINCIPAL\")" 2>&1); then
    if echo "$result" | grep -q "Unauthorized"; then
        echo_pass "remove_admin function exists (returns Unauthorized as expected)"
        ((TESTS_PASSED++))
    else
        echo_fail "remove_admin function exists (unexpected response: $result)"
        ((TESTS_FAILED++))
    fi
else
    echo_fail "remove_admin function exists (call failed)"
    ((TESTS_FAILED++))
fi

test_function "list_admins" "()" "list_admins function exists"
test_function "list_superadmins" "()" "list_superadmins function exists"

# Test that functions return expected types
echo_info "Testing function return types..."

# Test list_admins returns a vector
if result=$(dfx canister call "$CANISTER_ID" list_admins "()" --query 2>/dev/null); then
    if echo "$result" | grep -q "vec"; then
        echo_pass "list_admins returns vector"
        ((TESTS_PASSED++))
    else
        echo_fail "list_admins should return vector"
        ((TESTS_FAILED++))
    fi
else
    echo_fail "list_admins call failed"
    ((TESTS_FAILED++))
fi

# Test list_superadmins returns a vector
if result=$(dfx canister call "$CANISTER_ID" list_superadmins "()" --query 2>/dev/null); then
    if echo "$result" | grep -q "vec"; then
        echo_pass "list_superadmins returns vector"
        ((TESTS_PASSED++))
    else
        echo_fail "list_superadmins should return vector"
        ((TESTS_FAILED++))
    fi
else
    echo_fail "list_superadmins call failed"
    ((TESTS_FAILED++))
fi

# Note: is_admin is not exposed in the public Candid interface
# It's an internal function used for authorization checks

# Summary
echo_info "=== Admin Functions Test Summary ==="
echo_info "Tests passed: $TESTS_PASSED"
echo_info "Tests failed: $TESTS_FAILED"
echo_info "Total tests: $((TESTS_PASSED + TESTS_FAILED))"

if [ $TESTS_FAILED -eq 0 ]; then
    echo_success "All admin function tests passed!"
    exit 0
else
    echo_error "Some admin function tests failed!"
    exit 1
fi



