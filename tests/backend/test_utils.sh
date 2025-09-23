#!/bin/bash

# Basic test utilities - minimal setup

# Simple logging
echo_pass() {
    echo "[PASS] $1"
}

echo_fail() {
    echo "[FAIL] $1"
}

echo_info() {
    echo "[INFO] $1"
}

echo_warn() {
    echo "[WARN] $1"
}

# Additional logging functions for comprehensive test scripts
echo_success() {
    echo "[SUCCESS] $1"
}

echo_error() {
    echo "[ERROR] $1"
}

# Helper function to get a valid test principal
get_test_principal() {
    local identity_name="${1:-test-admin}"
    
    # Try to get principal from existing identity
    if dfx identity get-principal --identity "$identity_name" >/dev/null 2>&1; then
        dfx identity get-principal --identity "$identity_name" 2>/dev/null
        return 0
    fi
    
    # If identity doesn't exist, create it (suppress output)
    if dfx identity new "$identity_name" --storage-mode plaintext >/dev/null 2>&1; then
        dfx identity get-principal --identity "$identity_name" 2>/dev/null
        return 0
    fi
    
    # Fallback to a known valid principal
    echo "ur7ny-sza5i-m73am-naljv-rgxjo-bzm2w-k7q6l-p2qsc-b2mce-qlpsr-uae"
    return 0
}

# Helper function to get multiple test principals
get_test_principals() {
    local admin1=$(get_test_principal "test-admin-1")
    local admin2=$(get_test_principal "test-admin-2")
    
    echo "$admin1"
    echo "$admin2"
}

# Helper function to check if current identity is a superadmin
check_superadmin_status() {
    local canister_id="$1"
    local network_flag="$2"
    
    # Get current identity
    local current_identity=$(dfx identity get-principal $network_flag 2>/dev/null)
    echo_info "Current identity: $current_identity" >&2
    
    # Check if current identity is in superadmin list
    if dfx canister call "$canister_id" list_superadmins --query $network_flag 2>/dev/null | grep -q "$current_identity"; then
        echo_info "Current identity is a superadmin" >&2
        echo "true"
    else
        echo_info "Current identity is NOT a superadmin" >&2
        echo "false"
    fi
}

# Helper function to run a test with expected result
run_test_with_expected() {
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
            return 0
        else
            echo_fail "$test_name - Expected failure but got success"
            echo_error "Result: $result"
            return 1
        fi
    else
        if [ "$expected_result" = "failure" ]; then
            echo_pass "$test_name"
            return 0
        else
            echo_fail "$test_name - Expected success but got failure"
            echo_error "Result: $result"
            return 1
        fi
    fi
}

# Helper function to run a test and update counters
run_test_with_counters() {
    local test_name="$1"
    local command="$2"
    local expected_result="$3"
    local tests_passed_var="$4"
    local tests_failed_var="$5"
    
    if run_test_with_expected "$test_name" "$command" "$expected_result"; then
        eval "((++$tests_passed_var))"
    else
        eval "((++$tests_failed_var))"
    fi
}

echo_debug() {
    echo "[DEBUG] $1"
}

echo_header() {
    echo ""
    echo "=========================================="
    echo "$1"
    echo "=========================================="
    echo ""
}

# Helper function to call dfx with timeout
dfx_call_with_timeout() {
    local timeout=${TEST_TIMEOUT:-30}
    timeout "$timeout" dfx "$@"
}

# Helper function to extract values from Candid responses
extract_candid_value() {
    local response="$1"
    local field="$2"
    echo "$response" | grep -o "${field} = [^;]*" | sed "s/${field} = //"
}

# Helper function to check if canister is running
check_canister_status() {
    local canister_id="$1"
    dfx canister status "$canister_id" >/dev/null 2>&1
}

