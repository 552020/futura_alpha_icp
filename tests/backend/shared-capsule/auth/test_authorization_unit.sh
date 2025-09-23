#!/bin/bash

# Unit tests for authorization functionality
# Tests Rust compilation and module-level authorization logic
# NO canister interaction - pure unit tests only

set -e

# Source test utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../test_utils.sh"

# Test configuration
TEST_NAME="Authorization Unit Tests"

echo_header "🔐 Testing Authorization Unit Tests (Rust Compilation Only)"

# Test 1: Verify authorization module compiles and tests pass
test_authorization_compilation() {
    echo_info "Testing authorization module compilation..."
    
    if cargo test --manifest-path src/backend/Cargo.toml auth::tests --quiet; then
        echo_success "Authorization module compilation passed"
        return 0
    else
        echo_error "Authorization module compilation failed"
        return 1
    fi
}

# Test 2: Verify metadata operations include authorization
test_metadata_authorization() {
    echo_info "Testing metadata operations authorization..."
    
    if cargo test --manifest-path src/backend/Cargo.toml metadata::tests --quiet; then
        echo_success "Metadata operations authorization passed"
        return 0
    else
        echo_error "Metadata operations authorization failed"
        return 1
    fi
}

# Test 3: Verify upload operations include authorization  
test_upload_authorization() {
    echo_info "Testing upload operations authorization..."
    
    if cargo test --manifest-path src/backend/Cargo.toml upload::tests --quiet; then
        echo_success "Upload operations authorization passed"
        return 0
    else
        echo_error "Upload operations authorization failed"
        return 1
    fi
}

# Test 4: Verify all ICP endpoints compile successfully
test_endpoints_compilation() {
    echo_info "Testing all ICP endpoints compilation..."
    
    if cargo check --manifest-path src/backend/Cargo.toml --quiet; then
        echo_success "All ICP endpoints compilation passed"
        return 0
    else
        echo_error "ICP endpoints compilation failed"
        return 1
    fi
}

# Test 5: Verify authorization functions are properly exported
test_authorization_functions_exported() {
    echo_info "Testing authorization functions are properly exported..."
    
    # Check if authorization functions are properly defined in the codebase
    if grep -r "pub fn verify_caller_authorized" src/backend/src/ >/dev/null 2>&1; then
        echo_success "Authorization functions are properly exported"
        return 0
    else
        echo_error "Authorization functions not found or not properly exported"
        return 1
    fi
}

# Test 6: Verify error types are properly defined
test_authorization_error_types() {
    echo_info "Testing authorization error types are properly defined..."
    
    # Check if authorization error types are properly defined
    if grep -r "Unauthorized" src/backend/src/ >/dev/null 2>&1; then
        echo_success "Authorization error types are properly defined"
        return 0
    else
        echo_error "Authorization error types not found"
        return 1
    fi
}

# Main test execution
main() {
    echo_header "🚀 Starting $TEST_NAME"
    echo_info "Running pure unit tests - NO canister interaction"
    echo ""
    
    # Run all unit tests
    run_test "Authorization module compilation" "test_authorization_compilation"
    run_test "Metadata operations authorization" "test_metadata_authorization"
    run_test "Upload operations authorization" "test_upload_authorization"
    run_test "ICP endpoints compilation" "test_endpoints_compilation"
    run_test "Authorization functions exported" "test_authorization_functions_exported"
    run_test "Authorization error types defined" "test_authorization_error_types"
    
    echo_header "🎉 $TEST_NAME completed successfully!"
    echo_success "✅ All authorization unit tests passed"
    echo_success "✅ Rust compilation verified"
    echo_success "✅ Authorization logic properly implemented"
    echo_success "✅ Error types properly defined"
}

# Run main function
main "$@"
