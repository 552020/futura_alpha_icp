#!/bin/bash

# Pre-commit DID Check
# Quick check to ensure .did file is up to date before committing

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔍 Quick DID consistency check...${NC}"

# Check if dfx is running
if ! dfx ping >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  dfx not running. Skipping DID check.${NC}"
    echo -e "${YELLOW}   Run './scripts/deploy-local.sh' to update .did file${NC}"
    exit 0
fi

# Check if backend is deployed
if ! dfx canister id backend >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  Backend not deployed. Skipping DID check.${NC}"
    echo -e "${YELLOW}   Run './scripts/deploy-local.sh' to deploy and update .did file${NC}"
    exit 0
fi

# Quick check: generate fresh .did and compare
echo -e "${YELLOW}📝 Generating fresh .did file...${NC}"

if generate-did backend >/dev/null 2>&1; then
    TEMP_DID=".dfx/local/canisters/backend/backend.did"
    ORIGINAL_DID="src/backend/backend.did"
    
    if [ -f "$TEMP_DID" ] && [ -f "$ORIGINAL_DID" ]; then
        if diff -q "$ORIGINAL_DID" "$TEMP_DID" >/dev/null 2>&1; then
            echo -e "${GREEN}✅ DID file is up to date${NC}"
            exit 0
        else
            echo -e "${RED}❌ DID file is outdated!${NC}"
            echo -e "${YELLOW}   Run './scripts/deploy-local.sh' to update it${NC}"
            exit 1
        fi
    else
        echo -e "${YELLOW}⚠️  Could not compare .did files${NC}"
        exit 0
    fi
else
    echo -e "${YELLOW}⚠️  Could not generate .did file${NC}"
    exit 0
fi
