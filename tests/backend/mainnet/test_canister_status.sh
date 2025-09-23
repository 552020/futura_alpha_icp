#!/bin/bash

# Mainnet Canister Status Test
# Tests detailed canister status information from mainnet deployment
#
# DFX Methods Used:
# - dfx canister status backend --network ic

# Load test utilities and mainnet configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_utils.sh"
source "$SCRIPT_DIR/config.sh"

# Test configuration
TEST_NAME="Mainnet Canister Status Tests"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Helper function to run a test with proper counting
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

# Helper function to extract field from status output
extract_status_field() {
    local status_output="$1"
    local field_name="$2"
    echo "$status_output" | grep "^$field_name:" | sed "s/^$field_name: *//"
}

# Helper function to extract numeric value from field
extract_numeric_value() {
    local field_value="$1"
    echo "$field_value" | grep -o '[0-9,]*' | tr -d ','
}

# Test 1: Get detailed canister status
test_get_detailed_status() {
    echo_info "Testing detailed canister status retrieval..."
    
    local status_output=$(dfx canister status backend --network ic 2>/dev/null)
    
    if [[ $? -eq 0 && -n "$status_output" ]]; then
        echo_info "Canister status retrieved successfully"
        echo_info "Status output length: ${#status_output} characters"
        return 0
    else
        echo_error "Failed to retrieve canister status"
        return 1
    fi
}

# Test 2: Validate canister is running
test_canister_running() {
    echo_info "Testing canister running status..."
    
    local status_output=$(dfx canister status backend --network ic 2>/dev/null)
    local status=$(extract_status_field "$status_output" "Status")
    
    if [[ "$status" == "Running" ]]; then
        echo_info "Canister status: $status"
        return 0
    else
        echo_error "Canister is not running. Status: $status"
        return 1
    fi
}

# Test 3: Validate controller
test_controller_validation() {
    echo_info "Testing controller validation..."
    
    local status_output=$(dfx canister status backend --network ic 2>/dev/null)
    local controllers=$(extract_status_field "$status_output" "Controllers")
    
    if [[ -n "$controllers" ]]; then
        echo_info "Controllers: $controllers"
        
        # Check if current user is a controller
        local current_principal=$(dfx identity get-principal 2>/dev/null)
        if echo "$controllers" | grep -q "$current_principal"; then
            echo_info "Current user is a controller: $current_principal"
            return 0
        else
            echo_warn "Current user is not a controller"
            return 1
        fi
    else
        echo_error "No controllers found in status"
        return 1
    fi
}

# Test 4: Check memory allocation
test_memory_allocation() {
    echo_info "Testing memory allocation..."
    
    local status_output=$(dfx canister status backend --network ic 2>/dev/null)
    local memory_allocation=$(extract_status_field "$status_output" "Memory allocation")
    
    if [[ -n "$memory_allocation" ]]; then
        echo_info "Memory allocation: $memory_allocation"
        
        # Check if memory allocation is reasonable (not 0, not too high)
        if [[ "$memory_allocation" == "0 Bytes" ]]; then
            echo_info "Memory allocation is 0 (unlimited) - this is normal"
            return 0
        else
            echo_info "Memory allocation is set to: $memory_allocation"
            return 0
        fi
    else
        echo_error "Memory allocation not found in status"
        return 1
    fi
}

# Test 5: Check compute allocation
test_compute_allocation() {
    echo_info "Testing compute allocation..."
    
    local status_output=$(dfx canister status backend --network ic 2>/dev/null)
    local compute_allocation=$(extract_status_field "$status_output" "Compute allocation")
    
    if [[ -n "$compute_allocation" ]]; then
        echo_info "Compute allocation: $compute_allocation"
        
        # Check if compute allocation is reasonable
        if [[ "$compute_allocation" == "0 %" ]]; then
            echo_info "Compute allocation is 0% (unlimited) - this is normal"
            return 0
        else
            echo_info "Compute allocation is set to: $compute_allocation"
            return 0
        fi
    else
        echo_error "Compute allocation not found in status"
        return 1
    fi
}

# Test 6: Check cycles balance
test_cycles_balance() {
    echo_info "Testing cycles balance..."
    
    local status_output=$(dfx canister status backend --network ic 2>/dev/null)
    local balance=$(extract_status_field "$status_output" "Balance")
    
    if [[ -n "$balance" ]]; then
        echo_info "Cycles balance: $balance"
        
        # Extract numeric value and check if it's reasonable
        local balance_numeric=$(extract_numeric_value "$balance")
        if [[ $balance_numeric -gt 1000000000 ]]; then  # More than 1B cycles
            echo_info "Cycles balance is healthy: $balance_numeric cycles"
            return 0
        else
            echo_warn "Cycles balance is low: $balance_numeric cycles"
            return 1
        fi
    else
        echo_error "Cycles balance not found in status"
        return 1
    fi
}

# Test 7: Check memory size
test_memory_size() {
    echo_info "Testing memory size..."
    
    local status_output=$(dfx canister status backend --network ic 2>/dev/null)
    local memory_size=$(extract_status_field "$status_output" "Memory Size")
    
    if [[ -n "$memory_size" ]]; then
        echo_info "Memory size: $memory_size"
        
        # Extract numeric value and check if it's reasonable
        local memory_numeric=$(extract_numeric_value "$memory_size")
        if [[ $memory_numeric -gt 0 ]]; then
            echo_info "Memory size is reasonable: $memory_numeric bytes"
            return 0
        else
            echo_error "Memory size is 0 or invalid"
            return 1
        fi
    else
        echo_error "Memory size not found in status"
        return 1
    fi
}

