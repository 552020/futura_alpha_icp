#!/bin/bash

# Get Mainnet Canister Info
# Utility script to retrieve public canister information from mainnet deployment
#
# DFX Methods Used:
# - dfx canister info <canister_id> --network ic

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

# Function to get public canister info
get_public_info() {
    echo_info "Getting public canister information..."
    
    local result=$(dfx canister info "$MAINNET_CANISTER_ID" --network "$MAINNET_NETWORK" 2>/dev/null)
    
    if [[ $? -eq 0 && -n "$result" ]]; then
        echo_success "Public canister info retrieved successfully"
        echo ""
        echo "$result"
        return 0
    else
        echo_error "Failed to retrieve public canister info"
        return 1
    fi
}

# Function to extract and display specific fields
display_info_fields() {
    echo_info "Extracting and displaying specific fields..."
    
    local result=$(dfx canister info "$MAINNET_CANISTER_ID" --network "$MAINNET_NETWORK" 2>/dev/null)
    
    if [[ $? -eq 0 && -n "$result" ]]; then
        echo_success "Public canister info retrieved successfully"
        echo ""
        
        # Extract module hash
        local module_hash=$(echo "$result" | grep "^Module hash:" | sed 's/^Module hash: *//')
        if [[ -n "$module_hash" ]]; then
            echo_info "Module hash: $module_hash"
        fi
        
        # Extract controllers
        local controllers=$(echo "$result" | grep "^Controllers:" | sed 's/^Controllers: *//')
        if [[ -n "$controllers" ]]; then
            echo_info "Controllers: $controllers"
        fi
        
        return 0
    else
        echo_error "Failed to retrieve public canister info"
        return 1
    fi
}

# Function to save info to file
save_info_to_file() {
    local output_file="${1:-/tmp/mainnet_canister_info.txt}"
    
    echo_info "Saving public canister info to file: $output_file"
    
    local result=$(dfx canister info "$MAINNET_CANISTER_ID" --network "$MAINNET_NETWORK" 2>/dev/null)
    
    if [[ $? -eq 0 && -n "$result" ]]; then
        echo "$result" > "$output_file"
        
        if [[ -f "$output_file" && -s "$output_file" ]]; then
            local file_size=$(wc -c < "$output_file")
            echo_success "Public canister info saved to: $output_file ($file_size bytes)"
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

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --full              Show full public canister info (default)"
    echo "  --fields            Show only specific fields (module hash, controllers)"
    echo "  --save              Save info to file (default: /tmp/mainnet_canister_info.txt)"
    echo "  --file FILE         Specify custom output file"
    echo "  --help              Show this help"
    echo ""
    echo "Examples:"
    echo "  $0                  # Show full public info"
    echo "  $0 --fields         # Show only key fields"
    echo "  $0 --save           # Save to default file"
    echo "  $0 --save --file my_info.txt  # Save to custom file"
}

# Main execution
main() {
    echo_info "Mainnet Canister Info Utility"
    echo_info "============================="
    
    # Validate mainnet configuration
    if ! validate_mainnet_config; then
        echo_error "Mainnet configuration validation failed"
        exit 1
    fi
    
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
        get_public_info
        echo ""
    fi
    
    if [[ "$show_fields" == true ]]; then
        display_info_fields
        echo ""
    fi
    
    if [[ "$save_to_file" == true ]]; then
        save_info_to_file "$output_file"
        echo ""
    fi
    
    echo_success "Info operations completed!"
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
