#!/usr/bin/env bash
set -euo pipefail

# Load test configuration and utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../test_config.sh"
source "$SCRIPT_DIR/../../test_utils.sh"
source "$SCRIPT_DIR/upload_test_utils.sh"

# Test configuration
TEST_NAME="Upload Begin Tests"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

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

# Test functions
test_upload_begin_success() {
    local capsule_id=$(get_test_capsule_id)
    local session_id=$(begin_upload_session "$capsule_id" 4 "idem-1")
    
    if [[ -n "$session_id" ]]; then
        echo_info "Upload session created successfully: $session_id"
        export TEST_SESSION_ID="$session_id"
        return 0
    else
        echo_info "Upload session creation failed"
        return 1
    fi
}

test_upload_begin_idempotency() {
    local capsule_id=$(get_test_capsule_id)
    local session_id1=$(begin_upload_session "$capsule_id" 4 "idem-1")
    local session_id2=$(begin_upload_session "$capsule_id" 4 "idem-1")
    
    if [[ "$session_id1" = "$session_id2" ]] && [[ -n "$session_id1" ]]; then
        echo_info "Upload begin idempotency working correctly: same session ID returned"
        return 0
    else
        echo_info "Upload begin idempotency test failed. Session1: $session_id1, Session2: $session_id2"
        return 1
    fi
}

test_upload_begin_zero_chunks() {
    local capsule_id=$(get_test_capsule_id)
    local result=$(dfx canister call backend uploads_begin \
        '("'$capsule_id'", record { name = "test"; description = opt "test"; tags = vec {} }, 0, "idem-zero")' 2>/dev/null)
    
    if is_failure "$result" && echo "$result" | grep -q "expected_chunks_zero"; then
        echo_info "Upload begin correctly rejected zero chunks: $result"
        return 0
    else
        echo_info "Upload begin validation test result: $result"
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
    
    # Run all tests in order
    run_test "Upload begin (success)" "test_upload_begin_success"
    run_test "Upload begin (idempotency)" "test_upload_begin_idempotency"
    run_test "Upload begin (zero chunks validation)" "test_upload_begin_zero_chunks"
    
    # Print test summary
    print_test_summary "$TEST_NAME" "$TOTAL_TESTS" "$PASSED_TESTS" "$FAILED_TESTS"
    
    if [ $FAILED_TESTS -eq 0 ]; then
        exit 0
    else
        exit 1
    fi
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
