#!/bin/bash

# HTTP Module Test Runner
# Runs all HTTP module tests

set -e

# Source test utilities
source "$(dirname "$0")/../test_utils.sh"

echo_header "🚀 Running HTTP Module Tests"

# Configuration
DEBUG="${DEBUG:-false}"

# Run basic HTTP tests
echo_info "Running basic HTTP functionality tests..."
if "$(dirname "$0")/test_http_basic.sh"; then
    echo_success "✅ Basic HTTP tests passed"
else
    echo_error "❌ Basic HTTP tests failed"
    exit 1
fi

# Run HTTP core functionality tests
echo_info "Running HTTP core functionality tests..."
if "$(dirname "$0")/test_http_core_functionality.mjs"; then
    echo_success "✅ HTTP core functionality tests passed"
else
    echo_error "❌ HTTP core functionality tests failed"
    exit 1
fi

# Run local HTTP gateway tests
echo_info "Running local HTTP gateway tests..."
if "$(dirname "$0")/test_local_http_gateway.mjs"; then
    echo_success "✅ Local HTTP gateway tests passed"
else
    echo_error "❌ Local HTTP gateway tests failed"
    exit 1
fi

echo_success "🎉 All HTTP module tests completed successfully!"
