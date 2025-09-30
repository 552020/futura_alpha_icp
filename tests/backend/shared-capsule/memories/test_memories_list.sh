#!/bin/bash

# Test script for memories_list endpoint
# Tests the memories_list(capsule_id) endpoint

set -e

# Source test utilities
source "$(dirname "$0")/../../test_utils.sh"

# Configuration
CANISTER_ID="backend"
IDENTITY="default"
DEBUG="${DEBUG:-false}"  # Set DEBUG=true to enable debug output

echo_header "🧪 Testing memories_list endpoint"

# Test 1: Test memories_list with valid capsule ID
test_memories_list_valid_capsule() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memories_list with valid capsule ID..."
    
    # Get a capsule ID using utility function
    local capsule_id=$(get_test_capsule_id)
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to get capsule ID for testing"
        return 1
    fi
    
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing with capsule ID: $capsule_id"
    
    # Call memories_list with the capsule ID
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_list "(\"$capsule_id\")" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ memories_list with valid capsule ID succeeded"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 0
    else
        echo_error "❌ memories_list with valid capsule ID failed"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 1
    fi
}

# Test 2: Test memories_list with invalid capsule ID
test_memories_list_invalid_capsule() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memories_list with invalid capsule ID..."
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_list "(\"invalid_capsule_id\")" 2>/dev/null)
    
    if [[ $result == *"success = true"* && $result == *"memories = vec {}"* ]]; then
        echo_success "✅ memories_list with invalid capsule ID returned empty result (expected)"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 0
    else
        echo_error "❌ memories_list with invalid capsule ID returned unexpected result"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 1
    fi
}

# Test 3: Test memories_list with empty string
test_memories_list_empty_string() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memories_list with empty string..."
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_list "(\"\")" 2>/dev/null)
    
    if [[ $result == *"success = true"* && $result == *"memories = vec {}"* ]]; then
        echo_success "✅ memories_list with empty string returned empty result (expected)"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 0
    else
        echo_error "❌ memories_list with empty string returned unexpected result"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 1
    fi
}



# Test 5: Test memories_list with memory counting and ID extraction
test_memories_list_with_counting() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memories_list with memory counting and ID extraction..."
    
    # Get a valid capsule ID using utility function
    local capsule_id=$(get_test_capsule_id)
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to get capsule ID for testing"
        return 1
    fi
    
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing with capsule ID: $capsule_id"
    
    # Call memories_list with the capsule ID
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_list "(\"$capsule_id\")" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ memories_list with counting succeeded"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        
        # Count memories (like the old test_list_memories.sh did)
        local memory_count=$(echo "$result" | grep -o 'id = "[^"]*"' | wc -l)
        [[ "$DEBUG" == "true" ]] && echo_debug "Number of memories found: $memory_count"
        
        # Check if we have the test memory from other tests
        if [ -f /tmp/test_memory_id.txt ]; then
            local test_memory_id=$(cat /tmp/test_memory_id.txt)
            if echo "$result" | grep -q "$test_memory_id"; then
                echo_success "✅ Test memory found in list"
            else
                [[ "$DEBUG" == "true" ]] && echo_debug "Test memory not found in list (may have been deleted)"
            fi
        fi
        
        # Show memory IDs if any exist
        if [ "$memory_count" -gt 0 ]; then
            [[ "$DEBUG" == "true" ]] && echo_debug "Memory IDs:"
            echo "$result" | grep -o 'id = "[^"]*"' | sed 's/id = "\([^"]*\)"/\1/'
        fi
        
        return 0
    else
        echo_error "❌ memories_list with counting failed"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 1
    fi
}

# Test 6: Test memories_list response structure
test_memories_list_response_structure() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memories_list response structure..."
    
    # Get a valid capsule ID using utility function
    local capsule_id=$(get_test_capsule_id)
    
    if [[ -z "$capsule_id" ]]; then
        [[ "$DEBUG" == "true" ]] && echo_debug "No capsule found, skipping response structure test"
        return 0
    fi
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_list "(\"$capsule_id\")" 2>/dev/null)
    
    # Check for required fields in response
    if [[ $result == *"success = true"* ]] && \
       [[ $result == *"memories = vec"* ]] && \
       [[ $result == *"message = \"Memories retrieved successfully\""* ]]; then
        echo_success "✅ memories_list response structure is correct"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 0
    else
        echo_error "❌ memories_list response structure is incorrect"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 1
    fi
}

# Main test execution
main() {
    echo_header "🚀 Starting memories_list endpoint tests"
    
    local tests_passed=0
    local tests_failed=0
    
    if run_test "Valid capsule ID" test_memories_list_valid_capsule; then
        ((tests_passed++))
    else
        ((tests_failed++))
    fi
    
    if run_test "Invalid capsule ID" test_memories_list_invalid_capsule; then
        ((tests_passed++))
    else
        ((tests_failed++))
    fi
    
    if run_test "Empty string" test_memories_list_empty_string; then
        ((tests_passed++))
    else
        ((tests_failed++))
    fi
    
    if run_test "Memory counting and ID extraction" test_memories_list_with_counting; then
        ((tests_passed++))
    else
        ((tests_failed++))
    fi
    
    if run_test "Response structure" test_memories_list_response_structure; then
        ((tests_passed++))
    else
        ((tests_failed++))
    fi
    
    # Final summary - suppress debug output for clean summary
    echo ""
    echo "=========================================="
    if [[ $tests_failed -eq 0 ]]; then
        echo "🎉 All memories_list tests completed successfully! ($tests_passed/$((tests_passed + tests_failed)))"
    else
        echo "❌ Some memories_list tests failed! ($tests_passed passed, $tests_failed failed)"
        echo "=========================================="
        exit 1
    fi
    echo "=========================================="
    echo ""
}

# Run main function
main "$@"
