#!/bin/bash

# Test script for memories_create endpoint
# This script tests adding a memory to the caller's capsule

set -e

# Configuration
CANISTER_ID="$(dfx canister id backend)"
IDENTITY="default"

echo "🧪 Testing memories_create endpoint"
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

# Call memories_create endpoint with the capsule ID
echo "🚀 Calling memories_create..."
RESULT=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$CAPSULE_ID\", $TEST_MEMORY_DATA)")

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
echo "🎉 memories_create test completed successfully!"
