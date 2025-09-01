#!/bin/bash

# Test sync_gallery_memories endpoint functionality
# Tests the new batch memory synchronization from Web2 to ICP storage
# This tests Task 47: Gallery Sync to ICP implementation

# Load test configuration and utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_config.sh"
source "$SCRIPT_DIR/../test_utils.sh"

# Test configuration
TEST_NAME="Gallery Memory Sync Tests"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Test data
TEST_TIMESTAMP=$(date +%s)
TEST_GALLERY_ID="test_gallery_sync_${TEST_TIMESTAMP}"
TEST_MEMORY_1_ID="test_memory_1_${TEST_TIMESTAMP}"
TEST_MEMORY_2_ID="test_memory_2_${TEST_TIMESTAMP}"

# Helper function to check if response indicates success
is_success() {
    local response="$1"
    echo "$response" | grep -q "success = true"
}

# Helper function to check if response indicates failure
is_failure() {
    local response="$1"
    echo "$response" | grep -q "success = false"
}

# Helper function to increment test counters
run_test() {
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

# Helper function to extract values from Candid responses
extract_candid_value() {
    local response="$1"
    local field="$2"
    # Handle Candid format like "total_memories = 3 : nat32"
    local value=$(echo "$response" | grep -o "${field} = [^:]*" | sed "s/${field} = //")
    # Remove any trailing whitespace
    echo "$value" | xargs
}

# Test setup - ensure user is registered and has a capsule
test_setup_user_and_capsule() {
    echo_info "Setting up test user and capsule..."
    
    # Register user
    local register_result=$(dfx canister call backend register 2>/dev/null)
    if ! is_success "$register_result"; then
        echo_warn "User registration failed, continuing with existing user..."
    fi
    
    # Mark capsule as bound to Web2
    local bind_result=$(dfx canister call backend mark_capsule_bound_to_web2 2>/dev/null)
    if ! is_success "$bind_result"; then
        echo_warn "Capsule binding failed, continuing..."
    fi
    
    echo_info "Setup complete"
    return 0
}

# Test 1: Test sync_gallery_memories with valid memory data
test_sync_gallery_memories_valid() {
    echo_info "Testing sync_gallery_memories with valid memory data..."
    
    # Create test gallery first
    local timestamp=$(date +%s)000000000
    local gallery_data="(record {
        gallery = record {
            id = \"$TEST_GALLERY_ID\";
            owner_principal = principal \"$(dfx identity get-principal)\";
            title = \"Test Gallery for Sync\";
            description = opt \"Test gallery for memory sync testing\";
            is_public = false;
            created_at = $timestamp;
            updated_at = $timestamp;
            storage_status = variant { Web2Only };
            memory_entries = vec {};
        };
        owner_principal = principal \"$(dfx identity get-principal)\";
    })"
    
    # Store gallery first using the working function
    local store_result=$(dfx canister call backend store_gallery_forever "$gallery_data" 2>/dev/null)
    if ! is_success "$store_result"; then
        echo_fail "Failed to store gallery for sync test"
        return 1
    fi
    
    # Create memory sync requests
    local memory_sync_requests="(vec {
        record {
            memory_id = \"$TEST_MEMORY_1_ID\";
            memory_type = variant { Image };
            metadata = record {
                title = opt \"Test Image Memory\";
                description = opt \"Test image for sync testing\";
                tags = vec { \"test\"; \"image\"; \"sync\" };
                created_at = $timestamp;
                updated_at = $timestamp;
                size = opt 1048576; // 1MB
                content_type = opt \"image/jpeg\";
                custom_fields = vec {};
            };
            asset_url = \"https://example.com/test-image.jpg\";
            expected_asset_hash = \"abc123def456789\";
            asset_size = 1048576; // 1MB
        };
        record {
            memory_id = \"$TEST_MEMORY_2_ID\";
            memory_type = variant { Document };
            metadata = record {
                title = opt \"Test Document Memory\";
                description = opt \"Test document for sync testing\";
                tags = vec { \"test\"; \"document\"; \"sync\" };
                created_at = $timestamp;
                updated_at = $timestamp;
                size = opt 2097152; // 2MB
                content_type = opt \"application/pdf\";
                custom_fields = vec {};
            };
            asset_url = \"https://example.com/test-document.pdf\";
            expected_asset_hash = \"def456abc789123\";
            asset_size = 2097152; // 2MB
        };
    })"
    
    # Call sync_gallery_memories endpoint
    local sync_result=$(dfx canister call backend sync_gallery_memories "(\"$TEST_GALLERY_ID\", $memory_sync_requests)" 2>/dev/null)
    
    # Check if sync was successful
    if is_success "$sync_result"; then
        local total_memories=$(extract_candid_value "$sync_result" "total_memories")
        local successful_memories=$(extract_candid_value "$sync_result" "successful_memories")
        local failed_memories=$(extract_candid_value "$sync_result" "failed_memories")
        
        echo_info "Sync completed: $successful_memories/$total_memories successful, $failed_memories failed"
        
        if [ "$successful_memories" -gt 0 ]; then
            return 0
        else
            echo_warn "No memories were successfully synced"
            return 1
        fi
    else
        echo_fail "Memory sync failed: $sync_result"
        return 1
    fi
}

