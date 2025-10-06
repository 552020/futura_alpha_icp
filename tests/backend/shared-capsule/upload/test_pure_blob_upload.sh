#!/bin/bash

# Pure Blob Upload Test Script
# Tests the simplified upload flow that only creates blobs (no memories)

set -e

echo "========================================="
echo "Pure Blob Upload Test"
echo "========================================="
echo "ℹ️  This test validates pure blob upload without memory creation."
echo "ℹ️  File: avocado_medium_3.5mb.jpg (3.5MB)"
echo "ℹ️  Chunk size: 1.8MB (matches backend)"
echo "ℹ️  Expected chunks: 3"
echo ""

# Check prerequisites
echo "ℹ️  Checking prerequisites..."

# Check if DFX is running
if ! curl -s http://127.0.0.1:4943 > /dev/null; then
    echo "❌ DFX is not running. Please start DFX first."
    exit 1
fi
echo "✅ DFX is running"

# Check if backend canister is deployed
if ! dfx canister id backend > /dev/null 2>&1; then
    echo "❌ Backend canister is not deployed. Please deploy first."
    exit 1
fi
echo "✅ Backend canister is deployed"

# Check if Node.js is available
if ! command -v node > /dev/null; then
    echo "❌ Node.js is not available. Please install Node.js."
    exit 1
fi
echo "✅ Node.js is available"

# Check if test asset exists
if [ ! -f "assets/input/avocado_medium_3.5mb.jpg" ]; then
    echo "❌ Test asset not found: assets/input/avocado_medium_3.5mb.jpg"
    exit 1
fi
echo "✅ Test asset found: avocado_medium_3.5mb.jpg"

# Get backend canister ID
BACKEND_CANISTER_ID=$(dfx canister id backend)
echo "ℹ️  Backend canister ID: $BACKEND_CANISTER_ID"

echo "========================================="
echo "Running Pure Blob Upload Test"
echo "========================================="

# Run the pure blob upload test
echo "ℹ️  Starting pure blob upload test..."
node test_pure_blob_upload.mjs "$BACKEND_CANISTER_ID" "assets/input/avocado_medium_3.5mb.jpg"

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Pure blob upload test PASSED!"
    echo "ℹ️  Pure blob upload functionality is working correctly."
    echo "ℹ️  Ready to implement memory creation endpoints."
else
    echo ""
    echo "❌ Pure blob upload test FAILED!"
    echo "ℹ️  Please check the error messages above and fix issues."
    exit 1
fi