# Helper function to get a capsule ID for testing
# Creates a new capsule if none exist, otherwise returns the first available one
get_test_capsule_id() {
    local canister_id="${1:-backend}"
    local identity="${2:-default}"
    
    local capsule_result=$(dfx canister call --identity "$identity" "$canister_id" capsules_read_basic "(null)" 2>/dev/null)
    local capsule_id=""
    
    if [[ $capsule_result == *"null"* ]] || [[ $capsule_result == *"NotFound"* ]]; then
        echo_debug "No capsule found, creating one first..."
        local create_result=$(dfx canister call --identity "$identity" "$canister_id" capsules_create "(null)" 2>/dev/null)
        if is_success "$create_result"; then
            # Extract capsule ID from the new Result<Capsule> format
            capsule_id=$(echo "$create_result" | grep -o 'id = "[^"]*"' | sed 's/id = "//' | sed 's/"//')
        fi
    else
        if is_success "$capsule_result"; then
            capsule_id=$(echo "$capsule_result" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')
        fi
    fi
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to get capsule ID for testing"
        return 1
    fi
    
    echo "$capsule_id"
}

# Helper function to run tests with counters (simple version)
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo_info "Running: $test_name"
    
    if eval "$test_command"; then
        echo_success "$test_name passed"
        return 0
    else
        echo_error "$test_name failed"
        return 1
    fi
}

# Response parsing utilities (unified Result<T, Error> format)
is_success() {
    local response="$1"
    echo "$response" | grep -q "variant {" && (echo "$response" | grep -q "Ok =" || echo "$response" | grep -q "Ok }")
}

is_failure() {
    local response="$1"
    echo "$response" | grep -q "variant {" && echo "$response" | grep -q "Err ="
}

# Helper function to check if response is NotFound error
is_not_found() {
    local response="$1"
    echo "$response" | grep -q "Err = variant { NotFound }"
}

# Helper function to check if response is Unauthorized error
is_unauthorized() {
    local response="$1"
    echo "$response" | grep -q "Err = variant { Unauthorized }"
}

# Helper function to extract canister ID from response
extract_canister_id() {
    local response="$1"
    echo "$response" | grep -o 'canister_id = opt principal "[^"]*"' | sed 's/canister_id = opt principal "\([^"]*\)"/\1/'
}

# Helper function to extract creation status
extract_creation_status() {
    local response="$1"
    echo "$response" | grep -o 'status = variant { [^}]*}' | sed 's/status = variant { \([^}]*\) }/\1/'
}

# Helper function to extract principal from opt principal response
extract_principal() {
    local response="$1"
    echo "$response" | grep -o 'opt principal "[^"]*"' | sed 's/opt principal "\([^"]*\)"/\1/'
}

# Cycle monitoring utilities
get_cycles_balance() {
    local canister_id="$1"
    local network_flag="$2"
    
    if [[ "$network_flag" == "--network ic" ]]; then
        dfx canister status "$canister_id" --network ic 2>/dev/null | grep -o 'Balance: [0-9_,]*' | sed 's/Balance: //' | tr -d '_,'
    else
        dfx canister status "$canister_id" 2>/dev/null | grep -o 'Balance: [0-9_,]*' | sed 's/Balance: //' | tr -d '_,'
    fi
}

# Helper function to monitor cycle consumption
monitor_cycles() {
    local canister_id="$1"
    local network_flag="$2"
    local operation_name="$3"
    
    echo_info "Monitoring cycles for: $operation_name" >&2
    local initial_balance=$(get_cycles_balance "$canister_id" "$network_flag")
    echo_info "Initial cycles balance: $initial_balance" >&2
    
    # Return the initial balance for comparison later
    echo "$initial_balance"
}

# Helper function to calculate cycle consumption
calculate_cycle_consumption() {
    local initial_balance="$1"
    local canister_id="$2"
    local network_flag="$3"
    local operation_name="$4"
    
    local final_balance=$(get_cycles_balance "$canister_id" "$network_flag")
    local consumption=$((initial_balance - final_balance))
    
    echo_info "Final cycles balance: $final_balance" >&2
    echo_info "Cycle consumption for $operation_name: $consumption" >&2
    
    # Convert to approximate USD (rough estimate: 1 trillion cycles ≈ $1-2)
    if [[ $consumption -gt 0 ]]; then
        local usd_estimate=$(echo "scale=4; $consumption / 1000000000000" | bc 2>/dev/null || echo "N/A")
        echo_info "Estimated cost: ~$${usd_estimate} USD" >&2
    fi
    
    echo "$consumption"
}

# Helper function to warn about expensive operations
warn_expensive_operation() {
    local operation="$1"
    local estimated_cost="$2"
    
    echo_warn "⚠️  WARNING: $operation will cost cycles!"
    echo_warn "Estimated cost: ~\$${estimated_cost} USD"
    echo_warn "This operation will create/modify canisters on mainnet."
    echo ""
    echo_info "Press Ctrl+C to cancel, or wait 5 seconds to continue..."
    sleep 5
    echo_info "Proceeding with $operation..."
}

