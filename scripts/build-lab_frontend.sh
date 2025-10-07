#!/bin/bash

# Build script for lab_frontend
# This script ensures proper path resolution for declarations

set -e

echo "🔨 Building lab_frontend..."

# Ensure we're in the project root
cd "$(dirname "$0")/.."

# Generate lab_backend declarations from root directory
echo "📝 Generating lab_backend declarations..."
dfx generate lab_backend

# Build lab_frontend
echo "🏗️ Building lab_frontend..."
cd src/lab_frontend
pnpm install
pnpm run build

echo "✅ lab_frontend build complete!"
echo "📁 Built files are in: src/lab_frontend/dist/"
