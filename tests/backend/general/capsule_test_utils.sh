#!/bin/bash

# Capsule Test Utilities
# Shared functions for capsule-related test scripts

# Helper function to check if response indicates success (Result<T, Error> format)
is_success() {
    local response="$1"
    echo "$response" | grep -q "variant {" && (echo "$response" | grep -q "Ok =" || echo "$response" | grep -q "Ok }")
}

# Helper function to check if response indicates failure (Result<T, Error> format)
is_failure() {
    local response="$1"
    echo "$response" | grep -q "variant {" && echo "$response" | grep -q "Err ="
}

# Helper function to check if response contains capsule data (new Result<T, Error> format)
has_capsule_data() {
    local response="$1"
    echo "$response" | grep -q "Ok = record {"
}

# Helper function to check if response is NotFound error (new Result<T, Error> format)
is_not_found() {
    local response="$1"
    echo "$response" | grep -q "Err = variant { NotFound }"
}

# Helper function to check if response is Unauthorized error
is_unauthorized() {
    local response="$1"
    echo "$response" | grep -q "Err = variant { Unauthorized }"
}

# Helper function to get a capsule ID for testing
get_test_capsule_id() {
    local capsule_result=$(dfx canister call backend capsules_read_basic "(null)" 2>/dev/null)
    local capsule_id=""
    
    if [[ $capsule_result == *"null"* ]] || [[ $capsule_result == *"NotFound"* ]]; then
        echo_info "No capsule found, creating one first..."
        local create_result=$(dfx canister call backend capsules_create "(null)" 2>/dev/null)
        if is_success "$create_result"; then
            # Extract capsule ID from the new Result<Capsule> format
            capsule_id=$(echo "$create_result" | grep -o 'id = "[^"]*"' | sed 's/id = "//' | sed 's/"//')
        fi
    else
        if is_success "$capsule_result"; then
            capsule_id=$(echo "$capsule_result" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')
        fi
    fi
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to get capsule ID for testing"
        return 1
    fi
    
    echo "$capsule_id"
}

# Helper function to create capsule update data
create_capsule_update_data() {
    local bound_to_neon="$1"
    cat << EOF
(record {
  bound_to_neon = opt $bound_to_neon;
})
EOF
}

# Helper function to setup user and capsule
setup_user_and_capsule() {
    echo_info "Setting up test user and capsule..."
    
    # Register user with nonce (new method)
    local nonce=$(date +%s)
    local register_result=$(dfx canister call backend register_with_nonce "(\"$nonce\")" 2>/dev/null)
    if ! is_success "$register_result"; then
        echo_warn "User registration failed, continuing with existing user..."
    fi
    
    # Ensure we have a capsule
    local capsule_id=$(get_test_capsule_id)
    if [[ -n "$capsule_id" ]]; then
        echo_info "Using existing capsule: $capsule_id"
    else
        echo_error "Failed to create or get capsule for testing"
        return 1
    fi
    
    echo_info "Setup complete"
}

# Helper function to extract capsule ID from response
extract_capsule_id() {
    local response="$1"
    # Try different patterns for capsule ID extraction
    local capsule_id=""
    
    # Pattern 1: id = "capsule_id"
    capsule_id=$(echo "$response" | grep -o 'id = "[^"]*"' | sed 's/id = "//' | sed 's/"//')
    
    # Pattern 2: capsule_id = "capsule_id" (for CapsuleInfo responses)
    if [[ -z "$capsule_id" ]]; then
        capsule_id=$(echo "$response" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')
    fi
    
    echo "$capsule_id"
}

# Helper function to check if response contains expected capsule fields
has_expected_capsule_fields() {
    local response="$1"
    # Check for common capsule fields
    echo "$response" | grep -q "id = " && \
    echo "$response" | grep -q "subject = " && \
    echo "$response" | grep -q "owners = "
}

# Helper function to check if response contains expected CapsuleInfo fields
has_expected_capsule_info_fields() {
    local response="$1"
    # Check for expected CapsuleInfo fields
    echo "$response" | grep -q "capsule_id =" && \
    echo "$response" | grep -q "subject =" && \
    echo "$response" | grep -q "is_owner =" && \
    echo "$response" | grep -q "is_controller =" && \
    echo "$response" | grep -q "is_self_capsule =" && \
    echo "$response" | grep -q "bound_to_neon =" && \
    echo "$response" | grep -q "created_at =" && \
    echo "$response" | grep -q "updated_at =" && \
    echo "$response" | grep -q "memory_count =" && \
    echo "$response" | grep -q "gallery_count =" && \
    echo "$response" | grep -q "connection_count ="
}

# Helper function to check if response contains expected CapsuleHeader fields
has_expected_capsule_header_fields() {
    local response="$1"
    # Check for expected CapsuleHeader fields
    echo "$response" | grep -q "id = " && \
    echo "$response" | grep -q "subject = " && \
    echo "$response" | grep -q "owner_count = " && \
    echo "$response" | grep -q "controller_count = " && \
    echo "$response" | grep -q "memory_count = " && \
    echo "$response" | grep -q "created_at = " && \
    echo "$response" | grep -q "updated_at ="
}

# Helper function to check if response is empty (no capsules)
is_empty_response() {
    local response="$1"
    echo "$response" | grep -q "vec {}"
}

# Helper function to check if response contains capsules
has_capsules() {
    local response="$1"
    echo "$response" | grep -q "record {"
}

# Helper function to run a test with proper counting
run_capsule_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo_info "Running: $test_name"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if eval "$test_command"; then
        echo_pass "$test_name"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo_fail "$test_name"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    echo ""
}

# Helper function to print test summary
print_test_summary() {
    echo_info "=================================="
    echo_info "Test Summary:"
    echo_info "Total tests: $TOTAL_TESTS"
    echo_info "Passed: $PASSED_TESTS"
    echo_info "Failed: $FAILED_TESTS"
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo_pass "All tests passed!"
        return 0
    else
        echo_fail "Some tests failed!"
        return 1
    fi
}
