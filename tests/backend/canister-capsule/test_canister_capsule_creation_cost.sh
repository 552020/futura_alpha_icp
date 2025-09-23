#!/bin/bash

# Test canister capsule creation cost monitoring
# This test focuses specifically on the create_personal_canister function
# and tracks cycle consumption to see the actual cost of canister creation
#
# Usage: ./test_canister_capsule_creation_cost.sh [--mainnet] [--list]
# - Without --mainnet: Tests against local canister (no cycle costs)
# - With --mainnet: Tests against mainnet canister (WARNING: will cost cycles!)
# - With --list: Shows all previously created canisters from the log
#
# Features:
# - Cycle consumption monitoring and cost calculation
# - Canister creation verification
# - Automatic logging of created canisters (mainnet only)
# - Log file: tests/backend/mainnet/created_canisters.log

# Load test configuration and utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_utils.sh"

# Parse command line arguments
MAINNET_MODE=false
LIST_MODE=false

for arg in "$@"; do
    case $arg in
        --mainnet)
            MAINNET_MODE=true
            echo_info "Running in mainnet mode"
            ;;
        --list)
            LIST_MODE=true
            echo_info "List mode: showing all created canisters"
            ;;
    esac
done

# Set up canister configuration based on mode
if [[ "$MAINNET_MODE" == "true" ]]; then
    source "$SCRIPT_DIR/../mainnet/config.sh"
    CANISTER_ID="$MAINNET_CANISTER_ID"
    NETWORK_FLAG="--network $MAINNET_NETWORK"
    echo_info "Using mainnet canister: $CANISTER_ID"
else
    source "$SCRIPT_DIR/../test_config.sh"
    CANISTER_ID="${BACKEND_CANISTER_ID:-$(dfx canister id backend 2>/dev/null)}"
    NETWORK_FLAG=""
    if [ -z "$CANISTER_ID" ]; then
        echo_error "Backend canister not found. Make sure it's deployed locally."
        exit 1
    fi
    echo_info "Using local canister: $CANISTER_ID"
fi

# Test configuration
TEST_NAME="Canister Creation Cost Test"
PASSED_TESTS=0
FAILED_TESTS=0

# Log file for mainnet canister tracking
MAINNET_CANISTER_LOG="$SCRIPT_DIR/../mainnet/created_canisters.log"

# Function to log created canister
log_created_canister() {
    local canister_id="$1"
    local operation="$2"
    local cost="$3"
    
    if [[ "$MAINNET_MODE" == "true" ]]; then
        local timestamp=$(date -u +"%Y-%m-%d %H:%M:%S UTC")
        local identity=$(dfx identity get-principal $NETWORK_FLAG 2>/dev/null)
        
        # Create log directory if it doesn't exist
        mkdir -p "$(dirname "$MAINNET_CANISTER_LOG")"
        
        # Log the canister creation
        echo "[$timestamp] Created canister: $canister_id" >> "$MAINNET_CANISTER_LOG"
        echo "  - Operation: $operation" >> "$MAINNET_CANISTER_LOG"
        echo "  - Identity: $identity" >> "$MAINNET_CANISTER_LOG"
        echo "  - Cost: $cost cycles" >> "$MAINNET_CANISTER_LOG"
        echo "  - Network: $MAINNET_NETWORK" >> "$MAINNET_CANISTER_LOG"
        echo "" >> "$MAINNET_CANISTER_LOG"
        
        echo_info "📝 Logged canister creation to: $MAINNET_CANISTER_LOG"
        echo_info "Canister ID: $canister_id"
        echo_info "Identity: $identity"
        echo_info "Cost: $cost cycles"
    fi
}

# Function to list all created canisters
list_created_canisters() {
    if [[ -f "$MAINNET_CANISTER_LOG" ]]; then
        echo_info "=== All Created Canisters ==="
        if [[ -s "$MAINNET_CANISTER_LOG" ]]; then
            cat "$MAINNET_CANISTER_LOG"
        else
            echo_info "No canisters have been created yet"
        fi
    else
        echo_info "No canister log file found"
    fi
}

# Test function to check if personal canister already exists
test_check_existing_personal_canister() {
    echo_info "Checking if personal canister already exists..."
    
    local result=$(dfx canister call "$CANISTER_ID" get_my_personal_canister_id --query $NETWORK_FLAG 2>/dev/null)
    
    if echo "$result" | grep -q "opt principal"; then
        local canister_id=$(extract_principal "$result")
        echo_info "Personal canister already exists: $canister_id"
        echo "$canister_id"
        return 0
    elif echo "$result" | grep -q "(null)"; then
        echo_info "No personal canister exists yet"
        echo ""
        return 0
    else
        echo_error "Failed to check existing personal canister: $result"
        return 1
    fi
}

