#!/bin/bash

# Test script for add_memory_to_capsule endpoint
# This script tests adding a memory to the caller's capsule

set -e

# Configuration
CANISTER_ID="$(dfx canister id backend)"
IDENTITY="default"

echo "🧪 Testing add_memory_to_capsule endpoint"
echo "Canister ID: $CANISTER_ID"
echo "Identity: $IDENTITY"
echo ""

# Register user first (required for memory operations)
echo "👤 Registering user..."
REGISTER_RESULT=$(dfx canister call --identity $IDENTITY $CANISTER_ID register)
echo "Registration result: $REGISTER_RESULT"
echo ""

# Create test memory data in Candid format
echo "📝 Creating test memory data..."
TEST_MEMORY_DATA='(record {
  blob_ref = record {
    kind = variant { ICPCapsule };
    locator = "test_canister:test_key";
    hash = null;
  };
  data = opt blob "SGVsbG8gV29ybGQ=";
})'

echo "✅ Test memory data created"
echo ""

# Call add_memory_to_capsule endpoint
echo "🚀 Calling add_memory_to_capsule..."
RESULT=$(dfx canister call --identity $IDENTITY $CANISTER_ID add_memory_to_capsule "$TEST_MEMORY_DATA")

echo "📊 Result:"
echo "$RESULT"
echo ""

# Check if the call was successful
if echo "$RESULT" | grep -q 'success = true'; then
    echo "✅ Test PASSED: Memory added successfully"
    
    # Extract memory ID for further testing
    MEMORY_ID=$(echo "$RESULT" | grep -o 'memory_id = opt "[^"]*"' | sed 's/memory_id = opt "\([^"]*\)"/\1/')
    echo "📋 Memory ID: $MEMORY_ID"
    
    # Save memory ID for other tests
    echo "$MEMORY_ID" > /tmp/test_memory_id.txt
else
    echo "❌ Test FAILED: Failed to add memory"
    echo "Error details: $RESULT"
    exit 1
fi

echo ""
echo "🎉 add_memory_to_capsule test completed successfully!"
