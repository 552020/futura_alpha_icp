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

# Helper function to create a test memory and return its ID
# Creates a memory in the specified capsule for testing purposes
get_test_memory_id() {
    local capsule_id="$1"
    local canister_id="${2:-backend}"
    local identity="${3:-default}"
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Capsule ID is required for get_test_memory_id"
        return 1
    fi
    
    # Create test memory data
    local memory_data='(record {
      blob_ref = record {
        kind = variant { ICPCapsule };
        locator = "test_memory_'"$(date +%s)"'";
        hash = null;
      };
      data = opt blob "VGVzdCBtZW1vcnkgZGF0YQ==";
    })'
    
    # Create the memory
    local create_result=$(dfx canister call --identity "$identity" "$canister_id" memories_create "(\"$capsule_id\", $memory_data)" 2>/dev/null)
    local memory_id=$(echo "$create_result" | grep -o 'memory_id = opt "[^"]*"' | sed 's/memory_id = opt "//' | sed 's/"//')
    
    if [[ -z "$memory_id" ]]; then
        echo_error "Failed to create test memory"
        echo_debug "Create result: $create_result"
        return 1
    fi
    
    echo "$memory_id"
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