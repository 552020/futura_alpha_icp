#!/bin/bash

# Simple Avocado Upload Test
# This script uses the generic upload test to upload the avocado file

set -e

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
    
    # Check if test assets exist
    if [ ! -f "assets/input/avocado_medium_3.5mb.jpg" ]; then
        echo_error "Test asset not found: assets/input/avocado_medium_3.5mb.jpg"
        echo_info "Please ensure the test assets are available."
        exit 1
    fi
    echo_success "Test asset found: avocado_medium_3.5mb.jpg"
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
    echo_header "Simple Avocado Upload Test"
    echo_info "This test uses the generic upload test to upload the avocado file."
    echo_info "File: avocado_medium_3.5mb.jpg (3.5MB)"
    echo_info "Chunk size: 1.8MB (matches backend)"
    echo_info "Expected chunks: 3"
    echo ""
    
    # Check prerequisites
    check_prerequisites
    
    # Get backend ID
    get_backend_id
    
    echo_header "Running Avocado Upload Test"
    
    # Run the generic test with avocado file
    echo_info "Starting avocado upload test using generic upload test..."
    if node test_upload.mjs "$BACKEND_ID" "assets/input/avocado_medium_3.5mb.jpg"; then
        echo_success "Avocado upload test PASSED!"
        echo_info "Chunked upload functionality is working correctly."
        echo_info "You can now proceed with refactoring."
        exit 0
    else
        echo_error "Avocado upload test FAILED!"
        echo_info "Please check the error messages above and fix issues before refactoring."
        exit 1
    fi
}

# Run main function
main "$@"
