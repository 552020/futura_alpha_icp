#!/bin/bash

# Test script for uploading and downloading a single file
# Usage: ./test_upload_download_file.sh <file_path>

set -e

# Fix DFX color output issues (same as working upload tests)
export NO_COLOR=1
export DFX_COLOR=0
export CLICOLOR=0
export TERM=xterm-256color
export DFX_WARNING=-mainnet_plaintext_identity

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
IDENTITY="default"
CANISTER_ID="${BACKEND_CANISTER_ID:-backend}"  # Use BACKEND_CANISTER_ID from test config
OUTPUT_DIR="tests/backend/shared-capsule/upload/assets/output"

# Source test utilities
source tests/backend/test_utils.sh

# Function to print colored output
echo_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

echo_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

echo_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

echo_debug() {
    echo -e "${YELLOW}[DEBUG]${NC} $1"
}

# Function to get test capsule ID
get_test_capsule_id() {
    local capsule_result=$(dfx canister call $CANISTER_ID capsules_read_basic "(null)" 2>/dev/null)
    local capsule_id=""
    
    if [[ $capsule_result == *"null"* ]]; then
        echo_debug "No capsule found, creating one first..."
        local create_result=$(dfx canister call $CANISTER_ID capsules_create "(null)" 2>/dev/null)
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

# Function to get file size
get_file_size() {
    local file_path="$1"
    if [[ -f "$file_path" ]]; then
        stat -f%z "$file_path" 2>/dev/null || stat -c%s "$file_path" 2>/dev/null
    else
        echo "0"
    fi
}

# Function to create memory data for inline file upload
create_inline_memory_data() {
    local file_path="$1"
    local file_name=$(basename "$file_path")
    local file_size=$(get_file_size "$file_path")
    
    # Read file and encode to base64
    local base64_data=$(base64 -i "$file_path" | tr -d '\n')
    
    cat <<EOF
blob "$base64_data"
EOF
}

create_asset_metadata() {
    local file_path="$1"
    local file_name=$(basename "$file_path")
    local file_size=$(get_file_size "$file_path")
    
    cat <<EOF
(variant {
  Document = record {
    base = record {
      name = "$file_name";
      description = opt "Upload test file - $file_size bytes";
      tags = vec { "upload-test"; "file"; "size-$file_size" };
      asset_type = variant { Original };
      bytes = $file_size;
      mime_type = "application/octet-stream";
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

# Function to upload file using blob upload process
upload_file_via_blob() {
    local file_path="$1"
    local file_name=$(basename "$file_path")
    local file_size=$(get_file_size "$file_path")
    local capsule_id="$2"
    
    echo_debug "Starting blob upload for $file_name ($file_size bytes)"
    
    # Calculate chunk size (64KB chunks - matches backend CHUNK_SIZE)
    local chunk_size=65536
    local total_chunks=$(( (file_size + chunk_size - 1) / chunk_size ))
    
    echo_debug "File will be uploaded in $total_chunks chunks of $chunk_size bytes each"
    
    # Begin upload session
    local idem="test_blob_$(date +%s)"
    local memory_meta="(variant { Document = record { base = record { name = \"$file_name\"; description = opt \"Upload test file - $file_size bytes\"; tags = vec { \"upload-test\"; \"file\"; \"size-$file_size\" }; asset_type = variant { Original }; bytes = $file_size; mime_type = \"text/plain\"; sha256 = null; width = null; height = null; url = null; storage_key = null; bucket = null; asset_location = null; processing_status = null; processing_error = null; created_at = 0; updated_at = 0; deleted_at = null; }; page_count = null; document_type = null; language = null; word_count = null; }; })"
    local begin_result=$(dfx canister call $CANISTER_ID uploads_begin "(\"$capsule_id\", $memory_meta, $total_chunks, \"$idem\")" 2>/dev/null)
    
    if [[ $begin_result != *"Ok"* ]]; then
        echo_error "❌ Failed to begin upload session: $begin_result"
        echo_debug "Begin upload command: dfx canister call --identity $IDENTITY $CANISTER_ID uploads_begin \"(\\\"$capsule_id\\\", $memory_meta, $total_chunks, \\\"$idem\\\")\""
        return 1
    fi
    
    local session_id=$(echo "$begin_result" | grep -o 'Ok = [0-9]*' | sed 's/Ok = //')
    echo_debug "Upload session started with ID: $session_id"
    
    # Upload file in chunks
    local chunk_index=0
    local offset=0
    
    while [[ $offset -lt $file_size ]]; do
        local current_chunk_size=$(( file_size - offset < chunk_size ? file_size - offset : chunk_size ))
        
        # Extract chunk data and encode to base64
        local chunk_data=$(dd if="$file_path" bs=1 skip=$offset count=$current_chunk_size 2>/dev/null | base64 | tr -d '\n')
        
        # Calculate progress percentage
        local progress=$(( (chunk_index * 100) / total_chunks ))
        local progress_bar=""
        local bar_length=20
        local filled_length=$(( (progress * bar_length) / 100 ))
        
        # Create progress bar
        for ((i=0; i<filled_length; i++)); do
            progress_bar="${progress_bar}█"
        done
        for ((i=filled_length; i<bar_length; i++)); do
            progress_bar="${progress_bar}░"
        done
        
        # Show progress with carriage return to overwrite the same line
        printf "\r[%s] %d%% - Uploading chunk %d/%d (%d bytes)" "$progress_bar" "$progress" "$((chunk_index + 1))" "$total_chunks" "$current_chunk_size"
        
        # Upload chunk using debug endpoint (like the working tests)
        local chunk_result=$(dfx canister call $CANISTER_ID debug_put_chunk_b64 "($session_id, $chunk_index, \"$chunk_data\")" 2>/dev/null)
        
        if [[ $chunk_result != *"Ok"* ]]; then
            echo ""
            echo_error "❌ Failed to upload chunk $chunk_index: $chunk_result"
            echo_debug "Chunk upload command: dfx canister call $CANISTER_ID debug_put_chunk_b64 \"($session_id, $chunk_index, \\\"$chunk_data\\\")\""
            echo_debug "Chunk data length: ${#chunk_data}"
            return 1
        fi
        
        chunk_index=$((chunk_index + 1))
        offset=$((offset + current_chunk_size))
    done
    
    # Show 100% completion
    printf "\r[████████████████████] 100%% - Upload completed successfully!\n"
    
    # Compute SHA256 hash of the entire file
    local file_hash=$(shasum -a 256 "$file_path" | cut -d' ' -f1)
    
    # Finish upload using debug endpoint (like the working tests)
    echo_debug "Finishing upload with hash: $file_hash"
    local finish_result=$(dfx canister call $CANISTER_ID debug_finish_hex "($session_id, \"$file_hash\", $file_size)" 2>/dev/null)
    
    if [[ $finish_result == *"Ok"* ]] && [[ $finish_result == *"mem_"* ]]; then
        local memory_id=$(echo "$finish_result" | grep -o '"mem_[^"]*"' | sed 's/"//g')
        echo_success "✅ Blob upload successful - Memory ID: $memory_id"
        echo "$memory_id"
        return 0
    else
        echo_error "❌ Failed to finish upload: $finish_result"
        return 1
    fi
}

# Function to download file from memory
download_file_from_memory() {
    local memory_id="$1"
    local output_path="$2"
    local test_name="$3"
    
    echo_debug "Downloading file from memory ID: $memory_id"
    
    # Get the memory data
    local result=$(dfx canister call $CANISTER_ID memories_read "(\"$memory_id\")" 2>/dev/null)
    
    if [[ $result != *"Ok = record"* ]]; then
        echo_error "❌ $test_name - Failed to retrieve memory: $result"
        return 1
    fi
    
    # Extract the base64 data from the memory
    local base64_data=""
    if [[ $result == *"Inline"* ]]; then
        # For inline storage, extract the bytes field
        base64_data=$(echo "$result" | grep -o 'bytes = blob "[^"]*"' | sed 's/bytes = blob "//' | sed 's/"//')
    elif [[ $result == *"BlobRef"* ]]; then
        # For blob storage, we need to get the blob data
        local blob_locator=$(echo "$result" | grep -o 'locator = "[^"]*"' | sed 's/locator = "//' | sed 's/"//')
        if [[ -n "$blob_locator" ]]; then
            echo_debug "Fetching blob data for locator: $blob_locator"
            local blob_result=$(dfx canister call $CANISTER_ID blob_read "(\"$blob_locator\")" 2>/dev/null)
            if [[ $blob_result == *"Ok"* ]]; then
                # The response format is: Ok = blob "base64data"
                base64_data=$(echo "$blob_result" | grep -o 'Ok = blob "[^"]*"' | sed 's/Ok = blob "//' | sed 's/"//')
            else
                echo_debug "Blob read failed: $blob_result"
            fi
        fi
    fi
    
    if [[ -z "$base64_data" ]]; then
        echo_error "❌ $test_name - No file data found in memory"
        echo_debug "Memory result: $result"
        return 1
    fi
    
    # Decode base64 and save to file
    echo "$base64_data" | base64 -d > "$output_path"
    
    if [[ -f "$output_path" ]]; then
        local file_size=$(stat -f%z "$output_path" 2>/dev/null || stat -c%s "$output_path" 2>/dev/null)
        echo_success "✅ $test_name - File downloaded successfully to: $output_path (${file_size} bytes)"
        return 0
    else
        echo_error "❌ $test_name - Failed to save downloaded file"
        return 1
    fi
}

# Function to verify downloaded file
verify_downloaded_file() {
    local original_path="$1"
    local downloaded_path="$2"
    local test_name="$3"
    
    if [[ ! -f "$downloaded_path" ]]; then
        echo_error "Downloaded file not found: $downloaded_path"
        return 1
    fi
    
    local original_size=$(get_file_size "$original_path")
    local downloaded_size=$(get_file_size "$downloaded_path")
    
    echo_debug "Original size: $original_size bytes"
    echo_debug "Downloaded size: $downloaded_size bytes"
    
    # Allow for small differences due to compression/encoding
    local size_diff=$((original_size - downloaded_size))
    local size_diff_percent=$((size_diff * 100 / original_size))
    
    if [[ $size_diff_percent -lt 1 ]]; then
        echo_success "✅ $test_name - File size verification passed (${size_diff_percent}% difference)"
        return 0
    else
        echo_error "❌ $test_name - File size verification failed (${size_diff_percent}% difference)"
        return 1
    fi
}

# Main test function
test_file_upload_download() {
    local file_path="$1"
    local file_name=$(basename "$file_path")
    local test_name="File upload/download test"
    
    echo_info "=== File Upload and Download Test ==="
    echo_info "File: $file_name"
    echo_info "Path: $file_path"
    
    # Check if file exists
    if [[ ! -f "$file_path" ]]; then
        echo_error "❌ File not found: $file_path"
        return 1
    fi
    
    local file_size=$(get_file_size "$file_path")
    echo_info "Size: $file_size bytes"
    
    # Determine upload method based on file size
    # Base64 encoding increases size by ~33%, so we need to account for that
    # For safety, use inline for files <= 20KB (leaves room for base64 encoding)
    if [[ $file_size -gt 20480 ]]; then
        echo_info "📦 Large file detected ($file_size bytes), will use blob upload process"
        UPLOAD_METHOD="blob"
    else
        echo_info "📄 Small file detected ($file_size bytes), will use inline upload"
        UPLOAD_METHOD="inline"
    fi
    
    # Get capsule ID
    local capsule_id=$(get_test_capsule_id)
    if [[ -z "$capsule_id" ]]; then
        echo_error "❌ No test capsule found"
        return 1
    fi
    echo_debug "Using capsule ID: $capsule_id"
    
    # Upload the file using appropriate method
    echo_info "Uploading file..."
    local memory_id=""
    
    if [[ "$UPLOAD_METHOD" == "inline" ]]; then
        # Use inline upload for small files
        local memory_bytes=$(create_inline_memory_data "$file_path")
        local asset_metadata=$(create_asset_metadata "$file_path")
        local idem="test_inline_$(date +%s)"
        
        echo_debug "Inline upload command: dfx canister call $CANISTER_ID memories_create \"(\\\"$capsule_id\\\", $memory_bytes, $asset_metadata, \\\"$idem\\\")\""
        local result=$(dfx canister call $CANISTER_ID memories_create "(\"$capsule_id\", $memory_bytes, $asset_metadata, \"$idem\")" 2>/dev/null)
        
        if [[ $result == *"Ok"* ]] && [[ $result == *"mem_"* ]]; then
            memory_id=$(echo "$result" | grep -o '"mem_[^"]*"' | sed 's/"//g')
            echo_success "✅ Inline upload successful - Memory ID: $memory_id"
        else
            echo_error "❌ Inline upload failed: $result"
            echo_debug "Memory data length: ${#memory_data}"
            return 1
        fi
    else
        # Use blob upload for large files
        echo_debug "Calling upload_file_via_blob with file: $file_path, capsule: $capsule_id"
        memory_id=$(upload_file_via_blob "$file_path" "$capsule_id")
        local upload_result=$?
        echo_debug "upload_file_via_blob returned: $upload_result, memory_id: $memory_id"
        if [[ $upload_result -ne 0 ]]; then
            echo_error "❌ Blob upload failed with exit code: $upload_result"
            return 1
        fi
    fi
    
    # Create output directory if it doesn't exist
    mkdir -p "$OUTPUT_DIR"
    
    # Download the file
    echo_info "Downloading file..."
    local output_path="$OUTPUT_DIR/downloaded_$file_name"
    if download_file_from_memory "$memory_id" "$output_path" "$test_name"; then
        # Verify the downloaded file
        echo_info "Verifying downloaded file..."
        if verify_downloaded_file "$file_path" "$output_path" "$test_name"; then
            echo_success "🎉 File upload and download test completed successfully!"
            echo_info "📁 Original file: $file_path"
            echo_info "📁 Downloaded file: $output_path"
            return 0
        else
            echo_error "❌ File verification failed"
            return 1
        fi
    else
        echo_error "❌ File download failed"
        return 1
    fi
}

# Main execution
main() {
    if [[ $# -eq 0 ]]; then
        echo_error "Usage: $0 <file_path>"
        echo_info "Example: $0 /path/to/your/image.jpg"
        exit 1
    fi
    
    local file_path="$1"
    
    echo "========================================="
    echo "🧪 Starting File Upload/Download Test"
    echo "========================================="
    
    if test_file_upload_download "$file_path"; then
        echo "========================================="
        echo "Test Summary for File Upload/Download"
        echo "========================================="
        echo -e "${GREEN}[PASS]${NC} 🎉 File upload/download test passed!"
        echo_info "📁 Output directory: $OUTPUT_DIR"
        exit 0
    else
        echo "========================================="
        echo "Test Summary for File Upload/Download"
        echo "========================================="
        echo -e "${RED}[FAIL]${NC} 💥 File upload/download test failed!"
        exit 1
    fi
}

# Run main function with all arguments
main "$@"
