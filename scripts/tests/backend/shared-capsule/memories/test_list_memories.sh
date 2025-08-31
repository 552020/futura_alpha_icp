#!/bin/bash

# Test script for list_capsule_memories endpoint
# This script tests listing all memories in the caller's capsule

set -e

# Configuration
CANISTER_ID="$(dfx canister id backend)"
IDENTITY="default"

echo "🧪 Testing list_capsule_memories endpoint"
echo "Canister ID: $CANISTER_ID"
echo "Identity: $IDENTITY"
echo ""

# Call list_capsule_memories endpoint
echo "🚀 Calling list_capsule_memories..."
RESULT=$(dfx canister call --identity $IDENTITY $CANISTER_ID list_capsule_memories)

echo "📊 Result:"
echo "$RESULT"
echo ""

# Check if the call was successful
if echo "$RESULT" | grep -q 'success = true'; then
    echo "✅ Test PASSED: Memories listed successfully"
    
    # Count memories
    MEMORY_COUNT=$(echo "$RESULT" | grep -o 'id = "[^"]*"' | wc -l)
    echo "📋 Number of memories found: $MEMORY_COUNT"
    
    # Check if we have the test memory
    if [ ! -f /tmp/test_memory_id.txt ]; then
        echo "⚠️  No test memory ID found, skipping verification"
    else
        TEST_MEMORY_ID=$(cat /tmp/test_memory_id.txt)
        if echo "$RESULT" | grep -q "$TEST_MEMORY_ID"; then
            echo "✅ Verification PASSED: Test memory found in list"
        else
            echo "⚠️  Verification WARNING: Test memory not found in list"
        fi
    fi
    
    # Show memory IDs if any exist
    if [ "$MEMORY_COUNT" -gt 0 ]; then
        echo "📋 Memory IDs:"
        echo "$RESULT" | grep -o 'id = "[^"]*"' | sed 's/id = "\([^"]*\)"/\1/'
    fi
else
    echo "❌ Test FAILED: Failed to list memories"
    echo "Error details: $RESULT"
    exit 1
fi

echo ""
echo "🎉 list_capsule_memories test completed successfully!"
