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
# Delegates to create_test_memory (test_utils.sh), which uses the current
# 10-argument memories_create signature and returns a UUID memory id.
upload_test_memory() {
    local content="$1"
    local name="$2"

    local capsule_id=$(get_test_capsule_id)
    if [[ -z "$capsule_id" ]]; then
        return 1
    fi

    local memory_bytes=$(create_test_memory_data "$content" "$name")
    create_test_memory "$capsule_id" "test_${name}.txt" "Test memory for gallery testing" '"test"; "gallery";' "$memory_bytes"
}

# Map the legacy storage status argument onto the current storage_location
# (vec BlobHosting): ICPOnly -> Icp, Web2Only -> VercelBlob, Both -> both.
storage_location_vec() {
    case "${1:-ICPOnly}" in
        Web2Only) echo "vec { variant { VercelBlob } }" ;;
        Both)     echo "vec { variant { Icp }; variant { VercelBlob } }" ;;
        *)        echo "vec { variant { Icp } }" ;;
    esac
}

sharing_status_variant() {
    if [[ "$1" == "true" ]]; then echo "variant { Public }"; else echo "variant { Private }"; fi
}

slugify() {
    echo "$1" | tr '[:upper:]' '[:lower:]' | sed -E 's/[^a-z0-9]+/-/g; s/^-+|-+$//g'
}

# Build a GalleryData record in the current Candid shape:
#   Gallery { id; capsule_id; metadata: FolderMetadata; cover_memory_id;
#             created_at; updated_at; items: vec GalleryItem; access_entries }
build_gallery_data() {
    local gallery_id="$1" title="$2" description="$3" is_public="$4" storage_status="$5" items="$6" total="$7"
    local timestamp=$(date +%s)000000000
    local capsule_id=$(get_test_capsule_id)
    local principal=$(dfx identity get-principal)

    cat << GALLERY_EOF
(record {
  gallery = record {
    id = "$gallery_id";
    capsule_id = "$capsule_id";
    metadata = record {
      name = "$(slugify "$title")";
      title = opt "$title";
      description = opt "$description";
      sharing_status = $(sharing_status_variant "$is_public");
      storage_location = $(storage_location_vec "$storage_status");
      total_memories = $total;
      shared_count = 0;
    };
    cover_memory_id = null;
    created_at = $timestamp;
    updated_at = $timestamp;
    items = $items;
    access_entries = vec {};
  };
  owner_principal = principal "$principal";
})
GALLERY_EOF
}

# Helper function to create basic gallery data
create_basic_gallery_data() {
    local gallery_id="$1"
    local title="$2"
    local description="$3"
    local is_public="$4"
    local storage_status="${5:-ICPOnly}"

    if [[ -z "$gallery_id" ]]; then
        gallery_id="gallery_$(date +%s)_${RANDOM}"
    fi

    build_gallery_data "$gallery_id" "$title" "$description" "$is_public" "$storage_status" "vec {}" 0
}

# Helper function to create gallery data with memory entries (GalleryItem)
create_gallery_data_with_memories() {
    local gallery_id="$1"
    local title="$2"
    local description="$3"
    local is_public="$4"
    local memory_id1="$5"
    local memory_id2="${6:-}"
    local storage_status="${7:-ICPOnly}"

    if [[ -z "$gallery_id" ]]; then
        gallery_id="gallery_$(date +%s)_${RANDOM}"
    fi

    local items="vec {
      record {
        memory_id = \"$memory_id1\";
        memory_type = variant { Document };
        position = 1;
        caption = opt \"First memory in gallery\";
        metadata = vec {};
      };"
    local total=1
    if [[ -n "$memory_id2" ]]; then
        items="$items
      record {
        memory_id = \"$memory_id2\";
        memory_type = variant { Document };
        position = 2;
        caption = opt \"Second memory\";
        metadata = vec { record { \"tags\"; \"secondary\" } };
      };"
        total=2
    fi
    items="$items
    }"

    build_gallery_data "$gallery_id" "$title" "$description" "$is_public" "$storage_status" "$items" "$total"
}

# Helper function to extract gallery ID from creation response
extract_gallery_id() {
    local response="$1"
    echo "$response" | grep -o 'id = "[^"]*"' | sed 's/id = "\([^"]*\)"/\1/' | head -1
}

# Helper function to extract storage status from response, reported in the
# legacy names the tests use (ICPOnly / Web2Only / Both)
extract_storage_status() {
    local response="$1"
    local loc=$(echo "$response" | tr '\n' ' ' | grep -o 'storage_location = vec {[^;]*;[^;]*;\?[^}]*}' | head -1)
    local icp=0 web2=0
    [[ "$loc" == *"Icp"* ]] && icp=1
    [[ "$loc" == *"VercelBlob"* || "$loc" == *"S3"* || "$loc" == *"Neon"* ]] && web2=1
    if (( icp && web2 )); then echo "Both"
    elif (( icp )); then echo "ICPOnly"
    elif (( web2 )); then echo "Web2Only"
    fi
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
