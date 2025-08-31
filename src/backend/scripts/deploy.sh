#!/bin/bash

# Deployment script for backend canister
# This script builds and deploys the backend canister

set -e

echo "🚀 Starting backend deployment..."

# Check if dfx is running
if ! dfx ping 2>/dev/null; then
    echo "❌ dfx is not running. Starting dfx..."
    dfx start --clean --background
    sleep 5
fi

# Build the canister
echo "🔨 Building backend canister..."
dfx build backend

# Deploy the canister
echo "📦 Deploying backend canister..."
dfx deploy backend

echo "✅ Backend deployment completed successfully!"
echo "🎯 Canister ID: $(dfx canister id backend)"
