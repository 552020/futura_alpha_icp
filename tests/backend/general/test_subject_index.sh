#!/bin/bash

# Test script for subject index functionality in stable backend
# This tests the functionality that was previously only testable in IC environment

# Color issues are handled in test_utils.sh

# Parse command line arguments
MAINNET_MODE=false
NETWORK_FLAG=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --mainnet)
            MAINNET_MODE=true
            NETWORK_FLAG="--network ic"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--mainnet]"
            exit 1
            ;;
    esac
done

# Load test configuration and utilities first
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_config.sh"
source "$SCRIPT_DIR/../test_utils.sh"

# Set canister ID based on mode
if [[ "$MAINNET_MODE" == "true" ]]; then
    CANISTER_ID=$(get_canister_id "backend" "ic")
else
    CANISTER_ID=$(get_canister_id "backend")
fi

# Test configuration
TEST_NAME="Subject Index Tests"
if [[ "$MAINNET_MODE" == "true" ]]; then
    TEST_NAME="Subject Index Tests (Mainnet)"
fi
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Switch to superadmin identity for testing (only for local mode)
if [[ "$MAINNET_MODE" == "false" ]]; then
    dfx identity use 552020
fi

# Test function for subject index functionality
test_subject_index_functionality() {
    echo_info "Testing subject index functionality..."
    
    # Test 1: Create a capsule (this will be the subject)
    echo_info "Creating test capsule..."
    local create_result=$(dfx canister call "$CANISTER_ID" capsules_create "(null)" $NETWORK_FLAG 2>/dev/null)
    if is_success "$create_result"; then
        echo_info "Capsule created successfully"
        
        # Extract capsule ID from the result
        local capsule_id=$(extract_capsule_id "$create_result")
        echo_info "Created capsule ID: $capsule_id"
        
        # Test 2: Read the capsule back (this tests the subject index indirectly)
        echo_info "Reading capsule back..."
        local read_result=$(dfx canister call "$CANISTER_ID" capsules_read_full "(opt \"$capsule_id\")" $NETWORK_FLAG 2>/dev/null)
        if is_success "$read_result" && echo "$read_result" | grep -q "id.*$capsule_id"; then
            echo_info "Capsule read successfully"
        else
            echo_error "Failed to read capsule"
            return 1
        fi
        
        # Test 3: List capsules (this uses subject index to find caller's capsules)
        echo_info "Listing capsules..."
        local list_result=$(dfx canister call "$CANISTER_ID" capsules_list $NETWORK_FLAG 2>/dev/null)
        if echo "$list_result" | grep -q "id.*$capsule_id"; then
            echo_info "Capsule found in list (subject index working)"
        else
            echo_error "Capsule not found in list (subject index may be broken)"
            return 1
        fi
        
        # Test 4: Read basic capsule info
        echo_info "Reading basic capsule info..."
        local basic_result=$(dfx canister call "$CANISTER_ID" capsules_read_basic "(opt \"$capsule_id\")" $NETWORK_FLAG 2>/dev/null)
        if is_success "$basic_result" && echo "$basic_result" | grep -q "capsule_id.*$capsule_id"; then
            echo_info "Basic capsule info read successfully"
        else
            echo_error "Failed to read basic capsule info"
            return 1
        fi
        
        # Test 5: Test self-capsule access (None parameter should return caller's self-capsule)
        echo_info "Testing self-capsule access..."
        local self_result=$(dfx canister call "$CANISTER_ID" capsules_read_full "(null)" $NETWORK_FLAG 2>/dev/null)
        if is_success "$self_result" && echo "$self_result" | grep -q "id.*$capsule_id"; then
            echo_info "Self-capsule access working (subject index working)"
        else
            echo_error "Self-capsule access failed"
            return 1
        fi
        
        return 0
    else
        echo_error "Failed to create capsule: $create_result"
        return 1
    fi
}

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
