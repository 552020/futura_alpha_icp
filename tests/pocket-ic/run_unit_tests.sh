#!/bin/bash

# Unit Test Runner - Fast Tests Only
# 
# This script runs only unit tests, which are much faster than PocketIC tests.
# Use this for quick development feedback.

# Change to project root directory
cd "$(dirname "$0")/../.."

echo "⚡ Running unit tests (fast mode)..."

# Set environment variables
export RUST_LOG=warn
export RUST_BACKTRACE=1

echo "📦 Building..."
cargo build --package backend

echo "🧪 Running unit tests..."
cargo test --package backend --lib

echo "✅ Unit tests completed!"

