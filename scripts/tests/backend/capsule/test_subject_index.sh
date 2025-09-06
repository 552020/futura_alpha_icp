#!/bin/bash
# Test script for subject index functionality in stable backend
# This tests the functionality that was previously only testable in IC environment

set -e

# Configuration
CANISTER_ID="uxrrr-q7777-77774-qaaaq-cai" # Replace with your canister ID
SUPERADMIN_PRINCIPAL="otzfv-jscof-niinw-gtloq-25uz3-pglpg-u3kug-besf3-rzlbd-ylrmp-5ae"

echo "Testing subject index functionality..."
echo "Using canister: $CANISTER_ID"

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Helper function to run tests
test_function() {
    local description="$1"
    local command="$2"
    
    echo "Testing: $description"
    if eval "$command" >/dev/null 2>&1; then
        echo "PASS: $description"
        ((TESTS_PASSED++))
    else
        echo "FAIL: $description"
        ((TESTS_FAILED++))
    fi
}

# Switch to superadmin identity for testing
dfx identity use 552020

# Test 1: Create a capsule (this will be the subject)
echo "Creating test capsule..."
CREATE_RESULT=$(dfx canister call "$CANISTER_ID" capsules_create "(null)")
if echo "$CREATE_RESULT" | grep -q "success.*true"; then
    echo "PASS: Capsule created successfully"
    ((TESTS_PASSED++))
    
    # Extract capsule ID from the result
    CAPSULE_ID=$(echo "$CREATE_RESULT" | grep -o 'capsule_[0-9]*' | tail -1)
    echo "Created capsule ID: $CAPSULE_ID"
else
    echo "FAIL: Failed to create capsule"
    ((TESTS_FAILED++))
    exit 1
fi

# Test 2: Read the capsule back (this tests the subject index indirectly)
echo "Reading capsule back..."
READ_RESULT=$(dfx canister call "$CANISTER_ID" capsules_read_full "(opt \"$CAPSULE_ID\")")
if echo "$READ_RESULT" | grep -q "id.*$CAPSULE_ID"; then
    echo "PASS: Capsule read successfully"
    ((TESTS_PASSED++))
else
    echo "FAIL: Failed to read capsule"
    ((TESTS_FAILED++))
fi

# Test 3: List capsules (this uses subject index to find caller's capsules)
echo "Listing capsules..."
LIST_RESULT=$(dfx canister call "$CANISTER_ID" capsules_list "()")
if echo "$LIST_RESULT" | grep -q "id.*$CAPSULE_ID"; then
    echo "PASS: Capsule found in list (subject index working)"
    ((TESTS_PASSED++))
else
    echo "FAIL: Capsule not found in list (subject index may be broken)"
    ((TESTS_FAILED++))
fi

# Test 4: Read basic capsule info
echo "Reading basic capsule info..."
BASIC_RESULT=$(dfx canister call "$CANISTER_ID" capsules_read_basic "(opt \"$CAPSULE_ID\")")
if echo "$BASIC_RESULT" | grep -q "capsule_id.*$CAPSULE_ID"; then
    echo "PASS: Basic capsule info read successfully"
    ((TESTS_PASSED++))
else
    echo "FAIL: Failed to read basic capsule info"
    ((TESTS_FAILED++))
fi

# Test 5: Test self-capsule access (None parameter should return caller's self-capsule)
echo "Testing self-capsule access..."
SELF_RESULT=$(dfx canister call "$CANISTER_ID" capsules_read_full "(null)")
if echo "$SELF_RESULT" | grep -q "id.*$CAPSULE_ID"; then
    echo "PASS: Self-capsule access working (subject index working)"
    ((TESTS_PASSED++))
else
    echo "FAIL: Self-capsule access failed"
    ((TESTS_FAILED++))
fi

# Summary
echo "=== Subject Index Test Summary ==="
echo "Tests passed: $TESTS_PASSED"
echo "Tests failed: $TESTS_FAILED"
echo "Total tests: $((TESTS_PASSED + TESTS_FAILED))"

if [ $TESTS_FAILED -eq 0 ]; then
    echo "SUCCESS: All subject index tests passed!"
    echo "✅ Stable backend subject index is working correctly"
    exit 0
else
    echo "ERROR: Some subject index tests failed!"
    echo "❌ Stable backend subject index may have issues"
    exit 1
fi
