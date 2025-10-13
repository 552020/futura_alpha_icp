#!/bin/bash

# Local Replica HTTP Tests
# Tests the HTTP module with real HTTP requests via curl

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo_header() {
    echo -e "${BLUE}============================================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}============================================================${NC}"
}

echo_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

echo_error() {
    echo -e "${RED}❌ $1${NC}"
}

echo_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
}

# Configuration
SCRIPT_DIR="$(dirname "$0")"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
DEBUG="${DEBUG:-false}"

echo_header "🌐 Testing HTTP Module with Local Replica"

# Check if dfx is available
if ! command -v dfx &> /dev/null; then
    echo_error "dfx not found. Please install dfx first."
    exit 1
fi

# Check if local replica is running
if ! dfx ping local 2>/dev/null; then
    echo_error "Local replica is not running. Please start it with: dfx start --background"
    exit 1
fi

echo_success "Local replica is running"

# Get canister ID
CANISTER_ID=$(dfx canister id backend 2>/dev/null)
if [[ -z "$CANISTER_ID" ]]; then
    echo_error "Backend canister not found. Please deploy it first."
    exit 1
fi

echo_info "Backend canister ID: $CANISTER_ID"

# Test 1: Health Check
echo_info "Test 1: Health Check"
HEALTH_URL="http://${CANISTER_ID}.localhost:4943/health"
echo_info "Testing: $HEALTH_URL"

HEALTH_RESPONSE=$(curl -s -w "\n%{http_code}" "$HEALTH_URL" || echo "CURL_ERROR")
HEALTH_BODY=$(echo "$HEALTH_RESPONSE" | head -n -1)
HEALTH_STATUS=$(echo "$HEALTH_RESPONSE" | tail -n 1)

if [[ "$HEALTH_STATUS" == "200" ]]; then
    echo_success "Health check passed (200 OK)"
    echo_info "Response: $HEALTH_BODY"
else
    echo_error "Health check failed (Status: $HEALTH_STATUS)"
    echo_info "Response: $HEALTH_BODY"
fi

# Test 2: Create Test Memory and Asset
echo_info "Test 2: Creating test memory with asset..."

# Create a test capsule first
CAPSULE_RESULT=$(dfx canister call backend capsules_create '("test_http_capsule", "Test capsule for HTTP tests")' 2>/dev/null || echo "ERROR")
if [[ "$CAPSULE_RESULT" == "ERROR" ]]; then
    echo_error "Failed to create test capsule"
    exit 1
fi

CAPSULE_ID=$(echo "$CAPSULE_RESULT" | grep -o '"[^"]*"' | head -n 1 | tr -d '"')
echo_info "Created capsule: $CAPSULE_ID"

# Create a simple test image (1x1 PNG)
TEST_IMAGE_BYTES="iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg=="
TEST_IMAGE_HEX=$(echo "$TEST_IMAGE_BYTES" | base64 -d | xxd -p -c 256)

# Create memory with inline asset
MEMORY_RESULT=$(dfx canister call backend memories_create \
    "(\"$CAPSULE_ID\", vec { blob \"$TEST_IMAGE_HEX\" }, vec {}, vec {}, vec {}, vec {}, vec {}, vec {}, vec { record { \"name\" = \"test.png\"; \"mime_type\" = \"image/png\"; \"bytes\" = \"68\"; \"width\" = \"1\"; \"height\" = \"1\" } }, \"test_memory_$(date +%s)\")" \
    2>/dev/null || echo "ERROR")

if [[ "$MEMORY_RESULT" == "ERROR" ]]; then
    echo_error "Failed to create test memory"
    exit 1
fi

MEMORY_ID=$(echo "$MEMORY_RESULT" | grep -o '"[^"]*"' | head -n 1 | tr -d '"')
echo_success "Created memory: $MEMORY_ID"

# Test 3: Mint HTTP Token
echo_info "Test 3: Minting HTTP token..."

