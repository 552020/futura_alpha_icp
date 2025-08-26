#!/bin/bash

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🚀 Deploying ICP canisters...${NC}"

# Check if dfx is running
if ! dfx ping >/dev/null 2>&1; then
    echo -e "${YELLOW}Starting dfx...${NC}"
    dfx start --background
    sleep 3
fi

if dfx deploy; then
    echo -e "${GREEN}✅ Deployed${NC}"
    
    echo -e "${YELLOW}📝 Generating declarations...${NC}"
    if dfx generate; then
        echo -e "${GREEN}✅ Declarations generated${NC}"
        
        echo -e "${YELLOW}🔧 Fixing generated declarations...${NC}"
        if node scripts/fix-declarations.cjs; then
            echo -e "${GREEN}✅ Declaration fixes applied${NC}"
        else
            echo -e "${RED}❌ Declaration fixes failed${NC}"
            exit 1
        fi
    else
        echo -e "${RED}❌ Declaration generation failed${NC}"
        exit 1
    fi
else
    echo -e "${RED}❌ Failed${NC}"
    exit 1
fi
