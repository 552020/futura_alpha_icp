#!/bin/bash

# All PocketIC Tests Runner
# 
# This script runs all PocketIC integration tests in the correct order.
# These tests are slower but provide comprehensive integration testing.

# Change to project root directory
cd "$(dirname "$0")/../.."

echo "🚀 Running all PocketIC integration tests..."

# Set environment variables for better performance
export RUST_LOG=warn
export RUST_BACKTRACE=1

echo "📦 Building in release mode..."
cargo build --release --package http-integration-tests

echo "🧪 Running PocketIC tests..."

# Run all PocketIC test binaries
echo "1️⃣ HTTP Integration Tests..."
cargo test --package http-integration-tests --bin http_integration_tests

echo "2️⃣ Hello World Tests..."
cargo test --package http-integration-tests --bin hello_world_pocket_ic

echo "3️⃣ Simple PocketIC Tests..."
cargo test --package http-integration-tests --bin simple_pocket_ic

echo "4️⃣ Simple Memory Tests..."
cargo test --package http-integration-tests --bin simple_memory_test

echo "5️⃣ Memory Management Tests..."
cargo test --package http-integration-tests --bin memories_pocket_ic

echo "✅ All PocketIC tests completed!"
