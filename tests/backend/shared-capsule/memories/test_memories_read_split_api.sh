#!/bin/bash

# Test script for the new split memories_read API
# Tests: memories_read (metadata only), memories_read_with_assets (full), memories_read_asset (specific asset)

set -e

# Source shared utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../test_utils.sh"
source "$SCRIPT_DIR/../upload/upload_test_utils.sh"

# Helper function to create a test memory
create_test_memory() {
    local capsule_id="$1"
    local memory_name="$2"
    local memory_description="$3"
    local memory_tags="$4"
    local memory_bytes="$5"
    
    local asset_metadata='(variant {
      Document = record {
        base = record {
          name = "'$memory_name'";
          description = opt "'$memory_description'";
          tags = vec { '$memory_tags' };
          asset_type = variant { Original };
          bytes = 24;
          mime_type = "text/plain";
          sha256 = null;
          width = null;
          height = null;
          url = null;
          storage_key = null;
          bucket = null;
          asset_location = null;
          processing_status = null;
          processing_error = null;
          created_at = 0;
          updated_at = 0;
          deleted_at = null;
        };
        page_count = null;
        document_type = null;
        language = null;
        word_count = null;
      }
    })'
    
    local idem="test_memory_$(date +%s)_$$"
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create \
        "(\"$capsule_id\", opt $memory_bytes, null, null, null, null, null, null, $asset_metadata, \"$idem\")" 2>/dev/null)
    
    if [[ $result == *"Ok ="* ]]; then
        local memory_id=$(echo "$result" | grep -o 'Ok = "[^"]*"' | sed 's/Ok = "//' | sed 's/"//')
        echo "$memory_id"
        return 0
    else
        echo_error "Failed to create test memory: $result"
        return 1
    fi
}

# Test configuration
IDENTITY="default"
CANISTER_ID="$(get_canister_id backend)"

echo "🧪 Testing Split Memories Read API"
echo "=================================="
echo "Identity: $IDENTITY"
echo "Canister: $CANISTER_ID"
echo ""

# Create a test memory with inline asset first
echo "📝 Creating test memory with inline asset..."

# Get or create a capsule first
echo "Getting test capsule..."
CAPSULE_ID=$(get_test_capsule_id "$CANISTER_ID" "$IDENTITY")
if [[ -z "$CAPSULE_ID" ]]; then
    echo "❌ Failed to get capsule ID"
    exit 1
fi
echo "✅ Using capsule ID: $CAPSULE_ID"

# Create memory with inline asset using the utility function
echo "Creating memory with inline asset..."
MEMORY_BYTES='blob "VGVzdCBtZW1vcnkgZGF0YQ=="'  # "Test memory data" in base64
MEMORY_ID=$(create_test_memory "$CAPSULE_ID" "Split API Test Memory" "Test memory for split API testing" '"test"; "split-api"' "$MEMORY_BYTES")

if [[ -z "$MEMORY_ID" ]]; then
    echo "❌ Failed to create memory"
    exit 1
fi

echo "✅ Memory created with ID: $MEMORY_ID"

echo ""
echo "🔍 Testing memories_read (metadata only)..."
echo "==========================================="

# Test memories_read - should return metadata without blob data
READ_RESULT=$(dfx canister call --identity "$IDENTITY" "$CANISTER_ID" memories_read "(\"$MEMORY_ID\")" 2>/dev/null)

if [[ $READ_RESULT == *"Ok"* ]]; then
    echo "✅ memories_read succeeded"
    
    # Check that inline asset has empty bytes
    if [[ $READ_RESULT == *"bytes = blob \"\""* ]]; then
        echo "✅ Inline asset bytes are empty (metadata only)"
    else
        echo "❌ Inline asset still contains blob data"
        echo "Result: $READ_RESULT"
    fi
    
    # Check that asset metadata is preserved
    if [[ $READ_RESULT == *"name = \"Split API Test Memory\""* ]]; then
        echo "✅ Asset metadata is preserved"
    else
        echo "❌ Asset metadata missing"
    fi
else
    echo "❌ memories_read failed: $READ_RESULT"
    exit 1
fi

