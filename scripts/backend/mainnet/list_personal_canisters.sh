#!/bin/bash

# List all personal canisters
# This script lists all personal canisters created by the current user
# and shows information about them
#
# Usage: ./list_personal_canisters.sh [--mainnet]
# - Without --mainnet: Lists canisters from local backend
# - With --mainnet: Lists canisters from mainnet backend

# Load configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../../tests/backend/test_utils.sh"

# Parse command line arguments
MAINNET_MODE=false
if [[ "$1" == "--mainnet" ]]; then
    MAINNET_MODE=true
    echo_info "Running in mainnet mode"
fi

# Set up canister configuration based on mode
if [[ "$MAINNET_MODE" == "true" ]]; then
    source "$SCRIPT_DIR/../../../tests/backend/mainnet/config.sh"
    CANISTER_ID="$MAINNET_CANISTER_ID"
    NETWORK_FLAG="--network $MAINNET_NETWORK"
    echo_info "Using mainnet canister: $CANISTER_ID"
else
    CANISTER_ID="${BACKEND_CANISTER_ID:-$(dfx canister id backend 2>/dev/null)}"
    NETWORK_FLAG=""
    if [ -z "$CANISTER_ID" ]; then
        echo_error "Backend canister not found. Make sure it's deployed locally."
        exit 1
    fi
    echo_info "Using local canister: $CANISTER_ID"
fi

echo "========================================="
echo "Personal Canisters List"
echo "========================================="
echo ""

# Get current user's personal canister ID
echo_info "Getting current user's personal canister ID..."
my_canister_result=$(dfx canister call "$CANISTER_ID" get_my_personal_canister_id --query $NETWORK_FLAG 2>/dev/null)

if echo "$my_canister_result" | grep -q "opt principal"; then
    my_canister_id=$(extract_principal "$my_canister_result")
    echo_info "Your personal canister ID: $my_canister_id"
    
    # Get canister status
    echo_info "Getting canister status..."
    if dfx canister status "$my_canister_id" $NETWORK_FLAG >/dev/null 2>&1; then
        echo_info "✅ Canister is running"
        dfx canister status "$my_canister_id" $NETWORK_FLAG
    else
        echo_warn "❌ Canister not found or not running"
    fi
else
    echo_info "No personal canister found for current user"
fi

echo ""

# Try to list all personal canisters (if function exists)
echo_info "Attempting to list all personal canisters..."
all_canisters_result=$(dfx canister call "$CANISTER_ID" list_all_personal_canisters --query $NETWORK_FLAG 2>/dev/null)

if [[ $? -eq 0 ]]; then
    echo_info "All personal canisters:"
    echo "$all_canisters_result"
else
    echo_warn "Function 'list_all_personal_canisters' not available"
    echo_info "This function might not be implemented yet in the backend"
fi

echo ""

# Show log file if on mainnet
if [[ "$MAINNET_MODE" == "true" ]]; then
    log_file="$SCRIPT_DIR/../../../tests/backend/mainnet/created_canisters.log"
    if [[ -f "$log_file" ]]; then
        echo_info "=== Created Canisters Log ==="
        echo_info "Log file: $log_file"
        if [[ -s "$log_file" ]]; then
            echo_info "All canister creations:"
            cat "$log_file"
        else
            echo_info "No canisters have been created yet"
        fi
    else
        echo_info "No canister log file found"
    fi
fi

echo ""
echo "========================================="
echo "List Complete"
echo "========================================="
