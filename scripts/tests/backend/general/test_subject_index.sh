#!/bin/bash

# Test script for subject index functionality in stable backend
# This tests the functionality that was previously only testable in IC environment

# Load test configuration and utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_config.sh"
source "$SCRIPT_DIR/../test_utils.sh"
source "$SCRIPT_DIR/capsule_test_utils.sh"

# Test configuration
TEST_NAME="Subject Index Tests"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

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

# Main test execution
main() {
    echo_info "Starting $TEST_NAME"
    echo_info "=================================="
    
    # Setup
    setup_user_and_capsule
    
    # Run tests
    run_capsule_test "Subject index functionality" test_subject_index_functionality
    
    # Test summary
    print_test_summary
}

# Run main function
main "$@"
