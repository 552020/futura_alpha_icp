#!/bin/bash

# Test gallery upload functionality
# Tests store_gallery_forever endpoint, gallery creation, metadata, and retrieval

# Load test configuration and utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../test_config.sh"
source "$SCRIPT_DIR/../../test_utils.sh"

# Test configuration
TEST_NAME="Gallery Upload Functionality Tests"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

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

# Helper function to create test memory data
create_test_memory_data() {
    local content="$1"
    local name="$2"
    
    # Convert text to base64 for binary data
    local encoded_content=$(echo -n "$content" | base64)
    
    cat << EOF
(record {
  blob_ref = record {
    kind = variant { ICPCapsule };
    locator = "memory_${name}";
    hash = null;
  };
  data = opt blob "$encoded_content";
})
EOF
}

# Helper function to upload a test memory and return its ID
upload_test_memory() {
    local content="$1"
    local name="$2"
    
    local memory_data=$(create_test_memory_data "$content" "$name")
    local result=$(dfx canister call backend add_memory_to_capsule "$memory_data" 2>/dev/null)
    
    if echo "$result" | grep -q "success = true"; then
        local memory_id=$(echo "$result" | grep -o 'memory_id = opt "[^"]*"' | sed 's/memory_id = opt "\([^"]*\)"/\1/')
        echo "$memory_id"
        return 0
    else
        echo ""
        return 1
    fi
}

# Helper function to create gallery data
create_gallery_data() {
    local gallery_type="$1"
    local memory_id1="$2"
    local memory_id2="$3"
    
    # Get current timestamp in nanoseconds
    local timestamp=$(date +%s)000000000
    local gallery_id="gallery_${timestamp}"
    
    case "$gallery_type" in
        "basic")
            cat << EOF
(record {
  gallery = record {
    id = "$gallery_id";
    owner_principal = principal "$(dfx identity get-principal)";
    title = "Basic Test Gallery";
    description = opt "A simple test gallery";
    is_public = true;
    created_at = $timestamp;
    updated_at = $timestamp;
    storage_status = variant { ICPOnly };
    memory_entries = vec {
      record {
        memory_id = "$memory_id1";
        position = 1;
        gallery_caption = opt "First memory in gallery";
        is_featured = true;
        gallery_metadata = "{}";
      };
    };
  };
  owner_principal = principal "$(dfx identity get-principal)";
})
EOF
            ;;
        "multiple_memories")
            cat << EOF
(record {
  gallery = record {
    id = "$gallery_id";
    owner_principal = principal "$(dfx identity get-principal)";
    title = "Multi-Memory Gallery";
    description = opt "Gallery with multiple memories";
    is_public = false;
    created_at = $timestamp;
    updated_at = $timestamp;
    storage_status = variant { ICPOnly };
    memory_entries = vec {
      record {
        memory_id = "$memory_id1";
        position = 1;
        gallery_caption = opt "First memory";
        is_featured = true;
        gallery_metadata = "{\"tags\": [\"featured\"]}";
      };
      record {
        memory_id = "$memory_id2";
        position = 2;
        gallery_caption = opt "Second memory";
        is_featured = false;
        gallery_metadata = "{\"tags\": [\"secondary\"]}";
      };
    };
  };
  owner_principal = principal "$(dfx identity get-principal)";
})
EOF
            ;;
        "empty_gallery")
            cat << EOF
(record {
  gallery = record {
    id = "$gallery_id";
    owner_principal = principal "$(dfx identity get-principal)";
    title = "Empty Gallery";
    description = null;
    is_public = true;
    created_at = $timestamp;
    updated_at = $timestamp;
    storage_status = variant { ICPOnly };
    memory_entries = vec {};
  };
  owner_principal = principal "$(dfx identity get-principal)";
})
EOF
            ;;
        "private_gallery")
            cat << EOF
(record {
  gallery = record {
    id = "$gallery_id";
    owner_principal = principal "$(dfx identity get-principal)";
    title = "Private Gallery";
    description = opt "This is a private gallery";
    is_public = false;
    created_at = $timestamp;
    updated_at = $timestamp;
    storage_status = variant { ICPOnly };
    memory_entries = vec {
      record {
        memory_id = "$memory_id1";
        position = 1;
        gallery_caption = null;
        is_featured = false;
        gallery_metadata = "{\"private\": true}";
      };
    };
  };
  owner_principal = principal "$(dfx identity get-principal)";
})
EOF
            ;;
        *)
            echo ""
            ;;
    esac
}

