#!/bin/bash

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🚀 Deploying backend and Internet Identity canisters locally...${NC}"

# Check if dfx is running
if ! dfx ping >/dev/null 2>&1; then
    echo -e "${YELLOW}Starting dfx...${NC}"
    dfx start --background
    sleep 3
fi

# Check if required tools are installed
MISSING_TOOLS=()

# Check if required tools are installed (check both PATH and ~/.cargo/bin)
MISSING_TOOLS=()

# Helper function to check if tool exists in PATH or ~/.cargo/bin
check_tool() {
    local tool_name=$1
    if command -v "$tool_name" >/dev/null 2>&1 || command -v "$HOME/.cargo/bin/$tool_name" >/dev/null 2>&1; then
        return 0  # Tool found
    else
        return 1  # Tool not found
    fi
}

if ! check_tool "generate-did"; then
    MISSING_TOOLS+=("generate-did")
fi



if ! check_tool "candid-extractor"; then
    MISSING_TOOLS+=("candid-extractor")
fi

if [ ${#MISSING_TOOLS[@]} -gt 0 ]; then
    echo -e "${RED}❌ Missing required tools: ${MISSING_TOOLS[*]}${NC}"
    echo -e "${YELLOW}Please install them using:${NC}"
    echo -e "${CYAN}   cargo install generate-did${NC}"
    echo -e "${CYAN}   cargo install candid-extractor --locked${NC}"
    echo -e "${YELLOW}Then run this script again.${NC}"
    exit 1
fi

# Check if MIGRATION_ENABLED environment variable is set to false
if [ "${MIGRATION_ENABLED:-true}" = "false" ]; then
    echo -e "${YELLOW}Deploying backend without migration features...${NC}"
    DEPLOY_CMD="dfx deploy backend --argument '()' --mode=reinstall"
else
    echo -e "${YELLOW}Deploying backend with migration features (default)...${NC}"
    DEPLOY_CMD="dfx deploy backend"
fi

if eval "$DEPLOY_CMD" && dfx deploy internet_identity; then
    echo -e "${GREEN}✅ Deployed${NC}"
    
    echo -e "${YELLOW}📝 Generating .did file...${NC}"
    if generate-did backend; then
        echo -e "${GREEN}✅ .did file generated${NC}"
    else
        echo -e "${RED}❌ .did file generation failed${NC}"
        exit 1
    fi
    
    echo -e "${YELLOW}📝 Generating declarations (backend, internet_identity only)...${NC}"
    if dfx generate backend && dfx generate internet_identity; then
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
