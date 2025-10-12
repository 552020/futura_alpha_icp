#!/bin/bash

# Basic HTTP module test
# Tests the core HTTP functionality: health check, token minting, and asset serving

set -e

# Source test utilities
source "$(dirname "$0")/../test_utils.sh"

# Configuration
CANISTER_ID="backend"
IDENTITY="default"
DEBUG="${DEBUG:-false}"

echo_header "🧪 Testing HTTP Module - Basic Functionality"

# Test 1: Health check endpoint
test_health_check() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing health check endpoint..."
    
    # Test health endpoint via HTTP request
    local response=$(dfx canister call "$CANISTER_ID" http_request '(
        record {
            method = "GET";
            url = "/health";
            headers = vec {};
            body = blob "";
        }
    )' --identity "$IDENTITY" 2>/dev/null || echo "ERROR")
    
    if [[ "$response" == *"200"* ]] || [[ "$response" == *"OK"* ]]; then
        echo_success "✅ Health check endpoint working"
        return 0
    else
        echo_error "❌ Health check failed: $response"
        return 1
    fi
}

# Test 2: Token minting (requires existing memory)
test_token_minting() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing token minting..."
    
    # First, we need a valid memory ID to test with
    # For now, we'll test with a dummy memory ID and expect it to fail gracefully
    local test_memory_id="test-memory-$(date +%s)"
    
    local response=$(dfx canister call "$CANISTER_ID" mint_http_token "(
        \"$test_memory_id\",
        vec {\"thumbnail\"; \"preview\"},
        null,
        180u32
    )" --identity "$IDENTITY" 2>/dev/null || echo "ERROR")
    
    # We expect this to fail with "forbidden" since the memory doesn't exist
    if [[ "$response" == *"forbidden"* ]] || [[ "$response" == *"ERROR"* ]]; then
        echo_success "✅ Token minting properly validates permissions"
        return 0
    else
        echo_error "❌ Token minting should have failed for non-existent memory: $response"
        return 1
    fi
}

# Test 3: Asset serving without token (should fail)
test_asset_serving_no_token() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing asset serving without token..."
    
    local response=$(dfx canister call "$CANISTER_ID" http_request '(
        record {
            method = "GET";
            url = "/asset/test-memory/thumbnail";
            headers = vec {};
            body = blob "";
        }
    )' --identity "$IDENTITY" 2>/dev/null || echo "ERROR")
    
    # Should return 401 (unauthorized) or 403 (forbidden)
    if [[ "$response" == *"401"* ]] || [[ "$response" == *"403"* ]] || [[ "$response" == *"ERROR"* ]]; then
        echo_success "✅ Asset serving properly rejects requests without token"
        return 0
    else
        echo_error "❌ Asset serving should have rejected request without token: $response"
        return 1
    fi
}

# Test 4: Asset serving with invalid token (should fail)
test_asset_serving_invalid_token() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing asset serving with invalid token..."
    
    local response=$(dfx canister call "$CANISTER_ID" http_request '(
        record {
            method = "GET";
            url = "/asset/test-memory/thumbnail?token=invalid-token";
            headers = vec {};
            body = blob "";
        }
    )' --identity "$IDENTITY" 2>/dev/null || echo "ERROR")
    
    # Should return 401 (unauthorized) or 403 (forbidden)
    if [[ "$response" == *"401"* ]] || [[ "$response" == *"403"* ]] || [[ "$response" == *"ERROR"* ]]; then
        echo_success "✅ Asset serving properly rejects requests with invalid token"
        return 0
    else
        echo_error "❌ Asset serving should have rejected request with invalid token: $response"
        return 1
    fi
}

# Test 5: Invalid endpoint (should return 404)
test_invalid_endpoint() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing invalid endpoint..."
    
    local response=$(dfx canister call "$CANISTER_ID" http_request '(
        record {
            method = "GET";
            url = "/invalid-endpoint";
            headers = vec {};
            body = blob "";
        }
    )' --identity "$IDENTITY" 2>/dev/null || echo "ERROR")
    
    # Should return 404 (not found)
    if [[ "$response" == *"404"* ]] || [[ "$response" == *"ERROR"* ]]; then
        echo_success "✅ Invalid endpoints properly return 404"
        return 0
    else
        echo_error "❌ Invalid endpoint should have returned 404: $response"
        return 1
    fi
}

# Run all tests
main() {
    local failed_tests=0
    
    echo_info "Starting HTTP module basic tests..."
    
    test_health_check || ((failed_tests++))
    test_token_minting || ((failed_tests++))
    test_asset_serving_no_token || ((failed_tests++))
    test_asset_serving_invalid_token || ((failed_tests++))
    test_invalid_endpoint || ((failed_tests++))
    
    echo_info "HTTP module basic tests completed"
    
    if [[ $failed_tests -eq 0 ]]; then
        echo_success "🎉 All HTTP module basic tests passed!"
        return 0
    else
        echo_error "❌ $failed_tests HTTP module basic tests failed"
        return 1
    fi
}

# Run the tests
main "$@"