# Test functions for store_gallery_forever

test_store_basic_gallery() {
    # Upload a test memory first
    local memory_id=$(upload_test_memory "Content for basic gallery" "basic_gallery_memory")
    
    if [ -z "$memory_id" ]; then
        echo_info "Failed to upload test memory for basic gallery"
        return 1
    fi
    
    # Create and store basic gallery
    local gallery_data=$(create_gallery_data "basic" "$memory_id")
    local result=$(dfx canister call backend store_gallery_forever "$gallery_data" 2>/dev/null)
    
    if is_success "$result"; then
        local gallery_id=$(echo "$result" | grep -o 'gallery_id = opt "[^"]*"' | sed 's/gallery_id = opt "\([^"]*\)"/\1/' | head -1)
        echo_info "Basic gallery creation successful with ID: $gallery_id"
        return 0
    else
        echo_info "Basic gallery creation failed: $result"
        return 1
    fi
}

test_store_gallery_with_multiple_memories() {
    # Upload test memories first
    local memory_id1=$(upload_test_memory "First memory content" "multi_gallery_memory1")
    local memory_id2=$(upload_test_memory "Second memory content" "multi_gallery_memory2")
    
    if [ -z "$memory_id1" ] || [ -z "$memory_id2" ]; then
        echo_info "Failed to upload test memories for multi-memory gallery"
        return 1
    fi
    
    # Create and store gallery with multiple memories
    local gallery_data=$(create_gallery_data "multiple_memories" "$memory_id1" "$memory_id2")
    local result=$(dfx canister call backend store_gallery_forever "$gallery_data" 2>/dev/null)
    
    if is_success "$result"; then
        local gallery_id=$(echo "$result" | grep -o 'gallery_id = opt "[^"]*"' | sed 's/gallery_id = opt "\([^"]*\)"/\1/' | head -1)
        echo_info "Multi-memory gallery creation successful with ID: $gallery_id"
        return 0
    else
        echo_info "Multi-memory gallery creation failed: $result"
        return 1
    fi
}

test_store_empty_gallery() {
    # Create and store empty gallery (no memories)
    local gallery_data=$(create_gallery_data "empty_gallery")
    local result=$(dfx canister call backend store_gallery_forever "$gallery_data" 2>/dev/null)
    
    if is_success "$result"; then
        local gallery_id=$(echo "$result" | grep -o 'gallery_id = opt "[^"]*"' | sed 's/gallery_id = opt "\([^"]*\)"/\1/' | head -1)
        echo_info "Empty gallery creation successful with ID: $gallery_id"
        return 0
    else
        echo_info "Empty gallery creation failed: $result"
        return 1
    fi
}

test_store_private_gallery() {
    # Upload a test memory first
    local memory_id=$(upload_test_memory "Private gallery content" "private_gallery_memory")
    
    if [ -z "$memory_id" ]; then
        echo_info "Failed to upload test memory for private gallery"
        return 1
    fi
    
    # Create and store private gallery
    local gallery_data=$(create_gallery_data "private_gallery" "$memory_id")
    local result=$(dfx canister call backend store_gallery_forever "$gallery_data" 2>/dev/null)
    
    if is_success "$result"; then
        local gallery_id=$(echo "$result" | grep -o 'gallery_id = opt "[^"]*"' | sed 's/gallery_id = opt "\([^"]*\)"/\1/' | head -1)
        echo_info "Private gallery creation successful with ID: $gallery_id"
        return 0
    else
        echo_info "Private gallery creation failed: $result"
        return 1
    fi
}