# Test 8: Check freezing threshold
test_freezing_threshold() {
    echo_info "Testing freezing threshold..."
    
    local status_output=$(dfx canister status backend --network ic 2>/dev/null)
    local freezing_threshold=$(extract_status_field "$status_output" "Freezing threshold")
    
    if [[ -n "$freezing_threshold" ]]; then
        echo_info "Freezing threshold: $freezing_threshold"
        
        # Check if freezing threshold is reasonable (should be > 0)
        local threshold_numeric=$(extract_numeric_value "$freezing_threshold")
        if [[ $threshold_numeric -gt 0 ]]; then
            echo_info "Freezing threshold is set: $threshold_numeric seconds"
            return 0
        else
            echo_error "Freezing threshold is 0 or invalid"
            return 1
        fi
    else
        echo_error "Freezing threshold not found in status"
        return 1
    fi
}

# Test 9: Check module hash
test_module_hash() {
    echo_info "Testing module hash..."
    
    local status_output=$(dfx canister status backend --network ic 2>/dev/null)
    local module_hash=$(extract_status_field "$status_output" "Module hash")
    
    if [[ -n "$module_hash" ]]; then
        echo_info "Module hash: $module_hash"
        
        # Check if module hash is valid (should be 64 hex characters)
        if echo "$module_hash" | grep -qE "^0x[0-9a-f]{64}$"; then
            echo_info "Module hash format is valid"
            return 0
        else
            echo_error "Module hash format is invalid"
            return 1
        fi
    else
        echo_error "Module hash not found in status"
        return 1
    fi
}

# Test 10: Check cycles consumption rate
test_cycles_consumption() {
    echo_info "Testing cycles consumption rate..."
    
    local status_output=$(dfx canister status backend --network ic 2>/dev/null)
    local idle_cycles=$(extract_status_field "$status_output" "Idle cycles burned per day")
    
    if [[ -n "$idle_cycles" ]]; then
        echo_info "Idle cycles burned per day: $idle_cycles"
        
        # Extract numeric value and check if it's reasonable
        local cycles_numeric=$(extract_numeric_value "$idle_cycles")
        if [[ $cycles_numeric -gt 0 ]]; then
            echo_info "Cycles consumption rate is: $cycles_numeric cycles/day"
            return 0
        else
            echo_error "Cycles consumption rate is 0 or invalid"
            return 1
        fi
    else
        echo_error "Idle cycles burned per day not found in status"
        return 1
    fi
}

# Test 11: Save status to file for inspection
test_save_status_to_file() {
    echo_info "Testing saving status to file..."
    
    local status_output=$(dfx canister status backend --network ic 2>/dev/null)
    local output_file="/tmp/mainnet_canister_status.txt"
    
    if [[ $? -eq 0 && -n "$status_output" ]]; then
        echo "$status_output" > "$output_file"
        
        if [[ -f "$output_file" && -s "$output_file" ]]; then
            local file_size=$(wc -c < "$output_file")
            echo_info "Canister status saved to: $output_file"
            echo_info "File size: $file_size bytes"
            return 0
        else
            echo_error "Failed to save canister status to file"
            return 1
        fi
    else
        echo_error "Failed to retrieve canister status for file saving"
        return 1
    fi
}

# Main test execution
main() {
    echo_info "Starting $TEST_NAME"
    echo_info "=================================="
    
    # Validate mainnet configuration
    if ! validate_mainnet_config; then
        echo_error "Mainnet configuration validation failed"
        exit 1
    fi
    
    echo_info "Network: ICP Mainnet"
    echo ""
    
    # Check prerequisites
    if ! command -v dfx &> /dev/null; then
        echo_error "dfx command not found. Please install dfx first."
        exit 1
    fi
    
    # Run tests
    run_test "Get detailed canister status" "test_get_detailed_status"
    run_test "Validate canister is running" "test_canister_running"
    run_test "Validate controller" "test_controller_validation"
    run_test "Check memory allocation" "test_memory_allocation"
    run_test "Check compute allocation" "test_compute_allocation"
    run_test "Check cycles balance" "test_cycles_balance"
    run_test "Check memory size" "test_memory_size"
    run_test "Check freezing threshold" "test_freezing_threshold"
    run_test "Check module hash" "test_module_hash"
    run_test "Check cycles consumption rate" "test_cycles_consumption"
    run_test "Save status to file" "test_save_status_to_file"
    
    # Print test summary
    echo_info "=================================="
    echo_info "Test Summary:"
    echo_info "Total tests: $TOTAL_TESTS"
    echo_info "Passed: $PASSED_TESTS"
    echo_info "Failed: $FAILED_TESTS"
    echo ""
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo_pass "All canister status tests passed! 🎉"
        echo_info "Mainnet canister status is healthy and properly configured."
        echo_info "Status saved to: /tmp/mainnet_canister_status.txt"
        exit 0
    else
        echo_fail "Some canister status tests failed!"
        echo_info "Please check the mainnet canister configuration."
        exit 1
    fi
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
