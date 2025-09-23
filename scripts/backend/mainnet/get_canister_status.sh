#!/bin/bash

# Get Mainnet Canister Status
# Utility script to retrieve detailed canister status from mainnet deployment
#
# DFX Methods Used:
# - dfx canister status <canister_name> --network ic

# Load mainnet configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../../tests/backend/mainnet/config.sh"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
echo_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

echo_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

echo_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

echo_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Function to get full canister status
get_full_status() {
    echo_info "Getting full canister status..."
    
    local result=$(dfx canister status "$MAINNET_CANISTER_NAME" --network "$MAINNET_NETWORK" 2>/dev/null)
    
    if [[ $? -eq 0 && -n "$result" ]]; then
        echo_success "Canister status retrieved successfully"
        echo ""
        echo "$result"
        return 0
    else
        echo_error "Failed to retrieve canister status"
        return 1
    fi
}

# Function to extract and display key status fields
display_status_fields() {
    echo_info "Extracting and displaying key status fields..."
    
    local result=$(dfx canister status "$MAINNET_CANISTER_NAME" --network "$MAINNET_NETWORK" 2>/dev/null)
    
    if [[ $? -eq 0 && -n "$result" ]]; then
        echo_success "Canister status retrieved successfully"
        echo ""
        
        # Extract key fields
        local status=$(echo "$result" | grep "^Status:" | sed 's/^Status: *//')
        local controllers=$(echo "$result" | grep "^Controllers:" | sed 's/^Controllers: *//')
        local memory_size=$(echo "$result" | grep "^Memory Size:" | sed 's/^Memory Size: *//')
        local balance=$(echo "$result" | grep "^Balance:" | sed 's/^Balance: *//')
        local module_hash=$(echo "$result" | grep "^Module hash:" | sed 's/^Module hash: *//')
        local idle_cycles=$(echo "$result" | grep "^Idle cycles burned per day:" | sed 's/^Idle cycles burned per day: *//')
        
        if [[ -n "$status" ]]; then
            echo_info "Status: $status"
        fi
        
        if [[ -n "$controllers" ]]; then
            echo_info "Controllers: $controllers"
        fi
        
        if [[ -n "$memory_size" ]]; then
            echo_info "Memory Size: $memory_size"
        fi
        
        if [[ -n "$balance" ]]; then
            echo_info "Balance: $balance"
        fi
        
        if [[ -n "$module_hash" ]]; then
            echo_info "Module hash: $module_hash"
        fi
        
        if [[ -n "$idle_cycles" ]]; then
            echo_info "Idle cycles burned per day: $idle_cycles"
        fi
        
        return 0
    else
        echo_error "Failed to retrieve canister status"
        return 1
    fi
}

# Function to save status to file
save_status_to_file() {
    local output_file="${1:-/tmp/mainnet_canister_status.txt}"
    
    echo_info "Saving canister status to file: $output_file"
    
    local result=$(dfx canister status "$MAINNET_CANISTER_NAME" --network "$MAINNET_NETWORK" 2>/dev/null)
    
    if [[ $? -eq 0 && -n "$result" ]]; then
        echo "$result" > "$output_file"
        
        if [[ -f "$output_file" && -s "$output_file" ]]; then
            local file_size=$(wc -c < "$output_file")
            echo_success "Canister status saved to: $output_file ($file_size bytes)"
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

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --full              Show full canister status (default)"
    echo "  --fields            Show only key fields (status, controllers, memory, balance, etc.)"
    echo "  --save              Save status to file (default: /tmp/mainnet_canister_status.txt)"
    echo "  --file FILE         Specify custom output file"
    echo "  --help              Show this help"
    echo ""
    echo "Examples:"
    echo "  $0                  # Show full status"
    echo "  $0 --fields         # Show only key fields"
    echo "  $0 --save           # Save to default file"
    echo "  $0 --save --file my_status.txt  # Save to custom file"
}

# Main execution
main() {
    echo_info "Mainnet Canister Status Utility"
    echo_info "==============================="
    
    # Validate mainnet configuration
    if ! validate_mainnet_config; then
        echo_error "Mainnet configuration validation failed"
        exit 1
    fi
    
    echo_info "Canister Name: $MAINNET_CANISTER_NAME"
    echo_info "Canister ID: $MAINNET_CANISTER_ID"
    echo_info "Network: $MAINNET_NETWORK"
    echo ""
    
    # Check prerequisites
    if ! command -v dfx &> /dev/null; then
        echo_error "dfx command not found. Please install dfx first."
        exit 1
    fi
    
    # Parse command line arguments
    local show_full=true
    local show_fields=false
    local save_to_file=false
    local output_file=""
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --full)
                show_full=true
                show_fields=false
                shift
                ;;
            --fields)
                show_full=false
                show_fields=true
                shift
                ;;
            --save)
                save_to_file=true
                shift
                ;;
            --file)
                output_file="$2"
                shift
                shift
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                echo_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done
    
    # Execute requested operations
    if [[ "$show_full" == true ]]; then
        get_full_status
        echo ""
    fi
    
    if [[ "$show_fields" == true ]]; then
        display_status_fields
        echo ""
    fi
    
    if [[ "$save_to_file" == true ]]; then
        save_status_to_file "$output_file"
        echo ""
    fi
    
    echo_success "Status operations completed!"
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