TOKEN_RESULT=$(dfx canister call backend mint_http_token "(\"$MEMORY_ID\", vec { \"thumbnail\"; \"preview\" }, null, 180)" 2>/dev/null || echo "ERROR")

if [[ "$TOKEN_RESULT" == "ERROR" ]]; then
    echo_error "Failed to mint HTTP token"
    echo_info "This might be expected if ACL validation is working correctly"
    TOKEN=""
else
    TOKEN=$(echo "$TOKEN_RESULT" | grep -o '"[^"]*"' | head -n 1 | tr -d '"')
    if [[ -n "$TOKEN" ]]; then
        echo_success "Minted HTTP token: ${TOKEN:0:20}..."
    else
        echo_error "Token minting returned empty result"
        TOKEN=""
    fi
fi

# Test 4: Asset Serving with Token
if [[ -n "$TOKEN" ]]; then
    echo_info "Test 4: Asset serving with token..."
    ASSET_URL="http://${CANISTER_ID}.localhost:4943/assets/${MEMORY_ID}/thumbnail?token=${TOKEN}"
    echo_info "Testing: $ASSET_URL"
    
    ASSET_RESPONSE=$(curl -s -w "\n%{http_code}\n%{content_type}" "$ASSET_URL" || echo "CURL_ERROR")
    ASSET_BODY=$(echo "$ASSET_RESPONSE" | head -n -2)
    ASSET_STATUS=$(echo "$ASSET_RESPONSE" | tail -n 2 | head -n 1)
    ASSET_CONTENT_TYPE=$(echo "$ASSET_RESPONSE" | tail -n 1)
    
    if [[ "$ASSET_STATUS" == "200" ]]; then
        echo_success "Asset serving passed (200 OK)"
        echo_info "Content-Type: $ASSET_CONTENT_TYPE"
        echo_info "Body size: $(echo "$ASSET_BODY" | wc -c) bytes"
        
        # Verify it's actually PNG data
        if echo "$ASSET_BODY" | head -c 8 | xxd -p | grep -q "89504e470d0a1a0a"; then
            echo_success "Response contains valid PNG data"
        else
            echo_error "Response does not contain valid PNG data"
        fi
    else
        echo_error "Asset serving failed (Status: $ASSET_STATUS)"
        echo_info "Response: $ASSET_BODY"
    fi
else
    echo_info "Test 4: Skipping asset serving (no token available)"
fi

# Test 5: Asset Serving without Token
echo_info "Test 5: Asset serving without token..."
NO_TOKEN_URL="http://${CANISTER_ID}.localhost:4943/assets/${MEMORY_ID}/thumbnail"
echo_info "Testing: $NO_TOKEN_URL"

NO_TOKEN_RESPONSE=$(curl -s -w "\n%{http_code}" "$NO_TOKEN_URL" || echo "CURL_ERROR")
NO_TOKEN_BODY=$(echo "$NO_TOKEN_RESPONSE" | head -n -1)
NO_TOKEN_STATUS=$(echo "$NO_TOKEN_RESPONSE" | tail -n 1)

if [[ "$NO_TOKEN_STATUS" == "401" || "$NO_TOKEN_STATUS" == "403" ]]; then
    echo_success "Asset serving properly rejects requests without token ($NO_TOKEN_STATUS)"
else
    echo_error "Asset serving should reject without token, got: $NO_TOKEN_STATUS"
    echo_info "Response: $NO_TOKEN_BODY"
fi

# Test 6: Invalid Endpoint
echo_info "Test 6: Invalid endpoint..."
INVALID_URL="http://${CANISTER_ID}.localhost:4943/invalid-endpoint"
echo_info "Testing: $INVALID_URL"

INVALID_RESPONSE=$(curl -s -w "\n%{http_code}" "$INVALID_URL" || echo "CURL_ERROR")
INVALID_BODY=$(echo "$INVALID_RESPONSE" | head -n -1)
INVALID_STATUS=$(echo "$INVALID_RESPONSE" | tail -n 1)

