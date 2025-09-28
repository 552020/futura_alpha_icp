#!/usr/bin/env bash
set -euo pipefail

# Test script for Node.js uploader
# This bypasses DFX CLI limitations by using the @dfinity/agent directly
# Usage: ./test-node-upload.sh [--mainnet] <file_path>

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../../../" && pwd)"

# Parse command line arguments
MAINNET_MODE=false
FILE_PATH=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --mainnet)
            MAINNET_MODE=true
            shift
            ;;
        *)
            if [[ -z "$FILE_PATH" ]]; then
                FILE_PATH="$1"
            else
                echo_error "Unknown argument: $1"
                exit 1
            fi
            shift
            ;;
    esac
done

# Preserve the original SCRIPT_DIR before sourcing configs
ORIGINAL_SCRIPT_DIR="$SCRIPT_DIR"

# Load test configuration and utilities
if [[ "$MAINNET_MODE" == "true" ]]; then
    source "$PROJECT_ROOT/tests/backend/mainnet/config.sh"
    TEST_NAME="Node.js Upload Test (MAINNET)"
    IC_HOST="${IC_HOST:-https://ic0.app}"
    CANISTER_ID_VAR="MAINNET_CANISTER_ID"
    # Restore original SCRIPT_DIR after sourcing mainnet config
    SCRIPT_DIR="$ORIGINAL_SCRIPT_DIR"
else
    source "$PROJECT_ROOT/tests/backend/test_config.sh"
    TEST_NAME="Node.js Upload Test"
    IC_HOST="${IC_HOST:-http://127.0.0.1:4943}"
    CANISTER_ID_VAR="BACKEND_CANISTER_ID"
fi

source "$PROJECT_ROOT/tests/backend/test_utils.sh"
if [[ -f "$SCRIPT_DIR/upload_test_utils.sh" ]]; then
    source "$SCRIPT_DIR/upload_test_utils.sh"
fi

# Test configuration
CHUNK_SIZE="${CHUNK_SIZE:-65536}"  # 64KB chunks

# Use the appropriate canister ID
if [[ "$MAINNET_MODE" == "true" ]]; then
    if [[ -z "$MAINNET_CANISTER_ID" ]]; then
        echo_error "MAINNET_CANISTER_ID not set. Make sure mainnet/config.sh is loaded properly."
        exit 1
    fi
    CANISTER_ID="$MAINNET_CANISTER_ID"
else
    if [[ -z "$BACKEND_CANISTER_ID" ]]; then
        echo_error "BACKEND_CANISTER_ID not set. Make sure test_config.sh is loaded properly."
        exit 1
    fi
    CANISTER_ID="$BACKEND_CANISTER_ID"
fi

# Check if file argument provided
if [[ -z "$FILE_PATH" ]]; then
    echo_error "Usage: $0 [--mainnet] <file_path>"
    echo_info "Example: $0 tests/backend/shared-capsule/memories/assets/input/avocado_large.jpg"
    echo_info "Example: $0 --mainnet tests/backend/shared-capsule/memories/assets/input/avocado_large.jpg"
    if [[ "$MAINNET_MODE" == "true" ]]; then
        echo_warn "⚠️  WARNING: --mainnet flag will upload to MAINNET and cost cycles!"
    fi
    exit 1
fi

# Check if file exists
if [[ ! -f "$FILE_PATH" ]]; then
    echo_error "File not found: $FILE_PATH"
    exit 1
fi

# Check if Node.js is available
if ! command -v node &> /dev/null; then
    echo_error "Node.js not found. Please install Node.js to use this uploader."
    exit 1
fi

# Check if required npm packages are installed
if [[ ! -d "$PROJECT_ROOT/node_modules/@dfinity/agent" ]]; then
    echo_info "Installing required npm packages..."
    cd "$PROJECT_ROOT"
    npm install @dfinity/agent node-fetch
fi

# Get file info
FILE_SIZE=$(stat -f%z "$FILE_PATH" 2>/dev/null || stat -c%s "$FILE_PATH" 2>/dev/null)
FILE_NAME=$(basename "$FILE_PATH")

echo "========================================="
echo "Starting $TEST_NAME"
echo "========================================="
echo_info "File: $FILE_PATH"
echo_info "Size: $FILE_SIZE bytes"
echo_info "Chunk size: $CHUNK_SIZE bytes"
echo_info "IC Host: $IC_HOST"
echo_info "Backend ID: $CANISTER_ID"
if [[ "$MAINNET_MODE" == "true" ]]; then
    echo_warn "⚠️  WARNING: This will upload to MAINNET and cost cycles!"
    echo ""
    # Confirm before proceeding
    read -p "Do you want to continue with mainnet upload? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo_info "Upload cancelled by user"
        exit 0
    fi
fi
echo ""

# Run the Node.js uploader
echo_info "Starting upload with Node.js uploader..."
cd "$PROJECT_ROOT"

export IC_HOST="$IC_HOST"
export BACKEND_CANISTER_ID="$CANISTER_ID"
export CHUNK_SIZE="$CHUNK_SIZE"

TEST_START_TIME=$(date +%s)
echo_debug "Running: node $SCRIPT_DIR/ic-upload.mjs $FILE_PATH"
if node "$SCRIPT_DIR/ic-upload.mjs" "$FILE_PATH"; then
    TEST_END_TIME=$(date +%s)
    TEST_DURATION=$((TEST_END_TIME - TEST_START_TIME))
    echo_success "✅ Upload completed successfully!"
    echo_info "📁 File: $FILE_PATH"
    echo_info "📊 Size: $FILE_SIZE bytes"
    echo_info "⏱️  Test duration: ${TEST_DURATION}s"
    echo_info "🔧 Method: Node.js uploader (bypasses DFX CLI limits)"
    if [[ "$MAINNET_MODE" == "true" ]]; then
        echo_info "🌐 Network: MAINNET"
    else
        echo_info "🌐 Network: LOCAL"
    fi
    echo ""
    echo "========================================="
    echo "Test Summary for $TEST_NAME"
    echo "========================================="
    echo_success "🎉 Node.js upload test passed!"
    exit 0
else
    echo_error "❌ Upload failed!"
    echo ""
    echo "========================================="
    echo "Test Summary for $TEST_NAME"
    echo "========================================="
    echo_fail "💥 Node.js upload test failed!"
    exit 1
fi
