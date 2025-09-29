#!/bin/bash

# Gallery Test Utilities
# Shared functions for gallery-related test scripts

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

# Helper function to get a capsule ID for testing
get_test_capsule_id() {
    local capsule_result=$(dfx canister call backend capsules_read_basic "(null)" 2>/dev/null)
    local capsule_id=""
    
    if [[ $capsule_result == *"null"* ]]; then
        echo_info "No capsule found, creating one first..."
        local create_result=$(dfx canister call backend capsules_create "(null)" 2>/dev/null)
        capsule_id=$(echo "$create_result" | grep -o 'capsule_id = opt "[^"]*"' | sed 's/capsule_id = opt "//' | sed 's/"//')
    else
        capsule_id=$(echo "$capsule_result" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')
    fi
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to get capsule ID for testing"
        return 1
    fi
    
    echo "$capsule_id"
}

# Helper function to create test memory data
create_test_memory_data() {
    local content="$1"
    local name="$2"
    
    # Convert text to base64 for binary data
    local encoded_content=$(echo -n "$content" | base64)
    
    cat << EOF
blob "$encoded_content"
EOF
}

# Helper function to create test asset metadata
create_test_asset_metadata() {
    local name="$1"
    local content="$2"
    
    cat << EOF
(variant {
  Document = record {
    base = record {
      name = "test_${name}.txt";
      description = opt "Test memory for gallery testing";
      tags = vec { "test"; "gallery"; };
      asset_type = variant { Original };
      bytes = $(echo -n "$content" | wc -c);
      mime_type = "text/plain";
      sha256 = null;
      width = null;
      height = null;
      url = null;
      storage_key = null;
      bucket = null;
      asset_location = null;
      processing_status = null;
      processing_error = null;
      created_at = 0;
      updated_at = 0;
      deleted_at = null;
    };
    page_count = null;
    document_type = null;
    language = null;
    word_count = null;
  }
})
EOF
}

# Helper function to upload a test memory and return its ID
upload_test_memory() {
    local content="$1"
    local name="$2"
    
    # Generate a unique idempotency key
    local timestamp=$(date +%s)
    local idem="gallery_test_${timestamp}_${RANDOM}_${name}"
    
    local memory_bytes=$(create_test_memory_data "$content" "$name")
    local asset_metadata=$(create_test_asset_metadata "$name" "$content")
    local capsule_id=$(get_test_capsule_id)
    
    if [[ -z "$capsule_id" ]]; then
        return 1
    fi
    
    # Use the new API format: memories_create(capsule_id, bytes, asset_metadata, idem)
    local result=$(dfx canister call backend memories_create "(\"$capsule_id\", $memory_bytes, $asset_metadata, \"$idem\")" 2>/dev/null)

    # Check for successful Result<MemoryId, Error> response
    if echo "$result" | grep -q "variant {" && echo "$result" | grep -q "Ok =" && echo "$result" | grep -q "mem_"; then
        local memory_id=$(echo "$result" | grep -o '"mem_[^"]*"' | sed 's/"//g')
        echo "$memory_id"
        return 0
    else
        echo ""
        return 1
    fi
}

# Helper function to create basic gallery data
create_basic_gallery_data() {
    local gallery_id="$1"
    local title="$2"
    local description="$3"
    local is_public="$4"
    local storage_status="${5:-ICPOnly}"
    
    local timestamp=$(date +%s)000000000
    
    # Generate gallery ID if none provided
    if [[ -z "$gallery_id" ]]; then
        gallery_id="gallery_${timestamp}_${RANDOM}"
    fi
    
    cat << EOF
(record {
  gallery = record {
    id = "$gallery_id";
    owner_principal = principal "$(dfx identity get-principal)";
    title = "$title";
    description = opt "$description";
    is_public = $is_public;
    created_at = $timestamp;
    updated_at = $timestamp;
    storage_location = variant { $storage_status };
    memory_entries = vec {};
    bound_to_neon = false;
  };
  owner_principal = principal "$(dfx identity get-principal)";
})
EOF
}

# Helper function to create gallery data with memory entries
create_gallery_data_with_memories() {
    local gallery_id="$1"
    local title="$2"
    local description="$3"
    local is_public="$4"
    local memory_id1="$5"
    local memory_id2="${6:-}"
    local storage_status="${7:-ICPOnly}"
    
    local timestamp=$(date +%s)000000000
    
    # Generate gallery ID if none provided
    if [[ -z "$gallery_id" ]]; then
        gallery_id="gallery_${timestamp}_${RANDOM}"
    fi
    
    # Build memory entries
    local memory_entries="vec {
      record {
        memory_id = \"$memory_id1\";
        position = 1;
        gallery_caption = opt \"First memory in gallery\";
        is_featured = true;
        gallery_metadata = \"{}\";
      };"
    
    if [[ -n "$memory_id2" ]]; then
        memory_entries="$memory_entries
      record {
        memory_id = \"$memory_id2\";
        position = 2;
        gallery_caption = opt \"Second memory\";
        is_featured = false;
        gallery_metadata = \"{\\\"tags\\\": [\\\"secondary\\\"]}\";
      };"
    fi
    
    memory_entries="$memory_entries
    }"
    
    cat << EOF
(record {
  gallery = record {
    id = "$gallery_id";
    owner_principal = principal "$(dfx identity get-principal)";
    title = "$title";
    description = opt "$description";
    is_public = $is_public;
    created_at = $timestamp;
    updated_at = $timestamp;
    storage_location = variant { $storage_status };
    memory_entries = $memory_entries;
    bound_to_neon = false;
  };
  owner_principal = principal "$(dfx identity get-principal)";
})
EOF
}

