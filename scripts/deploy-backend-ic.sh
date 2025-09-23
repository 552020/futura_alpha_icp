#!/usr/bin/env bash
set -euo pipefail

echo "Deploying backend canister to ICP mainnet..."

# Check if we're on mainnet
if [ "${DFX_NETWORK:-}" != "ic" ]; then
  echo "Setting DFX_NETWORK=ic for mainnet deployment"
  export DFX_NETWORK=ic
fi

echo "Checking deployment prerequisites..."

# Check who is the deployer
echo "Checking deployer identity..."
DEPLOYER=$(dfx identity whoami)
echo "Deployer: $DEPLOYER"

# Confirm with user
read -p "Are you sure you want to deploy as '$DEPLOYER'? (y/N): " -r
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
  echo "Deployment cancelled by user"
  exit 1
fi

# Check cycles balance
echo "Checking cycles balance..."
CYCLES_BALANCE=$(dfx cycles balance --network ic)
echo "Cycles balance: $CYCLES_BALANCE"

# Extract numeric value from cycles balance (remove text)
CYCLES_NUMERIC=$(echo "$CYCLES_BALANCE" | grep -o '[0-9.]*' | head -1)
echo "Numeric cycles: $CYCLES_NUMERIC"

# Check if we have enough cycles (rough estimate: 0.1T cycles for deployment)
if (( $(echo "$CYCLES_NUMERIC < 0.1" | bc -l) )); then
  echo "Warning: Low cycles balance (< 0.1T). Deployment might fail."
  read -p "Continue anyway? (y/N): " -n 1 -r
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Deployment cancelled due to low cycles"
    exit 1
  fi
fi

# Check if canister exists, create if not
echo "Checking if canister exists..."
if ! dfx canister id backend --network ic 2>/dev/null; then
  echo "Canister does not exist. Creating backend canister..."
  dfx canister create backend --network ic
fi

echo "Building backend canister..."
# Check if MIGRATION_ENABLED environment variable is set to false
if [ "${MIGRATION_ENABLED:-true}" = "false" ]; then
    echo "Building backend without migration features..."
    dfx build backend --features=""
else
    echo "Building backend with migration features (default)..."
    dfx build backend
fi

# Generate .did file
echo "Generating .did file..."
generate-did backend

# Final confirmation before deployment
echo "FINAL CONFIRMATION: About to deploy backend canister to ICP mainnet"
echo "   - Deployer: $DEPLOYER"
echo "   - Network: ic (mainnet)"
echo "   - Canister: backend"
read -p "Proceed with deployment? (y/N): " -r
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
  echo "Deployment cancelled by user"
  exit 1
fi

echo "Deploying backend canister to ICP mainnet..."
# Use the same feature flag logic for deployment
if [ "${MIGRATION_ENABLED:-true}" = "false" ]; then
    echo "Deploying backend without migration features..."
    dfx deploy backend --network ic --argument '()' --mode=reinstall
else
    echo "Deploying backend with migration features (default)..."
    dfx deploy backend --network ic
fi

echo "Backend canister deployed successfully to ICP mainnet!"
echo "Canister ID: $(dfx canister id backend --network ic)"
echo "Canister URL: https://$(dfx canister id backend --network ic).ic0.app"
