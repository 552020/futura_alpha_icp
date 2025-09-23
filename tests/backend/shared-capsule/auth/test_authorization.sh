#!/bin/bash

# Authorization Test Runner
# Runs both unit tests and e2e tests for authorization functionality
# Provides clear separation between unit tests (no canister) and e2e tests (requires canister)

set -e

# Source test utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../test_utils.sh"

# Test configuration
TEST_NAME="Authorization Test Suite"

# Parse command line arguments
MAINNET_MODE=false
UNIT_ONLY=false
E2E_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --mainnet)
            MAINNET_MODE=true
            shift
            ;;
        --unit-only)
            UNIT_ONLY=true
            shift
            ;;
        --e2e-only)
            E2E_ONLY=true
            shift
            ;;
        --help)
            echo "Usage: $0 [--mainnet] [--unit-only] [--e2e-only] [--help]"
            echo ""
            echo "Options:"
            echo "  --mainnet     Run tests against mainnet canister"
            echo "  --unit-only   Run only unit tests (no canister required)"
            echo "  --e2e-only    Run only e2e tests (requires running canister)"
            echo "  --help        Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                    # Run all tests (unit + e2e if canister available)"
            echo "  $0 --unit-only        # Run only unit tests"
            echo "  $0 --e2e-only         # Run only e2e tests"
            echo "  $0 --mainnet          # Run all tests against mainnet"
            echo "  $0 --mainnet --e2e-only # Run only e2e tests against mainnet"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo_header "🔐 Authorization Test Suite"

# Function to run unit tests
run_unit_tests() {
    echo_info "🧪 Running Authorization Unit Tests..."
    echo_info "=========================================="
    
    if [[ "$MAINNET_MODE" == "true" ]]; then
        echo_warn "Unit tests don't use mainnet - running in local mode"
    fi
    
    if "$SCRIPT_DIR/test_authorization_unit.sh"; then
        echo_success "✅ Unit tests passed"
        return 0
    else
        echo_error "❌ Unit tests failed"
        return 1
    fi
}

# Function to run e2e tests
run_e2e_tests() {
    echo_info "🌐 Running Authorization E2E Tests..."
    echo_info "=========================================="
    
    if [[ "$MAINNET_MODE" == "true" ]]; then
        echo_info "Running e2e tests against mainnet"
        if "$SCRIPT_DIR/test_authorization_e2e.sh" --mainnet; then
            echo_success "✅ E2E tests passed (mainnet)"
            return 0
        else
            echo_error "❌ E2E tests failed (mainnet)"
            return 1
        fi
    else
        echo_info "Running e2e tests against local canister"
        if "$SCRIPT_DIR/test_authorization_e2e.sh"; then
            echo_success "✅ E2E tests passed (local)"
            return 0
        else
            echo_error "❌ E2E tests failed (local)"
            return 1
        fi
    fi
}

# Main test execution
main() {
    echo_header "🚀 Starting $TEST_NAME"
    
    if [[ "$MAINNET_MODE" == "true" ]]; then
        echo_info "Running in MAINNET mode"
    else
        echo_info "Running in LOCAL mode"
    fi
    
    echo ""
    
    local unit_result=0
    local e2e_result=0
    
    # Run unit tests
    if [[ "$E2E_ONLY" != "true" ]]; then
        run_unit_tests
        unit_result=$?
    else
        echo_info "⏭️  Skipping unit tests (--e2e-only specified)"
    fi
    
    echo ""
    
    # Run e2e tests
    if [[ "$UNIT_ONLY" != "true" ]]; then
        run_e2e_tests
        e2e_result=$?
    else
        echo_info "⏭️  Skipping e2e tests (--unit-only specified)"
    fi
    
    echo ""
    echo_header "📊 Test Suite Summary"
    echo_info "=========================================="
    
    if [[ "$E2E_ONLY" != "true" ]]; then
        if [[ $unit_result -eq 0 ]]; then
            echo_success "✅ Unit Tests: PASSED"
        else
            echo_error "❌ Unit Tests: FAILED"
        fi
    fi
    
    if [[ "$UNIT_ONLY" != "true" ]]; then
        if [[ $e2e_result -eq 0 ]]; then
            echo_success "✅ E2E Tests: PASSED"
        else
            echo_error "❌ E2E Tests: FAILED"
        fi
    fi
    
    echo ""
    
    # Overall result
    if [[ $unit_result -eq 0 && $e2e_result -eq 0 ]]; then
        echo_header "🎉 Authorization Test Suite completed successfully!"
        echo_success "✅ All tests passed"
        echo_success "✅ Authorization system is working correctly"
        exit 0
    else
        echo_header "💥 Authorization Test Suite completed with failures"
        echo_error "❌ Some tests failed"
        echo_info "Check the output above for details"
        exit 1
    fi
}

# Run main function
main "$@"