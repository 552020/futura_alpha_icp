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

# Create test memory data
echo "📝 Creating test memory data..."
cat > /tmp/test_memory_data.json << 'EOF'
{
  "blob_ref": {
    "kind": {
      "ICPCapsule": null
    },
    "locator": "test_canister:test_key",
    "hash": null
  },
  "data": "SGVsbG8gV29ybGQ="
}
EOF

echo "✅ Test memory data created"
echo ""

# Call add_memory_to_capsule endpoint
echo "🚀 Calling add_memory_to_capsule..."
RESULT=$(dfx canister call --identity $IDENTITY $CANISTER_ID add_memory_to_capsule "$(cat /tmp/test_memory_data.json)")

echo "📊 Result:"
echo "$RESULT"
echo ""

# Check if the call was successful
if echo "$RESULT" | grep -q '"success" : true'; then
    echo "✅ Test PASSED: Memory added successfully"
    
    # Extract memory ID for further testing
    MEMORY_ID=$(echo "$RESULT" | grep -o '"memory_id" : "[^"]*"' | cut -d'"' -f4)
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
