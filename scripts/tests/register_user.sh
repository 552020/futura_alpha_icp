#!/bin/bash

# Simple user registration script
# This script registers the current user with the backend canister

set -e

# Configuration
CANISTER_ID="$(dfx canister id backend)"
IDENTITY="default"

echo "👤 Registering user with backend canister"
echo "Canister ID: $CANISTER_ID"
echo "Identity: $IDENTITY"
echo ""

# Register user
echo "🚀 Calling register..."
RESULT=$(dfx canister call --identity $IDENTITY $CANISTER_ID register)

echo "📊 Result: $RESULT"
echo ""

# Check if registration was successful
if echo "$RESULT" | grep -q "true"; then
    echo "✅ User registration successful!"
else
    echo "❌ User registration failed or user already registered"
fi

echo ""
echo "🎉 Registration completed!"