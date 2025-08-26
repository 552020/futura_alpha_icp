#!/usr/bin/env bash
set -euo pipefail

echo "Updating backend canister on ICP mainnet..."

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
read -p "Are you sure you want to update as '$DEPLOYER'? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
  echo "Update cancelled by user"
  exit 1
fi

# Hardcoded mainnet canister ID
CANISTER_ID="izhgj-eiaaa-aaaaj-a2f7q-cai"
echo "Using canister ID: $CANISTER_ID"

echo "Building backend canister..."
dfx build backend

# Final confirmation before update
echo "FINAL CONFIRMATION: About to update backend canister on ICP mainnet"
echo "   - Deployer: $DEPLOYER"
echo "   - Network: ic (mainnet)"
echo "   - Canister: backend ($CANISTER_ID)"
read -p "Proceed with update? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
  echo "Update cancelled by user"
  exit 1
fi

echo "Updating backend canister on ICP mainnet..."
dfx deploy backend --network ic

echo "Backend canister updated successfully on ICP mainnet!"
echo "Canister ID: $CANISTER_ID"
echo "Canister URL: https://$CANISTER_ID.ic0.app"
