#!/bin/bash

# 2-Lane + 4-Asset Upload System Test Runner
# 
# This script runs the comprehensive test that validates the 2-lane + 4-asset
# upload system using ICP backend, reproducing the S3 system concept.

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

# Check if test image exists
TEST_IMAGE_PATH="tests/backend/shared-capsule/upload/assets/input/avocado_big_21mb.jpg"
if [ ! -f "$TEST_IMAGE_PATH" ]; then
    echo_error "Test image not found: $TEST_IMAGE_PATH"
    echo_info "Please ensure the test assets are available"
    exit 1
fi

echo_info "Test image found: $TEST_IMAGE_PATH"

# Run the test
echo_info "Running 2-Lane + 4-Asset Upload System Test..."
echo_info "This test validates the parallel processing concept for ICP backend"
echo ""

cd tests/backend/shared-capsule/upload
node test_upload_2lane_4asset_system.mjs "$BACKEND_CANISTER_ID"

# Check exit code
if [ $? -eq 0 ]; then
    echo_success "2-Lane + 4-Asset Upload System Test completed successfully!"
    echo_info "The concept is validated and ready for frontend implementation"
else
    echo_error "2-Lane + 4-Asset Upload System Test failed!"
    echo_info "Please check the test output above for details"
    exit 1
fi

