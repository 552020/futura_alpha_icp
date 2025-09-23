#!/bin/bash

# Mainnet Canister Info Test
# Tests public canister information using dfx canister info (no authentication required)
#
# DFX Methods Used:
# - dfx canister info <canister_id> --network ic
# - dfx canister status backend --network ic (for comparison)

# Load test utilities and mainnet configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_utils.sh"
source "$SCRIPT_DIR/config.sh"

# Test configuration
TEST_NAME="Mainnet Canister Info Tests"
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

# Helper function to extract field from info output
extract_info_field() {
    local info_output="$1"
    local field_name="$2"
    echo "$info_output" | grep "^$field_name:" | sed "s/^$field_name: *//"
}

# Test 1: Get public canister info
test_get_public_info() {
    echo_info "Testing public canister info retrieval..."
    
    local info_output=$(dfx canister info "$MAINNET_CANISTER_ID" --network "$MAINNET_NETWORK" 2>/dev/null)
    
    if [[ $? -eq 0 && -n "$info_output" ]]; then
        echo_info "Public canister info retrieved successfully"
        echo_info "Info output length: ${#info_output} characters"
        return 0
    else
        echo_error "Failed to retrieve public canister info"
        return 1
    fi
}

# Test 2: Validate module hash
test_module_hash_validation() {
    echo_info "Testing module hash validation..."
    
    local info_output=$(dfx canister info "$MAINNET_CANISTER_ID" --network "$MAINNET_NETWORK" 2>/dev/null)
    local module_hash=$(extract_info_field "$info_output" "Module hash")
    
    if [[ -n "$module_hash" ]]; then
        echo_info "Module hash: $module_hash"
        
        # Check if module hash is valid (should be 64 hex characters after 0x)
        if echo "$module_hash" | grep -qE "^0x[0-9a-f]{64}$"; then
            echo_info "Module hash format is valid"
            return 0
        else
            echo_error "Module hash format is invalid"
            return 1
        fi
    else
        echo_error "Module hash not found in public info"
        return 1
    fi
}

# Test 3: Display controller information
test_display_controllers() {
    echo_info "Displaying controller information..."
    
    local info_output=$(dfx canister info "$MAINNET_CANISTER_ID" --network "$MAINNET_NETWORK" 2>/dev/null)
    local controllers=$(extract_info_field "$info_output" "Controllers")
    
    if [[ -n "$controllers" ]]; then
        echo_info "Controllers: $controllers"
        return 0
    else
        echo_error "Controllers not found in public info"
        return 1
    fi
}

# Test 4: Test public accessibility
test_public_accessibility() {
    echo_info "Testing public accessibility (no authentication required)..."
    
    # This test verifies that dfx canister info works without being a controller
    local info_output=$(dfx canister info "$MAINNET_CANISTER_ID" --network "$MAINNET_NETWORK" 2>/dev/null)
    
    if [[ $? -eq 0 && -n "$info_output" ]]; then
        echo_info "Public canister info is accessible without authentication"
        return 0
    else
        echo_error "Public canister info is not accessible"
        return 1
    fi
}

# Test 5: Save info to file for inspection
test_save_info_to_file() {
    echo_info "Testing saving public info to file..."
    
    local info_output=$(dfx canister info "$MAINNET_CANISTER_ID" --network "$MAINNET_NETWORK" 2>/dev/null)
    local output_file="/tmp/mainnet_canister_info.txt"
    
    if [[ $? -eq 0 && -n "$info_output" ]]; then
        echo "$info_output" > "$output_file"
        
        if [[ -f "$output_file" && -s "$output_file" ]]; then
            local file_size=$(wc -c < "$output_file")
            echo_info "Public canister info saved to: $output_file"
            echo_info "File size: $file_size bytes"
            return 0
        else
            echo_error "Failed to save public canister info to file"
            return 1
        fi
    else
        echo_error "Failed to retrieve public canister info for file saving"
        return 1
    fi
}

# Test 6: Compare module hash with status
test_module_hash_consistency() {
    echo_info "Testing module hash consistency between info and status..."
    
    local info_output=$(dfx canister info "$MAINNET_CANISTER_ID" --network "$MAINNET_NETWORK" 2>/dev/null)
    local status_output=$(dfx canister status "$MAINNET_CANISTER_NAME" --network "$MAINNET_NETWORK" 2>/dev/null)
    
    if [[ $? -eq 0 && -n "$info_output" && -n "$status_output" ]]; then
        local info_hash=$(extract_info_field "$info_output" "Module hash")
        local status_hash=$(echo "$status_output" | grep "^Module hash:" | sed 's/^Module hash: *//')
        
        if [[ "$info_hash" == "$status_hash" ]]; then
            echo_info "Module hash is consistent between info and status"
            echo_info "Hash: $info_hash"
            return 0
        else
            echo_error "Module hash mismatch between info and status"
            echo_info "Info hash: $info_hash"
            echo_info "Status hash: $status_hash"
            return 1
        fi
    else
        echo_error "Cannot compare module hashes - missing data"
        return 1
    fi
}

# Test 7: Validate info output structure
test_info_output_structure() {
    echo_info "Testing public info output structure..."
    
    local info_output=$(dfx canister info "$MAINNET_CANISTER_ID" --network "$MAINNET_NETWORK" 2>/dev/null)
    
    if [[ $? -eq 0 && -n "$info_output" ]]; then
        # Check for expected fields
        local expected_fields=("Module hash" "Controllers")
        local missing_fields=()
        
        for field in "${expected_fields[@]}"; do
            if ! echo "$info_output" | grep -q "^$field:"; then
                missing_fields+=("$field")
            fi
        done
        
        if [[ ${#missing_fields[@]} -eq 0 ]]; then
            echo_info "All expected fields present in public info"
            return 0
        else
            echo_error "Missing expected fields: ${missing_fields[*]}"
            return 1
        fi
    else
        echo_error "Cannot validate info structure - missing data"
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
    echo_info "Testing public canister info (no authentication required)"
    echo ""
    
    # Check prerequisites
    if ! command -v dfx &> /dev/null; then
        echo_error "dfx command not found. Please install dfx first."
        exit 1
    fi
    
    # Run tests
    run_test "Get public canister info" "test_get_public_info"
    run_test "Validate module hash" "test_module_hash_validation"
    run_test "Display controller information" "test_display_controllers"
    run_test "Test public accessibility" "test_public_accessibility"
    run_test "Save info to file" "test_save_info_to_file"
    run_test "Compare module hash with status" "test_module_hash_consistency"
    run_test "Validate info output structure" "test_info_output_structure"
    
    # Print test summary
    echo_info "=================================="
    echo_info "Test Summary:"
    echo_info "Total tests: $TOTAL_TESTS"
    echo_info "Passed: $PASSED_TESTS"
    echo_info "Failed: $FAILED_TESTS"
    echo ""
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo_pass "All public canister info tests passed! 🎉"
        echo_info "Mainnet canister public information is accessible and valid."
        echo_info "Info saved to: /tmp/mainnet_canister_info.txt"
        exit 0
    else
        echo_fail "Some public canister info tests failed!"
        echo_info "Please check the mainnet canister public information."
        exit 1
    fi
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