test_store_gallery_with_metadata() {
    # Upload a test memory first
    local memory_id=$(upload_test_memory "Content with metadata" "metadata_gallery_memory")
    
    if [ -z "$memory_id" ]; then
        echo_info "Failed to upload test memory for metadata gallery"
        return 1
    fi
    
    # Create gallery with rich metadata
    local timestamp=$(date +%s)000000000
    local gallery_id="gallery_metadata_${timestamp}"
    
    local gallery_data=$(cat << EOF
(record {
  gallery = record {
    id = "$gallery_id";
    owner_principal = principal "$(dfx identity get-principal)";
    title = "Gallery with Rich Metadata";
    description = opt "This gallery has detailed metadata for testing";
    is_public = true;
    created_at = $timestamp;
    updated_at = $timestamp;
    storage_status = variant { ICPOnly };
    memory_entries = vec {
      record {
        memory_id = "$memory_id";
        position = 1;
        gallery_caption = opt "Memory with detailed metadata";
        is_featured = true;
        gallery_metadata = "{\"tags\": [\"test\", \"metadata\"], \"category\": \"experimental\", \"rating\": 5}";
      };
    };
  };
  owner_principal = principal "$(dfx identity get-principal)";
})
EOF
)
    
    local result=$(dfx canister call backend store_gallery_forever "$gallery_data" 2>/dev/null)
    
    if is_success "$result"; then
        echo_info "Gallery with metadata creation successful"
        return 0
    else
        echo_info "Gallery with metadata creation failed: $result"
        return 1
    fi
}

# Test functions for get_gallery_by_id

test_retrieve_gallery_by_id() {
    # Upload a test memory first
    local memory_id=$(upload_test_memory "Content for retrieval test" "retrieval_gallery_memory")
    
    if [ -z "$memory_id" ]; then
        echo_info "Failed to upload test memory for retrieval test"
        return 1
    fi
    
    # Create and store gallery
    local gallery_data=$(create_gallery_data "basic" "$memory_id")
    local store_result=$(dfx canister call backend store_gallery_forever "$gallery_data" 2>/dev/null)
    
    if ! is_success "$store_result"; then
        echo_info "Failed to store gallery for retrieval test"
        return 1
    fi
    
    # Extract gallery ID from store result (take first match to avoid duplicates)
    local gallery_id=$(echo "$store_result" | grep -o 'gallery_id = opt "[^"]*"' | sed 's/gallery_id = opt "\([^"]*\)"/\1/' | head -1)
    
    if [ -z "$gallery_id" ]; then
        echo_info "Failed to extract gallery ID from store result: $store_result"
        return 1
    fi
    
    # Retrieve the gallery (add small delay for persistence)
    echo_info "Attempting to retrieve gallery with ID: $gallery_id"
    sleep 1
    local retrieve_result=$(dfx canister call backend get_gallery_by_id "(\"$gallery_id\")" 2>/dev/null)
    
    # Check if gallery was retrieved successfully
    if echo "$retrieve_result" | grep -q "opt record" && echo "$retrieve_result" | grep -q "title = \"Basic Test Gallery\""; then
        echo_info "Gallery retrieval successful for ID: $gallery_id"
        return 0
    else
        echo_info "Gallery retrieval failed: $retrieve_result"
        return 1
    fi
}

test_retrieve_nonexistent_gallery() {
    # Try to retrieve a gallery that doesn't exist
    local fake_id="nonexistent_gallery_12345"
    local result=$(dfx canister call backend get_gallery_by_id "(\"$fake_id\")" 2>/dev/null)
    
    # Should return null for non-existent gallery
    if echo "$result" | grep -q "(null)"; then
        echo_info "Correctly returned null for non-existent gallery"
        return 0
    else
        echo_info "Unexpected result for non-existent gallery: $result"
        return 1
    fi
}

test_gallery_memory_associations() {
    # Upload test memories
    local memory_id1=$(upload_test_memory "First associated memory" "assoc_memory1")
    local memory_id2=$(upload_test_memory "Second associated memory" "assoc_memory2")
    
    if [ -z "$memory_id1" ] || [ -z "$memory_id2" ]; then
        echo_info "Failed to upload test memories for association test"
        return 1
    fi
    
    # Create gallery with multiple memories
    local gallery_data=$(create_gallery_data "multiple_memories" "$memory_id1" "$memory_id2")
    local store_result=$(dfx canister call backend store_gallery_forever "$gallery_data" 2>/dev/null)
    
    if ! is_success "$store_result"; then
        echo_info "Failed to store gallery for association test"
        return 1
    fi
    
    # Extract gallery ID (take first match to avoid duplicates)
    local gallery_id=$(echo "$store_result" | grep -o 'gallery_id = opt "[^"]*"' | sed 's/gallery_id = opt "\([^"]*\)"/\1/' | head -1)
    
    # Retrieve and verify memory associations
    echo_info "Retrieving gallery for association test with ID: $gallery_id"
    sleep 1
    local retrieve_result=$(dfx canister call backend get_gallery_by_id "(\"$gallery_id\")" 2>/dev/null)
    
    # Check if both memory IDs are present in the gallery
    if echo "$retrieve_result" | grep -q "$memory_id1" && echo "$retrieve_result" | grep -q "$memory_id2"; then
        echo_info "Gallery memory associations verified successfully"
        return 0
    else
        echo_info "Gallery memory associations verification failed. Expected: $memory_id1, $memory_id2"
        echo_info "Actual result: $retrieve_result"
        return 1
    fi
}

