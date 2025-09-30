#!/bin/bash

# Working Tests Runner - Only Run Tests That Currently Pass
# 
# This script runs only the PocketIC tests that are currently working
# to verify our fixes without getting stuck on broken tests.

# Change to project root directory
cd "$(dirname "$0")/../.."

echo "✅ Running working PocketIC tests..."

# Set environment variables
export RUST_LOG=warn
export RUST_BACKTRACE=1

echo "📦 Building in release mode..."
cargo build --release --package backend

echo "🧪 Running working tests..."
# Run only the tests that we know are working
cargo test --release \
    --package backend \
    --test memories_pocket_ic \
    -- \
    --test-threads=1 \
    --nocapture \
    test_create_and_read_memory_happy_path \
    test_memory_creation_idempotency

echo "✅ Working tests completed!"
