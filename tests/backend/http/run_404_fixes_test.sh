#!/bin/bash

# Test script for 404 fixes
# This script runs the integration tests to verify the 404 fixes are working

set -e

echo "🔧 404 Fixes Integration Test Runner"
echo "===================================="

# Check if we're in the right directory
if [ ! -f "test_404_fixes.mjs" ]; then
    echo "❌ Error: test_404_fixes.mjs not found. Please run this script from the tests/backend/http directory."
    exit 1
fi

# Check if Node.js is available
if ! command -v node &> /dev/null; then
    echo "❌ Error: Node.js is not installed or not in PATH"
    exit 1
fi

# Check if the local canister is running
echo "🔍 Checking if local canister is running..."
if ! curl -s http://127.0.0.1:4943 > /dev/null; then
    echo "❌ Error: Local canister is not running on http://127.0.0.1:4943"
    echo "   Please start your local canister first:"
    echo "   dfx start --clean"
    echo "   dfx deploy"
    exit 1
fi

echo "✅ Local canister is running"

# Run the tests
echo ""
echo "🧪 Running 404 fixes integration tests..."
echo ""

node test_404_fixes.mjs

echo ""
echo "✅ Test run completed!"
echo ""
echo "📝 Next steps:"
echo "   1. Check the canister logs for diagnostic output:"
echo "      dfx logs"
echo "   2. Look for [HTTP-ASSET], [ASSET-LOOKUP], and [VARIANT-RESOLVE] log entries"
echo "   3. Verify that token.subject is being used correctly"
echo "   4. Check that variant-to-asset-id resolution is working"
echo ""




