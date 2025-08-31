#!/bin/bash

# Test script for get_memory_from_capsule endpoint
# This script tests retrieving a memory from the caller's capsule

set -e

# Configuration
CANISTER_ID="$(dfx canister id backend)"
IDENTITY="default"

echo "🧪 Testing get_memory_from_capsule endpoint"
echo "Canister ID: $CANISTER_ID"
echo "Identity: $IDENTITY"
echo ""

# Check if we have a memory ID from previous test
if [ ! -f /tmp/test_memory_id.txt ]; then
    echo "❌ No memory ID found. Please run test_add_memory.sh first."
    exit 1
fi

MEMORY_ID=$(cat /tmp/test_memory_id.txt)
echo "📋 Using Memory ID: $MEMORY_ID"
echo ""

# Call get_memory_from_capsule endpoint
echo "🚀 Calling get_memory_from_capsule..."
RESULT=$(dfx canister call --identity $IDENTITY $CANISTER_ID get_memory_from_capsule "(\"$MEMORY_ID\")")

echo "📊 Result:"
echo "$RESULT"
echo ""

# Check if the call was successful
if echo "$RESULT" | grep -q '"id" : "'$MEMORY_ID'"'; then
    echo "✅ Test PASSED: Memory retrieved successfully"
    
    # Extract some memory details for verification
    MEMORY_NAME=$(echo "$RESULT" | grep -o '"name" : "[^"]*"' | cut -d'"' -f4)
    MEMORY_TYPE=$(echo "$RESULT" | grep -o '"memory_type" : "[^"]*"' | cut -d'"' -f4)
    
    echo "📋 Memory Name: $MEMORY_NAME"
    echo "📋 Memory Type: $MEMORY_TYPE"
else
    echo "❌ Test FAILED: Failed to retrieve memory"
    echo "Error details: $RESULT"
    exit 1
fi

echo ""
echo "🎉 get_memory_from_capsule test completed successfully!"
