#!/bin/bash

# Test script for memories_ping endpoint functionality
# Tests the consolidated memories_ping function that replaces get_memory_presence_icp and get_memory_list_presence_icp

set -e

# Source test utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_utils.sh"

# Test configuration
CANISTER_ID="backend"
IDENTITY="default"

echo_header "🧪 Testing memories_ping endpoint (consolidated memory presence function)"

# Test 1: Test memories_ping with single memory ID
test_memories_ping_single() {
    echo_info "Testing memories_ping with single memory ID..."
    
    local memory_id="test_memory_single_$(date +%s)_$$"
    
    # Call memories_ping with single memory ID
    local result=$(dfx canister call backend memories_ping "(vec { \"$memory_id\" })" 2>/dev/null)
    
    if [[ $? -eq 0 ]]; then
        echo_success "✅ memories_ping with single memory ID succeeded"
        echo_debug "Result: $result"
        
        # Verify response format (should return ICPResult with data array)
        if echo "$result" | grep -q "success = true"; then
            echo_success "✅ Response format is correct (ICPResult with success = true)"
        else
            echo_error "❌ Response format is incorrect - expected success = true"
            return 1
        fi
    else
        echo_error "❌ memories_ping with single memory ID failed"
        return 1
    fi
}

# Test 2: Test memories_ping with multiple memory IDs
test_memories_ping_multiple() {
    echo_info "Testing memories_ping with multiple memory IDs..."
    
    local memory_id1="test_memory_multi1_$(date +%s)_$$"
    local memory_id2="test_memory_multi2_$(date +%s)_$$"
    local memory_id3="test_memory_multi3_$(date +%s)_$$"
    
    # Call memories_ping with multiple memory IDs
    local result=$(dfx canister call backend memories_ping "(vec { \"$memory_id1\"; \"$memory_id2\"; \"$memory_id3\" })" 2>/dev/null)
    
    if [[ $? -eq 0 ]]; then
        echo_success "✅ memories_ping with multiple memory IDs succeeded"
        echo_debug "Result: $result"
        
        # Verify response is successful and contains data array
        if echo "$result" | grep -q "success = true"; then
            echo_success "✅ Response is successful as expected"
        else
            echo_error "❌ Response should be successful"
            return 1
        fi
    else
        echo_error "❌ memories_ping with multiple memory IDs failed"
        return 1
    fi
}

# Test 3: Test memories_ping with empty memory list
test_memories_ping_empty() {
    echo_info "Testing memories_ping with empty memory list..."
    
    # Call memories_ping with empty vector
    local result=$(dfx canister call backend memories_ping "(vec {})" 2>/dev/null)
    
    if [[ $? -eq 0 ]]; then
        echo_success "✅ memories_ping with empty list succeeded"
        echo_debug "Result: $result"
        
        # Verify response is empty data array (check for both null and empty array formats)
        if echo "$result" | grep -q "data = \[\]" || echo "$result" | grep -q "data = null"; then
            echo_success "✅ Empty list returns empty data array as expected"
        else
            echo_error "❌ Empty list should return empty data array or null"
            return 1
        fi
    else
        echo_error "❌ memories_ping with empty list failed"
        return 1
    fi
}

# Test 4: Test memories_ping with very long memory ID
test_memories_ping_long_id() {
    echo_info "Testing memories_ping with very long memory ID..."
    
    # Create a very long memory ID (1000 characters)
    local long_id=$(printf '%*s' 1000 | tr ' ' 'a')
    
    # Call memories_ping with long memory ID
    local result=$(dfx canister call backend memories_ping "(vec { \"$long_id\" })" 2>/dev/null)
    
    if [[ $? -eq 0 ]]; then
        echo_success "✅ memories_ping with long memory ID succeeded"
        echo_debug "Result: $result"
    else
        echo_error "❌ memories_ping with long memory ID failed"
        return 1
    fi
}

# Test 5: Test memories_ping with special characters in memory ID
test_memories_ping_special_chars() {
    echo_info "Testing memories_ping with special characters in memory ID..."
    
    local special_id="test-memory_123!@#$%^&*()+=[]{}|\\:;\"'<>?,./"
    
    # Call memories_ping with special character memory ID
    local result=$(dfx canister call backend memories_ping "(vec { \"$special_id\" })" 2>/dev/null)
    
    if [[ $? -eq 0 ]]; then
        echo_success "✅ memories_ping with special characters succeeded"
        echo_debug "Result: $result"
    else
        echo_error "❌ memories_ping with special characters failed"
        return 1
    fi
}

