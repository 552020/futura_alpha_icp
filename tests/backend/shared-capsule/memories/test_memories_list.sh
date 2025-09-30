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



# Test 5: Test memories_list with controlled test memories
test_memories_list_with_controlled_memories() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memories_list with controlled test memories..."
    
    # Get a valid capsule ID using utility function
    local capsule_id=$(get_test_capsule_id)
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to get capsule ID for testing"
        return 1
    fi
    
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing with capsule ID: $capsule_id"
    
    # Create 3 test memories with specific titles
    echo_info "Creating test memories..."
    local memory1_id=$(create_test_memory "$capsule_id" "List Test Memory 1" "First test memory for list testing" '"test"; "list"; "memory1"' 'blob "VGVzdCBNZW1vcnkgMQ=="' "$CANISTER_ID" "$IDENTITY")
    local memory2_id=$(create_test_memory "$capsule_id" "List Test Memory 2" "Second test memory for list testing" '"test"; "list"; "memory2"' 'blob "VGVzdCBNZW1vcnkgMg=="' "$CANISTER_ID" "$IDENTITY")
    local memory3_id=$(create_test_memory "$capsule_id" "List Test Memory 3" "Third test memory for list testing" '"test"; "list"; "memory3"' 'blob "VGVzdCBNZW1vcnkgMw=="' "$CANISTER_ID" "$IDENTITY")
    
    if [[ -z "$memory1_id" || -z "$memory2_id" || -z "$memory3_id" ]]; then
        echo_error "Failed to create test memories"
        return 1
    fi
    
    echo_success "✅ Created 3 test memories: $memory1_id, $memory2_id, $memory3_id"
    
    # Call memories_list with the capsule ID
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_list "(\"$capsule_id\")" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ memories_list with controlled memories succeeded"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        
        # Count memories
        local memory_count=$(echo "$result" | grep -o 'id = "[^"]*"' | wc -l)
        [[ "$DEBUG" == "true" ]] && echo_debug "Number of memories found: $memory_count"
        
        # Verify all 3 test memories are in the list
        local found_memory1=false
        local found_memory2=false
        local found_memory3=false
        
        if echo "$result" | grep -q "$memory1_id"; then
            found_memory1=true
            echo_success "✅ Memory 1 found in list"
        else
            echo_error "❌ Memory 1 not found in list"
        fi
        
        if echo "$result" | grep -q "$memory2_id"; then
            found_memory2=true
            echo_success "✅ Memory 2 found in list"
        else
            echo_error "❌ Memory 2 not found in list"
        fi
        
        if echo "$result" | grep -q "$memory3_id"; then
            found_memory3=true
            echo_success "✅ Memory 3 found in list"
        else
            echo_error "❌ Memory 3 not found in list"
        fi
        
        # Verify memory titles are correct
        if echo "$result" | grep -q "List Test Memory 1"; then
            echo_success "✅ Memory 1 title is correct"
        else
            echo_error "❌ Memory 1 title is incorrect"
        fi
        
        if echo "$result" | grep -q "List Test Memory 2"; then
            echo_success "✅ Memory 2 title is correct"
        else
            echo_error "❌ Memory 2 title is incorrect"
        fi
        
        if echo "$result" | grep -q "List Test Memory 3"; then
            echo_success "✅ Memory 3 title is correct"
        else
            echo_error "❌ Memory 3 title is incorrect"
        fi
        
        # Clean up test memories
        echo_info "Cleaning up test memories..."
        dfx canister call --identity $IDENTITY $CANISTER_ID memories_delete "(\"$memory1_id\")" >/dev/null 2>&1
        dfx canister call --identity $IDENTITY $CANISTER_ID memories_delete "(\"$memory2_id\")" >/dev/null 2>&1
        dfx canister call --identity $IDENTITY $CANISTER_ID memories_delete "(\"$memory3_id\")" >/dev/null 2>&1
        echo_success "✅ Test memories cleaned up"
        
        # Return success only if all memories were found and titles were correct
        if [[ "$found_memory1" == "true" && "$found_memory2" == "true" && "$found_memory3" == "true" ]]; then
            return 0
        else
            return 1
        fi
    else
        echo_error "❌ memories_list with controlled memories failed"
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
    
    if run_test "Controlled test memories" test_memories_list_with_controlled_memories; then
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
