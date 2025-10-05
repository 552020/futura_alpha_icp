#!/bin/bash

# Asset ID Endpoints Test
# Tests the new asset_id-based API endpoints for asset management

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Test configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
TEST_NAME="Asset ID Endpoints Test"

echo -e "${CYAN}🚀 $TEST_NAME${NC}"
echo -e "${BLUE}📍 Project Root: $PROJECT_ROOT${NC}"

# Check if dfx is running
if ! curl -s http://127.0.0.1:4943 > /dev/null 2>&1; then
    echo -e "${RED}❌ Local ICP replica is not running. Please start it with: dfx start${NC}"
    exit 1
fi

# Check if backend canister is deployed
if ! dfx canister call backend health_check 2>/dev/null > /dev/null; then
    echo -e "${RED}❌ Backend canister is not deployed. Please deploy it with: dfx deploy backend${NC}"
    exit 1
fi

# Set environment variables
export BACKEND_CANISTER_ID=$(dfx canister id backend)
export IC_HOST="http://127.0.0.1:4943"

echo -e "${BLUE}🔧 Backend Canister ID: $BACKEND_CANISTER_ID${NC}"

# Run the test
echo -e "${CYAN}🧪 Running Asset ID Endpoints Test...${NC}"

cd "$PROJECT_ROOT"

# Run the JavaScript test
node tests/backend/utils/test_asset_id_endpoints.mjs

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Asset ID Endpoints Test completed successfully!${NC}"
    echo -e "${GREEN}🎉 All asset_id-based API endpoints are working correctly${NC}"
else
    echo -e "${RED}❌ Asset ID Endpoints Test failed${NC}"
    exit 1
fi
