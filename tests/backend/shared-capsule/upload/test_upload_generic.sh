#!/bin/bash

# Generic Upload Test Runner
# This script uses the generic upload test to upload any file

set -e

# Source test utilities (includes DFX color fix)
SCRIPT_DIR="$(dirname "$0")"
source "$SCRIPT_DIR/../../test_utils.sh"

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

echo_header() {
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}=========================================${NC}"
}

# Check prerequisites
check_prerequisites() {
    echo_info "Checking prerequisites..."
    
    # Check if dfx is running
    if ! dfx ping >/dev/null 2>&1; then
        echo_error "DFX is not running. Please start it with: dfx start --background"
        exit 1
    fi
    echo_success "DFX is running"
    
    # Check if backend canister is deployed
    if ! dfx canister id backend >/dev/null 2>&1; then
        echo_error "Backend canister is not deployed. Please deploy it first."
        exit 1
    fi
    echo_success "Backend canister is deployed"
    
    # Check if Node.js is available
    if ! command -v node >/dev/null 2>&1; then
        echo_error "Node.js is not installed. Please install Node.js."
        exit 1
    fi
    echo_success "Node.js is available"
    
    # Check if test file exists (passed as argument)
    if [ -z "$1" ]; then
        echo_error "Usage: $0 <FILE_PATH>"
        echo_info "Example: $0 assets/input/avocado_medium_3.5mb.jpg"
        exit 1
    fi
    
    if [ ! -f "$1" ]; then
        echo_error "Test file not found: $1"
        echo_info "Please ensure the test file exists."
        exit 1
    fi
    echo_success "Test file found: $1"
}

# Get backend canister ID
get_backend_id() {
    BACKEND_ID=$(dfx canister id backend)
    if [ -z "$BACKEND_ID" ]; then
        echo_error "Failed to get backend canister ID"
        exit 1
    fi
    echo_info "Backend canister ID: $BACKEND_ID"
}

# Main test execution
main() {
    FILE_PATH="$1"
    echo_header "Generic Upload Test"
    echo_info "This test uses the generic upload test to upload any file."
    echo_info "File: $FILE_PATH"
    echo_info "Chunk size: 1.8MB (matches backend)"
    echo ""
    
    # Check prerequisites
    check_prerequisites "$FILE_PATH"
    
    # Get backend ID
    get_backend_id
    
    echo_header "Running Generic Upload Test"
    
    # Run the generic test with the specified file
    echo_info "Starting upload test for: $FILE_PATH"
    if node test_upload.mjs "$BACKEND_ID" "$FILE_PATH"; then
        echo_success "Upload test PASSED!"
        echo_info "Chunked upload functionality is working correctly."
        echo_info "You can now proceed with refactoring."
        exit 0
    else
        echo_error "Upload test FAILED!"
        echo_info "Please check the error messages above and fix issues before refactoring."
        exit 1
    fi
}

# Run main function
main "$@"
