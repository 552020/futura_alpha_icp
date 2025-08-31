#!/bin/bash

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Testing migration feature compilation...${NC}"

echo -e "${YELLOW}1. Testing build with migration features (default)...${NC}"
if cargo build --target wasm32-unknown-unknown --release -p backend; then
    echo -e "${GREEN}✅ Build with migration features successful${NC}"
else
    echo -e "${RED}❌ Build with migration features failed${NC}"
    exit 1
fi

echo -e "${YELLOW}2. Testing build without migration features...${NC}"
if cargo build --target wasm32-unknown-unknown --release -p backend --no-default-features; then
    echo -e "${GREEN}✅ Build without migration features successful${NC}"
else
    echo -e "${RED}❌ Build without migration features failed${NC}"
    exit 1
fi

echo -e "${YELLOW}3. Testing dfx build with migration features...${NC}"
if dfx build backend; then
    echo -e "${GREEN}✅ dfx build with migration features successful${NC}"
else
    echo -e "${RED}❌ dfx build with migration features failed${NC}"
    exit 1
fi

echo -e "${GREEN}🎉 All migration feature tests passed!${NC}"
echo -e "${YELLOW}Usage examples:${NC}"
echo -e "  Deploy with migration features (default): ./scripts/deploy-local.sh"
echo -e "  Deploy without migration features: MIGRATION_ENABLED=false ./scripts/deploy-local.sh"