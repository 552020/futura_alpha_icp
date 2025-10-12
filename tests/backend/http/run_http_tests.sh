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
if ./test_http_basic.sh; then
    echo_success "✅ Basic HTTP tests passed"
else
    echo_error "❌ Basic HTTP tests failed"
    exit 1
fi

echo_success "🎉 All HTTP module tests completed successfully!"