test_gallery_metadata_preservation() {
    # Upload a test memory
    local memory_id=$(upload_test_memory "Content for metadata test" "metadata_preservation_memory")
    
    if [ -z "$memory_id" ]; then
        echo_info "Failed to upload test memory for metadata preservation test"
        return 1
    fi
    
    # Create gallery with specific metadata
    local gallery_data=$(create_gallery_data "basic" "$memory_id")
    local store_result=$(dfx canister call backend store_gallery_forever "$gallery_data" 2>/dev/null)
    
    if ! is_success "$store_result"; then
        echo_info "Failed to store gallery for metadata preservation test"
        return 1
    fi
    
    # Extract gallery ID (take first match to avoid duplicates)
    local gallery_id=$(echo "$store_result" | grep -o 'gallery_id = opt "[^"]*"' | sed 's/gallery_id = opt "\([^"]*\)"/\1/' | head -1)
    
    # Retrieve and verify metadata preservation
    echo_info "Retrieving gallery for metadata test with ID: $gallery_id"
    sleep 1
    local retrieve_result=$(dfx canister call backend get_gallery_by_id "(\"$gallery_id\")" 2>/dev/null)
    
    # Check if key metadata fields are preserved
    if echo "$retrieve_result" | grep -q "title = \"Basic Test Gallery\"" && \
       echo "$retrieve_result" | grep -q "description = opt \"A simple test gallery\"" && \
       echo "$retrieve_result" | grep -q "is_public = true"; then
        echo_info "Gallery metadata preservation verified successfully"
        return 0
    else
        echo_info "Gallery metadata preservation verification failed. Result: $retrieve_result"
        return 1
    fi
}

# Main test execution
main() {
    echo "========================================="
    echo "Starting $TEST_NAME"
    echo "========================================="
    echo ""
    
    # Check if backend canister ID is set
    if [ -z "$BACKEND_CANISTER_ID" ]; then
        echo_fail "BACKEND_CANISTER_ID not set in test_config.sh"
        echo_info "Please set the backend canister ID before running tests"
        exit 1
    fi
    
    # Check if dfx is available
    if ! command -v dfx &> /dev/null; then
        echo_fail "dfx command not found"
        echo_info "Please install dfx and ensure it's in your PATH"
        exit 1
    fi
    
    # Register user first (required for gallery operations)
    echo_info "Registering user for gallery operations..."
    local register_result=$(dfx canister call backend register 2>/dev/null)
    if ! echo "$register_result" | grep -q "true"; then
        echo_warn "User registration returned: $register_result"
    fi
    
    # Run store_gallery_forever tests
    echo_info "=== Testing store_gallery_forever endpoint ==="
    run_test "Store basic gallery" "test_store_basic_gallery"
    run_test "Store gallery with multiple memories" "test_store_gallery_with_multiple_memories"
    run_test "Store empty gallery" "test_store_empty_gallery"
    run_test "Store private gallery" "test_store_private_gallery"
    run_test "Store gallery with metadata" "test_store_gallery_with_metadata"
    
    echo_info "=== Testing get_gallery_by_id endpoint ==="
    run_test "Retrieve gallery by ID" "test_retrieve_gallery_by_id"
    run_test "Retrieve non-existent gallery" "test_retrieve_nonexistent_gallery"
    
    echo_info "=== Testing gallery functionality ==="
    run_test "Gallery memory associations" "test_gallery_memory_associations"
    run_test "Gallery metadata preservation" "test_gallery_metadata_preservation"
    
    # Print test summary
    echo "========================================="
    echo "Test Summary for $TEST_NAME"
    echo "========================================="
    echo "Total tests: $TOTAL_TESTS"
    echo "Passed: $PASSED_TESTS"
    echo "Failed: $FAILED_TESTS"
    echo ""
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo_pass "All tests passed!"
        exit 0
    else
        echo_fail "$FAILED_TESTS test(s) failed"
        exit 1
    fi
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi