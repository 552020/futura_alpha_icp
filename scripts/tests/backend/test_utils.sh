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

# Helper function to run tests with counters
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