#!/bin/bash

# DID Drift Detection Script
# This script checks if the committed .did file matches what the deployed canister actually implements

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔍 Checking for DID drift...${NC}"

# Check if dfx is running
if ! dfx ping >/dev/null 2>&1; then
    echo -e "${YELLOW}Starting dfx...${NC}"
    dfx start --background
    sleep 3
fi

# Check if required tools are installed
if ! command -v "generate-did" >/dev/null 2>&1; then
    echo -e "${RED}❌ generate-did tool not found${NC}"
    echo -e "${YELLOW}Please install it using: cargo install generate-did${NC}"
    exit 1
fi

# Store the original .did file
ORIGINAL_DID="src/backend/backend.did"
TEMP_DID="src/backend/backend.did.temp"

# Backup the original .did file
if [ -f "$ORIGINAL_DID" ]; then
    cp "$ORIGINAL_DID" "$TEMP_DID"
    echo -e "${YELLOW}📝 Backed up original .did file${NC}"
else
    echo -e "${RED}❌ Original .did file not found at $ORIGINAL_DID${NC}"
    exit 1
fi

echo -e "${YELLOW}📝 Deploying backend to generate fresh .did file...${NC}"

# Deploy the backend (this will generate a fresh .did file)
if ! dfx deploy backend --mode reinstall --yes; then
    echo -e "${RED}❌ Failed to deploy backend${NC}"
    # Restore original file
    mv "$TEMP_DID" "$ORIGINAL_DID"
    exit 1
fi

echo -e "${YELLOW}📝 Generating fresh .did file from deployed canister...${NC}"

# Generate the .did file from the deployed canister
if ! generate-did backend; then
    echo -e "${RED}❌ Failed to generate .did file${NC}"
    # Restore original file
    mv "$TEMP_DID" "$ORIGINAL_DID"
    exit 1
fi

# Check if the generated .did file exists
if [ ! -f "$ORIGINAL_DID" ]; then
    echo -e "${RED}❌ Generated .did file not found at $ORIGINAL_DID${NC}"
    # Restore original file
    mv "$TEMP_DID" "$ORIGINAL_DID"
    exit 1
fi

echo -e "${YELLOW}🔍 Comparing committed .did file with deployed canister interface...${NC}"

# Compare the files (original vs fresh generated)
if diff -q "$TEMP_DID" "$ORIGINAL_DID" >/dev/null 2>&1; then
    echo -e "${GREEN}✅ No DID drift detected!${NC}"
    echo -e "${GREEN}   The committed .did file matches the deployed canister interface${NC}"
    # Clean up temp file
    rm -f "$TEMP_DID"
    exit 0
else
    echo -e "${RED}❌ DID drift detected!${NC}"
    echo -e "${RED}   The committed .did file doesn't match the deployed canister interface${NC}"
    echo ""
    echo -e "${YELLOW}📋 Differences found:${NC}"
    echo -e "${BLUE}--- Committed .did file (original)${NC}"
    echo -e "${BLUE}+++ Deployed canister interface (fresh)${NC}"
    diff "$TEMP_DID" "$ORIGINAL_DID" || true
    echo ""
    echo -e "${YELLOW}💡 To fix this:${NC}"
    echo -e "${CYAN}   1. Run: ./scripts/deploy-local.sh${NC}"
    echo -e "${CYAN}   2. Commit the updated .did file${NC}"
    echo -e "${CYAN}   3. Or manually update the .did file to match the deployed interface${NC}"
    # Restore original file
    mv "$TEMP_DID" "$ORIGINAL_DID"
    exit 1
fi