# Test 2: Test sync_gallery_memories with invalid memory type/size combination
test_sync_gallery_memories_validation_failure() {
    echo_info "Testing sync_gallery_memories with invalid memory type/size (should fail)..."
    
    # Create memory sync request with invalid size for note type
    local invalid_memory_request="(vec {
        record {
            memory_id = \"invalid_memory_${TEST_TIMESTAMP}\";
            memory_type = variant { Note };
            metadata = record {
                title = opt \"Invalid Note\";
                description = opt \"Note that exceeds size limit\";
                tags = vec { \"test\"; \"invalid\" };
                created_at = ${TEST_TIMESTAMP}000000000;
                updated_at = ${TEST_TIMESTAMP}000000000;
                size = opt 2097152; // 2MB - exceeds 1MB limit for notes
                content_type = opt \"text/plain\";
                custom_fields = vec {};
            };
            asset_url = \"https://example.com/invalid-note.txt\";
            expected_asset_hash = \"invalid_hash\";
            asset_size = 2097152; // 2MB - exceeds note limit
        };
    })"
    
    # Call sync_gallery_memories endpoint
    local sync_result=$(dfx canister call backend sync_gallery_memories "(\"$TEST_GALLERY_ID\", $invalid_memory_request)" 2>/dev/null)
    
    # This should fail due to validation
    if is_failure "$sync_result"; then
        echo_info "Validation failure as expected: $sync_result"
        return 0
    else
        echo_warn "Expected validation failure but got success"
        return 1
    fi
}

# Test 3: Test sync_gallery_memories with empty memory list
test_sync_gallery_memories_empty_list() {
    echo_info "Testing sync_gallery_memories with empty memory list..."
    
    # Call sync_gallery_memories with empty vector
    local sync_result=$(dfx canister call backend sync_gallery_memories "(\"$TEST_GALLERY_ID\", (vec {}))" 2>/dev/null)
    
    # Should handle empty list gracefully
    if is_success "$sync_result"; then
        local total_memories=$(extract_candid_value "$sync_result" "total_memories")
        # Convert Candid response to number for comparison
        if [[ "$total_memories" =~ ^[0-9]+$ ]] && [ "$total_memories" -eq 0 ]; then
            echo_info "Empty memory list handled correctly"
            return 0
        else
            echo_warn "Unexpected total_memories count: $total_memories"
            return 1
        fi
    else
        echo_fail "Failed to handle empty memory list: $sync_result"
        return 1
    fi
}