# Helper function to extract gallery ID from creation response
extract_gallery_id() {
    local response="$1"
    echo "$response" | grep -o 'id = "[^"]*"' | sed 's/id = "\([^"]*\)"/\1/' | head -1
}

# Helper function to extract storage status from response
extract_storage_status() {
    local response="$1"
    echo "$response" | grep -o 'storage_location = variant { [^}]*}' | sed 's/storage_location = variant { \([^}]*\) }/\1/'
}

# Helper function to create a test gallery and return its ID
create_test_gallery() {
    local title="$1"
    local description="$2"
    local is_public="$3"
    local storage_status="${4:-ICPOnly}"
    
    local timestamp=$(date +%s)
    local gallery_id="test_gallery_${timestamp}_${RANDOM}"
    
    local gallery_data=$(create_basic_gallery_data "$gallery_id" "$title" "$description" "$is_public" "$storage_status")
    local result=$(dfx canister call backend galleries_create "$gallery_data" 2>/dev/null)
    
    if is_success "$result"; then
        local returned_id=$(extract_gallery_id "$result")
        echo "${returned_id:-$gallery_id}"
        return 0
    else
        echo ""
        return 1
    fi
}

# Helper function to create a test gallery with memories and return its ID
create_test_gallery_with_memories() {
    local title="$1"
    local description="$2"
    local is_public="$3"
    local memory_content1="$4"
    local memory_content2="${5:-}"
    local storage_status="${6:-ICPOnly}"
    
    local timestamp=$(date +%s)
    local gallery_id="test_gallery_${timestamp}_${RANDOM}"
    
    # Upload memories first
    local memory_id1=$(upload_test_memory "$memory_content1" "memory1")
    if [[ -z "$memory_id1" ]]; then
        echo ""
        return 1
    fi
    
    local memory_id2=""
    if [[ -n "$memory_content2" ]]; then
        memory_id2=$(upload_test_memory "$memory_content2" "memory2")
        if [[ -z "$memory_id2" ]]; then
            echo ""
            return 1
        fi
    fi
    
    # Create gallery with memories
    local gallery_data=$(create_gallery_data_with_memories "$gallery_id" "$title" "$description" "$is_public" "$memory_id1" "$memory_id2" "$storage_status")
    local result=$(dfx canister call backend galleries_create "$gallery_data" 2>/dev/null)
    
    if is_success "$result"; then
        local returned_id=$(extract_gallery_id "$result")
        echo "${returned_id:-$gallery_id}"
        return 0
    else
        echo ""
        return 1
    fi
}

# Helper function to setup user and capsule for testing
setup_user_and_capsule() {
    echo_info "Setting up test user and capsule..."
    
    # Register user
    local nonce=$(date +%s)
    local register_result=$(dfx canister call backend register_with_nonce "test_nonce_${nonce}" 2>/dev/null)
    if ! is_success "$register_result"; then
        echo_warn "User registration failed, continuing with existing user..."
    fi
    
    # Mark capsule as bound to Web2 (optional)
    local bind_result=$(dfx canister call backend 'capsules_bind_neon("Capsule", "", true)' 2>/dev/null)
    if ! is_success "$bind_result"; then
        echo_warn "Capsule binding failed, continuing..."
    fi
    
    echo_info "Setup complete"
    return 0
}

# Helper function to run a test with proper counting
run_gallery_test() {
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
