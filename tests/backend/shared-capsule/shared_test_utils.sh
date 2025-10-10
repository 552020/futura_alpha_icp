#!/bin/bash

# Shared Test Utilities for Memory Testing
# This file provides standardized utilities for all memory test files

# Helper function to convert base64 string to Candid vec format
b64_to_vec() {
    local base64_content="$1"
    local hex_content=$(echo -n "$base64_content" | base64 -d | xxd -p -c 256 | tr -d '\n')
    local vec_content=""
    
    # Convert hex to vec format
    for ((i=0; i<${#hex_content}; i+=2)); do
        local byte="${hex_content:$i:2}"
        if [[ -n "$vec_content" ]]; then
            vec_content="${vec_content}; 0x${byte}"
        else
            vec_content="0x${byte}"
        fi
    done
    
    echo "vec { $vec_content }"
}

# Helper function to create test memory with standardized API
create_test_memory() {
    local capsule_id="$1"
    local memory_name="$2"
    local memory_description="$3"
    local memory_tags="$4"
    local memory_bytes="$5"
    local canister_id="${6:-backend}"
    local identity="${7:-default}"
    
    # Convert blob format to vec format for new API
    local inline_data
    if [[ "$memory_bytes" =~ ^[[:space:]]*vec[[:space:]]*\{ ]]; then
        inline_data="$memory_bytes"
    else
        # Convert blob format to vec format
        local base64_content=$(echo "$memory_bytes" | sed 's/^blob "//' | sed 's/"$//')
        inline_data=$(b64_to_vec "$base64_content")
    fi
    
    # Calculate actual byte size for metadata
    local actual_bytes
    if [[ "$memory_bytes" =~ ^[[:space:]]*vec[[:space:]]*\{ ]]; then
        # For vec format, count the bytes
        actual_bytes=$(echo "$memory_bytes" | grep -o '0x[0-9a-fA-F][0-9a-fA-F]' | wc -l)
    else
        # For blob format, decode and count
        local base64_content=$(echo "$memory_bytes" | sed 's/^blob "//' | sed 's/"$//')
        actual_bytes=$(echo -n "$base64_content" | base64 -d | wc -c)
    fi
    
    # Create asset metadata with correct byte size
    local asset_metadata='(variant {
      Document = record {
        base = record {
          name = "'$memory_name'";
          description = opt "'$memory_description'";
          tags = vec { '$memory_tags' };
          asset_type = variant { Original };
          bytes = '$actual_bytes';
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
    })'
    
    local idem="test_memory_$(date +%s)_$$"
    
    local result=$(dfx canister call --identity "$identity" "$canister_id" memories_create \
        "(\"$capsule_id\", opt $inline_data, null, null, null, null, null, null, $asset_metadata, \"$idem\")" 2>/dev/null)
    
    if echo "$result" | grep -q "Ok"; then
        # Extract memory ID from the response
        local memory_id=$(echo "$result" | grep -o 'Ok = "[^"]*"' | sed 's/Ok = "//' | sed 's/"//')
        echo "$memory_id"
        return 0
    else
        echo_error "Failed to create test memory: $result"
        return 1
    fi
}

# Helper function to create test memory with specific asset type
create_test_memory_with_asset_type() {
    local capsule_id="$1"
    local memory_name="$2"
    local memory_description="$3"
    local memory_tags="$4"
    local memory_bytes="$5"
    local asset_type="$6"  # "Document", "Image", "Audio", "Video"
    local mime_type="$7"
    local canister_id="${8:-backend}"
    local identity="${9:-default}"
    
    # Convert blob format to vec format for new API
    local inline_data
    if [[ "$memory_bytes" =~ ^[[:space:]]*vec[[:space:]]*\{ ]]; then
        inline_data="$memory_bytes"
    else
        # Convert blob format to vec format
        local base64_content=$(echo "$memory_bytes" | sed 's/^blob "//' | sed 's/"$//')
        inline_data=$(b64_to_vec "$base64_content")
    fi
    
    # Calculate actual byte size for metadata
    local actual_bytes
    if [[ "$memory_bytes" =~ ^[[:space:]]*vec[[:space:]]*\{ ]]; then
        # For vec format, count the bytes
        actual_bytes=$(echo "$memory_bytes" | grep -o '0x[0-9a-fA-F][0-9a-fA-F]' | wc -l)
    else
        # For blob format, decode and count
        local base64_content=$(echo "$memory_bytes" | sed 's/^blob "//' | sed 's/"$//')
        actual_bytes=$(echo -n "$base64_content" | base64 -d | wc -c)
    fi
    
    # Create asset metadata based on asset type
    local asset_metadata
    case "$asset_type" in
        "Image")
            asset_metadata='(variant {
              Image = record {
                base = record {
                  name = "'$memory_name'";
                  description = opt "'$memory_description'";
                  tags = vec { '$memory_tags' };
                  asset_type = variant { Original };
                  bytes = '$actual_bytes';
                  mime_type = "'$mime_type'";
                  sha256 = null;
                  width = opt 1;
                  height = opt 1;
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
                color_space = null;
                exif_data = null;
                compression_ratio = null;
                dpi = null;
                orientation = null;
              }
            })'
            ;;
        "Document")
            asset_metadata='(variant {
              Document = record {
                base = record {
                  name = "'$memory_name'";
                  description = opt "'$memory_description'";
                  tags = vec { '$memory_tags' };
                  asset_type = variant { Original };
                  bytes = '$actual_bytes';
                  mime_type = "'$mime_type'";
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
                page_count = opt 1;
                document_type = opt "PDF";
                language = null;
                word_count = null;
              }
            })'
            ;;
        *)
            # Default to Document type
            asset_metadata='(variant {
              Document = record {
                base = record {
                  name = "'$memory_name'";
                  description = opt "'$memory_description'";
                  tags = vec { '$memory_tags' };
                  asset_type = variant { Original };
                  bytes = '$actual_bytes';
                  mime_type = "'$mime_type'";
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
            })'
            ;;
    esac
    
    local idem="test_memory_$(date +%s)_$$"
    
    local result=$(dfx canister call --identity "$identity" "$canister_id" memories_create \
        "(\"$capsule_id\", opt $inline_data, null, null, null, null, null, null, $asset_metadata, \"$idem\")" 2>/dev/null)
    
    if echo "$result" | grep -q "Ok"; then
        # Extract memory ID from the response
        local memory_id=$(echo "$result" | grep -o 'Ok = "[^"]*"' | sed 's/Ok = "//' | sed 's/"//')
        echo "$memory_id"
        return 0
    else
        echo_error "Failed to create test memory: $result"
        return 1
    fi
}

# Helper function to get a capsule ID for testing
get_test_capsule_id() {
    local canister_id="${1:-backend}"
    local identity="${2:-default}"
    
    local capsule_result=$(dfx canister call --identity "$identity" "$canister_id" capsules_read_basic "(null)" 2>/dev/null)
    local capsule_id=""
    
    if [[ $capsule_result == *"null"* ]] || [[ $capsule_result == *"NotFound"* ]]; then
        echo_debug "No capsule found, creating one first..."
        local create_result=$(dfx canister call --identity "$identity" "$canister_id" capsules_create "(null)" 2>/dev/null)
        if echo "$create_result" | grep -q "Ok"; then
            # Extract capsule ID from the new Result<Capsule> format
            capsule_id=$(echo "$create_result" | grep -o 'id = "[^"]*"' | sed 's/id = "//' | sed 's/"//')
        fi
    else
        if echo "$capsule_result" | grep -q "Ok"; then
            # For capsules_read_basic, extract the capsule_id field
            capsule_id=$(echo "$capsule_result" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')
        fi
    fi
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to get capsule ID for testing"
        return 1
    fi
    
    echo "$capsule_id"
    return 0
}

# Helper function to check if response indicates success
is_success() {
    local response="$1"
    echo "$response" | grep -q "variant {" && (echo "$response" | grep -q "Ok =" || echo "$response" | grep -q "Ok }")
}

# Helper function to check if response indicates failure
is_failure() {
    local response="$1"
    echo "$response" | grep -q "variant {" && echo "$response" | grep -q "Err ="
}

# Helper function to print error messages
echo_error() {
    echo "[ERROR] $1" >&2
}

# Helper function to print info messages
echo_info() {
    echo "[INFO] $1"
}

# Helper function to print debug messages
echo_debug() {
    if [[ "${DEBUG:-false}" == "true" ]]; then
        echo "[DEBUG] $1"
    fi
}

# Helper function to print success messages
echo_success() {
    echo "[SUCCESS] $1"
}

# Helper function to print header
echo_header() {
    echo "=========================================="
    echo "$1"
    echo "=========================================="
}

# Helper function to run a test and track results
run_test() {
    local test_name="$1"
    local test_function="$2"
    
    echo_info "Running: $test_name"
    
    if $test_function; then
        echo_success "✅ $test_name"
        return 0
    else
        echo_error "❌ $test_name"
        return 1
    fi
}
