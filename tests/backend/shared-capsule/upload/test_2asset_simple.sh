#!/bin/bash

# Simple 2-Asset Test Script
# Tests uploading 2 small assets, creating memory, and deleting everything

set -e

# Source test utilities for DFX color fix
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../test_utils.sh"

echo "🚀 Simple 2-Asset Test"
echo "======================"

# Check if DFX is running
if ! curl -s http://127.0.0.1:4943/api/v2/status > /dev/null; then
    echo "❌ DFX is not running. Please start DFX first."
    exit 1
fi

# Get backend canister ID
echo "ℹ️  Getting backend canister ID..."
BACKEND_CANISTER_ID=$(dfx canister id backend 2>/dev/null || echo "")
if [ -z "$BACKEND_CANISTER_ID" ]; then
    echo "❌ Could not get backend canister ID. Is the backend deployed?"
    exit 1
fi
echo "ℹ️  Backend canister ID: $BACKEND_CANISTER_ID"

# Run the test
echo "ℹ️  Running Simple 2-Asset Test..."
echo "ℹ️  This test uploads 2 small assets, creates a memory, and tests deletion"
echo ""

node "$SCRIPT_DIR/test_2asset_simple.mjs" "$BACKEND_CANISTER_ID"

echo ""
echo "✅ Simple 2-Asset Test completed successfully!"
