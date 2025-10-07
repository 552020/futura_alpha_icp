#!/bin/bash

# Deploy script for lab_backend to mainnet
# This script handles the complete lab_backend deployment process to IC mainnet

set -e

echo "🚀 Deploying lab_backend to mainnet..."

# Ensure we're in the project root
cd "$(dirname "$0")/.."

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

# Deploy lab_backend to mainnet
echo "🚀 Deploying lab_backend to mainnet..."
dfx deploy lab_backend --network ic

# Get canister ID
LAB_BACKEND_ID=$(dfx canister id lab_backend --network ic)

echo "✅ lab_backend mainnet deployment complete!"
echo "🆔 Lab Backend Canister ID: $LAB_BACKEND_ID"
echo "🌐 Mainnet URL: https://$LAB_BACKEND_ID.ic0.app"