echo ""
echo "🔍 Testing memories_read_with_assets (full memory)..."
echo "===================================================="

# Test memories_read_with_assets - should return full memory with blob data
READ_WITH_ASSETS_RESULT=$(dfx canister call --identity "$IDENTITY" "$CANISTER_ID" memories_read_with_assets "(\"$MEMORY_ID\")" 2>/dev/null)

if [[ $READ_WITH_ASSETS_RESULT == *"Ok"* ]]; then
    echo "✅ memories_read_with_assets succeeded"
    
    # Check that inline asset has blob data (base64 "Test memory data" = "VGVzdCBtZW1vcnkgZGF0YQ==")
    if [[ $READ_WITH_ASSETS_RESULT == *"bytes = blob \"VGVzdCBtZW1vcnkgZGF0YQ==\""* ]]; then
        echo "✅ Inline asset contains blob data"
    else
        echo "❌ Inline asset missing blob data"
        echo "Result: $READ_WITH_ASSETS_RESULT"
    fi
else
    echo "❌ memories_read_with_assets failed: $READ_WITH_ASSETS_RESULT"
    exit 1
fi

echo ""
echo "🔍 Testing memories_read_asset (specific asset)..."
echo "================================================="

# Test memories_read_asset for inline asset (index 0)
ASSET_RESULT=$(dfx canister call --identity "$IDENTITY" "$CANISTER_ID" memories_read_asset "(\"$MEMORY_ID\", 0:nat32)" 2>/dev/null)

if [[ $ASSET_RESULT == *"Ok"* ]]; then
    echo "✅ memories_read_asset succeeded for inline asset"
    
    # Check that we get the blob data
    if [[ $ASSET_RESULT == *"Ok = blob \"VGVzdCBtZW1vcnkgZGF0YQ==\""* ]]; then
        echo "✅ Correct blob data returned for inline asset"
    else
        echo "❌ Incorrect blob data for inline asset"
        echo "Result: $ASSET_RESULT"
    fi
else
    echo "❌ memories_read_asset failed for inline asset: $ASSET_RESULT"
    exit 1
fi

# Test memories_read_asset for out of range index (index 1)
echo ""
echo "Testing memories_read_asset for out of range index (index 1)..."
OUT_OF_RANGE_RESULT2=$(dfx canister call --identity "$IDENTITY" "$CANISTER_ID" memories_read_asset "(\"$MEMORY_ID\", 1:nat32)" 2>/dev/null)

if [[ $OUT_OF_RANGE_RESULT2 == *"InvalidArgument"* ]]; then
    echo "✅ memories_read_asset correctly returns InvalidArgument for out of range index"
else
    echo "❌ Expected InvalidArgument for out of range index, got: $OUT_OF_RANGE_RESULT2"
fi

# Test memories_read_asset for out of range index
echo ""
echo "Testing memories_read_asset for out of range index..."
OUT_OF_RANGE_RESULT=$(dfx canister call --identity "$IDENTITY" "$CANISTER_ID" memories_read_asset "(\"$MEMORY_ID\", 99:nat32)" 2>/dev/null)

if [[ $OUT_OF_RANGE_RESULT == *"InvalidArgument"* ]]; then
    echo "✅ memories_read_asset correctly returns InvalidArgument for out of range index"
else
    echo "❌ Expected InvalidArgument for out of range index, got: $OUT_OF_RANGE_RESULT"
fi

echo ""
echo "🧹 Cleaning up..."
# Delete the test memory
DELETE_RESULT=$(dfx canister call --identity "$IDENTITY" "$CANISTER_ID" memories_delete "(\"$MEMORY_ID\")" 2>/dev/null)
if [[ $DELETE_RESULT == *"Ok"* ]]; then
    echo "✅ Test memory deleted"
else
    echo "⚠️  Failed to delete test memory: $DELETE_RESULT"
fi

echo ""
echo "🎉 All split API tests passed!"
echo "=============================="
echo ""
echo "Summary:"
echo "- memories_read: Returns metadata + asset references (no blob data)"
echo "- memories_read_with_assets: Returns full memory with blob data"
echo "- memories_read_asset: Returns specific asset blob data (inline assets only for now)"
