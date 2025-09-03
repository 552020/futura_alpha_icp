#!/bin/bash

# Test script for memories_list endpoint
# This script tests listing all memories in the caller's capsule

set -e

# Configuration
CANISTER_ID="$(dfx canister id backend)"
IDENTITY="default"

echo "🧪 Testing memories_list endpoint"
echo "Canister ID: $CANISTER_ID"
echo "Identity: $IDENTITY"
echo ""

# First get a capsule ID to test with
echo "🚀 Getting capsule ID for testing..."
CAPSULE_RESULT=$(dfx canister call --identity $IDENTITY $CANISTER_ID capsules_read_basic "(null)")

if [[ $CAPSULE_RESULT == *"null"* ]]; then
    echo "No capsule found, creating one first..."
    CREATE_RESULT=$(dfx canister call --identity $IDENTITY $CANISTER_ID capsules_create "(null)")
            CAPSULE_ID=$(echo "$CREATE_RESULT" | grep -o 'capsule_id = opt "[^"]*"' | sed 's/capsule_id = opt "//' | sed 's/"//')
else
    CAPSULE_ID=$(echo "$CAPSULE_RESULT" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')
fi

if [[ -z "$CAPSULE_ID" ]]; then
    echo "❌ Failed to get capsule ID for testing"
    exit 1
fi

echo "Testing with capsule ID: $CAPSULE_ID"

# Call memories_list endpoint with the capsule ID
echo "🚀 Calling memories_list..."
RESULT=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_list "(\"$CAPSULE_ID\")")

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
echo "🎉 memories_list test completed successfully!"
