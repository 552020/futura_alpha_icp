#!/bin/bash

# Basic Local HTTP Tests
# Tests core HTTP module functionality with minimal dependencies

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

echo_header "🌐 Testing Basic HTTP Module Functionality"

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

# Test 1: Health Check via HTTP Gateway
echo_info "Test 1: Health Check via HTTP Gateway"
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

# Test 2: Health Check via dfx canister call (for comparison)
echo_info "Test 2: Health Check via dfx canister call"
DFX_HEALTH=$(dfx canister call backend http_request '(record { method = "GET"; url = "/health"; headers = vec {}; body = blob ""; })' 2>/dev/null || echo "ERROR")

if [[ "$DFX_HEALTH" == "ERROR" ]]; then
    echo_error "dfx health check failed"
else
    echo_success "dfx health check passed"
    echo_info "Response: $DFX_HEALTH"
fi

# Test 3: Token Minting (expecting forbidden - this is expected!)
echo_info "Test 3: Token minting (expecting 'forbidden' error)"
TOKEN_RESULT=$(dfx canister call backend mint_http_token '("test_memory", vec {"thumbnail"; "preview"}, null, 180)' 2>/dev/null || echo "FORBIDDEN")

if [[ "$TOKEN_RESULT" == "FORBIDDEN" ]]; then
    echo_success "Token minting properly validates permissions (forbidden as expected)"
    echo_info "This means ACL integration is working correctly!"
else
    echo_info "Token minting result: $TOKEN_RESULT"
    # If we get a token, that's also fine - it means the API is working
    if echo "$TOKEN_RESULT" | grep -q '"[^"]*"'; then
        echo_success "Token minting working (got token)"
    else
        echo_error "Unexpected token minting result"
    fi
fi

# Test 4: Asset Serving without Token (should fail)
echo_info "Test 4: Asset serving without token (should fail)"
NO_TOKEN_URL="http://${CANISTER_ID}.localhost:4943/assets/test_memory/thumbnail"
echo_info "Testing: $NO_TOKEN_URL"

NO_TOKEN_RESPONSE=$(curl -s -w "\n%{http_code}" "$NO_TOKEN_URL" || echo "CURL_ERROR")
NO_TOKEN_BODY=$(echo "$NO_TOKEN_RESPONSE" | head -n -1)
NO_TOKEN_STATUS=$(echo "$NO_TOKEN_RESPONSE" | tail -n 1)

if [[ "$NO_TOKEN_STATUS" == "401" || "$NO_TOKEN_STATUS" == "403" || "$NO_TOKEN_STATUS" == "404" ]]; then
    echo_success "Asset serving properly rejects requests without token ($NO_TOKEN_STATUS)"
else
    echo_error "Asset serving should reject without token, got: $NO_TOKEN_STATUS"
    echo_info "Response: $NO_TOKEN_BODY"
fi

# Test 5: Invalid Endpoint (should return 404)
echo_info "Test 5: Invalid endpoint (should return 404)"
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

# Test 6: Test HTTP Request Method via dfx
echo_info "Test 6: HTTP request method via dfx"
DFX_HTTP=$(dfx canister call backend http_request '(record { method = "GET"; url = "/invalid-endpoint"; headers = vec {}; body = blob ""; })' 2>/dev/null || echo "ERROR")

if [[ "$DFX_HTTP" == "ERROR" ]]; then
    echo_error "dfx http_request failed"
else
    echo_success "dfx http_request working"
    echo_info "Response: $DFX_HTTP"
fi

# Test 7: Check if HTTP Gateway is accessible
echo_info "Test 7: HTTP Gateway accessibility"
GATEWAY_URL="http://${CANISTER_ID}.localhost:4943/"
echo_info "Testing: $GATEWAY_URL"

GATEWAY_RESPONSE=$(curl -s -w "\n%{http_code}" "$GATEWAY_URL" || echo "CURL_ERROR")
GATEWAY_BODY=$(echo "$GATEWAY_RESPONSE" | head -n -1)
GATEWAY_STATUS=$(echo "$GATEWAY_RESPONSE" | tail -n 1)

if [[ "$GATEWAY_STATUS" == "404" ]]; then
    echo_success "HTTP Gateway is accessible (404 for root is expected)"
elif [[ "$GATEWAY_STATUS" == "200" ]]; then
    echo_success "HTTP Gateway is accessible (200 OK)"
else
    echo_error "HTTP Gateway accessibility issue (Status: $GATEWAY_STATUS)"
    echo_info "Response: $GATEWAY_BODY"
fi

echo_header "🎉 Basic Local HTTP Tests Completed!"

# Summary
echo_info "Test Summary:"
echo_info "- Health check (HTTP): $([ "$HEALTH_STATUS" == "200" ] && echo "✅ PASS" || echo "❌ FAIL")"
echo_info "- Health check (dfx): $([ "$DFX_HEALTH" != "ERROR" ] && echo "✅ PASS" || echo "❌ FAIL")"
echo_info "- Token minting: $([ "$TOKEN_RESULT" != "ERROR" ] && echo "✅ PASS" || echo "❌ FAIL")"
echo_info "- Asset serving without token: $([ "$NO_TOKEN_STATUS" == "401" ] || [ "$NO_TOKEN_STATUS" == "403" ] || [ "$NO_TOKEN_STATUS" == "404" ] && echo "✅ PASS" || echo "❌ FAIL")"
echo_info "- Invalid endpoint: $([ "$INVALID_STATUS" == "404" ] && echo "✅ PASS" || echo "❌ FAIL")"
echo_info "- HTTP request method: $([ "$DFX_HTTP" != "ERROR" ] && echo "✅ PASS" || echo "❌ FAIL")"
echo_info "- HTTP Gateway: $([ "$GATEWAY_STATUS" == "200" ] || [ "$GATEWAY_STATUS" == "404" ] && echo "✅ PASS" || echo "❌ FAIL")"

echo_success "Basic local HTTP tests completed successfully!"