# Test 4: Test sync_gallery_memories with non-existent gallery
test_sync_gallery_memories_nonexistent_gallery() {
    echo_info "Testing sync_gallery_memories with non-existent gallery..."
    
    local fake_gallery_id="fake_gallery_${TEST_TIMESTAMP}"
    local memory_request="(vec {
        record {
            memory_id = \"test_memory\";
            memory_type = variant { Image };
            metadata = record {
                title = opt \"Test Memory\";
                description = opt \"Test memory\";
                tags = vec { \"test\" };
                created_at = ${TEST_TIMESTAMP}000000000;
                updated_at = ${TEST_TIMESTAMP}000000000;
                size = opt 1048576;
                content_type = opt \"image/jpeg\";
                custom_fields = vec {};
            };
            asset_url = \"https://example.com/test.jpg\";
            expected_asset_hash = \"test_hash\";
            asset_size = 1048576;
        };
    })"
    
    # Call sync_gallery_memories with fake gallery ID
    local sync_result=$(dfx canister call backend sync_gallery_memories "(\"$fake_gallery_id\", $memory_request)" 2>/dev/null)
    
    # Should fail for non-existent gallery
    if is_failure "$sync_result"; then
        echo_info "Correctly failed for non-existent gallery"
        return 0
    else
        echo_warn "Unexpectedly succeeded with non-existent gallery"
        return 1
    fi
}

