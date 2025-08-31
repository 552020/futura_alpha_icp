#!/bin/bash

# Test script for delete_memory_from_capsule endpoint
# This script tests deleting a memory from the caller's capsule

set -e

# Configuration
CANISTER_ID="$(dfx canister id backend)"
IDENTITY="default"

echo "🧪 Testing delete_memory_from_capsule endpoint"
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

# Call delete_memory_from_capsule endpoint
echo "🚀 Calling delete_memory_from_capsule..."
RESULT=$(dfx canister call --identity $IDENTITY $CANISTER_ID delete_memory_from_capsule "(\"$MEMORY_ID\")")

echo "📊 Result:"
echo "$RESULT"
echo ""

# Check if the call was successful
if echo "$RESULT" | grep -q '"success" : true'; then
    echo "✅ Test PASSED: Memory deleted successfully"
    
    # Verify deletion by trying to get the memory again
    echo "🔍 Verifying deletion by attempting to retrieve memory..."
    GET_RESULT=$(dfx canister call --identity $IDENTITY $CANISTER_ID get_memory_from_capsule "(\"$MEMORY_ID\")")
    
    if echo "$GET_RESULT" | grep -q "null"; then
        echo "✅ Verification PASSED: Memory successfully deleted (returns null)"
    else
        echo "⚠️  Verification WARNING: Memory may still exist"
        echo "Get result: $GET_RESULT"
    fi
    
    # Clean up the memory ID file
    rm -f /tmp/test_memory_id.txt
    echo "🧹 Cleaned up test memory ID file"
else
    echo "❌ Test FAILED: Failed to delete memory"
    echo "Error details: $RESULT"
    exit 1
fi

echo ""
echo "🎉 delete_memory_from_capsule test completed successfully!"
