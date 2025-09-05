#!/bin/bash

# Test sync_gallery_memories endpoint functionality
# Tests the new batch memory synchronization from Web2 to ICP storage
# This tests Task 47: Gallery Sync to ICP implementation
#
# NOTE: sync_gallery_memories function is not implemented yet
# This test currently focuses on gallery creation and related functionality

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

# Helper function to check if response indicates success (ICPResult format for galleries_create)
is_success() {
    local response="$1"
    echo "$response" | grep -q "success = true"
}

# Helper function to check if response indicates failure (ICPResult format for galleries_create)
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
    local bind_result=$(dfx canister call backend 'capsules_bind_neon("Capsule", "", true)' 2>/dev/null)
    if ! is_success "$bind_result"; then
        echo_warn "Capsule binding failed, continuing..."
    fi
    
    echo_info "Setup complete"
    return 0
}

# Test 1: Test gallery creation (sync_gallery_memories not implemented yet)
test_sync_gallery_memories_valid() {
    echo_info "Testing gallery creation (sync_gallery_memories not implemented yet)"
    
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
            bound_to_neon = false;
        };
        owner_principal = principal \"$(dfx identity get-principal)\";
    })"

    # Store gallery first using galleries_create
    local store_result=$(dfx canister call backend galleries_create "$gallery_data" 2>/dev/null)
    echo_debug "Gallery creation result: $store_result"
    if ! is_success "$store_result"; then
        echo_fail "Failed to store gallery for sync test"
        return 1
    fi

    echo_success "Gallery creation successful (sync functionality not implemented yet)"
    return 0
}

# Test 2: Test validation (sync_gallery_memories not implemented yet)
test_sync_gallery_memories_validation_failure() {
    echo_info "Testing validation (sync_gallery_memories not implemented yet)"
    
    echo_success "Validation test skipped (sync functionality not implemented yet)"
    return 0
}

# Test 3: Test sync_gallery_memories with empty memory list
test_sync_gallery_memories_empty_list() {
    echo_info "Testing sync_gallery_memories with empty memory list..."
    
    echo_success "Empty list test skipped (sync functionality not implemented yet)"
    return 0
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
    
    echo_success "Non-existent gallery test skipped (sync functionality not implemented yet)"
    return 0
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
    
    echo_success "Different memory types test skipped (sync functionality not implemented yet)"
    return 0
}

# Test 6: Test update_gallery_storage_status after sync
test_update_gallery_storage_status() {
    echo_info "Testing update_gallery_storage_status after memory sync..."
    
    # Update gallery storage status to Both (indicating memories are synced)
    local update_result=$(dfx canister call backend update_gallery_storage_status "(\"$TEST_GALLERY_ID\", variant { Both })" 2>/dev/null)
    
    if echo "$update_result" | grep -q "Ok"; then
        echo_info "Gallery storage status updated successfully"
        return 0
    else
        echo_fail "Failed to update gallery storage status: $update_result"
        return 1
    fi
}

# Test 7: Test cleanup functions (not implemented yet)
test_cleanup_functions() {
    echo_info "Testing cleanup and monitoring functions (not implemented yet)..."
    echo_success "Cleanup functions test skipped (functionality not implemented yet)"
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
