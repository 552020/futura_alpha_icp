#!/bin/bash

# Individual Test Runner for 2-Lane + 4-Asset System
# 
# This script runs individual test components to debug specific issues
# without running the full complex test suite.

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

# Check if we're in the right directory
if [ ! -f "dfx.json" ]; then
    echo_error "Please run this script from the project root directory"
    exit 1
fi

# Check if dfx is running
if ! dfx ping 2>/dev/null; then
    echo_error "dfx is not running. Please start it with: dfx start"
    exit 1
fi

# Get backend canister ID
echo_info "Getting backend canister ID..."
BACKEND_CANISTER_ID=$(dfx canister id backend 2>/dev/null)
if [ -z "$BACKEND_CANISTER_ID" ]; then
    echo_error "Failed to get backend canister ID. Is the backend canister deployed?"
    exit 1
fi

echo_info "Backend canister ID: $BACKEND_CANISTER_ID"

# Parse command line arguments
TEST_TYPE=${1:-"all"}

echo_info "Running individual tests for: $TEST_TYPE"
echo ""

cd tests/backend/shared-capsule/upload

# Function to run a test
run_test() {
    local test_name="$1"
    local test_file="$2"
    
    echo_info "Running: $test_name"
    echo "----------------------------------------"
    
    if node "$test_file" "$BACKEND_CANISTER_ID"; then
        echo_success "$test_name completed successfully!"
    else
        echo_error "$test_name failed!"
        return 1
    fi
    
    echo ""
}

# Run tests based on type
case "$TEST_TYPE" in
    "lane-a"|"original")
        run_test "Lane A: Original Upload" "test_lane_a_original_upload.mjs"
        ;;
    "lane-b"|"processing")
        run_test "Lane B: Image Processing" "test_lane_b_image_processing.mjs"
        ;;
    "memory"|"creation")
        run_test "Memory Creation Debug" "test_memory_creation_debug.mjs"
        ;;
    "all")
        echo_info "Running all individual tests..."
        echo ""
        
        # Run Lane A test
        if run_test "Lane A: Original Upload" "test_lane_a_original_upload.mjs"; then
            echo_success "Lane A test passed!"
        else
            echo_error "Lane A test failed!"
            exit 1
        fi
        
        # Run Lane B test
        if run_test "Lane B: Image Processing" "test_lane_b_image_processing.mjs"; then
            echo_success "Lane B test passed!"
        else
            echo_error "Lane B test failed!"
            exit 1
        fi
        
        # Run Memory Creation test
        if run_test "Memory Creation Debug" "test_memory_creation_debug.mjs"; then
            echo_success "Memory Creation test passed!"
        else
            echo_error "Memory Creation test failed!"
            exit 1
        fi
        
        echo_success "All individual tests passed! 🎉"
        ;;
    *)
        echo_error "Unknown test type: $TEST_TYPE"
        echo_info "Available test types:"
        echo_info "  lane-a, original    - Test Lane A (original file upload)"
        echo_info "  lane-b, processing  - Test Lane B (image processing)"
        echo_info "  memory, creation    - Test memory creation with simplified metadata"
        echo_info "  all                 - Run all individual tests"
        echo ""
        echo_info "Usage: $0 [test_type]"
        echo_info "Example: $0 lane-a"
        echo_info "Example: $0 memory"
        echo_info "Example: $0 all"
        exit 1
        ;;
esac
