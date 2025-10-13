#!/bin/bash

# Backend Unit Test Runner - Fast Tests Only
# 
# This script runs only backend unit tests (no PocketIC), which are much faster.
# Use this for quick development feedback during coding.

# Change to project root directory
cd "$(dirname "$0")/../.."

echo "⚡ Running backend unit tests (fast mode)..."

# Set environment variables
export RUST_LOG=warn
export RUST_BACKTRACE=1

echo "📦 Building backend..."
cargo build --package backend

echo "🧪 Running backend unit tests (excluding IC-dependent tests)..."
cargo test --package backend --lib -- \
    --skip capsule::time::tests \
    --skip session::compat::tests \
    --skip memories::core::update::tests \
    --skip upload::blob_store::tests

echo "✅ Backend unit tests completed!"