# ============================================================================
# CAPSULE-SPECIFIC UTILITIES
# ============================================================================

# Helper function to check if response contains capsule data
has_capsule_data() {
    local response="$1"
    echo "$response" | grep -q "Ok = record {"
}

# Helper function to extract capsule ID from response
extract_capsule_id() {
    local response="$1"
    # Try different patterns for capsule ID extraction
    local capsule_id=""
    
    # Pattern 1: id = "capsule_id"
    capsule_id=$(echo "$response" | grep -o 'id = "[^"]*"' | sed 's/id = "//' | sed 's/"//')
    
    # Pattern 2: capsule_id = "capsule_id" (for CapsuleInfo responses)
    if [[ -z "$capsule_id" ]]; then
        capsule_id=$(echo "$response" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')
    fi
    
    echo "$capsule_id"
}

# Helper function to check if response contains expected capsule fields
has_expected_capsule_fields() {
    local response="$1"
    # Check for common capsule fields
    echo "$response" | grep -q "id = " && \
    echo "$response" | grep -q "subject = " && \
    echo "$response" | grep -q "owners = "
}

# Helper function to check if response contains expected CapsuleInfo fields
has_expected_capsule_info_fields() {
    local response="$1"
    # Check for expected CapsuleInfo fields
    echo "$response" | grep -q "capsule_id =" && \
    echo "$response" | grep -q "subject =" && \
    echo "$response" | grep -q "is_owner =" && \
    echo "$response" | grep -q "is_controller =" && \
    echo "$response" | grep -q "is_self_capsule =" && \
    echo "$response" | grep -q "bound_to_neon =" && \
    echo "$response" | grep -q "created_at =" && \
    echo "$response" | grep -q "updated_at =" && \
    echo "$response" | grep -q "memory_count =" && \
    echo "$response" | grep -q "gallery_count =" && \
    echo "$response" | grep -q "connection_count ="
}

# Helper function to check if response contains expected CapsuleHeader fields
has_expected_capsule_header_fields() {
    local response="$1"
    # Check for expected CapsuleHeader fields
    echo "$response" | grep -q "id = " && \
    echo "$response" | grep -q "subject = " && \
    echo "$response" | grep -q "owner_count = " && \
    echo "$response" | grep -q "controller_count = " && \
    echo "$response" | grep -q "memory_count = " && \
    echo "$response" | grep -q "created_at = " && \
    echo "$response" | grep -q "updated_at ="
}

# Helper function to check if response is empty (no capsules)
is_empty_response() {
    local response="$1"
    echo "$response" | grep -q "vec {}"
}

# Helper function to check if response contains capsules
has_capsules() {
    local response="$1"
    echo "$response" | grep -q "record {"
}

# Helper function to create capsule update data
create_capsule_update_data() {
    local bound_to_neon="$1"
    cat << EOF
(record {
  bound_to_neon = opt $bound_to_neon;
})
EOF
}

# Helper function to setup user and capsule
setup_user_and_capsule() {
    echo_info "Setting up test user and capsule..."
    
    # Register user with nonce (new method)
    local nonce=$(date +%s)
    local register_result=$(dfx canister call backend register_with_nonce "(\"$nonce\")" 2>/dev/null)
    if ! is_success "$register_result"; then
        echo_warn "User registration failed, continuing with existing user..."
    fi
    
    # Ensure we have a capsule
    local capsule_id=$(get_test_capsule_id)
    if [[ -n "$capsule_id" ]]; then
        echo_info "Using existing capsule: $capsule_id"
    else
        echo_error "Failed to create or get capsule for testing"
        return 1
    fi
    
    echo_info "Setup complete"
}

# Helper function to run a test with proper counting (standardized)
run_capsule_test() {
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

# Helper function to print test summary (standardized)
print_test_summary() {
    echo_info "=================================="
    echo_info "Test Summary:"
    echo_info "Total tests: $TOTAL_TESTS"
    echo_info "Passed: $PASSED_TESTS"
    echo_info "Failed: $FAILED_TESTS"
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo_pass "All tests passed!"
        return 0
    else
        echo_fail "Some tests failed!"
        return 1
    fi
}