#!/bin/bash

# Deploy script for lab_backend to local network
# This script handles the complete lab_backend deployment process

set -e

echo "🚀 Deploying lab_backend to local network..."

# Ensure we're in the project root
cd "$(dirname "$0")/.."

# Start local replica if not running
echo "🔄 Starting local replica..."
if dfx start --background 2>/dev/null; then
    echo "✅ Local replica started"
else
    echo "ℹ️  Local replica already running"
fi

# Create lab_backend canister if it doesn't exist
echo "📦 Creating lab_backend canister..."
if dfx canister create lab_backend 2>/dev/null; then
    echo "✅ lab_backend canister created"
else
    echo "ℹ️  lab_backend canister already exists"
fi

# Build lab_backend
echo "🔨 Building lab_backend..."
dfx build lab_backend

# Deploy lab_backend
echo "🚀 Deploying lab_backend..."
dfx deploy lab_backend --network local

# Get canister ID
LAB_BACKEND_ID=$(dfx canister id lab_backend --network local)

echo "✅ lab_backend deployment complete!"
echo "🆔 Lab Backend Canister ID: $LAB_BACKEND_ID"
echo "🌐 Local URL: http://localhost:4943/?canisterId=$LAB_BACKEND_ID"
