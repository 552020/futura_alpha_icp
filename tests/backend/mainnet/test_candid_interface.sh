#!/bin/bash

# Mainnet Candid Interface Test
# Tests that we can retrieve the Candid interface from the mainnet canister
#
# DFX Methods Used:
# - dfx canister call backend --network ic __get_candid_interface_tmp_hack()

# Load test utilities and mainnet configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_utils.sh"
source "$SCRIPT_DIR/config.sh"

# Test configuration
TEST_NAME="Mainnet Candid Interface Tests"
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

# Test 1: Get Candid interface from mainnet canister
test_get_candid_interface() {
    echo_info "Testing Candid interface retrieval from mainnet canister..."
    
    local result=$(dfx canister call backend --network ic __get_candid_interface_tmp_hack "()" 2>/dev/null)
    
    if [[ $? -eq 0 ]]; then
        if [[ -n "$result" ]]; then
            echo_info "Candid interface retrieved successfully"
            echo_info "Interface length: ${#result} characters"
            return 0
        else
            echo_error "Candid interface is empty"
            return 1
        fi
    else
        echo_error "Failed to retrieve Candid interface"
        return 1
    fi
}

# Test 2: Validate Candid interface format
test_candid_interface_format() {
    echo_info "Testing Candid interface format validation..."
    
    local result=$(dfx canister call backend --network ic __get_candid_interface_tmp_hack "()" 2>/dev/null)
    
    if [[ $? -eq 0 && -n "$result" ]]; then
        # Check for common Candid interface patterns
        if echo "$result" | grep -q "service" && echo "$result" | grep -q "func"; then
            echo_info "Candid interface has valid service and function definitions"
            return 0
        else
            echo_error "Candid interface missing expected service/func definitions"
            echo_debug "Interface content: $result"
            return 1
        fi
    else
        echo_error "Failed to retrieve Candid interface for format validation"
        return 1
    fi
}

