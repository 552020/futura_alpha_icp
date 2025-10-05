#!/bin/bash

# Golden E2E Test: Memory Creation and Retrieval Workflow
# 
# This test validates the complete memory workflow:
# 1. Create memory with real content
# 2. Retrieve memory using the returned ID
# 3. Verify content integrity
# 4. Clean up properly
#
# This serves as a guardrail against interface regressions and ensures
# the core memory API works end-to-end.

# Load test configuration and utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../test_utils.sh"

# Test configuration
TEST_NAME="Golden E2E Memory Workflow Test"
DEBUG="${DEBUG:-false}"

echo_header "🎯 $TEST_NAME"
echo "This test validates the complete memory workflow:"
echo "  • Create memory with real content"
echo "  • Retrieve memory using the returned ID"
echo "  • Verify content integrity"
echo "  • Clean up properly"
echo ""

# Check prerequisites
echo_info "Checking prerequisites..."

# Check if dfx is available
if ! command -v dfx &> /dev/null; then
    echo_fail "dfx command not found"
    echo_error "Please install dfx and ensure it's in your PATH"
    exit 1
fi

# Check if node is available
if ! command -v node &> /dev/null; then
    echo_fail "node command not found"
    echo_error "Please install Node.js and ensure it's in your PATH"
    exit 1
fi

# Check if backend canister is running
if ! dfx canister call backend whoami &> /dev/null; then
    echo_fail "Backend canister is not responding"
    echo_error "Please ensure dfx is running and backend canister is deployed"
    echo_error "Run: dfx start && dfx deploy backend"
    exit 1
fi

echo_pass "Prerequisites check passed"

# Set environment variables
export BACKEND_CANISTER_ID=$(dfx canister id backend)
export IC_HOST="http://127.0.0.1:4943"

echo_info "Environment configuration:"
echo_info "  BACKEND_CANISTER_ID: $BACKEND_CANISTER_ID"
echo_info "  IC_HOST: $IC_HOST"
echo ""

# Run the golden E2E test
echo_info "Running golden E2E memory workflow test..."
echo ""

# Execute the JavaScript test
if node "$SCRIPT_DIR/test_memory_golden_e2e.mjs"; then
    echo ""
    echo_pass "🎉 Golden E2E test completed successfully!"
    echo_pass "Memory workflow validation passed"
    echo ""
    echo_info "This test confirms:"
    echo_info "  ✅ Memory creation works correctly"
    echo_info "  ✅ Memory retrieval works correctly"
    echo_info "  ✅ Content integrity is preserved"
    echo_info "  ✅ Memory ID consistency is maintained"
    echo_info "  ✅ Cleanup works properly"
    echo ""
    echo_pass "The core memory API is working end-to-end!"
    exit 0
else
    echo ""
    echo_fail "❌ Golden E2E test failed!"
    echo_error "Memory workflow validation failed"
    echo ""
    echo_error "This indicates a problem with:"
    echo_error "  • Memory creation API"
    echo_error "  • Memory retrieval API"
    echo_error "  • Content integrity"
    echo_error "  • Interface consistency"
    echo ""
    echo_error "Please check the error messages above and fix the issues."
    exit 1
fi