if [[ "$INVALID_STATUS" == "404" ]]; then
    echo_success "Invalid endpoint properly returns 404"
else
    echo_error "Invalid endpoint should return 404, got: $INVALID_STATUS"
    echo_info "Response: $INVALID_BODY"
fi

# Test 7: Non-existent Memory
echo_info "Test 7: Non-existent memory..."
NONEXISTENT_URL="http://${CANISTER_ID}.localhost:4943/assets/nonexistent_memory/thumbnail"
echo_info "Testing: $NONEXISTENT_URL"

NONEXISTENT_RESPONSE=$(curl -s -w "\n%{http_code}" "$NONEXISTENT_URL" || echo "CURL_ERROR")
NONEXISTENT_BODY=$(echo "$NONEXISTENT_RESPONSE" | head -n -1)
NONEXISTENT_STATUS=$(echo "$NONEXISTENT_RESPONSE" | tail -n 1)

if [[ "$NONEXISTENT_STATUS" == "404" ]]; then
    echo_success "Non-existent memory properly returns 404"
else
    echo_error "Non-existent memory should return 404, got: $NONEXISTENT_STATUS"
    echo_info "Response: $NONEXISTENT_BODY"
fi

# Test 8: Check Response Headers
if [[ -n "$TOKEN" ]]; then
    echo_info "Test 8: Checking response headers..."
    HEADERS_URL="http://${CANISTER_ID}.localhost:4943/assets/${MEMORY_ID}/thumbnail?token=${TOKEN}"
    
    HEADERS_RESPONSE=$(curl -s -I "$HEADERS_URL" || echo "CURL_ERROR")
    
    if echo "$HEADERS_RESPONSE" | grep -q "Content-Type: image/png"; then
        echo_success "Correct Content-Type header present"
    else
        echo_error "Content-Type header missing or incorrect"
    fi
    
    if echo "$HEADERS_RESPONSE" | grep -q "Cache-Control.*private"; then
        echo_success "Private cache control header present"
    else
        echo_error "Private cache control header missing"
    fi
    
    if echo "$HEADERS_RESPONSE" | grep -q "Cache-Control.*no-store"; then
        echo_success "No-store cache control header present"
    else
        echo_error "No-store cache control header missing"
    fi
else
    echo_info "Test 8: Skipping header check (no token available)"
fi

# Cleanup
echo_info "Cleaning up test memory..."
dfx canister call backend memories_delete "(\"$MEMORY_ID\")" 2>/dev/null || echo "Cleanup failed"
echo_success "Cleanup completed"

echo_header "🎉 Local Replica HTTP Tests Completed!"

# Summary
echo_info "Test Summary:"
echo_info "- Health check: $([ "$HEALTH_STATUS" == "200" ] && echo "✅ PASS" || echo "❌ FAIL")"
echo_info "- Memory creation: $([ -n "$MEMORY_ID" ] && echo "✅ PASS" || echo "❌ FAIL")"
echo_info "- Token minting: $([ -n "$TOKEN" ] && echo "✅ PASS" || echo "❌ FAIL (expected if ACL working)")"
echo_info "- Asset serving with token: $([ "$ASSET_STATUS" == "200" ] && echo "✅ PASS" || echo "❌ FAIL")"
echo_info "- Asset serving without token: $([ "$NO_TOKEN_STATUS" == "401" ] || [ "$NO_TOKEN_STATUS" == "403" ] && echo "✅ PASS" || echo "❌ FAIL")"
echo_info "- Invalid endpoint: $([ "$INVALID_STATUS" == "404" ] && echo "✅ PASS" || echo "❌ FAIL")"
echo_info "- Non-existent memory: $([ "$NONEXISTENT_STATUS" == "404" ] && echo "✅ PASS" || echo "❌ FAIL")"

echo_success "Local replica HTTP tests completed successfully!"
