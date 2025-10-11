#!/bin/bash

# Test Environment Setup Script
# 
# This script sets up the test environment by:
# 1. Copying backend declarations to test directory
# 2. Creating proper package.json for ES modules
# 3. Ensuring test dependencies are available
#
# Usage: ./setup-test-environment.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
echo_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

echo_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

echo_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

echo_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../" && pwd)"
UPLOAD_TEST_DIR="$SCRIPT_DIR/shared-capsule/upload"

echo_info "Setting up test environment..."
echo_info "Script directory: $SCRIPT_DIR"
echo_info "Project root: $PROJECT_ROOT"
echo_info "Upload test directory: $UPLOAD_TEST_DIR"

# Check if we're in the right directory
if [ ! -f "$UPLOAD_TEST_DIR/test_upload_begin.mjs" ]; then
    echo_error "Upload test directory not found at: $UPLOAD_TEST_DIR"
    exit 1
fi

# Check if NextJS declarations exist
NEXTJS_DECLARATIONS="$PROJECT_ROOT/src/nextjs/src/ic/declarations/backend"
if [ ! -d "$NEXTJS_DECLARATIONS" ]; then
    echo_error "NextJS declarations not found at: $NEXTJS_DECLARATIONS"
    echo_error "Please ensure the backend canister is deployed and declarations are generated"
    exit 1
fi

echo_info "Found NextJS declarations at: $NEXTJS_DECLARATIONS"

# Create shared test declarations directory at backend level
SHARED_DECLARATIONS="$SCRIPT_DIR/declarations/backend"
echo_info "Creating shared test declarations directory: $SHARED_DECLARATIONS"
mkdir -p "$SHARED_DECLARATIONS"

# Copy declarations from NextJS to shared test directory
echo_info "Copying backend declarations to shared location..."
cp -r "$NEXTJS_DECLARATIONS"/* "$SHARED_DECLARATIONS/"

# Verify the copy was successful
if [ ! -f "$SHARED_DECLARATIONS/backend.did.js" ]; then
    echo_error "Failed to copy backend.did.js"
    exit 1
fi

echo_success "Backend declarations copied successfully"

# Create package.json for ES modules at backend level
echo_info "Creating package.json for ES modules..."
cat > "$SCRIPT_DIR/package.json" << EOF
{
  "type": "module",
  "name": "upload-tests",
  "version": "1.0.0",
  "description": "ICP Upload Tests Environment",
  "engines": {
    "node": ">=16.0.0"
  },
  "dependencies": {
    "@dfinity/agent": "^3.2.6",
    "@dfinity/identity": "^3.2.6",
    "node-fetch": "^3.3.2"
  }
}
EOF

echo_success "Package.json created with ES module configuration"

# Update test imports to use shared declarations
echo_info "Updating test imports to use shared declarations..."

# Find all test files that import from the old path
TEST_FILES=$(find "$SCRIPT_DIR" -name "*.mjs" -type f)

for test_file in $TEST_FILES; do
    if grep -q "../../../../src/nextjs/src/ic/declarations/backend/backend.did.js" "$test_file"; then
        echo_info "Updating imports in: $(basename "$test_file")"
        
        # Create backup
        cp "$test_file" "$test_file.backup"
        
        # Replace the import path to use shared declarations
        # Calculate relative path from test file to shared declarations
        TEST_DIR="$(dirname "$test_file")"
        RELATIVE_PATH=$(python3 -c "import os; print(os.path.relpath('$SHARED_DECLARATIONS', '$TEST_DIR'))")
        sed -i '' "s|../../../../src/nextjs/src/ic/declarations/backend/backend.did.js|$RELATIVE_PATH/backend.did.js|g" "$test_file"
        
        echo_success "Updated imports in $(basename "$test_file")"
    fi
done

# Verify the setup
echo_info "Verifying test environment setup..."

# Check if package.json exists
if [ -f "$SCRIPT_DIR/package.json" ]; then
    echo_success "✅ Package.json exists"
else
    echo_error "❌ Package.json missing"
    exit 1
fi

# Check if declarations exist
if [ -f "$SHARED_DECLARATIONS/backend.did.js" ]; then
    echo_success "✅ Backend declarations exist"
else
    echo_error "❌ Backend declarations missing"
    exit 1
fi

# Check if at least one test file was updated
if find "$SCRIPT_DIR" -name "*.mjs" -exec grep -l "declarations/backend/backend.did.js" {} \; | grep -q .; then
    echo_success "✅ Test imports updated"
else
    echo_warning "⚠️  No test imports found to update"
fi

echo_success "🎉 Test environment setup completed successfully!"
echo_info ""
echo_info "Shared declarations are now available at: $SHARED_DECLARATIONS"
echo_info "All tests can now use the shared declarations with relative imports"
echo_info ""
echo_info "You can now run tests from any test directory with:"
echo_info "  node test_upload_begin.mjs <CANISTER_ID>"
echo_info ""
echo_info "To restore original imports, run:"
echo_info "  find . -name '*.backup' -exec sh -c 'mv \"\$1\" \"\${1%.backup}\"' _ {} \;"