# Test 3: Check for key functions in interface
test_key_functions_present() {
    echo_info "Testing for key functions in Candid interface..."
    
    local result=$(dfx canister call backend --network ic __get_candid_interface_tmp_hack "()" 2>/dev/null)
    
    if [[ $? -eq 0 && -n "$result" ]]; then
        local missing_functions=()
        
        # Check for essential functions
        if ! echo "$result" | grep -q "greet"; then
            missing_functions+=("greet")
        fi
        
        if ! echo "$result" | grep -q "whoami"; then
            missing_functions+=("whoami")
        fi
        
        if ! echo "$result" | grep -q "capsules_create"; then
            missing_functions+=("capsules_create")
        fi
        
        if ! echo "$result" | grep -q "memories_create"; then
            missing_functions+=("memories_create")
        fi
        
        if ! echo "$result" | grep -q "galleries_create"; then
            missing_functions+=("galleries_create")
        fi
        
        if [[ ${#missing_functions[@]} -eq 0 ]]; then
            echo_info "All key functions found in Candid interface"
            return 0
        else
            echo_error "Missing key functions: ${missing_functions[*]}"
            return 1
        fi
    else
        echo_error "Failed to retrieve Candid interface for function validation"
        return 1
    fi
}

# Test 4: Save interface to file for inspection
test_save_interface_to_file() {
    echo_info "Testing saving Candid interface to file..."
    
    local result=$(dfx canister call backend --network ic __get_candid_interface_tmp_hack "()" 2>/dev/null)
    local output_file="/tmp/mainnet_candid_interface.did"
    
    if [[ $? -eq 0 && -n "$result" ]]; then
        echo "$result" > "$output_file"
        
        if [[ -f "$output_file" && -s "$output_file" ]]; then
            local file_size=$(wc -c < "$output_file")
            echo_info "Candid interface saved to: $output_file"
            echo_info "File size: $file_size bytes"
            return 0
        else
            echo_error "Failed to save Candid interface to file"
            return 1
        fi
    else
        echo_error "Failed to retrieve Candid interface for file saving"
        return 1
    fi
}

# Test 5: Compare with local interface (if available)
test_compare_with_local_interface() {
    echo_info "Testing comparison with local Candid interface..."
    
    local mainnet_result=$(dfx canister call backend --network ic __get_candid_interface_tmp_hack "()" 2>/dev/null)
    local local_interface_file="src/backend/backend.did"
    
    if [[ $? -eq 0 && -n "$mainnet_result" ]]; then
        if [[ -f "$local_interface_file" ]]; then
            local local_content=$(cat "$local_interface_file")
            
            # Basic comparison - check if mainnet interface contains local interface functions
            local local_functions=$(echo "$local_content" | grep -o "func [a-zA-Z_][a-zA-Z0-9_]*" | wc -l)
            local mainnet_functions=$(echo "$mainnet_result" | grep -o "func [a-zA-Z_][a-zA-Z0-9_]*" | wc -l)
            
            echo_info "Local interface functions: $local_functions"
            echo_info "Mainnet interface functions: $mainnet_functions"
            
            if [[ $mainnet_functions -ge $local_functions ]]; then
                echo_info "Mainnet interface has equal or more functions than local"
                return 0
            else
                echo_warn "Mainnet interface has fewer functions than local"
                return 1
            fi
        else
            echo_warn "Local interface file not found: $local_interface_file"
            return 0  # Not a failure, just can't compare
        fi
    else
        echo_error "Failed to retrieve mainnet Candid interface for comparison"
        return 1
    fi
}

# Test 6: Validate interface syntax
test_interface_syntax() {
    echo_info "Testing Candid interface syntax validation..."
    
    local result=$(dfx canister call backend --network ic __get_candid_interface_tmp_hack "()" 2>/dev/null)
    
    if [[ $? -eq 0 && -n "$result" ]]; then
        # Basic syntax checks
        local syntax_errors=0
        
        # Check for balanced braces
        local open_braces=$(echo "$result" | tr -cd '{' | wc -c)
        local close_braces=$(echo "$result" | tr -cd '}' | wc -c)
        
        if [[ $open_braces -ne $close_braces ]]; then
            echo_error "Unbalanced braces in Candid interface"
            syntax_errors=$((syntax_errors + 1))
        fi
        
        # Check for balanced parentheses
        local open_parens=$(echo "$result" | tr -cd '(' | wc -c)
        local close_parens=$(echo "$result" | tr -cd ')' | wc -c)
        
        if [[ $open_parens -ne $close_parens ]]; then
            echo_error "Unbalanced parentheses in Candid interface"
            syntax_errors=$((syntax_errors + 1))
        fi
        
        # Check for service definition
        if ! echo "$result" | grep -q "service"; then
            echo_error "Missing service definition in Candid interface"
            syntax_errors=$((syntax_errors + 1))
        fi
        
        if [[ $syntax_errors -eq 0 ]]; then
            echo_info "Candid interface syntax appears valid"
            return 0
        else
            echo_error "Candid interface has $syntax_errors syntax errors"
            return 1
        fi
    else
        echo_error "Failed to retrieve Candid interface for syntax validation"
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
    run_test "Get Candid interface from mainnet" "test_get_candid_interface"
    run_test "Validate Candid interface format" "test_candid_interface_format"
    run_test "Check key functions present" "test_key_functions_present"
    run_test "Save interface to file" "test_save_interface_to_file"
    run_test "Compare with local interface" "test_compare_with_local_interface"
    run_test "Validate interface syntax" "test_interface_syntax"
    
    # Print test summary
    echo_info "=================================="
    echo_info "Test Summary:"
    echo_info "Total tests: $TOTAL_TESTS"
    echo_info "Passed: $PASSED_TESTS"
    echo_info "Failed: $FAILED_TESTS"
    echo ""
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo_pass "All Candid interface tests passed! 🎉"
        echo_info "Mainnet canister interface is accessible and valid."
        echo_info "Interface saved to: /tmp/mainnet_candid_interface.did"
        exit 0
    else
        echo_fail "Some Candid interface tests failed!"
        echo_info "Please check the mainnet canister interface."
        exit 1
    fi
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
