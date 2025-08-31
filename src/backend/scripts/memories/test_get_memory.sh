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

# Check if we have a memory ID from previous test, if not create one
if [ ! -f /tmp/test_memory_id.txt ]; then
    echo "📝 No existing memory ID found. Creating a test memory first..."
    
    # Register user first (required for memory operations)
    echo "👤 Registering user..."
    REGISTER_RESULT=$(dfx canister call --identity $IDENTITY $CANISTER_ID register)
    echo "Registration result: $REGISTER_RESULT"
    
    # Create test memory data in Candid format
    TEST_MEMORY_DATA='(record {
      blob_ref = record {
        kind = variant { ICPCapsule };
        locator = "test_get_memory:test_key";
        hash = null;
      };
      data = opt blob "VGVzdCBtZW1vcnkgZm9yIGdldCB0ZXN0";
    })'
    
    # Add the memory
    echo "🚀 Adding test memory..."
    ADD_RESULT=$(dfx canister call --identity $IDENTITY $CANISTER_ID add_memory_to_capsule "$TEST_MEMORY_DATA")
    
    if echo "$ADD_RESULT" | grep -q 'success = true'; then
        MEMORY_ID=$(echo "$ADD_RESULT" | grep -o 'memory_id = opt "[^"]*"' | sed 's/memory_id = opt "\([^"]*\)"/\1/')
        echo "$MEMORY_ID" > /tmp/test_memory_id.txt
        echo "✅ Test memory created with ID: $MEMORY_ID"
    else
        echo "❌ Failed to create test memory: $ADD_RESULT"
        exit 1
    fi
else
    MEMORY_ID=$(cat /tmp/test_memory_id.txt)
fi
echo "📋 Using Memory ID: $MEMORY_ID"
echo ""

# Call get_memory_from_capsule endpoint
echo "🚀 Calling get_memory_from_capsule..."
RESULT=$(dfx canister call --identity $IDENTITY $CANISTER_ID get_memory_from_capsule "(\"$MEMORY_ID\")")

echo "📊 Result:"
echo "$RESULT"
echo ""

# Check if the call was successful
if echo "$RESULT" | grep -q "opt record" && echo "$RESULT" | grep -q "id = \"$MEMORY_ID\""; then
    echo "✅ Test PASSED: Memory retrieved successfully"
    
    # Extract some memory details for verification (using Candid format)
    if echo "$RESULT" | grep -q "name = "; then
        MEMORY_NAME=$(echo "$RESULT" | grep -o 'name = "[^"]*"' | sed 's/name = "\([^"]*\)"/\1/')
        echo "📋 Memory Name: $MEMORY_NAME"
    fi
    
    if echo "$RESULT" | grep -q "memory_type = "; then
        MEMORY_TYPE=$(echo "$RESULT" | grep -o 'memory_type = variant { [^}]*}' | sed 's/memory_type = variant { \([^}]*\) }/\1/')
        echo "📋 Memory Type: $MEMORY_TYPE"
    fi
elif echo "$RESULT" | grep -q "(null)"; then
    echo "❌ Test FAILED: Memory not found (returned null)"
    exit 1
else
    echo "❌ Test FAILED: Failed to retrieve memory"
    echo "Error details: $RESULT"
    exit 1
fi

echo ""
echo "🎉 get_memory_from_capsule test completed successfully!"