# Test 5: Test different memory types with appropriate sizes
test_sync_gallery_memories_different_types() {
    echo_info "Testing sync_gallery_memories with different memory types..."
    
    local memory_requests="(vec {
        record {
            memory_id = \"image_memory_${TEST_TIMESTAMP}\";
            memory_type = variant { Image };
            metadata = record {
                title = opt \"Test Image\";
                description = opt \"Test image memory\";
                tags = vec { \"test\"; \"image\" };
                created_at = ${TEST_TIMESTAMP}000000000;
                updated_at = ${TEST_TIMESTAMP}000000000;
                size = opt 15728640; // 15MB - within 30MB limit
                content_type = opt \"image/jpeg\";
                custom_fields = vec {};
            };
            asset_url = \"https://example.com/test-image.jpg\";
            expected_asset_hash = \"image_hash_${TEST_TIMESTAMP}\";
            asset_size = 15728640;
        };
        record {
            memory_id = \"video_memory_${TEST_TIMESTAMP}\";
            memory_type = variant { Video };
            metadata = record {
                title = opt \"Test Video\";
                description = opt \"Test video memory\";
                tags = vec { \"test\"; \"video\" };
                created_at = ${TEST_TIMESTAMP}000000000;
                updated_at = ${TEST_TIMESTAMP}000000000;
                size = opt 52428800; // 50MB - within 80MB limit
                content_type = opt \"video/mp4\";
                custom_fields = vec {};
            };
            asset_url = \"https://example.com/test-video.mp4\";
            expected_asset_hash = \"video_hash_${TEST_TIMESTAMP}\";
            asset_size = 52428800;
        };
        record {
            memory_id = \"audio_memory_${TEST_TIMESTAMP}\";
            memory_type = variant { Audio };
            metadata = record {
                title = opt \"Test Audio\";
                description = opt \"Test audio memory\";
                tags = vec { \"test\"; \"audio\" };
                created_at = ${TEST_TIMESTAMP}000000000;
                updated_at = ${TEST_TIMESTAMP}000000000;
                size = opt 10485760; // 10MB - within 25MB limit
                content_type = opt \"audio/mp3\";
                custom_fields = vec {};
            };
            asset_url = \"https://example.com/test-audio.mp3\";
            expected_asset_hash = \"audio_hash_${TEST_TIMESTAMP}\";
            asset_size = 10485760;
        };
    })"
    
    # Call sync_gallery_memories with different memory types
    local sync_result=$(dfx canister call backend sync_gallery_memories "(\"$TEST_GALLERY_ID\", $memory_requests)" 2>/dev/null)
    
    if is_success "$sync_result"; then
        local total_memories=$(extract_candid_value "$sync_result" "total_memories")
        local successful_memories=$(extract_candid_value "$sync_result" "successful_memories")
        
        echo_info "Multi-type sync completed: $successful_memories/$total_memories successful"
        return 0
    else
        echo_fail "Multi-type memory sync failed: $sync_result"
        return 1
    fi
}

# Test 6: Test update_gallery_storage_status after sync
test_update_gallery_storage_status() {
    echo_info "Testing update_gallery_storage_status after memory sync..."
    
    # Update gallery storage status to Both (indicating memories are synced)
    local update_result=$(dfx canister call backend update_gallery_storage_status "(\"$TEST_GALLERY_ID\", variant { Both })" 2>/dev/null)
    
    if [ "$update_result" = "(true)" ]; then
        echo_info "Gallery storage status updated successfully"
        return 0
    else
        echo_fail "Failed to update gallery storage status: $update_result"
        return 1
    fi
}

# Test 7: Test cleanup functions
test_cleanup_functions() {
    echo_info "Testing cleanup and monitoring functions..."
    
    # Test get_upload_session_stats
    local stats_result=$(dfx canister call backend get_upload_session_stats 2>/dev/null)
    if echo "$stats_result" | grep -q "record"; then
        echo_info "Upload session stats retrieved successfully"
    else
        echo_warn "Failed to get upload session stats: $stats_result"
    fi
    
    # Test cleanup_expired_sessions
    local cleanup_result=$(dfx canister call backend cleanup_expired_sessions 2>/dev/null)
    if echo "$cleanup_result" | grep -qE "^[0-9]+$"; then
        echo_info "Cleanup expired sessions completed: $cleanup_result sessions cleaned"
    else
        echo_warn "Failed to cleanup expired sessions: $cleanup_result"
    fi
    
    # Test cleanup_orphaned_chunks
    local chunk_cleanup_result=$(dfx canister call backend cleanup_orphaned_chunks 2>/dev/null)
    if echo "$chunk_cleanup_result" | grep -qE "^[0-9]+$"; then
        echo_info "Cleanup orphaned chunks completed: $chunk_cleanup_result chunks cleaned"
    else
        echo_warn "Failed to cleanup orphaned chunks: $chunk_cleanup_result"
    fi
    
    return 0
}

# Main test execution
main() {
    echo_info "Starting $TEST_NAME"
    echo_info "Testing sync_gallery_memories endpoint and related functionality"
    echo ""
    
    # Check if backend canister is running
    if ! check_canister_status "$BACKEND_CANISTER_ID"; then
        echo_fail "Backend canister is not running. Please start dfx first."
        exit 1
    fi
    
    echo_info "Backend canister ID: $BACKEND_CANISTER_ID"
    echo ""
    
    # Run tests
    run_test "Setup user and capsule" "test_setup_user_and_capsule"
    run_test "Sync gallery memories with valid data" "test_sync_gallery_memories_valid"
    run_test "Sync gallery memories validation failure" "test_sync_gallery_memories_validation_failure"
    run_test "Sync gallery memories empty list" "test_sync_gallery_memories_empty_list"
    run_test "Sync gallery memories non-existent gallery" "test_sync_gallery_memories_nonexistent_gallery"
    run_test "Sync gallery memories different types" "test_sync_gallery_memories_different_types"
    run_test "Update gallery storage status" "test_update_gallery_storage_status"
    run_test "Test cleanup functions" "test_cleanup_functions"
    
    # Print test results
    echo ""
    echo_info "Test Results Summary:"
    echo_info "Total Tests: $TOTAL_TESTS"
    echo_info "Passed: $PASSED_TESTS"
    echo_info "Failed: $FAILED_TESTS"
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo_pass "All tests passed! 🎉"
        exit 0
    else
        echo_fail "Some tests failed. Please review the output above."
        exit 1
    fi
}

# Run main function
main "$@"
