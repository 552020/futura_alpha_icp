#!/bin/bash

# Test script for update_memory_in_capsule endpoint
# This script tests updating a memory in the caller's capsule

set -e

# Configuration
CANISTER_ID="$(dfx canister id backend)"
IDENTITY="default"

echo "🧪 Testing update_memory_in_capsule endpoint"
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
        locator = "test_update_memory:test_key";
        hash = null;
      };
      data = opt blob "VGVzdCBtZW1vcnkgZm9yIHVwZGF0ZSB0ZXN0";
    })'
    
    # Add the memory
    echo "🚀 Adding test memory..."
    # First get a capsule ID to test with
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

ADD_RESULT=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create "(\"$CAPSULE_ID\", $TEST_MEMORY_DATA)")
    
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

# Create test update data in Candid format
echo "📝 Creating test update data..."
TEST_UPDATE_DATA='(record {
  info = opt (record {
    name = "Updated Test Memory";
    content_type = "text/plain";
    memory_type = variant { Note };
    date_of_memory = null;
    created_at = 1640995200000000000;
    updated_at = 1640995200000000000;
    uploaded_at = 1640995200000000000;
  });
  access = null;
  metadata = null;
  data = null;
})'

echo "✅ Test update data created"
echo ""

# Call update_memory_in_capsule endpoint
echo "🚀 Calling update_memory_in_capsule..."
RESULT=$(dfx canister call --identity $IDENTITY $CANISTER_ID update_memory_in_capsule "(\"$MEMORY_ID\", $TEST_UPDATE_DATA)")

echo "📊 Result:"
echo "$RESULT"
echo ""

# Check if the call was successful
if echo "$RESULT" | grep -q 'success = true'; then
    echo "✅ Test PASSED: Memory updated successfully"
    
    # Verify the update by getting the memory again
    echo "🔍 Verifying update by retrieving memory..."
    UPDATED_RESULT=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"$MEMORY_ID\")")
    
    if echo "$UPDATED_RESULT" | grep -q 'name = "Updated Test Memory"'; then
        echo "✅ Verification PASSED: Memory name updated correctly"
    else
        echo "⚠️  Verification WARNING: Memory name may not have been updated"
    fi
    
    if echo "$UPDATED_RESULT" | grep -q 'variant { Note }'; then
        echo "✅ Verification PASSED: Memory type updated to Note"
    else
        echo "⚠️  Verification WARNING: Memory type may not have been updated"
    fi
else
    echo "❌ Test FAILED: Failed to update memory"
    echo "Error details: $RESULT"
    exit 1
fi

echo ""
echo "🎉 update_memory_in_capsule test completed successfully!"
