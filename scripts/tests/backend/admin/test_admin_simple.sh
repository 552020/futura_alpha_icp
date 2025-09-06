#!/bin/bash

# Simple Admin Functions Test
# Tests that admin functions exist and can be called

echo "Testing admin functions..."

# Get canister ID
CANISTER_ID="${CANISTER_ID:-$(dfx canister id backend 2>/dev/null)}"
if [ -z "$CANISTER_ID" ]; then
    echo "ERROR: Backend canister not found. Make sure it's deployed."
    exit 1
fi

echo "Using canister: $CANISTER_ID"

# Test principals
TEST_PRINCIPAL="ur7ny-sza5i-m73am-naljv-rgxjo-bzm2w-k7q6l-p2qsc-b2mce-qlpsr-uae"

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Helper function
test_function() {
    local func_name="$1"
    local args="$2"
    local description="$3"
    
    echo "Testing: $description"
    
    if dfx canister call "$CANISTER_ID" "$func_name" "$args" --query >/dev/null 2>&1; then
        echo "PASS: $description"
        ((TESTS_PASSED++))
    else
        echo "FAIL: $description"
        ((TESTS_FAILED++))
    fi
}

# Test admin functions exist and can be called
# Note: add_admin and remove_admin are update functions, not query functions
echo "Testing: add_admin function exists"
if dfx canister call "$CANISTER_ID" add_admin "(principal \"$TEST_PRINCIPAL\")" >/dev/null 2>&1; then
    echo "PASS: add_admin function exists"
    ((TESTS_PASSED++))
else
    echo "FAIL: add_admin function exists"
    ((TESTS_FAILED++))
fi

echo "Testing: remove_admin function exists"
if dfx canister call "$CANISTER_ID" remove_admin "(principal \"$TEST_PRINCIPAL\")" >/dev/null 2>&1; then
    echo "PASS: remove_admin function exists"
    ((TESTS_PASSED++))
else
    echo "FAIL: remove_admin function exists"
    ((TESTS_FAILED++))
fi
test_function "list_admins" "()" "list_admins function exists"
test_function "list_superadmins" "()" "list_superadmins function exists"

# Test that functions return expected types
echo "Testing function return types..."

# Test list_admins returns a vector
if result=$(dfx canister call "$CANISTER_ID" list_admins "()" --query 2>/dev/null); then
    if echo "$result" | grep -q "vec"; then
        echo "PASS: list_admins returns vector"
        ((TESTS_PASSED++))
    else
        echo "FAIL: list_admins should return vector"
        ((TESTS_FAILED++))
    fi
else
    echo "FAIL: list_admins call failed"
    ((TESTS_FAILED++))
fi

# Test list_superadmins returns a vector
if result=$(dfx canister call "$CANISTER_ID" list_superadmins "()" --query 2>/dev/null); then
    if echo "$result" | grep -q "vec"; then
        echo "PASS: list_superadmins returns vector"
        ((TESTS_PASSED++))
    else
        echo "FAIL: list_superadmins should return vector"
        ((TESTS_FAILED++))
    fi
else
    echo "FAIL: list_superadmins call failed"
    ((TESTS_FAILED++))
fi

# Note: is_admin is not exposed in the Candid interface, so we skip this test

# Summary
echo "=== Admin Functions Test Summary ==="
echo "Tests passed: $TESTS_PASSED"
echo "Tests failed: $TESTS_FAILED"
echo "Total tests: $((TESTS_PASSED + TESTS_FAILED))"

if [ $TESTS_FAILED -eq 0 ]; then
    echo "SUCCESS: All admin function tests passed!"
    exit 0
else
    echo "ERROR: Some admin function tests failed!"
    exit 1
fi
