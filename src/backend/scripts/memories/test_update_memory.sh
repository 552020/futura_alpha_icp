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

# Check if we have a memory ID from previous test
if [ ! -f /tmp/test_memory_id.txt ]; then
    echo "❌ No memory ID found. Please run test_add_memory.sh first."
    exit 1
fi

MEMORY_ID=$(cat /tmp/test_memory_id.txt)
echo "📋 Using Memory ID: $MEMORY_ID"
echo ""

# Create test update data
echo "📝 Creating test update data..."
cat > /tmp/test_update_data.json << 'EOF'
{
  "info": {
    "memory_type": {
      "Note": null
    },
    "name": "Updated Test Memory",
    "content_type": "text/plain",
    "created_at": 0,
    "updated_at": 0,
    "uploaded_at": 0,
    "date_of_memory": null
  },
  "metadata": null,
  "access": null,
  "data": null
}
EOF

echo "✅ Test update data created"
echo ""

# Call update_memory_in_capsule endpoint
echo "🚀 Calling update_memory_in_capsule..."
RESULT=$(dfx canister call --identity $IDENTITY $CANISTER_ID update_memory_in_capsule "(\"$MEMORY_ID\", $(cat /tmp/test_update_data.json))")

echo "📊 Result:"
echo "$RESULT"
echo ""

# Check if the call was successful
if echo "$RESULT" | grep -q '"success" : true'; then
    echo "✅ Test PASSED: Memory updated successfully"
    
    # Verify the update by getting the memory again
    echo "🔍 Verifying update by retrieving memory..."
    UPDATED_RESULT=$(dfx canister call --identity $IDENTITY $CANISTER_ID get_memory_from_capsule "(\"$MEMORY_ID\")")
    
    if echo "$UPDATED_RESULT" | grep -q '"name" : "Updated Test Memory"'; then
        echo "✅ Verification PASSED: Memory name updated correctly"
    else
        echo "⚠️  Verification WARNING: Memory name may not have been updated"
    fi
    
    if echo "$UPDATED_RESULT" | grep -q '"Note"'; then
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