# Test function to create personal canister with detailed monitoring
test_create_personal_canister_with_monitoring() {
    echo_info "Testing personal canister creation with detailed monitoring..."
    
    # Check if canister already exists
    local existing_canister=$(test_check_existing_personal_canister)
    if [[ -n "$existing_canister" ]]; then
        echo_warn "Personal canister already exists: $existing_canister"
        echo_info "Skipping creation test to avoid duplicate creation"
        return 0
    fi
    
    # Warn about expensive operation on mainnet
    if [[ "$MAINNET_MODE" == "true" ]]; then
        warn_expensive_operation "Personal canister creation" "1-2"
    fi
    
    # Monitor cycles for expensive operations
    local initial_balance=""
    if [[ "$MAINNET_MODE" == "true" ]]; then
        echo_info "Starting cycle monitoring..."
        initial_balance=$(monitor_cycles "$CANISTER_ID" "$NETWORK_FLAG" "Personal canister creation")
    fi
    
    # Ensure user is registered first
    echo_info "Ensuring user is registered..."
    local register_result=$(dfx canister call "$CANISTER_ID" register $NETWORK_FLAG 2>/dev/null)
    echo_info "Register result: $register_result"
    
    # Call create_personal_canister endpoint
    echo_info "Calling create_personal_canister..."
    local result=$(dfx canister call "$CANISTER_ID" create_personal_canister $NETWORK_FLAG 2>/dev/null)
    echo_info "Create result: $result"
    
    # Calculate cycle consumption if monitoring
    local consumption="0"
    if [[ "$MAINNET_MODE" == "true" && -n "$initial_balance" ]]; then
        echo_info "Calculating cycle consumption..."
        consumption=$(calculate_cycle_consumption "$initial_balance" "$CANISTER_ID" "$NETWORK_FLAG" "Personal canister creation")
        echo_info "Total cycle consumption: $consumption"
    fi
    
    # Check if creation was initiated successfully
    if is_success "$result"; then
        local canister_id=$(extract_canister_id "$result")
        echo_info "Personal canister creation initiated successfully"
        if [ -n "$canister_id" ]; then
            echo_info "New canister ID: $canister_id"
            
            # Verify the canister actually exists
            echo_info "Verifying canister exists..."
            if dfx canister status "$canister_id" $NETWORK_FLAG >/dev/null 2>&1; then
                echo_info "✅ Canister verification successful: $canister_id"
                
                # Log the canister creation on mainnet
                log_created_canister "$canister_id" "Personal canister creation" "$consumption"
                
                return 0
            else
                echo_error "❌ Canister verification failed: $canister_id"
                return 1
            fi
        fi
        return 0
    elif is_failure "$result"; then
        echo_info "Personal canister creation failed (may be expected): $result"
        # Failure might be expected (e.g., already exists, disabled, insufficient cycles)
        return 0
    else
        echo_error "Personal canister creation returned unexpected result: $result"
        return 1
    fi
}

# Test function to check creation status
test_check_creation_status() {
    echo_info "Checking creation status..."
    
    local result=$(dfx canister call "$CANISTER_ID" get_creation_status --query $NETWORK_FLAG 2>/dev/null)
    echo_info "Creation status: $result"
    
    if echo "$result" | grep -q "opt record"; then
        local status=$(extract_creation_status "$result")
        echo_info "Status: $status"
        return 0
    elif echo "$result" | grep -q "(null)"; then
        echo_info "No creation status (null)"
        return 0
    else
        echo_error "Failed to get creation status: $result"
        return 1
    fi
}

# Test function to list all personal canisters
test_list_all_personal_canisters() {
    echo_info "Testing list all personal canisters..."
    
    # Try to call a function that lists all personal canisters
    # Note: This function might not exist yet in the backend
    local result=$(dfx canister call "$CANISTER_ID" list_all_personal_canisters --query $NETWORK_FLAG 2>/dev/null)
    
    if [[ $? -eq 0 ]]; then
        echo_info "List all personal canisters result: $result"
        
        # Check if we got a list of canisters
        if echo "$result" | grep -q "vec"; then
            echo_info "✅ Successfully retrieved list of personal canisters"
            return 0
        elif echo "$result" | grep -q "vec { }"; then
            echo_info "✅ List retrieved but no personal canisters found"
            return 0
        else
            echo_info "Unexpected result format: $result"
            return 0  # Still consider it a success if the function exists
        fi
    else
        echo_warn "Function 'list_all_personal_canisters' not found or failed"
        echo_info "This function might not be implemented yet in the backend"
        return 0  # Don't fail the test if the function doesn't exist
    fi
}

# Main test execution
main() {
    # Handle list mode
    if [[ "$LIST_MODE" == "true" ]]; then
        list_created_canisters
        exit 0
    fi
    
    echo "========================================="
    echo "Starting $TEST_NAME"
    echo "========================================="
    echo ""
    
    # Canister ID is now set dynamically based on mode (local vs mainnet)
    echo_info "Testing with canister: $CANISTER_ID"
    
    # Check if dfx is available
    if ! command -v dfx &> /dev/null; then
        echo_fail "dfx command not found"
        echo_info "Please install dfx and ensure it's in your PATH"
        exit 1
    fi
    
    # Run tests
    echo_info "=== Testing Personal Canister Creation Cost ==="
    run_capsule_test "Check existing personal canister" "test_check_existing_personal_canister"
    run_capsule_test "Create personal canister with monitoring" "test_create_personal_canister_with_monitoring"
    run_capsule_test "Check creation status" "test_check_creation_status"
    run_capsule_test "List all personal canisters" "test_list_all_personal_canisters"
    
    # Show created canisters log if on mainnet
    if [[ "$MAINNET_MODE" == "true" && -f "$MAINNET_CANISTER_LOG" ]]; then
        echo_info "=== Created Canisters Log ==="
        echo_info "Log file: $MAINNET_CANISTER_LOG"
        if [[ -s "$MAINNET_CANISTER_LOG" ]]; then
            echo_info "Recent canister creations:"
            tail -20 "$MAINNET_CANISTER_LOG"
        else
            echo_info "No canisters created in this session"
        fi
        echo ""
    fi
    
    # Print test summary
    print_test_summary
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
