#!/bin/bash

# Source test utilities (includes DFX color fixes)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/tests/backend/test_utils.sh"

# Simple debug script to check memory data
IDENTITY="default"
CANISTER_ID="backend"

echo "=== Debug Memory Data ==="
echo "Memory ID: mem_1758659689190113000"

# Try to read the memory
echo "Reading memory..."
dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"mem_1758659689190113000\")" > /tmp/memory_debug.txt 2>&1

echo "Memory read result:"
cat /tmp/memory_debug.txt

echo ""
echo "=== Checking if memory exists ==="
dfx canister call --identity $IDENTITY $CANISTER_ID memories_list "(\"default\")" > /tmp/memory_list.txt 2>&1

echo "Memory list result:"
cat /tmp/memory_list.txt
