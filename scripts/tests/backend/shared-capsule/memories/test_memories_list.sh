#!/bin/bash

# Test script for memories_list endpoint
# Tests the memories_list(capsule_id) endpoint

set -e

# Source test utilities
source "$(dirname "$0")/../test_utils.sh"

# Configuration
CANISTER_ID="backend"
IDENTITY="default"

echo_header "🧪 Testing memories_list endpoint"

# Test 1: Test memories_list with valid capsule ID
test_memories_list_valid_capsule() {
    echo_debug "Testing memories_list with valid capsule ID..."
    
    # First, get a capsule ID to test with
    local capsule_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID capsules_read_basic "(null)" 2>/dev/null)
    
    if [[ $capsule_result == *"null"* ]]; then
        echo_debug "No capsule found, creating one first..."
        local create_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID capsules_create "(null)" 2>/dev/null)
        echo_debug "Created capsule: $create_result"
        
        # Extract capsule ID from creation result
        local capsule_id=$(echo "$create_result" | grep -o 'capsule_id = opt "[^"]*"' | sed 's/capsule_id = opt "//' | sed 's/"//')
        if [[ -z "$capsule_id" ]]; then
            echo_error "Failed to extract capsule ID from creation result"
            return 1
        fi
    else
        # Extract capsule ID from existing result
        local capsule_id=$(echo "$capsule_result" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')
        if [[ -z "$capsule_id" ]]; then
            echo_error "Failed to extract capsule ID from existing result"
            return 1
        fi
    fi
    
    echo_debug "Testing with capsule ID: $capsule_id"
    
    # Call memories_list with the capsule ID
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_list "(\"$capsule_id\")" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ memories_list with valid capsule ID succeeded"
        echo_debug "Result: $result"
    else
        echo_error "❌ memories_list with valid capsule ID failed"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 2: Test memories_list with invalid capsule ID
test_memories_list_invalid_capsule() {
    echo_debug "Testing memories_list with invalid capsule ID..."
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_list "(\"invalid_capsule_id\")" 2>/dev/null)
    
    if [[ $result == *"success = true"* && $result == *"memories = vec {}"* ]]; then
        echo_success "✅ memories_list with invalid capsule ID returned empty result (expected)"
        echo_debug "Result: $result"
    else
        echo_error "❌ memories_list with invalid capsule ID returned unexpected result"
        echo_debug "Result: $result"
        return 1
    fi
}

# Test 3: Test memories_list with empty string
test_memories_list_empty_string() {
    echo_debug "Testing memories_list with empty string..."
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_list "(\"\")" 2>/dev/null)
    
    if [[ $result == *"success = true"* && $result == *"memories = vec {}"* ]]; then
        echo_success "✅ memories_list with empty string returned empty result (expected)"
        echo_debug "Result: $result"
    else
        echo_error "❌ memories_list with empty string returned unexpected result"
        echo_debug "Result: $result"
        return 1
    fi
}



# Test 5: Test memories_list response structure
test_memories_list_response_structure() {
    echo_debug "Testing memories_list response structure..."
    
    # Get a valid capsule ID first
    local capsule_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID capsules_read_basic "()" 2>/dev/null)
    local capsule_id=$(echo "$capsule_result" | grep -o 'capsule_id = "[^"]*"' | cut -d'"' -f2)
    
    if [[ -z "$capsule_id" ]]; then
        echo_debug "No capsule found, skipping response structure test"
        return 0
    fi
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_list "(\"$capsule_id\")" 2>/dev/null)
    
    # Check for required fields in response
    if [[ $result == *"success = true"* ]] && \
       [[ $result == *"memories = vec"* ]] && \
       [[ $result == *"message = \"Memories retrieved successfully\""* ]]; then
        echo_success "✅ memories_list response structure is correct"
        echo_debug "Result: $result"
    else
        echo_error "❌ memories_list response structure is incorrect"
        echo_debug "Result: $result"
        return 1
    fi
}

# Main test execution
main() {
    echo_header "🚀 Starting memories_list endpoint tests"
    
    run_test "Valid capsule ID" test_memories_list_valid_capsule
    run_test "Invalid capsule ID" test_memories_list_invalid_capsule
    run_test "Empty string" test_memories_list_empty_string
    run_test "Response structure" test_memories_list_response_structure
    
    echo_header "🎉 All memories_list tests completed successfully!"
}

# Run main function
main "$@"