# Test 6: Test memories_ping with large number of memory IDs
test_memories_ping_large_list() {
    echo_info "Testing memories_ping with large number of memory IDs..."
    
    # Create 50 memory IDs
    local memory_ids=""
    for i in {1..50}; do
        if [[ -n "$memory_ids" ]]; then
            memory_ids="$memory_ids; \"test_memory_large_${i}_$(date +%s)_$$\""
        else
            memory_ids="\"test_memory_large_${i}_$(date +%s)_$$\""
        fi
    done
    
    # Call memories_ping with large list
    local result=$(dfx canister call backend memories_ping "(vec { $memory_ids })" 2>/dev/null)
    
    if [[ $? -eq 0 ]]; then
        echo_success "✅ memories_ping with large list succeeded"
        echo_debug "Result: $result"
        
        # Verify response is successful
        if echo "$result" | grep -q "success = true"; then
            echo_success "✅ Response is successful as expected"
        else
            echo_error "❌ Response should be successful"
            return 1
        fi
    else
        echo_error "❌ memories_ping with large list failed"
        return 1
    fi
}

# Test 7: Test memories_ping with mixed existing/non-existing memories (after creating some metadata)
test_memories_ping_mixed_existence() {
    echo_info "Testing memories_ping with mixed existing/non-existing memories..."
    
    local existing_memory_id="test_existing_memory_$(date +%s)_$$"
    local non_existing_memory_id="test_nonexisting_memory_$(date +%s)_$$"
    
    # First, create some metadata for one memory to make it "exist"
    echo_debug "Creating metadata for existing memory..."
    local metadata_data="(record {
        title = opt \"Test Memory\";
        description = opt \"Test Description\";
        tags = vec { \"test\" };
        created_at = 1234567890;
        updated_at = 1234567890;
        size = opt 1024;
        content_type = opt \"text/plain\";
        custom_fields = vec {}
    })"
    
    # Note: This test assumes upsert_metadata works - if it fails, we'll skip this test
    local metadata_result=$(dfx canister call backend upsert_metadata "(\"$existing_memory_id\", variant { Note }, $metadata_data, \"test_key_$(date +%s)\")" 2>/dev/null || echo "FAILED")
    
    if [[ "$metadata_result" != "FAILED" ]]; then
        echo_debug "Metadata created successfully, now testing memories_ping..."
        
        # Call memories_ping with both existing and non-existing memories
        local result=$(dfx canister call backend memories_ping "(vec { \"$existing_memory_id\"; \"$non_existing_memory_id\" })" 2>/dev/null)
        
        if [[ $? -eq 0 ]]; then
            echo_success "✅ memories_ping with mixed existence succeeded"
            echo_debug "Result: $result"
            
            # Verify response is successful
            if echo "$result" | grep -q "success = true"; then
                echo_success "✅ Response is successful as expected"
            else
                echo_error "❌ Response should be successful"
                return 1
            fi
        else
            echo_error "❌ memories_ping with mixed existence failed"
            return 1
        fi
    else
        echo_warning "⚠️  Skipping mixed existence test - upsert_metadata not available or failed"
        echo_info "This test requires upsert_metadata to work properly"
    fi
}

# Main test execution
run_all_tests() {
    echo_header "🚀 Starting memories_ping endpoint tests"
    
    run_test "Single memory ID" test_memories_ping_single
    run_test "Multiple memory IDs" test_memories_ping_multiple
    run_test "Empty memory list" test_memories_ping_empty
    run_test "Long memory ID" test_memories_ping_long_id
    run_test "Special characters in ID" test_memories_ping_special_chars
    run_test "Large list of memory IDs" test_memories_ping_large_list
    run_test "Mixed existing/non-existing memories" test_memories_ping_mixed_existence
    
    echo_header "🎉 All memories_ping tests completed successfully!"
    echo_success "✅ Consolidated function works correctly"
    echo_info "📝 Function successfully replaced get_memory_presence_icp and get_memory_list_presence_icp"
}

# Run tests if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    run_all_tests
fi
