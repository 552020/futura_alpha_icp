#!/bin/bash

# Get Mainnet Canister Metadata
# Utility script to retrieve canister metadata from mainnet deployment
#
# DFX Methods Used:
# - dfx canister metadata <canister_id> --network ic
# - dfx canister metadata <canister_id> --network ic candid:service

# Load mainnet configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../tests/backend/mainnet_test_config.sh"

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

# Function to get all available metadata
get_all_metadata() {
    echo_info "Getting available canister metadata..."
    
    # List common metadata types that might be available
    local metadata_types=("candid:service" "candid:args" "candid:types" "candid:metadata")
    local found_metadata=false
    
    for metadata_type in "${metadata_types[@]}"; do
        echo_info "Checking metadata: $metadata_type"
        local result=$(dfx canister metadata "$MAINNET_CANISTER_ID" --network "$MAINNET_NETWORK" "$metadata_type" 2>/dev/null)
        
        if [[ $? -eq 0 && -n "$result" ]]; then
            echo_success "Found metadata: $metadata_type"
            echo "=== $metadata_type ==="
            echo "$result"
            echo ""
            found_metadata=true
        else
            echo_warning "No metadata found for: $metadata_type"
        fi
    done
    
    if [[ "$found_metadata" == true ]]; then
        echo_success "Metadata retrieval completed"
        return 0
    else
        echo_error "No metadata found for any common types"
        return 1
    fi
}

# Function to get Candid interface via metadata
get_candid_metadata() {
    echo_info "Getting Candid interface via metadata..."
    
    local result=$(dfx canister metadata "$MAINNET_CANISTER_ID" --network "$MAINNET_NETWORK" candid:service 2>/dev/null)
    
    if [[ $? -eq 0 && -n "$result" ]]; then
        echo_success "Candid interface retrieved via metadata"
        echo "$result"
        return 0
    else
        echo_error "Failed to retrieve Candid interface via metadata"
        return 1
    fi
}

# Function to save metadata to file
save_metadata_to_file() {
    local output_file="${1:-/tmp/mainnet_canister_metadata.txt}"
    
    echo_info "Saving available metadata to file: $output_file"
    
    # Get all available metadata and save to file
    local metadata_types=("candid:service" "candid:args" "candid:types" "candid:metadata")
    local found_metadata=false
    
    # Clear the output file
    > "$output_file"
    
    for metadata_type in "${metadata_types[@]}"; do
        local result=$(dfx canister metadata "$MAINNET_CANISTER_ID" --network "$MAINNET_NETWORK" "$metadata_type" 2>/dev/null)
        
        if [[ $? -eq 0 && -n "$result" ]]; then
            echo "=== $metadata_type ===" >> "$output_file"
            echo "$result" >> "$output_file"
            echo "" >> "$output_file"
            found_metadata=true
        fi
    done
    
    if [[ "$found_metadata" == true && -f "$output_file" && -s "$output_file" ]]; then
        local file_size=$(wc -c < "$output_file")
        echo_success "Metadata saved to: $output_file ($file_size bytes)"
        return 0
    else
        echo_error "Failed to save metadata to file"
        return 1
    fi
}

# Function to save Candid interface to file
save_candid_to_file() {
    local output_file="${1:-/tmp/mainnet_candid_interface.did}"
    
    echo_info "Saving Candid interface to file: $output_file"
    
    local result=$(dfx canister metadata "$MAINNET_CANISTER_ID" --network "$MAINNET_NETWORK" candid:service 2>/dev/null)
    
    if [[ $? -eq 0 && -n "$result" ]]; then
        echo "$result" > "$output_file"
        
        if [[ -f "$output_file" && -s "$output_file" ]]; then
            local file_size=$(wc -c < "$output_file")
            echo_success "Candid interface saved to: $output_file ($file_size bytes)"
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

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --all              Get all canister metadata"
    echo "  --candid           Get Candid interface via metadata"
    echo "  --save-metadata    Save all metadata to file (default: /tmp/mainnet_canister_metadata.txt)"
    echo "  --save-candid      Save Candid interface to file (default: /tmp/mainnet_candid_interface.did)"
    echo "  --file FILE        Specify custom output file"
    echo "  --help             Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 --all                    # Get all metadata"
    echo "  $0 --candid                 # Get Candid interface"
    echo "  $0 --save-metadata          # Save metadata to default file"
    echo "  $0 --save-candid --file my_interface.did  # Save Candid to custom file"
}

# Main execution
main() {
    echo_info "Mainnet Canister Metadata Utility"
    echo_info "=================================="
    
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
    local show_all=false
    local show_candid=false
    local save_metadata=false
    local save_candid=false
    local output_file=""
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --all)
                show_all=true
                shift
                ;;
            --candid)
                show_candid=true
                shift
                ;;
            --save-metadata)
                save_metadata=true
                shift
                ;;
            --save-candid)
                save_candid=true
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
    
    # If no options specified, show all metadata by default
    if [[ "$show_all" == false && "$show_candid" == false && "$save_metadata" == false && "$save_candid" == false ]]; then
        show_all=true
    fi
    
    # Execute requested operations
    if [[ "$show_all" == true ]]; then
        get_all_metadata
        echo ""
    fi
    
    if [[ "$show_candid" == true ]]; then
        get_candid_metadata
        echo ""
    fi
    
    if [[ "$save_metadata" == true ]]; then
        save_metadata_to_file "$output_file"
        echo ""
    fi
    
    if [[ "$save_candid" == true ]]; then
        save_candid_to_file "$output_file"
        echo ""
    fi
    
    echo_success "Metadata operations completed!"
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
