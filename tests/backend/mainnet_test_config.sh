#!/bin/bash

# Mainnet Test Configuration
# Shared configuration for all mainnet tests

# Load test utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/test_utils.sh"

# Mainnet configuration
MAINNET_NETWORK="ic"
MAINNET_CANISTER_NAME="backend"

# Get mainnet canister ID from canister_ids.json
get_mainnet_canister_id() {
    local canister_ids_file="canister_ids.json"
    
    # Check if we're in the project root
    if [[ -f "$canister_ids_file" ]]; then
        # We're in project root
        local canister_id=$(grep '"ic"' "$canister_ids_file" | sed 's/.*"ic": *"\([^"]*\)".*/\1/')
    elif [[ -f "../../$canister_ids_file" ]]; then
        # We're in tests/backend/ directory
        local canister_id=$(grep '"ic"' "../../$canister_ids_file" | sed 's/.*"ic": *"\([^"]*\)".*/\1/')
    else
        echo_error "Cannot find canister_ids.json file"
        return 1
    fi
    
    if [[ -n "$canister_id" ]]; then
        echo "$canister_id"
        return 0
    else
        echo_error "Cannot extract mainnet canister ID from canister_ids.json"
        return 1
    fi
}

# Set mainnet canister ID
MAINNET_CANISTER_ID=$(get_mainnet_canister_id)

# Validate mainnet configuration
validate_mainnet_config() {
    if [[ -z "$MAINNET_CANISTER_ID" ]]; then
        echo_error "Mainnet canister ID not set"
        return 1
    fi
    
    if [[ -z "$MAINNET_CANISTER_NAME" ]]; then
        echo_error "Mainnet canister name not set"
        return 1
    fi
    
    if [[ -z "$MAINNET_NETWORK" ]]; then
        echo_error "Mainnet network not set"
        return 1
    fi
    
    echo_info "Mainnet configuration validated:"
    echo_info "  Canister ID: $MAINNET_CANISTER_ID"
    echo_info "  Canister Name: $MAINNET_CANISTER_NAME"
    echo_info "  Network: $MAINNET_NETWORK"
    
    return 0
}

# Helper function to run dfx canister call on mainnet
dfx_mainnet_call() {
    local function_name="$1"
    local args="${2:-()}"
    
    dfx canister call "$MAINNET_CANISTER_NAME" --network "$MAINNET_NETWORK" "$function_name" "$args"
}

# Helper function to run dfx canister status on mainnet
dfx_mainnet_status() {
    dfx canister status "$MAINNET_CANISTER_NAME" --network "$MAINNET_NETWORK"
}

# Helper function to run dfx canister info on mainnet
dfx_mainnet_info() {
    dfx canister info "$MAINNET_CANISTER_ID" --network "$MAINNET_NETWORK"
}

# Helper function to get mainnet canister ID via dfx
dfx_mainnet_canister_id() {
    dfx canister id "$MAINNET_CANISTER_NAME" --network "$MAINNET_NETWORK"
}

# Export configuration for use in other scripts
export MAINNET_CANISTER_ID
export MAINNET_CANISTER_NAME
export MAINNET_NETWORK
