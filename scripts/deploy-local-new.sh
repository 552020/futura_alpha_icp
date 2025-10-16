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
    
    # Get canister IDs
    BACKEND_CANISTER_ID=$(dfx canister id backend 2>/dev/null)
    II_CANISTER_ID=$(dfx canister id internet_identity 2>/dev/null)
    
    if [ -n "$BACKEND_CANISTER_ID" ]; then
        echo -e "${BLUE}📋 Backend Canister ID: ${BACKEND_CANISTER_ID}${NC}"
        
        # Update or create .env files (both root and Next.js)
        echo -e "${YELLOW}📝 Updating .env files...${NC}"
        
        # Root .env file
        ENV_FILE=".env"
        touch "$ENV_FILE"
        
        # Next.js .env.local file
        NEXTJS_ENV_FILE="src/nextjs/.env.local"
        mkdir -p "$(dirname "$NEXTJS_ENV_FILE")"
        touch "$NEXTJS_ENV_FILE"
        
        # Helper function to update env file
        update_env_var() {
            local file=$1
            local key=$2
            local value=$3
            
            if grep -q "^${key}=" "$file" 2>/dev/null; then
                # Update existing entry
                if [[ "$OSTYPE" == "darwin"* ]]; then
                    sed -i '' "s|^${key}=.*|${key}=${value}|" "$file"
                else
                    sed -i "s|^${key}=.*|${key}=${value}|" "$file"
                fi
            else
                # Add newline before first entry if file is not empty and doesn't end with newline
                if [ -s "$file" ] && [ "$(tail -c 1 "$file" | wc -l)" -eq 0 ]; then
                    echo "" >> "$file"
                fi
                # Add new entry
                echo "${key}=${value}" >> "$file"
            fi
        }
        
        # Process each env file separately to avoid duplicates
        echo -e "${CYAN}   Updating root .env...${NC}"
        update_env_var "$ENV_FILE" "NEXT_PUBLIC_CANISTER_ID_BACKEND" "$BACKEND_CANISTER_ID"
        update_env_var "$ENV_FILE" "CANISTER_ID_BACKEND" "$BACKEND_CANISTER_ID"
        update_env_var "$ENV_FILE" "DFX_NETWORK" "local"
        update_env_var "$ENV_FILE" "NEXT_PUBLIC_DFX_NETWORK" "local"
        
        echo -e "${CYAN}   Updating src/nextjs/.env.local...${NC}"
        update_env_var "$NEXTJS_ENV_FILE" "NEXT_PUBLIC_CANISTER_ID_BACKEND" "$BACKEND_CANISTER_ID"
        update_env_var "$NEXTJS_ENV_FILE" "CANISTER_ID_BACKEND" "$BACKEND_CANISTER_ID"
        update_env_var "$NEXTJS_ENV_FILE" "DFX_NETWORK" "local"
        update_env_var "$NEXTJS_ENV_FILE" "NEXT_PUBLIC_DFX_NETWORK" "local"
        
        echo -e "${GREEN}   ✓ Updated backend canister environment variables${NC}"
    fi
    
    if [ -n "$II_CANISTER_ID" ]; then
        echo -e "${BLUE}📋 Internet Identity Canister ID: ${II_CANISTER_ID}${NC}"
        
        # Update or add Internet Identity canister ID
        echo -e "${CYAN}   Updating root .env...${NC}"
        update_env_var "$ENV_FILE" "NEXT_PUBLIC_CANISTER_ID_INTERNET_IDENTITY" "$II_CANISTER_ID"
        update_env_var "$ENV_FILE" "CANISTER_ID_INTERNET_IDENTITY" "$II_CANISTER_ID"
        
        echo -e "${CYAN}   Updating src/nextjs/.env.local...${NC}"
        update_env_var "$NEXTJS_ENV_FILE" "NEXT_PUBLIC_CANISTER_ID_INTERNET_IDENTITY" "$II_CANISTER_ID"
        update_env_var "$NEXTJS_ENV_FILE" "CANISTER_ID_INTERNET_IDENTITY" "$II_CANISTER_ID"
        
        echo -e "${GREEN}   ✓ Updated Internet Identity environment variables${NC}"
    fi
    
    # Update /etc/hosts
    if [ -n "$BACKEND_CANISTER_ID" ] || [ -n "$II_CANISTER_ID" ]; then
        echo -e "${YELLOW}🌐 Updating /etc/hosts for local development...${NC}"
        
        HOSTS_ENTRIES=()
        if [ -n "$BACKEND_CANISTER_ID" ]; then
            HOSTS_ENTRIES+=("127.0.0.1 ${BACKEND_CANISTER_ID}.localhost")
        fi
        if [ -n "$II_CANISTER_ID" ]; then
            HOSTS_ENTRIES+=("127.0.0.1 ${II_CANISTER_ID}.localhost")
        fi
        
        NEEDS_UPDATE=false
        for entry in "${HOSTS_ENTRIES[@]}"; do
            canister_hostname=$(echo "$entry" | awk '{print $2}')
            if ! grep -q "$canister_hostname" /etc/hosts 2>/dev/null; then
                NEEDS_UPDATE=true
                break
            fi
        done
        
        if [ "$NEEDS_UPDATE" = true ]; then
            echo -e "${YELLOW}   Adding entries to /etc/hosts (requires sudo)...${NC}"
            for entry in "${HOSTS_ENTRIES[@]}"; do
                canister_hostname=$(echo "$entry" | awk '{print $2}')
                if ! grep -q "$canister_hostname" /etc/hosts 2>/dev/null; then
                    echo "$entry" | sudo tee -a /etc/hosts > /dev/null
                    echo -e "${GREEN}   ✓ Added: $entry${NC}"
                else
                    echo -e "${BLUE}   ℹ Already exists: $entry${NC}"
                fi
            done
            echo -e "${GREEN}✅ /etc/hosts updated${NC}"
        else
            echo -e "${BLUE}   ℹ All entries already exist in /etc/hosts${NC}"
        fi
        
        # Show current canister-related entries
        echo -e "${CYAN}📋 Current canister entries in /etc/hosts:${NC}"
        grep "\.localhost" /etc/hosts 2>/dev/null | grep -E "(${BACKEND_CANISTER_ID}|${II_CANISTER_ID})" || echo -e "${YELLOW}   No matching entries found${NC}"
    fi
    
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
        
        echo -e "${YELLOW}📋 Setting up test environment...${NC}"
        if ./tests/backend/setup-mjs-test-environment.sh; then
            echo -e "${GREEN}✅ Test environment setup completed${NC}"
        else
            echo -e "${RED}❌ Test environment setup failed${NC}"
            exit 1
        fi
    else
        echo -e "${RED}❌ Declaration generation failed${NC}"
        exit 1
    fi
    
    echo ""
    echo -e "${GREEN}🎉 Deployment completed successfully!${NC}"
    echo ""
    echo -e "${CYAN}📋 Next steps:${NC}"
    echo -e "${YELLOW}   1. Restart your Next.js dev server to pick up new environment variables:${NC}"
    echo -e "${CYAN}      cd src/nextjs && npm run dev${NC}"
    echo ""
    echo -e "${YELLOW}   2. Access your application:${NC}"
    echo -e "${CYAN}      Frontend: http://localhost:3000${NC}"
    if [ -n "$BACKEND_CANISTER_ID" ]; then
        echo -e "${CYAN}      Backend: http://${BACKEND_CANISTER_ID}.localhost:4943${NC}"
        echo -e "${CYAN}      Backend (alt): http://127.0.0.1:4943/?canisterId=${BACKEND_CANISTER_ID}${NC}"
    fi
    if [ -n "$II_CANISTER_ID" ]; then
        echo -e "${CYAN}      Internet Identity: http://${II_CANISTER_ID}.localhost:4943${NC}"
    fi
    echo ""
else
    echo -e "${RED}❌ Failed${NC}"
    exit 1
fi