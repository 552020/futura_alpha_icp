#!/bin/bash
# Quick test - just Lane A (single file upload)

set -e

cd "$(dirname "$0")"
BACKEND_CANISTER_ID=$(dfx canister id backend)

echo "🚀 Quick Single Upload Test"
echo "============================"
echo ""
echo "Backend canister: $BACKEND_CANISTER_ID"
echo ""

# Run just Lane A test
node ../../../../src/nextjs/scripts/test/test_2lane_4asset.mjs --test="Lane A: Original Upload"

echo ""
echo "✅ Test complete!"

