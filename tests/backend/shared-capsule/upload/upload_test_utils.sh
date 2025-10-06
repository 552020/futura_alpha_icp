#!/bin/bash

# Upload Test Utilities
# Shared functions for upload-related test scripts

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

# Helper function to print error messages
echo_error() {
    echo "[ERROR] $1" >&2
}

# Helper function to print info messages
echo_info() {
    echo "[INFO] $1"
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
        if is_success "$create_result"; then
            # Extract capsule ID from the new Result<Capsule> format
            capsule_id=$(echo "$create_result" | grep -o 'id = "[^"]*"' | sed 's/id = "//' | sed 's/"//')
        fi
    else
        if is_success "$capsule_result"; then
            # For capsules_read_basic, extract the capsule_id field
            capsule_id=$(echo "$capsule_result" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')
        fi
    fi
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to get capsule ID for testing"
        return 1
    fi
    
    echo "$capsule_id"
}

# Helper function to create test chunk data
create_test_chunk() {
    local chunk_index="$1"
    local chunk_size="$2"
    
    # Create chunk data with pattern based on index
    local pattern=$(printf "%02d" $chunk_index)
    local chunk_data=""
    for ((i=0; i<chunk_size; i++)); do
        chunk_data="${chunk_data}${pattern}"
    done
    
    # Convert to base64 for binary data (macOS compatible)
    echo -n "$chunk_data" | base64
}

# Helper function to create a 32-byte hash for testing
create_test_hash() {
    local data="$1"
    # Create exactly 32 bytes of data
    local hash_data=""
    while [[ ${#hash_data} -lt 32 ]]; do
        hash_data="${hash_data}${data}"
    done
    # Take exactly 32 bytes and convert to Candid blob format
    local hash_bytes="${hash_data:0:32}"
    local blob_arg=""
    for ((i=0; i<32; i++)); do
        local byte=$(printf "%d" "'${hash_bytes:$i:1}")
        blob_arg="${blob_arg}\\$(printf "%02x" $byte)"
    done
    echo "$blob_arg"
}

# Helper function to compute SHA256 hash of test data
compute_test_hash() {
    local chunk_data="$1"
    local chunk_count="$2"
    
    # Compute hash of all chunks concatenated together
    # This matches what the backend computes in store_from_chunks
    for ((i=0; i<chunk_count; i++)); do
        local chunk=$(create_test_chunk $i 50)
        echo -n "$chunk" | base64 -d
    done | shasum -a 256 | cut -d' ' -f1
}

# Portable sha256 to hex (Linux/macOS)
sha256_hex() {
    if command -v sha256sum >/dev/null 2>&1; then
        # strip trailing "  -"
        awk '{print $1}' <(printf %s "$1" | sha256sum)
    else
        printf %s "$1" | shasum -a 256 | awk '{print $1}'
    fi
}

# hex → Candid vec { 0x..; }
hex_to_candid_vec() {
    local hex="$1"
    local out="vec {"
    local i
    for ((i=0;i<${#hex};i+=2)); do
        out+=" 0x${hex:$i:2};"
    done
    out+=" }"
    printf %s "$out"
}

# base64 without newlines/wrapping (portable)
b64_nolf() {
    printf %s "$1" | base64 | tr -d '\n'
}

# byte length of raw data
byte_len() {
    # POSIX wc -c counts bytes from stdin
    printf %s "$1" | wc -c | awk '{print $1}'
}

# Helper function to begin an upload session
begin_upload_session() {
    local capsule_id="$1"
    local chunk_count="$2"
    local idem="$3"
    
    # Create proper AssetMetadata variant for Document type
    local asset_metadata='(variant {
      Document = record {
        base = record {
          name = "test-upload";
          description = opt "Test upload";
          tags = vec {};
          asset_type = variant { Original };
          bytes = 0;
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
    
    local result=$(dfx canister call "$BACKEND_CANISTER_ID" uploads_begin \
        "(\"$capsule_id\", $asset_metadata, $chunk_count, \"$idem\")" 2>/dev/null)
    
    if is_success "$result"; then
        # Extract session ID from response (format: Ok = 1 : nat64)
        local session_id=$(echo "$result" | grep -o 'Ok = [0-9]*' | sed 's/Ok = //')
        echo "$session_id"
        return 0
    else
        echo_error "Failed to begin upload session: $result"
        return 1
    fi
}

# Helper function to upload a chunk
upload_chunk() {
    local session_id="$1"
    local chunk_index="$2"
    local chunk_data="$3"
    
    local result=$(dfx canister call "$BACKEND_CANISTER_ID" uploads_put_chunk \
        "($session_id, $chunk_index:nat32, blob \"$chunk_data\")" 2>/dev/null)
    
    if is_success "$result"; then
        echo_info "Chunk $chunk_index uploaded successfully"
        return 0
    else
        echo_error "Failed to upload chunk $chunk_index: $result"
        return 1
    fi
}

# Helper function to finish an upload
finish_upload() {
    local session_id="$1"
    local expected_hash="$2"  # Hex string hash
    local total_len="$3"
    
    # Convert hex hash to Candid vec nat8 format
    local hash_vec=$(hex_to_candid_vec "$expected_hash")
    
    local result=$(dfx canister call "$BACKEND_CANISTER_ID" uploads_finish \
        "($session_id, $hash_vec, $total_len:nat64)" 2>/dev/null)
    
    if is_success "$result"; then
        echo_info "Upload finished successfully: $result"
        return 0
    else
        echo_error "Failed to finish upload: $result"
        return 1
    fi
}

# Helper function to abort an upload
abort_upload() {
    local session_id="$1"
    
    local result=$(dfx canister call "$BACKEND_CANISTER_ID" uploads_abort "$session_id" 2>/dev/null)
    
    if is_success "$result"; then
        echo_info "Upload aborted successfully: $result"
        return 0
    else
        echo_error "Failed to abort upload: $result"
        return 1
    fi
}

# Helper function to run a complete upload workflow
run_complete_upload_workflow() {
    local capsule_id="$1"
    local chunk_count="$2"
    local chunk_size="$3"
    local idem="$4"
    
    echo_info "Starting complete upload workflow with $chunk_count chunks of $chunk_size bytes each"
    
    # Begin upload
    local session_id=$(begin_upload_session "$capsule_id" "$chunk_count" "$idem")
    if [[ -z "$session_id" ]]; then
        return 1
    fi
    
    # Upload all chunks
    for ((i=0; i<chunk_count; i++)); do
        local chunk_data=$(create_test_chunk $i $chunk_size)
        if ! upload_chunk "$session_id" $i "$chunk_data"; then
            return 1
        fi
    done
    
    # Finish upload
    local expected_hash=$(create_test_hash "test")
    local total_len=$((chunk_count * chunk_size))
    if ! finish_upload "$session_id" "$expected_hash" "$total_len"; then
        return 1
    fi
    
    echo_success "Complete upload workflow completed successfully"
    return 0
}

# Helper function to test upload validation
test_upload_validation() {
    local test_name="$1"
    local test_command="$2"
    local expected_error="$3"
    
    echo_info "Testing: $test_name"
    local result=$(eval "$test_command" 2>/dev/null)
    
    if is_failure "$result" && [[ -n "$expected_error" ]] && echo "$result" | grep -q "$expected_error"; then
        echo_success "$test_name - correctly rejected with expected error"
        return 0
    elif is_failure "$result"; then
        echo_success "$test_name - correctly rejected: $result"
        return 0
    else
        echo_error "$test_name - should have been rejected: $result"
        return 1
    fi
}

# Helper function to convert hex string to Candid vec nat8 format
hex_to_candid_vec() {
    local hex="$1"
    local out="vec {"
    local i
    for ((i=0;i<${#hex};i+=2)); do
        out+=" 0x${hex:$i:2};"
    done
    echo "$out }"
}

# Helper function to create a blob using the upload workflow
# This creates a blob that can then be referenced in BlobRef tests
create_test_blob() {
    local capsule_id="$1"
    local blob_data="$2"  # Raw data (will be base64 encoded)
    local blob_name="$3"  # Name for the blob
    local canister_id="${4:-backend}"
    local identity="${5:-default}"
    
    echo_info "Creating test blob: $blob_name"
    
    # 1) Convert raw data to Candid vec { 0x..; } format (raw bytes, not base64)
    local chunk_vec=""
    local i
    for ((i=0; i<${#blob_data}; i++)); do
        local char="${blob_data:$i:1}"
        local byte=$(printf "%d" "'$char")
        chunk_vec="${chunk_vec} 0x$(printf "%02x" $byte);"
    done
    chunk_vec="vec {${chunk_vec} }"
    
    # 2) Compute the SHA256 over the SAME raw bytes the backend will hash
    local hash_hex
    hash_hex=$(sha256_hex "$blob_data")
    local hash_vec
    hash_vec=$(hex_to_candid_vec "$hash_hex")
    
    # 3) Total length in BYTES (matches what backend expects)
    local total_len
    total_len=$(byte_len "$blob_data")
    
    # Create Candid blob format hash for metadata (convert hex to Candid blob format)
    local blob_data_hash=""
    for ((i=0; i<64; i+=2)); do
        local hex_byte="${hash_hex:$i:2}"
        local decimal_byte=$((16#$hex_byte))
        blob_data_hash="${blob_data_hash}\\$(printf "%02x" $decimal_byte)"
    done
    
    # Create asset metadata for the blob (using proper AssetMetadata enum structure)
    local blob_asset_metadata='(variant {
      Document = record {
        base = record {
          name = "'$blob_name'";
          description = opt "Test blob created for testing";
          tags = vec { "test"; "blob"; "utility" };
          asset_type = variant { Original };
          bytes = '$total_len';
          mime_type = "text/plain";
          sha256 = opt blob "'$blob_data_hash'";
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
    
    local blob_idem="test_blob_$(date +%s)_$$"
    
    # Begin upload session (1 chunk for small data)
    local session_result=$(dfx canister call --identity "$identity" "$canister_id" uploads_begin "(\"$capsule_id\", 1, \"$blob_idem\")" 2>/dev/null)
    
    if ! is_success "$session_result"; then
        echo_error "Failed to begin upload session for blob creation: $session_result"
        return 1
    fi
    
    # Extract session ID
    local session_id=$(echo "$session_result" | grep -o 'Ok = [0-9]*' | sed 's/Ok = //')
    echo_info "Created upload session: $session_id"
    
    # 4) Call regular endpoints with precise types
    local chunk_result=$(dfx canister call --identity "$identity" "$canister_id" uploads_put_chunk "($session_id, 0:nat32, $chunk_vec)" 2>/dev/null)
    
    if ! is_success "$chunk_result"; then
        echo_error "Failed to upload chunk for blob creation: $chunk_result"
        return 1
    fi
    
    local finish_result=$(dfx canister call --identity "$identity" "$canister_id" uploads_finish "($session_id, $hash_vec, $total_len:nat64)" 2>/dev/null)
    
    if ! is_success "$finish_result"; then
        echo_error "Failed to finish upload for blob creation: $finish_result"
        return 1
    fi
    
    echo_success "Successfully created blob: $blob_name (session: $session_id, size: $total_len bytes)"
    
    # Return blob information for use in tests
    echo "BLOB_ID:$session_id"
    echo "BLOB_HASH:$blob_data_hash"
    echo "BLOB_SIZE:$total_len"
    echo "BLOB_LOCATOR:blob:$session_id"
    
    return 0
}

# Helper function to create a minimal test blob (1 byte)
create_minimal_test_blob() {
    local capsule_id="$1"
    local canister_id="${2:-backend}"
    local identity="${3:-default}"
    
    create_test_blob "$capsule_id" "A" "minimal_test_blob.txt" "$canister_id" "$identity"
}

# Helper function to create a test blob with custom data
create_custom_test_blob() {
    local capsule_id="$1"
    local blob_data="$2"
    local blob_name="$3"
    local canister_id="${4:-backend}"
    local identity="${5:-default}"
    
    create_test_blob "$capsule_id" "$blob_data" "$blob_name" "$canister_id" "$identity"
}

# Helper function to extract blob information from create_test_blob output
extract_blob_info() {
    local blob_output="$1"
    local info_type="$2"  # "ID", "HASH", "SIZE", "LOCATOR"
    
    echo "$blob_output" | grep "BLOB_${info_type}:" | cut -d':' -f2-
}

# Helper function to print test summary
print_test_summary() {
    local test_name="$1"
    local total_tests="$2"
    local passed_tests="$3"
    local failed_tests="$4"
    
    echo "========================================="
    echo "Test Summary for $test_name"
    echo "========================================="
    echo "Total tests: $total_tests"
    echo "Passed: $passed_tests"
    echo "Failed: $failed_tests"
    echo ""
    
    if [ $failed_tests -eq 0 ]; then
        echo_success "All $test_name tests passed!"
        return 0
    else
        echo_error "$failed_tests $test_name test(s) failed"
        return 1
    fi
}
