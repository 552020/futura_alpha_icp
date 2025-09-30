#!/bin/bash

# PocketIC Test Runner - Optimized for Performance
# 
# This script runs PocketIC tests with optimal settings to avoid
# the common issues of slow/hanging tests.
#
# Usage: ./tests/pocket-ic/run_pocket_ic_tests.sh [test_name]
# Example: ./tests/pocket-ic/run_pocket_ic_tests.sh test_create_and_read_memory_happy_path

# Change to project root directory
cd "$(dirname "$0")/../.."

echo "🚀 Running PocketIC tests with optimized settings..."
echo "📁 Working directory: $(pwd)"

# Set environment variables for better performance
export RUST_LOG=warn  # Reduce log noise
export RUST_BACKTRACE=1  # Better error reporting

# Check if a specific test was requested
if [ $# -eq 1 ]; then
    TEST_NAME="$1"
    echo "🎯 Running specific test: $TEST_NAME"
    TEST_FILTER="--test $TEST_NAME"
else
    echo "🧪 Running all PocketIC tests"
    TEST_FILTER="--test memories_pocket_ic"
fi

# Run tests with optimal settings:
# - --release: Use optimized build (faster execution)
# - --test-threads=1: Avoid PocketIC server conflicts
# - --nocapture: Show output for debugging
# - --package backend: Only run backend tests

echo "📦 Building in release mode..."
cargo build --release --package backend

echo "🧪 Running PocketIC tests..."
cargo test --release \
    --package backend \
    $TEST_FILTER \
    -- \
    --test-threads=1 \
    --nocapture

echo "✅ PocketIC tests completed!"
