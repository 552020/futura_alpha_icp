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
    
    if [[ $capsule_result == *"null"* ]]; then
        echo_debug "No capsule found, creating one first..."
        local create_result=$(dfx canister call --identity "$identity" "$canister_id" capsules_create "(null)" 2>/dev/null)
        capsule_id=$(echo "$create_result" | grep -o 'capsule_id = opt "[^"]*"' | sed 's/capsule_id = opt "//' | sed 's/"//')
    else
        capsule_id=$(echo "$capsule_result" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')
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

# Response parsing utilities
is_success() {
    local response="$1"
    echo "$response" | grep -q "success = true"
}

is_failure() {
    local response="$1"
    echo "$response" | grep -q "success = false"
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